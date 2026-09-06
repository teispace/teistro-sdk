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
//! order, and no knob to turn. The whole scenario is about twenty-five
//! milliseconds of work, which is a nightly digest's rounding error and a
//! comfortable second under a machine simulator, so both gates can afford
//! all of it and neither has to measure a subset of what the other
//! checked.

use teistro_astro::ayanamsha::{self, Basis};
use teistro_astro::delta_t::{DeltaTModel, delta_t};
use teistro_astro::houses::{self, Input};
use teistro_astro::iau;
use teistro_astro::precession::PrecessionModel;
use teistro_calendar::fixed::FixedDay;
use teistro_calendar::shipped;
use teistro_core::catalogue::{Ayanamsha, Calendar, Graha, HouseSystem};
use teistro_core::quantity::{JulianDay, Tt, Ut1};
use teistro_core::settings::{AyanamshaChoice, PolarPolicy};
use teistro_siddhanta::SuryaSiddhanta;

/// One section of the scenario: its name and the values it computed. A
/// value is kept as its bits, so an integer is hashed as an integer and a
/// double as a double, and neither is squeezed through the other.
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

/// The sections in the order every report lists them.
pub const SECTIONS: [&str; 4] = ["calendar", "astro", "houses", "siddhanta"];

/// One section by name, or `None` when nothing is called that.
#[must_use]
pub fn section(name: &str) -> Option<Section> {
    match name {
        "calendar" => Some(calendars()),
        "astro" => Some(astronomy()),
        "houses" => Some(houses()),
        "siddhanta" => Some(siddhanta()),
        _ => None,
    }
}

/// Every section, in order.
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
