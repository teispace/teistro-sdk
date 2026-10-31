//! An ashauch, as a binding asks it (`03-design/ashauch.md`): the place
//! and the clock its days are found at, the table (the
//! *Dharmasindhu*'s when left out), and the request itself.

use serde::{Deserialize, Serialize};
use teistro_core::envelope::Envelope;
use teistro_core::error::Error;
use teistro_core::quantity::Place;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::time::UtcOffset;
use teistro_panchanga::ashauch::{
    AshauchRequest, AshauchTable, Household, Kin, KinAsked, KinTable, RiteAnswer, RiteAsked,
    RiteTable,
};
use teistro_panchanga::death_time::DeathTimeTable;

use crate::AlmanacArea;
use crate::asked::{offset_of, place_of};

/// The record's name where a binding sends it, which a refusal is named
/// under.
const ASHAUCH: &str = "ashauch";
/// As [`ASHAUCH`], for a household.
const HOUSEHOLD: &str = "household";
/// As [`ASHAUCH`], for a relation path.
const KIN: &str = "kin";
/// As [`ASHAUCH`], for a rite.
const RITE: &str = "rite";
/// As [`ASHAUCH`], for a death-time check.
const DEATH_TIME: &str = "deathTime";

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

/// The table `table` or `tradition` names, the *Dharmasindhu*'s when
/// both are left out, and checked.
fn table_of(
    table: Option<AshauchTable>,
    tradition: Option<AshauchTradition>,
) -> Result<AshauchTable, Error> {
    let table = match (table, tradition) {
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
    Ok(table)
}

impl AshauchAsked {
    fn query(self) -> Result<AshauchQuery, Error> {
        let table = table_of(self.table, self.tradition)?;
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

/// Names a façade's refusal under its record, as a binding spells it.
fn named_under(why: Error, root: &str) -> Error {
    if why.field().is_some_and(|field| !field.is_empty()) {
        why.under(root)
    } else {
        why
    }
}

/// A household's place, clock, table and ties, as a binding asks them
/// (`03-design/ashauch.md` §11).
#[derive(Clone, Debug, PartialEq)]
pub struct HouseholdQuery {
    /// The place whose sunrises count the days.
    pub place: Place,
    /// The offset its civil days run at.
    pub offset: UtcOffset,
    /// The tradition's table.
    pub table: AshauchTable,
    /// The events and observers.
    pub household: Household,
}

/// The query as a binding writes it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "HouseholdQuery"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct HouseholdAsked {
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
    /// The events and observers.
    household: Household,
}

impl HouseholdQuery {
    /// The record a binding sends, as JSON: the place and clock as an
    /// ashauch's, `table` or `tradition`, and `household`.
    ///
    /// ```
    /// use teistro::HouseholdQuery;
    ///
    /// let asked = HouseholdQuery::from_json(
    ///     r#"{"latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700,
    ///         "household": {
    ///           "events": [{"key": "grandfather", "event": {"event": "DEATH", "at": 2461051.677083}}],
    ///           "observers": [{"key": "son", "ties": [{"event": "grandfather", "relation": "CHILD"}]}]}}"#,
    /// )?;
    /// assert_eq!(asked.household.observers.len(), 1);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a place or offset
    /// out of range, or a household that does not hold together, each
    /// named under `household`.
    pub fn from_json(text: &str) -> Result<HouseholdQuery, Error> {
        let asked: HouseholdAsked = teistro_core::strict::read(text, HOUSEHOLD)?;
        asked.query().map_err(|why| why.under(HOUSEHOLD))
    }
}

impl HouseholdAsked {
    fn query(self) -> Result<HouseholdQuery, Error> {
        let table = table_of(self.table, self.tradition)?;
        self.household.check().map_err(|why| why.under(HOUSEHOLD))?;
        Ok(HouseholdQuery {
            place: place_of(self.latitude_deg, self.longitude_deg, self.altitude_m)?,
            offset: offset_of(self.utc_offset_seconds)?,
            table,
            household: self.household,
        })
    }
}

/// A relation path and the table it is read under, as a binding asks it
/// (`03-design/ashauch.md` §10).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KinQuery {
    /// The table, every row data; the *Dharmasindhu*'s when left out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub table: Option<KinTable>,
    /// The path and the facts it reads.
    pub kin: KinAsked,
}

impl KinQuery {
    /// The record a binding sends, as JSON.
    ///
    /// ```
    /// use teistro::KinQuery;
    ///
    /// let asked = KinQuery::from_json(
    ///     r#"{"kin": {"path": ["FATHER", "SON"], "observer": {"sex": "MALE"}}}"#,
    /// )?;
    /// assert_eq!(asked.kin.path.len(), 2);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, named under `kin`.
    pub fn from_json(text: &str) -> Result<KinQuery, Error> {
        teistro_core::strict::read(text, KIN)
    }

    /// The relation and degree the path resolves to.
    ///
    /// ```
    /// use teistro::KinQuery;
    /// use teistro::ashauch::Relation;
    ///
    /// let asked = KinQuery::from_json(
    ///     r#"{"kin": {"path": ["FATHER", "SON"], "observer": {"sex": "MALE"}}}"#,
    /// )?;
    /// assert_eq!(asked.answer()?.relation, Some(Relation::Sapinda));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`teistro_panchanga::ashauch::resolve`], named under `kin`.
    pub fn answer(&self) -> Result<Kin, Error> {
        let table = self.table.clone().unwrap_or_else(KinTable::dharmasindhu);
        teistro_panchanga::ashauch::resolve(&table, &self.kin)
            .map_err(|why| named_under(why, "kin.kin"))
    }
}

/// A rite and the table it is read under, as a binding asks it
/// (`03-design/ashauch.md` §12).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RiteQuery {
    /// The table, every row data; the *Dharmasindhu*'s when left out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub table: Option<RiteTable>,
    /// The rite and the facts it reads.
    pub rite: RiteAsked,
}

impl RiteQuery {
    /// The record a binding sends, as JSON.
    ///
    /// ```
    /// use teistro::RiteQuery;
    /// use teistro::ashauch::Rite;
    ///
    /// let asked = RiteQuery::from_json(r#"{"rite": {"rite": "MARRIAGE", "begun": true}}"#)?;
    /// assert_eq!(asked.rite.rite, Rite::Marriage);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, named under `rite`.
    pub fn from_json(text: &str) -> Result<RiteQuery, Error> {
        teistro_core::strict::read(text, RITE)
    }

    /// Whether the rite goes on, and by which clause.
    ///
    /// ```
    /// use teistro::RiteQuery;
    /// use teistro::ashauch::RiteVerdict;
    ///
    /// let asked = RiteQuery::from_json(r#"{"rite": {"rite": "DAILY_RULES"}}"#)?;
    /// assert_eq!(asked.answer()?.verdict, RiteVerdict::GoesOn);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`teistro_panchanga::ashauch::rite`], named under `rite`.
    pub fn answer(&self) -> Result<RiteAnswer, Error> {
        let table = self.table.clone().unwrap_or_else(RiteTable::dharmasindhu);
        teistro_panchanga::ashauch::rite(&table, &self.rite)
            .map_err(|why| named_under(why, "rite.rite"))
    }
}

/// A death-time check, as a binding asks it (`03-design/death-time.md`).
#[derive(Clone, Debug, PartialEq)]
pub struct DeathTimeQuery {
    /// The place, whose day and sky are read.
    pub place: Place,
    /// The offset its civil days run at.
    pub offset: UtcOffset,
    /// The death's instant.
    pub at: JulianDay<Utc>,
    /// The cremation's instant.
    pub cremated_at: Option<JulianDay<Utc>>,
    /// The tradition's lists.
    pub table: DeathTimeTable,
}

/// The query as a binding writes it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "DeathTimeQuery"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct DeathTimeAsked {
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
    /// The death's instant, a UTC Julian day.
    at: JulianDay<Utc>,
    /// The cremation's instant, a UTC Julian day.
    #[serde(default)]
    cremated_at: Option<JulianDay<Utc>>,
    /// The tradition's lists; the *Dharmasindhu*'s when left out.
    #[serde(default)]
    table: Option<DeathTimeTable>,
}

impl DeathTimeQuery {
    /// The record a binding sends, as JSON.
    ///
    /// ```
    /// use teistro::DeathTimeQuery;
    ///
    /// let asked = DeathTimeQuery::from_json(
    ///     r#"{"latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700,
    ///         "at": 2461051.677083}"#,
    /// )?;
    /// assert_eq!(asked.table.name, "DHARMASINDHU");
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a place or offset
    /// out of range, or a cremation before the death, named under
    /// `deathTime`.
    pub fn from_json(text: &str) -> Result<DeathTimeQuery, Error> {
        let asked: DeathTimeAsked = teistro_core::strict::read(text, DEATH_TIME)?;
        if asked
            .cremated_at
            .is_some_and(|at| at.get() < asked.at.get())
        {
            return Err(Error::invalid_arg("a cremation follows the death")
                .with_field("crematedAt")
                .under(DEATH_TIME));
        }
        let located = place_of(asked.latitude_deg, asked.longitude_deg, asked.altitude_m)
            .and_then(|place| Ok((place, offset_of(asked.utc_offset_seconds)?)))
            .map_err(|why| why.under(DEATH_TIME))?;
        Ok(DeathTimeQuery {
            place: located.0,
            offset: located.1,
            at: asked.at,
            cremated_at: asked.cremated_at,
            table: asked.table.unwrap_or_else(DeathTimeTable::dharmasindhu),
        })
    }
}

impl AlmanacArea<'_> {
    /// As [`AlmanacArea::household`] for a query a binding sent, every
    /// catalogue member written as its full key.
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::household`], named under `household`.
    pub fn household_query(
        self,
        asked: &HouseholdQuery,
    ) -> Result<Envelope<serde_json::Value>, Error> {
        let Envelope { value, provenance } = self
            .household(&asked.table, &asked.household, &asked.place, asked.offset)
            .map_err(|why| named_under(why, "household.household"))?;
        Ok(Envelope::sealing(value.in_full()?, provenance))
    }

    /// As [`AlmanacArea::death_time`] for a query a binding sent, every
    /// catalogue member written as its full key.
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::death_time`].
    pub fn death_time_query(
        self,
        asked: &DeathTimeQuery,
    ) -> Result<Envelope<serde_json::Value>, Error> {
        let Envelope { value, provenance } = self.death_time(
            &asked.table,
            asked.at,
            asked.cremated_at,
            &asked.place,
            asked.offset,
        )?;
        Ok(Envelope::sealing(value.in_full()?, provenance))
    }
}
