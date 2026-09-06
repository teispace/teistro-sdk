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

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

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
    values: Vec<u64>,
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

    /// The section's digest: every value's bits, in order.
    fn digest(&self) -> String {
        let mut hasher = Sha256::new();
        for value in &self.values {
            hasher.update(value.to_le_bytes());
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

/// Every value as its bits, so two machines that disagree can be told how
/// far apart they are rather than only that they are.
fn values_file(sections: &[Section], path: &Path) -> std::io::Result<()> {
    let mut out = String::new();
    for section in sections {
        for (index, value) in section.values.iter().enumerate() {
            // As a number, not as its bytes: a byte-reversed hex string
            // reads back as a value nowhere near the one written, and
            // every distance taken over it is meaningless.
            let _ = writeln!(out, "{}\t{index}\t{value:016x}", section.name);
        }
    }
    std::fs::write(path, out)
}

pub(crate) fn report(values: Option<&Path>) -> i32 {
    let sections = [calendars(), astronomy(), houses(), siddhanta()];
    let mut all = Sha256::new();
    println!("{:<12} {:>9}  digest", "section", "values");
    for section in &sections {
        let digest = section.digest();
        all.update(digest.as_bytes());
        println!("{:<12} {:>9}  {digest}", section.name, section.values.len());
    }
    println!("{:<12} {:>9}  {}", "all", "", hex(&all.finalize()));
    if let Some(path) = values {
        if let Err(error) = values_file(&sections, path) {
            eprintln!("cannot write {}: {error}", path.display());
            return 1;
        }
        eprintln!("wrote every value to {}", path.display());
    }
    0
}

/// Compares two value files and reports how far apart they are: how many
/// values differ, and the largest difference in units in the last place.
/// A difference in the last place is still a difference, but knowing it
/// is one place rather than a thousand is the difference between a
/// rounding mode and a wrong formula.
pub(crate) fn compare(left: &Path, right: &Path) -> i32 {
    let (Ok(left_text), Ok(right_text)) = (
        std::fs::read_to_string(left),
        std::fs::read_to_string(right),
    ) else {
        eprintln!("both value files must be readable");
        return 1;
    };
    let (left_values, right_values) = (read_values(&left_text), read_values(&right_text));
    if left_values.len() != right_values.len() {
        println!(
            "the two runs computed {} and {} values; they are not the same scenario",
            left_values.len(),
            right_values.len()
        );
        return 1;
    }
    let mut counts: BTreeMap<&str, Difference> = BTreeMap::new();
    for (index, ((section, left), (_, right))) in left_values.iter().zip(&right_values).enumerate()
    {
        let entry = counts.entry(section.as_str()).or_default();
        entry.values += 1;
        if left == right {
            continue;
        }
        if entry.examples.len() < 3 {
            entry
                .examples
                .push((index, f64::from_bits(*left), f64::from_bits(*right)));
        }
        entry.differ += 1;
        let places = ulps(*left, *right);
        entry.ulps = entry.ulps.max(places);
        match places {
            0..=1 => entry.near += 1,
            2..=1_000 => entry.close += 1,
            _ => entry.far += 1,
        }
        let (a, b) = (f64::from_bits(*left), f64::from_bits(*right));
        let scale = a.abs().max(b.abs()).max(f64::MIN_POSITIVE);
        entry.relative = entry.relative.max((a - b).abs() / scale);
    }
    let mut differing = 0;
    println!(
        "{:<12} {:>8} {:>8} {:>8} {:>8} {:>8} {:>13}",
        "section", "values", "differ", "1 place", "to 1e3", "beyond", "max relative"
    );
    for (section, difference) in &counts {
        differing += difference.differ;
        println!(
            "{section:<12} {:>8} {:>8} {:>8} {:>8} {:>8} {:>13.3e}",
            difference.values,
            difference.differ,
            difference.near,
            difference.close,
            difference.far,
            difference.relative
        );
    }
    if differing == 0 {
        println!("every value is bit for bit the same");
        return 0;
    }
    for (section, difference) in &counts {
        for (index, left, right) in &difference.examples {
            println!("{section}[{index}]: {left:.17e} against {right:.17e}");
        }
    }
    println!("{differing} value(s) differ");
    let far: usize = counts.values().map(|d| d.far).sum();
    if far > 0 {
        println!(
            "{far} of them by more than a thousand places, which is a wrap or a sign rather than a rounding"
        );
    }
    1
}

/// What two runs did to one section: how many values differ, how far
/// apart they are, and how the distances are spread. A difference of a
/// place or two is a maths library rounding differently; a difference of
/// a whole turn is a value at a wrap, where one place below 360 on one
/// machine is one place above 0 on the other, and the quantity has not
/// really moved at all. The spread tells them apart.
#[derive(Default)]
struct Difference {
    values: usize,
    differ: usize,
    /// Differing by one place.
    near: usize,
    /// Differing by two to a thousand places.
    close: usize,
    /// Differing by more, which is a wrap, a sign or a real disagreement.
    far: usize,
    ulps: u64,
    relative: f64,
    /// The first few, so a reader can see what kind of difference it is
    /// rather than infer it from a distance.
    examples: Vec<(usize, f64, f64)>,
}

/// A value file as its sections and bits.
fn read_values(text: &str) -> Vec<(String, u64)> {
    text.lines()
        .filter_map(|line| {
            let mut parts = line.split('\t');
            let section = parts.next()?;
            let _index = parts.next()?;
            let bits = u64::from_str_radix(parts.next()?, 16).ok()?;
            Some((section.to_string(), bits))
        })
        .collect()
}

/// How many representable doubles lie between two of them, which is the
/// honest measure of "how different" for numbers of the same magnitude.
fn ulps(left: u64, right: u64) -> u64 {
    // The bits of a double order like an integer within a sign; the two
    // signs are counted from zero outwards and added, which is the
    // distance across zero.
    let split = |bits: u64| -> (bool, u64) { (bits >> 63 == 1, bits & !(1 << 63)) };
    match (split(left), split(right)) {
        ((a_sign, a), (b_sign, b)) if a_sign == b_sign => a.abs_diff(b),
        ((_, a), (_, b)) => a.saturating_add(b),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, reason = "tests fail by panicking")]

    use super::*;

    #[test]
    fn the_distance_between_two_doubles_is_the_places_between_them() {
        let one = 1.0_f64.to_bits();
        assert_eq!(ulps(one, one), 0);
        assert_eq!(ulps(one, (1.0_f64 + f64::EPSILON).to_bits()), 1);
        assert_eq!(ulps(one, (1.0_f64 - f64::EPSILON / 2.0).to_bits()), 1);
        assert_eq!(ulps(0.0_f64.to_bits(), (-0.0_f64).to_bits()), 0);
        assert_eq!(ulps(1.0_f64.to_bits(), (-1.0_f64).to_bits()), 2 * one);
    }
}
