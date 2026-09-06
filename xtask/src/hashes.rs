//! The determinism digest: what this build computes for a fixed
//! scenario, hashed, so two architectures can be compared without moving
//! numbers between them (the roadmap's cross-architecture hash matrix,
//! and Phase 1's exit criterion "hash-identical across x86-64 and
//! aarch64").
//!
//! Every value is hashed as its bits, because a difference of one unit in
//! the last place is a difference: the point is to find out whether the
//! same source computes the same numbers on another machine, not to
//! decide how close is close enough. The digest is reported per section,
//! so a difference says which layer moved rather than only that one did.
//!
//! `cargo xtask hashes` prints the report; the matrix runs it on each
//! architecture and compares what they print.

use std::fmt::Write as _;

use sha2::{Digest, Sha256};
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
struct Section {
    name: &'static str,
    values: Vec<[u8; 8]>,
}

impl Section {
    fn new(name: &'static str) -> Section {
        Section {
            name,
            values: Vec::new(),
        }
    }

    fn push(&mut self, value: f64) {
        self.values.push(value.to_bits().to_le_bytes());
    }

    fn push_int(&mut self, value: i64) {
        self.values.push(value.to_le_bytes());
    }

    /// The section's digest: every value's bits, in order.
    fn digest(&self) -> String {
        let mut hasher = Sha256::new();
        for value in &self.values {
            hasher.update(value);
        }
        hex(&hasher.finalize())
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut out, byte| {
        let _ = write!(out, "{byte:02x}");
        out
    })
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

pub(crate) fn report() -> i32 {
    let sections = [calendars(), astronomy(), houses(), siddhanta()];
    let mut all = Sha256::new();
    println!("{:<12} {:>9}  digest", "section", "values");
    for section in &sections {
        let digest = section.digest();
        all.update(digest.as_bytes());
        println!("{:<12} {:>9}  {digest}", section.name, section.values.len());
    }
    println!("{:<12} {:>9}  {}", "all", "", hex(&all.finalize()));
    0
}
