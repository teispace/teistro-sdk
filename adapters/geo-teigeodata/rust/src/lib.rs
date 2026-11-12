//! The teigeodata adapter of the Teistro SDK's geo port
//! (`docs/03-design/geo-port.md` §10, SR-88).
//!
//! [`TeigeodataGeo`] is a [`GeoProvider`] over a teigeodata [`Engine`]: it
//! answers the IANA zone, the ground height and the admin units at a point
//! from the packs (`.tgd` files) the engine holds, and names teigeodata, its
//! version and the packs it answered from as the [`GeoSource`] every
//! resolution carries.
//!
//! ```no_run
//! use teistro_geo_teigeodata::TeigeodataGeo;
//! use teistro_port_geo::{GeoPoint, resolve};
//!
//! let geo = TeigeodataGeo::open(["np.places.2026.11.0.tgd"])?;
//! let place = resolve(&geo, GeoPoint::try_from_degrees(27.7172, 85.324)?)?;
//! assert_eq!(place.zone.as_deref(), Some("Asia/Kathmandu"));
//! # Ok::<(), teistro_core::error::Error>(())
//! ```
//!
//! The adapter depends on the SDK's port and never the reverse: it is a
//! standalone crate outside the SDK workspace (ADR-0019), built against
//! teigeodata's `v0.1.0` tag.

use std::path::Path;
use std::sync::{Mutex, PoisonError};

use teigeodata_core::{E7, Point};
use teigeodata_engine::{Engine, Layer, LayerOptions, Place, Scratch};
use teistro_core::error::{Error, Status};
use teistro_port_geo::{AdminUnit, GeoPoint, GeoProvider, GeoSource, GroundHeight};

/// The name every resolution through this adapter carries as its
/// source's `name`.
pub const NAME: &str = "teigeodata";

/// The version of teigeodata's engine this adapter is built against, the
/// tag its `Cargo.toml` names; a test holds it to the version the lock
/// resolved.
pub const ENGINE_VERSION: &str = "0.1.0";

/// The longest name, in bytes, the adapter reads for a zone or an admin
/// unit. A zone name is at most a few dozen bytes and a unit's canonical
/// name a few hundred; a longer one is refused as the provider's failure.
const NAME_BYTES: usize = 1024;

/// A teigeodata engine answering the geo port's three questions.
///
/// Each question is asked of the engine as a whole, so the layer that
/// answers is the one teigeodata's precedence picks: a country pack over
/// the baseline, a newer release over an older one.
///
/// - the zone is the name of the timezone area holding the point
///   (`Engine::tz_at`), which is its tzdb name;
/// - the height is the terrain height from the first pack whose DEM has
///   samples there (`Engine::elevation_at`), in metres above EGM2008;
/// - the admin units are the units holding the point, coarse to fine
///   (`Engine::admin_at`), each with its ISO 3166 code where the pack
///   states one (`Engine::iso_code`) and its canonical name.
///
/// A point no pack answers for answers `None` or an empty list, never a
/// refusal. A refusal is a pack that failed: a section that does not match
/// its hash, or a component the query needs that was not installed.
///
/// ```
/// use teistro_geo_teigeodata::TeigeodataGeo;
///
/// // An engine with no pack answers nothing, so it is refused.
/// let refused = TeigeodataGeo::open(Vec::<&str>::new()).unwrap_err();
/// assert_eq!(refused.field(), Some("packs"));
/// ```
#[derive(Debug)]
pub struct TeigeodataGeo {
    engine: Engine,
    source: GeoSource,
    /// The engine's working memory, reused from one query to the next so
    /// that a warm query does not allocate; behind a lock because the port
    /// asks through a shared reference from any thread.
    scratch: Mutex<Scratch>,
}

impl TeigeodataGeo {
    /// A provider over the pack files at `packs`, each mapped with the
    /// engine's default options: verified lazily, precedence by release
    /// and address.
    ///
    /// ```no_run
    /// use teistro_geo_teigeodata::TeigeodataGeo;
    ///
    /// // A country pack, and a terrain pack beside it.
    /// let geo = TeigeodataGeo::open(["np.places.tgd", "np.dem.tgd"])?;
    /// assert_eq!(geo.engine().layers().len(), 2);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `PACK` naming `packs[i]` for a file that cannot be read or is not a
    /// pack; `INVALID_ARG` on `packs` when there is none.
    pub fn open<P: AsRef<Path>>(
        packs: impl IntoIterator<Item = P>,
    ) -> Result<TeigeodataGeo, Error> {
        let mut engine = Engine::new();
        for (at, path) in packs.into_iter().enumerate() {
            let path = path.as_ref();
            let layer = Layer::open(path, LayerOptions::default())
                .map_err(|why| unopened(&why, at, &path.display().to_string()))?;
            engine.add(layer);
        }
        TeigeodataGeo::from_engine(engine)
    }

    /// A provider over packs held in memory, as a platform without file
    /// mapping (the web) or an application that embeds its packs has them.
    ///
    /// ```
    /// use teistro_geo_teigeodata::TeigeodataGeo;
    ///
    /// let refused = TeigeodataGeo::from_bytes([b"not a pack".to_vec()]).unwrap_err();
    /// assert_eq!(refused.field(), Some("packs[0]"));
    /// ```
    ///
    /// # Errors
    ///
    /// As [`TeigeodataGeo::open`].
    pub fn from_bytes<B: Into<Box<[u8]>>>(
        packs: impl IntoIterator<Item = B>,
    ) -> Result<TeigeodataGeo, Error> {
        let mut engine = Engine::new();
        for (at, bytes) in packs.into_iter().enumerate() {
            let layer = Layer::from_bytes(bytes, LayerOptions::default())
                .map_err(|why| unopened(&why, at, "the bytes"))?;
            engine.add(layer);
        }
        TeigeodataGeo::from_engine(engine)
    }

    /// A provider over an engine the caller assembled: its embedded
    /// baseline, its installed packs with their priorities and installed
    /// components, or a snapshot of teigeodata's `Shared`.
    ///
    /// ```
    /// use teigeodata_engine::Engine;
    /// use teistro_geo_teigeodata::TeigeodataGeo;
    ///
    /// let refused = TeigeodataGeo::from_engine(Engine::new()).unwrap_err();
    /// assert_eq!(refused.field(), Some("packs"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on `packs` for an engine with no pack, which has no
    /// data to answer from or to name.
    pub fn from_engine(engine: Engine) -> Result<TeigeodataGeo, Error> {
        let data_version = engine
            .layers()
            .map(pack_name)
            .collect::<Vec<_>>()
            .join(" + ");
        if data_version.is_empty() {
            return Err(
                Error::invalid_arg("a teigeodata provider needs at least one pack")
                    .with_field("packs")
                    .with_hint("open a pack file, such as a country's `np.places` pack"),
            );
        }
        let source = GeoSource {
            name: String::from(NAME),
            version: String::from(ENGINE_VERSION),
            data_version,
            tzdb: engine.tz_version().map(str::to_owned),
        };
        Ok(TeigeodataGeo {
            engine,
            source,
            scratch: Mutex::new(Scratch::default()),
        })
    }

    /// The engine the provider asks, for the questions the port does not
    /// put: a place's name, a search, the zone rules.
    #[must_use]
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    /// The canonical name of `place`, or `None` where its pack has none.
    fn name_of(&self, place: &Place, field: &str) -> Result<Option<String>, Error> {
        let mut buf = [0_u8; NAME_BYTES];
        self.engine
            .name(place, &mut buf)
            .map(|name| name.map(str::to_owned))
            .map_err(|why| failed(&why, field))
    }

    /// The engine's working memory, taken for one query. A query that
    /// panicked leaves nothing a later one relies on (each clears what it
    /// reads), so a poisoned lock is taken over.
    fn scratch(&self) -> std::sync::MutexGuard<'_, Scratch> {
        self.scratch.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// A pack as the source names it: its address, component and release,
/// such as `np.places.2026.11.0`.
fn pack_name(layer: &Layer) -> String {
    format!(
        "{}.{}.{}",
        layer.address(),
        layer.component(),
        layer.release()
    )
}

/// The engine's point for `at`, on its 1e-7° grid.
fn grid(at: GeoPoint, field: &str) -> Result<Point<E7>, Error> {
    Point::from_degrees(at.latitude.get(), at.longitude.get()).map_err(|why| {
        Error::new(
            Status::Provider,
            format!("teigeodata does not place the point: {why}"),
        )
        .with_field(field)
    })
}

/// A pack that did not open, as the port names it: `PACK` on `packs[at]`.
fn unopened(why: &teigeodata_engine::Error, at: usize, what: &str) -> Error {
    Error::new(
        Status::Pack,
        format!("teigeodata could not open {what} as a pack: {why}"),
    )
    .with_field(format!("packs[{at}]"))
}

/// A query the engine refused, as the port names a provider's failure:
/// `PROVIDER` on the question's field (`zone`, `height`, `admin`).
fn failed(why: &teigeodata_engine::Error, field: &str) -> Error {
    let error = Error::new(Status::Provider, format!("teigeodata: {why}")).with_field(field);
    match why {
        teigeodata_engine::Error::MissingComponent { component, .. } => error.with_hint(format!(
            "install the pack's `{}` component, or open the whole pack",
            component.name()
        )),
        teigeodata_engine::Error::Integrity { .. } => {
            error.with_hint("the pack is corrupt or was changed after it was installed")
        }
        _ => error,
    }
}

impl GeoProvider for TeigeodataGeo {
    fn source(&self) -> &GeoSource {
        &self.source
    }

    fn zone_at(&self, at: GeoPoint) -> Result<Option<String>, Error> {
        let point = grid(at, "zone")?;
        let Some(area) = self
            .engine
            .tz_at(point)
            .map_err(|why| failed(&why, "zone"))?
        else {
            return Ok(None);
        };
        self.name_of(&area, "zone")
    }

    fn height_at(&self, at: GeoPoint) -> Result<Option<GroundHeight>, Error> {
        let point = grid(at, "height")?;
        let height = self
            .engine
            .elevation_at(point, &mut self.scratch())
            .map_err(|why| failed(&why, "height"))?;
        Ok(height.map(|height| GroundHeight {
            metres: height.meters,
            source: match height.source {
                Some(product) => String::from(product.name()),
                None => match self.engine.layer(height.layer) {
                    Some(layer) => format!("the DEM of {}", pack_name(layer)),
                    None => String::from("a teigeodata DEM"),
                },
            },
            resolution_m: height.resolution_m,
            datum: String::from(height.datum.name()),
        }))
    }

    fn admin_at(&self, at: GeoPoint) -> Result<Vec<AdminUnit>, Error> {
        let point = grid(at, "admin")?;
        let mut places = Vec::new();
        self.engine
            .admin_at(point, &mut self.scratch(), &mut places)
            .map_err(|why| failed(&why, "admin"))?;
        places
            .iter()
            .map(|place| {
                let code = self
                    .engine
                    .iso_code(place)
                    .map_err(|why| failed(&why, "admin"))?;
                Ok(AdminUnit {
                    level: place.admin_level,
                    code: code.map(|code| code.as_str().to_owned()),
                    name: self.name_of(place, "admin")?,
                })
            })
            .collect()
    }
}
