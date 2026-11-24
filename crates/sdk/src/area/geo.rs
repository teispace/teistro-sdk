//! `sdk.geo`: a place resolved through the geo port, its zone, ground
//! height and admin codes (`03-design/geo-port.md`, SR-87), and the place
//! catalogue's questions: a search, a reverse, a place's names and a
//! unit's places (§12, SR-141 and SR-129).

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use teistro_core::error::{Error, Status};
use teistro_port_geo::{
    FoundPlace, GeoPoint, GeoProvider, GeoSource, GeoTable, LookedUpPlace, NamedPlace,
    NearbyPlaces, PlaceListing, PlaceLookup, PlaceNames, PlaceNamesRequest, PlaceReverse,
    PlaceSearch, ResolvedPlace, resolve,
};

use crate::context::Context;

/// A place to resolve, as `geo.resolve` reads it.
///
/// ```
/// use teistro::PlaceRequest;
///
/// let asked = PlaceRequest::from_json(r#"{"latitudeDeg": 27.7172, "longitudeDeg": 85.324}"#)?;
/// assert_eq!(asked.longitude_deg, 85.324);
/// let refused = PlaceRequest::from_json(r#"{"latitudeDeg": 27.7, "longitude": 85.3}"#).unwrap_err();
/// assert!(refused.field().is_some_and(|field| field.starts_with("geo")));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlaceRequest {
    /// The latitude, in degrees, north positive.
    pub latitude_deg: f64,
    /// The longitude, in degrees, east positive.
    pub longitude_deg: f64,
}

impl PlaceRequest {
    /// The request read from its JSON, refused by its field under `geo`.
    ///
    /// ```
    /// use teistro::PlaceRequest;
    ///
    /// assert!(PlaceRequest::from_json("[27.7, 85.3]").is_err());
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record or a key it does not
    /// read.
    pub fn from_json(text: &str) -> Result<PlaceRequest, Error> {
        teistro_core::strict::read(text, "geo")
    }
}

/// `sdk.geo`: a place's zone, ground height and admin codes from the
/// context's geo provider, which the SDK never consults on its own.
///
/// ```
/// use teistro::{Context, PlaceRequest, Status};
///
/// // A context is given no provider unless it is asked for one.
/// let sdk = Context::builder().build()?;
/// let refused = sdk.geo().resolve(&PlaceRequest { latitude_deg: 27.7, longitude_deg: 85.3 }).unwrap_err();
/// assert_eq!((refused.status, refused.field()), (Status::Capability, Some("geo")));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Copy, Debug)]
pub struct GeoArea<'a> {
    context: &'a Context,
}

impl<'a> GeoArea<'a> {
    pub(crate) fn of(context: &'a Context) -> GeoArea<'a> {
        GeoArea { context }
    }

    /// The context this area was read off.
    ///
    /// ```
    /// let sdk = teistro::Context::builder().build()?;
    /// assert_eq!(sdk.geo().context().profile(), sdk.profile());
    /// # Ok::<(), teistro::Error>(())
    /// ```
    #[must_use]
    pub fn context(self) -> &'a Context {
        self.context
    }

    /// A place resolved: the IANA zone, the ground height with its source,
    /// resolution and datum, and the admin units with their ISO 3166 codes
    /// at the point, each checked, with the provider's name and data
    /// release. What a chart request's `place` takes, so the request a
    /// receipt keeps holds the answer and replays without the provider.
    ///
    /// ```
    /// use std::sync::Arc;
    /// use teistro::{Context, GeoTable, PlaceRequest};
    ///
    /// let table = GeoTable::from_json(r#"{"source": {"name": "places", "version": "1", "dataVersion": "2026.11.0", "tzdb": null},
    ///     "areas": [{"south": 27.6, "west": 85.2, "north": 27.8, "east": 85.5, "zone": "Asia/Kathmandu",
    ///                "height": {"metres": 1337, "source": "survey", "resolutionM": 30, "datum": "EGM2008"}}]}"#)?;
    /// let sdk = Context::builder().geo(Arc::new(table)).build()?;
    /// let place = sdk.geo().resolve(&PlaceRequest { latitude_deg: 27.7172, longitude_deg: 85.324 })?;
    /// assert_eq!(place.height.map(|height| height.metres), Some(1337.0));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `CAPABILITY` on `geo` for a context with no provider; `INVALID_ARG`
    /// on `latitudeDeg` or `longitudeDeg` out of range; the provider's own
    /// refusal; `PROVIDER` naming an answer that is not well formed.
    pub fn resolve(self, asked: &PlaceRequest) -> Result<ResolvedPlace, Error> {
        let point = GeoPoint::try_from_degrees(asked.latitude_deg, asked.longitude_deg)?;
        resolve(&*self.provider()?, point)
    }

    /// The places matching a name, best first, as the provider ranks them:
    /// in any script the provider's data holds (`काठमाडौं`, `Kathmandu`),
    /// `"Name, Parent"` putting the named parent's places first, a focus
    /// point preferring places near it. Each place is named in `lang` and
    /// carries its kind, admin level, ISO code, point, population and
    /// parents (`03-design/geo-port.md` §12).
    ///
    /// ```
    /// use std::sync::Arc;
    /// use teistro::{Context, GeoTable, PlaceSearch};
    ///
    /// let table = GeoTable::from_json(include_str!("../../tests/geo-table.json"))?;
    /// let sdk = Context::builder().geo(Arc::new(table)).build()?;
    /// let asked = PlaceSearch { query: "काठमाडौं".into(), focus: None, limit: 3, lang: Some("en".into()) };
    /// let found = sdk.geo().search(&asked)?;
    /// assert_eq!(found[0].place.name, "Kathmandu");
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `CAPABILITY` on `geo` for a context with no provider; `INVALID_ARG`
    /// on `query`, `limit`, `focus` or `lang`; the provider's own refusal
    /// (`CAPABILITY` on `search` for a provider built without search);
    /// `PROVIDER` naming an answer that is not well formed.
    pub fn search(self, asked: &PlaceSearch) -> Result<Vec<FoundPlace>, Error> {
        asked.checked()?;
        teistro_port_geo::search(&*self.provider()?, asked)
    }

    /// The named places nearest a point, nearest first, and the admin units
    /// holding it, coarse to fine: what names "my location", to the finest
    /// unit the provider's data outlines (the ward in Nepal where it holds
    /// them) or the nearest settlement it falls inside. A level the data
    /// does not hold is absent, never a refusal.
    ///
    /// ```
    /// use std::sync::Arc;
    /// use teistro::{Context, GeoTable, PlaceReverse};
    ///
    /// let table = GeoTable::from_json(include_str!("../../tests/geo-table.json"))?;
    /// let sdk = Context::builder().geo(Arc::new(table)).build()?;
    /// let asked = PlaceReverse { latitude_deg: 27.7172, longitude_deg: 85.324, limit: 1, lang: None };
    /// let nearby = sdk.geo().reverse(&asked)?;
    /// assert_eq!(nearby.admin[0].code.as_deref(), Some("NP"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`GeoArea::search`], the point refused on `latitudeDeg` or
    /// `longitudeDeg`.
    pub fn reverse(self, asked: &PlaceReverse) -> Result<NearbyPlaces, Error> {
        asked.checked()?;
        teistro_port_geo::reverse(&*self.provider()?, asked)
    }

    /// A place's names, each with its language, script, type and origin
    /// (a reviewed name says so), and the one chosen for each language
    /// asked: a name tagged with it, else one in its script.
    ///
    /// ```
    /// use std::sync::Arc;
    /// use teistro::{Context, GeoTable, PlaceNamesRequest};
    ///
    /// let table = GeoTable::from_json(include_str!("../../tests/geo-table.json"))?;
    /// let sdk = Context::builder().geo(Arc::new(table)).build()?;
    /// let asked = PlaceNamesRequest { id: "np-ktm".into(), langs: vec!["ne".into(), "en".into()] };
    /// let names = sdk.geo().names(&asked)?;
    /// assert_eq!(names.best[0].name.as_ref().map(|name| name.text.as_str()), Some("काठमाडौं"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`GeoArea::search`], on `id` or `langs[i]`.
    pub fn names(self, asked: &PlaceNamesRequest) -> Result<PlaceNames, Error> {
        asked.checked()?;
        teistro_port_geo::names(&*self.provider()?, asked)
    }

    /// The place an id names, as a saved place reads it back: its record
    /// named in the language asked, with its id line, time zone and
    /// population. A provider follows a retired id to the place that
    /// replaced it and says so (`moved`), so a caller stores the answer's
    /// id over the one it kept; no place for an id the provider does not
    /// know (SR-164).
    ///
    /// ```
    /// use std::sync::Arc;
    /// use teistro::{Context, GeoTable, PlaceLookup};
    ///
    /// let table = GeoTable::from_json(include_str!("../../tests/geo-table.json"))?;
    /// let sdk = Context::builder().geo(Arc::new(table)).build()?;
    /// let asked = PlaceLookup { id: "np-ktm".into(), lang: Some("ne".into()) };
    /// let found = sdk.geo().place(&asked)?;
    /// let place = found.place.ok_or("held")?;
    /// assert_eq!((place.id.as_str(), place.name.as_str(), found.moved), ("np-ktm", "काठमाडौं", false));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`GeoArea::search`], on `id` or `lang`.
    pub fn place(self, asked: &PlaceLookup) -> Result<LookedUpPlace, Error> {
        asked.checked()?;
        teistro_port_geo::place(&*self.provider()?, asked)
    }

    /// The places of a unit (named by its id or its ISO 3166 code),
    /// filtered by admin level and kind, ranked by population, each with
    /// its number of people and whether it was counted or modelled.
    ///
    /// ```
    /// use std::sync::Arc;
    /// use teistro::{Context, GeoTable, PlaceListing};
    ///
    /// let table = GeoTable::from_json(include_str!("../../tests/geo-table.json"))?;
    /// let sdk = Context::builder().geo(Arc::new(table)).build()?;
    /// let asked = PlaceListing::within("NP-P3").at_any_depth();
    /// let places = sdk.geo().list(&asked)?;
    /// assert_eq!(places[0].name, "Kathmandu");
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`GeoArea::search`], on `within`, `depth`, `limit` or `lang`.
    pub fn list(self, asked: &PlaceListing) -> Result<Vec<NamedPlace>, Error> {
        asked.checked()?;
        teistro_port_geo::list(&*self.provider()?, asked)
    }

    /// The context's provider, or `CAPABILITY` on `geo`.
    fn provider(self) -> Result<Arc<dyn GeoProvider>, Error> {
        self.context.geo_provider().ok_or_else(|| {
            Error::new(
                Status::Capability,
                "this context has no geo provider to ask",
            )
            .with_field("geo")
            .with_hint("load a geo table with `geo.load`, or build the context with a provider")
        })
    }

    /// Installs a [`GeoTable`] read from its JSON as this context's geo
    /// provider, in place of any it had, answering the table's source. A
    /// refused table leaves the context as it was.
    ///
    /// ```
    /// use teistro::{Context, PlaceRequest};
    ///
    /// let sdk = Context::builder().build()?;
    /// let source = sdk.geo().load(r#"{"source": {"name": "places", "version": "1", "dataVersion": "7", "tzdb": null},
    ///     "areas": [{"south": 26.3, "west": 80.0, "north": 30.5, "east": 88.2, "zone": "Asia/Kathmandu"}]}"#)?;
    /// assert_eq!(source.data_version, "7");
    /// let place = sdk.geo().resolve(&PlaceRequest { latitude_deg: 27.7, longitude_deg: 85.3 })?;
    /// assert_eq!(place.zone.as_deref(), Some("Asia/Kathmandu"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming the part of the table that does not read or is
    /// not well formed, under `geo`.
    pub fn load(self, table_json: &str) -> Result<GeoSource, Error> {
        let table = GeoTable::from_json(table_json)?;
        let source = table.source.clone();
        self.context.install_geo(Arc::new(table));
        Ok(source)
    }
}
