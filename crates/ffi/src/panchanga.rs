//! The daily panchanga at the C boundary: a batch of almanacs as the
//! blob its schema describes, and the three enums it carries that no
//! other entry point needed.
//!
//! Designed in `03-design/chart-at-the-boundary.md` §5 and laid out from
//! `03-design/panchanga-at-the-boundary-measured.md`, which measured the
//! one thing a chart blob never had to decide: **a day's lists are
//! ragged**. Ten of the fifteen vary in length, and a rectangular layout
//! wastes 78.1% of its rows once a single polar day joins a batch,
//! because that day sets the stride for every other. So every list is
//! concatenated across the batch and the `counts` section says how many
//! rows are each day's — one rule for all thirteen, since two layouts in
//! one blob is two things for a reader to learn.
//!
#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]
//!
//! Three enums are declared here rather than taken from the catalogue.
//! `TsLunarMonth` is a settings knob, so it converts **fallibly** over
//! the knob's own `ALL` for the reason `chart.rs` gives: a knob is
//! `#[non_exhaustive]`, so a match on one needs a `_` arm and a member
//! added later would fall into it silently. `TsMoonEvent` and
//! `TsYogaCause` are this boundary's own: the first names which of two
//! lists a merged section's row came from, the second is the kind half of
//! a tagged enum whose payload fields sit beside it.

use teistro_astro::precession::PrecessionModel;
use teistro_calendar::CalendarDate;
use teistro_calendar::lunisolar::MonthKind;
use teistro_calendar::shipped;
use teistro_calendar::solar::drik::DrikSun;
use teistro_core::catalogue::{Ayanamsha, Calendar};
use teistro_core::envelope::{Provenance, content_hash};
use teistro_core::error::{Error, Status};
use teistro_core::interval::Interval;
use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
use teistro_core::settings::{AyanamshaChoice, LunarMonth};
use teistro_core::time::UtcOffset;
use teistro_idl::blob::{FixedValue, Writer};
use teistro_panchanga::almanac::{Almanac, Panchanga};
use teistro_panchanga::span::Span;

use crate::blob::TsBlob;
use crate::context::TsContext;
use crate::support::{with_context, write_plain};

/// Which lunar-month convention a day's month leads with.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsLunarMonth {
    /// New moon to new moon.
    Amanta = 0,
    /// Full moon to full moon.
    Purnimanta = 1,
}

impl TsLunarMonth {
    /// The id this build gives a member, or `None` for one it does not
    /// know — a member added to the knob since this was written, which
    /// is refused by name rather than defaulted.
    #[must_use]
    pub fn of(month: LunarMonth) -> Option<TsLunarMonth> {
        match month {
            LunarMonth::Amanta => Some(TsLunarMonth::Amanta),
            LunarMonth::Purnimanta => Some(TsLunarMonth::Purnimanta),
            _ => None,
        }
    }
}

/// Whether the Moon rose or set.
///
/// The two lists are one section with this to tell them apart, as the
/// muhurtas' `daylight` and the choghadiya's `daytime` do: a day's moon
/// events are one question asked twice, and two sections of one column
/// each would be two counts, two offsets and two types for it.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsMoonEvent {
    /// The Moon crossed the horizon upward.
    Rise = 0,
    /// The Moon crossed it downward.
    Set = 1,
}

/// What made a muhurta yoga hold: the kind half of a tagged enum.
///
/// The payload fields sit beside it, as many as the widest variant needs
/// (`03-design/chart-at-the-boundary.md` §8). Both variants carry a vara
/// and a nakshatra; only the second carries a tithi, and the first leaves
/// `because_tithi` at zero.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsYogaCause {
    /// The vara and the nakshatra the Moon was in.
    VaraNakshatra = 0,
    /// The vara, the tithi's class and the nakshatra's number of feet.
    VaraTithiNakshatra = 1,
}

/// Whether a lunar month is ordinary, intercalary or omitted.
///
/// The Indian lunisolar calendar decides it from the count of sankrantis
/// between the month's two new moons — none is adhika, one ordinary, two
/// kshaya (`03-design/calendar-indian-lunisolar.md` §2). An exhaustive
/// match, so a kind added to the calendar breaks this build rather than
/// silently crossing as whatever came first.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsMonthKind {
    /// One sankranti: the ordinary month.
    Nija = 0,
    /// None: intercalary, and the name repeats.
    Adhika = 1,
    /// Two: the next name is skipped this year.
    Kshaya = 2,
}

impl From<MonthKind> for TsMonthKind {
    fn from(kind: MonthKind) -> TsMonthKind {
        match kind {
            MonthKind::Nija => TsMonthKind::Nija,
            MonthKind::Adhika => TsMonthKind::Adhika,
            MonthKind::Kshaya => TsMonthKind::Kshaya,
        }
    }
}

/// What to found an almanac over: a range of dates at one place.
///
/// A **range**, not a grid of dates, because that is the shape the
/// almanac itself leads with and the one that is cheaper than its parts:
/// consecutive windows share a boundary, so day *n*'s next sunrise is day
/// *n+1*'s sunrise (`03-design/panchanga-day.md` §14). A caller wanting
/// one day passes a range of one.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TsPanchangaRequest {
    /// `sizeof(ts_panchanga_request)` as the caller compiled it.
    pub struct_size: u32,
    /// The calendar the range's dates are written in.
    /// `api: enum=Calendar example=0`
    pub calendar: u16,
    /// Reserved; write zero.
    pub reserved: u16,
    /// The first day's astronomical year.
    /// `api: example=2026`
    pub from_year: i32,
    /// The first day's month, 1-based.
    /// `api: range=[1,13] example=9`
    pub from_month: u8,
    /// The first day's day of the month, 1-based.
    /// `api: range=[1,32] example=1`
    pub from_day: u8,
    /// The last day's month, 1-based.
    /// `api: range=[1,13] example=9`
    pub to_month: u8,
    /// The last day's day of the month, 1-based.
    /// `api: range=[1,32] example=30`
    pub to_day: u8,
    /// The last day's astronomical year.
    /// `api: example=2026`
    pub to_year: i32,
    /// The place's latitude, degrees north.
    /// `api: unit=deg range=[-90,90] example=27.7172`
    pub latitude_deg: f64,
    /// The place's longitude, degrees east.
    /// `api: unit=deg range=[-180,180] example=85.324`
    pub longitude_deg: f64,
    /// The place's altitude, metres above the ellipsoid.
    /// `api: unit=m range=[-500,9000] example=1400`
    pub altitude_m: f64,
    /// The local clock's offset from UTC in seconds, east positive: the
    /// clock the days' dates are read in.
    /// `api: unit=s range=[-64800,64800] example=20700`
    pub utc_offset_seconds: i32,
    /// Reserved; write zero.
    pub reserved_tail: i32,
}

/// A day's own values, in the order `days` declares them.
#[must_use]
fn day_row(day: &Panchanga) -> Vec<FixedValue> {
    let mut row = interval(day.window);
    row.extend([
        u64::from(day.month.month.id()).into(),
        u64::from(day.month.amanta.id()).into(),
        u64::from(day.month.purnimanta.id()).into(),
        u64::from(day.month.paksha.id()).into(),
        (TsMonthKind::from(day.month.kind) as u64).into(),
        u64::from(day.sun.ayana.id()).into(),
        u64::from(day.omens.disha_shool.id()).into(),
        flag(day.sun.sankranti.is_some()),
        day.sun
            .sankranti
            .map_or(0.0, teistro_core::quantity::JulianDay::get)
            .into(),
        flag(day.muhurtas.abhijit.is_some()),
    ]);
    row.extend(interval(day.muhurtas.abhijit.unwrap_or(EMPTY)));
    row.extend([
        flag(day.muhurtas.abhijit_effective),
        flag(day.muhurtas.brahma.is_some()),
    ]);
    row.extend(interval(day.muhurtas.brahma.unwrap_or(EMPTY)));
    row.extend(interval(day.moon.window));
    row
}

/// How many rows of each per-day section belong to a day, in the order
/// `counts` declares them.
#[must_use]
fn count_row(day: &Panchanga) -> Vec<FixedValue> {
    [
        day.limbs.tithi.len(),
        day.limbs.nakshatra.len(),
        day.limbs.yoga.len(),
        day.limbs.karana.len(),
        day.omens.panchaka.len(),
        day.moon.signs.len(),
        day.sun.signs.len(),
        day.kaalas.len(),
        day.choghadiya.len(),
        day.horas.len(),
        day.muhurtas.daylight.len() + day.muhurtas.night.len(),
        day.moon.rises.len() + day.moon.sets.len(),
        day.omens.yogas.len(),
    ]
    .into_iter()
    .map(|count| u64::try_from(count).unwrap_or(u64::MAX).into())
    .collect()
}

/// An interval that stands for an absent one, beside a presence flag
/// that is what actually says so.
const EMPTY: Interval = Interval {
    from: teistro_core::quantity::JulianDay::literal(0.0),
    to: teistro_core::quantity::JulianDay::literal(0.0),
};

/// An interval as the two columns every section writes it in.
#[must_use]
fn interval(at: Interval) -> Vec<FixedValue> {
    vec![at.from.get().into(), at.to.get().into()]
}

/// A boolean as the byte a column carries, since the format has no
/// boolean of its own.
#[must_use]
fn flag(yes: bool) -> FixedValue {
    u64::from(yes).into()
}

/// One list's spans across the batch, days outermost.
///
/// Seven of the panchanga's lists are a `Span<T>` of some catalogue, so
/// this is written once and the member's id is the only thing that
/// differs.
#[must_use]
fn span_rows<T>(
    days: &[Panchanga],
    list: impl Fn(&Panchanga) -> &[Span<T>],
    id: impl Fn(&T) -> u16,
) -> Vec<Vec<FixedValue>> {
    days.iter()
        .flat_map(|day| list(day).iter())
        .map(|span| {
            let mut row = vec![u64::from(id(&span.member)).into()];
            row.extend(interval(span.whole));
            row.extend(interval(span.inside));
            row
        })
        .collect()
}

/// The seven lists that are a span of a catalogue's members.
type Spans = (
    Vec<Vec<FixedValue>>,
    Vec<Vec<FixedValue>>,
    Vec<Vec<FixedValue>>,
    Vec<Vec<FixedValue>>,
    Vec<Vec<FixedValue>>,
    Vec<Vec<FixedValue>>,
    Vec<Vec<FixedValue>>,
);

/// The six lists that are a period or an event.
type Periods = (
    Vec<Vec<FixedValue>>,
    Vec<Vec<FixedValue>>,
    Vec<Vec<FixedValue>>,
    Vec<Vec<FixedValue>>,
    Vec<Vec<FixedValue>>,
    Vec<Vec<FixedValue>>,
);

/// Every span list, concatenated days outermost.
fn spans(days: &[Panchanga]) -> Spans {
    let tithi = span_rows(days, |day| &day.limbs.tithi, |m| m.id());
    let nakshatra = span_rows(days, |day| &day.limbs.nakshatra, |m| m.id());
    let yoga = span_rows(days, |day| &day.limbs.yoga, |m| m.id());
    let karana = span_rows(days, |day| &day.limbs.karana, |m| m.id());
    let panchaka = span_rows(days, |day| &day.omens.panchaka, |m| m.id());
    let moon_signs = span_rows(days, |day| &day.moon.signs, |m| m.id());
    let sun_signs = span_rows(days, |day| &day.sun.signs, |m| m.id());

    (
        tithi, nakshatra, yoga, karana, panchaka, moon_signs, sun_signs,
    )
}

/// Every period and event list, concatenated days outermost.
fn periods(days: &[Panchanga]) -> Periods {
    let kaalas: Vec<Vec<FixedValue>> = days
        .iter()
        .flat_map(|day| day.kaalas.iter())
        .map(|kaala| {
            let mut row = vec![u64::from(kaala.kaala.id()).into()];
            row.extend(interval(kaala.at));
            row
        })
        .collect();
    let choghadiya: Vec<Vec<FixedValue>> = days
        .iter()
        .flat_map(|day| day.choghadiya.iter())
        .map(|part| {
            let mut row = vec![
                u64::from(part.choghadiya.id()).into(),
                u64::from(part.lord.id()).into(),
            ];
            row.extend(interval(part.at));
            row.push(flag(part.is_daytime));
            row
        })
        .collect();
    let horas: Vec<Vec<FixedValue>> = days
        .iter()
        .flat_map(|day| day.horas.iter())
        .map(|hora| {
            vec![
                u64::from(hora.number).into(),
                u64::from(hora.lord.id()).into(),
                hora.start.get().into(),
                hora.end.get().into(),
            ]
        })
        .collect();
    // The daylight's fifteen then the night's, per day, so a reader
    // walking the section in order walks the day in order.
    let muhurtas: Vec<Vec<FixedValue>> = days
        .iter()
        .flat_map(|day| {
            day.muhurtas
                .daylight
                .iter()
                .map(|at| (at, true))
                .chain(day.muhurtas.night.iter().map(|at| (at, false)))
        })
        .map(|(at, daylight)| {
            let mut row = interval(*at);
            row.push(flag(daylight));
            row
        })
        .collect();
    let moon_events: Vec<Vec<FixedValue>> = days
        .iter()
        .flat_map(|day| {
            day.moon
                .rises
                .iter()
                .map(|at| (at, TsMoonEvent::Rise))
                .chain(day.moon.sets.iter().map(|at| (at, TsMoonEvent::Set)))
        })
        .map(|(at, kind)| vec![(kind as u64).into(), at.get().into()])
        .collect();
    let muhurta_yogas: Vec<Vec<FixedValue>> = days
        .iter()
        .flat_map(|day| day.omens.yogas.iter())
        .map(|held| {
            use teistro_panchanga::omen::YogaCause;
            let (kind, vara, tithi, nakshatra) = match held.because {
                YogaCause::VaraNakshatra { vara, nakshatra } => {
                    (TsYogaCause::VaraNakshatra, vara.id(), 0, nakshatra.id())
                }
                YogaCause::VaraTithiNakshatra {
                    vara,
                    tithi,
                    nakshatra,
                } => (
                    TsYogaCause::VaraTithiNakshatra,
                    vara.id(),
                    tithi.id(),
                    nakshatra.id(),
                ),
            };
            let mut row = vec![u64::from(held.yoga.id()).into()];
            row.extend(interval(held.at));
            let tail: [FixedValue; 4] = [
                (kind as u64).into(),
                u64::from(vara).into(),
                u64::from(tithi).into(),
                u64::from(nakshatra).into(),
            ];
            row.extend(tail);
            row
        })
        .collect();

    (
        kaalas,
        choghadiya,
        horas,
        muhurtas,
        moon_events,
        muhurta_yogas,
    )
}

/// Every ragged section's rows, concatenated across the batch.
///
/// Days outermost, so a day's share of each list is the slice its
/// `counts` row names. Built in one pass so `encode` reads as the order
/// the sections are written rather than as thirteen loops.
struct RaggedRows {
    tithi: Vec<Vec<FixedValue>>,
    nakshatra: Vec<Vec<FixedValue>>,
    yoga: Vec<Vec<FixedValue>>,
    karana: Vec<Vec<FixedValue>>,
    panchaka: Vec<Vec<FixedValue>>,
    moon_signs: Vec<Vec<FixedValue>>,
    sun_signs: Vec<Vec<FixedValue>>,
    kaalas: Vec<Vec<FixedValue>>,
    choghadiya: Vec<Vec<FixedValue>>,
    horas: Vec<Vec<FixedValue>>,
    muhurtas: Vec<Vec<FixedValue>>,
    moon_events: Vec<Vec<FixedValue>>,
    muhurta_yogas: Vec<Vec<FixedValue>>,
}

impl RaggedRows {
    fn of(days: &[Panchanga]) -> RaggedRows {
        let (tithi, nakshatra, yoga, karana, panchaka, moon_signs, sun_signs) = spans(days);
        let (kaalas, choghadiya, horas, muhurtas, moon_events, muhurta_yogas) = periods(days);
        RaggedRows {
            tithi,
            nakshatra,
            yoga,
            karana,
            panchaka,
            moon_signs,
            sun_signs,
            kaalas,
            choghadiya,
            horas,
            muhurtas,
            moon_events,
            muhurta_yogas,
        }
    }
}

/// A batch of almanacs as the blob its schema describes.
///
/// Every per-day list runs days outermost and is concatenated across the
/// batch, with `counts` saying how many rows are each day's. A batch of
/// one is the ordinary case and the counts say so.
///
/// # Errors
///
/// Whatever the writer refuses: a section the schema does not have, or a
/// column of the wrong length. Neither can happen for a value this crate
/// built, so a failure here is a schema that has drifted from this
/// function rather than a caller's mistake.
pub fn encode(
    days: &[Panchanga],
    place: &Place,
    calendar: Calendar,
    provenance: &Provenance,
) -> Result<Vec<u8>, Error> {
    let schema = crate::schemas::panchanga();
    let mut writer = Writer::new(&schema);
    let day_count = u64::try_from(days.len()).unwrap_or(u64::MAX);
    let first = days.first();
    let convention = first
        .and_then(|day| TsLunarMonth::of(day.month.convention))
        .map_or(u64::from(u8::MAX), |month| month as u64);
    let model = first.map_or("", |day| day.day.model.as_str());

    let ragged = RaggedRows::of(days);
    let day_rows: Vec<Vec<FixedValue>> = days.iter().map(day_row).collect();
    let count_rows: Vec<Vec<FixedValue>> = days.iter().map(count_row).collect();
    let local_days: Vec<Vec<FixedValue>> = days
        .iter()
        .map(|day| crate::chart::day_values(&day.day))
        .collect();

    let write = || -> Result<Vec<u8>, teistro_idl::blob::BlobError> {
        writer.fixed(
            "summary",
            &[
                day_count.into(),
                place.latitude.get().into(),
                place.longitude.get().into(),
                place.altitude.get().into(),
                u64::from(calendar.id()).into(),
                convention.into(),
            ],
        )?;
        writer.rows("days", &day_rows)?;
        writer.rows("counts", &count_rows)?;
        writer.rows("day", &local_days)?;
        writer.rows("tithi", &ragged.tithi)?;
        writer.rows("nakshatra", &ragged.nakshatra)?;
        writer.rows("yoga", &ragged.yoga)?;
        writer.rows("karana", &ragged.karana)?;
        writer.rows("panchaka", &ragged.panchaka)?;
        writer.rows("moon_signs", &ragged.moon_signs)?;
        writer.rows("sun_signs", &ragged.sun_signs)?;
        writer.rows("kaalas", &ragged.kaalas)?;
        writer.rows("choghadiya", &ragged.choghadiya)?;
        writer.rows("horas", &ragged.horas)?;
        writer.rows("muhurtas", &ragged.muhurtas)?;
        writer.rows("moon_events", &ragged.moon_events)?;
        writer.rows("muhurta_yogas", &ragged.muhurta_yogas)?;
        writer.bytes("model", model.as_bytes())?;
        writer.bytes(
            "provenance",
            teistro_core::envelope::canonical_json(provenance).as_bytes(),
        )?;
        writer.finish()
    };
    write().map_err(|error| {
        Error::new(
            Status::Internal,
            format!("the panchanga blob could not be written: {error}"),
        )
    })
}

/// Founds the almanac of every day in a range at one place and answers
/// with its blob: the four moving limbs, the periods, the lunar month,
/// what the Moon and the Sun did, and what each day is said to be.
///
/// A **range** rather than a grid, because consecutive windows share a
/// boundary — day *n*'s next sunrise is day *n+1*'s sunrise — so a month
/// of days is much cheaper than thirty days computed separately. A caller
/// wanting one day passes a range of one. A range holding more than a
/// year and a day is `OUT_OF_RANGE` naming the limit.
///
/// Everything but the request is the context's settings, so two calls
/// under one context are comparable and the settings hash says why.
///
/// A context without an ephemeris is `CAPABILITY`; a provider failure is
/// `PROVIDER` with the provider's own code in the last error. A polar day
/// under `day.polar_day_policy = UNDEFINED` is `UNSUPPORTED` naming the
/// policies that would synthesise one.
///
/// `api: blob=panchanga`
///
/// # Safety
///
/// `context` must be a live handle; `request` valid for a read; `out_blob`
/// valid for a write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_panchanga_days(
    context: *const TsContext,
    request: *const TsPanchangaRequest,
    out_blob: *mut TsBlob,
) -> Status {
    with_context(context, |ctx| {
        if request.is_null() {
            return Err(crate::support::null("request"));
        }
        // SAFETY: non-null; the caller promises a readable request.
        let asked = unsafe { *request };
        let provider = ctx.provider().ok_or_else(|| {
            Error::new(
                Status::Capability,
                "the context has no ephemeris: pass a provider vtable to ts_context_new, or the TS_CONTEXT_TEST_PROVIDER flag for tests",
            )
            .with_field("provider")
        })?;
        let resolved = ctx.resolved();
        let settings = &resolved.settings;
        let place = Place::new(
            Latitude::try_new(asked.latitude_deg)
                .map_err(|e| Error::from(e).with_field("latitude_deg"))?,
            Longitude::try_new(asked.longitude_deg)
                .map_err(|e| Error::from(e).with_field("longitude_deg"))?,
            Altitude::try_new(asked.altitude_m)
                .map_err(|e| Error::from(e).with_field("altitude_m"))?,
        );
        let asked_calendar = Calendar::from_id(asked.calendar).ok_or_else(|| {
            Error::new(
                Status::InvalidArg,
                format!("no calendar with id {}", asked.calendar),
            )
            .with_field("calendar")
        })?;
        let calendar = shipped(asked_calendar).ok_or_else(|| {
            Error::new(
                Status::Unsupported,
                format!("the SDK does not ship the `{asked_calendar}` calendar"),
            )
            .with_field("calendar")
        })?;
        let clock = UtcOffset::try_from_seconds(asked.utc_offset_seconds)
            .map_err(|e| Error::from(e).with_field("utc_offset_seconds"))?;
        let ayanamsha = match settings.frame.ayanamsha {
            AyanamshaChoice::Catalogued { id } => id,
            AyanamshaChoice::Custom { .. } => Ayanamsha::Lahiri,
        };
        let model = DrikSun::new(
            provider,
            ayanamsha,
            settings.day.sunrise,
            settings.provider.overrides,
            ctx.delta_t(),
        );
        let from = CalendarDate::defined(
            asked_calendar,
            asked.from_year,
            asked.from_month,
            asked.from_day,
        );
        let to = CalendarDate::defined(asked_calendar, asked.to_year, asked.to_month, asked.to_day);
        let founded = Almanac::new(
            provider,
            resolved,
            &model,
            calendar,
            &clock,
            PrecessionModel::default(),
            ctx.delta_t(),
        )
        .between(&from, &to, &place)?;
        let mut provenance = founded.provenance;
        // The founder leaves the placeholder; the boundary fills it, as
        // `ts_positions` and `ts_chart_found` do.
        provenance.content_hash = content_hash(&founded.value);
        let encoded = encode(&founded.value, &place, asked_calendar, &provenance)?;
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_blob, "out_blob", TsBlob::from_vec(encoded)) }
    })
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::float_cmp,
        clippy::cast_possible_wrap,
        reason = "a test fails by panicking, indexes its own blob and compares the numbers it wrote"
    )]

    use super::{Almanac, Panchanga, TsLunarMonth, TsMoonEvent, TsYogaCause};
    use teistro_astro::delta_t::DeltaTModel;
    use teistro_astro::precession::PrecessionModel;
    use teistro_calendar::solar::drik::DrikSun;
    use teistro_calendar::{CalendarDate, Gregorian};
    use teistro_core::catalogue::{Ayanamsha, Calendar};
    use teistro_core::envelope::{Provenance, content_hash};
    use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
    use teistro_core::settings::{
        DEFAULT_PROFILE, LunarMonth, OverridePolicy, Profile, SettingsPatch, Sunrise,
    };
    use teistro_core::time::UtcOffset;
    use teistro_port_ephemeris::test_provider::TestProvider;

    /// Kathmandu, where the corpus's own charts are.
    fn place() -> Place {
        Place::new(
            Latitude::literal(27.7172),
            Longitude::literal(85.3240),
            Altitude::literal(1400.0),
        )
    }

    /// A lunar month of days at Kathmandu, with the provenance the
    /// boundary would seal.
    ///
    /// A month rather than a day: the ragged layout cannot be told from a
    /// scalar one over a single day, and it takes a stretch of them for
    /// the limb counts to differ, which is what the offsets have to
    /// survive. Five was not enough — the first version of this test
    /// found its own sample too uniform to prove anything, which is what
    /// the guard at the end is for.
    fn founded() -> (Vec<Panchanga>, Provenance) {
        let provider = TestProvider;
        let resolved = Profile::shipped(DEFAULT_PROFILE)
            .expect("the default profile")
            .resolve(&SettingsPatch::default())
            .expect("it resolves");
        let model = DrikSun::new(
            &provider,
            Ayanamsha::Lahiri,
            Sunrise::CentreNoRefraction.into(),
            OverridePolicy::PreferNative,
            DeltaTModel::TableThenModel,
        );
        let clock = UtcOffset::literal(5, 45, 0);
        let founded = Almanac::new(
            &provider,
            &resolved,
            &model,
            &Gregorian,
            &clock,
            PrecessionModel::default(),
            DeltaTModel::TableThenModel,
        )
        .between(
            &CalendarDate::defined(Calendar::Gregorian, 2024, 6, 17),
            &CalendarDate::defined(Calendar::Gregorian, 2024, 6, 30),
            &place(),
        )
        .expect("a month of founded days");
        let mut provenance = founded.provenance;
        provenance.content_hash = content_hash(&founded.value);
        (founded.value, provenance)
    }

    /// A batch of almanacs crosses, and each day's rows are where its
    /// counts say they are.
    ///
    /// The whole point of the module. A ragged section is only readable
    /// if the counts and the concatenation agree, and nothing else checks
    /// that: a blob whose offsets were off by a day would still parse,
    /// into another day's tithis.
    #[test]
    fn a_batch_of_almanacs_round_trips_through_its_blob() {
        use teistro_idl::blob::Reader;

        let (days, provenance) = founded();
        let bytes =
            super::encode(&days, &place(), Calendar::Gregorian, &provenance).expect("it encodes");
        let schema = crate::schemas::panchanga();
        let reader = Reader::parse(&bytes, &schema).expect("a well-formed blob");

        let summary = reader.fixed("summary").expect("the summary");
        assert_eq!(summary[0].as_i64(), days.len() as i64);
        assert_eq!(summary[1].as_f64(), place().latitude.get());

        // Every count column sums to its section's row count. This is
        // what makes the offsets meaningful at all.
        for (column, section) in [
            ("tithi", "tithi"),
            ("nakshatra", "nakshatra"),
            ("karana", "karana"),
            ("kaalas", "kaalas"),
            ("choghadiya", "choghadiya"),
            ("horas", "horas"),
            ("muhurtas", "muhurtas"),
            ("moon_events", "moon_events"),
        ] {
            let counts = reader.column("counts", column).expect("a count column");
            let total: i64 = counts.iter().map(|c| c.as_i64()).sum();
            assert_eq!(
                total,
                reader.count(section).unwrap_or(0) as i64,
                "`counts.{column}` must account for every row of `{section}`"
            );
        }

        // And each day's own rows are the ones its offset names. Walking
        // the counts is what a reader does, so the test walks them too.
        let counts = reader
            .column("counts", "karana")
            .expect("the karana counts");
        let members = reader
            .column("karana", "member")
            .expect("the karana members");
        let mut at = 0usize;
        for (index, day) in days.iter().enumerate() {
            let count = usize::try_from(counts[index].as_i64()).unwrap_or(0);
            assert_eq!(count, day.limbs.karana.len(), "day {index}'s karana count");
            for (row, span) in day.limbs.karana.iter().enumerate() {
                assert_eq!(
                    members[at + row].as_i64(),
                    i64::from(span.member.id()),
                    "day {index}, karana {row}"
                );
            }
            at += count;
        }
        // The sample must actually be ragged, or the walk above passes
        // over a rectangular layout too and proves nothing.
        assert_ne!(
            days.iter().map(|d| d.limbs.karana.len()).min(),
            days.iter().map(|d| d.limbs.karana.len()).max(),
            "the sample must hold days of differing karana counts"
        );

        // The day section is the shared one, and carries no field that
        // belongs to an instant.
        let vara = reader.column("day", "vara").expect("the varas");
        for (index, day) in days.iter().enumerate() {
            assert_eq!(vara[index].as_i64(), i64::from(day.day.vara.id()));
        }
        assert!(
            reader.column("day", "part").is_err(),
            "`part` belongs to a chart's instant, not to a day"
        );

        let model = reader.bytes("model").expect("the model");
        assert_eq!(model, days[0].day.model.as_bytes());
    }

    /// A range of one day is a blob, and the ordinary case.
    #[test]
    fn a_range_of_one_day_is_a_batch_of_one() {
        use teistro_idl::blob::Reader;

        let (days, provenance) = founded();
        let one = &days[..1];
        let bytes =
            super::encode(one, &place(), Calendar::Gregorian, &provenance).expect("it encodes");
        let schema = crate::schemas::panchanga();
        let reader = Reader::parse(&bytes, &schema).expect("a well-formed blob");
        assert_eq!(reader.fixed("summary").expect("the summary")[0].as_i64(), 1);
        assert_eq!(
            reader.count("horas"),
            Some(24),
            "one day, twenty-four horas"
        );
    }

    /// Every member of the knob crosses, and to an id of its own.
    #[test]
    fn every_lunar_month_member_crosses_to_an_id_of_its_own() {
        let ids: Vec<u8> = LunarMonth::ALL
            .iter()
            .map(|member| {
                TsLunarMonth::of(*member)
                    .unwrap_or_else(|| panic!("`{member}` has no id at the boundary"))
                    as u8
            })
            .collect();
        assert_eq!(ids, vec![0, 1], "one id each, in order");
    }

    /// Every kind the calendar has crosses, and to an id of its own.
    ///
    /// `MonthKind` is the calendar's and exhaustive, so a kind added
    /// there stops this crate compiling rather than crossing as whatever
    /// came first — which is the guard the settings knobs cannot have
    /// (`chart.rs`) and this one can.
    #[test]
    fn every_month_kind_crosses_to_an_id_of_its_own() {
        use teistro_calendar::lunisolar::MonthKind;

        let ids: Vec<u8> = [MonthKind::Nija, MonthKind::Adhika, MonthKind::Kshaya]
            .into_iter()
            .map(|kind| super::TsMonthKind::from(kind) as u8)
            .collect();
        assert_eq!(ids, vec![0, 1, 2], "one id each, in order");
    }

    /// The two boundary-owned enums number from nought and do not
    /// collide with each other's meaning.
    #[test]
    fn the_boundary_enums_number_from_nought() {
        assert_eq!(TsMoonEvent::Rise as u8, 0);
        assert_eq!(TsMoonEvent::Set as u8, 1);
        assert_eq!(TsYogaCause::VaraNakshatra as u8, 0);
        assert_eq!(TsYogaCause::VaraTithiNakshatra as u8, 1);
    }
}
