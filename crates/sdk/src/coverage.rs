//! What the SDK answers over, as data (SR-136, `03-design/coverage-edges.md`
//! §5): the civil days the configured ephemeris covers, the Bikram Sambat
//! years the shipped table holds and those the government published, and
//! the years each shipped holiday set lists.
//!
//! A consumer clamps its date pickers to these rather than hard-coding a
//! range and learning the real one from a refusal (C2250).

use serde::Serialize;
use teistro_calendar::{BikramSambat, CalendarDay, CalendarSystem, FixedDay, Gregorian};
use teistro_core::error::Error;
use teistro_core::quantity::JulianDay;
use teistro_panchanga::festival::HolidaySet;

use crate::Context;

/// What the SDK answers over.
///
/// ```
/// use teistro::{Context, Ephemeris};
///
/// let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
/// let coverage = sdk.coverage()?;
/// let span = coverage.ephemeris.ok_or("an ephemeris")?;
/// assert_eq!((span.first.year, span.first.month, span.first.day), (1800, 1, 1));
/// assert_eq!((span.last.year, span.last.month, span.last.day), (2399, 12, 31));
/// assert!(coverage.bikram_sambat.official_first_year <= 2083);
/// assert_eq!(coverage.holidays[0].set, "NEPAL_GAZETTE");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Coverage {
    /// The days the configured ephemeris covers, as the profile's almanac
    /// reads it; absent for a context with no ephemeris, or one whose
    /// ephemeris states no bounds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ephemeris: Option<EphemerisSpan>,
    /// The Bikram Sambat years the shipped table holds.
    pub bikram_sambat: BikramSambatSpan,
    /// Each shipped holiday set and the Bikram Sambat years it lists.
    pub holidays: Vec<HolidayYears>,
}

/// The instants and the civil days an ephemeris covers.
///
/// The span is stated on UT1, so its ends are not midnights: the built-in
/// ephemeris's opens 14 seconds before 0 h UT on 1 January 1800 under the
/// drik profile and an hour after it under the official one. `first` and
/// `last` are the first and last Gregorian dates the span holds at least
/// half of, on UT. A day is answered whenever its sunrise is inside the
/// span: so every date from `first` to `last` is, at a place whose sunrise
/// on `first` falls after `fromJd` (everywhere west of about 97° E on 1
/// January 1800 over the built-in ephemeris, C2244).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct EphemerisSpan {
    /// The first instant answered, a Julian day on UT1.
    pub from_jd: f64,
    /// The last instant answered.
    pub to_jd: f64,
    /// The first Gregorian date, on UT, the span holds at least half of.
    pub first: CalendarDay,
    /// The last.
    pub last: CalendarDay,
}

/// The Bikram Sambat years the shipped table holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct BikramSambatSpan {
    /// The table's first year.
    pub first_year: i32,
    /// Its last.
    pub last_year: i32,
    /// The first year the government's calendar committee published.
    pub official_first_year: i32,
    /// The last.
    pub official_last_year: i32,
}

/// A shipped holiday set and the years it lists.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct HolidayYears {
    /// The set's key, as a festival request names it.
    pub set: String,
    /// The Bikram Sambat years it lists, in order.
    pub years: Vec<i32>,
}

impl Context {
    /// What this context answers over (SR-136, C2250): the civil days its
    /// ephemeris covers under its profile, the Bikram Sambat table's
    /// years, official and computed, and each holiday set's years.
    ///
    /// # Errors
    ///
    /// None for a shipped ephemeris; a date the Gregorian calendar cannot
    /// write, which an ephemeris's span cannot reach.
    pub fn coverage(&self) -> Result<Coverage, Error> {
        let ephemeris = match self.ephemeris() {
            Some(_) => {
                let span = self
                    .almanac()
                    .with_utc_almanac(|almanac| Ok(almanac.coverage()))?;
                if span.from.is_finite() && span.to.is_finite() {
                    Some(EphemerisSpan {
                        from_jd: span.from,
                        to_jd: span.to,
                        first: utc_date(span.from + HALF_DAY)?,
                        last: utc_date(span.to - HALF_DAY)?,
                    })
                } else {
                    None
                }
            }
            None => None,
        };
        let shipped = BikramSambat::shipped();
        let ((first_year, last_year), (official_first_year, official_last_year)) =
            (shipped.years(), shipped.official_years());
        Ok(Coverage {
            ephemeris,
            bikram_sambat: BikramSambatSpan {
                first_year,
                last_year,
                official_first_year,
                official_last_year,
            },
            holidays: HolidaySet::ALL
                .iter()
                .map(|set| HolidayYears {
                    set: set.key().to_owned(),
                    years: vec![set.year()],
                })
                .collect(),
        })
    }
}

/// Half a day: a date the span holds at least half of has its UT midnight
/// within half a day inside an end.
const HALF_DAY: f64 = 0.5;

/// The Gregorian date, on UT, of a Julian day.
fn utc_date(jd: f64) -> Result<CalendarDay, Error> {
    let (day, _) = FixedDay::from_jd(JulianDay::try_new(jd)?);
    Ok(CalendarDay::from(&Gregorian.date_of(day)?))
}
