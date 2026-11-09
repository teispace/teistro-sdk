//! A death's first year, as a binding asks it (`03-design/shraddha.md`
//! §3.3): the place and the clock its days are found at, and the request.

use serde::{Deserialize, Serialize};
use teistro_core::envelope::Envelope;
use teistro_core::error::Error;
use teistro_core::quantity::Place;
use teistro_panchanga::first_year::FirstYear;
use teistro_time::ZoneSpec;

use crate::asked::place_of;
use crate::{AlmanacArea, DayClock};

/// The record's name where a binding sends it, which a refusal is named
/// under.
const FIRST_YEAR: &str = "firstYear";

/// A first year's place, clock and request, as a binding asks it.
#[derive(Clone, Debug, PartialEq)]
pub struct FirstYearQuery {
    /// The place whose sunrises count the days.
    pub place: Place,
    /// The clock its civil days run on: an offset stated, or a zone each
    /// instant is read on.
    pub clock: DayClock,
    /// The death and the readings.
    pub request: FirstYear,
}

/// The query as a binding writes it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "FirstYearQuery"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct FirstYearAsked {
    /// The place's latitude, degrees north positive.
    latitude_deg: f64,
    /// The place's longitude, degrees east positive.
    longitude_deg: f64,
    /// The place's height above sea level in metres, 0 when left out.
    #[serde(default)]
    altitude_m: f64,
    /// The offset from UTC the place's civil days run at, in seconds,
    /// east positive; or `zone`, not both.
    #[serde(default)]
    utc_offset_seconds: Option<i32>,
    /// The zone each instant is read on, at its offset then; or
    /// `utcOffsetSeconds`, not both.
    #[serde(default)]
    zone: Option<ZoneSpec>,
    /// The death, a UTC Julian day, and the readings.
    request: FirstYear,
}

impl FirstYearQuery {
    /// The record a binding sends, as JSON: `latitudeDeg`, `longitudeDeg`,
    /// `altitudeM` (0 when left out), `utcOffsetSeconds` or a `zone`, and
    /// `request`,
    /// whose `at` is a UTC Julian day and whose readings are the
    /// *Dharmasindhu*'s when left out.
    ///
    /// ```
    /// use teistro::FirstYearQuery;
    /// use teistro::first_year::SapindanaAt;
    ///
    /// let asked = FirstYearQuery::from_json(
    ///     r#"{"latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1337,
    ///         "utcOffsetSeconds": 20700,
    ///         "request": {"key": "GRANDFATHER", "at": 2461051.677083}}"#,
    /// )?;
    /// assert_eq!(asked.request.sapindana, SapindanaAt::TwelfthDay);
    /// let unread = FirstYearQuery::from_json(
    ///     r#"{"latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700,
    ///         "request": {"key": "GRANDFATHER", "at": 2461051.677083, "caste": "any"}}"#,
    /// )
    /// .unwrap_err();
    /// assert_eq!(unread.field(), Some("firstYear.request.caste"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, a place or offset out of range, both or neither of
    /// `utcOffsetSeconds` and `zone`, or a request [`FirstYear::check`]
    /// refuses, each named under `firstYear`.
    pub fn from_json(text: &str) -> Result<FirstYearQuery, Error> {
        let asked: FirstYearAsked = teistro_core::strict::read(text, FIRST_YEAR)?;
        asked.query().map_err(|why| why.under(FIRST_YEAR))
    }
}

impl FirstYearAsked {
    fn query(self) -> Result<FirstYearQuery, Error> {
        self.request.check().map_err(|why| why.under("request"))?;
        Ok(FirstYearQuery {
            place: place_of(self.latitude_deg, self.longitude_deg, self.altitude_m)?,
            clock: DayClock::asked(self.utc_offset_seconds, self.zone)?,
            request: self.request,
        })
    }
}

impl AlmanacArea<'_> {
    /// As [`AlmanacArea::first_year`] for a query a binding sent, every
    /// catalogue member written as its full key: what the boundary
    /// carries and seals.
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::first_year`]; a zone the database does not have.
    pub fn first_year_query(
        self,
        asked: &FirstYearQuery,
    ) -> Result<Envelope<serde_json::Value>, Error> {
        let Envelope { value, provenance } = self.clocked(
            &asked.clock,
            || Ok(asked.request.at),
            |almanac, offset| almanac.first_year(&asked.request, &asked.place, offset),
        )?;
        Ok(Envelope::sealing(value.in_full()?, provenance))
    }
}
