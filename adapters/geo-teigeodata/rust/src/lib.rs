//! The teigeodata adapter of the Teistro SDK's geo port
//! (`docs/03-design/geo-port.md` §10, §12 and §13; SR-88, SR-141, SR-142,
//! SR-129).
//!
//! [`TeigeodataGeo`] is a [`GeoProvider`] over a teigeodata engine: it
//! answers the IANA zone, the ground height and the admin units at a point,
//! and the place catalogue's questions (a name to places across scripts, a
//! point to its nearest places and units, a place's names by language and
//! script, the places of a unit ranked by population), from the packs
//! (`.tgd` files) the engine holds, and names teigeodata, its version and
//! the packs it answered from as the [`GeoSource`] every resolution carries.
//!
//! It holds a teigeodata [`Shared`], the engine teigeodata's store fills
//! and refills (`Shared::follow`): packs added or replaced after a context
//! is built answer the next question, with no rebuild.
//!
//! ```no_run
//! use teistro_geo_teigeodata::TeigeodataGeo;
//! use teistro_port_geo::{GeoPoint, resolve};
//!
//! let geo = TeigeodataGeo::open(["np-places-main.tgd", "baseline-main.tgd"])?;
//! let place = resolve(&geo, GeoPoint::try_from_degrees(27.7172, 85.324)?)?;
//! assert_eq!(place.zone.as_deref(), Some("Asia/Kathmandu"));
//! # Ok::<(), teistro_core::error::Error>(())
//! ```
//!
//! The adapter depends on the SDK's port and never the reverse: it is a
//! standalone crate outside the SDK workspace (ADR-0019), built against
//! teigeodata's `v1.1.0` tag.

#[cfg(feature = "mmap")]
use std::path::Path;
use std::sync::{Arc, Mutex, PoisonError};

use teigeodata_core::{E7, Point};
use teigeodata_engine::{Engine, Layer, LayerOptions, Place, Scratch, Shared};
use teistro_core::error::{Error, Status};
use teistro_port_geo::{
    AdminUnit, FoundPlace, GeoPoint, GeoProvider, GeoSource, GroundHeight, NamedPlace,
    NearbyPlaces, PlaceListing, PlaceNames, PlaceNamesRequest, PlaceReverse, PlaceSearch,
};
use teistro_port_timezone::TimeZoneProvider;

mod catalogue;
#[cfg(feature = "zone-rules")]
mod zones;
#[cfg(feature = "zone-rules")]
pub use zones::TeigeodataZones;

/// The name every resolution through this adapter carries as its
/// source's `name`.
pub const NAME: &str = "teigeodata";

/// The version of teigeodata's engine this adapter is built against, the
/// tag its `Cargo.toml` names; a test holds it to the version the lock
/// resolved.
pub const ENGINE_VERSION: &str = "1.1.0";

/// The source's `dataVersion` while the engine holds no pack: a provider
/// over a store that has installed nothing yet (§13.2).
pub const NO_PACKS: &str = "none";

/// The address of teigeodata's world layers, the baseline and the app
/// pack: the only layers that name places the world over.
pub const WORLD: &str = "world";

/// The longest name, in bytes, the adapter reads for a zone or a place. A
/// zone name is at most a few dozen bytes and a place's name a few
/// hundred; a longer one is refused as the provider's failure.
const NAME_BYTES: usize = 1024;

/// A teigeodata engine answering the geo port.
///
/// Each question is asked of one snapshot of the engine as a whole
/// ([`Shared::load`]), so the layer that answers is the one teigeodata's
/// precedence picks (a country pack over the baseline, a newer release over
/// an older one), and a pack the store installs between two questions
/// answers the second.
///
/// - the zone is the name of the timezone area holding the point
///   (`Engine::tz_at`), which is its tzdb name;
/// - the height is the terrain height from the first pack whose DEM has
///   samples there (`Engine::elevation_at`, feature `terrain`), in metres
///   above EGM2008;
/// - the admin units are the units holding the point, coarse to fine
///   (`Engine::admin_at`), each with its ISO 3166 code where the pack
///   states one (`Engine::iso_code`) and its canonical name;
/// - a search, a reverse, a place's names and a listing answer from the
///   engine's own search, nearest places, names and hierarchy (§12).
///
/// A point no pack answers for answers `None` or an empty list, never a
/// refusal, and a level a pack holds no outline for is absent. A refusal is
/// a pack that failed: a section that does not match its hash, or a
/// component a question needs that was not installed.
///
/// ```
/// use teistro_geo_teigeodata::TeigeodataGeo;
///
/// // An engine with no pack answers nothing, so it is refused.
/// let refused = TeigeodataGeo::from_bytes(Vec::<Vec<u8>>::new()).unwrap_err();
/// assert_eq!(refused.field(), Some("packs"));
/// ```
#[derive(Debug)]
pub struct TeigeodataGeo {
    /// The engine, which the application's store may refill.
    shared: Arc<Shared>,
    /// The engine's working memory, reused from one query to the next so
    /// that a warm query does not allocate; behind a lock because the port
    /// asks through a shared reference from any thread.
    scratch: Mutex<Scratch>,
}

impl TeigeodataGeo {
    /// A provider over the pack files at `packs`, each mapped with the
    /// engine's default options: verified lazily, precedence by release
    /// and address. Native targets only (feature `mmap`).
    ///
    /// ```no_run
    /// use teistro_geo_teigeodata::TeigeodataGeo;
    ///
    /// // A country pack, and the world's baseline beside it.
    /// let geo = TeigeodataGeo::open(["np-places-main.tgd", "baseline-main.tgd"])?;
    /// assert_eq!(geo.snapshot().layers().len(), 2);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `PACK` naming `packs[i]` for a file that cannot be read or is not a
    /// pack; `INVALID_ARG` on `packs` when there is none.
    #[cfg(feature = "mmap")]
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
    /// `PACK` naming `packs[i]` for bytes that are not a pack;
    /// `INVALID_ARG` on `packs` when there is none.
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
    /// components, or a snapshot of a teigeodata `Shared`. The provider
    /// keeps it in a `Shared` of its own ([`TeigeodataGeo::shared`]), so
    /// packs can still be added later.
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
        if engine.layers().len() == 0 {
            return Err(
                Error::invalid_arg("a teigeodata provider needs at least one pack")
                    .with_field("packs")
                    .with_hint(
                        "open a pack file, such as a country's `np-places` pack, or hand over \
                         the store's `Shared` with `from_shared`",
                    ),
            );
        }
        let geo = TeigeodataGeo::from_shared(Arc::new(Shared::new(engine)));
        // Rules a pack carries but cannot list are refused here, where the
        // caller can still act on it; `time_zones` later takes them as none.
        #[cfg(feature = "zone-rules")]
        geo.zones()?;
        Ok(geo)
    }

    /// A provider over the application's own teigeodata `Shared`, the
    /// engine its store fills: after `shared.follow(...)` or
    /// `shared.update(...)` installs, updates or repairs packs, the next
    /// question reads the new set, and the context holding the provider is
    /// not rebuilt (§13.2). An empty `Shared` is taken: until a pack
    /// arrives every question answers nothing, and the source's
    /// `dataVersion` is [`NO_PACKS`].
    ///
    /// ```
    /// use std::sync::Arc;
    /// use teigeodata_engine::Shared;
    /// use teistro_geo_teigeodata::{NO_PACKS, TeigeodataGeo};
    /// use teistro_port_geo::{GeoPoint, GeoProvider};
    ///
    /// let store = Arc::new(Shared::default());
    /// let geo = TeigeodataGeo::from_shared(Arc::clone(&store));
    /// assert_eq!(geo.source().data_version, NO_PACKS);
    /// assert_eq!(geo.zone_at(GeoPoint::try_from_degrees(27.7, 85.3)?)?, None);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn from_shared(shared: Arc<Shared>) -> TeigeodataGeo {
        TeigeodataGeo {
            shared,
            scratch: Mutex::new(Scratch::default()),
        }
    }

    /// The `Shared` the provider asks, for adding or replacing packs:
    /// `geo.shared().update(|engine| { engine.add(layer); })`.
    #[must_use]
    pub fn shared(&self) -> &Arc<Shared> {
        &self.shared
    }

    /// The engine as it is now, for the questions the port does not put;
    /// a later install does not change a snapshot already taken.
    #[must_use]
    pub fn snapshot(&self) -> Arc<Engine> {
        self.shared.current()
    }

    /// The packs' zone rules as the snapshot now current holds them, or
    /// `None` where no pack has any; what [`GeoProvider::time_zones`] hands
    /// a context, with the refusal it cannot carry.
    ///
    /// # Errors
    ///
    /// `PROVIDER` on `zones` when the packs' rules cannot be listed, such
    /// as rules a pack carries whose component was not installed.
    #[cfg(feature = "zone-rules")]
    pub fn zones(&self) -> Result<Option<TeigeodataZones>, Error> {
        TeigeodataZones::new(self.shared.current())
    }

    /// The engine's working memory, taken for one query. A query that
    /// panicked leaves nothing a later one relies on (each clears what it
    /// reads), so a poisoned lock is taken over.
    fn scratch(&self) -> std::sync::MutexGuard<'_, Scratch> {
        self.scratch.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// The canonical name of `place`, or `None` where its pack has none.
fn name_of(engine: &Engine, place: &Place, field: &str) -> Result<Option<String>, Error> {
    let mut buf = [0_u8; NAME_BYTES];
    engine
        .name(place, &mut buf)
        .map(|name| name.map(str::to_owned))
        .map_err(|why| failed(&why, field))
}

/// A pack as the source names it: its address, component and release,
/// such as `np.places.2026.11.15`.
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
pub(crate) fn failed(why: &teigeodata_engine::Error, field: &str) -> Error {
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
    /// teigeodata, its version, and the packs of the snapshot now current,
    /// highest precedence first, joined by ` + `.
    fn source(&self) -> GeoSource {
        let engine = self.shared.load();
        let data_version = engine
            .layers()
            .map(pack_name)
            .collect::<Vec<_>>()
            .join(" + ");
        GeoSource {
            name: String::from(NAME),
            version: String::from(ENGINE_VERSION),
            data_version: if data_version.is_empty() {
                String::from(NO_PACKS)
            } else {
                data_version
            },
            tzdb: tz_version(&engine),
        }
    }

    fn zone_at(&self, at: GeoPoint) -> Result<Option<String>, Error> {
        let point = grid(at, "zone")?;
        let engine = self.shared.load();
        let area = match engine.tz_at(point).map_err(|why| failed(&why, "zone"))? {
            Some(area) => area,
            // No area outline holds the point (teigeodata's app pack
            // outlines Nepal alone): the zone of a settlement or locality
            // the point lies within, by teigeodata's radii (C2218).
            None => match catalogue::zone_within(&engine, &mut self.scratch(), at)? {
                Some(area) => area,
                None => return Ok(None),
            },
        };
        name_of(&engine, &area, "zone")
    }

    #[cfg(feature = "terrain")]
    fn height_at(&self, at: GeoPoint) -> Result<Option<GroundHeight>, Error> {
        let point = grid(at, "height")?;
        let engine = self.shared.load();
        let height = engine
            .elevation_at(point, &mut self.scratch())
            .map_err(|why| failed(&why, "height"))?;
        Ok(height.map(|height| GroundHeight {
            metres: height.meters,
            source: match height.source {
                Some(product) => String::from(product.name()),
                None => match engine.layer(height.layer) {
                    Some(layer) => format!("the DEM of {}", pack_name(layer)),
                    None => String::from("a teigeodata DEM"),
                },
            },
            resolution_m: height.resolution_m,
            datum: String::from(height.datum.name()),
        }))
    }

    /// No height: the adapter was built without its `terrain` feature.
    #[cfg(not(feature = "terrain"))]
    fn height_at(&self, at: GeoPoint) -> Result<Option<GroundHeight>, Error> {
        grid(at, "height")?;
        Ok(None)
    }

    /// The packs' own zone rules, as the snapshot now current holds them,
    /// so a context reckons a place this provider resolves by the tzdb
    /// release its packs carry:
    /// `Context::builder().geo(geo.clone()).time_zones(zones)`. A context
    /// takes them once, when it is built (C2204).
    fn time_zones(&self) -> Option<Arc<dyn TimeZoneProvider>> {
        #[cfg(feature = "zone-rules")]
        {
            // Rules that cannot be listed are no rules here, as the port's
            // `Option` cannot say why: `from_engine` refused them, and
            // `zones()` gives the refusal to a caller of `from_shared`.
            self.zones()
                .ok()
                .flatten()
                .map(|zones| Arc::new(zones) as Arc<dyn TimeZoneProvider>)
        }
        #[cfg(not(feature = "zone-rules"))]
        {
            None
        }
    }

    fn admin_at(&self, at: GeoPoint) -> Result<Vec<AdminUnit>, Error> {
        let point = grid(at, "admin")?;
        let engine = self.shared.load();
        let mut places = Vec::new();
        engine
            .admin_at(point, &mut self.scratch(), &mut places)
            .map_err(|why| failed(&why, "admin"))?;
        places
            .iter()
            .map(|place| {
                let code = engine
                    .iso_code(place)
                    .map_err(|why| failed(&why, "admin"))?;
                Ok(AdminUnit {
                    level: place.admin_level,
                    code: code.map(|code| code.as_str().to_owned()),
                    name: name_of(&engine, place, "admin")?,
                })
            })
            .collect()
    }

    /// At sea (SR-144): a world layer (address [`WORLD`]) is installed, no
    /// zone and no admin unit holds the point, and the data does outline
    /// the land nearest it: the nearest settlement (or locality, or unit)
    /// lies inside a unit the packs hold. A country pack knows no world;
    /// teigeodata's app pack is a world layer that outlines Nepal alone,
    /// so abroad its nearest place lies in no unit either. Both cannot
    /// tell the sea from land they lack, and say no (C2216, C2219).
    fn at_sea(&self, at: GeoPoint) -> Result<bool, Error> {
        let point = grid(at, "zone")?;
        let engine = self.shared.load();
        if !engine.layers().any(|layer| layer.address() == WORLD) {
            return Ok(false);
        }
        if engine
            .tz_at(point)
            .map_err(|why| failed(&why, "zone"))?
            .is_some()
        {
            return Ok(false);
        }
        let mut scratch = self.scratch();
        let mut units = Vec::new();
        if !catalogue::units_at(&engine, &mut scratch, point, &mut units)? || !units.is_empty() {
            return Ok(false);
        }
        let Some(nearest) = catalogue::nearest_named(&engine, &mut scratch, at)? else {
            return Ok(false);
        };
        Ok(
            catalogue::units_at(&engine, &mut scratch, nearest.to_e7(), &mut units)?
                && !units.is_empty(),
        )
    }

    fn search(&self, asked: &PlaceSearch) -> Result<Vec<FoundPlace>, Error> {
        catalogue::search(&self.shared.load(), &mut self.scratch(), asked)
    }

    fn reverse(&self, asked: &PlaceReverse) -> Result<NearbyPlaces, Error> {
        catalogue::reverse(&self.shared.load(), &mut self.scratch(), asked)
    }

    fn names(&self, asked: &PlaceNamesRequest) -> Result<PlaceNames, Error> {
        catalogue::names(&self.shared.load(), asked)
    }

    fn list(&self, asked: &PlaceListing) -> Result<Vec<NamedPlace>, Error> {
        catalogue::list(&self.shared.load(), asked)
    }
}

/// The tzdb release the engine's zone rules state, `None` without rules
/// or without the `zone-rules` feature.
fn tz_version(engine: &Engine) -> Option<String> {
    #[cfg(feature = "zone-rules")]
    {
        engine.tz_version().map(str::to_owned)
    }
    #[cfg(not(feature = "zone-rules"))]
    {
        let _ = engine;
        None
    }
}
