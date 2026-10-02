//! The essential dignities through the façade: the sect read from the
//! Sun's altitude in every zodiac and at the poles, the chart's daylight as
//! the named alternative, and every knob reported back; and the
//! accidental fortitudes read from the chart's own houses, motions and
//! stars; and the lots, Fortune by night read off the Moon's own horizon
//! (`docs/03-design/essential-dignities.md`, `sect-measured.md`,
//! `hellenistic-lots.md`).

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they found"
)]

use teistro::catalogue::{Graha, HouseSystem, Rashi};
use teistro::hellenistic::house_of;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{
    Accident, AccidentalRules, ChartRequest, ConsiderationRules, Context, DignityRequest,
    DignityRules, Document, Ephemeris, FortitudeRequest, FortuneRule, ImpedimentKind, Lot,
    LotRequest, PerfectionRules, PtolemaicAspect, Scores, Sect, SectRule, Terms, UtcOffset,
};

fn context(patch: Option<&str>) -> Context {
    let builder = Context::builder()
        .profile("conformance-baseline")
        .ephemeris([Ephemeris::Builtin]);
    match patch {
        Some(json) => builder.settings_json(json),
        None => builder,
    }
    .build()
    .unwrap()
}

fn chart(sdk: &Context, latitude: f64, longitude: f64, jd: f64) -> Document {
    let place = Place::new(
        Latitude::literal(latitude),
        Longitude::literal(longitude),
        Altitude::literal(0.0),
    );
    sdk.chart()
        .reading(
            JulianDay::<Utc>::literal(jd),
            &ChartRequest::at(place, UtcOffset::UTC),
        )
        .unwrap()
        .value
}

/// Tromsø, the corpus's polar fixtures (c028, c029).
const TROMSO: (f64, f64) = (69.6492, 18.9553);

#[test]
fn a_polar_night_noon_is_a_night_chart_and_a_midnight_sun_noon_a_day_chart() {
    let sdk = context(None);
    // 21 December 1988, 11:00 UTC: the Sun culminates three degrees under
    // the horizon. Valens's degrees from the Ascendant read it as a day
    // chart there, which is why the horizon is the altitude.
    let winter = chart(&sdk, TROMSO.0, TROMSO.1, 2_447_516.958_333_333_5);
    let read = sdk
        .chart()
        .dignities(&winter, &DignityRequest::default())
        .unwrap();
    assert_eq!(read.sect, Sect::Night);
    // 21 June 1988, 10:00 UTC: the Sun has not set for weeks.
    let summer = chart(&sdk, TROMSO.0, TROMSO.1, 2_447_333.916_666_666_5);
    let read = sdk
        .chart()
        .dignities(&summer, &DignityRequest::default())
        .unwrap();
    assert_eq!(read.sect, Sect::Day);
}

#[test]
fn the_sect_does_not_depend_on_the_zodiac_and_the_signs_do() {
    let sidereal = context(None);
    let tropical = context(Some(r#"{"frame": {"zodiac": "TROPICAL"}}"#));
    // Kathmandu, through a day at three-hour steps.
    for step in 0..8 {
        let jd = 2_460_676.5 + f64::from(step) * 0.125;
        let one = chart(&sidereal, 27.7172, 85.324, jd);
        let other = chart(&tropical, 27.7172, 85.324, jd);
        let asked = DignityRequest::default();
        let (one, other) = (
            sidereal.chart().dignities(&one, &asked).unwrap(),
            tropical.chart().dignities(&other, &asked).unwrap(),
        );
        assert_eq!(one.sect, other.sect, "step {step}");
        // The same planets, a whole ayanamsha apart.
        let shift = other.planets[3].longitude_deg - one.planets[3].longitude_deg;
        assert!(
            (shift.rem_euclid(360.0) - 24.2).abs() < 0.5,
            "step {step}: {shift}"
        );
    }
}

#[test]
fn the_daylight_rule_reads_the_charts_own_day_part() {
    let sdk = context(None);
    let lit = DignityRequest::default().with_sect_rule(SectRule::Daylight);
    for step in 0..8 {
        let document = chart(&sdk, 27.7172, 85.324, 2_460_676.5 + f64::from(step) * 0.125);
        let read = sdk.chart().dignities(&document, &lit).unwrap();
        let daylight = document.foundation.day.part.is_daylight();
        assert_eq!(read.sect == Sect::Day, daylight, "step {step}");
        assert_eq!(read.sect_rule, SectRule::Daylight);
    }
}

#[test]
fn every_knob_comes_back_and_moves_what_it_names() {
    let sdk = context(None);
    let document = chart(&sdk, 27.7172, 85.324, 2_460_676.75);
    let scores = Scores::new(1, 1, 1, 1, 1, -1, -1, 0);
    let rules = DignityRules::LILLY.with_terms(Terms::Egyptian);
    let asked = DignityRequest::default()
        .with_sect_rule(SectRule::Night)
        .with_rules(rules)
        .with_scores(scores);
    let read = sdk.chart().dignities(&document, &asked).unwrap();
    assert_eq!(
        (read.sect, read.sect_rule, read.rules, read.scores),
        (Sect::Night, SectRule::Night, rules, scores)
    );
    let order: Vec<_> = read.planets.iter().map(|at| at.planet).collect();
    assert_eq!(order, teistro::hellenistic::CHALDEAN_ORDER);
    for at in read.planets {
        let placed = document.foundation.graha(at.planet).unwrap();
        assert_eq!(at.longitude_deg, placed.longitude_deg, "{:?}", at.planet);
        assert_eq!(at.score, at.dignity.score(&scores), "{:?}", at.planet);
    }
    // The Sun and the Moon take no term under any table.
    for luminary in [Graha::Sun, Graha::Moon] {
        let at = read
            .planets
            .iter()
            .find(|at| at.planet == luminary)
            .unwrap();
        assert!(!at.dignity.term, "{luminary:?}");
    }
}

/// London, 1 January 2000 at noon UTC.
fn london(sdk: &Context) -> Document {
    chart(sdk, 51.5, -0.12, 2_451_545.0)
}

#[test]
fn the_fortitudes_read_the_charts_own_houses_and_motions() {
    let sdk = context(Some(r#"{"frame": {"zodiac": "TROPICAL"}}"#));
    let natal = london(&sdk);
    let read = sdk
        .chart()
        .fortitudes(&natal, &FortitudeRequest::default())
        .unwrap();
    assert_eq!(read.sky.houses, HouseSystem::Regiomontanus);
    let rules = AccidentalRules::LILLY;
    for (at, planet) in read.planets.iter().zip(read.dignities.planets) {
        assert_eq!(at.planet, planet.planet);
        let house = house_of(
            planet.longitude_deg,
            &read.sky.cusps_deg,
            rules.cusp_orb_deg,
        );
        assert_eq!(at.house, house, "{:?}", at.planet);
        let graha = natal.foundation.graha(at.planet).unwrap();
        let backward = at.accidents.contains(&Accident::Retrograde);
        assert_eq!(backward, graha.speed_deg_per_day < 0.0, "{:?}", at.planet);
    }
    // The tenth cusp is the chart's own midheaven.
    let angles = sdk.chart().angles(&natal).unwrap();
    assert!((read.sky.cusps_deg[9] - angles.midheaven_deg).abs() < 1e-9);
    assert!((read.sky.cusps_deg[0] - angles.ascendant_deg).abs() < 1e-9);
    // Each net is the essential score and reception with the accidental
    // fortitudes less the debilities.
    let sun = &read.dignities.planets[3];
    let accidental = &read.planets[3];
    assert_eq!(
        read.net(Graha::Sun),
        Some(sun.score + sun.reception + accidental.fortitude - accidental.debility)
    );
}

#[test]
fn the_stars_are_read_in_the_charts_zodiac() {
    let tropical = context(Some(r#"{"frame": {"zodiac": "TROPICAL"}}"#));
    let sidereal = context(None);
    let request = FortitudeRequest::default();
    let one = tropical
        .chart()
        .fortitudes(&london(&tropical), &request)
        .unwrap();
    let other = sidereal
        .chart()
        .fortitudes(&london(&sidereal), &request)
        .unwrap();
    // Regulus at the end of Leo in 2000, tropically.
    assert!(
        (one.sky.regulus_deg - 149.86).abs() < 0.1,
        "{}",
        one.sky.regulus_deg
    );
    // Every star moves by the zodiac's offset, which the Sun shows.
    let offset = one.dignities.planets[3].longitude_deg - other.dignities.planets[3].longitude_deg;
    for (a, b) in [
        (one.sky.regulus_deg, other.sky.regulus_deg),
        (one.sky.spica_deg, other.sky.spica_deg),
        (one.sky.algol_deg, other.sky.algol_deg),
        (one.sky.north_node_deg, other.sky.north_node_deg),
    ] {
        assert!(((a - b).rem_euclid(360.0) - offset.rem_euclid(360.0)).abs() < 1e-6);
    }
}

#[test]
fn a_profile_names_the_division_the_fortitudes_count_houses_in() {
    let sdk = context(Some(
        r#"{"frame": {"zodiac": "TROPICAL"}, "houses": {"module_overrides": {"hellenistic": "PLACIDUS"}}}"#,
    ));
    let read = sdk
        .chart()
        .fortitudes(&london(&sdk), &FortitudeRequest::default())
        .unwrap();
    assert_eq!(read.sky.houses, HouseSystem::Placidus);
}

#[test]
fn valens_own_fortune_follows_the_moon_across_one_night() {
    // London, from 18:00 UTC on 1 January 2000 to 07:00 the next morning:
    // a waning crescent Moon, below the horizon in the evening and risen
    // before dawn, so the night holds both of III.11's cases.
    let tropical = context(Some(r#"{"frame": {"zodiac": "TROPICAL"}}"#));
    let sidereal = context(None);
    let valens = LotRequest::VALENS.with_fortune(FortuneRule::ReversedWhileMoonUp);
    let lilly = LotRequest::VALENS.with_fortune(FortuneRule::DayAndNight);
    let fortune = |sdk: &Context, natal: &Document, request| {
        sdk.chart()
            .lots_with_request(natal, &[Lot::Fortune], request)
            .unwrap()
    };
    let mut seen = [false; 2];
    for hour in 0..14 {
        let jd = 2_451_545.25 + f64::from(hour) / 24.0;
        let natal = chart(&tropical, 51.5, -0.12, jd);
        let read = fortune(&tropical, &natal, valens);
        assert_eq!(read.sect, Sect::Night, "hour {hour}");
        let moon_up = read.fortune_reversed;
        seen[usize::from(moon_up)] = true;
        // Reversed while the Moon is up, Lilly's once it has set.
        let rival = fortune(
            &tropical,
            &natal,
            if moon_up { LotRequest::VALENS } else { lilly },
        );
        assert_eq!(read.lots[0].place, rival.lots[0].place, "hour {hour}");
        // The horizon is the same in every zodiac.
        let other = chart(&sidereal, 51.5, -0.12, jd);
        assert_eq!(
            fortune(&sidereal, &other, valens).fortune_reversed,
            moon_up,
            "hour {hour}"
        );
    }
    assert_eq!(seen, [true, true], "the Moon both set and up in one night");
}

#[test]
fn a_lot_the_caller_writes_is_read_as_the_catalogues() {
    let sdk = context(None);
    let natal = london(&sdk);
    let read = sdk.chart().lots(&natal, &Lot::ALL).unwrap();
    assert_eq!(read.lots.len(), Lot::ALL.len());
    for placed in &read.lots {
        let own = sdk
            .chart()
            .lot_place(&natal, &placed.lot.formula(), LotRequest::VALENS)
            .unwrap();
        // By day no lot's arc depends on Fortune's rule.
        assert_eq!(own, placed.place, "{:?}", placed.lot);
    }
}

/// The considerations read the chart's own hour and Ascendant, and the
/// Moon's next perfection, read back in the chart cast at the moment it
/// promises, finds her at that aspect within 0.01° (0.007° measured).
/// Geocentric, as Lilly's ephemerides were: a topocentric Moon swings
/// with parallax, and the same projection missed by 2.3°.
#[test]
fn the_moons_next_aspect_is_where_the_later_chart_finds_her() {
    let sdk = context(Some(r#"{"frame": {"centre": "GEOCENTRIC"}}"#));
    let mut worst: f64 = 0.0;
    let mut perfections = 0;
    for hour in 0..48 {
        let jd = 2_451_545.0 + f64::from(hour) / 24.0;
        let figure = chart(&sdk, 51.5, -0.12, jd);
        let read = sdk
            .chart()
            .considerations(
                &figure,
                &FortitudeRequest::default(),
                ConsiderationRules::LILLY,
            )
            .unwrap();
        assert_eq!(
            read.radicality.hour_lord,
            figure.foundation.timing.hora.lord
        );
        let ascendant = sdk.chart().angles(&figure).unwrap().ascendant_deg;
        assert_eq!(read.ascendant.sign, Rashi::of_longitude(ascendant));
        let moon = figure.foundation.graha(Graha::Moon).unwrap().longitude_deg;
        assert_eq!(read.moon.sign, Rashi::of_longitude(moon));
        let Some(next) = read.moon.course.next else {
            continue;
        };
        perfections += 1;
        let later = chart(&sdk, 51.5, -0.12, jd + next.days);
        let at = |graha| later.foundation.graha(graha).unwrap().longitude_deg;
        let gap = at(Graha::Moon) - at(next.planet);
        let off = [next.aspect.degrees(), -next.aspect.degrees()]
            .into_iter()
            .map(|side| ((gap - side + 180.0).rem_euclid(360.0) - 180.0).abs())
            .fold(f64::INFINITY, f64::min);
        worst = worst.max(off);
    }
    assert!(perfections > 0);
    assert!(worst < 0.01, "worst {worst}");
}

/// How far two planets in a later chart stand from an aspect, degrees.
fn off_the_aspect(later: &Document, first: Graha, second: Graha, aspect: PtolemaicAspect) -> f64 {
    let at = |graha| later.foundation.graha(graha).unwrap().longitude_deg;
    let gap = at(first) - at(second);
    [aspect.degrees(), -aspect.degrees()]
        .into_iter()
        .map(|side| ((gap - side + 180.0).rem_euclid(360.0) - 180.0).abs())
        .fold(f64::INFINITY, f64::min)
}

/// Lilly's perfection read back elsewhere: every application and every
/// impediment's contact the search promises is where the chart cast at
/// that instant finds the two planets, and every refranation's station
/// is where the planet's motion turns.
#[test]
fn every_promised_contact_is_where_the_later_chart_finds_it() {
    let sdk = context(Some(r#"{"frame": {"centre": "GEOCENTRIC"}}"#));
    let pairs = [
        (Graha::Venus, Graha::Mars),
        (Graha::Mercury, Graha::Jupiter),
        (Graha::Moon, Graha::Saturn),
        (Graha::Sun, Graha::Mars),
    ];
    let (mut worst, mut applications, mut impediments) = (0.0_f64, 0, 0);
    for month in 0..12 {
        let jd = 2_451_545.0 + f64::from(month) * 61.0;
        let figure = chart(&sdk, 51.5, -0.12, jd);
        for (querent, quesited) in pairs {
            let matter = sdk
                .chart()
                .perfection(
                    &figure,
                    &FortitudeRequest::default(),
                    querent,
                    quesited,
                    PerfectionRules::LILLY,
                )
                .unwrap();
            if let Some(application) = matter.application {
                applications += 1;
                let later = chart(&sdk, 51.5, -0.12, jd + application.days);
                worst = worst.max(off_the_aspect(
                    &later,
                    querent,
                    quesited,
                    application.aspect,
                ));
            }
            for impediment in &matter.impediments {
                match (impediment.kind, impediment.third) {
                    (ImpedimentKind::Refranation, None) => {
                        let speed = |days: f64| {
                            chart(&sdk, 51.5, -0.12, jd + impediment.days + days)
                                .foundation
                                .graha(impediment.significator)
                                .unwrap()
                                .speed_deg_per_day
                        };
                        assert!(speed(-0.5) * speed(0.5) < 0.0, "{impediment:?}");
                    }
                    (_, Some(third)) => {
                        impediments += 1;
                        let later = chart(&sdk, 51.5, -0.12, jd + impediment.days);
                        let off = off_the_aspect(
                            &later,
                            impediment.significator,
                            third,
                            impediment.aspect,
                        );
                        worst = worst.max(off);
                    }
                    other => panic!("an impediment without its third: {other:?}"),
                }
            }
        }
    }
    assert!(
        applications > 10 && impediments > 10,
        "{applications} {impediments}"
    );
    assert!(worst < 1e-3, "worst {worst}");
}

#[test]
fn a_perfection_refuses_one_planet_for_both_significators() {
    let sdk = context(None);
    let figure = london(&sdk);
    let same = sdk
        .chart()
        .perfection(
            &figure,
            &FortitudeRequest::default(),
            Graha::Mars,
            Graha::Mars,
            PerfectionRules::LILLY,
        )
        .unwrap_err();
    assert_eq!(same.field(), Some("quesited"));
    let node = sdk
        .chart()
        .perfection(
            &figure,
            &FortitudeRequest::default(),
            Graha::Rahu,
            Graha::Mars,
            PerfectionRules::LILLY,
        )
        .unwrap_err();
    assert_eq!(node.field(), Some("querent"));
}
