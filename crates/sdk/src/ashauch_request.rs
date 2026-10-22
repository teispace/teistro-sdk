//! An ashauch, as a binding asks it (`03-design/ashauch.md`): the place
//! and the clock its days are found at, the table (the
//! *Dharmasindhu*'s when left out), and the request itself.

use serde::{Deserialize, Serialize};
use teistro_core::envelope::Envelope;
use teistro_core::error::Error;
use teistro_core::quantity::Place;
use teistro_core::time::UtcOffset;
use teistro_panchanga::ashauch::{AshauchRequest, AshauchTable};

use crate::AlmanacArea;
use crate::asked::{offset_of, place_of};

/// The record's name where a binding sends it, which a refusal is named
/// under.
const ASHAUCH: &str = "ashauch";

/// An ashauch's place, clock, table and request, as a binding asks it.
#[derive(Clone, Debug, PartialEq)]
pub struct AshauchQuery {
    /// The place whose sunrises count the days.
    pub place: Place,
    /// The offset its civil days run at.
    pub offset: UtcOffset,
    /// The tradition's table.
    pub table: AshauchTable,
    /// The event, the observer and what is already running.
    pub request: AshauchRequest,
}

/// The query as a binding writes it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "AshauchQuery"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AshauchAsked {
    /// The place's latitude, degrees north positive.
    latitude_deg: f64,
    /// The place's longitude, degrees east positive.
    longitude_deg: f64,
    /// The place's height above sea level in metres, 0 when left out.
    #[serde(default)]
    altitude_m: f64,
    /// The offset from UTC the place's civil days run at, in seconds,
    /// east positive.
    utc_offset_seconds: i32,
    /// The tradition's table, every row data; the *Dharmasindhu*'s when
    /// left out.
    #[serde(default)]
    table: Option<AshauchTable>,
    /// A shipped table by its name, in place of `table`.
    #[serde(default)]
    tradition: Option<AshauchTradition>,
    /// The event, the observer and the periods already running.
    request: AshauchRequest,
}

/// A table the SDK ships, by its name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum AshauchTradition {
    /// [`AshauchTable::dharmasindhu`], the default.
    Dharmasindhu,
    /// [`AshauchTable::mitakshara`].
    Mitakshara,
}

impl AshauchQuery {
    /// The record a binding sends, as JSON: `latitudeDeg`, `longitudeDeg`,
    /// `altitudeM` (0 when left out) and `utcOffsetSeconds`; `table`, the
    /// *Dharmasindhu*'s when left out, or `tradition`, a shipped table by
    /// its name (`DHARMASINDHU`, `MITAKSHARA`); and `request`, its instants
    /// UTC Julian days.
    ///
    /// ```
    /// use teistro::AshauchQuery;
    /// use teistro::ashauch::Relation;
    ///
    /// let asked = AshauchQuery::from_json(
    ///     r#"{"latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1337,
    ///         "utcOffsetSeconds": 20700,
    ///         "request": {"key": "grandfather", "relation": "SAPINDA",
    ///                     "event": {"event": "DEATH", "at": 2461051.677083}}}"#,
    /// )?;
    /// assert_eq!(asked.request.relation, Relation::Sapinda);
    /// assert_eq!(asked.table.name, "DHARMASINDHU");
    /// let mitakshara = AshauchQuery::from_json(
    ///     r#"{"latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700,
    ///         "tradition": "MITAKSHARA",
    ///         "request": {"key": "grandfather", "relation": "SAPINDA",
    ///                     "event": {"event": "DEATH", "at": 2461051.677083}}}"#,
    /// )?;
    /// assert_eq!(mitakshara.table.name, "MITAKSHARA");
    /// let unread = AshauchQuery::from_json(
    ///     r#"{"latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700,
    ///         "request": {"key": "k", "relation": "SAPINDA", "caste": "any",
    ///                     "event": {"event": "DEATH", "at": 2461051.677083}}}"#,
    /// )
    /// .unwrap_err();
    /// assert_eq!(unread.field(), Some("ashauch.request.caste"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, a place or offset out of range, or a table or request that
    /// cannot be counted ([`AshauchTable::check`],
    /// [`AshauchRequest::check`]), each named under `ashauch`.
    pub fn from_json(text: &str) -> Result<AshauchQuery, Error> {
        let asked: AshauchAsked = teistro_core::strict::read(text, ASHAUCH)?;
        asked.query().map_err(|why| why.under(ASHAUCH))
    }
}

impl AshauchAsked {
    fn query(self) -> Result<AshauchQuery, Error> {
        let table = match (self.table, self.tradition) {
            (Some(_), Some(_)) => {
                return Err(Error::invalid_arg(
                    "an ashauch names a `table` or a `tradition`, not both",
                )
                .with_field("tradition"));
            }
            (Some(table), None) => table,
            (None, Some(AshauchTradition::Mitakshara)) => AshauchTable::mitakshara(),
            (None, Some(AshauchTradition::Dharmasindhu) | None) => AshauchTable::dharmasindhu(),
        };
        table.check().map_err(|why| why.under("table"))?;
        self.request.check().map_err(|why| why.under("request"))?;
        Ok(AshauchQuery {
            place: place_of(self.latitude_deg, self.longitude_deg, self.altitude_m)?,
            offset: offset_of(self.utc_offset_seconds)?,
            table,
            request: self.request,
        })
    }
}

impl AlmanacArea<'_> {
    /// As [`AlmanacArea::ashauch`] for a query a binding sent, every
    /// catalogue member written as its full key: what the boundary
    /// carries and seals. A refusal naming a part of the request
    /// (a fact a band needs, a relation the table names no row for) is
    /// named under `ashauch.request`, as the record spells it.
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::ashauch`].
    pub fn ashauch_query(self, asked: &AshauchQuery) -> Result<Envelope<serde_json::Value>, Error> {
        let Envelope { value, provenance } = self
            .ashauch(&asked.table, &asked.request, &asked.place, asked.offset)
            .map_err(|why| {
                if why.field().is_some_and(|field| !field.is_empty()) {
                    why.under("request").under(ASHAUCH)
                } else {
                    why
                }
            })?;
        Ok(Envelope::sealing(value.in_full()?, provenance))
    }
}
