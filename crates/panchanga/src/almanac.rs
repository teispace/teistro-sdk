//! The almanac itself: one day at one place, assembled and stamped.
//!
//! Everything beneath this module answers one question well; this one
//! asks each of them once, asks the provider once, and hands the answers
//! on with a provenance. The rule it follows is the chart foundation's:
//! it holds what is needed to read a day and never what a module above it
//! computes.
//!
//! The range is the primary shape (principle 5). A month of days is what
//! an application actually asks for, and it is far cheaper than thirty
//! days computed one at a time: consecutive windows share a boundary, so
//! one crossing search over the month replaces thirty overlapping ones.

use serde::{Deserialize, Serialize};
use teistro_astro::completion::Completion;
use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::lockstep::Lockstep;
use teistro_astro::precession::PrecessionModel;
use teistro_astro::rise_set::Solver;
use teistro_calendar::lunisolar::{MonthKind, SAMSARPA_REACH_MONTHS};
use teistro_calendar::solar::{SolarModel, civil_day_lights};
use teistro_calendar::{CalendarDate, CalendarSystem, FixedDay};
use teistro_chart::zodiac::ChartZodiac;
use teistro_core::catalogue::{
    AgniVasa, AshubhaYoga, BhadraVasa, DayMarker, Direction, Graha, Nakshatra, Panchaka, Rashi,
    Ritu, ShivaVasa, Tithi, Vara,
};
use teistro_core::envelope::{Envelope, Hash, Provenance, Version, content_hash};
use teistro_core::error::{Error, Status};
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Place, Ut1, Utc};
use teistro_core::settings::{
    Centre, GhatiReckoning, MoonEvents, PanchakaName, Resolved, RituReckoning, Settings,
};
use teistro_core::time::{ClockReading, LocalClock, ZoneSource};
use teistro_port_ephemeris::{Body, EphemerisProvider, Frame, Horizon, HorizonEventKind};
use teistro_time::ghati::{self, GhatiPala};
use teistro_time::hora::{self, Hora};
use teistro_time::local_day::{
    Carried, LocalDay, local_day, local_day_carrying, local_day_lit, local_midnight,
};

use crate::ashauch::{
    self, Ashauch, AshauchRequest, AshauchTable, Household, HouseholdAnswer, HouseholdPeriods,
    Running,
};
use crate::change::{ChangeDay, ChangePoints};
use crate::death_time::{self, DeathInstant, DeathTime, DeathTimeTable};
use crate::festival::{
    FestivalDay, Half, MoonArc, MoonArcs, Moonrise, MoonriseAt, Origin, OriginFound, SunEntry,
};
use crate::first_year::{self, FirstYear, FirstYearSchedule};
use crate::limb::{self, Limbs, RangeCrossings, Zodiac};
use crate::month::{self, LunarMonth};
use crate::omen::{self, Omens};
use crate::period::{self, Durmuhurta, Kaalas, Muhurtas, Part};
use crate::punya::{self, Arcs, PunyaKala};
use crate::rulers::{self, YearRuler};
use crate::sankalpa::{self, AyanaReckoning, Sankalpa};
use crate::sky::{self, MoonDay, SunDay};
use crate::span::{self, Span, Sunrises};
use crate::year::{LunarYear, name_counts};
use teistro_calendar::samvatsara::JovianYear;

/// The almanac of one day at one place.
///
/// A plain value: no interior mutability, no handle, no lazy field. Two
/// panchangas of the same inputs under the same settings are equal field
/// for field, which is what the determinism contract asks of everything
/// that crosses a binding.
///
/// ```
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_calendar::solar::drik::DrikSun;
/// # use teistro_calendar::{CalendarDate, Gregorian};
/// # use teistro_core::catalogue::{Ayanamsha, Calendar};
/// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
/// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
/// # use teistro_core::time::UtcOffset;
/// # use teistro_panchanga::Almanac;
/// # use teistro_port_ephemeris::TestProvider;
/// #
/// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
/// # // elements that need no engine. A real provider reads the real sky.
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
/// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
///
/// use teistro_core::catalogue::Vara;
///
/// let day = almanac.day(&date(11, 25), &kathmandu)?.value;
/// assert_eq!(day.vara(), Vara::Budhavara);
/// // Twenty-four horas, eight choghadiya by day and eight by night, thirty muhurtas.
/// assert_eq!((day.horas.len(), day.choghadiya.len()), (24, 16));
/// assert_eq!(day.muhurtas.daylight.len() + day.muhurtas.night.len(), 30);
/// // The lagna table is empty unless asked for.
/// assert!(day.lagnas.is_empty());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Panchanga {
    /// The day, with its arc, its vara, its polar state and the sunrise
    /// convention it was reckoned under.
    pub day: LocalDay,
    /// What the limb spans are clipped to: the arc under a sunrise
    /// boundary, the civil day under a midnight one.
    pub window: Interval,
    /// The four moving limbs.
    pub limbs: Limbs,
    /// The inauspicious eighths of the daylight the day has.
    pub kaalas: Vec<Kaalas>,
    /// Eight choghadiya of the daylight and eight of the night, when the
    /// day has both.
    pub choghadiya: Vec<Part>,
    /// Twenty-four horas, from `time::hora`.
    pub horas: Vec<Hora>,
    /// The thirty muhurtas, with Abhijit and Brahma muhurta named.
    pub muhurtas: Muhurtas,
    /// The day's censured muhurtas under `panchanga.durmuhurta`, in the
    /// order they run (`03-design/panchanga-day.md` §21, crux C398).
    pub durmuhurtas: Vec<Durmuhurta>,
    /// The lunar month, under both conventions.
    pub month: LunarMonth,
    /// What the Moon did.
    pub moon: MoonDay,
    /// What the Sun did.
    pub sun: SunDay,
    /// What the day is said to be.
    pub omens: Omens,
    /// How the day's ghatis are counted, `day.ghati_reckoning` as
    /// applied: what [`Panchanga::ghati_pala`] reads.
    pub ghati_reckoning: GhatiReckoning,
    /// The day's lagna table: each sign rising from sunrise to the next
    /// sunrise, its `whole` span and the part `inside` that arc, under
    /// `panchanga.lagna_table` (crux C680, C681). **Empty unless asked
    /// for** (the façade's `AlmanacRequest::with_lagnas`), since a day's
    /// cuts cost some four hundred ascendants; and left out of the
    /// day's document when empty, so a day not asked for it hashes as it
    /// did before the table existed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lagnas: Vec<Span<Rashi>>,
    /// The punya kala that falls in the day's daylight, of a sankranti
    /// in the day or in the night either side of it, under
    /// `panchanga.punya_kala` (`03-design/panchanga-day.md` §21.5, crux
    /// C752). Left out of the day's document when the day holds none,
    /// which is all but some twelve days a year.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub punya_kala: Option<PunyaKala>,
    /// The day's homa ahuti: the graha whose mouth an oblation falls in,
    /// by the Moon's nakshatra counted from the Sun's in threes, one span
    /// for each stretch the two stars both held (Muhurta Chintamani,
    /// Nakshatra-prakarana v. 35; crux C751). **Empty unless asked for**
    /// (the façade's `AlmanacRequest::with_homa_ahuti`), since the Sun's
    /// stars cost a search a day does not otherwise make; and left out of
    /// the day's document when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub homa_ahuti: Vec<Span<Graha>>,
    /// The day's markers from the Moon alone, ganda mula, gandanta and
    /// vinchudo, in the order they begin ([`Panchanga::mark`],
    /// `03-design/day-markers.md` §3). **Empty unless asked for** (the
    /// façade's `AlmanacRequest::with_markers`), so a day not asked pays
    /// nothing; and left out of the day's document when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub markers: Vec<Span<DayMarker>>,
    /// The day's Gowri panchangam: eight Gowri of the daylight and eight
    /// of the night ([`Panchanga::divide_gowri`],
    /// `03-design/gowri-panchangam.md`), a printed reading (crux C1052).
    /// **Empty unless asked for** (the façade's
    /// `AlmanacRequest::with_gowri`), and left out of the day's document
    /// when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gowri: Vec<period::GowriPart>,
    /// Where Shiva dwells, one span for each tithi the day held, under
    /// `panchanga.shiva_vasa`: **a printed convention** with no text
    /// behind it, which the answer's provenance names
    /// (`03-design/printed-conventions.md`, crux C750). Empty under
    /// `NONE`, and then left out of the day's document.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shiva_vasa: Vec<Span<ShivaVasa>>,
    /// The panchaka the day holds, named by the vara it began on, under
    /// `panchanga.panchaka_name`: **a printed convention**, which the
    /// answer's provenance names (crux C821). `whole` runs from the
    /// panchaka's first instant to the Moon's leaving Revati, `inside` is
    /// its part in the day. None on a day without panchaka, for one begun
    /// on a Wednesday or a Thursday, which the almanacs leave unnamed, and
    /// under `NONE`; left out of the day's document then.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub named_panchaka: Option<Span<Panchaka>>,
    /// The clock the day was asked on, at the day's opening, for a day
    /// asked by zone or local mean time. A day asked by a bare offset has
    /// nothing to add to its answer's provenance, and leaves this out of
    /// its document (`docs/03-design/time-and-timezone.md` §3.4).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clock: Option<ClockReading>,
}

impl Panchanga {
    /// The day's vara, which is the arc's and not the civil date's.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// # // Wednesday 25 November 2026.
    /// # let day = almanac.day(&date(11, 25), &kathmandu)?.value;
    /// # let sunrise = day.day.sunrise;
    ///
    /// use teistro_core::catalogue::Vara;
    ///
    /// assert_eq!(day.vara(), Vara::Budhavara);
    /// assert_eq!(day.vara(), day.day.vara);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn vara(&self) -> Vara {
        self.day.vara
    }

    /// The day's Nepal Sambat date: its year, its month and its half
    /// (`03-design/calendar-indian-lunisolar.md` §11).
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// # // Wednesday 25 November 2026.
    /// # let day = almanac.day(&date(11, 25), &kathmandu)?.value;
    /// # let sunrise = day.day.sunrise;
    ///
    /// // Nepal Sambat 1147 opened on 10 November 2026, at Kachhala.
    /// let ns = day.nepal_sambat();
    /// assert_eq!((ns.year, ns.month), (1147, 1));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn nepal_sambat(&self) -> crate::NepalSambatDate {
        crate::NepalSambatDate::of(self)
    }

    /// Which of the day's two sunrises a span of this day was running
    /// at ([`Span::sunrises`]), by the day's own sunrises.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// # // Wednesday 25 November 2026.
    /// # let day = almanac.day(&date(11, 25), &kathmandu)?.value;
    /// # let sunrise = day.day.sunrise;
    ///
    /// use teistro_panchanga::span::Sunrises;
    ///
    /// // The first tithi of the day is the one its sunrise holds.
    /// let first = day.sunrises(&day.limbs.tithi[0]);
    /// assert!(matches!(first, Sunrises::Opening | Sunrises::Both));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn sunrises<T>(&self, span: &Span<T>) -> Sunrises {
        span.sunrises(self.day.sunrise, self.day.next_sunrise)
    }

    /// The member of a limb that names two days running: the one this
    /// day's sunrise and the next both fall in (vriddhi).
    ///
    /// ```no_run
    /// # fn day() -> teistro_panchanga::almanac::Panchanga { unimplemented!() }
    /// let day = day();
    /// if let Some(tithi) = day.vriddhi(&day.limbs.tithi) {
    ///     println!("{tithi:?} names tomorrow too");
    /// }
    /// for tithi in day.kshaya(&day.limbs.tithi) {
    ///     println!("{tithi:?} names no day");
    /// }
    /// ```
    #[must_use]
    pub fn vriddhi<T: Copy>(&self, spans: &[Span<T>]) -> Option<T> {
        spans
            .iter()
            .find(|span| self.sunrises(span).is_vriddhi())
            .map(|span| span.member)
    }

    /// The members of a limb that name no day: begun after this day's
    /// sunrise and ended before the next (kshaya).
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// # // Wednesday 25 November 2026.
    /// # let day = almanac.day(&date(11, 25), &kathmandu)?.value;
    /// # let sunrise = day.day.sunrise;
    ///
    /// use teistro_panchanga::span::Sunrises;
    ///
    /// // A kshaya tithi holds neither sunrise.
    /// for tithi in day.kshaya(&day.limbs.tithi) {
    ///     let span = day.limbs.tithi.iter().find(|span| span.member == tithi).ok_or("its span")?;
    ///     assert_eq!(day.sunrises(span), Sunrises::Neither);
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn kshaya<T: Copy>(&self, spans: &[Span<T>]) -> Vec<T> {
        spans
            .iter()
            .filter(|span| self.sunrises(span).is_kshaya())
            .map(|span| span.member)
            .collect()
    }

    /// An instant of the day as ghati, pala and vipala from its sunrise,
    /// counted under `ghati_reckoning`; an instant past the next sunrise
    /// counts as the next sunrise, and one before this sunrise as the
    /// sunrise, so a span's end reads as the patro prints it, clipped to
    /// the day.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// # // Wednesday 25 November 2026.
    /// # let day = almanac.day(&date(11, 25), &kathmandu)?.value;
    /// # let sunrise = day.day.sunrise;
    ///
    /// // Sunrise is nought; an instant before it reads as sunrise.
    /// let at_sunrise = day.ghati_pala(sunrise)?;
    /// assert_eq!((at_sunrise.ghati, at_sunrise.pala, at_sunrise.vipala), (0, 0, 0));
    /// assert_eq!(day.ghati_pala(JulianDay::literal(sunrise.get() - 0.1))?, at_sunrise);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A reckoning this build does not know.
    pub fn ghati_pala(&self, instant: JulianDay<Utc>) -> Result<GhatiPala, Error> {
        let clamped = if instant.get() < self.day.sunrise.get() {
            self.day.sunrise
        } else if instant.get() >= self.day.next_sunrise.get() {
            // The day's own count runs to its next sunrise, which is
            // outside `LocalDay::contains`; its full length is the count.
            return Ok(ghati::day_length(
                &self.day,
                self.ghati_reckoning.try_into()?,
            ));
        } else {
            instant
        };
        ghati::ghati_pala(&self.day, clamped, self.ghati_reckoning.try_into()?)
    }

    /// The direction not to travel in.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// # // Wednesday 25 November 2026.
    /// # let day = almanac.day(&date(11, 25), &kathmandu)?.value;
    /// # let sunrise = day.day.sunrise;
    ///
    /// use teistro_core::catalogue::Direction;
    ///
    /// // A Wednesday: not north.
    /// assert_eq!(day.disha_shool(), Direction::North);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn disha_shool(&self) -> Direction {
        self.omens.disha_shool
    }

    /// The direction the day's nakshatras block travel in, while they run
    /// ([`omen::nakshatra_shool`]): empty on a day the Moon crosses none
    /// of the four.
    ///
    /// ```no_run
    /// # fn day() -> teistro_panchanga::almanac::Panchanga { unimplemented!() }
    /// let day = day();
    /// println!("not {:?} by the vara", day.disha_shool());
    /// for shool in day.nakshatra_shool() {
    ///     println!("not {:?} until {}", shool.member, shool.inside.to);
    /// }
    /// ```
    #[must_use]
    pub fn nakshatra_shool(&self) -> Vec<Span<Direction>> {
        omen::nakshatra_shools(&self.limbs.nakshatra)
    }

    /// The direction the Moon dwells in over the day, by the sign it
    /// stood in ([`omen::chandra_vasa`]).
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// # // Wednesday 25 November 2026.
    /// # let day = almanac.day(&date(11, 25), &kathmandu)?.value;
    /// # let sunrise = day.day.sunrise;
    ///
    /// // One direction for each sign the Moon stood in over the day.
    /// assert_eq!(day.chandra_vasa().len(), day.moon.signs.len());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn chandra_vasa(&self) -> Vec<Span<Direction>> {
        omen::chandra_vasas(&self.moon.signs)
    }

    /// The day's inauspicious yogas, its vara with a tithi or a nakshatra
    /// ([`omen::ashubha_yogas`], Muhurta Chintamani,
    /// Shubhashubha-prakarana vv. 8–9, crux C760): a span for each tithi
    /// that is dagdha, visha or hutashana on the day's vara, and for the
    /// nakshatra that is its Yamaghanta. Read from the stored day, so it
    /// costs nothing until asked and changes no day's hash.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// # // Wednesday 25 November 2026.
    /// # let day = almanac.day(&date(11, 25), &kathmandu)?.value;
    /// # let sunrise = day.day.sunrise;
    ///
    /// // Each yoga runs over a tithi or a nakshatra the day held.
    /// let wholes: Vec<_> = day.limbs.tithi.iter().map(|t| t.whole).chain(day.limbs.nakshatra.iter().map(|n| n.whole)).collect();
    /// assert!(day.ashubha_yogas().iter().all(|yoga| wholes.contains(&yoga.whole)));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn ashubha_yogas(&self) -> Vec<Span<AshubhaYoga>> {
        omen::ashubha_yogas(&self.limbs.tithi, &self.limbs.nakshatra, self.day.vara)
    }

    /// Where fire dwells over the day for a kamya homa, one span for each
    /// tithi the day held, read with the day's vara
    /// ([`omen::agni_vasa`], Muhurta Chintamani, Nakshatra-prakarana
    /// v. 36).
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// # // Wednesday 25 November 2026.
    /// # let day = almanac.day(&date(11, 25), &kathmandu)?.value;
    /// # let sunrise = day.day.sunrise;
    ///
    /// // One place for each tithi the day held.
    /// assert_eq!(day.agni_vasa().len(), day.limbs.tithi.len());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn agni_vasa(&self) -> Vec<Span<AgniVasa>> {
        omen::agni_vasas(&self.limbs.tithi, self.day.vara)
    }

    /// Where Bhadra dwells while the vishti karana holds, by the sign the
    /// Moon stands in ([`omen::bhadra_vasa`], Muhurta Chintamani,
    /// Shubhashubha-prakarana v. 45): one span a stretch of vishti and a
    /// sign, so a sign change in Bhadra splits it; only `MARTYA`, the
    /// earth, is harmful. Empty on a day with no vishti.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// # // Wednesday 25 November 2026.
    /// # let day = almanac.day(&date(11, 25), &kathmandu)?.value;
    /// use teistro_core::catalogue::Karana;
    ///
    /// // Each loka lies inside a vishti of the day.
    /// for loka in day.bhadra_vasa() {
    ///     assert!(day.limbs.karana.iter().any(|k| k.member == Karana::Vishti && k.whole.clipped_to(loka.whole) == Some(loka.whole)));
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn bhadra_vasa(&self) -> Vec<Span<BhadraVasa>> {
        omen::bhadra_vasas(&self.limbs.karana, &self.moon.signs, self.window)
    }

    /// Fills [`Panchanga::markers`] from the day's own stars and the
    /// Moon's signs ([`omen::day_markers`], `03-design/day-markers.md`
    /// §3): ganda mula over each of the six junction stars, gandanta over
    /// two ghatis each side of a junction (Muhurta Chintamani,
    /// Vivaha-prakarana v. 41), and vinchudo while the Moon is in Scorpio,
    /// in the order they begin. Ganda mula's whole star and vinchudo are
    /// printed readings (crux C1051).
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// # // Wednesday 25 November 2026.
    /// # let mut day = almanac.day(&date(11, 25), &kathmandu)?.value;
    /// use teistro_core::catalogue::{DayMarker, Rashi};
    ///
    /// assert!(day.markers.is_empty(), "not asked for");
    /// day.mark();
    /// // Vinchudo holds exactly while the Moon is in Scorpio.
    /// let vinchudo = day.markers.iter().filter(|m| m.member == DayMarker::Vinchudo).count();
    /// assert_eq!(vinchudo, day.moon.signs.iter().filter(|s| s.member == Rashi::Scorpio).count());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn mark(&mut self) {
        self.markers = omen::day_markers(&self.limbs.nakshatra, &self.moon.signs, self.window);
    }

    /// Fills [`Panchanga::gowri`]: the day's eight Gowri of the daylight
    /// and eight of the night ([`period::gowri`],
    /// `03-design/gowri-panchangam.md`), a printed reading (crux C1052).
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// # // Wednesday 25 November 2026.
    /// # let mut day = almanac.day(&date(11, 25), &kathmandu)?.value;
    /// use teistro_core::catalogue::{Gowri, Kaala};
    ///
    /// assert!(day.gowri.is_empty(), "not asked for");
    /// day.divide_gowri()?;
    /// assert_eq!(day.gowri.len(), 16);
    /// // Visham by day is the rahu kaala.
    /// let rahu = day.kaalas.iter().find(|k| k.kaala == Kaala::RahuKaala).ok_or("rahu kaala")?;
    /// assert!(day.gowri.iter().any(|g| g.is_daytime && g.gowri == Gowri::Visham && g.at == rahu.at));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A day whose sunset precedes its sunrise, which a built day never is.
    pub fn divide_gowri(&mut self) -> Result<(), Error> {
        let daylight = Interval::new(self.day.sunrise, self.day.sunset)?;
        let night = Interval::new(self.day.sunset, self.day.next_sunrise)?;
        self.gowri = period::gowri(daylight, night, self.day.vara);
        Ok(())
    }

    /// The directions Rahu walks over the daylight, an eighth of sunrise
    /// to sunset each, from the east ([`omen::rahu_vasas`],
    /// *Jyotirnibandha* p. 337); none at night, and none on a day with no
    /// daylight.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// # // Wednesday 25 November 2026.
    /// # let day = almanac.day(&date(11, 25), &kathmandu)?.value;
    /// # let sunrise = day.day.sunrise;
    ///
    /// // Eight from sunrise, the first in the east, the last ending at sunset.
    /// let walk = day.rahu_vasa();
    /// assert_eq!(walk.len(), 8);
    /// assert_eq!(walk[0].whole.from, sunrise);
    /// assert!((walk[7].whole.to.get() - day.day.sunset.get()).abs() < 1e-9);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn rahu_vasa(&self) -> Vec<Span<Direction>> {
        let daylight = Interval::literal(
            self.day.sunrise.get(),
            self.day.sunset.get().max(self.day.sunrise.get()),
        );
        omen::rahu_vasas(daylight, self.window)
    }

    /// Whose chandrashtama the day holds: each member is a **natal** Moon
    /// sign, and its intervals the Moon's run through the eighth sign from
    /// it ([`omen::chandrashtama_of`]).
    ///
    /// ```no_run
    /// # fn day() -> teistro_panchanga::almanac::Panchanga { unimplemented!() }
    /// for span in day().chandrashtama() {
    ///     println!("chandrashtama for a {:?} Moon until {}", span.member, span.inside.to);
    /// }
    /// ```
    #[must_use]
    pub fn chandrashtama(&self) -> Vec<Span<Rashi>> {
        omen::chandrashtama(&self.moon.signs)
    }

    /// Whether an instant falls inside one of the day's durmuhurtas.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// # // Wednesday 25 November 2026.
    /// # let day = almanac.day(&date(11, 25), &kathmandu)?.value;
    /// # let sunrise = day.day.sunrise;
    ///
    /// for durmuhurta in &day.durmuhurtas {
    ///     assert!(day.is_durmuhurta(durmuhurta.at.from));
    /// }
    /// assert!(!day.is_durmuhurta(day.day.next_sunrise));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn is_durmuhurta(&self, instant: JulianDay<Utc>) -> bool {
        self.durmuhurtas
            .iter()
            .any(|durmuhurta| durmuhurta.at.contains(instant))
    }

    /// The tithi running at an instant.
    ///
    /// One of **four**, one a moving limb, and for a while there were two:
    /// the tithi and the nakshatra could be asked for at an instant and
    /// the yoga and the karana could not, though the data and `span::at`
    /// were both already here. A caller wanting the limbs at a birth had
    /// to reach into `limbs` for half of them.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// # // Wednesday 25 November 2026.
    /// # let day = almanac.day(&date(11, 25), &kathmandu)?.value;
    /// # let sunrise = day.day.sunrise;
    ///
    /// assert_eq!(day.tithi_at(sunrise), day.limbs.tithi.first());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn tithi_at(
        &self,
        instant: JulianDay<Utc>,
    ) -> Option<&Span<teistro_core::catalogue::Tithi>> {
        span::at(&self.limbs.tithi, instant)
    }

    /// The nakshatra the Moon was in at an instant.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// # // Wednesday 25 November 2026.
    /// # let day = almanac.day(&date(11, 25), &kathmandu)?.value;
    /// # let sunrise = day.day.sunrise;
    ///
    /// assert_eq!(day.nakshatra_at(sunrise), day.limbs.nakshatra.first());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn nakshatra_at(
        &self,
        instant: JulianDay<Utc>,
    ) -> Option<&Span<teistro_core::catalogue::Nakshatra>> {
        span::at(&self.limbs.nakshatra, instant)
    }

    /// The nitya yoga running at an instant.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// # // Wednesday 25 November 2026.
    /// # let day = almanac.day(&date(11, 25), &kathmandu)?.value;
    /// # let sunrise = day.day.sunrise;
    ///
    /// assert_eq!(day.yoga_at(sunrise), day.limbs.yoga.first());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn yoga_at(&self, instant: JulianDay<Utc>) -> Option<&Span<teistro_core::catalogue::Yoga>> {
        span::at(&self.limbs.yoga, instant)
    }

    /// The karana running at an instant.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// # // Wednesday 25 November 2026.
    /// # let day = almanac.day(&date(11, 25), &kathmandu)?.value;
    /// # let sunrise = day.day.sunrise;
    ///
    /// assert_eq!(day.karana_at(sunrise), day.limbs.karana.first());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn karana_at(
        &self,
        instant: JulianDay<Utc>,
    ) -> Option<&Span<teistro_core::catalogue::Karana>> {
        span::at(&self.limbs.karana, instant)
    }

    /// The hora running at an instant.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// # // Wednesday 25 November 2026.
    /// # let day = almanac.day(&date(11, 25), &kathmandu)?.value;
    /// # let sunrise = day.day.sunrise;
    ///
    /// // The day's first hora, Mercury's on a Wednesday.
    /// assert_eq!(day.hora_at(sunrise), day.horas.first());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn hora_at(&self, instant: JulianDay<Utc>) -> Option<&Hora> {
        self.horas
            .iter()
            .find(|hora| instant.get() >= hora.start.get() && instant.get() < hora.end.get())
            .or_else(|| {
                self.horas
                    .last()
                    .filter(|hora| instant.get() <= hora.end.get())
            })
    }

    /// The choghadiya running at an instant.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// # // Wednesday 25 November 2026.
    /// # let day = almanac.day(&date(11, 25), &kathmandu)?.value;
    /// # let sunrise = day.day.sunrise;
    ///
    /// let first = day.choghadiya_at(sunrise).ok_or("a daytime choghadiya")?;
    /// assert!(first.is_daytime);
    /// assert_eq!(Some(first), day.choghadiya.first());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn choghadiya_at(&self, instant: JulianDay<Utc>) -> Option<&Part> {
        period::choghadiya_at(&self.choghadiya, instant)
    }

    /// Whether an instant falls inside one of the day's inauspicious
    /// eighths.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// # // Wednesday 25 November 2026.
    /// # let day = almanac.day(&date(11, 25), &kathmandu)?.value;
    /// # let sunrise = day.day.sunrise;
    ///
    /// for kaala in &day.kaalas {
    ///     assert!(day.is_inauspicious(kaala.at.from));
    /// }
    /// assert!(!day.is_inauspicious(sunrise) || day.kaalas.iter().any(|k| k.at.from == sunrise));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn is_inauspicious(&self, instant: JulianDay<Utc>) -> bool {
        period::is_inauspicious(&self.kaalas, instant)
    }
}

/// What the almanac hashes as its input.
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct Input {
    date: String,
    latitude_deg: f64,
    longitude_deg: f64,
    altitude_m: f64,
}

/// What a sankalpa's answer is a function of.
#[derive(Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct SankalpaInput {
    at: f64,
    latitude_deg: f64,
    longitude_deg: f64,
    altitude_m: f64,
    ayana: AyanaReckoning,
}

/// What an ashauch's answer is a function of.
#[derive(Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct AshauchInput<'r> {
    table: &'r AshauchTable,
    request: &'r AshauchRequest,
    latitude_deg: f64,
    longitude_deg: f64,
    altitude_m: f64,
}

/// What a household's answer is a function of.
#[derive(Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct HouseholdInput<'r> {
    table: &'r AshauchTable,
    household: &'r Household,
    latitude_deg: f64,
    longitude_deg: f64,
    altitude_m: f64,
}

/// What a death-time check's answer is a function of.
#[derive(Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct DeathTimeInput<'r> {
    table: &'r DeathTimeTable,
    at: f64,
    cremated_at: Option<f64>,
    latitude_deg: f64,
    longitude_deg: f64,
    altitude_m: f64,
}

/// What a first year's answer is a function of.
#[derive(Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct FirstYearInput<'r> {
    request: &'r FirstYear,
    latitude_deg: f64,
    longitude_deg: f64,
    altitude_m: f64,
}

/// How long a moment is, days, over which a body's sign at an instant is
/// read: a tenth of a second, inside which no body changes sign but on
/// the instant itself.
const MOMENT_DAYS: f64 = 1e-6;

/// How near a karana's start must be to its tithi's to be its first: a
/// second, where the two searches agree to a few microseconds and a
/// karana lasts some six hours.
const HALF_TOLERANCE_DAYS: f64 = 1.0 / 86_400.0;

/// The same, for a range.
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct RangeInput {
    from: String,
    to: String,
    latitude_deg: f64,
    longitude_deg: f64,
    altitude_m: f64,
}

/// How many days the range from `first` to `last` holds, inclusive,
/// refused as every range is: one that ends before it begins, or holds
/// more than [`MOST_DAYS`], named by `to`.
///
/// # Errors
///
/// `INVALID_ARG` for a range the wrong way round, `OUT_OF_RANGE` for one
/// too long.
fn days_in(
    first: FixedDay,
    last: FixedDay,
    from: &CalendarDate,
    to: &CalendarDate,
    most: usize,
) -> Result<usize, Error> {
    let days = first.days_until(last);
    if days < 0 {
        return Err(Error::invalid_arg(format!(
            "a range of days ends before it begins: {from} to {to}"
        ))
        .with_field("to"));
    }
    let count = usize::try_from(days).unwrap_or(0) + 1;
    if count > most {
        return Err(Error::new(
            Status::OutOfRange,
            format!("a range holds at most {most} days, not {count}"),
        )
        .with_field("to"));
    }
    Ok(count)
}

/// The most days one range may hold.
///
/// A year and a day: an application asking for more is asking for a
/// different shape, and the refusal names the limit rather than running
/// for a minute.
pub const MOST_DAYS: usize = 366;

/// The most days one span of festival days may hold: a century of
/// anniversaries, and the lunation and days around it its edges read
/// (`03-design/anniversary-span.md` §3). Only [`Almanac::festival_span`]
/// reads this far.
pub const MOST_SPAN_DAYS: usize = 36_600;

/// The almanac over a provider, a solar model for the day's arcs, a
/// calendar and a clock.
///
/// The same collaborators the chart foundation takes, because it is the
/// same day model underneath.
///
/// ```
/// use teistro_astro::delta_t::DeltaTModel;
/// use teistro_astro::precession::PrecessionModel;
/// use teistro_calendar::solar::drik::DrikSun;
/// use teistro_calendar::{CalendarDate, Gregorian};
/// use teistro_core::catalogue::{Ayanamsha, Calendar};
/// use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
/// use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
/// use teistro_core::time::UtcOffset;
/// use teistro_panchanga::Almanac;
/// use teistro_port_ephemeris::TestProvider;
///
/// // Kathmandu on the Nepal clock, over the analytic test provider: fixed
/// // elements that need no engine. A real provider reads the real sky.
/// let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
/// let provider = TestProvider;
/// let resolved = Profile::shipped(DEFAULT_PROFILE)
///     .ok_or("a shipped profile")?
///     .resolve(&SettingsPatch::default())
///     .map_err(|d| format!("{d:?}"))?;
/// let (delta_t, precession) = (DeltaTModel::TableThenModel, PrecessionModel::Vondrak2011);
/// let model = DrikSun::new(&provider, Ayanamsha::Lahiri, Sunrise::CentreNoRefraction.into(), OverridePolicy::PreferNative, delta_t);
/// let clock = UtcOffset::literal(5, 45, 0);
/// let almanac = Almanac::new(&provider, &resolved, &model, &Gregorian, &clock, precession, delta_t);
/// let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
///
/// // A day, three days, and a range's festival days, each stamped.
/// let day = almanac.day(&date(11, 25), &kathmandu)?;
/// let days = almanac.between(&date(11, 25), &date(11, 27), &kathmandu)?;
/// assert_eq!(days.value.len(), 3);
/// assert_eq!(days.value[0], day.value);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct Almanac<'a, P: EphemerisProvider + ?Sized> {
    provider: &'a P,
    resolved: &'a Resolved,
    model: &'a dyn SolarModel,
    calendar: &'a dyn CalendarSystem,
    clock: &'a dyn LocalClock,
    precession: PrecessionModel,
    delta_t: DeltaTModel,
}

impl<P: EphemerisProvider + ?Sized> core::fmt::Debug for Almanac<'_, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Almanac")
            .field("profile", &self.resolved.profile.as_str())
            .field("model", &self.model.describe())
            .finish_non_exhaustive()
    }
}

impl<'a, P: EphemerisProvider + ?Sized> Almanac<'a, P> {
    /// An almanac.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    ///
    /// // Every collaborator is borrowed; the almanac is a view over them.
    /// assert!(format!("{almanac:?}").starts_with("Almanac"));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub const fn new(
        provider: &'a P,
        resolved: &'a Resolved,
        model: &'a dyn SolarModel,
        calendar: &'a dyn CalendarSystem,
        clock: &'a dyn LocalClock,
        precession: PrecessionModel,
        delta_t: DeltaTModel,
    ) -> Almanac<'a, P> {
        Almanac {
            provider,
            resolved,
            model,
            calendar,
            clock,
            precession,
            delta_t,
        }
    }

    /// The resolved settings every part reads.
    const fn settings(&self) -> &Settings {
        &self.resolved.settings
    }

    /// The almanac of one date at one place.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    ///
    /// let day = almanac.day(&date(11, 25), &kathmandu)?;
    /// assert_eq!((day.value.day.date.month, day.value.day.date.day), (11, 25));
    /// // The same inputs, the same hash.
    /// assert_eq!(day.provenance, almanac.day(&date(11, 25), &kathmandu)?.provenance);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// The day's own refusals (a polar day under `UNDEFINED`, a date the
    /// calendar does not have), an ayanamsha the catalogue cannot
    /// evaluate, a provider that cannot answer for the window, or a
    /// muhurta yoga or durmuhurta table the SDK does not ship.
    pub fn day(&self, date: &CalendarDate, place: &Place) -> Result<Envelope<Panchanga>, Error> {
        let (value, frame) = self.value(date, place, &mut Memo::default())?;
        let mut provenance = self.provenance_from(
            content_hash(&Input {
                date: date.to_string(),
                latitude_deg: place.latitude.get(),
                longitude_deg: place.longitude.get(),
                altitude_m: place.altitude.get(),
            }),
            frame,
            self.calendar.fixed_of(date)?,
        );
        provenance
            .applied_conventions
            .extend(omen::applied_conventions(&self.settings().panchanga));
        Ok(Envelope::new(value, provenance))
    }

    /// The almanac of the day an instant belongs to, which before sunrise
    /// is not the day of its civil date.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// # // Wednesday 25 November 2026.
    /// # let day = almanac.day(&date(11, 25), &kathmandu)?.value;
    /// # let sunrise = day.day.sunrise;
    ///
    /// // A minute before sunrise is the day before's.
    /// let before = almanac.at(JulianDay::literal(sunrise.get() - 1.0 / 1440.0), &kathmandu)?.value;
    /// assert_eq!(before.day.date.day, 24);
    /// assert_eq!(almanac.at(sunrise, &kathmandu)?.value, day);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`Almanac::day`].
    pub fn at(&self, instant: JulianDay<Utc>, place: &Place) -> Result<Envelope<Panchanga>, Error> {
        let day = teistro_chart::day::chart_day(
            self.model,
            self.calendar,
            self.clock,
            place,
            instant,
            self.settings().day.polar_day_policy,
        )?;
        self.day(&day.day.date, place)
    }

    /// The almanac of every day in a range, inclusive of both ends.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    ///
    /// let days = almanac.between(&date(11, 25), &date(11, 27), &kathmandu)?.value;
    /// // Each day ends at the next one's sunrise.
    /// assert!(days.windows(2).all(|pair| pair[0].day.next_sunrise == pair[1].day.sunrise));
    /// // Backwards is refused.
    /// assert!(almanac.between(&date(11, 27), &date(11, 25), &kathmandu).is_err());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`Almanac::day`], plus `OUT_OF_RANGE` for a range that ends
    /// before it begins or holds more than [`MOST_DAYS`] days.
    pub fn between(
        &self,
        from: &CalendarDate,
        to: &CalendarDate,
        place: &Place,
    ) -> Result<Envelope<Vec<Panchanga>>, Error> {
        let mut days = self.run(from, to, place, Moon::Searched, Self::value)?;
        days.provenance
            .applied_conventions
            .extend(omen::applied_conventions(&self.settings().panchanga));
        Ok(days)
    }

    /// What festival rules read of every day in a range, inclusive of
    /// both ends ([`FestivalDay`]): its sunrise, sunset and next sunrise,
    /// its lunar month and its tithis and nakshatras, **each the value
    /// [`Almanac::between`] gives that day**, to the bit, and stamped as
    /// it stamps the range. The day's other parts are not computed, the
    /// Moon's rises and sets least of all, which are most of a day's
    /// requests: a year of festivals is a year of these.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    ///
    /// let days = almanac.between(&date(11, 25), &date(11, 26), &kathmandu)?.value;
    /// let festival = almanac.festival_days(&date(11, 25), &date(11, 26), &kathmandu)?.value;
    /// // The same sunrises and tithis, to the bit.
    /// assert_eq!(festival[0].sunrise, days[0].day.sunrise);
    /// assert_eq!(festival[1].tithi, days[1].limbs.tithi);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`Almanac::between`], less the refusals of the parts it does not
    /// compute.
    pub fn festival_days(
        &self,
        from: &CalendarDate,
        to: &CalendarDate,
        place: &Place,
    ) -> Result<Envelope<Vec<FestivalDay>>, Error> {
        self.run(from, to, place, Moon::Unread, Self::festival_value)
    }

    /// As [`Almanac::festival_days`], up to [`MOST_SPAN_DAYS`]: one run of
    /// days, founded a range at a time. A range's rounds step every day at
    /// once, and a provider's segment cache is sized to a year of them; one
    /// round over a decade thrashes it, 37 s against 1.3 s on the built-in
    /// ephemeris (`03-design/anniversary-span.md` §5). The provenance is the
    /// first range's.
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
    /// # let date = |year, month, day| CalendarDate::defined(Calendar::Gregorian, year, month, day);
    /// // Two years, past a range's limit of 366 days.
    /// let days = almanac.festival_span(&date(2026, 1, 1), &date(2027, 12, 31), &kathmandu)?.value;
    /// assert_eq!(days.len(), 730);
    /// assert!(days.windows(2).all(|pair| pair[0].sunrise.get() < pair[1].sunrise.get()));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`Almanac::festival_days`], the range's limit being
    /// [`MOST_SPAN_DAYS`].
    pub fn festival_span(
        &self,
        from: &CalendarDate,
        to: &CalendarDate,
        place: &Place,
    ) -> Result<Envelope<Vec<FestivalDay>>, Error> {
        let (first, last) = (self.calendar.fixed_of(from)?, self.calendar.fixed_of(to)?);
        let count = days_in(first, last, from, to, MOST_SPAN_DAYS)?;
        let mut days = Vec::with_capacity(count);
        let mut provenance = None;
        for start in (0..count).step_by(MOST_DAYS) {
            let end = (start + MOST_DAYS).min(count) - 1;
            let at = |step: usize| {
                self.calendar
                    .date_of(first.plus_days(i64::try_from(step).unwrap_or(0)))
            };
            let range = self.festival_days(&at(start)?, &at(end)?, place)?;
            provenance.get_or_insert(range.provenance);
            days.extend(range.value);
        }
        let Some(provenance) = provenance else {
            return Err(Error::internal("a span of days answered no day"));
        };
        Ok(Envelope::new(days, provenance))
    }

    /// Every instant in a range at which a limb, the Moon's or the Sun's
    /// sign or the vara changes, one sorted list
    /// (`docs/03-design/change-points.md`). The range's own walker over its
    /// shared crossings, reading no rise or set.
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
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// use teistro_panchanga::change::ChangeKind;
    ///
    /// let changes = almanac.changes(&date(11, 25), &date(11, 26), &kathmandu)?.value;
    /// assert!(changes.points.windows(2).all(|pair| pair[0].at.get() <= pair[1].at.get()));
    /// // The second day's sunrise turns the vara, once.
    /// assert_eq!(changes.points.iter().filter(|point| point.kind == ChangeKind::Vara).count(), 1);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`Almanac::between`], less the refusals of the parts it does not
    /// compute.
    pub fn changes(
        &self,
        from: &CalendarDate,
        to: &CalendarDate,
        place: &Place,
    ) -> Result<Envelope<ChangePoints>, Error> {
        let days = self.run(from, to, place, Moon::Unread, Self::change_value)?;
        let points = ChangePoints::of(&days.value)?;
        Ok(Envelope::new(points, days.provenance))
    }

    /// Fills each day's first moonrise between its sunrise and the next
    /// ([`FestivalDay::moonrise`]), for the rules that read one
    /// ([`crate::festival::Window::Moonrise`]): every day's search in
    /// lockstep, so a round of them is one request. The search is the one
    /// [`Almanac::between`] makes over a sunrise day, so a day's moonrise
    /// is its panchanga's first, to the bit. A polar day is left
    /// unsearched, since it has no windows.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    ///
    /// use teistro_panchanga::festival::Moonrise;
    ///
    /// let mut days = almanac.festival_days(&date(11, 25), &date(11, 27), &kathmandu)?.value;
    /// assert_eq!(days[0].moonrise, Moonrise::Unsearched);
    /// almanac.moonrises(&mut days, &kathmandu)?;
    /// assert!(days.iter().all(|day| day.moonrise != Moonrise::Unsearched));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A search the provider refuses.
    pub fn moonrises(&self, days: &mut [FestivalDay], place: &Place) -> Result<(), Error> {
        let completion = Completion::new(
            self.provider,
            self.settings().provider.overrides,
            self.delta_t,
        );
        let solver = self.moon_solver(&completion, place);
        let round = Lockstep::new();
        let searched: Vec<Interval> = days
            .iter()
            .filter(|day| day.normal)
            .map(FestivalDay::sunrise_day)
            .collect();
        let mut found = round
            .run(searched.iter().map(|window| {
                solver.event_in(
                    &round,
                    HorizonEventKind::Rise,
                    JulianDay::<Ut1>::literal(window.from.get()),
                    window.days(),
                )
            }))
            .into_iter();
        for day in days.iter_mut().filter(|day| day.normal) {
            let event = found
                .next()
                .ok_or_else(|| Error::internal("a moonrise search came apart"))??;
            day.moonrise = match event {
                Some(event) if event.instant.get() < day.next_sunrise.get() => {
                    Moonrise::Rose(MoonriseAt {
                        at: JulianDay::literal(event.instant.get()),
                        place: *place,
                    })
                }
                _ => Moonrise::NoRise,
            };
        }
        Ok(())
    }

    /// Fills each day's entry of the sidereal Sun into a nakshatra
    /// ([`FestivalDay::sun_entry`]), for the rules kept on one
    /// ([`crate::festival::Dated::SunEnters`]): the Sun's nakshatra at each
    /// local midnight on the clock, in the same sky and zodiac a day's
    /// sankranti is found in, so two midnights' readings say whether the
    /// civil day between them held an entry. Neighbouring days share a
    /// midnight, so a range of `n` days reads the Sun `n + 1` times at
    /// most.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    ///
    /// use teistro_panchanga::festival::SunEntry;
    ///
    /// let mut days = almanac.festival_days(&date(11, 25), &date(11, 27), &kathmandu)?.value;
    /// almanac.sun_entries(&mut days, &kathmandu)?;
    /// assert!(days.iter().all(|day| day.sun_entry != SunEntry::Unsearched));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A day its calendar cannot convert, or a reading the provider
    /// refuses.
    pub fn sun_entries(&self, days: &mut [FestivalDay], place: &Place) -> Result<(), Error> {
        let completion = Completion::new(
            self.provider,
            self.settings().provider.overrides,
            self.delta_t,
        );
        let at_midnight = |fixed: FixedDay| -> Result<Nakshatra, Error> {
            let midnight = local_midnight(self.clock, fixed)?;
            let (frame, zodiac) = self.at_instant(&completion, midnight)?;
            let mut longitudes = completion.longitudes(frame);
            if frame.centre == teistro_port_ephemeris::Centre::Topocentric {
                longitudes = longitudes.with_observer(*place);
            }
            limb::sun_nakshatra_at(&longitudes, midnight, zodiac)
        };
        let mut read: Option<(FixedDay, Nakshatra)> = None;
        for day in days.iter_mut() {
            let fixed = self.calendar.fixed_of(&day.date)?;
            let opening = match read {
                Some((at, nakshatra)) if at == fixed => nakshatra,
                _ => at_midnight(fixed)?,
            };
            let next = fixed.plus_days(1);
            let closing = at_midnight(next)?;
            read = Some((next, closing));
            day.sun_entry = if closing == opening {
                SunEntry::Stayed
            } else {
                SunEntry::Entered(closing)
            };
        }
        Ok(())
    }

    /// Fills each day's homa ahuti ([`Panchanga::homa_ahuti`]): the Sun's
    /// nakshatras over the day, searched in the sky and zodiac the day's
    /// own limbs were read in, laid against the Moon's
    /// ([`omen::homa_ahuti`], crux C751).
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// let mut days = almanac.between(&date(11, 25), &date(11, 26), &kathmandu)?.value;
    /// assert!(days[0].homa_ahuti.is_empty(), "not asked for");
    /// almanac.homa_ahuti(&mut days, &kathmandu)?;
    /// // At least one share for each star the Moon held in the day.
    /// assert!(days.iter().all(|day| day.homa_ahuti.len() >= day.limbs.nakshatra.len()));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A reading the provider refuses.
    pub fn homa_ahuti(&self, days: &mut [Panchanga], place: &Place) -> Result<(), Error> {
        let completion = Completion::new(
            self.provider,
            self.settings().provider.overrides,
            self.delta_t,
        );
        for day in days.iter_mut() {
            let (frame, zodiac) = self.at_instant(&completion, day.window.from)?;
            let mut longitudes = completion.longitudes(frame);
            if frame.centre == teistro_port_ephemeris::Centre::Topocentric {
                longitudes = longitudes.with_observer(*place);
            }
            let sun = limb::nakshatras_of(&longitudes, Body::Sun, day.window, zodiac)?;
            day.homa_ahuti = omen::homa_ahuti(&day.limbs.nakshatra, &sun, day.window);
        }
        Ok(())
    }

    /// The adhika lunations among `days` that are the *samsarpa*, the
    /// adhika month before a kshaya month (*Dharmasindhu* p. 3, C179), each
    /// as its span from new moon to new moon, for the anniversaries kept in
    /// it too (C669). The kshaya month is looked for a lunation at a time
    /// past the adhika, up to [`SAMSARPA_REACH_MONTHS`], in the sky and by
    /// the rule the days' own months were named by, so a range ending
    /// before the kshaya month still knows its samsarpa.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    ///
    /// use teistro_calendar::lunisolar::MonthKind;
    ///
    /// // Late November holds no adhika month, so no samsarpa.
    /// let days = almanac.festival_days(&date(11, 25), &date(11, 27), &kathmandu)?.value;
    /// assert!(days.iter().all(|day| day.month.kind == MonthKind::Nija));
    /// assert!(almanac.samsarpas(&days, &kathmandu)?.is_empty());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A search the provider refuses.
    pub fn samsarpas(&self, days: &[FestivalDay], place: &Place) -> Result<Vec<Interval>, Error> {
        let completion = Completion::new(
            self.provider,
            self.settings().provider.overrides,
            self.delta_t,
        );
        let mut found: Vec<Interval> = Vec::new();
        for day in days
            .iter()
            .filter(|day| day.month.kind == MonthKind::Adhika)
        {
            if found.iter().any(|span| {
                span.from.get() <= day.sunrise.get() && day.sunrise.get() < span.to.get()
            }) {
                continue;
            }
            let (frame, zodiac) = self.at_instant(&completion, day.sunrise)?;
            let mut longitudes = completion.longitudes(frame);
            if frame.centre == teistro_port_ephemeris::Centre::Topocentric {
                longitudes = longitudes.with_observer(*place);
            }
            let lunation_at = |at: f64| {
                let at = JulianDay::try_new(at)?;
                limb::lunar_month_span(&longitudes, Interval::new(at, next_by(at, 1.0))?, zodiac)
            };
            let adhika = lunation_at(day.sunrise.get())?;
            let mut next = adhika;
            for _ in 0..SAMSARPA_REACH_MONTHS {
                // A day past the new moon that closes the month, so the
                // span found is the month after it.
                next = lunation_at(next.to.get() + 1.0)?;
                match teistro_calendar::lunisolar::kind_of(self.model, next.from, next.to)? {
                    MonthKind::Nija => {}
                    MonthKind::Kshaya => {
                        found.push(adhika);
                        break;
                    }
                    // The next adhika month: this one was no samsarpa.
                    MonthKind::Adhika => break,
                }
            }
        }
        Ok(found)
    }

    /// Fills each day's [`FestivalDay::moon_arcs`]: the Moon's stay in each
    /// of `arcs`, sidereal degrees inside the nakshatra named beside them,
    /// for the parana bars that read them (C700). Each stay is searched
    /// at the arc's own bounds, in the sky and zodiac a day's nakshatras
    /// are found in, and only on a day that nakshatra meets; a day keeps
    /// the stays that meet it, sunrise to the next.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    ///
    /// use teistro_panchanga::festival::MoonArcs;
    ///
    /// // No arcs asked: each day is searched for none and holds none.
    /// let mut days = almanac.festival_days(&date(11, 25), &date(11, 27), &kathmandu)?.value;
    /// almanac.moon_arcs(&mut days, &kathmandu, &[])?;
    /// assert!(days.iter().all(|day| day.moon_arcs != MoonArcs::Unsearched));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A search the provider refuses.
    pub fn moon_arcs(
        &self,
        days: &mut [FestivalDay],
        place: &Place,
        arcs: &[(Nakshatra, (f64, f64))],
    ) -> Result<(), Error> {
        let completion = Completion::new(
            self.provider,
            self.settings().provider.overrides,
            self.delta_t,
        );
        let span_deg = 360.0 / 27.0;
        // A stay is searched once though the days beside it meet it too.
        let mut found: Vec<MoonArc> = Vec::new();
        for day in days.iter_mut() {
            let mut kept = Vec::new();
            for &(nakshatra, (from_deg, to_deg)) in arcs {
                for stay in day.nakshatra.iter().filter(|span| span.member == nakshatra) {
                    // Inside the arc's stay: its middle, in proportion
                    // through the nakshatra's own, misses by minutes, and
                    // the shortest share a bar names lasts hours.
                    let start_deg = f64::from(nakshatra.id()) * span_deg;
                    let share = (from_deg + to_deg).mul_add(0.5, -start_deg) / span_deg;
                    let whole = stay.whole;
                    let inside =
                        (whole.to.get() - whole.from.get()).mul_add(share, whole.from.get());
                    let known = found.iter().find(|arc| {
                        arc.is_of((from_deg, to_deg))
                            && arc.span.from.get() <= inside
                            && inside <= arc.span.to.get()
                    });
                    let arc = if let Some(&arc) = known {
                        arc
                    } else {
                        let at = JulianDay::<Utc>::literal(inside);
                        let (frame, zodiac) = self.at_instant(&completion, at)?;
                        let mut longitudes = completion.longitudes(frame);
                        if frame.centre == teistro_port_ephemeris::Centre::Topocentric {
                            longitudes = longitudes.with_observer(*place);
                        }
                        let span = limb::moon_between(&longitudes, at, zodiac, (from_deg, to_deg))?;
                        let arc = MoonArc {
                            from_deg,
                            to_deg,
                            span,
                        };
                        found.push(arc);
                        arc
                    };
                    let meets = arc.span.to.get() > day.sunrise.get()
                        && arc.span.from.get() < day.next_sunrise.get();
                    if meets && !kept.contains(&arc) {
                        kept.push(arc);
                    }
                }
            }
            day.moon_arcs = MoonArcs::Found(kept);
        }
        Ok(())
    }

    /// Every day of a range as `value` computes it, the range found
    /// ahead and the days carried one to the next, under one stamp.
    fn run<T>(
        &self,
        from: &CalendarDate,
        to: &CalendarDate,
        place: &Place,
        moon: Moon,
        value: impl Fn(&Self, &CalendarDate, &Place, &mut Memo) -> Result<(T, Frame), Error>,
    ) -> Result<Envelope<Vec<T>>, Error> {
        let (first, last) = (self.calendar.fixed_of(from)?, self.calendar.fixed_of(to)?);
        let count = days_in(first, last, from, to, MOST_DAYS)?;
        let mut values = Vec::with_capacity(count);
        // The frame is the first day's: the batch asks the provider for
        // one frame, so one is what the stamp names -- and it comes back
        // from the day that computed it rather than being computed
        // again.
        let mut frame = None;
        // What one day found and the next can read rather than search
        // again: the day before's sunrise and sunset, and the lunation.
        let mut memo = Memo {
            last: Some(last),
            ..Memo::default()
        };
        self.ahead(first, count, place, moon, &mut memo);
        for step in 0..count {
            let offset = i64::try_from(step).unwrap_or(0);
            let date = self.calendar.date_of(first.plus_days(offset))?;
            let (value, asked) = value(self, &date, place, &mut memo)?;
            frame = frame.or(Some(asked));
            values.push(value);
        }
        let Some(frame) = frame else {
            return Err(Error::internal("a range of days answered no day"));
        };
        let provenance = self.provenance_from(
            content_hash(&RangeInput {
                from: from.to_string(),
                to: to.to_string(),
                latitude_deg: place.latitude.get(),
                longitude_deg: place.longitude.get(),
                altitude_m: place.altitude.get(),
            }),
            frame,
            first,
        );
        Ok(Envelope::new(values, provenance))
    }

    /// The day an instant falls in, sunrise to sunrise: its civil day's,
    /// or the day before's when the instant is before that day's sunrise.
    fn holding(&self, at: JulianDay<Utc>, place: &Place) -> Result<(Panchanga, Frame), Error> {
        let (civil, _) = FixedDay::from_local_jd(self.clock.local_jd(at));
        for back in [0, 1] {
            let date = self.calendar.date_of(civil.plus_days(-back))?;
            let (day, frame) = self.value(&date, place, &mut Memo::default())?;
            if day.window.from.get() <= at.get() && at.get() < day.window.to.get() {
                return Ok((day, frame));
            }
        }
        Err(Error::new(
            Status::OutOfRange,
            format!(
                "the instant {at} falls in neither its civil day's window nor the day before's"
            ),
        ))
    }

    /// An anniversary's origin as found at a place (`03-design/shraddha.md`
    /// §2.1, C661, C668): the tithi at the moment, the lunation holding it
    /// and its kind, the half of the tithi, and the nakshatra; for a date
    /// alone, its sunrise tithi, marked as assumed, with every tithi the
    /// date held midnight to midnight. A lunar origin is taken as given.
    ///
    /// The lunation is the one holding the moment, not the one at the
    /// day's sunrise: a death after the day's new moon falls in the next
    /// month. In a kshaya lunation the tithi's half names the month, the
    /// first half the lunation's first name and the second its second.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// # // Wednesday 25 November 2026.
    /// # let day = almanac.day(&date(11, 25), &kathmandu)?.value;
    /// # let sunrise = day.day.sunrise;
    ///
    /// use teistro_panchanga::festival::Origin;
    ///
    /// // A death at noon: the tithi then running, and the instant kept.
    /// let noon = JulianDay::literal(sunrise.get() + 0.25);
    /// let found = almanac.origin(&Origin::Instant { at: noon }, &kathmandu)?;
    /// assert_eq!(Some(found.tithi), day.tithi_at(noon).map(|span| span.member));
    /// assert_eq!(found.at, Some(noon));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`Almanac::day`] for the day holding the moment, the day after it
    /// and, for a date, the day before.
    pub fn origin(&self, origin: &Origin, place: &Place) -> Result<OriginFound, Error> {
        match origin {
            Origin::Instant { at } => {
                let (day, _) = self.holding(*at, place)?;
                self.found_at(&day, *at, place, false, Vec::new())
            }
            Origin::Date { date } => {
                let date = &date.date();
                let (day, _) = self.value(date, place, &mut Memo::default())?;
                let fixed = self.calendar.fixed_of(date)?;
                let civil = Interval::new(
                    local_midnight(self.clock, fixed)?,
                    local_midnight(self.clock, fixed.plus_days(1))?,
                )?;
                let (before, _) = self.value(
                    &self.calendar.date_of(fixed.plus_days(-1))?,
                    place,
                    &mut Memo::default(),
                )?;
                let mut held: Vec<Tithi> = Vec::new();
                for span in before.limbs.tithi.iter().chain(&day.limbs.tithi) {
                    if span.whole.from.get() < civil.to.get()
                        && civil.from.get() < span.whole.to.get()
                        && !held.contains(&span.member)
                    {
                        held.push(span.member);
                    }
                }
                if held.len() < 2 {
                    held.clear();
                }
                self.found_at(&day, day.day.sunrise, place, true, held)
            }
            Origin::Lunar {
                month,
                tithi,
                adhika,
                half,
                at,
            } => Ok(OriginFound::lunar(*month, *tithi, *adhika, *half, *at)),
        }
    }

    /// The origin at an instant inside a day.
    fn found_at(
        &self,
        day: &Panchanga,
        at: JulianDay<Utc>,
        place: &Place,
        assumed_sunrise: bool,
        candidates: Vec<Tithi>,
    ) -> Result<OriginFound, Error> {
        let limb = |name: &str| {
            Error::internal(format!(
                "the day's {name} spans do not hold the instant {at}"
            ))
        };
        let holds = |span: &&Span<Tithi>| {
            span.whole.from.get() <= at.get() && at.get() < span.whole.to.get()
        };
        let tithi = day
            .limbs
            .tithi
            .iter()
            .find(holds)
            .ok_or_else(|| limb("tithi"))?;
        let at_sunrise =
            sankalpa::at(&day.limbs.tithi, day.day.sunrise).ok_or_else(|| limb("tithi"))?;
        // A new moon between the sunrise and the moment opens the next
        // lunation, whose month the next day's sunrise names.
        let lunation = if tithi.member.id() < at_sunrise.id() {
            let next = self
                .calendar
                .date_of(self.calendar.fixed_of(&day.day.date)?.plus_days(1))?;
            self.value(&next, place, &mut Memo::default())?.0.month
        } else {
            day.month
        };
        // The tithi's first karana begins with it.
        let karana = day
            .limbs
            .karana
            .iter()
            .find(|span| span.whole.from.get() <= at.get() && at.get() < span.whole.to.get())
            .ok_or_else(|| limb("karana"))?;
        let half = if (karana.whole.from.get() - tithi.whole.from.get()).abs() < HALF_TOLERANCE_DAYS
        {
            Half::First
        } else {
            Half::Second
        };
        let month = if lunation.kind == MonthKind::Kshaya && half == Half::Second {
            month::next(lunation.amanta)
        } else {
            lunation.amanta
        };
        Ok(OriginFound {
            tithi: tithi.member,
            month,
            kind: lunation.kind,
            half,
            nakshatra: sankalpa::at(&day.limbs.nakshatra, at),
            at: Some(at),
            assumed_sunrise,
            candidates,
        })
    }

    /// The time a sankalpa recites for a rite begun at `at`
    /// (`03-design/sankalpa.md`): the day's year, ayana, season and month
    /// as its sunrise holds them, and the tithi, nakshatra, yoga, karana
    /// and the Sun's, the Moon's and Jupiter's signs as the instant does.
    ///
    /// The day is the one the instant falls in, sunrise to sunrise, so a
    /// rite before dawn is the day before's. The month and the season are
    /// reckoned as the profile's days are, and the half of the year under
    /// `ayana`; the answer names each.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// # // Wednesday 25 November 2026.
    /// # let day = almanac.day(&date(11, 25), &kathmandu)?.value;
    /// # let sunrise = day.day.sunrise;
    ///
    /// use teistro_panchanga::sankalpa::AyanaReckoning;
    ///
    /// // A rite at noon recites that day's weekday and its year.
    /// let noon = JulianDay::literal(sunrise.get() + 0.25);
    /// let said = almanac.sankalpa(noon, &kathmandu, AyanaReckoning::Sidereal)?.value;
    /// assert_eq!((said.vara, said.sunrise), (day.vara(), sunrise));
    /// assert_eq!(said.year.vikrama - said.year.shaka, 135);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`Almanac::day`] for the instant's day and the one before, and
    /// [`Almanac::years`] for its year.
    pub fn sankalpa(
        &self,
        at: JulianDay<Utc>,
        place: &Place,
        ayana: AyanaReckoning,
    ) -> Result<Envelope<Sankalpa>, Error> {
        let settings = self.settings();
        let (day, frame) = self.holding(at, place)?;
        let year = self
            .lunar_years(&day.day.date, &day.day.date, place, false)?
            .value
            .into_iter()
            .next()
            .ok_or_else(|| Error::internal("a day's own range answered no lunar year"))?;
        let completion = Completion::new(self.provider, settings.provider.overrides, self.delta_t);
        let (instant_frame, zodiac) = self.at_instant(&completion, at)?;
        let mut longitudes = completion.longitudes(instant_frame);
        if instant_frame.centre == teistro_port_ephemeris::Centre::Topocentric {
            longitudes = longitudes.with_observer(*place);
        }
        let sign = |body: Body, from: JulianDay<Utc>, zodiac: Zodiac| -> Result<Rashi, Error> {
            Self::signs(
                &longitudes,
                body,
                Interval::new(from, next_by(from, MOMENT_DAYS))?,
                zodiac,
            )?
            .first()
            .map(|span| span.member)
            .ok_or_else(|| Error::internal(format!("{body:?} stood in no sign at {from}")))
        };
        let ayana_at_sunrise = match ayana {
            AyanaReckoning::Sidereal => day.sun.ayana,
            AyanaReckoning::Tropical => sky::ayana_of(sign(
                Body::Sun,
                day.day.sunrise,
                Zodiac {
                    ayanamsha: None,
                    ..zodiac
                },
            )?),
        };
        let limb = |name: &str| {
            Error::internal(format!(
                "the day's {name} spans do not hold the instant {at}"
            ))
        };
        let tithi = sankalpa::at(&day.limbs.tithi, at).ok_or_else(|| limb("tithi"))?;
        let value = Sankalpa {
            at,
            year,
            ayana: ayana_at_sunrise,
            ayana_reckoning: ayana,
            ritu: day.sun.ritu,
            ritu_reckoning: settings.panchanga.ritu,
            month: day.month,
            paksha: tithi.attributes().paksha,
            tithi,
            nakshatra: sankalpa::at(&day.limbs.nakshatra, at).ok_or_else(|| limb("nakshatra"))?,
            yoga: sankalpa::at(&day.limbs.yoga, at).ok_or_else(|| limb("yoga"))?,
            karana: sankalpa::at(&day.limbs.karana, at).ok_or_else(|| limb("karana"))?,
            sun: sign(Body::Sun, at, zodiac)?,
            moon: sign(Body::Moon, at, zodiac)?,
            jupiter: sign(Body::Jupiter, at, zodiac)?,
            date: day.day.date,
            vara: day.day.vara,
            sunrise: day.day.sunrise,
            next_sunrise: day.day.next_sunrise,
        };
        let provenance = self.provenance_at(
            content_hash(&SankalpaInput {
                at: at.get(),
                latitude_deg: place.latitude.get(),
                longitude_deg: place.longitude.get(),
                altitude_m: place.altitude.get(),
                ayana,
            }),
            frame,
            Some(at),
        );
        Ok(Envelope::new(value, provenance))
    }

    /// The ashauch `request` asks under `table` at a place
    /// (`03-design/ashauch.md`): the days it reads are found around each
    /// of [`AshauchRequest::anchors`], the day before through
    /// [`AshauchRequest::reach`] days after, runs that meet joined into
    /// one, so news heard years after the event costs two short runs
    /// rather than the years between.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// # // Wednesday 25 November 2026.
    /// # let day = almanac.day(&date(11, 25), &kathmandu)?.value;
    /// # let sunrise = day.day.sunrise;
    ///
    /// use teistro_panchanga::ashauch::{AshauchEvent, AshauchRequest, AshauchTable, Deceased, Nearness, Relation};
    ///
    /// // A parent's death at noon: ten days, the first that day.
    /// let request = AshauchRequest {
    ///     key: String::from("parent"),
    ///     event: AshauchEvent::Death {
    ///         at: JulianDay::literal(sunrise.get() + 0.25),
    ///         cremated_at: None,
    ///         ahitagni: false,
    ///         deceased: Deceased::default(),
    ///     },
    ///     relation: Relation::Child,
    ///     heard_at: None,
    ///     abroad: false,
    ///     nearness: Nearness::SameVillage,
    ///     full_days: None,
    ///     during: Vec::new(),
    ///     fire_giver: false,
    /// };
    /// let found = almanac.ashauch(&AshauchTable::dharmasindhu(), &request, &kathmandu)?.value;
    /// assert_eq!((found.days, found.day_one.day), (Some(10), 25));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`ashauch::ashauch`], and as [`Almanac::festival_days`] for each
    /// run.
    pub fn ashauch(
        &self,
        table: &AshauchTable,
        request: &AshauchRequest,
        place: &Place,
    ) -> Result<Envelope<Ashauch>, Error> {
        table.check()?;
        request.check()?;
        let reach = i64::from(request.reach(table));
        let mut runs: Vec<(FixedDay, FixedDay)> = Vec::new();
        for at in request.anchors() {
            let (civil, _) = FixedDay::from_local_jd(self.clock.local_jd(at));
            let (from, to) = (civil.plus_days(-2), civil.plus_days(reach));
            match runs.last_mut() {
                Some((_, last)) if from <= last.plus_days(1) => *last = (*last).max(to),
                _ => runs.push((from, to)),
            }
        }
        let mut days = Vec::new();
        let mut stamp = None;
        for (from, to) in runs {
            let run = self.festival_days(
                &self.calendar.date_of(from)?,
                &self.calendar.date_of(to)?,
                place,
            )?;
            days.extend(run.value);
            stamp.get_or_insert(run.provenance);
        }
        let mut provenance =
            stamp.ok_or_else(|| Error::internal("an ashauch request named no instant"))?;
        provenance.input_hash = content_hash(&AshauchInput {
            table,
            request,
            latitude_deg: place.latitude.get(),
            longitude_deg: place.longitude.get(),
            altitude_m: place.altitude.get(),
        });
        let value = ashauch::ashauch(table, request, &days, self.clock)?;
        Ok(Envelope::new(value, provenance))
    }

    /// A household's periods at a place (`03-design/ashauch.md` §11):
    /// each observer's ties asked in the order the news reached them,
    /// each with the periods before it running, so the overlaps resolve
    /// as the texts order them, by the knowing.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// # // Wednesday 25 November 2026.
    /// # let day = almanac.day(&date(11, 25), &kathmandu)?.value;
    /// # let sunrise = day.day.sunrise;
    ///
    /// use teistro_panchanga::ashauch::{AshauchTable, Household};
    ///
    /// // A father's death at noon, heard at once by his son.
    /// let household: Household = serde_json::from_value(serde_json::json!({
    ///     "events": [{ "key": "father", "event": { "event": "DEATH", "at": sunrise.get() + 0.25 } }],
    ///     "observers": [{ "key": "son", "ties": [{ "event": "father", "relation": "CHILD" }] }],
    /// }))?;
    /// let found = almanac.household(&AshauchTable::dharmasindhu(), &household, &kathmandu)?.value;
    /// let son = &found.observers[0];
    /// assert_eq!(son.periods[0].days, Some(10));
    /// assert_eq!(son.pure_at, son.periods[0].end);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`Household::check`], and as [`Almanac::ashauch`] for each tie,
    /// a refusal named under its observer.
    pub fn household(
        &self,
        table: &AshauchTable,
        household: &Household,
        place: &Place,
    ) -> Result<Envelope<HouseholdAnswer>, Error> {
        table.check()?;
        household.check()?;
        let mut stamp = None;
        let mut observers = Vec::with_capacity(household.observers.len());
        for (index, observer) in household.observers.iter().enumerate() {
            let mut periods: Vec<Ashauch> = Vec::new();
            for mut request in household.requests(observer) {
                request.during = periods.iter().map(Running::from).collect();
                let found = self
                    .ashauch(table, &request, place)
                    .map_err(|why| why.under(&format!("observers[{index}]")))?;
                stamp.get_or_insert(found.provenance);
                periods.push(found.value);
            }
            let pure_at = periods
                .iter()
                .map(|period| period.end)
                .max_by(|one, other| one.get().total_cmp(&other.get()))
                .ok_or_else(|| Error::internal("a checked observer named no tie"))?;
            observers.push(HouseholdPeriods {
                key: observer.key.clone(),
                periods,
                pure_at,
            });
        }
        let mut provenance: Provenance =
            stamp.ok_or_else(|| Error::internal("a checked household named no tie"))?;
        provenance.input_hash = content_hash(&HouseholdInput {
            table,
            household,
            latitude_deg: place.latitude.get(),
            longitude_deg: place.longitude.get(),
            altitude_m: place.altitude.get(),
        });
        Ok(Envelope::new(HouseholdAnswer { observers }, provenance))
    }

    /// The death-time check at a place (`03-design/death-time.md`): what
    /// stood at a death and, when given, its cremation, and the rites the
    /// texts tie to them. Each instant is read as a sankalpa reads it,
    /// the tithi and nakshatra at the instant and the vara of the sunrise
    /// day holding it.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// # // Wednesday 25 November 2026.
    /// # let day = almanac.day(&date(11, 25), &kathmandu)?.value;
    /// # let sunrise = day.day.sunrise;
    ///
    /// use teistro_panchanga::death_time::DeathTimeTable;
    ///
    /// // A death at noon, the burning left to fall as the death did.
    /// let at = JulianDay::literal(sunrise.get() + 0.25);
    /// let found = almanac.death_time(&DeathTimeTable::dharmasindhu(), at, None, &kathmandu)?.value;
    /// assert_eq!(found.death.at, at);
    /// assert!(found.cremation.is_none());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`Almanac::sankalpa`] for each instant.
    pub fn death_time(
        &self,
        table: &DeathTimeTable,
        at: JulianDay<Utc>,
        cremated_at: Option<JulianDay<Utc>>,
        place: &Place,
    ) -> Result<Envelope<DeathTime>, Error> {
        let read = |instant: JulianDay<Utc>| {
            let Envelope { value, provenance } =
                self.sankalpa(instant, place, AyanaReckoning::Sidereal)?;
            let read = DeathInstant::read(
                table,
                instant,
                value.nakshatra,
                value.tithi,
                value.vara,
                value.moon,
            );
            Ok::<_, Error>((read, provenance))
        };
        let (death, mut provenance) = read(at)?;
        let cremation = cremated_at
            .map(|instant| read(instant).map(|(reading, _)| reading))
            .transpose()?;
        provenance.input_hash = content_hash(&DeathTimeInput {
            table,
            at: at.get(),
            cremated_at: cremated_at.map(JulianDay::get),
            latitude_deg: place.latitude.get(),
            longitude_deg: place.longitude.get(),
            altitude_m: place.altitude.get(),
        });
        let rites = death_time::owed(&death, cremation.as_ref());
        Ok(Envelope::new(
            DeathTime {
                death,
                cremation,
                rites,
                table: table.name.clone(),
            },
            provenance,
        ))
    }

    /// A death's first year at a place (`03-design/shraddha.md` §3.3):
    /// the sixteen shraddhas, the sapindana and the first abdika, over
    /// the run of days [`FirstYear::reach`] names around the death, its
    /// origin the tithi at the instant.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// # // Wednesday 25 November 2026.
    /// # let day = almanac.day(&date(11, 25), &kathmandu)?.value;
    /// # let sunrise = day.day.sunrise;
    ///
    /// use teistro_panchanga::first_year::FirstYear;
    ///
    /// let death = JulianDay::literal(sunrise.get() + 0.25);
    /// let schedule = almanac.first_year(&FirstYear::dharmasindhu("parent", death), &kathmandu)?.value;
    /// assert_eq!(schedule.key, "parent");
    /// assert_eq!(schedule.day_one.day, 25);
    /// assert!(!schedule.rites.is_empty());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`first_year::first_year`], [`Almanac::origin`] and
    /// [`Almanac::festival_days`].
    pub fn first_year(
        &self,
        request: &FirstYear,
        place: &Place,
    ) -> Result<Envelope<FirstYearSchedule>, Error> {
        request.check()?;
        let origin = self.origin(&Origin::Instant { at: request.at }, place)?;
        let (civil, _) = FixedDay::from_local_jd(self.clock.local_jd(request.at));
        let (back, ahead) = FirstYear::reach();
        // Founded in two runs that join, a range holding at most a year.
        let middle = ahead / 2;
        let mut days = self.festival_days(
            &self.calendar.date_of(civil.plus_days(-back))?,
            &self.calendar.date_of(civil.plus_days(middle))?,
            place,
        )?;
        days.value.extend(
            self.festival_days(
                &self.calendar.date_of(civil.plus_days(middle + 1))?,
                &self.calendar.date_of(civil.plus_days(ahead))?,
                place,
            )?
            .value,
        );
        let mut provenance = days.provenance;
        provenance.input_hash = content_hash(&FirstYearInput {
            request,
            latitude_deg: place.latitude.get(),
            longitude_deg: place.longitude.get(),
            altitude_m: place.altitude.get(),
        });
        let value = first_year::first_year(request, &origin, &days.value, self.clock)?;
        Ok(Envelope::new(value, provenance))
    }

    /// The lunar years the days of a range fall in, each named by the
    /// sixty-year cycle under `calendars.samvatsara`
    /// (`03-design/samvatsara-measured.md`).
    ///
    /// A year runs from the sunrise opening Chaitra Shukla Pratipada to
    /// the next year's, so a day's year is the one holding its sunrise;
    /// the first year is the one holding the range's first day, and the
    /// list runs until a year holds its last. The openings are found in
    /// this almanac's sky, the one that named the days' months.
    ///
    /// ```
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // Kathmandu on the Nepal clock, over the analytic test provider: fixed
    /// # // elements that need no engine. A real provider reads the real sky.
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
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    ///
    /// // November 2026 lies in one lunar year, Vikrama 2083.
    /// let years = almanac.years(&date(11, 1), &date(11, 30), &kathmandu)?.value;
    /// assert_eq!(years.len(), 1);
    /// assert_eq!((years[0].vikrama, years[0].shaka), (2083, 1948));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`Almanac::between`], and a place whose sunrise the polar
    /// policy leaves undefined on a pratipada.
    pub fn years(
        &self,
        from: &CalendarDate,
        to: &CalendarDate,
        place: &Place,
    ) -> Result<Envelope<Vec<LunarYear>>, Error> {
        self.lunar_years(from, to, place, true)
    }

    /// [`Almanac::years`], with each year's office-bearers when `rulers`
    /// asks for them: a sankalpa reads the year's name and numbers only.
    fn lunar_years(
        &self,
        from: &CalendarDate,
        to: &CalendarDate,
        place: &Place,
        rulers: bool,
    ) -> Result<Envelope<Vec<LunarYear>>, Error> {
        let (first, last) = (self.calendar.fixed_of(from)?, self.calendar.fixed_of(to)?);
        days_in(first, last, from, to, MOST_DAYS)?;
        let start = local_midnight(self.clock, first)?;
        let end = local_midnight(self.clock, last.plus_days(1))?;
        let completion = Completion::new(
            self.provider,
            self.settings().provider.overrides,
            self.delta_t,
        );
        let count = self.settings().calendars.samvatsara;
        // The opening at or before the range's first instant, then the
        // one before it too when the range starts ahead of that year's
        // first sunrise.
        let mut opened = self.opening(&completion, place, start)?;
        let mut began = self.pratipada(place, opened)?;
        if began.get() > start.get() {
            opened = self.opening(&completion, place, next_by(opened, -1.0))?;
            began = self.pratipada(place, opened)?;
        }
        // Each year's opening and first sunrise, and the next year's
        // after the last: a year's name can wait on the next one's.
        let mut run = vec![(opened, began)];
        while let Some(&(last, first_sunrise)) = run.last() {
            if first_sunrise.get() >= end.get() && run.len() > 1 {
                break;
            }
            // A lunar year is at most 385 days, so the opening before
            // 390 days on is the next one.
            let next = self.opening(&completion, place, next_by(last, 390.0))?;
            run.push((next, self.pratipada(place, next)?));
        }
        let text = &teistro_calendar::samvatsara::Parameters::TEXT;
        let mut running = run
            .iter()
            .map(|&(_, sunrise)| JovianYear::at(text, sunrise).map(|year| year.count))
            .collect::<Result<Vec<i64>, Error>>()?;
        // The years before the first, as many as the naming reads.
        let mut earliest = opened;
        let mut before = 0;
        while (before..running.len()).any(|index| name_counts(&running, index, count).is_none()) {
            if before == MOST_YEARS_BACK {
                return Err(Error::new(
                    Status::NotConverged,
                    format!(
                        "the samvatsara at {began} was not settled by {MOST_YEARS_BACK} years before it"
                    ),
                ));
            }
            earliest = self.opening(&completion, place, next_by(earliest, -1.0))?;
            let sunrise = self.pratipada(place, earliest)?;
            running.insert(0, JovianYear::at(text, sunrise)?.count);
            before += 1;
        }
        let named = |index: usize| {
            name_counts(&running, before + index, count).ok_or_else(|| {
                Error::internal("a lunar year's name was checked settled and is not")
            })
        };
        let mut years = Vec::with_capacity(run.len().saturating_sub(1));
        for (index, pair) in run.windows(2).enumerate() {
            let &[(opened, began), (_, ended)] = pair else {
                continue;
            };
            let mut year = LunarYear::of(
                count,
                opened,
                began,
                ended,
                (named(index)?, named(index + 1)?),
            )?;
            if rulers {
                year.rulers = self.rulers(&completion, place, opened)?;
            }
            years.push(year);
        }
        let frame = self.at_instant(&completion, opened)?.0;
        let provenance = self.provenance(
            content_hash(&RangeInput {
                from: from.to_string(),
                to: to.to_string(),
                latitude_deg: place.latitude.get(),
                longitude_deg: place.longitude.get(),
                altitude_m: place.altitude.get(),
            }),
            frame,
        );
        Ok(Envelope::new(years, provenance))
    }

    /// The new moon that opened the first Chaitra at or before an
    /// instant, in this almanac's sky.
    fn opening(
        &self,
        completion: &Completion<'_, P>,
        place: &Place,
        at: JulianDay<Utc>,
    ) -> Result<JulianDay<Utc>, Error> {
        let (frame, zodiac) = self.at_instant(completion, at)?;
        let mut longitudes = completion.longitudes(frame);
        if frame.centre == teistro_port_ephemeris::Centre::Topocentric {
            longitudes = longitudes.with_observer(*place);
        }
        crate::year::chaitra_opening(&longitudes, zodiac, at)
    }

    /// The office-bearers of the year a new moon opened
    /// (`03-design/year-rulers.md`): each office's event found in this
    /// almanac's sky, the one that found the opening, and its weekday read
    /// at the place under every rule the office's texts give.
    fn rulers(
        &self,
        completion: &Completion<'_, P>,
        place: &Place,
        opened: JulianDay<Utc>,
    ) -> Result<Vec<YearRuler>, Error> {
        let (frame, zodiac) = self.at_instant(completion, opened)?;
        let mut longitudes = completion.longitudes(frame);
        if frame.centre == teistro_port_ephemeris::Centre::Topocentric {
            longitudes = longitudes.with_observer(*place);
        }
        let year = rulers::events(&longitudes, zodiac, opened)?;
        year.offices
            .into_iter()
            .map(|(office, event)| {
                let holding = teistro_chart::day::chart_day(
                    self.model,
                    self.calendar,
                    self.clock,
                    place,
                    event,
                    self.settings().day.polar_day_policy,
                )?
                .day;
                Ok(YearRuler {
                    office,
                    event,
                    texts: office.texts().to_vec(),
                    readings: office
                        .rules()
                        .iter()
                        .map(|&rule| rulers::read(rule, event, &holding, Some(year.pratipada_ends)))
                        .collect(),
                })
            })
            .collect()
    }

    /// The sunrise that opens the day after a new moon's: the first
    /// sunrise after it, whichever civil date it falls on.
    fn pratipada(&self, place: &Place, new_moon: JulianDay<Utc>) -> Result<JulianDay<Utc>, Error> {
        let day = teistro_chart::day::chart_day(
            self.model,
            self.calendar,
            self.clock,
            place,
            new_moon,
            self.settings().day.polar_day_policy,
        )?;
        Ok(day.day.next_sunrise)
    }

    /// One day's value, unstamped, **and the frame it asked the provider
    /// for** — which the stamp needs and which this is the only place
    /// that computes.
    ///
    /// Handed back rather than recomputed. Asking `self.frame` a second
    /// time for the stamp cost `panchanga` **8.95%** of its instructions
    /// and the instruction-count gate refused it at a 3% budget: a frame
    /// is an ayanamsha at an instant, so it is a precession evaluation
    /// and not a field read.
    /// What festival rules read of one day, unstamped, and its frame: the
    /// first steps of [`Almanac::value`], taken the same way over the same
    /// memo, so each part is the value that day's panchanga holds.
    fn festival_value(
        &self,
        date: &CalendarDate,
        place: &Place,
        memo: &mut Memo,
    ) -> Result<(FestivalDay, Frame), Error> {
        let date = self.calendar.date_of(self.calendar.fixed_of(date)?)?;
        let fixed = self.calendar.fixed_of(&date)?;
        let (day, _) = self.local_day_of(&date, fixed, place, memo)?;
        let window = self.window(&day)?;
        // The daylight and the night a panchanga divides, refused alike.
        Interval::new(day.sunrise, day.sunset)?;
        Interval::new(day.sunset, day.next_sunrise)?;
        let completion = Completion::new(
            self.provider,
            self.settings().provider.overrides,
            self.delta_t,
        );
        let (frame, zodiac) = self.at_instant(&completion, window.from)?;
        let mut longitudes = completion.longitudes(frame);
        if frame.centre == teistro_port_ephemeris::Centre::Topocentric {
            longitudes = longitudes.with_observer(*place);
        }
        let known = memo.crossings(&longitudes, (zodiac, frame), window, fixed)?;
        let limbs = limb::limbs_with(&longitudes, window, zodiac, known.as_ref())?;
        let month = self.month(&longitudes, window, &limbs, (zodiac, frame), memo)?;
        memo.known = known.map(|known| (frame, known));
        Ok((
            FestivalDay::of(&day, month, limbs.tithi, limbs.nakshatra),
            frame,
        ))
    }

    /// What a day gives a range's change points: as
    /// [`Almanac::festival_value`], and the two signs from the same
    /// crossings.
    fn change_value(
        &self,
        date: &CalendarDate,
        place: &Place,
        memo: &mut Memo,
    ) -> Result<(ChangeDay, Frame), Error> {
        let date = self.calendar.date_of(self.calendar.fixed_of(date)?)?;
        let fixed = self.calendar.fixed_of(&date)?;
        let (day, _) = self.local_day_of(&date, fixed, place, memo)?;
        let window = self.window(&day)?;
        let completion = Completion::new(
            self.provider,
            self.settings().provider.overrides,
            self.delta_t,
        );
        let (frame, zodiac) = self.at_instant(&completion, window.from)?;
        let mut longitudes = completion.longitudes(frame);
        if frame.centre == teistro_port_ephemeris::Centre::Topocentric {
            longitudes = longitudes.with_observer(*place);
        }
        let known = memo.crossings(&longitudes, (zodiac, frame), window, fixed)?;
        let limbs = limb::limbs_with(&longitudes, window, zodiac, known.as_ref())?;
        // As a day's own: the Moon's signs geocentric, whatever the limbs'
        // centre, and the range's crossings only where they are that.
        let geocentric = known
            .as_ref()
            .filter(|_| frame.centre != teistro_port_ephemeris::Centre::Topocentric);
        let moon_signs = limb::signs_known(
            &completion.longitudes(frame),
            Body::Moon,
            window,
            zodiac,
            None,
            geocentric,
        )?;
        let sun_signs =
            limb::signs_known(&longitudes, Body::Sun, window, zodiac, None, known.as_ref())?;
        memo.known = known.map(|known| (frame, known));
        Ok((
            ChangeDay {
                window,
                sunrise: day.sunrise,
                vara: day.vara,
                limbs,
                moon_signs,
                sun_signs,
            },
            frame,
        ))
    }

    fn value(
        &self,
        date: &CalendarDate,
        place: &Place,
        memo: &mut Memo,
    ) -> Result<(Panchanga, Frame), Error> {
        let settings = self.settings();
        // The date is taken through the calendar and back, so that a day
        // asked for by a date a caller wrote and the same day reached
        // from an instant are equal values and not merely the same day:
        // a `CalendarDate` a caller builds carries no era numbers and one
        // the calendar renders does.
        let date = self.calendar.date_of(self.calendar.fixed_of(date)?)?;
        let fixed = self.calendar.fixed_of(&date)?;
        let (day, found_moon) = self.local_day_of(&date, fixed, place, memo)?;
        let window = self.window(&day)?;
        let daylight = Interval::new(day.sunrise, day.sunset)?;
        let night = Interval::new(day.sunset, day.next_sunrise)?;
        let completion = Completion::new(self.provider, settings.provider.overrides, self.delta_t);
        // One ayanamsha evaluation for the day; everything below reads
        // it rather than asking again at the same instant.
        let (frame, zodiac) = self.at_instant(&completion, window.from)?;
        let mut longitudes = completion.longitudes(frame);
        if frame.centre == teistro_port_ephemeris::Centre::Topocentric {
            longitudes = longitudes.with_observer(*place);
        }
        let known = memo.crossings(&longitudes, (zodiac, frame), window, fixed)?;
        let limbs = limb::limbs_with(&longitudes, window, zodiac, known.as_ref())?;

        let horas = hora::horas(&day, settings.day.hora_reckoning.try_into()?)?;
        let previous_night = match memo.day.take() {
            Some((before, yesterday)) if before.plus_days(1) == fixed => {
                Interval::new(yesterday.sunset, yesterday.next_sunrise).ok()
            }
            _ => self.previous_night(place, &date)?,
        };
        memo.day = Some((fixed, day.clone()));
        let muhurtas = period::muhurtas(daylight, night, previous_night, day.vara);
        let durmuhurtas = period::durmuhurtas(&muhurtas, day.vara, settings.panchanga.durmuhurta)?;
        let month = self.month(&longitudes, window, &limbs, (zodiac, frame), memo)?;
        // The Moon's own day reads geocentric longitudes, which are the
        // range's crossings' source unless the almanac is topocentric.
        let geocentric = known
            .as_ref()
            .filter(|_| frame.centre != teistro_port_ephemeris::Centre::Topocentric);
        let moon = self.moon(
            &completion,
            place,
            (&day, window),
            (frame, zodiac, geocentric),
            found_moon,
        )?;
        let sun_signs =
            limb::signs_known(&longitudes, Body::Sun, window, zodiac, None, known.as_ref())?;
        let ritu = self.ritu(
            (&date, place),
            &sun_signs,
            &month,
            (&longitudes, window, zodiac),
        )?;
        let punya_kala = punya_kala_of(&sun_signs, (daylight, night), previous_night, settings)?;
        let sun = sky::sun_day(sun_signs, ritu);
        let moon_sky = (&longitudes, zodiac, known.as_ref());
        let omens = self.omens(&limbs, day.vara, moon_sky)?;
        let held = (window, &mut memo.panchaka);
        let named_panchaka = self.named_panchaka((place, &day), &omens.panchaka, held, moon_sky)?;
        memo.known = known.map(|known| (frame, known));
        Ok((
            Panchanga {
                kaalas: period::kaalas(daylight, day.vara),
                choghadiya: period::choghadiya(daylight, night, day.vara),
                horas,
                muhurtas,
                durmuhurtas,
                month,
                moon,
                sun,
                omens,
                shiva_vasa: omen::shiva_vasas(&limbs.tithi, settings.panchanga.shiva_vasa),
                limbs,
                window,
                day,
                ghati_reckoning: settings.day.ghati_reckoning,
                lagnas: Vec::new(),
                punya_kala,
                homa_ahuti: Vec::new(),
                markers: Vec::new(),
                gowri: Vec::new(),
                named_panchaka,
                clock: Some(self.clock.reading(window.from))
                    .filter(|clock| clock.source != ZoneSource::Manual),
            },
            frame,
        ))
    }

    /// A day's local day, and its Moon's rises and sets when the range
    /// found them ahead ([`Almanac::ahead`]); a day found alone is
    /// carried from the day before.
    fn local_day_of(
        &self,
        date: &CalendarDate,
        fixed: FixedDay,
        place: &Place,
        memo: &mut Memo,
    ) -> Result<(LocalDay, Option<MoonAhead>), Error> {
        if let Some(ahead) = memo.ahead.take_day(fixed) {
            return Ok((ahead.day?, ahead.moon));
        }
        let day = local_day_carrying(
            self.model,
            self.calendar,
            self.clock,
            place,
            date,
            self.settings().day.polar_day_policy,
            &mut memo.light,
        )?;
        Ok((day, None))
    }

    /// The window the day's limbs are clipped to: the arc under a sunrise
    /// boundary, and the civil day under a midnight one.
    ///
    /// This is the first reader of `day.day_boundary`, declared in Phase
    /// 1 and unread until now. The *periods* are not affected by it: a
    /// choghadiya divides the daylight whatever the window is.
    fn window(&self, day: &LocalDay) -> Result<Interval, Error> {
        use teistro_core::settings::DayBoundary;
        match self.settings().day.day_boundary {
            DayBoundary::Sunset => Interval::new(day.sunset, day.sunset)
                .and_then(|_| Interval::new(day.sunset, next_by(day.sunset, 1.0))),
            DayBoundary::Noon | DayBoundary::Midnight => {
                let midnight = local_midnight(self.clock, self.calendar.fixed_of(&day.date)?)?;
                Interval::new(midnight, next_by(midnight, 1.0))
            }
            _ => Interval::new(day.sunrise, day.next_sunrise),
        }
    }

    /// The night that ends at this day's sunrise, which is the night
    /// Brahma muhurta belongs to.
    ///
    /// The recording engine sizes it from the night that *follows* the
    /// day instead — a median ten seconds out over the corpus, and entry
    /// 17 of the deliberate-difference registry.
    fn previous_night(
        &self,
        place: &Place,
        date: &CalendarDate,
    ) -> Result<Option<Interval>, Error> {
        let fixed = self.calendar.fixed_of(date)?.plus_days(-1);
        let Ok(yesterday) = self.calendar.date_of(fixed) else {
            return Ok(None);
        };
        let Ok(before) = local_day(
            self.model,
            self.calendar,
            self.clock,
            place,
            &yesterday,
            self.settings().day.polar_day_policy,
        ) else {
            return Ok(None);
        };
        Ok(Interval::new(before.sunset, before.next_sunrise).ok())
    }

    /// **One ayanamsha evaluation for a day**, and the two things read
    /// off it: the frame to ask the provider for, and the zodiac the
    /// limbs are measured from.
    ///
    /// The frame is tropical — the shift is applied by the limb kernel
    /// at each instant, so the elongation, the Moon's longitude and
    /// their sum are all measured from one ayanamsha — except where the
    /// provider **defines** the zodiac, when it is the provider's own
    /// sidereal frame and nothing is shifted
    /// ([`ChartZodiac::searched`]). Its centre is `panchanga.centre`,
    /// geocentric by default however the chart is computed, because an
    /// almanac is geocentric everywhere.
    ///
    /// The two used to be separate calls, and a day asked for one or the
    /// other **six times** at the same instant: the frame for the
    /// provider, the zodiac for the limbs, the zodiac again for the
    /// month, and the zodiac once more for each of the Sun's and the
    /// Moon's signs — plus a frame inside the Moon's own day. Every one
    /// of them ran `ChartZodiac::of`, which is a precession evaluation
    /// rather than a field read, and every one of them returned the same
    /// answer because the instant is the same.
    ///
    /// One evaluation, passed down. The instruction-count gate is what
    /// made the waste visible, by refusing an unrelated 8.95% and
    /// sending a reader through this path.
    fn at_instant(
        &self,
        completion: &Completion<'_, P>,
        at: JulianDay<Utc>,
    ) -> Result<(Frame, Zodiac), Error> {
        let searched = ChartZodiac::searched(
            completion,
            self.settings(),
            tt(at, self.delta_t)?,
            (self.precession, self.delta_t),
        )?;
        let frame = Frame {
            centre: match self.settings().panchanga.centre {
                Centre::Topocentric => teistro_port_ephemeris::Centre::Topocentric,
                _ => teistro_port_ephemeris::Centre::Geocentric,
            },
            ..searched.frame
        };
        Ok((frame, searched.zodiac))
    }

    /// The lunar month: the amanta month the new moon's solar sign names,
    /// and the purnimanta one that follows from the tithi the day opens
    /// in.
    ///
    /// Whether the month is intercalary the Indian lunisolar calendar
    /// decides, from the count of sankrantis between the month's two new
    /// moons (`03-design/calendar-indian-lunisolar.md` §2). The **name**
    /// is right either way and needs no case for the intercalary month:
    /// the Sun stands in the same sign at an adhika month's new moon as
    /// at the following nija month's, so `amanta_month` names both.
    ///
    /// The rule runs over **this** almanac's sky rather than the text's,
    /// because the month it marks is the one this almanac named.
    fn month<S: teistro_astro::events::Longitudes + ?Sized>(
        &self,
        longitudes: &S,
        window: Interval,
        limbs: &Limbs,
        (zodiac, frame): (Zodiac, Frame),
        memo: &mut Memo,
    ) -> Result<LunarMonth, Error> {
        let at_sunrise = limbs
            .tithi
            .first()
            .map(|span| span.member)
            .ok_or_else(|| Error::internal("a day with no tithi"))?;
        let from = window.from.get();
        let lunation = match memo.lunation {
            // A later day of the same lunation: the last new moon before
            // its window is the one already found, which the anchored
            // scan refines to the same instant from any window holding
            // it, and the next is found from it alone. So the span, its
            // name and its kind are the ones a search would return.
            Some(known)
                if known.frame == frame
                    && known.span.from.get() < from
                    && from < known.span.to.get() =>
            {
                known
            }
            _ => {
                let span = limb::lunar_month_span(longitudes, window, zodiac)?;
                let found = Lunation {
                    span,
                    frame,
                    amanta: limb::masa_at(longitudes, span.from, zodiac)?,
                    kind: teistro_calendar::lunisolar::kind_of(self.model, span.from, span.to)?,
                };
                memo.lunation = Some(found);
                found
            }
        };
        let (amanta, kind) = (lunation.amanta, lunation.kind);
        Ok(month::of(
            amanta,
            at_sunrise,
            self.settings().calendars.lunar_month,
            kind,
        ))
    }

    /// The Moon's day: its rises and sets in the window the profile
    /// names, and the signs it stood in.
    ///
    /// The frame is **passed in**, not computed: its caller has already
    /// asked for the same one, and a frame is an ayanamsha at an instant
    /// rather than a field read. Recomputing it here was a third
    /// precession evaluation per day that nothing needed.
    fn moon(
        &self,
        completion: &Completion<'_, P>,
        place: &Place,
        (day, window): (&LocalDay, Interval),
        (frame, zodiac, known): (Frame, Zodiac, Option<&RangeCrossings>),
        ahead: Option<MoonAhead>,
    ) -> Result<MoonDay, Error> {
        let search = self.moon_search(day, window)?;
        let (rises, sets) = match ahead {
            Some(found) if found.search == search => (found.rises?, found.sets?),
            _ => {
                let solver = self.moon_solver(completion, place);
                (
                    events(&solver, HorizonEventKind::Rise, search)?,
                    events(&solver, HorizonEventKind::Set, search)?,
                )
            }
        };
        let (rises, sets) = match self.settings().panchanga.moon_events {
            MoonEvents::CivilDay => (first_only(rises), first_only(sets)),
            _ => (rises, sets),
        };
        let longitudes = completion.longitudes(frame);
        Ok(MoonDay {
            rises,
            sets,
            signs: limb::signs_known(&longitudes, Body::Moon, window, zodiac, None, known)?,
            window: search,
        })
    }

    /// The window a day's moonrises and moonsets are searched in.
    fn moon_search(&self, day: &LocalDay, window: Interval) -> Result<Interval, Error> {
        Ok(match self.settings().panchanga.moon_events {
            // The engine's reading: the first rise and set at or after
            // the local civil midnight (entry 18), which is the next
            // civil day's when the Moon, a lunar day of some 24.8 hours
            // behind, does not rise or set in this one: four of the
            // corpus's 108 do so. Two days hold the first of each
            // wherever the Moon rises and sets at all, and
            // [`first_only`] keeps that first.
            MoonEvents::CivilDay => {
                let midnight = local_midnight(self.clock, self.calendar.fixed_of(&day.date)?)?;
                Interval::new(midnight, next_by(midnight, 2.0))?
            }
            _ => window,
        })
    }

    /// The solver a day's moonrises and moonsets are found by.
    fn moon_solver<'c>(&self, completion: &'c Completion<'_, P>, place: &Place) -> Solver<'c> {
        Solver::new(
            completion,
            Body::Moon,
            *place,
            Horizon::from_convention(self.settings().day.sunrise),
            self.delta_t,
        )
    }

    /// What a range's days can find before any of them is computed, found
    /// together: every day's local day, carried from the one before, and
    /// every day's moonrises and moonsets, searched in lockstep so that a
    /// round of every day's searches is one request
    /// ([`teistro_astro::lockstep`]). Each search takes the steps it
    /// takes for its day alone, so each day is what it is alone, to the
    /// bit; a day's failure is kept for that day to report, as it would
    /// have when the range reached it.
    fn ahead(&self, first: FixedDay, count: usize, place: &Place, moon: Moon, memo: &mut Memo) {
        let settings = self.settings();
        // Every day's light and the one after the last, found together:
        // a day reads its own and the next one's.
        let civil: Vec<FixedDay> = (0..=count)
            .map(|step| first.plus_days(i64::try_from(step).unwrap_or(0)))
            .collect();
        let lights = civil_day_lights(self.model, self.clock, place, &civil);
        let mut days = Vec::with_capacity(count);
        for (step, pair) in lights.windows(2).enumerate() {
            let fixed = first.plus_days(i64::try_from(step).unwrap_or(0));
            let day = match pair {
                [Ok(today), Ok(tomorrow)] => self.calendar.date_of(fixed).and_then(|date| {
                    local_day_lit(
                        self.model,
                        self.clock,
                        place,
                        (&date, fixed),
                        settings.day.polar_day_policy,
                        (*today, *tomorrow),
                    )
                }),
                [Err(error), _] | [_, Err(error)] => Err(error.clone()),
                _ => Err(Error::internal("a pair of day lights came apart")),
            };
            days.push((fixed, day));
        }
        let searches: Vec<Option<Interval>> = days
            .iter()
            .map(|(_, day)| {
                let day = day.as_ref().ok().filter(|_| moon == Moon::Searched)?;
                self.window(day)
                    .and_then(|window| self.moon_search(day, window))
                    .ok()
            })
            .collect();
        let completion = Completion::new(self.provider, settings.provider.overrides, self.delta_t);
        let solver = self.moon_solver(&completion, place);
        let round = Lockstep::new();
        let mut found = round
            .run(searches.iter().flatten().flat_map(|search| {
                [HorizonEventKind::Rise, HorizonEventKind::Set]
                    .map(|kind| events_with(&solver, Some(&round), kind, *search))
            }))
            .into_iter();
        memo.ahead = Ahead {
            first,
            days: days
                .into_iter()
                .zip(searches)
                .map(|((_, day), search)| AheadDay {
                    day,
                    moon: search.map(|search| MoonAhead {
                        search,
                        rises: found.next().unwrap_or_else(|| Ok(Vec::new())),
                        sets: found.next().unwrap_or_else(|| Ok(Vec::new())),
                    }),
                })
                .map(Some)
                .collect(),
        };
    }

    /// The day's omens: the panchaka from its nakshatras and the Moon's
    /// signs, the muhurta yogas, varjyam, amrit kalam and the disha shool.
    fn omens<S: teistro_astro::events::Longitudes + ?Sized>(
        &self,
        limbs: &limb::Limbs,
        vara: Vara,
        (longitudes, zodiac, known): (&S, Zodiac, Option<&RangeCrossings>),
    ) -> Result<Omens, Error> {
        let settings = self.settings();
        let moon_signs =
            |within| limb::signs_known(longitudes, Body::Moon, within, zodiac, None, known);
        Ok(Omens {
            panchaka: omen::panchaka(
                &limbs.nakshatra,
                settings.panchanga.panchaka_start,
                moon_signs,
            )?,
            yogas: omen::yogas(
                &settings.panchanga.muhurta_tables,
                vara,
                &limbs.nakshatra,
                &limbs.tithi,
            )?,
            varjyam: omen::varjyam(&limbs.nakshatra, settings.panchanga.varjyam)?,
            amrit_kalam: omen::amrit_kalam(
                &limbs.nakshatra,
                settings.panchanga.amrit_kalam,
                settings.panchanga.varjyam,
            )?,
            disha_shool: omen::disha_shool(vara),
        })
    }

    /// The panchaka a day holds, named by the vara it began on
    /// ([`omen::panchaka_by_vara`], crux C821), under
    /// `panchanga.panchaka_name`.
    ///
    /// The day's own panchaka spans are extended, not recomputed: their
    /// first gives the panchaka's first instant when it is Mrityu's, the
    /// one a panchaka opens with, and their last its end when it is
    /// Roga's. Otherwise the Moon's nakshatras before (or after) the day
    /// are read through [`omen::panchaka`] itself, as the day's were, over
    /// just the stars left to it, [`PANCHAKA_STAR_DAYS`] each, from a
    /// range's crossings when they reach that far. The vara is the one of
    /// the day, sunrise to sunrise, the first instant falls in.
    fn named_panchaka<S: teistro_astro::events::Longitudes + ?Sized>(
        &self,
        (place, day): (&Place, &LocalDay),
        spans: &[Span<Panchaka>],
        (window, held): (Interval, &mut Option<(Interval, Option<Panchaka>)>),
        (longitudes, zodiac, known): (&S, Zodiac, Option<&RangeCrossings>),
    ) -> Result<Option<Span<Panchaka>>, Error> {
        let settings = self.settings();
        let (Some(first), Some(last)) = (spans.first(), spans.last()) else {
            return Ok(None);
        };
        if settings.panchanga.panchaka_name == PanchakaName::None {
            return Ok(None);
        }
        // A range's later day of the same panchaka reads the one found.
        if let Some((whole, name)) = *held {
            if whole.contains(first.whole.from) {
                return Ok(name.and_then(|name| Span::new(name, whole, window)));
            }
        }
        let around = |within: Interval| {
            let stars = limb::nakshatras_known(longitudes, Body::Moon, within, zodiac, known)?;
            omen::panchaka(&stars, settings.panchanga.panchaka_start, |inside| {
                limb::signs_known(longitudes, Body::Moon, inside, zodiac, None, known)
            })
        };
        let from = if first.member == Panchaka::Mrityu {
            first.whole.from
        } else {
            let before = Interval::new(
                next_by(first.whole.from, -stars_days(first.member as u8)),
                first.whole.from,
            )?;
            around(before)?
                .iter()
                .rev()
                .find(|span| span.member == Panchaka::Mrityu)
                .map(|span| span.whole.from)
                .ok_or_else(|| Error::internal("a panchaka with no first star before it"))?
        };
        let to = if last.member == Panchaka::Roga {
            last.whole.to
        } else {
            let left = Panchaka::Roga as u8 - last.member as u8;
            let after = Interval::new(last.whole.to, next_by(last.whole.to, stars_days(left)))?;
            around(after)?
                .iter()
                .find(|span| span.member == Panchaka::Roga)
                .map(|span| span.whole.to)
                .ok_or_else(|| Error::internal("a panchaka with no last star after it"))?
        };
        // The vara of the day that holds the first instant: the day being
        // answered's when it opens there, and otherwise its civil date's or
        // the date before's when it falls before that sunrise. A polar day
        // whose policy puts its bounds elsewhere holds it in neither, and
        // keeps its civil date's.
        let vara = if day.contains(from) {
            day.vara
        } else {
            self.vara_holding(place, from)?
        };
        let whole = Interval::new(from, to)?;
        let name = omen::panchaka_by_vara(vara);
        *held = Some((whole, name));
        Ok(name.and_then(|name| Span::new(name, whole, window)))
    }

    /// The vara of the day, sunrise to sunrise, an instant falls in.
    fn vara_holding(&self, place: &Place, from: JulianDay<Utc>) -> Result<Vara, Error> {
        let policy = self.settings().day.polar_day_policy;
        let civil = teistro_calendar::solar::rule::local_day(self.clock, from).0;
        let today = local_day(
            self.model,
            self.calendar,
            self.clock,
            place,
            &self.calendar.date_of(civil)?,
            policy,
        )?;
        Ok(if from.get() < today.sunrise.get() {
            let before = local_day(
                self.model,
                self.calendar,
                self.clock,
                place,
                &self.calendar.date_of(civil.plus_days(-1))?,
                policy,
            )?;
            if before.contains(from) {
                before.vara
            } else {
                today.vara
            }
        } else {
            today.vara
        })
    }

    /// The day's season under `panchanga.ritu`.
    ///
    /// The solar season is the solar month's, and a month begins on the
    /// civil day `panchanga.solar_month_start` places its sankranti on:
    /// the Sun's sign at sunrise names the next month for a day whose
    /// sankranti falls after dawn, which Nepal's daily panchanga refutes
    /// (`03-design/ritu-measured.md`). The lunar season is the amanta
    /// month's, an adhika month taking its name's; the tropical has no
    /// civil rule to follow and is read as the day opens, the one reading
    /// that searches again, and only when asked.
    fn ritu<S: teistro_astro::events::Longitudes + ?Sized>(
        &self,
        (date, place): (&CalendarDate, &Place),
        sun: &[Span<Rashi>],
        month: &LunarMonth,
        (longitudes, window, zodiac): (&S, Interval, Zodiac),
    ) -> Result<Ritu, Error> {
        let settings = self.settings();
        let opening = |signs: &[Span<Rashi>]| {
            signs
                .first()
                .map_or(Ritu::Vasanta, |span| sky::ritu_of(span.member))
        };
        Ok(match settings.panchanga.ritu {
            RituReckoning::Solar => {
                let fixed = self.calendar.fixed_of(date)?;
                // Under `NEAREST_EVENT` a polar day's window stands at the
                // nearest sunrise, weeks off, and its signs are not the
                // date's (C930). Every other window holds the civil noon.
                let midnight = teistro_time::local_day::local_midnight(self.clock, fixed)?;
                let civil = if window.contains(midnight.plus_days(0.5)?) {
                    None
                } else {
                    let day = Interval::new(midnight, midnight.plus_days(1.0)?)?;
                    Some(Self::signs(longitudes, Body::Sun, day, zodiac)?)
                };
                sky::solar_month(
                    settings.panchanga.solar_month_start,
                    fixed,
                    civil.as_deref().unwrap_or(sun),
                    (place, self.clock, self.model, settings.day.polar_day_policy),
                )?
                .map_or_else(|| opening(sun), sky::ritu_of)
            }
            RituReckoning::Lunar => month.amanta.attributes().ritu,
            RituReckoning::Tropical => {
                let tropical = Zodiac {
                    ayanamsha: None,
                    ..zodiac
                };
                opening(&Self::signs(longitudes, Body::Sun, window, tropical)?)
            }
            other => {
                return Err(Error::unsupported(format!(
                    "the panchanga does not know the panchanga.ritu value {other:?} yet"
                ))
                .with_field("panchanga.ritu"));
            }
        })
    }

    /// The signs a body stood in over the window.
    fn signs<S: teistro_astro::events::Longitudes + ?Sized>(
        longitudes: &S,
        body: Body,
        window: Interval,
        zodiac: Zodiac,
    ) -> Result<Vec<Span<Rashi>>, Error> {
        limb::signs(longitudes, body, window, zodiac)
    }

    /// The stamp every value carries.
    ///
    /// **The provider is named**, as the chart foundation names it. It
    /// was not until a Rust example printed `provider` off an almanac's
    /// envelope and got an empty string: a stored page said nothing
    /// about what computed it, in all four bindings, because nothing
    /// printed the field.
    ///
    /// `flags_used` is **empty and not a guess**. The chart passes the
    /// completion's steps there, and this path does not have them: its
    /// positions come through `FrameLongitudes`, which keeps no step
    /// list, so there is nothing here to vouch for. An empty list is the
    /// refusal; a plausible one would be a fabrication.
    fn provenance(&self, input: Hash, frame: Frame) -> Provenance {
        let mut provenance = self.resolved.provenance(
            Version::parse(env!("CARGO_PKG_VERSION")).unwrap_or(Version::new(0, 0, 0)),
            input,
        );
        provenance.provider = self
            .provider
            .capabilities()
            .identity
            .stamp(frame, Vec::new());
        provenance.time.delta_t_model = self.delta_t.key().to_string();
        provenance.time.leap_table = teistro_time::leap::version().to_string();
        provenance
    }

    /// The provenance of an answer whose days open on `first`: its clock
    /// stated at that day's noon, away from any midnight change
    /// (`docs/03-design/time-and-timezone.md` §3.4).
    fn provenance_from(&self, input: Hash, frame: Frame, first: FixedDay) -> Provenance {
        let noon = first
            .jd_at_midnight()
            .ok()
            .and_then(|midnight| JulianDay::try_new(midnight.get() + 0.5).ok());
        self.provenance_at(input, frame, noon)
    }

    /// The provenance of an answer, its clock stated at an instant.
    fn provenance_at(
        &self,
        input: Hash,
        frame: Frame,
        instant: Option<JulianDay<Utc>>,
    ) -> Provenance {
        let mut provenance = self.provenance(input, frame);
        if let Some(instant) = instant {
            let clock = self.clock.reading(instant);
            if clock.source == ZoneSource::Iana {
                provenance.time.tzdb_version.clone_from(&clock.tzdb_version);
            }
            provenance.time.clock = Some(clock);
        }
        provenance
    }
}

/// Whether a range's days read the Moon's rises and sets, and so whether
/// [`Almanac::ahead`] searches them.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Moon {
    Searched,
    Unread,
}

/// What one day of a range leaves for the next, so that the next reads it
/// rather than searching for it again. Every field is a value the next
/// day would compute from the same arguments, so a range's days are the
/// days asked for one at a time, to the bit.
#[derive(Default)]
struct Memo {
    /// The last day computed, by its fixed day: the next day's previous
    /// night is its sunset to its next sunrise.
    day: Option<(FixedDay, LocalDay)>,
    /// The lunation the last day fell in.
    lunation: Option<Lunation>,
    /// The next day's light, which the last day read as its tomorrow.
    light: Carried,
    /// The last day of the range, when the days are a range: how far the
    /// crossings found for the first day are worth finding.
    last: Option<FixedDay>,
    /// What the range found for its days before computing them.
    ahead: Ahead,
    /// The limbs' and the signs' crossings found once for the range
    /// ([`RangeCrossings`]), by the frame their source was read in, read
    /// by every day whose window they cover.
    known: Option<(Frame, RangeCrossings)>,
    /// The last panchaka named, its whole and its name: the next day of
    /// the same panchaka reads it rather than searching its ends again.
    panchaka: Option<(Interval, Option<Panchaka>)>,
}

/// A range's days as found ahead of computing them ([`Almanac::ahead`]).
struct Ahead {
    first: FixedDay,
    days: Vec<Option<AheadDay>>,
}

impl Default for Ahead {
    fn default() -> Self {
        Ahead {
            first: FixedDay::new(0),
            days: Vec::new(),
        }
    }
}

impl Ahead {
    /// What was found ahead for a day, once.
    fn take_day(&mut self, day: FixedDay) -> Option<AheadDay> {
        let at = usize::try_from(self.first.days_until(day)).ok()?;
        self.days.get_mut(at)?.take()
    }
}

/// One day found ahead: its local day, and its Moon's rises and sets when
/// its window could be had.
struct AheadDay {
    day: Result<LocalDay, Error>,
    moon: Option<MoonAhead>,
}

/// A day's moonrises and moonsets found ahead, in the window searched.
struct MoonAhead {
    search: Interval,
    rises: Result<Vec<JulianDay<Utc>>, Error>,
    sets: Result<Vec<JulianDay<Utc>>, Error>,
}

impl Memo {
    /// The range's crossings for a day, read in the day's frame, found on
    /// the first day that needs them: from this day's window to the range's last day and a day
    /// past it, which a last day's window ends inside. A day outside them,
    /// or under another zodiac, is searched for alone, so a crossing set
    /// that does not cover a day never changes it; a single day asked for
    /// alone has no range and searches as it always did.
    fn crossings<S: teistro_astro::events::Longitudes + ?Sized>(
        &mut self,
        longitudes: &S,
        (zodiac, frame): (Zodiac, Frame),
        window: Interval,
        day: FixedDay,
    ) -> Result<Option<RangeCrossings>, Error> {
        if let Some((read_in, known)) = self.known.take()
            && read_in == frame
            && known.answers(zodiac, window)
        {
            return Ok(Some(known));
        }
        let Some(last) = self.last else {
            return Ok(None);
        };
        let left = day.days_until(last);
        if left < 1 {
            return Ok(None);
        }
        // A whole number of days, a few thousand at most.
        #[allow(
            clippy::cast_precision_loss,
            reason = "a range's length in days is a small whole number"
        )]
        let to = window.from.get() + left as f64 + 2.0;
        let run = Interval::new(window.from, JulianDay::<Utc>::literal(to))?;
        RangeCrossings::over(longitudes, zodiac, run).map(Some)
    }
}

/// A lunar month as the almanac names it: its span between two new
/// moons, the frame they were found in, its amanta name and its kind.
#[derive(Clone, Copy)]
struct Lunation {
    span: Interval,
    frame: Frame,
    amanta: teistro_core::catalogue::Masa,
    kind: MonthKind,
}

/// The most lunar years before a range the samvatsara's naming may read:
/// measured, it reads three at most.
const MOST_YEARS_BACK: usize = 12;

/// Every event of a kind inside a window, in order.
///
/// A moonrise and the next are about 24 h 50 m apart, so a 24-hour window
/// holds none, one or two of each; the loop is what says so.
fn events(
    solver: &Solver<'_>,
    kind: HorizonEventKind,
    window: Interval,
) -> Result<Vec<JulianDay<Utc>>, Error> {
    teistro_astro::lockstep::finished(events_with(solver, None, kind, window)).unwrap_or_else(
        || {
            Err(Error::internal(
                "an event search on the spot waited for a round",
            ))
        },
    )
}

/// [`events`], its searches on the spot or in `round`'s requests.
async fn events_with<'a>(
    solver: &Solver<'a>,
    round: Option<&Lockstep<'a>>,
    kind: HorizonEventKind,
    window: Interval,
) -> Result<Vec<JulianDay<Utc>>, Error> {
    let mut found = Vec::new();
    let mut from = window.from.get();
    while from < window.to.get() {
        let remaining = window.to.get() - from;
        let start = JulianDay::<Ut1>::literal(from);
        let event = match round {
            Some(round) => solver.event_in(round, kind, start, remaining).await?,
            None => solver.event(kind, start, remaining)?,
        };
        let Some(event) = event else {
            break;
        };
        let at = event.instant.get();
        if at >= window.to.get() {
            break;
        }
        found.push(JulianDay::literal(at));
        // Past the event by a minute, so the next search cannot find the
        // same crossing again.
        from = at + 1.0 / 1440.0;
    }
    Ok(found)
}

/// The first of a day's events, under a reading that names one.
fn first_only(mut events: Vec<JulianDay<Utc>>) -> Vec<JulianDay<Utc>> {
    events.truncate(1);
    events
}

/// How long the Moon takes over one star at its slowest, days, rounded
/// up: 13°20' at twelve degrees a day is a little over 1.1.
const PANCHAKA_STAR_DAYS: f64 = 1.2;

/// How far `stars` of a panchaka's stars reach, days.
fn stars_days(stars: u8) -> f64 {
    f64::from(stars) * PANCHAKA_STAR_DAYS
}

/// An instant so many days on.
fn next_by(instant: JulianDay<Utc>, days: f64) -> JulianDay<Utc> {
    JulianDay::literal(instant.get() + days)
}

/// The terrestrial time of an instant.
fn tt(
    instant: JulianDay<Utc>,
    delta_t: DeltaTModel,
) -> Result<JulianDay<teistro_core::quantity::Tt>, Error> {
    let (tt, _) = teistro_astro::scale::tt_of(JulianDay::<Ut1>::literal(instant.get()), delta_t)?;
    Ok(tt)
}

/// The punya kala the day's daylight holds, from each sankranti the Sun's
/// signs bound: each span's entry, and the next sign's at the last span's
/// end, which may lie past a window that closes at midnight.
///
/// A day with no night before it (the calendar's first) places only the
/// entries of its own daylight and night.
fn punya_kala_of(
    signs: &[Span<Rashi>],
    (daylight, night): (Interval, Interval),
    previous_night: Option<Interval>,
    settings: &Settings,
) -> Result<Option<PunyaKala>, Error> {
    let arcs = Arcs {
        before: previous_night.unwrap_or(Interval {
            from: daylight.from,
            ..daylight
        }),
        daylight,
        after: night,
    };
    let table = settings.panchanga.punya_kala;
    let reckoning = settings.day.ghati_reckoning.try_into()?;
    let entries = signs
        .iter()
        .map(|span| (span.member, span.whole.from))
        .chain(
            signs
                .last()
                .map(|span| (sky::next(span.member), span.whole.to)),
        );
    for (sign, sankranti) in entries {
        if let Some(kala) = punya::punya_kala(sign, sankranti, &arcs, table, reckoning)? {
            return Ok(Some(kala));
        }
    }
    Ok(None)
}
