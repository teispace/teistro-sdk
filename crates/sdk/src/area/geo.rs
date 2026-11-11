//! `sdk.geo`: a place resolved through the geo port, its zone, ground
//! height and admin codes (`03-design/geo-port.md`, SR-87).

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use teistro_core::error::{Error, Status};
use teistro_port_geo::{GeoPoint, GeoSource, GeoTable, ResolvedPlace, resolve};

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
        let provider = self.context.geo_provider().ok_or_else(|| {
            Error::new(
                Status::Capability,
                "this context has no geo provider to resolve a place through",
            )
            .with_field("geo")
            .with_hint("load a geo table with `geo.load`, or build the context with a provider")
        })?;
        resolve(&*provider, point)
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
