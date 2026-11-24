//! The geo port: what the SDK asks of a geographic dataset and nothing
//! else (`docs/03-design/geo-port.md`, SR-87).
//!
//! A provider answers three questions at a point: the IANA zone holding it,
//! the ground height there with the height's source, resolution and datum,
//! and the admin units holding it with their ISO 3166 codes. It names itself
//! and its data's release, which every resolution carries, and it may hand
//! over the zone rules of the tzdb release its zone names come from.
//!
//! The SDK never consults a provider on its own: a caller resolves a place
//! ([`resolve`]) and sends the [`ResolvedPlace`] in the request that reads
//! it, so a stored answer replays without the provider.
//!
//! ```
//! use teistro_port_geo::{GeoPoint, GeoTable, resolve};
//!
//! let table = GeoTable::from_json(r#"{
//!     "source": {"name": "places", "version": "1", "dataVersion": "2026.11.0", "tzdb": null},
//!     "areas": [{"south": 27.6, "west": 85.2, "north": 27.8, "east": 85.5,
//!                "zone": "Asia/Kathmandu", "height": null, "admin": []}]
//! }"#)?;
//! let place = resolve(&table, GeoPoint::try_from_degrees(27.7172, 85.324)?)?;
//! assert_eq!(place.zone.as_deref(), Some("Asia/Kathmandu"));
//! assert_eq!(place.source.data_version, "2026.11.0");
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use std::sync::Arc;

use teistro_core::error::{Error, Status};
use teistro_core::quantity::{Altitude, Latitude, Longitude};
use teistro_port_timezone::TimeZoneProvider;

mod catalogue;
mod table;

pub use catalogue::{
    ChosenName, FOCUS_SCALE_M, FoundPlace, LangTag, LookedUpPlace, MOST_FOUND, MOST_LISTED,
    NamedPlace, NearPlace, NearbyPlaces, ParentPlace, PlaceKind, PlaceListing, PlaceLookup,
    PlaceName, PlaceNames, PlaceNamesRequest, PlaceReverse, PlaceSearch, Population,
    PopulationKind, SearchFocus, label, list, names, place, reverse, search,
};
pub use table::{TableName, TablePlace};

/// A point on the Earth a provider is asked about.
///
/// ```
/// use teistro_port_geo::GeoPoint;
///
/// let kathmandu = GeoPoint::try_from_degrees(27.7172, 85.324)?;
/// assert_eq!(kathmandu.latitude.get(), 27.7172);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeoPoint {
    /// North positive.
    pub latitude: Latitude,
    /// East positive.
    pub longitude: Longitude,
}

impl GeoPoint {
    /// A point from checked parts.
    ///
    /// ```
    /// use teistro_core::quantity::{Latitude, Longitude};
    /// use teistro_port_geo::GeoPoint;
    ///
    /// let point = GeoPoint::new(Latitude::literal(27.7172), Longitude::literal(85.324));
    /// assert_eq!(point.longitude.get(), 85.324);
    /// ```
    #[must_use]
    pub const fn new(latitude: Latitude, longitude: Longitude) -> GeoPoint {
        GeoPoint {
            latitude,
            longitude,
        }
    }

    /// A point from bare degrees, refused as `latitudeDeg` or
    /// `longitudeDeg`, the names a request writes them under.
    ///
    /// ```
    /// use teistro_port_geo::GeoPoint;
    ///
    /// let refused = GeoPoint::try_from_degrees(27.7, 185.0).unwrap_err();
    /// assert_eq!(refused.field(), Some("longitudeDeg"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on a latitude or longitude out of range.
    pub fn try_from_degrees(latitude_deg: f64, longitude_deg: f64) -> Result<GeoPoint, Error> {
        Ok(GeoPoint {
            latitude: Latitude::try_new(latitude_deg)
                .map_err(|why| Error::from(why).with_field("latitudeDeg"))?,
            longitude: Longitude::try_new(longitude_deg)
                .map_err(|why| Error::from(why).with_field("longitudeDeg"))?,
        })
    }
}

/// Who answers a resolution, and from which data.
///
/// ```
/// use teistro_port_geo::GeoSource;
///
/// let source = GeoSource {
///     name: "places".into(),
///     version: "1".into(),
///     data_version: "2026.11.0".into(),
///     tzdb: Some("2026c".into()),
/// };
/// assert_eq!(serde_json::to_value(&source)?["dataVersion"], "2026.11.0");
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GeoSource {
    /// The provider's name.
    pub name: String,
    /// The provider's own version, its code's.
    pub version: String,
    /// The release of the data it answered from (`2026.11.0`).
    pub data_version: String,
    /// The tzdb release its zone names come from (`2026c`), or `null`
    /// where it does not say.
    pub tzdb: Option<String>,
}

/// The ground height at a point, and where it comes from.
///
/// ```
/// use teistro_port_geo::GroundHeight;
///
/// let height = GroundHeight {
///     metres: 1337.0,
///     source: "survey".into(),
///     resolution_m: 30.0,
///     datum: "EGM2008".into(),
/// };
/// assert_eq!(serde_json::to_value(&height)?["resolutionM"], 30.0);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GroundHeight {
    /// The height in metres above the datum.
    pub metres: f64,
    /// The dataset it was read from, as the provider names it (`SRTM GL1`).
    pub source: String,
    /// The data's horizontal resolution, the step between its samples, in
    /// metres.
    pub resolution_m: f64,
    /// The vertical datum the height is above: `EGM2008`, `EGM96`, or
    /// `WGS84` for the ellipsoid.
    pub datum: String,
}

/// An admin unit holding a point.
///
/// ```
/// use teistro_port_geo::AdminUnit;
///
/// let bagmati = AdminUnit { level: 1, code: Some("NP-P3".into()), name: Some("Bagmati".into()) };
/// assert_eq!(serde_json::to_value(&bagmati)?["code"], "NP-P3");
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdminUnit {
    /// Its level: 0 for a country, then coarse to fine.
    pub level: u8,
    /// Its ISO 3166-1 alpha-2 code at level 0 (`NP`), its ISO 3166-2 code
    /// below (`NP-P3`), or `null` for a unit with none.
    pub code: Option<String>,
    /// Its name as the provider gives it, or `null`.
    pub name: Option<String>,
}

/// A place as a provider resolved it: the point asked, the zone, the
/// ground height and the admin units there, and who answered.
///
/// What `geo.resolve` answers and a chart request's `place` takes, so the
/// request a receipt keeps holds what the provider answered.
///
/// ```
/// use teistro_port_geo::{GeoPoint, GeoTable, resolve};
///
/// let table = GeoTable::from_json(r#"{"source": {"name": "t", "version": "1", "dataVersion": "1", "tzdb": null}, "areas": []}"#)?;
/// let nowhere = resolve(&table, GeoPoint::try_from_degrees(0.0, 0.0)?)?;
/// // No answer is an answer: no zone, no height, no unit.
/// assert!(nowhere.zone.is_none() && nowhere.height.is_none() && nowhere.admin.is_empty());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResolvedPlace {
    /// The latitude asked, in degrees.
    pub latitude_deg: f64,
    /// The longitude asked, in degrees.
    pub longitude_deg: f64,
    /// The IANA zone holding the point, or `null` where the data has none.
    /// At sea, where the provider knows no area holds the point, the
    /// nautical zone [`nautical_zone`] reckons from the longitude, with
    /// [`ResolvedPlace::nautical`] set (SR-144).
    pub zone: Option<String>,
    /// The zone is the nautical one (`Etc/GMT±n` from the longitude), not
    /// a land zone; written only when true, so a land answer reads as it
    /// always has.
    #[serde(default, skip_serializing_if = "core::ops::Not::not")]
    pub nautical: bool,
    /// The ground height there, or `null` where the data has none.
    pub height: Option<GroundHeight>,
    /// The admin units holding the point, coarse to fine.
    pub admin: Vec<AdminUnit>,
    /// Who answered, and from which data.
    pub source: GeoSource,
}

impl ResolvedPlace {
    /// The place checked as a resolution is: the point in range, and every
    /// answer well formed, refused with `status` (`PROVIDER` for a
    /// provider's answer, `INVALID_ARG` for one a caller wrote).
    ///
    /// ```
    /// use teistro_core::error::Status;
    /// use teistro_port_geo::{GeoSource, ResolvedPlace};
    ///
    /// let mut place = ResolvedPlace {
    ///     latitude_deg: 27.7, longitude_deg: 85.3, zone: Some(String::new()), nautical: false, height: None, admin: vec![],
    ///     source: GeoSource { name: "t".into(), version: "1".into(), data_version: "1".into(), tzdb: None },
    /// };
    /// assert_eq!(place.checked(Status::InvalidArg).unwrap_err().field(), Some("zone"));
    /// place.zone = None;
    /// assert!(place.checked(Status::InvalidArg).is_ok());
    /// ```
    ///
    /// # Errors
    ///
    /// The first part that is not well formed, by its field.
    pub fn checked(&self, status: Status) -> Result<GeoPoint, Error> {
        let point = GeoPoint::try_from_degrees(self.latitude_deg, self.longitude_deg)?;
        let answers = Answers {
            zone: self.zone.as_deref(),
            height: self.height.as_ref(),
            admin: &self.admin,
        };
        answers.check("").map_err(|why| restated(why, status))?;
        if self.nautical && self.zone.as_deref() != Some(nautical_zone(point).as_str()) {
            return Err(restated(
                refused(
                    "",
                    "nautical",
                    format!(
                        "a nautical place's zone is {} at this longitude",
                        nautical_zone(point)
                    ),
                ),
                status,
            ));
        }
        self.source
            .check("source")
            .map_err(|why| restated(why, status))?;
        Ok(point)
    }
}

/// A geographic dataset.
///
/// ```
/// use teistro_port_geo::{GeoPoint, GeoProvider, GeoTable};
///
/// let table = GeoTable::from_json(r#"{"source": {"name": "t", "version": "1", "dataVersion": "7", "tzdb": null}, "areas": []}"#)?;
/// // A reference to a provider is a provider.
/// fn release(geo: impl GeoProvider) -> String {
///     geo.source().data_version.clone()
/// }
/// assert_eq!(release(&table), "7");
/// assert!(table.time_zones().is_none());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
pub trait GeoProvider: Send + Sync {
    /// Who answers, and from which data: stamped on every resolution.
    ///
    /// ```
    /// use teistro_port_geo::{GeoProvider, GeoTable};
    ///
    /// let table = GeoTable::from_json(r#"{"source": {"name": "t", "version": "1", "dataVersion": "7", "tzdb": "2026c"}, "areas": []}"#)?;
    /// assert_eq!(table.source().tzdb.as_deref(), Some("2026c"));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    fn source(&self) -> GeoSource;

    /// The IANA zone holding the point; `None` where the data has none.
    ///
    /// ```
    /// use teistro_port_geo::{GeoPoint, GeoProvider, GeoTable};
    ///
    /// let table = GeoTable::from_json(r#"{"source": {"name": "t", "version": "1", "dataVersion": "1", "tzdb": null},
    ///     "areas": [{"south": 26.3, "west": 80.0, "north": 30.5, "east": 88.2, "zone": "Asia/Kathmandu", "height": null, "admin": []}]}"#)?;
    /// assert_eq!(table.zone_at(GeoPoint::try_from_degrees(28.2, 83.98)?)?.as_deref(), Some("Asia/Kathmandu"));
    /// assert_eq!(table.zone_at(GeoPoint::try_from_degrees(51.5, 0.0)?)?, None);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// The provider failed: data missing that should be there, or corrupt.
    fn zone_at(&self, at: GeoPoint) -> Result<Option<String>, Error>;

    /// The ground height at the point; `None` where the data has none (the
    /// sea, a point outside every dataset).
    ///
    /// ```
    /// use teistro_port_geo::{GeoPoint, GeoProvider, GeoTable};
    ///
    /// let table = GeoTable::from_json(r#"{"source": {"name": "t", "version": "1", "dataVersion": "1", "tzdb": null},
    ///     "areas": [{"south": 27.6, "west": 85.2, "north": 27.8, "east": 85.5, "zone": null,
    ///                "height": {"metres": 1337, "source": "survey", "resolutionM": 30, "datum": "EGM2008"}, "admin": []}]}"#)?;
    /// let height = table.height_at(GeoPoint::try_from_degrees(27.7, 85.3)?)?.ok_or("a height")?;
    /// assert_eq!((height.metres, height.datum.as_str()), (1337.0, "EGM2008"));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`GeoProvider::zone_at`].
    fn height_at(&self, at: GeoPoint) -> Result<Option<GroundHeight>, Error>;

    /// The admin units holding the point, coarse to fine; empty where none
    /// does.
    ///
    /// ```
    /// use teistro_port_geo::{GeoPoint, GeoProvider, GeoTable};
    ///
    /// let table = GeoTable::from_json(r#"{"source": {"name": "t", "version": "1", "dataVersion": "1", "tzdb": null},
    ///     "areas": [{"south": 26.3, "west": 80.0, "north": 30.5, "east": 88.2, "zone": null, "height": null,
    ///                "admin": [{"level": 0, "code": "NP", "name": "Nepal"}]}]}"#)?;
    /// let units = table.admin_at(GeoPoint::try_from_degrees(27.7, 85.3)?)?;
    /// assert_eq!(units[0].code.as_deref(), Some("NP"));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`GeoProvider::zone_at`].
    fn admin_at(&self, at: GeoPoint) -> Result<Vec<AdminUnit>, Error>;

    /// Whether the point is at sea: the provider's data covers the whole
    /// world and no area of it holds the point. [`resolve`] then answers
    /// the nautical zone, marked so (SR-144). `false` by default: a
    /// provider that knows only part of the world cannot tell the sea from
    /// land it lacks.
    ///
    /// ```
    /// use teistro_port_geo::{GeoPoint, GeoProvider, GeoTable};
    ///
    /// let table = GeoTable::from_json(r#"{"source": {"name": "t", "version": "1", "dataVersion": "1", "tzdb": null}, "areas": []}"#)?;
    /// assert!(!table.at_sea(GeoPoint::try_from_degrees(0.0, -30.0)?)?);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`GeoProvider::zone_at`].
    fn at_sea(&self, at: GeoPoint) -> Result<bool, Error> {
        let _ = at;
        Ok(false)
    }

    /// The zone rules of the tzdb release [`GeoProvider::zone_at`]'s names
    /// come from, when the provider carries them; `None` by default.
    ///
    /// ```
    /// use teistro_port_geo::{GeoProvider, GeoTable};
    ///
    /// let table = GeoTable::from_json(r#"{"source": {"name": "t", "version": "1", "dataVersion": "1", "tzdb": null}, "areas": []}"#)?;
    /// assert!(table.time_zones().is_none());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    fn time_zones(&self) -> Option<Arc<dyn TimeZoneProvider>> {
        None
    }

    /// The places matching a name, best first, as the provider ranks them
    /// (§12.3); none by default. Asked through [`search`], which checks
    /// the request and the answer.
    ///
    /// ```
    /// use teistro_port_geo::{GeoProvider, GeoTable, PlaceSearch};
    ///
    /// let table = GeoTable::from_json(r#"{"source": {"name": "t", "version": "1", "dataVersion": "1", "tzdb": null}, "areas": []}"#)?;
    /// let asked = PlaceSearch { query: "Pokhara".into(), focus: None, limit: 5, lang: None };
    /// assert!(table.search(&asked)?.is_empty());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`GeoProvider::zone_at`], and `CAPABILITY` on `search` for a
    /// provider built without its search.
    fn search(&self, asked: &PlaceSearch) -> Result<Vec<FoundPlace>, Error> {
        let _ = asked;
        Ok(Vec::new())
    }

    /// The named places nearest a point and the admin units holding it
    /// (§12.5); none by default.
    ///
    /// ```
    /// use teistro_port_geo::{GeoProvider, GeoTable, PlaceReverse};
    ///
    /// let table = GeoTable::from_json(r#"{"source": {"name": "t", "version": "1", "dataVersion": "1", "tzdb": null}, "areas": []}"#)?;
    /// let asked = PlaceReverse { latitude_deg: 27.7, longitude_deg: 85.3, limit: 5, lang: None };
    /// assert!(table.reverse(&asked)?.near.is_empty());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`GeoProvider::zone_at`].
    fn reverse(&self, asked: &PlaceReverse) -> Result<NearbyPlaces, Error> {
        let _ = asked;
        Ok(NearbyPlaces::default())
    }

    /// A place's names by language and script, and the one chosen for each
    /// language asked (§12.4); none by default.
    ///
    /// ```
    /// use teistro_port_geo::{GeoProvider, GeoTable, PlaceNamesRequest};
    ///
    /// let table = GeoTable::from_json(r#"{"source": {"name": "t", "version": "1", "dataVersion": "1", "tzdb": null}, "areas": []}"#)?;
    /// let names = table.names(&PlaceNamesRequest { id: "np/1".into(), langs: vec!["ne".into()] })?;
    /// assert!(names.all.is_empty() && names.best[0].name.is_none());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`GeoProvider::zone_at`].
    fn names(&self, asked: &PlaceNamesRequest) -> Result<PlaceNames, Error> {
        Ok(PlaceNames::none(asked))
    }

    /// The place an id names, a retired id followed to the place that
    /// replaced it (§12.7, SR-164); `None` by default, and for an id the
    /// provider does not know.
    ///
    /// ```
    /// use teistro_port_geo::{GeoProvider, GeoTable, PlaceLookup};
    ///
    /// let table = GeoTable::from_json(r#"{"source": {"name": "t", "version": "1", "dataVersion": "1", "tzdb": null}, "areas": []}"#)?;
    /// assert!(table.place(&PlaceLookup { id: "np/1".into(), lang: None })?.is_none());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`GeoProvider::zone_at`].
    fn place(&self, asked: &PlaceLookup) -> Result<Option<NamedPlace>, Error> {
        let _ = asked;
        Ok(None)
    }

    /// The places of a unit, ranked by population (§12.6); none by
    /// default.
    ///
    /// ```
    /// use teistro_port_geo::{GeoProvider, GeoTable, PlaceListing};
    ///
    /// let table = GeoTable::from_json(r#"{"source": {"name": "t", "version": "1", "dataVersion": "1", "tzdb": null}, "areas": []}"#)?;
    /// let asked: PlaceListing = serde_json::from_str(r#"{"within": "NP"}"#).map_err(|e| teistro_core::error::Error::invalid_arg(e.to_string()))?;
    /// assert!(table.list(&asked)?.is_empty());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`GeoProvider::zone_at`].
    fn list(&self, asked: &PlaceListing) -> Result<Vec<NamedPlace>, Error> {
        let _ = asked;
        Ok(Vec::new())
    }
}

impl<P: GeoProvider + ?Sized> GeoProvider for &P {
    fn source(&self) -> GeoSource {
        (**self).source()
    }

    fn zone_at(&self, at: GeoPoint) -> Result<Option<String>, Error> {
        (**self).zone_at(at)
    }

    fn height_at(&self, at: GeoPoint) -> Result<Option<GroundHeight>, Error> {
        (**self).height_at(at)
    }

    fn admin_at(&self, at: GeoPoint) -> Result<Vec<AdminUnit>, Error> {
        (**self).admin_at(at)
    }

    fn at_sea(&self, at: GeoPoint) -> Result<bool, Error> {
        (**self).at_sea(at)
    }

    fn time_zones(&self) -> Option<Arc<dyn TimeZoneProvider>> {
        (**self).time_zones()
    }

    fn search(&self, asked: &PlaceSearch) -> Result<Vec<FoundPlace>, Error> {
        (**self).search(asked)
    }

    fn reverse(&self, asked: &PlaceReverse) -> Result<NearbyPlaces, Error> {
        (**self).reverse(asked)
    }

    fn names(&self, asked: &PlaceNamesRequest) -> Result<PlaceNames, Error> {
        (**self).names(asked)
    }

    fn place(&self, asked: &PlaceLookup) -> Result<Option<NamedPlace>, Error> {
        (**self).place(asked)
    }

    fn list(&self, asked: &PlaceListing) -> Result<Vec<NamedPlace>, Error> {
        (**self).list(asked)
    }
}

/// A shared dataset answers as the dataset does.
impl<P: GeoProvider + ?Sized> GeoProvider for Arc<P> {
    fn source(&self) -> GeoSource {
        (**self).source()
    }

    fn zone_at(&self, at: GeoPoint) -> Result<Option<String>, Error> {
        (**self).zone_at(at)
    }

    fn height_at(&self, at: GeoPoint) -> Result<Option<GroundHeight>, Error> {
        (**self).height_at(at)
    }

    fn admin_at(&self, at: GeoPoint) -> Result<Vec<AdminUnit>, Error> {
        (**self).admin_at(at)
    }

    fn at_sea(&self, at: GeoPoint) -> Result<bool, Error> {
        (**self).at_sea(at)
    }

    fn time_zones(&self) -> Option<Arc<dyn TimeZoneProvider>> {
        (**self).time_zones()
    }

    fn search(&self, asked: &PlaceSearch) -> Result<Vec<FoundPlace>, Error> {
        (**self).search(asked)
    }

    fn reverse(&self, asked: &PlaceReverse) -> Result<NearbyPlaces, Error> {
        (**self).reverse(asked)
    }

    fn names(&self, asked: &PlaceNamesRequest) -> Result<PlaceNames, Error> {
        (**self).names(asked)
    }

    fn place(&self, asked: &PlaceLookup) -> Result<Option<NamedPlace>, Error> {
        (**self).place(asked)
    }

    fn list(&self, asked: &PlaceListing) -> Result<Vec<NamedPlace>, Error> {
        (**self).list(asked)
    }
}

/// A place resolved by `provider`: each of the three questions asked at
/// `at`, every answer checked, and the provider's source stamped on it.
///
/// One check for every provider, as the SDK checks an ephemeris's answers:
/// an empty zone, a height outside the altitudes a place can have, a
/// resolution that is not a positive number, or an ISO code of the wrong
/// shape is the provider's failure, `PROVIDER`.
///
/// ```
/// use teistro_port_geo::{GeoPoint, GeoTable, resolve};
///
/// let table = GeoTable::from_json(r#"{"source": {"name": "t", "version": "1", "dataVersion": "1", "tzdb": null},
///     "areas": [{"south": 26.3, "west": 80.0, "north": 30.5, "east": 88.2, "zone": "Asia/Kathmandu", "height": null,
///                "admin": [{"level": 0, "code": "NP", "name": null}]}]}"#)?;
/// let place = resolve(&table, GeoPoint::try_from_degrees(27.7172, 85.324)?)?;
/// assert_eq!((place.latitude_deg, place.admin.len()), (27.7172, 1));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// The provider's own refusal, or `PROVIDER` naming the answer that is not
/// well formed (`zone`, `height.metres`, `admin[1].code`, `source.name`).
pub fn resolve(provider: &dyn GeoProvider, at: GeoPoint) -> Result<ResolvedPlace, Error> {
    let mut zone = provider.zone_at(at)?;
    let nautical = zone.is_none() && provider.at_sea(at)?;
    if nautical {
        zone = Some(nautical_zone(at));
    }
    let place = ResolvedPlace {
        latitude_deg: at.latitude.get(),
        longitude_deg: at.longitude.get(),
        zone,
        nautical,
        height: provider.height_at(at)?,
        admin: provider.admin_at(at)?,
        source: provider.source(),
    };
    place.checked(Status::Provider)?;
    Ok(place)
}

/// The nautical zone at a point (SR-144): `n = round(longitude / 15)`,
/// named as the tz database names it, its sign inverted (`Etc/GMT-6` is
/// six hours ahead of UTC, east of Greenwich), `Etc/GMT` at `n = 0`.
/// Halfway longitudes round away from Greenwich, and 180° is
/// `Etc/GMT-12`, −180° `Etc/GMT+12`.
///
/// ```
/// use teistro_port_geo::{GeoPoint, nautical_zone};
///
/// let at = |lat, lon| GeoPoint::try_from_degrees(lat, lon);
/// assert_eq!(nautical_zone(at(15.0, 88.0)?), "Etc/GMT-6");
/// assert_eq!(nautical_zone(at(0.0, -30.0)?), "Etc/GMT+2");
/// assert_eq!(nautical_zone(at(0.0, 7.0)?), "Etc/GMT");
/// assert_eq!(nautical_zone(at(0.0, -180.0)?), "Etc/GMT+12");
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[must_use]
pub fn nautical_zone(at: GeoPoint) -> String {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "n lies in -12..=12, as the longitude lies in -180..=180"
    )]
    let n = (at.longitude.get() / 15.0).round() as i8;
    match n {
        0 => String::from("Etc/GMT"),
        east if east > 0 => format!("Etc/GMT-{east}"),
        west => format!("Etc/GMT+{}", -west),
    }
}

/// `why` with `status` in place of its own.
fn restated(mut why: Error, status: Status) -> Error {
    why.status = status;
    why
}

/// An `INVALID_ARG` at `field` under `root`.
fn refused(root: &str, field: &str, message: String) -> Error {
    let at = if root.is_empty() {
        field.to_owned()
    } else {
        format!("{root}.{field}")
    };
    Error::invalid_arg(message).with_field(at)
}

impl GeoSource {
    /// Refuses an empty name or data version.
    fn check(&self, root: &str) -> Result<(), Error> {
        if self.name.trim().is_empty() {
            return Err(refused(
                root,
                "name",
                String::from("a geo source names itself"),
            ));
        }
        if self.data_version.trim().is_empty() {
            return Err(refused(
                root,
                "dataVersion",
                String::from("a geo source names its data's release"),
            ));
        }
        Ok(())
    }
}

/// The three answers at a point, as a provider or a table gives them.
struct Answers<'a> {
    zone: Option<&'a str>,
    height: Option<&'a GroundHeight>,
    admin: &'a [AdminUnit],
}

impl Answers<'_> {
    /// Refuses the first answer that is not well formed, by its field
    /// under `root`.
    fn check(&self, root: &str) -> Result<(), Error> {
        if let Some(zone) = self.zone
            && (zone.is_empty() || zone.chars().any(char::is_whitespace))
        {
            return Err(
                refused(root, "zone", format!("`{zone}` is not an IANA zone name"))
                    .with_hint("a name such as `Asia/Kathmandu`, or null where the data has none"),
            );
        }
        if let Some(height) = self.height {
            height.check(&field_of(root, "height"))?;
        }
        let mut country: Option<&str> = None;
        let mut last = 0_u8;
        for (at, unit) in self.admin.iter().enumerate() {
            let root = format!("{}[{at}]", field_of(root, "admin"));
            if unit.level < last {
                return Err(refused(
                    &root,
                    "level",
                    String::from("admin units are listed coarse to fine"),
                ));
            }
            last = unit.level;
            let Some(code) = unit.code.as_deref() else {
                continue;
            };
            if unit.level == 0 {
                if !country_code(code) {
                    return Err(refused(
                        &root,
                        "code",
                        format!("`{code}` is not an ISO 3166-1 alpha-2 code"),
                    )
                    .with_hint("two capital letters, as `NP`"));
                }
                country = Some(code);
            } else if !subdivision_code(code) || country.is_some_and(|c| !code.starts_with(c)) {
                return Err(refused(
                    &root,
                    "code",
                    format!("`{code}` is not an ISO 3166-2 code of this country"),
                )
                .with_hint(
                    "the country's code, a hyphen and one to three letters or digits, as `NP-P3`",
                ));
            }
        }
        Ok(())
    }
}

/// `field` under `root`.
fn field_of(root: &str, field: &str) -> String {
    if root.is_empty() {
        field.to_owned()
    } else {
        format!("{root}.{field}")
    }
}

/// Two capital letters.
fn country_code(code: &str) -> bool {
    code.len() == 2 && code.bytes().all(|b| b.is_ascii_uppercase())
}

/// A country's code, a hyphen, and one to three capital letters or digits.
fn subdivision_code(code: &str) -> bool {
    match code.split_once('-') {
        Some((country, part)) => {
            country_code(country)
                && (1..=3).contains(&part.len())
                && part
                    .bytes()
                    .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
        }
        None => false,
    }
}

impl GroundHeight {
    /// Refuses a height a place cannot have, a resolution that is not a
    /// positive number, or an empty source or datum.
    fn check(&self, root: &str) -> Result<(), Error> {
        if let Err(why) = Altitude::try_new(self.metres) {
            return Err(Error::from(why).with_field(field_of(root, "metres")));
        }
        if !(self.resolution_m.is_finite() && self.resolution_m > 0.0) {
            return Err(refused(
                root,
                "resolutionM",
                format!(
                    "a resolution of {} m is not a positive number",
                    self.resolution_m
                ),
            ));
        }
        if self.source.trim().is_empty() {
            return Err(refused(
                root,
                "source",
                String::from("a height names the dataset it was read from"),
            ));
        }
        if self.datum.trim().is_empty() {
            return Err(refused(
                root,
                "datum",
                String::from("a height names its vertical datum"),
            )
            .with_hint("`EGM2008`, `EGM96` or `WGS84`"));
        }
        Ok(())
    }
}

/// One box of a [`GeoTable`], and what it answers for the points inside it.
///
/// ```
/// use teistro_port_geo::GeoBox;
///
/// let nepal: GeoBox = serde_json::from_str(r#"{"south": 26.3, "west": 80.0, "north": 30.5, "east": 88.2,
///     "zone": "Asia/Kathmandu", "height": null, "admin": []}"#)?;
/// assert_eq!(nepal.zone.as_deref(), Some("Asia/Kathmandu"));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GeoBox {
    /// Its southern edge, in degrees.
    pub south: f64,
    /// Its western edge, in degrees; east of `east` for a box that crosses
    /// the antimeridian.
    pub west: f64,
    /// Its northern edge, in degrees.
    pub north: f64,
    /// Its eastern edge, in degrees.
    pub east: f64,
    /// The zone it answers, or `null`.
    #[serde(default)]
    pub zone: Option<String>,
    /// The ground height it answers, or `null`.
    #[serde(default)]
    pub height: Option<GroundHeight>,
    /// The admin units it answers; none when empty.
    #[serde(default)]
    pub admin: Vec<AdminUnit>,
}

impl GeoBox {
    /// Whether the box holds the point, edges included.
    fn holds(&self, at: GeoPoint) -> bool {
        let (lat, lon) = (at.latitude.get(), at.longitude.get());
        let across = if self.west <= self.east {
            (self.west..=self.east).contains(&lon)
        } else {
            lon >= self.west || lon <= self.east
        };
        (self.south..=self.north).contains(&lat) && across
    }

    /// Refuses edges out of range or crossed, and any answer that is not
    /// well formed, under `root`.
    fn check(&self, root: &str) -> Result<(), Error> {
        for (field, value, limit) in [
            ("south", self.south, 90.0),
            ("north", self.north, 90.0),
            ("west", self.west, 180.0),
            ("east", self.east, 180.0),
        ] {
            if !(value.is_finite() && value.abs() <= limit) {
                return Err(refused(
                    root,
                    field,
                    format!("{value} is not within ±{limit} degrees"),
                ));
            }
        }
        if self.south > self.north {
            return Err(refused(
                root,
                "north",
                String::from("a box's north is not south of its south"),
            ));
        }
        Answers {
            zone: self.zone.as_deref(),
            height: self.height.as_ref(),
            admin: &self.admin,
        }
        .check(root)
    }
}

/// The provider the SDK ships: boxes, each answering any of the three
/// questions for the points inside it, the **last** box holding a point
/// that answers a question giving the answer. A table is written coarse
/// to fine, so a finer box overrides a coarser one for what it states and
/// inherits the rest.
///
/// The test provider of every question and every binding, and a small
/// place catalogue a consumer can write.
///
/// ```
/// use teistro_port_geo::{GeoPoint, GeoProvider, GeoTable};
///
/// let table = GeoTable::from_json(r#"{
///     "source": {"name": "places", "version": "1", "dataVersion": "2026.11.0", "tzdb": "2026c"},
///     "areas": [
///         {"south": 26.3, "west": 80.0, "north": 30.5, "east": 88.2, "zone": "Asia/Kathmandu",
///          "admin": [{"level": 0, "code": "NP", "name": "Nepal"}]},
///         {"south": 27.6, "west": 85.2, "north": 27.8, "east": 85.5,
///          "height": {"metres": 1337, "source": "survey", "resolutionM": 30, "datum": "EGM2008"},
///          "admin": [{"level": 0, "code": "NP", "name": "Nepal"}, {"level": 1, "code": "NP-P3", "name": "Bagmati"}]}
///     ]
/// }"#)?;
/// let kathmandu = GeoPoint::try_from_degrees(27.7172, 85.324)?;
/// // The fine box's units, and the coarse box's zone, which it inherits.
/// assert_eq!(table.admin_at(kathmandu)?.len(), 2);
/// assert_eq!(table.zone_at(kathmandu)?.as_deref(), Some("Asia/Kathmandu"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GeoTable {
    /// Who answers, and from which data.
    pub source: GeoSource,
    /// The boxes, coarse to fine.
    pub areas: Vec<GeoBox>,
    /// The places of its catalogue, which answer a search, a reverse, a
    /// place's names and a listing (§12.8); none when left out.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub places: Vec<TablePlace>,
}

impl GeoTable {
    /// A table from its JSON, read strictly and checked: every key one it
    /// reads, every edge in range, every answer well formed.
    ///
    /// ```
    /// use teistro_port_geo::GeoTable;
    ///
    /// let refused = GeoTable::from_json(r#"{"source": {"name": "t", "version": "1", "dataVersion": "1", "tzdb": null},
    ///     "areas": [{"south": 0, "west": 0, "north": 1, "east": 1, "admin": [{"level": 0, "code": "Nepal", "name": null}]}]}"#).unwrap_err();
    /// assert_eq!(refused.field(), Some("geo.areas[0].admin[0].code"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming the first part that does not read or is not
    /// well formed, under `geo`.
    pub fn from_json(text: &str) -> Result<GeoTable, Error> {
        let table: GeoTable = teistro_core::strict::read(text, "geo")?;
        table.source.check("geo.source")?;
        for (at, area) in table.areas.iter().enumerate() {
            area.check(&format!("geo.areas[{at}]"))?;
        }
        table::check_places(&table.places)?;
        Ok(table)
    }

    /// The last box holding `at` that answers what `answer` reads.
    fn answer<T>(&self, at: GeoPoint, answer: impl Fn(&GeoBox) -> Option<T>) -> Option<T> {
        self.areas
            .iter()
            .rev()
            .filter(|area| area.holds(at))
            .find_map(answer)
    }
}

impl GeoProvider for GeoTable {
    fn source(&self) -> GeoSource {
        self.source.clone()
    }

    fn zone_at(&self, at: GeoPoint) -> Result<Option<String>, Error> {
        Ok(self.answer(at, |area| area.zone.clone()))
    }

    fn height_at(&self, at: GeoPoint) -> Result<Option<GroundHeight>, Error> {
        Ok(self.answer(at, |area| area.height.clone()))
    }

    fn admin_at(&self, at: GeoPoint) -> Result<Vec<AdminUnit>, Error> {
        Ok(self
            .answer(at, |area| {
                (!area.admin.is_empty()).then(|| area.admin.clone())
            })
            .unwrap_or_default())
    }

    fn search(&self, asked: &PlaceSearch) -> Result<Vec<FoundPlace>, Error> {
        Ok(self.search_places(asked))
    }

    fn reverse(&self, asked: &PlaceReverse) -> Result<NearbyPlaces, Error> {
        Ok(self.reverse_places(asked))
    }

    fn names(&self, asked: &PlaceNamesRequest) -> Result<PlaceNames, Error> {
        Ok(self.place_names(asked))
    }

    fn place(&self, asked: &PlaceLookup) -> Result<Option<NamedPlace>, Error> {
        Ok(self.place_of(asked))
    }

    fn list(&self, asked: &PlaceListing) -> Result<Vec<NamedPlace>, Error> {
        Ok(self.list_places(asked))
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::indexing_slicing,
        clippy::float_cmp,
        reason = "tests fail by panicking, and compare the numbers a table states"
    )]

    use super::*;

    const TABLE: &str = r#"{
        "source": {"name": "test", "version": "1", "dataVersion": "2026.11.0", "tzdb": "2026c"},
        "areas": [
            {"south": 26.3, "west": 80.0, "north": 30.5, "east": 88.2, "zone": "Asia/Kathmandu",
             "admin": [{"level": 0, "code": "NP", "name": "Nepal"}]},
            {"south": 27.6, "west": 85.2, "north": 27.8, "east": 85.5,
             "height": {"metres": 1337, "source": "survey", "resolutionM": 30, "datum": "EGM2008"},
             "admin": [{"level": 0, "code": "NP", "name": "Nepal"}, {"level": 1, "code": "NP-P3", "name": "Bagmati"}]},
            {"south": -20.0, "west": 170.0, "north": -10.0, "east": -170.0, "zone": "Pacific/Fiji"}
        ]
    }"#;

    fn table() -> GeoTable {
        GeoTable::from_json(TABLE).unwrap()
    }

    fn at(lat: f64, lon: f64) -> GeoPoint {
        GeoPoint::try_from_degrees(lat, lon).unwrap()
    }

    #[test]
    fn each_question_is_answered_by_the_last_box_that_answers_it() {
        let table = table();
        let kathmandu = resolve(&table, at(27.7172, 85.324)).unwrap();
        assert_eq!(kathmandu.zone.as_deref(), Some("Asia/Kathmandu"));
        assert_eq!(kathmandu.height.as_ref().unwrap().metres, 1337.0);
        assert_eq!(kathmandu.admin[1].code.as_deref(), Some("NP-P3"));
        assert_eq!(kathmandu.source.data_version, "2026.11.0");
        // Pokhara: the coarse box alone, so no height.
        let pokhara = resolve(&table, at(28.2096, 83.9856)).unwrap();
        assert_eq!(pokhara.zone.as_deref(), Some("Asia/Kathmandu"));
        assert!(pokhara.height.is_none());
        assert_eq!(pokhara.admin.len(), 1);
        // Across the antimeridian, on both sides.
        assert_eq!(
            table.zone_at(at(-18.0, 178.0)).unwrap().as_deref(),
            Some("Pacific/Fiji")
        );
        assert_eq!(
            table.zone_at(at(-18.0, -179.0)).unwrap().as_deref(),
            Some("Pacific/Fiji")
        );
        assert_eq!(table.zone_at(at(-18.0, 160.0)).unwrap(), None);
        assert_eq!(table.zone_at(at(-25.0, 178.0)).unwrap(), None);
        // A resolved place reads back as written.
        let json = serde_json::to_string(&kathmandu).unwrap();
        assert!(json.contains("\"resolutionM\":30.0") && json.contains("\"tzdb\":\"2026c\""));
        let back: ResolvedPlace = serde_json::from_str(&json).unwrap();
        assert_eq!(back, kathmandu);
        assert!(back.checked(Status::InvalidArg).is_ok());
    }

    #[test]
    fn a_table_is_refused_by_the_part_that_is_wrong() {
        let refused = |areas: &str| {
            GeoTable::from_json(&format!(
                r#"{{"source": {{"name": "t", "version": "1", "dataVersion": "1", "tzdb": null}}, "areas": [{areas}]}}"#
            ))
            .unwrap_err()
            .field()
            .map(str::to_owned)
        };
        let field = |text: &str| refused(text).unwrap();
        assert_eq!(
            field(r#"{"south": 0, "west": 0, "north": 91, "east": 1}"#),
            "geo.areas[0].north"
        );
        assert_eq!(
            field(r#"{"south": 0, "west": 181, "north": 1, "east": 1}"#),
            "geo.areas[0].west"
        );
        assert_eq!(
            field(r#"{"south": 2, "west": 0, "north": 1, "east": 1}"#),
            "geo.areas[0].north"
        );
        assert_eq!(
            field(r#"{"south": 0, "west": 0, "north": 1, "east": 1, "zone": ""}"#),
            "geo.areas[0].zone"
        );
        assert_eq!(
            field(r#"{"south": 0, "west": 0, "north": 1, "east": 1, "zone": "Asia/Kath mandu"}"#),
            "geo.areas[0].zone"
        );
        let height = |body: &str| {
            field(&format!(
                r#"{{"south": 0, "west": 0, "north": 1, "east": 1, "height": {body}}}"#
            ))
        };
        assert_eq!(
            height(r#"{"metres": 20000, "source": "s", "resolutionM": 30, "datum": "EGM2008"}"#),
            "geo.areas[0].height.metres"
        );
        assert_eq!(
            height(r#"{"metres": 1, "source": "s", "resolutionM": 0, "datum": "EGM2008"}"#),
            "geo.areas[0].height.resolutionM"
        );
        assert_eq!(
            height(r#"{"metres": 1, "source": " ", "resolutionM": 1, "datum": "EGM2008"}"#),
            "geo.areas[0].height.source"
        );
        assert_eq!(
            height(r#"{"metres": 1, "source": "s", "resolutionM": 1, "datum": ""}"#),
            "geo.areas[0].height.datum"
        );
        let admin = |units: &str| {
            field(&format!(
                r#"{{"south": 0, "west": 0, "north": 1, "east": 1, "admin": {units}}}"#
            ))
        };
        assert_eq!(
            admin(r#"[{"level": 0, "code": "np", "name": null}]"#),
            "geo.areas[0].admin[0].code"
        );
        assert_eq!(
            admin(
                r#"[{"level": 0, "code": "NP", "name": null}, {"level": 1, "code": "IN-MH", "name": null}]"#
            ),
            "geo.areas[0].admin[1].code"
        );
        assert_eq!(
            admin(r#"[{"level": 1, "code": "NP-P3456", "name": null}]"#),
            "geo.areas[0].admin[0].code"
        );
        assert_eq!(
            admin(r#"[{"level": 1, "code": "NPP3", "name": null}]"#),
            "geo.areas[0].admin[0].code"
        );
        assert_eq!(
            admin(
                r#"[{"level": 2, "code": null, "name": "x"}, {"level": 1, "code": null, "name": null}]"#
            ),
            "geo.areas[0].admin[1].level"
        );
        // A unit with no code is a unit, and a subdivision alone is checked by its shape.
        assert!(
            GeoTable::from_json(
                r#"{"source": {"name": "t", "version": "1", "dataVersion": "1", "tzdb": null}, "areas": [{"south": 0, "west": 0, "north": 1, "east": 1, "admin": [{"level": 1, "code": "NP-P3", "name": null}, {"level": 3, "code": null, "name": "Kathmandu"}]}]}"#
            )
            .is_ok()
        );
        assert!(
            field(r#"{"south": 0, "west": 0, "north": 1, "east": 1, "tz": "x"}"#)
                .starts_with("geo.areas[0]")
        );
        let source = GeoTable::from_json(
            r#"{"source": {"name": "", "version": "1", "dataVersion": "1", "tzdb": null}, "areas": []}"#,
        );
        assert_eq!(source.unwrap_err().field(), Some("geo.source.name"));
        let release = GeoTable::from_json(
            r#"{"source": {"name": "t", "version": "1", "dataVersion": "", "tzdb": null}, "areas": []}"#,
        );
        assert_eq!(release.unwrap_err().field(), Some("geo.source.dataVersion"));
    }

    /// A provider whose answers are whatever a test sets.
    struct Plain {
        source: GeoSource,
        zone: Option<String>,
        height: Option<GroundHeight>,
        admin: Vec<AdminUnit>,
        fails: bool,
        sea: bool,
    }

    impl GeoProvider for Plain {
        fn source(&self) -> GeoSource {
            self.source.clone()
        }
        fn zone_at(&self, _: GeoPoint) -> Result<Option<String>, Error> {
            if self.fails {
                return Err(
                    Error::new(Status::Provider, "the data is not installed").with_field("dem")
                );
            }
            Ok(self.zone.clone())
        }
        fn height_at(&self, _: GeoPoint) -> Result<Option<GroundHeight>, Error> {
            Ok(self.height.clone())
        }
        fn admin_at(&self, _: GeoPoint) -> Result<Vec<AdminUnit>, Error> {
            Ok(self.admin.clone())
        }
        fn at_sea(&self, _: GeoPoint) -> Result<bool, Error> {
            Ok(self.sea)
        }
    }

    fn plain() -> Plain {
        Plain {
            source: table().source,
            zone: Some(String::from("Asia/Kathmandu")),
            height: None,
            admin: Vec::new(),
            fails: false,
            sea: false,
        }
    }

    #[test]
    fn the_sea_answers_its_nautical_zone_and_says_so() {
        let mut provider = plain();
        provider.zone = None;
        // Not at sea: no zone is no zone, as before.
        let land = resolve(&provider, at(15.0, 88.0)).unwrap();
        assert_eq!((land.zone, land.nautical), (None, false));
        provider.sea = true;
        let bay = resolve(&provider, at(15.0, 88.0)).unwrap();
        assert_eq!(
            (bay.zone.as_deref(), bay.nautical),
            (Some("Etc/GMT-6"), true)
        );
        let json = serde_json::to_value(&bay).unwrap();
        assert_eq!(json["nautical"], true);
        // A land zone is never replaced, and a land answer keeps its shape.
        provider.zone = Some(String::from("Asia/Kolkata"));
        let coast = resolve(&provider, at(15.0, 88.0)).unwrap();
        assert!(!coast.nautical);
        assert!(
            serde_json::to_value(&coast)
                .unwrap()
                .get("nautical")
                .is_none()
        );
        // A nautical mark on any other zone is refused.
        let wrong = ResolvedPlace {
            longitude_deg: 30.0,
            ..bay
        };
        assert_eq!(
            wrong.checked(Status::InvalidArg).unwrap_err().field(),
            Some("nautical")
        );
        for (lon, zone) in [
            (7.49, "Etc/GMT"),
            (7.5, "Etc/GMT-1"),
            (-7.5, "Etc/GMT+1"),
            (172.4, "Etc/GMT-11"),
            (180.0, "Etc/GMT-12"),
        ] {
            assert_eq!(nautical_zone(at(0.0, lon)), zone, "{lon}");
        }
    }

    #[test]
    fn a_providers_malformed_answer_is_its_failure() {
        let point = at(27.7, 85.3);
        let status = |provider: &Plain| {
            let why = resolve(provider, point).unwrap_err();
            (why.status, why.field().unwrap().to_owned())
        };
        let mut provider = plain();
        provider.zone = Some(String::new());
        assert_eq!(status(&provider), (Status::Provider, String::from("zone")));
        let mut provider = plain();
        provider.height = Some(GroundHeight {
            metres: f64::NAN,
            source: String::from("s"),
            resolution_m: 1.0,
            datum: String::from("EGM2008"),
        });
        assert_eq!(
            status(&provider),
            (Status::Provider, String::from("height.metres"))
        );
        let mut provider = plain();
        provider.admin = vec![AdminUnit {
            level: 0,
            code: Some(String::from("NPL")),
            name: None,
        }];
        assert_eq!(
            status(&provider),
            (Status::Provider, String::from("admin[0].code"))
        );
        let mut provider = plain();
        provider.source.name = String::new();
        assert_eq!(
            status(&provider),
            (Status::Provider, String::from("source.name"))
        );
        // Its own refusal passes through as it was.
        let mut provider = plain();
        provider.fails = true;
        assert_eq!(status(&provider), (Status::Provider, String::from("dem")));
        // A point out of range is the caller's.
        let place = ResolvedPlace {
            latitude_deg: 95.0,
            ..resolve(&plain(), point).unwrap()
        };
        let why = place.checked(Status::InvalidArg).unwrap_err();
        assert_eq!(
            (why.status, why.field()),
            (Status::InvalidArg, Some("latitudeDeg"))
        );
    }

    /// A borrowed or shared provider answers every question as the
    /// provider does, refusals included.
    #[test]
    fn a_borrowed_provider_answers_as_the_provider() {
        fn answers<P: GeoProvider>(provider: &P, point: GeoPoint) -> String {
            format!(
                "{:?} {:?} {:?} {:?} {}",
                provider.source(),
                provider.zone_at(point),
                provider.height_at(point),
                provider.admin_at(point),
                provider.time_zones().is_some()
            )
        }
        for point in [at(27.7172, 85.324), at(0.0, 0.0)] {
            assert_eq!(answers(&&table(), point), answers(&table(), point));
            let shared = Arc::new(table());
            assert_eq!(answers(&shared, point), answers(&table(), point));
        }
        let mut failing = plain();
        failing.fails = true;
        let borrowed = answers(&&failing, at(0.0, 0.0));
        assert!(borrowed.contains("not installed"));
        assert_eq!(answers(&Arc::new(failing), at(0.0, 0.0)), borrowed);
    }
}
