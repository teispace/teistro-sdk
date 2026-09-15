//! The dasha crate's budgets (`docs/02-architecture/09-performance-architecture.md`,
//! "Budgets"; `docs/03-design/dasha-kernels.md`, "The cursor"): `at(t, 5)`
//! under 20 microseconds with no allocation, and a materialised depth-3
//! tree under 500 microseconds, for every kernel and for a consumer's own
//! row. Allocations are held by `tests/allocations.rs`; this times them.

#![allow(
    missing_docs,
    clippy::expect_used,
    reason = "a benchmark binary stops on a bad fixture"
)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use teistro_core::angle::Nas;
use teistro_core::catalogue::{Dignity, Nakshatra, Rashi};
use teistro_core::interval::Interval;
use teistro_core::quantity::{Degrees, Depth, JulianDay};
use teistro_core::settings::{
    AfterCycle, Balance, BirthPeriod, KalachakraAfterNinth, KalachakraBalance,
    KalachakraMembership, SeedOverflow, YearLength,
};
use teistro_dasha::{
    Birth, Dasha, KalachakraDasha, KalachakraRules, RashiChart, RashiDasha, Rules, Timeline,
    UduDefinition, VIMSHOTTARI, rashi_row,
};

const BIRTH: f64 = 2_447_995.489_583_333_5;

fn birth() -> Birth {
    Birth {
        instant: JulianDay::literal(BIRTH),
        moon: Nas::from_degrees(Degrees::try_new(221.786_980_828_370_36).expect("finite")),
        moon_span: None,
    }
}

const RULES: Rules = Rules {
    balance: Balance::Spatial,
    year_length: YearLength::Julian36525,
    birth_period: BirthPeriod::Compressed,
    after_cycle: AfterCycle::Repeat,
    seed_overflow: SeedOverflow::WrapToStart,
};

fn chart() -> RashiChart {
    RashiChart {
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
    }
}

/// The instants a timeline asks at: spread over a long life, so a chain
/// deep in a late mahadasha is timed and not only the first.
fn instants() -> Vec<JulianDay<teistro_core::quantity::Utc>> {
    (0..64)
        .map(|i| JulianDay::literal(BIRTH + f64::from(i) * 365.25 * 1.37))
        .collect()
}

/// `at` over every instant, depth five, as a timeline draws it.
fn chains(c: &mut Criterion, name: &str, timeline: &impl Timeline) {
    let at = instants();
    let depth = Depth::try_new(5).expect("in range");
    c.bench_function(&format!("{name}: at(t, 5), per instant"), |b| {
        let mut cycle = at.iter().copied().cycle();
        b.iter(|| {
            let instant = cycle.next().expect("a cycle never ends");
            black_box(timeline.at(black_box(instant), depth))
        });
    });
}

fn benches(c: &mut Criterion) {
    let birth = birth();

    let vimshottari = Dasha::new(&VIMSHOTTARI, &birth, RULES).expect("a dasha");
    c.bench_function("vimshottari: new", |b| {
        b.iter(|| Dasha::new(&VIMSHOTTARI, black_box(&birth), RULES));
    });
    chains(c, "vimshottari", &vimshottari);
    let life = Interval::literal(BIRTH, BIRTH + 120.0 * 365.25);
    let three = Depth::try_new(3).expect("in range");
    c.bench_function("vimshottari: periods(120 years, 3), materialised", |b| {
        b.iter(|| black_box(vimshottari.periods(black_box(life), three)));
    });

    let ashtottari = Dasha::new(&teistro_dasha::row::ASHTOTTARI, &birth, RULES).expect("a dasha");
    chains(c, "ashtottari", &ashtottari);

    let definition = UduDefinition {
        lords: VIMSHOTTARI.lords.to_vec(),
        ..UduDefinition::of("ACME_VIMSHOTTARI", Nakshatra::Ashwini)
    };
    let registered = Dasha::new(&definition.row(), &birth, RULES).expect("a dasha");
    chains(c, "a registered row", &registered);

    let chara_row = rashi_row(teistro_core::catalogue::DashaSystem::Chara).expect("a row");
    let signs = chart();
    let rashi = RashiDasha::new(
        chara_row,
        &signs,
        JulianDay::literal(BIRTH),
        YearLength::Julian36525,
        AfterCycle::Repeat,
        teistro_dasha::RashiRules::RECORDING_ENGINE,
    )
    .expect("a dasha");
    chains(c, "chara", &rashi);

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
    )
    .expect("a dasha");
    chains(c, "kalachakra", &kalachakra);
}

criterion_group!(dasha, benches);
criterion_main!(dasha);
