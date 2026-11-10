//! A range of calendar cells: what a month grid prints of each date, and no
//! more (`docs/03-design/month-cells.md`, SR-84).
//!
//! A cell is read off the days festival rules read
//! ([`Almanac::festival_days`](crate::Almanac::festival_days)), the range
//! and the day either side of it, so its sunrise and its tithis are the
//! ones the full day holds, to the bit, and no Moon rise or set, period or
//! muhurta is computed for it.

use serde::{Deserialize, Serialize};
use teistro_calendar::CalendarDate;
use teistro_core::catalogue::{Paksha, Tithi, Vara};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};

use teistro_time::local_day::{DayState, LocalDay};

use crate::festival::Observances;
use crate::month::LunarMonth;
use crate::span::Span;

/// How a cell's tithis stand against its sunrises: whether the tithi at
/// its sunrise held the sunrise before as well, and whether a tithi began
/// after its sunrise and ended before the next.
///
/// ```
/// use teistro_panchanga::cell::CellKind;
///
/// assert_eq!(CellKind::of(false, true), CellKind::Kshaya);
/// assert_eq!(serde_json::to_string(&CellKind::VriddhiAndKshaya).unwrap(), r#""VRIDDHI_AND_KSHAYA""#);
/// assert!(CellKind::Vriddhi.is_vriddhi() && !CellKind::Vriddhi.is_kshaya());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CellKind {
    /// The sunrise tithi is not the one the sunrise before held, and every
    /// tithi ending before the next sunrise held a sunrise.
    Plain,
    /// *Kṣaya*: a tithi began after this sunrise and ended before the
    /// next, so no date is named by it; [`TithiEnd::kshaya`] names it.
    Kshaya,
    /// *Vṛddhi*: the tithi at this sunrise held the sunrise before as
    /// well, so it names two dates running and this is the second.
    Vriddhi,
    /// Both at once: the sunrise tithi held the sunrise before and a tithi
    /// after it began and ended before the next sunrise. Tithis next to
    /// one another differ in length by less than an hour, so this asks a
    /// tithi of a day and a little more beside one of a little less; it
    /// is kept apart rather than folded into either.
    VriddhiAndKshaya,
}

impl CellKind {
    /// The kind of a cell whose sunrise tithi held the sunrise before
    /// (`vriddhi`) and which holds a skipped tithi (`kshaya`).
    ///
    /// ```
    /// use teistro_panchanga::cell::CellKind;
    ///
    /// assert_eq!(CellKind::of(true, true), CellKind::VriddhiAndKshaya);
    /// assert_eq!(CellKind::of(false, false), CellKind::Plain);
    /// ```
    #[must_use]
    pub const fn of(vriddhi: bool, kshaya: bool) -> CellKind {
        match (vriddhi, kshaya) {
            (false, false) => CellKind::Plain,
            (false, true) => CellKind::Kshaya,
            (true, false) => CellKind::Vriddhi,
            (true, true) => CellKind::VriddhiAndKshaya,
        }
    }

    /// Whether the sunrise tithi held the sunrise before.
    ///
    /// ```
    /// use teistro_panchanga::cell::CellKind;
    ///
    /// assert!(CellKind::VriddhiAndKshaya.is_vriddhi());
    /// assert!(!CellKind::Kshaya.is_vriddhi());
    /// ```
    #[must_use]
    pub const fn is_vriddhi(self) -> bool {
        matches!(self, CellKind::Vriddhi | CellKind::VriddhiAndKshaya)
    }

    /// Whether a tithi began and ended between this sunrise and the next.
    ///
    /// ```
    /// use teistro_panchanga::cell::CellKind;
    ///
    /// assert!(CellKind::VriddhiAndKshaya.is_kshaya());
    /// assert!(!CellKind::Vriddhi.is_kshaya());
    /// ```
    #[must_use]
    pub const fn is_kshaya(self) -> bool {
        matches!(self, CellKind::Kshaya | CellKind::VriddhiAndKshaya)
    }
}

/// A tithi that ends after a cell's sunrise and at or before the next.
///
/// ```
/// use teistro_core::catalogue::Tithi;
/// use teistro_core::quantity::JulianDay;
/// use teistro_panchanga::cell::TithiEnd;
///
/// let end = TithiEnd { tithi: Tithi::ShuklaDvitiya, at: JulianDay::literal(2_461_345.75), kshaya: false };
/// assert_eq!(serde_json::to_value(end).unwrap()["tithi"], "SHUKLA_DVITIYA");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TithiEnd {
    /// The tithi.
    pub tithi: Tithi,
    /// The instant it ends.
    pub at: JulianDay<Utc>,
    /// Whether it also began after the cell's sunrise: a *kṣaya* tithi,
    /// which names no date.
    pub kshaya: bool,
}

/// What a month grid prints of one date: its vara and sunrise, the tithi
/// at the sunrise and its paksha, the lunar month, the tithis that end
/// before the next sunrise, and whether a tithi is skipped or repeated.
///
/// ```
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_calendar::solar::drik::DrikSun;
/// # use teistro_calendar::{CalendarDate, Gregorian};
/// # use teistro_core::catalogue::{Ayanamsha, Calendar};
/// # use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
/// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::Almanac;
/// # use teistro_port_ephemeris::TestProvider;
/// #
/// # let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
/// # let provider = TestProvider;
/// # let resolved = Profile::shipped(DEFAULT_PROFILE)
/// #     .ok_or("a shipped profile")?
/// #     .resolve(&SettingsPatch::default())
/// #     .map_err(|d| format!("{d:?}"))?;
/// # let (delta_t, precession) = (DeltaTModel::TableThenModel, PrecessionModel::Vondrak2011);
/// # let model = DrikSun::new(&provider, Ayanamsha::Lahiri, Sunrise::CentreNoRefraction.into(), OverridePolicy::PreferNative, delta_t);
/// # let clock = UtcOffset::literal(5, 45, 0);
/// # let almanac = Almanac::new(&provider, &resolved, &model, &Gregorian, &clock, precession, delta_t);
/// # let date = |day| CalendarDate::defined(Calendar::Gregorian, 2026, 11, day);
/// let cell = &almanac.cells(&date(25), &date(25), &kathmandu)?.value[0];
/// // Every tithi ending in the cell ends after its sunrise.
/// assert!(cell.ends.iter().all(|end| end.at.get() > cell.sunrise.get()));
/// assert_eq!(cell.paksha, cell.tithi.attributes().paksha);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct DayCell {
    /// The civil date, in the calendar the range was asked in.
    pub date: CalendarDate,
    /// Its vara, from this sunrise to the next.
    pub vara: Vara,
    /// Its sunrise: the full day's `day.sunrise`, to the bit.
    pub sunrise: JulianDay<Utc>,
    /// The next sunrise, the next date's `sunrise`.
    pub next_sunrise: JulianDay<Utc>,
    /// Whether the Sun rose and set; a polar day is read under the
    /// profile's `day.polarDayPolicy`, as the full day is.
    pub normal: bool,
    /// The tithi at the sunrise: the date's name.
    pub tithi: Tithi,
    /// That tithi's paksha.
    pub paksha: Paksha,
    /// The lunar month at the sunrise, with its kind (adhika, nija or
    /// kshaya).
    pub month: LunarMonth,
    /// Every tithi ending after the sunrise and at or before the next, in
    /// order: the sunrise tithi first unless it runs past the next
    /// sunrise, then any skipped one.
    pub ends: Vec<TithiEnd>,
    /// Whether a tithi is skipped or repeated here.
    pub kind: CellKind,
    /// The keys of the festivals, fasts, dated days, anniversaries and
    /// published days kept on this date, in that order; only when they
    /// were asked for, and left out when none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub festivals: Vec<String>,
}

/// What a cell reads of one day
/// ([`Almanac::cell_days`](crate::Almanac::cell_days)): each part the one
/// the day's [`FestivalDay`](crate::festival::FestivalDay) holds, to the
/// bit.
///
/// ```
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_calendar::solar::drik::DrikSun;
/// # use teistro_calendar::{CalendarDate, Gregorian};
/// # use teistro_core::catalogue::{Ayanamsha, Calendar};
/// # use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
/// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::Almanac;
/// # use teistro_port_ephemeris::TestProvider;
/// #
/// # let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
/// # let provider = TestProvider;
/// # let resolved = Profile::shipped(DEFAULT_PROFILE)
/// #     .ok_or("a shipped profile")?
/// #     .resolve(&SettingsPatch::default())
/// #     .map_err(|d| format!("{d:?}"))?;
/// # let (delta_t, precession) = (DeltaTModel::TableThenModel, PrecessionModel::Vondrak2011);
/// # let model = DrikSun::new(&provider, Ayanamsha::Lahiri, Sunrise::CentreNoRefraction.into(), OverridePolicy::PreferNative, delta_t);
/// # let clock = UtcOffset::literal(5, 45, 0);
/// # let almanac = Almanac::new(&provider, &resolved, &model, &Gregorian, &clock, precession, delta_t);
/// # let date = |day| CalendarDate::defined(Calendar::Gregorian, 2026, 11, day);
/// use teistro_panchanga::cell;
///
/// let days = almanac.cell_days(&date(25), &date(26), &kathmandu)?.value;
/// // The range and a day either side, each opening at the one before's next sunrise.
/// assert_eq!(days.len(), 4);
/// assert!(days.windows(2).all(|pair| pair[0].next_sunrise == pair[1].sunrise));
/// assert_eq!(cell::cells(&days)?.len(), 2);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct CellDay {
    /// The civil date.
    pub date: CalendarDate,
    /// Its vara.
    pub vara: Vara,
    /// Its sunrise.
    pub sunrise: JulianDay<Utc>,
    /// The next sunrise.
    pub next_sunrise: JulianDay<Utc>,
    /// Whether the Sun rose and set.
    pub normal: bool,
    /// The lunar month at the sunrise.
    pub month: LunarMonth,
    /// The tithi spans of the day's window, in order.
    pub tithi: Vec<Span<Tithi>>,
}

impl CellDay {
    /// A day's cell parts, from its local day, its month and its tithis.
    pub(crate) fn of(day: &LocalDay, month: LunarMonth, tithi: Vec<Span<Tithi>>) -> CellDay {
        CellDay {
            date: day.date.clone(),
            vara: day.vara,
            sunrise: day.sunrise,
            next_sunrise: day.next_sunrise,
            normal: matches!(day.state, DayState::Normal),
            month,
            tithi,
        }
    }
}

/// The cells of a run of days that holds one day more than the range on
/// either side ([`Almanac::cell_days`](crate::Almanac::cell_days)): one
/// cell for each day but the first and the last, which only lend the
/// sunrise before the first and the tithis past the last.
///
/// ```
/// use teistro_panchanga::cell;
///
/// // A run without a day either side founds no cell.
/// assert!(cell::cells(&[]).is_err());
/// ```
///
/// # Errors
///
/// `INTERNAL` for fewer than three days, days not each opening at the one
/// before's next sunrise, or a sunrise no tithi span holds, none of which
/// a run of cell days gives.
pub fn cells(days: &[CellDay]) -> Result<Vec<DayCell>, Error> {
    if days.len() < 3 {
        return Err(Error::internal(
            "a run of cells is founded with a day either side",
        ));
    }
    // Every tithi the run met, once and in order: a tithi crossing a
    // day's edge is met again by the next day, and a polar day's window
    // can reach past the next day's opening, so a span is kept only when
    // it begins after the last one kept.
    let mut spans: Vec<&Span<Tithi>> = Vec::new();
    for span in days.iter().flat_map(|day| &day.tithi) {
        if spans
            .last()
            .is_none_or(|last| span.whole.from.get() > last.whole.from.get())
        {
            spans.push(span);
        }
    }
    days.windows(3)
        .map(|run| {
            let [before, day, _] = run else {
                return Err(Error::internal("a window of three days held fewer"));
            };
            cell(before.sunrise, day, &spans)
        })
        .collect()
}

/// One cell, from its day, the sunrise before it and the run's tithis.
fn cell(before: JulianDay<Utc>, day: &CellDay, spans: &[&Span<Tithi>]) -> Result<DayCell, Error> {
    let (sunrise, next) = (day.sunrise, day.next_sunrise);
    let Some(opening) = spans.iter().find(|span| span.contains(sunrise)) else {
        return Err(Error::internal(format!(
            "no tithi span holds the sunrise of {}",
            day.date
        )));
    };
    let ends: Vec<TithiEnd> = spans
        .iter()
        .filter(|span| span.whole.to.get() > sunrise.get() && span.whole.to.get() <= next.get())
        .map(|span| TithiEnd {
            tithi: span.member,
            at: span.whole.to,
            kshaya: span.sunrises(sunrise, next).is_kshaya(),
        })
        .collect();
    // A polar day under a nearest-event policy can open on the same
    // sunrise as the day before, which repeats nothing.
    let vriddhi = before.get() < sunrise.get() && opening.contains(before);
    Ok(DayCell {
        date: day.date.clone(),
        vara: day.vara,
        sunrise,
        next_sunrise: next,
        normal: day.normal,
        tithi: opening.member,
        paksha: opening.member.attributes().paksha,
        month: day.month,
        kind: CellKind::of(vriddhi, ends.iter().any(|end| end.kshaya)),
        ends,
        festivals: Vec::new(),
    })
}

/// Writes onto each cell the keys of what `found` keeps on its date.
///
/// ```
/// use teistro_panchanga::cell::with_festivals;
/// use teistro_panchanga::festival::Observances;
///
/// let mut cells = Vec::new();
/// with_festivals(&mut cells, &Observances::default());
/// assert!(cells.is_empty());
/// ```
pub fn with_festivals(cells: &mut [DayCell], found: &Observances) {
    for cell in cells {
        cell.festivals = found.keys_on(&cell.date);
    }
}
