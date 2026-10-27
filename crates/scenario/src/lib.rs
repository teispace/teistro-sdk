//! The fixed scenario the SDK measures itself with.
//!
//! Two gates walk the same code for two different reasons: the
//! cross-architecture matrix hashes every value it computes, to find out
//! whether another machine computes the same numbers, and the
//! instruction-count benchmarks count the instructions it takes, to find
//! out whether a change made the SDK slower. They must walk the *same*
//! code, or a benchmark measures a path the determinism gate never
//! checked and a determinism gate checks a path nobody benchmarks; so the
//! scenario lives here and both read it.
//!
//! Everything is deterministic and takes no input: the same fixed days,
//! the same instants, the same latitudes and meridians, in the same
//! order, and no knob to turn. The first nine sections are about forty
//! milliseconds of work in release; the four over a range on the
//! built-in ephemeris below are about a second and a half more (the
//! transit hits and Sade Sati, through the façade, about half a second of
//! it), and a chart's judgements through the façade fifty milliseconds,
//! which is still a nightly digest's rounding error and a minute or two
//! under callgrind, so both gates can afford all of it and neither has to
//! measure a subset of what the other checked.
//!
//! The **panchanga** section is the newest and was added for a gap
//! rather than for coverage's sake. The digest had sections for the
//! calendar, the astronomy, the houses and the classical model and none
//! for the panchanga, so the instants a crossing search *produces* — the
//! limb boundaries, the sunrises, the Moon's rises — were computed by
//! nothing this gate watched. Three changes in a row moved where a scan
//! puts its samples, and the digest was identical to the bit each time,
//! which said nothing at all about the layer where they moved. It costs
//! ten milliseconds and it sees them: moving `SCAN_ANCHOR_JD` by half a
//! step leaves the `astro` digest unchanged and moves this one.
//!
//! The **dashas** section walks every kernel's cursor: the chain at depth
//! five at instants spread over three centuries, for each nakshatra-seeded
//! row, each sign-based row and the Kalachakra, from Moons spread round the
//! zodiac. A boundary is a sum of products of years and days, which is the
//! arithmetic two machines could round differently, and `at` is the call a
//! timeline makes per pixel, so the instruction count watches the budget
//! `09-performance-architecture.md` sets it (`crates/dasha/benches` times
//! it).
//!
//! The rows counted over the twenty-eight nakshatras with Abhijit are a
//! section of their own, **dashas-28**, and not more rows in `dashas`. The
//! benchmark gate compares a section with itself on the base commit, so
//! a row added to a section reads as that section's code getting slower:
//! adding the two to `dashas` read as 46% more instructions for a kernel
//! whose cost per dasha had not moved. A new section is reported as new.
//!
//! The last three sections stand on the built-in ephemeris rather than
//! the test provider, because what they watch is the cost of a range: the
//! built-in fits the Moon and the Earth in segments and caches them, and
//! a range is answered in rounds that each visit every day of it.
//! **festivals** is the Nepal pack, the days counted after its festivals
//! and the Ekadashis over eight weeks of autumn; **muhurta** a three-day
//! marriage search with every window judged; **panchanga-year** a year of
//! days in one call, where a cache sized for a month once made a year cost
//! twice what it had. Each is the work the app-speed rows of the same
//! names time by the clock, counted instead, so a pull request that makes
//! one slower fails here before anyone runs the clock.

#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use teistro_astro::Completion;
use teistro_astro::ayanamsha::{self, Basis};
use teistro_astro::delta_t::{DeltaTModel, delta_t};
use teistro_astro::houses::{self, Input};
use teistro_astro::iau;
use teistro_astro::precession::PrecessionModel;
use teistro_astro::visibility::{Criterion, Heliacal};
use teistro_calendar::fixed::FixedDay;
use teistro_calendar::shipped;
use teistro_calendar::solar::drik::DrikSun;
use teistro_calendar::{CalendarDate, CalendarSystem, Gregorian};
use teistro_chart::bhava::{Bhavas, Chalit};
use teistro_chart::foundation::Founder;
use teistro_core::angle::Nas;
use teistro_core::catalogue::{Ayanamsha, Calendar, Dignity, Graha, HouseSystem, Rashi};
use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
use teistro_core::quantity::{Degrees, Depth, JulianDay, Tt, Ut1, Utc};
use teistro_core::settings::{
    AfterCycle, AshtottariGrouping, AyanamshaChoice, Balance, BirthPeriod, KalachakraAfterNinth,
    KalachakraBalance, KalachakraMembership, PolarPolicy, SeedOverflow, YearLength,
};
use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
use teistro_core::time::UtcOffset;
use teistro_dasha::{
    ASHTOTTARI_BPHS, Birth, Chain, Dasha, KalachakraDasha, KalachakraRules, RASHI_ROWS, ROWS,
    RashiChart, RashiDasha, Rules as DashaRules, Timeline, UduRow, Wheel,
};
use teistro_ephemeris_builtin::provider::Builtin;
use teistro_geometry::{Body, Placements, Point, place, rows};
use teistro_muhurta::sources::Over;
use teistro_muhurta::{ActivityRules, ProviderSources, Ranking, Request as MuhurtaRequest};
use teistro_panchanga::Almanac;
use teistro_panchanga::festival::{
    EkadashiRule, FestivalRule, FollowingRule, Observances, ekadashis, following, observances,
};
use teistro_port_ephemeris::Horizon;
use teistro_port_ephemeris::TestProvider;
use teistro_render_svg::{Labels, Style, render as render_svg};
use teistro_siddhanta::SuryaSiddhanta;

/// One section of the scenario: its name and the values it computed. A
/// value is kept as its bits, so an integer is hashed as an integer and a
/// double as a double, and neither is squeezed through the other.
///
/// ```
/// use teistro_scenario::section;
///
/// let calendar = section("calendar").ok_or("a section")?;
/// assert_eq!(calendar.name, "calendar");
/// assert!(!calendar.values.is_empty());
/// # Ok::<(), &str>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    /// What the section is called in every report.
    pub name: &'static str,
    /// Every value it computed, in order, as its bits.
    pub values: Vec<u64>,
}

impl Section {
    fn new(name: &'static str) -> Section {
        Section {
            name,
            values: Vec::new(),
        }
    }

    fn push(&mut self, value: f64) {
        self.values.push(value.to_bits());
    }

    fn push_int(&mut self, value: i64) {
        self.values.push(u64::from_ne_bytes(value.to_ne_bytes()));
    }
}

/// Every value of `sections` as its bits, one line each: the section, the
/// value's index in it, and the bits as sixteen hex digits. This is the
/// file two machines compare (`cargo xtask compare-hashes`), so every
/// platform writes it here, natively through `cargo xtask hashes` and in
/// wasm32 through the scenario binary's `values` mode.
///
/// As a number, not as its bytes: a byte-reversed hex string reads back as
/// a value nowhere near the one written, and every distance taken over it
/// is meaningless.
///
/// ```
/// use teistro_scenario::{Section, write_values};
///
/// // One line a value: the section, the index, the bits as sixteen hex digits.
/// let one = Section { name: "example", values: vec![1.0_f64.to_bits(), 42] };
/// let mut out = Vec::new();
/// write_values(&[one], &mut out)?;
/// assert_eq!(String::from_utf8(out)?, "example\t0\t3ff0000000000000\nexample\t1\t000000000000002a\n");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Whatever writing to `out` refuses.
pub fn write_values(sections: &[Section], out: &mut impl std::io::Write) -> std::io::Result<()> {
    for section in sections {
        for (index, value) in section.values.iter().enumerate() {
            writeln!(out, "{}\t{index}\t{value:016x}", section.name)?;
        }
    }
    Ok(())
}

/// The sections in the order every report lists them.
///
/// ```
/// use teistro_scenario::{SECTIONS, section};
///
/// // Every name listed is a section.
/// assert!(SECTIONS.iter().all(|name| section(name).is_some()));
/// ```
pub const SECTIONS: [&str; 14] = [
    "calendar",
    "astro",
    "houses",
    "siddhanta",
    "panchanga",
    "geometry",
    "render",
    "dashas",
    "dashas-28",
    "festivals",
    "muhurta",
    "panchanga-year",
    "hits",
    "judgement",
];

/// One section by name, or `None` when nothing is called that.
///
/// ```
/// use teistro_scenario::section;
///
/// // The same section twice is the same bits: the scenario is deterministic.
/// assert_eq!(section("houses"), section("houses"));
/// assert_eq!(section("horoscope"), None);
/// ```
#[must_use]
pub fn section(name: &str) -> Option<Section> {
    match name {
        "calendar" => Some(calendars()),
        "astro" => Some(astronomy()),
        "houses" => Some(houses()),
        "siddhanta" => Some(siddhanta()),
        "panchanga" => Some(panchanga()),
        "geometry" => Some(geometry()),
        "render" => Some(render()),
        "dashas" => Some(dashas()),
        "dashas-28" => Some(dashas_28()),
        "festivals" => Some(festivals()),
        "muhurta" => Some(muhurta()),
        "panchanga-year" => Some(panchanga_year()),
        _ => ASKED.iter().find(|a| a.name == name).map(asked),
    }
}

/// Every section, in order.
///
/// ```
/// use teistro_scenario::{SECTIONS, all};
///
/// let names: Vec<&str> = all().iter().map(|section| section.name).collect();
/// assert_eq!(names, SECTIONS);
/// ```
#[must_use]
pub fn all() -> Vec<Section> {
    SECTIONS.iter().filter_map(|name| section(name)).collect()
}

/// The instants the scenario walks: every tenth day of four centuries,
/// which is enough trigonometry to catch a library that rounds
/// differently and few enough to run in a second.
fn instants() -> Vec<f64> {
    (0..1_460)
        .map(|i| 2_378_496.5 + f64::from(i) * 100.0)
        .collect()
}

/// The calendars: every shipped calendar over a span of fixed days, with
/// the weekday and the month's length.
fn calendars() -> Section {
    let mut section = Section::new("calendar");
    for fixed in (600_000_i64..760_000).step_by(997) {
        for calendar in [
            Calendar::Gregorian,
            Calendar::Julian,
            Calendar::BikramSambat,
            Calendar::IsoWeek,
        ] {
            let Some(system) = shipped(calendar) else {
                continue;
            };
            let Ok(date) = system.date_of(FixedDay::new(fixed)) else {
                continue;
            };
            section.push_int(i64::from(date.year));
            section.push_int(i64::from(date.month));
            section.push_int(i64::from(date.day));
            if let Ok(back) = system.fixed_of(&date) {
                section.push_int(back.get());
            }
        }
    }
    section
}

/// The astronomy: the obliquity, the sidereal times, the nutation, Delta
/// T and the ayanamshas, over the whole span.
fn astronomy() -> Section {
    let mut section = Section::new("astro");
    for jd in instants() {
        section.push(iau::obl06(jd, 0.0));
        section.push(iau::obl80(jd, 0.0));
        section.push(iau::era00(jd, 0.0));
        section.push(iau::gmst06(jd, 0.0, jd, 0.0));
        section.push(iau::gst06b(jd, 0.0, jd, 0.0));
        let nutation = iau::nut00b(jd, 0.0);
        section.push(nutation.dpsi);
        section.push(nutation.deps);
        if let Ok(at) = JulianDay::<Ut1>::try_new(jd) {
            if let Ok(delta) = delta_t(at, DeltaTModel::TableThenModel) {
                section.push(delta.seconds);
            }
        }
        let Ok(tt) = JulianDay::<Tt>::try_new(jd) else {
            continue;
        };
        for id in [Ayanamsha::Lahiri, Ayanamsha::FaganBradley, Ayanamsha::Raman] {
            let choice = AyanamshaChoice::Catalogued { id };
            if let Ok(value) = ayanamsha::value_deg(
                &choice,
                tt,
                Basis::True,
                PrecessionModel::Iau2006,
                DeltaTModel::TableThenModel,
            ) {
                section.push(value);
            }
        }
    }
    section
}

/// The houses: every system this build constructs, at a spread of
/// latitudes and meridians.
fn houses() -> Section {
    let mut section = Section::new("houses");
    let systems = [
        HouseSystem::Placidus,
        HouseSystem::Koch,
        HouseSystem::Regiomontanus,
        HouseSystem::Campanus,
        HouseSystem::Porphyry,
        HouseSystem::WholeSign,
        HouseSystem::Equal,
        HouseSystem::Alcabitius,
        HouseSystem::Topocentric,
        HouseSystem::Morinus,
    ];
    for latitude in [-66.0, -45.0, -27.7, 0.0, 27.7, 45.0, 51.5, 66.0] {
        for armc in (0..360).step_by(7) {
            let input = Input {
                armc_deg: f64::from(armc),
                latitude_deg: latitude,
                obliquity_deg: 23.439_291_1,
                sun_declination_deg: None,
                sidereal_offset_deg: 0.0,
            };
            for system in systems {
                let Ok(built) = houses::houses(system, &input, PolarPolicy::FallbackWholeSign)
                else {
                    continue;
                };
                for cusp in built.cusps {
                    section.push(cusp);
                }
                section.push(built.angles.ascendant_deg);
                section.push(built.angles.midheaven_deg);
            }
        }
    }
    section
}

/// The chart geometry: the two radial layouts over a sweep of ascendants
/// with real Placidus cusps. The Western wheel is the one place a drawing
/// takes a platform's `sin` and `cos`; its coordinates are rounded to a grain
/// so the last unit two libraries may differ in never reaches the output,
/// and this section is what checks that the rounding holds on every
/// architecture (`03-design/chart-geometry.md` §7).
fn geometry() -> Section {
    let mut section = Section::new("geometry");
    let mut push = |point: Point| {
        section.push(point.x);
        section.push(point.y);
    };
    let (wheel, chakra) = (rows::western_wheel(), rows::sudarshan_chakra());
    for chart in sweep(&[0.0, 97.0, 211.5, 333.3]) {
        for layout in [&wheel, &chakra] {
            let Ok(placed) = place(layout, &chart) else {
                continue;
            };
            for cell in &placed.cells {
                push(cell.outline.start);
                for segment in &cell.outline.segments {
                    push(segment.end());
                }
                push(cell.label);
                push(cell.anchor);
            }
            for mark in &placed.marks {
                push(mark.at);
            }
        }
    }
    section
}

/// Charts over a sweep of real Placidus ascendants at Kathmandu's latitude,
/// with the Sun, Moon, Mars and Saturn (and the rest of the grahas, where
/// more offsets are given) that far past the ascendant.
fn sweep(offsets: &'static [f64]) -> impl Iterator<Item = Placements> {
    const GRAHAS: [Graha; 9] = [
        Graha::Sun,
        Graha::Moon,
        Graha::Mars,
        Graha::Saturn,
        Graha::Mercury,
        Graha::Jupiter,
        Graha::Venus,
        Graha::Rahu,
        Graha::Ketu,
    ];
    (0..360).step_by(11).filter_map(move |armc| {
        let input = Input {
            armc_deg: f64::from(armc),
            latitude_deg: 27.7,
            obliquity_deg: 23.439_291_1,
            sun_declination_deg: None,
            sidereal_offset_deg: 0.0,
        };
        let built = houses::houses(
            HouseSystem::Placidus,
            &input,
            PolarPolicy::FallbackWholeSign,
        )
        .ok()?;
        let ascendant = built.angles.ascendant_deg;
        Some(Placements {
            lagna_deg: Some(ascendant),
            houses: Some(Bhavas::of(Chalit::of(HouseSystem::Placidus), &built.cusps)),
            bodies: GRAHAS
                .iter()
                .zip(offsets)
                .map(|(graha, offset)| {
                    Body::at(graha.key_id(), (ascendant + offset).rem_euclid(360.0))
                })
                .collect(),
            ..Placements::new(Body::at(Graha::Sun.key_id(), ascendant).sign)
        })
    })
}

/// The renderer: the wheel and the chakra drawn over the same sweep, with a
/// conjunction of five so the marks are spread, and every byte of each
/// drawing pushed eight at a time. The wheel's spreading takes a platform's
/// `atan2`, `sin` and `cos`, rounded to the geometry's grain, and this is
/// what shows that rounding holds (`03-design/render-svg.md` §5).
fn render() -> Section {
    let mut section = Section::new("render");
    let (wheel, chakra) = (rows::western_wheel(), rows::sudarshan_chakra());
    let names = ["Su", "Mo", "Ma", "Sa", "Me", "Ju", "Ve", "Ra", "Ke"];
    let style = Style::light();
    for chart in sweep(&[0.0, 2.5, 3.0, 97.0, 4.5, 211.5, 333.3, 180.0, 1.0]) {
        for layout in [&wheel, &chakra] {
            let Ok(placed) = place(layout, &chart) else {
                continue;
            };
            let labels = Labels {
                title: None,
                cells: placed
                    .cells
                    .iter()
                    .map(|cell| cell.house.to_string())
                    .collect(),
                bodies: chart
                    .bodies
                    .iter()
                    .zip(names)
                    .map(|(body, name)| (body.key, name.to_owned()))
                    .collect(),
                lagna: Some(String::from("As")),
            };
            let Ok(svg) = render_svg(&placed, &labels, &style) else {
                continue;
            };
            for chunk in svg.as_bytes().chunks(8) {
                let mut word = [0u8; 8];
                word.iter_mut()
                    .zip(chunk)
                    .for_each(|(to, from)| *to = *from);
                section.push_int(i64::from_le_bytes(word));
            }
        }
    }
    section
}

/// The classical model: the text's own longitudes for every graha.
fn siddhanta() -> Section {
    let mut section = Section::new("siddhanta");
    let text = SuryaSiddhanta::text();
    for jd in instants() {
        let Ok(at) = JulianDay::<Ut1>::try_new(jd) else {
            continue;
        };
        for graha in [
            Graha::Sun,
            Graha::Moon,
            Graha::Mars,
            Graha::Mercury,
            Graha::Jupiter,
            Graha::Venus,
            Graha::Saturn,
            Graha::Rahu,
        ] {
            let Ok(position) = text.graha(graha, at) else {
                continue;
            };
            section.push(position.longitude.get());
            section.push(position.speed_deg_per_day);
        }
    }
    section
}

/// The panchanga: the layer whose instants the searches produce, which
/// the matrix could not see.
///
/// The digest had sections for the calendar, the astronomy, the houses
/// and the classical model, and **none for the panchanga** — so a change
/// to how a crossing is searched for, which is most of what the
/// performance work touched, moved instants no cross-architecture gate
/// was watching. Three changes in a row moved them and the digest was
/// identical to the bit each time, which said nothing about the layer
/// where they moved.
///
/// What it walks is what a search produces: the day's own instants, and
/// every limb's boundaries as well as the member that ran. A boundary is
/// where two machines would first disagree, because it is the output of
/// a bracket and a narrowing rather than of one expression.
///
/// The analytic test provider, so the section takes no data and no
/// engine and is the same everywhere. Ten days rather than one: the
/// searches share instants between neighbouring days, and a change to
/// that sharing is exactly what a single day cannot show.
fn panchanga() -> Section {
    let mut section = Section::new("panchanga");
    let resolved = Profile::shipped(DEFAULT_PROFILE)
        .and_then(|profile| profile.resolve(&SettingsPatch::default()).ok());
    let Some(resolved) = resolved else {
        return section;
    };
    let provider = TestProvider::new();
    let model = DrikSun::new(
        &provider,
        Ayanamsha::Lahiri,
        Sunrise::CentreNoRefraction.into(),
        OverridePolicy::SdkOnly,
        DeltaTModel::TableThenModel,
    );
    let clock = UtcOffset::literal(5, 45, 0);
    let almanac = Almanac::new(
        &provider,
        &resolved,
        &model,
        &Gregorian,
        &clock,
        PrecessionModel::Iau2006,
        DeltaTModel::TableThenModel,
    );
    let place = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.3240),
        Altitude::literal(1400.0),
    );
    let from = CalendarDate::defined(Calendar::Gregorian, 2024, 4, 1);
    let Ok(first) = Gregorian.fixed_of(&from) else {
        return section;
    };
    let Ok(to) = Gregorian.date_of(first.plus_days(9)) else {
        return section;
    };
    let Ok(days) = almanac.between(&from, &to, &place) else {
        return section;
    };
    for day in &days.value {
        section.push(day.day.sunrise.get());
        section.push(day.day.sunset.get());
        section.push(day.day.next_sunrise.get());
        section.push(day.window.from.get());
        section.push(day.window.to.get());
        section.push_int(i64::from(day.month.amanta as u16));
        section.push_int(i64::from(day.month.purnimanta as u16));
        // Every limb's boundaries and the member that ran between them:
        // the instants a search produced, and the integer path that read
        // them, which is the pair a machine could disagree on.
        for span in &day.limbs.tithi {
            section.push_int(i64::from(span.member as u16));
            section.push(span.whole.from.get());
            section.push(span.whole.to.get());
        }
        for span in &day.limbs.nakshatra {
            section.push_int(i64::from(span.member as u16));
            section.push(span.whole.from.get());
            section.push(span.whole.to.get());
        }
        for span in &day.limbs.yoga {
            section.push_int(i64::from(span.member as u16));
            section.push(span.whole.from.get());
            section.push(span.whole.to.get());
        }
        for span in &day.limbs.karana {
            section.push_int(i64::from(span.member as u16));
            section.push(span.whole.from.get());
            section.push(span.whole.to.get());
        }
        for rise in &day.moon.rises {
            section.push(rise.get());
        }
        for set in &day.moon.sets {
            section.push(set.get());
        }
    }
    section
}

/// The Moons both dasha sections are seeded from, spread round the zodiac.
const MOONS: [f64; 4] = [3.7, 101.25, 221.786_980_828_370_36, 347.9];

/// The birth both dasha sections count from.
const DASHA_BIRTH_JD: f64 = 2_447_995.489_583_333_5;

/// The rules every nakshatra-seeded row is walked under.
const DASHA_RULES: DashaRules = DashaRules {
    balance: Balance::Spatial,
    year_length: YearLength::Julian36525,
    birth_period: BirthPeriod::Compressed,
    after_cycle: AfterCycle::Repeat,
    seed_overflow: SeedOverflow::WrapToStart,
    ashtottari_grouping: AshtottariGrouping::ThreeEach,
};

/// A birth with the Moon at `moon` degrees.
fn dasha_birth(moon: f64) -> Option<Birth> {
    Some(Birth {
        instant: JulianDay::literal(DASHA_BIRTH_JD),
        moon: Nas::from_degrees(Degrees::try_new(moon).ok()?),
        moon_span: None,
    })
}

/// Walks a timeline's chain at depth five at instants over three
/// centuries, pushing each period's lord and bounds.
fn push_chains(section: &mut Section, timeline: impl Fn(JulianDay<Utc>, Depth) -> Chain) {
    let Ok(depth) = Depth::try_new(5) else {
        return;
    };
    for i in 0..300 {
        let at = JulianDay::literal(DASHA_BIRTH_JD + f64::from(i) * 365.25 + 0.371 * f64::from(i));
        for period in timeline(at, depth).iter() {
            section.push_int(i64::from(period.lord as u16));
            section.push(period.interval.from.get());
            section.push(period.interval.to.get());
        }
    }
}

/// The nakshatra-seeded rows counted round `wheel`, as each section
/// walks them.
fn rows_on(wheel: Wheel) -> impl Iterator<Item = &'static UduRow> {
    ROWS.iter()
        .chain([&ASHTOTTARI_BPHS])
        .filter(move |row| row.wheel == wheel)
}

/// The dashas: the chain at depth five, at instants over three centuries,
/// for every shipped row on the twenty-seven nakshatras, the Kalachakra
/// and every sign-based row, from Moons spread round the zodiac; each
/// period's lord and bounds.
///
/// The rows counted over the twenty-eight with Abhijit are
/// [`dashas_28`]'s, so a change to either wheel is priced on its own and
/// neither section's work grows when the other gains a row.
fn dashas() -> Section {
    let mut section = Section::new("dashas");
    for birth in MOONS.into_iter().filter_map(dasha_birth) {
        for row in rows_on(Wheel::Nakshatras) {
            if let Ok(dasha) = Dasha::new(row, &birth, DASHA_RULES) {
                push_chains(&mut section, |at, depth| dasha.at(at, depth));
            }
        }
        let kalachakra = KalachakraDasha::new(
            &birth,
            KalachakraRules {
                balance: Balance::Spatial,
                year_length: YearLength::Julian36525,
                after_cycle: AfterCycle::Repeat,
                membership: KalachakraMembership::Listed,
                balance_of: KalachakraBalance::WholePada,
                after_ninth: KalachakraAfterNinth::Reverse,
            },
        );
        if let Ok(dasha) = kalachakra {
            push_chains(&mut section, |at, depth| dasha.at(at, depth));
        }
    }
    let chart = RashiChart {
        lagna: Rashi::Pisces,
        arudha_lagna: Rashi::Gemini,
        navamsa_lagna: Rashi::Aquarius,
        signs: [
            Rashi::Aries,
            Rashi::Scorpio,
            Rashi::Aquarius,
            Rashi::Aries,
            Rashi::Gemini,
            Rashi::Aquarius,
            Rashi::Capricorn,
            Rashi::Capricorn,
            Rashi::Cancer,
        ],
        dignities: [Dignity::Neutral; 9],
        brahma: None,
    };
    for row in RASHI_ROWS {
        let made = RashiDasha::new(
            row,
            &chart,
            JulianDay::literal(DASHA_BIRTH_JD),
            YearLength::Julian36525,
            AfterCycle::Repeat,
            teistro_dasha::RashiRules::RECORDING_ENGINE,
        );
        if let Ok(dasha) = made {
            push_chains(&mut section, |at, depth| dasha.at(at, depth));
        }
    }
    section
}

/// The rows counted over the twenty-eight nakshatras with Abhijit
/// (`03-design/dasha-kernels.md`, "The 28-nakshatra wheel"), from the
/// same Moons and one inside Abhijit, the segment only this wheel has.
fn dashas_28() -> Section {
    let mut section = Section::new("dashas-28");
    for birth in MOONS.into_iter().chain([278.0]).filter_map(dasha_birth) {
        for row in rows_on(Wheel::WithAbhijit) {
            if let Ok(dasha) = Dasha::new(row, &birth, DASHA_RULES) {
                push_chains(&mut section, |at, depth| dasha.at(at, depth));
            }
        }
    }
    section
}

/// The profile the built-in sections read: the shipped default.
fn default_settings() -> Option<teistro_core::settings::Resolved> {
    Profile::shipped(DEFAULT_PROFILE)
        .and_then(|profile| profile.resolve(&SettingsPatch::default()).ok())
}

/// Kathmandu, where every built-in section stands.
fn kathmandu() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.3240),
        Altitude::literal(1400.0),
    )
}

/// A Gregorian day.
fn gregorian(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, year, month, day)
}

/// Every observance's day and the tithi that decided it.
fn push_observances(section: &mut Section, found: &Observances) {
    for observance in &found.observances {
        section.push_int(Gregorian.fixed_of(&observance.day).map_or(0, FixedDay::get));
        section.push(observance.tithi.from.get());
        section.push(observance.tithi.to.get());
    }
}

/// **festivals**: the Nepal pack's festivals, the days counted after them
/// and the Ekadashis over Dashain and Tihar 2024 at Kathmandu, on the
/// built-in ephemeris: the rule evaluation a festival calendar costs, over
/// the days it reads.
fn festivals() -> Section {
    let mut section = Section::new("festivals");
    let Some(resolved) = default_settings() else {
        return section;
    };
    let provider = Builtin::new();
    let delta_t = DeltaTModel::TableThenModel;
    let model = DrikSun::new(
        &provider,
        Ayanamsha::Lahiri,
        Sunrise::CentreNoRefraction.into(),
        OverridePolicy::SdkOnly,
        delta_t,
    );
    let clock = UtcOffset::literal(5, 45, 0);
    let almanac = Almanac::new(
        &provider,
        &resolved,
        &model,
        &Gregorian,
        &clock,
        PrecessionModel::Iau2006,
        delta_t,
    );
    let Ok(days) = almanac.festival_days(
        &gregorian(2024, 9, 25),
        &gregorian(2024, 11, 20),
        &kathmandu(),
    ) else {
        return section;
    };
    let days = days.value;
    let Ok(karmakala) = observances(&FestivalRule::nepal(), &days) else {
        return section;
    };
    push_observances(&mut section, &karmakala);
    if let Ok(counted) = following(&FollowingRule::nepal(), &karmakala.observances, &days) {
        push_observances(&mut section, &counted);
    }
    if let Ok(fasts) = ekadashis(&EkadashiRule::dharmasindhu(), &days) {
        push_observances(&mut section, &fasts);
    }
    section
}

/// **muhurta**: three days' marriage search by Raman's rules at Kathmandu on
/// the built-in ephemeris, every window judged: the search a muhurta
/// request costs.
fn muhurta() -> Section {
    let mut section = Section::new("muhurta");
    let Some(resolved) = default_settings() else {
        return section;
    };
    let provider = Builtin::new();
    let place = kathmandu();
    let (delta_t, precession) = (DeltaTModel::TableThenModel, PrecessionModel::Vondrak2011);
    let model = DrikSun::new(
        &provider,
        Ayanamsha::Lahiri,
        Sunrise::CentreNoRefraction.into(),
        OverridePolicy::PreferNative,
        delta_t,
    );
    let clock = UtcOffset::literal(5, 45, 0);
    let almanac = Almanac::new(
        &provider, &resolved, &model, &Gregorian, &clock, precession, delta_t,
    );
    let founder = Founder::new(
        &provider, &resolved, &model, &Gregorian, &clock, precession, delta_t,
    );
    let completion = Completion::new(&provider, resolved.settings.provider.overrides, delta_t);
    let heliacal = Heliacal::new(
        &completion,
        place,
        Criterion::SURYA_SIDDHANTA,
        Horizon::CENTRE_NO_REFRACTION,
        delta_t,
    );
    let over = Over {
        almanac: &almanac,
        founder: &founder,
        completion: &completion,
        heliacal: &heliacal,
        calendar: &Gregorian,
        clock: &clock,
        settings: &resolved.settings,
        precession,
        delta_t,
    };
    // The chart zodiac taken at 2026-11-01 00:00 UTC.
    let Ok(sources) = ProviderSources::new(&over, place, JulianDay::literal(2_461_345.5)) else {
        return section;
    };
    let request = MuhurtaRequest {
        rules: ActivityRules::raman_marriage(),
        from: gregorian(2026, 11, 25),
        to: gregorian(2026, 11, 27),
        native: None,
        ranking: Ranking::Texts,
        days_with_windows: 3,
        most: 12,
        direction: None,
    };
    let Ok(answer) = teistro_muhurta::search(&sources, &request) else {
        return section;
    };
    section.push_int(i64::try_from(answer.days_judged).unwrap_or(-1));
    section.push_int(i64::try_from(answer.closed.len()).unwrap_or(-1));
    for window in &answer.windows {
        section.push(window.at.from.get());
        section.push(window.at.to.get());
        section.push_int(i64::try_from(window.clauses.len()).unwrap_or(-1));
        section.push_int(i64::try_from(window.barred_by.len()).unwrap_or(-1));
    }
    section
}

/// **panchanga-year**: a year of almanac days at Kathmandu on the built-in
/// ephemeris, the range a year view asks for in one call. A range is
/// answered in rounds that each visit every day of it, so this is where a
/// cache sized for a month thrashes (a year once cost twice what it did
/// before a segment cache meant for a month).
fn panchanga_year() -> Section {
    let mut section = Section::new("panchanga-year");
    let Some(resolved) = default_settings() else {
        return section;
    };
    let provider = Builtin::new();
    let delta_t = DeltaTModel::TableThenModel;
    let model = DrikSun::new(
        &provider,
        Ayanamsha::Lahiri,
        Sunrise::CentreNoRefraction.into(),
        OverridePolicy::SdkOnly,
        delta_t,
    );
    let clock = UtcOffset::literal(5, 45, 0);
    let almanac = Almanac::new(
        &provider,
        &resolved,
        &model,
        &Gregorian,
        &clock,
        PrecessionModel::Iau2006,
        delta_t,
    );
    let Ok(days) = almanac.between(
        &gregorian(2024, 1, 1),
        &gregorian(2024, 12, 31),
        &kathmandu(),
    ) else {
        return section;
    };
    for day in &days.value {
        section.push(day.day.sunrise.get());
        for span in &day.limbs.tithi {
            section.push_int(i64::from(span.member as u16));
            section.push(span.whole.to.get());
        }
        for span in &day.limbs.nakshatra {
            section.push_int(i64::from(span.member as u16));
            section.push(span.whole.to.get());
        }
    }
    section
}

/// A section asked of the façade's own `chart.found` record, as a
/// consumer asks it, on the built-in ephemeris: every number the answer
/// carries in `parts` is pushed, in the order serde writes them.
#[derive(Clone, Copy, Debug)]
pub struct Asked {
    /// The section's name.
    pub name: &'static str,
    /// The request, as JSON.
    pub request: &'static str,
    /// The answer's parts read, in order.
    pub parts: &'static [&'static str],
}

/// The chart at Kathmandu both façade sections ask about.
macro_rules! natal {
    ($($rest:tt)*) => {
        concat!(
            r#"{"instant": 2451545.25, "latitudeDeg": 27.7172, "longitudeDeg": 85.324,
            "altitudeM": 1400, "utcOffsetSeconds": 20700, "#,
            $($rest)*,
            "}"
        )
    };
}

/// The façade's sections.
///
/// **hits** is a year of transit hits (the Sun, Mercury and Saturn
/// ingressing and aspecting the chart's points) and ten years of Sade Sati:
/// the two searches over time a chart request can ask for, which the
/// fuzzer's timeouts found the most expensive.
///
/// **judgement** is what a chart is read for: the Shadbala and Bhava Bala,
/// the Ashtakavarga and the Vimshopaka (all in `charts`), a year of
/// transits read by gochar, an Ashta Koota match and three Tajika annual
/// charts. None of them is over a range a scan walks, so their cost is the
/// rules', which nothing else here counts.
pub const ASKED: [Asked; 2] = [
    Asked {
        name: "hits",
        request: natal!(
            r#""hits": {"from": 2460676.5, "to": 2461041.5, "grahas": ["SUN", "MERCURY", "SATURN"],
                "aspects": [0, 90, 180], "orbDeg": 2},
            "sadeSati": {"from": 2460676.5, "to": 2464329.0, "reckoning": "DEGREE", "spells": [4, 7, 8]}"#
        ),
        parts: &["hits", "sade_sati"],
    },
    Asked {
        name: "judgement",
        request: natal!(
            r#""shadbala": true, "bhavaBala": true, "ashtakavarga": true, "vimshopaka": true,
            "gochar": {"instants": [2460676.5, 2460767.5, 2460859.5, 2460951.5]},
            "matching": {"partner": {"instant": 2451910.75,
                "place": {"latitude": 28.2096, "longitude": 83.9856, "altitude": 820},
                "utcOffsetSeconds": 20700}, "partnerRole": "BRIDE"},
            "varsha": {"through": 3}"#
        ),
        parts: &["charts", "gochar", "matchings", "varsha"],
    },
];

/// One of [`ASKED`], answered. A refusal leaves the section empty, which
/// `tests/asked.rs` holds against: every part each reads is answered.
fn asked(asked: &Asked) -> Section {
    let mut section = Section::new(asked.name);
    let Some(record) = teistro::records::record("chart.found") else {
        return section;
    };
    let Ok(sdk) = teistro::Context::builder()
        .ephemeris([teistro::Ephemeris::Builtin])
        .build()
    else {
        return section;
    };
    if let Ok(answered) = record.answer(&sdk, asked.request) {
        for part in asked.parts {
            push_numbers(&mut section, answered.value.get(*part));
        }
    }
    section
}

/// Every number in `value`, depth first in serde's order.
fn push_numbers(section: &mut Section, value: Option<&serde_json::Value>) {
    match value {
        Some(serde_json::Value::Number(number)) => {
            if let Some(int) = number.as_i64() {
                section.push_int(int);
            } else if let Some(float) = number.as_f64() {
                section.push(float);
            }
        }
        Some(serde_json::Value::Array(items)) => {
            for item in items {
                push_numbers(section, Some(item));
            }
        }
        Some(serde_json::Value::Object(fields)) => {
            for field in fields.values() {
                push_numbers(section, Some(field));
            }
        }
        _ => {}
    }
}
