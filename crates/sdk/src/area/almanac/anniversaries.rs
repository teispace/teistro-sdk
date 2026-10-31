//! Anniversaries kept by the tithi over a span of years, for many people
//! in one call (`03-design/anniversary-span.md`, SR-04, SR-24): the span's
//! days founded once, which every person's days are then read from,
//! rather than a festival request a year.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use teistro_calendar::CalendarDate;
use teistro_core::catalogue::Calendar;
use teistro_core::envelope::{Convention, Envelope, content_hash};
use teistro_core::error::{Error, Status};
use teistro_core::quantity::Place;
use teistro_core::time::UtcOffset;
use teistro_panchanga::festival::{Anniversary, Observances};

use super::{AlmanacArea, event_convention, without_printed};
use crate::area::system_of;
use crate::asked::{offset_of, place_of};

/// The field an anniversaries request's refusals are named under.
const ANNIVERSARIES: &str = "anniversaries";

/// The most years one request may span: a century, a lifetime's janma
/// tithis. Past it the refusal names `years`.
pub const MOST_YEARS: u16 = 100;

/// The most anniversaries one request may carry; past it the refusal
/// names `anniversaries`, and the rest go in a second request.
pub const MOST_ANNIVERSARIES: usize = 1_000;

/// Days founded before the span: a tithi begun the day before the first
/// day, and the lunation a first year's doubled month is read from, as a
/// festival range widens for an anniversary.
const BEFORE: i64 = 33;
/// Days founded after the span, for an occurrence at its end.
const AFTER: i64 = 2;

/// A span of years and the anniversaries asked over it, at one place.
///
/// ```
/// use teistro::AnniversariesRequest;
///
/// let asked = AnniversariesRequest::from_json(
///     r#"{"fromYear": 2026, "years": 10,
///         "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700,
///         "anniversaries": [{"key": "GRANDFATHER",
///             "origin": {"by": "LUNAR", "month": "KARTIKA", "tithi": "SHUKLA_DASHAMI"}}]}"#,
/// )?;
/// assert_eq!((asked.years, asked.anniversaries.len()), (10, 1));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct AnniversariesRequest {
    /// The first year, in the request's calendar.
    pub from_year: i32,
    /// How many years, the first among them.
    pub years: u16,
    /// The place every day is judged at.
    pub place: Place,
    /// The offset the civil days run at.
    pub offset: UtcOffset,
    /// The calendar the years are counted in and the days written in.
    pub calendar: Calendar,
    /// The anniversaries, each key once.
    pub anniversaries: Vec<Anniversary>,
}

/// The request as a binding writes it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "AnniversariesRequest"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AnniversariesAsked {
    /// The first year, in `calendar`: the span begins on its month 1, day 1.
    from_year: i32,
    /// 1 to 100: the span ends the day before month 1 of the year after.
    years: u16,
    latitude_deg: f64,
    longitude_deg: f64,
    #[serde(default)]
    altitude_m: f64,
    utc_offset_seconds: i32,
    #[serde(default)]
    calendar: Option<Calendar>,
    /// Up to 1000, each key once.
    anniversaries: Vec<Anniversary>,
}

impl AnniversariesRequest {
    /// The record a binding sends, as JSON: `fromYear` and `years`, 1 to
    /// 100, counted in `calendar` (Gregorian when left out), which the
    /// answer's days are written in too; `latitudeDeg`, `longitudeDeg`,
    /// `altitudeM` and `utcOffsetSeconds`; and `anniversaries`, each as a
    /// festival request writes one.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, a place or offset out of range, an anniversary its own check
    /// refuses or a key given twice; `OUT_OF_RANGE` for `years` of 0 or
    /// past [`MOST_YEARS`]; `LIMIT` for more anniversaries than
    /// [`MOST_ANNIVERSARIES`]; each named under `anniversaries`.
    pub fn from_json(text: &str) -> Result<AnniversariesRequest, Error> {
        let asked: AnniversariesAsked = teistro_core::strict::read(text, ANNIVERSARIES)?;
        asked.request().map_err(|why| why.under(ANNIVERSARIES))
    }
}

/// The size a request may be, read or built by hand.
fn check_size(years: u16, anniversaries: usize) -> Result<(), Error> {
    if !(1..=MOST_YEARS).contains(&years) {
        return Err(Error::new(
            Status::OutOfRange,
            format!("a request spans 1 to {MOST_YEARS} years, not {years}"),
        )
        .with_field("years"));
    }
    if anniversaries > MOST_ANNIVERSARIES {
        return Err(Error::limit(format!(
            "a request carries at most {MOST_ANNIVERSARIES} anniversaries, not {anniversaries}"
        ))
        .with_field("anniversaries")
        .with_hint("send the rest in a second request"));
    }
    Ok(())
}

impl AnniversariesAsked {
    fn request(self) -> Result<AnniversariesRequest, Error> {
        check_size(self.years, self.anniversaries.len())?;
        let mut seen = BTreeSet::new();
        for (index, anniversary) in self.anniversaries.iter().enumerate() {
            let at = format!("anniversaries[{index}]");
            anniversary.check().map_err(|why| why.under(&at))?;
            if !seen.insert(&anniversary.key) {
                return Err(Error::invalid_arg(format!(
                    "`{}` is given twice, which would answer days a reader cannot tell apart",
                    anniversary.key
                ))
                .with_field(format!("{at}.key")));
            }
        }
        Ok(AnniversariesRequest {
            from_year: self.from_year,
            years: self.years,
            place: place_of(self.latitude_deg, self.longitude_deg, self.altitude_m)?,
            offset: offset_of(self.utc_offset_seconds)?,
            calendar: self.calendar.unwrap_or(Calendar::Gregorian),
            anniversaries: self.anniversaries,
        })
    }
}

impl AlmanacArea<'_> {
    /// Every asked anniversary's days over the span of years, as a festival
    /// answer holds them, each naming the clause that chose it and whether
    /// its tithi was kshaya or vriddhi (`03-design/anniversary-span.md`).
    /// The span's days are founded once and every person's are read from
    /// them, so fifty people cost little more than one. A year holds one
    /// day for most people, none or two for some: a lunar year is short of
    /// a solar one, and a samsarpa keeps a second.
    ///
    /// ```no_run
    /// use teistro::{AnniversariesRequest, Context};
    ///
    /// let sdk = Context::builder().profile("nepali-committee").build()?;
    /// let asked = AnniversariesRequest::from_json(
    ///     r#"{"fromYear": 2026, "years": 10,
    ///         "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700,
    ///         "anniversaries": [{"key": "GRANDFATHER",
    ///             "origin": {"by": "LUNAR", "month": "KARTIKA", "tithi": "SHUKLA_DASHAMI"}}]}"#,
    /// )?;
    /// for kept in &sdk.almanac().anniversaries(&asked)?.value.anniversaries {
    ///     println!("{} {:?}", kept.observance.day, kept.day_rule.clause);
    /// }
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// An origin the almanac refuses, named `anniversaries[i].origin`; a
    /// context with no ephemeris; a calendar the SDK does not ship; a span
    /// the ephemeris does not reach; and, should the request have been
    /// built by hand, a size past the limits or an anniversary its own
    /// check refuses.
    pub fn anniversaries(
        self,
        request: &AnniversariesRequest,
    ) -> Result<Envelope<Observances>, Error> {
        check_size(request.years, request.anniversaries.len())
            .map_err(|why| why.under(ANNIVERSARIES))?;
        let (calendar, offset, place) = (request.calendar, request.offset, &request.place);
        let system = system_of(calendar)?;
        let first_of = |year| system.fixed_of(&CalendarDate::defined(calendar, year, 1, 1));
        let first = first_of(request.from_year)?;
        let last =
            first_of(request.from_year.saturating_add(i32::from(request.years)))?.plus_days(-1);
        let (from, to) = (
            system.date_of(first.plus_days(-BEFORE))?,
            system.date_of(last.plus_days(AFTER))?,
        );
        let Envelope {
            value: viewed,
            mut provenance,
        } = self.with_almanac(calendar, offset, |almanac| {
            almanac.festival_span(&from, &to, place)
        })?;
        without_printed(&mut provenance);
        let mut found = self.kept_anniversaries(
            calendar,
            offset,
            place,
            (&request.anniversaries, ANNIVERSARIES),
            &viewed,
        )?;
        found.anniversaries.retain(|kept| {
            system
                .fixed_of(&kept.observance.day)
                .is_ok_and(|day| first.days_until(day) >= 0 && day.days_until(last) >= 0)
        });
        let (start, end) = (
            viewed.iter().find(|day| {
                system
                    .fixed_of(&day.date)
                    .is_ok_and(|fixed| first.days_until(fixed) >= 0)
            }),
            viewed.iter().rev().find(|day| {
                system
                    .fixed_of(&day.date)
                    .is_ok_and(|fixed| fixed.days_until(last) >= 0)
            }),
        );
        found.unjudged.retain(|unjudged| {
            start.is_some_and(|day| unjudged.tithi.to.get() > day.sunrise.get())
                && end.is_some_and(|day| unjudged.tithi.from.get() < day.next_sunrise.get())
        });
        provenance.input_hash = content_hash(&serde_json::json!({
            "fromYear": request.from_year,
            "years": request.years,
            "place": request.place,
            "utcOffsetSeconds": request.offset.seconds(),
            "calendar": request.calendar,
            "anniversaries": request.anniversaries,
        }));
        event_convention(&mut provenance, &request.anniversaries);
        provenance.applied_conventions.push(Convention {
            knob: String::from("anniversary.span"),
            value: format!("{from}..{to}"),
            reason: String::from(
                "the span's days were founded once, a lunation before it and two days after, as a festival range widens for an anniversary; every person's days were read from them",
            ),
        });
        Ok(Envelope::sealing(found, provenance))
    }

    /// As [`AlmanacArea::anniversaries`], the answer written in full and
    /// sealed over what is written: what the boundary carries.
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::anniversaries`].
    pub fn anniversaries_in_full(
        self,
        request: &AnniversariesRequest,
    ) -> Result<Envelope<Value>, Error> {
        let Envelope { value, provenance } = self.anniversaries(request)?;
        Ok(Envelope::sealing(value.in_full()?, provenance))
    }
}
