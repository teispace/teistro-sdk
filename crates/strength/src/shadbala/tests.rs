#![allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::unwrap_used,
    reason = "tests index what they built, compare exact values and fail by panicking"
)]

use teistro_core::catalogue::{BalaScheme, Graha, Rashi};
use teistro_core::settings::{
    Benefics, DigKendras, Drekkana, Drik, Kranti, PreDawnNight, Profile, Saptavargaja,
    SettingsPatch, root,
};

use super::kaala::{ayana, nathonnatha, paksha, weekday_lord};
use super::sthana::compound_virupas;
use super::*;

/// A chart at 0° Aries tropical for every graha, everything in the first
/// house of a Mesha lagna, at noon of a twelve-hour day.
fn chart() -> ShadbalaChart {
    let at = ShadbalaGraha {
        longitude: 0.0,
        tropical: 0.0,
        latitude: 0.0,
        house: 1,
    };
    ShadbalaChart {
        grahas: [at; 7],
        rahu: None,
        vargas: [[Rashi::Aries; 7]; 7],
        instant: 2_451_545.0,
        sunrise: 2_451_544.75,
        sunset: 2_451_545.25,
        next_sunrise: 2_451_545.75,
        after_midnight: false,
        civil_day: 2_451_545,
        weekday_lord: Graha::Saturn,
        hora_lord: Graha::Sun,
        sankranti_lord: Some(Graha::Venus),
        ascendant: 0.0,
        midheaven: 270.0,
        ayanamsha: 0.0,
        obliquity: 23.44,
    }
}

fn of(chart: &ShadbalaChart, rules: ShadbalaRules, graha: Graha) -> GrahaShadbala {
    ShadbalaReading::of(chart, rules)
        .grahas
        .into_iter()
        .find(|g| g.graha == graha)
        .unwrap()
}

#[test]
fn the_rules_follow_the_settings_and_refuse_an_undefined_scheme() {
    assert_eq!(ShadbalaRules::of(&root()).unwrap(), ShadbalaRules::BPHS);
    let engine = Profile::shipped("conformance-baseline")
        .unwrap()
        .resolve(&SettingsPatch::default())
        .unwrap()
        .settings;
    assert_eq!(
        ShadbalaRules::of(&engine).unwrap(),
        ShadbalaRules::RECORDING_ENGINE
    );
    let mut extended = root();
    extended.strength.bala_scheme = BalaScheme::ParasharaExtended;
    let refused = ShadbalaRules::of(&extended).unwrap_err();
    assert_eq!(refused.field(), Some("strength.bala_scheme"));
}

#[test]
fn the_ahargana_s_lords_are_the_chapter_s_and_raman_s() {
    // Burgess's day, 1 January 1860, was a Sunday.
    assert_eq!(weekday_lord(714_404_108_572), Graha::Sun);
    // v. 13's worked month: 2176 months completed, a Friday.
    assert_eq!(weekday_lord(2176 * 2), Graha::Venus);
    // Raman's Standard Horoscope, 16 October 1918: the ahargana counted to
    // and including the day is 714,404,130,045, and its year's lord Saturn,
    // its month's Mercury (Examples 20 and 21).
    let day = ShadbalaChart {
        civil_day: 2_421_883,
        ..chart()
    };
    let reading = ShadbalaReading::of(&day, ShadbalaRules::BPHS);
    let abda: Vec<Graha> = reading
        .grahas
        .iter()
        .filter(|g| g.kaala.abda == 15.0)
        .map(|g| g.graha)
        .collect();
    let masa: Vec<Graha> = reading
        .grahas
        .iter()
        .filter(|g| g.kaala.masa == 30.0)
        .map(|g| g.graha)
        .collect();
    assert_eq!(abda, [Graha::Saturn]);
    assert_eq!(masa, [Graha::Mercury]);
    // The engine's reads the sankranti and the Sun's sign.
    let engine = of(&day, ShadbalaRules::RECORDING_ENGINE, Graha::Venus);
    assert_eq!(engine.kaala.abda, 15.0);
}

#[test]
fn nathonnatha_runs_from_midnight_under_the_text() {
    let rules = ShadbalaRules::BPHS;
    let noon = chart();
    assert_eq!(nathonnatha(Graha::Sun, &noon, &rules), 60.0);
    assert_eq!(nathonnatha(Graha::Moon, &noon, &rules), 0.0);
    assert_eq!(nathonnatha(Graha::Mercury, &noon, &rules), 60.0);
    let midnight = ShadbalaChart {
        instant: 2_451_545.5,
        ..chart()
    };
    assert_eq!(nathonnatha(Graha::Saturn, &midnight, &rules), 60.0);
    assert_eq!(nathonnatha(Graha::Jupiter, &midnight, &rules), 0.0);
    // Six hours from midnight is fifteen ghatis: half strength either way.
    let dawn = ShadbalaChart {
        instant: 2_451_545.75 - 1e-9,
        ..chart()
    };
    assert!((nathonnatha(Graha::Mars, &dawn, &rules) - 30.0).abs() < 1e-5);
}

#[test]
fn the_engine_loses_a_pre_dawn_night_and_the_text_does_not() {
    let pre_dawn = ShadbalaChart {
        instant: 2_451_545.6,
        after_midnight: true,
        ..chart()
    };
    let engine = ShadbalaRules::RECORDING_ENGINE;
    assert_eq!(nathonnatha(Graha::Moon, &pre_dawn, &engine), 0.0);
    assert_eq!(of(&pre_dawn, engine, Graha::Mars).kaala.tribhaga, 0.0);
    let previous = ShadbalaRules {
        pre_dawn_night: PreDawnNight::PreviousEvening,
        ..engine
    };
    assert!(nathonnatha(Graha::Moon, &pre_dawn, &previous) > 0.0);
    // 0.35 of a half-day night: its last third, Mars's.
    assert_eq!(of(&pre_dawn, previous, Graha::Mars).kaala.tribhaga, 60.0);
}

#[test]
fn the_luminaries_ayana_and_cheshta_follow_each_reading() {
    // At the equinox with no latitude every graha's Ayana is 30.
    let equinox = chart();
    for kranti in [Kranti::True, Kranti::Ecliptic, Kranti::HinduTable] {
        assert!((ayana(Graha::Moon, &equinox, kranti) - 30.0).abs() < 1e-12);
    }
    let text = of(&equinox, ShadbalaRules::BPHS, Graha::Sun);
    assert_eq!(text.kaala.ayana, 2.0 * text.cheshta);
    let engine = of(&equinox, ShadbalaRules::RECORDING_ENGINE, Graha::Sun);
    assert_eq!(engine.kaala.ayana, 0.0);
    let sripati = of(&equinox, ShadbalaRules::SRIPATI, Graha::Sun);
    assert_eq!(sripati.cheshta, 0.0);
    // Opposite the Sun: a full Moon, whose Paksha is doubled.
    let mut full = chart();
    full.grahas[1].longitude = 180.0;
    let moon = of(&full, ShadbalaRules::BPHS, Graha::Moon);
    assert_eq!(moon.kaala.paksha, 120.0);
    assert_eq!(moon.cheshta, moon.kaala.paksha);
    assert_eq!(
        of(&full, ShadbalaRules::RECORDING_ENGINE, Graha::Moon).cheshta,
        60.0
    );
    assert_eq!(of(&full, ShadbalaRules::SRIPATI, Graha::Moon).cheshta, 0.0);
    // A graha's true declination lifts with its latitude.
    let mut north = chart();
    north.grahas[3].latitude = 3.0;
    assert!(ayana(Graha::Mercury, &north, Kranti::True) > 30.0);
    assert!((ayana(Graha::Mercury, &north, Kranti::Ecliptic) - 30.0).abs() < 1e-12);
}

#[test]
fn the_hindu_table_gives_raman_s_krantis() {
    // Raman's Example 32: the Sun at sayana 202° 10′, 8.75° south, whose
    // doubled Ayana is 38.12 (Example 33).
    let mut chart = chart();
    chart.grahas[0].tropical = 202.0 + 10.0 / 60.0;
    let sun = ayana(Graha::Sun, &chart, Kranti::HinduTable);
    assert!((2.0 * sun - 38.12).abs() < 0.05, "{sun}");
    // A bhuja of 90° reaches the full 24°.
    chart.grahas[4].tropical = 90.0;
    assert_eq!(ayana(Graha::Jupiter, &chart, Kranti::HinduTable), 60.0);
}

#[test]
fn the_benefics_are_conditional_under_sripati() {
    let mut chart = chart();
    // A new Moon is a malefic, a full Moon a benefic.
    chart.grahas[1].longitude = 10.0;
    assert!(!is_benefic(Graha::Moon, &chart, Benefics::Conditional));
    assert!(is_benefic(Graha::Moon, &chart, Benefics::Fixed));
    chart.grahas[1].longitude = 150.0;
    assert!(is_benefic(Graha::Moon, &chart, Benefics::Conditional));
    // Mercury beside the Sun is a malefic; alone it is a benefic.
    assert!(!is_benefic(Graha::Mercury, &chart, Benefics::Conditional));
    chart.grahas[3].longitude = 45.0;
    assert!(is_benefic(Graha::Mercury, &chart, Benefics::Conditional));
    chart.rahu = Some(40.0);
    assert!(!is_benefic(Graha::Mercury, &chart, Benefics::Conditional));
    // Paksha follows: Mercury beside Rahu takes the malefic's share.
    let rules = ShadbalaRules::SRIPATI;
    assert_eq!(paksha(Graha::Mercury, &chart, &rules), 60.0 - 50.0);
}

#[test]
fn the_natural_strengths_are_sevenths_or_the_engine_s_hundredths() {
    let text = ShadbalaReading::of(&chart(), ShadbalaRules::BPHS);
    let sum: f64 = text.grahas.iter().map(|g| g.naisargika).sum();
    assert!((sum - 240.0).abs() < 1e-12, "28 sevenths of 60");
    let engine = ShadbalaReading::of(&chart(), ShadbalaRules::RECORDING_ENGINE);
    assert_eq!(engine.grahas[1].naisargika, 51.43);
    assert_eq!(text.grahas[0].required_rupas, 6.5);
    assert_eq!(engine.grahas[0].required_rupas, 5.0);
}

#[test]
fn dig_is_full_at_the_graha_s_own_angle() {
    let mut chart = chart();
    chart.grahas[0].longitude = chart.midheaven;
    assert_eq!(dig::of(Graha::Sun, &chart, DigKendras::Angles), 60.0);
    assert_eq!(dig::of(Graha::Mercury, &chart, DigKendras::Angles), 60.0);
    chart.midheaven = 300.0;
    chart.grahas[0].longitude = 300.0;
    assert_eq!(dig::of(Graha::Sun, &chart, DigKendras::Angles), 60.0);
    assert_eq!(
        dig::of(Graha::Sun, &chart, DigKendras::LagnaProjection),
        50.0
    );
}

#[test]
fn drik_weighs_each_reading_s_drishtis() {
    let mut chart = chart();
    // Jupiter opposite the Sun, in its seventh house, and every other graha
    // with the Sun, looking at nothing.
    chart.grahas[4].longitude = 180.0;
    chart.grahas[4].house = 7;
    let rules = |drik| ShadbalaRules {
        drik,
        ..ShadbalaRules::BPHS
    };
    let sun = |chart: &ShadbalaChart, drik| of(chart, rules(drik), Graha::Sun).drik;
    assert_eq!(sun(&chart, Drik::QuarterWithJupiterMercury), 15.0 + 60.0);
    assert_eq!(sun(&chart, Drik::Quarter), 15.0);
    assert_eq!(sun(&chart, Drik::Full), 60.0);
    // Saturn there instead takes a quarter away under both sphuta readings.
    chart.grahas[4].longitude = 0.0;
    chart.grahas[4].house = 1;
    chart.grahas[6].longitude = 180.0;
    chart.grahas[6].house = 7;
    assert_eq!(sun(&chart, Drik::QuarterWithJupiterMercury), -15.0);
    assert_eq!(sun(&chart, Drik::Full), -60.0);
}

#[test]
fn the_saptavargaja_counts_the_moolatrikona_in_the_rasi_alone() {
    let rasi = [Rashi::Leo; 7];
    // The Sun in Leo, its moolatrikona rasi: 45 there, and its own sign's 30
    // in any other varga (B.V. Raman, Art. 30).
    assert_eq!(compound_virupas(Graha::Sun, Rashi::Leo, true, &rasi), 45.0);
    assert_eq!(compound_virupas(Graha::Sun, Rashi::Leo, false, &rasi), 30.0);
    let mut chart = chart();
    chart.vargas = [[Rashi::Leo; 7]; 7];
    let rules = |saptavargaja| ShadbalaRules {
        saptavargaja,
        ..ShadbalaRules::BPHS
    };
    let sun = |rule| of(&chart, rules(rule), Graha::Sun).sthana.saptavargaja;
    assert_eq!(sun(Saptavargaja::Compound), 45.0 + 6.0 * 30.0);
    assert_eq!(sun(Saptavargaja::Natural), 7.0 * 30.0);
}

#[test]
fn the_drekkana_order_is_each_reading_s() {
    let mut chart = chart();
    // Mercury, neuter, in the middle decanate; the Moon, female, in the last.
    chart.grahas[3].longitude = 15.0;
    chart.grahas[1].longitude = 25.0;
    let rules = |drekkana| ShadbalaRules {
        drekkana,
        ..ShadbalaRules::BPHS
    };
    let at = |rule, graha| of(&chart, rules(rule), graha).sthana.drekkana;
    assert_eq!(at(Drekkana::MaleNeuterFemale, Graha::Mercury), 15.0);
    assert_eq!(at(Drekkana::MaleNeuterFemale, Graha::Moon), 15.0);
    assert_eq!(at(Drekkana::MaleFemaleNeuter, Graha::Mercury), 0.0);
    assert_eq!(at(Drekkana::MaleFemaleNeuter, Graha::Moon), 0.0);
}

#[test]
fn a_war_moves_its_difference_from_the_vanquished_to_the_victor() {
    let mut chart = chart();
    // Mars at 10.2°, Saturn at 10.7°: at war, Mars of lesser longitude.
    chart.grahas[2].longitude = 10.2;
    chart.grahas[6].longitude = 10.7;
    let at_war = ShadbalaReading::of(&chart, ShadbalaRules::BPHS);
    let peace = ShadbalaReading::of(
        &chart,
        ShadbalaRules {
            yuddha: teistro_core::settings::Yuddha::None,
            ..ShadbalaRules::BPHS
        },
    );
    let (mars, saturn) = (at_war.grahas[2], at_war.grahas[6]);
    assert!(mars.kaala.yuddha > 0.0);
    assert_eq!(mars.kaala.yuddha, -saturn.kaala.yuddha);
    let weight = |g: &GrahaShadbala| g.sthana.total() + g.dig + g.kaala.to_hora();
    let expected = (weight(&peace.grahas[2]) - weight(&peace.grahas[6])).abs() / (158.0 - 9.4);
    assert!((mars.kaala.yuddha - expected).abs() < 1e-12);
    assert_eq!(peace.grahas[2].kaala.yuddha, 0.0);
}

#[test]
fn every_graha_s_total_is_its_six_strengths_under_every_reading() {
    for rules in [
        ShadbalaRules::BPHS,
        ShadbalaRules::SRIPATI,
        ShadbalaRules::RECORDING_ENGINE,
    ] {
        for graha in ShadbalaReading::of(&chart(), rules).grahas {
            let six = graha.sthana.total()
                + graha.dig
                + graha.kaala.total()
                + graha.cheshta
                + graha.naisargika
                + graha.drik;
            assert_eq!(graha.virupas, six);
            assert_eq!(graha.rupas, six / 60.0);
            assert_eq!(graha.strong, graha.rupas >= graha.required_rupas);
        }
    }
}

#[test]
fn the_phalas_are_each_reading_s_means() {
    let mut chart = chart();
    // Mars 90° from its debilitation point: an Uchcha of 30.
    chart.grahas[2].longitude = 208.0;
    let rules = |ishta_kashta| ShadbalaRules {
        ishta_kashta,
        ..ShadbalaRules::BPHS
    };
    let mars = |rule| of(&chart, rules(rule), Graha::Mars);
    let rays = mars(teistro_core::settings::IshtaKashta::Rays);
    assert!((rays.ishta - f64::midpoint(rays.sthana.uchcha, rays.cheshta)).abs() < 1e-12);
    assert!((rays.ishta + rays.kashta - 60.0).abs() < 1e-12);
    // V. 5: the rays' mean, 1 + 3 x 30 / 30 = 4 for the Uchcha, and 8 less.
    let cheshta_rays = 1.0 + rays.cheshta / 10.0;
    assert!((rays.subha_rashmi - f64::midpoint(4.0, cheshta_rays)).abs() < 1e-12);
    assert!((rays.subha_rashmi + rays.ashubha_rashmi - 8.0).abs() < 1e-12);
    let roots = mars(teistro_core::settings::IshtaKashta::SquareRoots);
    assert!((roots.ishta - (roots.sthana.uchcha * roots.cheshta).sqrt()).abs() < 1e-12);
    assert!(
        (roots.kashta - ((60.0 - roots.sthana.uchcha) * (60.0 - roots.cheshta)).sqrt()).abs()
            < 1e-12
    );
    // The Sun's own kendra: sayana at 0°, three signs on, a third of 90°.
    let sun = of(
        &chart,
        rules(teistro_core::settings::IshtaKashta::SquareRoots),
        Graha::Sun,
    );
    assert!((sun.ishta - (sun.sthana.uchcha * 30.0).sqrt()).abs() < 1e-12);
    let engine = of(&chart, ShadbalaRules::RECORDING_ENGINE, Graha::Sun);
    assert!((engine.ishta - (engine.sthana.uchcha * engine.cheshta).sqrt()).abs() < 1e-12);
}
