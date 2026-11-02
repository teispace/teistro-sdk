//! The birth-time sensitivity against skies of straight lines, whose every
//! change is arithmetic.

#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they built"
)]

use teistro_core::angle::Nas;
use teistro_core::catalogue::{DashaSystem, Graha, Nakshatra, Rashi, Vara, Varga};
use teistro_core::error::Error;
use teistro_core::quantity::{Depth, JulianDay, Utc};
use teistro_core::settings::{Balance, root};
use teistro_dasha::{Birth, Dasha, Timeline, VIMSHOTTARI};
use teistro_vargas::{Scheme, sign};

use crate::sensitivity::{
    Change, DashaAt, Fact, Pada, SensitivityRules, SensitivitySky, sensitivity,
};
use crate::{Day, EDGE_TOLERANCE_DAYS, Sky, Window};

const BASE: f64 = 2_460_000.25;

/// A lagna turning once a day from 0° at `BASE`, a Moon moving `speed`
/// degrees a day from `moon`, and, when `middles`, bhava middles that are
/// the lagna turned on by `offset` and every thirtieth after it.
struct Line {
    moon: f64,
    speed: f64,
    middles: Option<f64>,
}

impl Sky for Line {
    fn ascendant_deg(&self, at: JulianDay<Utc>) -> Result<f64, Error> {
        Ok(((at.get() - BASE) * 360.0).rem_euclid(360.0))
    }
    fn sun_deg(&self, _: JulianDay<Utc>) -> Result<f64, Error> {
        Ok(10.0)
    }
    fn moon_deg(&self, at: JulianDay<Utc>) -> Result<f64, Error> {
        Ok((self.moon + self.speed * (at.get() - BASE)).rem_euclid(360.0))
    }
    fn day(&self, _: JulianDay<Utc>) -> Result<Day, Error> {
        Ok(Day::literal(BASE, BASE + 0.5, BASE + 1.0, Vara::Budhavara))
    }
}

impl SensitivitySky for Line {
    fn middles_deg(&self, at: JulianDay<Utc>) -> Result<Option<[f64; 12]>, Error> {
        let Some(offset) = self.middles else {
            return Ok(None);
        };
        let lagna = self.ascendant_deg(at)?;
        Ok(Some(core::array::from_fn(|house| {
            (lagna + offset + 30.0 * f64::from(u8::try_from(house).expect("twelve")))
                .rem_euclid(360.0)
        })))
    }
}

const STILL: Line = Line {
    moon: 100.0,
    speed: 0.0,
    middles: None,
};

fn around(birth: f64, hours: f64) -> Window {
    Window::between(
        JulianDay::literal(birth - hours / 24.0),
        JulianDay::literal(birth + hours / 24.0),
    )
    .expect("a window")
}

/// Every change of a fact, held a second inside it to the birth's value
/// and a second past it to the reported one, as `read` states the fact at
/// an instant.
fn held<T: Clone + PartialEq + core::fmt::Debug>(
    fact: &Fact<T>,
    birth: f64,
    read: impl Fn(f64) -> T,
) {
    let second = 1.0 / 86_400.0;
    if let Some(Change { at, minutes, to }) = &fact.later {
        assert!(((at.get() - birth) * 1440.0 - minutes).abs() < 1e-9);
        assert_eq!(
            read(at.get() - second),
            fact.value,
            "a second before the later change"
        );
        assert_eq!(
            read(at.get() + second),
            *to,
            "a second after the later change"
        );
    }
    if let Some(Change { at, minutes, to }) = &fact.earlier {
        assert!(((birth - at.get()) * 1440.0 - minutes).abs() < 1e-9);
        assert_eq!(
            read(at.get() + second),
            fact.value,
            "a second after the earlier change"
        );
        assert_eq!(
            read(at.get() - second),
            *to,
            "a second before the earlier change"
        );
    }
}

/// The lagna's sign, pada and navamsha change where the arithmetic puts
/// them, to the stated precision, and every change holds a second either
/// side.
#[test]
fn the_lagnas_changes_are_the_arithmetics() {
    // 47.3° of a lagna turning 360° a day.
    let birth = BASE + 47.3 / 360.0;
    let found = sensitivity(
        JulianDay::literal(birth),
        around(birth, 2.0),
        &STILL,
        &SensitivityRules::default(),
    )
    .expect("a reading");
    let minutes_to = |deg: f64| (deg - 47.3) / 360.0 * 1440.0;
    let tolerance = EDGE_TOLERANCE_DAYS * 1440.0;

    assert_eq!(found.lagna_sign.value, Rashi::Taurus);
    let later = found.lagna_sign.later.as_ref().expect("Gemini");
    assert!((later.minutes - minutes_to(60.0)).abs() < tolerance);
    assert_eq!(later.to, Rashi::Gemini);
    let earlier = found.lagna_sign.earlier.as_ref().expect("Aries");
    assert!((earlier.minutes + minutes_to(30.0)).abs() < tolerance);
    assert_eq!(earlier.to, Rashi::Aries);

    // Rohini runs 40° to 53°20′: its third pada 46°40′ to 50°.
    assert_eq!(
        found.lagna_pada.value,
        Pada {
            nakshatra: Nakshatra::Rohini,
            pada: 3
        }
    );
    let later = found.lagna_pada.later.as_ref().expect("a pada");
    assert!((later.minutes - minutes_to(50.0)).abs() < tolerance);
    let earlier = found.lagna_pada.earlier.as_ref().expect("a pada");
    assert!((earlier.minutes + minutes_to(140.0 / 3.0)).abs() < tolerance);

    let lagna = |at: f64| {
        STILL
            .ascendant_deg(JulianDay::literal(at))
            .expect("a lagna")
    };
    held(&found.lagna_sign, birth, |at| {
        Rashi::of_longitude(lagna(at))
    });
    held(&found.lagna_pada, birth, |at| {
        Pada::of(lagna(at)).expect("a pada")
    });
    for (varga, fact) in found.vargas.iter().map(|one| (one.varga, &one.lagna)) {
        let scheme = Scheme::of(varga);
        held(fact, birth, |at| {
            sign(&scheme, Nas::try_from_degrees(lagna(at)).expect("finite"))
        });
        assert!(fact.earlier.is_some() && fact.later.is_some(), "{varga}");
    }
    assert_eq!(found.vargas[0].varga, Varga::D9);
    assert!((found.precision_seconds - 0.008_64).abs() < 1e-12);
    // The Moon stood still, no dasha was asked, and no chalit was read.
    assert!(found.moon_pada.earlier.is_none() && found.moon_pada.later.is_none());
    assert!(found.dasha.is_none() && found.cusps.is_empty());
}

/// Every varga the catalogue names can be followed, and each change holds.
#[test]
fn every_vargas_lagna_holds_either_side_of_its_changes() {
    let birth = BASE + 211.123 / 360.0;
    let vargas: Vec<Varga> = Varga::ALL.into_iter().filter(|v| *v != Varga::D1).collect();
    let rules = SensitivityRules {
        vargas: vargas.clone(),
        ..SensitivityRules::default()
    };
    let found = sensitivity(
        JulianDay::literal(birth),
        around(birth, 3.0),
        &STILL,
        &rules,
    )
    .expect("a reading");
    assert_eq!(found.vargas.len(), vargas.len());
    for one in &found.vargas {
        let scheme = Scheme::of(one.varga);
        held(&one.lagna, birth, |at| {
            let lagna = STILL
                .ascendant_deg(JulianDay::literal(at))
                .expect("a lagna");
            sign(&scheme, Nas::try_from_degrees(lagna).expect("finite"))
        });
    }
}

/// A moving Moon moves its pada and the dasha's balance; each change is
/// where the Moon's arithmetic and the dasha crate put it.
#[test]
fn the_moon_moves_its_pada_and_the_running_dasha() {
    // The Moon 13.2° a day from 221°, the lagna as before.
    let sky = Line {
        moon: 221.0,
        speed: 13.2,
        middles: None,
    };
    let birth = BASE + 0.3;
    let reference = JulianDay::literal(birth + 30.0 * 365.25);
    let rules = teistro_dasha::Rules::of(&root().dasha, DashaSystem::Vimshottari);
    let asked = SensitivityRules {
        dasha: Some(DashaAt {
            at: reference,
            system: DashaSystem::Vimshottari,
            depth: Depth::try_new(3).expect("three"),
            rules,
        }),
        ..SensitivityRules::default()
    };
    let found = sensitivity(JulianDay::literal(birth), around(birth, 12.0), &sky, &asked)
        .expect("a reading");
    let moon = |at: f64| sky.moon_deg(JulianDay::literal(at)).expect("a Moon");
    held(&found.moon_pada, birth, |at| {
        Pada::of(moon(at)).expect("a pada")
    });
    // 221° + 13.2° × 0.3 = 224.96°: Anuradha's 4th pada ends at 226°40′.
    let later = found.moon_pada.later.as_ref().expect("a later pada");
    let to_minutes = (680.0 / 3.0 - 224.96) / 13.2 * 1440.0;
    assert!((later.minutes - to_minutes).abs() < 1e-3);

    let running = found.dasha.as_ref().expect("asked");
    assert_eq!(running.lords.value.len(), 3);
    let lords = |at: f64| -> Vec<Graha> {
        let birth = Birth {
            instant: JulianDay::literal(at),
            moon: Nas::try_from_degrees(moon(at)).expect("finite"),
            moon_span: None,
        };
        Dasha::new(&VIMSHOTTARI, &birth, rules)
            .expect("a dasha")
            .at(reference, Depth::try_new(3).expect("three"))
            .iter()
            .map(|period| period.lord)
            .collect()
    };
    assert!(running.lords.later.is_some() || running.lords.earlier.is_some());
    held(&running.lords, birth, lords);
}

/// A temporal balance reads the Moon's stay in her nakshatra, found by the
/// same bisection, and the dasha it gives agrees with one given that stay
/// by its arithmetic.
#[test]
fn a_temporal_balance_reads_the_moons_stay() {
    let sky = Line {
        moon: 221.0,
        speed: 13.2,
        middles: None,
    };
    let birth = BASE + 0.3;
    let reference = JulianDay::literal(birth + 10.0 * 365.25);
    let rules = teistro_dasha::Rules {
        balance: Balance::Temporal,
        ..teistro_dasha::Rules::of(&root().dasha, DashaSystem::Vimshottari)
    };
    let depth = Depth::try_new(2).expect("two");
    let asked = SensitivityRules {
        dasha: Some(DashaAt {
            at: reference,
            system: DashaSystem::Vimshottari,
            depth,
            rules,
        }),
        ..SensitivityRules::default()
    };
    // Two hours either side keep the Moon in Anuradha, whose stay is below.
    let found = sensitivity(JulianDay::literal(birth), around(birth, 2.0), &sky, &asked)
        .expect("a reading");
    // Anuradha runs 213°20′ to 226°40′.
    let enters = BASE + (640.0 / 3.0 - 221.0) / 13.2;
    let leaves = BASE + (680.0 / 3.0 - 221.0) / 13.2;
    let lords = |at: f64| -> Vec<Graha> {
        let birth = Birth {
            instant: JulianDay::literal(at),
            moon: Nas::try_from_degrees(sky.moon_deg(JulianDay::literal(at)).expect("a Moon"))
                .expect("finite"),
            moon_span: Some(teistro_core::interval::Interval::literal(enters, leaves)),
        };
        Dasha::new(&VIMSHOTTARI, &birth, rules)
            .expect("a dasha")
            .at(reference, depth)
            .iter()
            .map(|period| period.lord)
            .collect()
    };
    let running = found.dasha.as_ref().expect("asked");
    assert_eq!(running.lords.value, lords(birth));
    held(&running.lords, birth, lords);
}

/// The chalit's middles are read when the sky gives them, house by house.
#[test]
fn the_middles_change_with_their_own_degrees() {
    let sky = Line {
        moon: 100.0,
        speed: 0.0,
        middles: Some(17.0),
    };
    let birth = BASE + 5.0 / 360.0;
    let found = sensitivity(
        JulianDay::literal(birth),
        around(birth, 1.0),
        &sky,
        &SensitivityRules::default(),
    )
    .expect("a reading");
    assert_eq!(found.cusps.len(), 12);
    for cusp in &found.cusps {
        let house = f64::from(cusp.house - 1);
        held(&cusp.sign, birth, |at| {
            let lagna = sky.ascendant_deg(JulianDay::literal(at)).expect("a lagna");
            Rashi::of_longitude(lagna + 17.0 + 30.0 * house)
        });
        // 22° into a sign, 8° to the next: thirty-two minutes.
        let later = cusp.sign.later.as_ref().expect("a change");
        assert!((later.minutes - 32.0).abs() < 1e-3, "{}", later.minutes);
    }
}

/// A fact that holds over the whole span says so, and a wider span finds
/// its change.
#[test]
fn a_fact_with_no_change_in_the_span_says_so() {
    let birth = BASE + 45.0 / 360.0;
    let narrow = sensitivity(
        JulianDay::literal(birth),
        around(birth, 0.5),
        &STILL,
        &SensitivityRules::default(),
    )
    .expect("a reading");
    assert!(narrow.lagna_sign.earlier.is_none() && narrow.lagna_sign.later.is_none());
    let wide = sensitivity(
        JulianDay::literal(birth),
        around(birth, 1.5),
        &STILL,
        &SensitivityRules::default(),
    )
    .expect("a reading");
    assert!(wide.lagna_sign.earlier.is_some() && wide.lagna_sign.later.is_some());
}

/// A varga named twice and a coarse seed are refused, named.
#[test]
fn the_rules_are_refused_by_field() {
    let twice = SensitivityRules {
        vargas: vec![Varga::D9, Varga::D9],
        ..SensitivityRules::default()
    };
    assert_eq!(
        twice.check().expect_err("refused").field(),
        Some("rectification.vargas")
    );
    let fine = SensitivityRules {
        seed_minutes: 0.001,
        ..SensitivityRules::default()
    };
    assert_eq!(
        fine.check().expect_err("refused").field(),
        Some("rectification.seedMinutes")
    );
}
