//! The Sarvatobhadra chakra through the façade (`03-design/gochar.md` §8,
//! C428 to C432): the native's points read back from the natal document,
//! each transit read back from a chart founded at its instant, and the
//! five's motion as each source gives it, over a sky that happened.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]

use teistro::catalogue::Graha;
use teistro::gochar::Transit;
use teistro::gochar::sarvatobhadra::{
    Motion, NameSounds, SarvatobhadraRules, Sight, StarPlace, Vowel, sarvatobhadra,
};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{
    ChartRequest, Context, Document, Ephemeris, House, SarvatobhadraMotion, SarvatobhadraRequest,
    UtcOffset,
};

fn sdk() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

fn day(jd: f64) -> JulianDay<Utc> {
    JulianDay::<Utc>::try_new(jd).unwrap()
}

fn kathmandu() -> Place {
    Place::new(
        Latitude::literal(27.7),
        Longitude::literal(85.3),
        Altitude::literal(1400.0),
    )
}

fn natal(sdk: &Context) -> Document {
    let asked = ChartRequest::at(kathmandu(), UtcOffset::literal(5, 45, 0)).with_panchanga();
    sdk.chart()
        .reading(day(2_447_995.489_583_333_5), &asked)
        .unwrap()
        .value
}

/// Every fourth day over two years from 2025: Mercury's and Venus's
/// retrogrades, Saturn's and Jupiter's, all fall inside it.
fn days() -> Vec<JulianDay<Utc>> {
    (0..183)
        .map(|n| day(2_460_676.5 + 4.0 * f64::from(n)))
        .collect()
}

/// The transit charts at `days`, founded as the façade founds them: at
/// the natal place, in the natal chart's kind.
fn transits(sdk: &Context, natal: &Document, days: &[JulianDay<Utc>]) -> Vec<Document> {
    days.iter()
        .map(|at| {
            let asked = ChartRequest::at(natal.foundation.place, UtcOffset::UTC);
            sdk.chart().reading(*at, &asked).unwrap().value
        })
        .collect()
}

fn sign(longitude_deg: f64) -> teistro::catalogue::Rashi {
    Transit::at_longitude(longitude_deg).sign
}

#[test]
fn the_natives_points_are_the_natal_moons_and_the_births_almanacs() {
    let sdk = sdk();
    let natal = natal(&sdk);
    let read = sdk
        .chart()
        .sarvatobhadra(&natal, &SarvatobhadraRequest::at(day(2_461_000.5)))
        .unwrap()
        .value;
    let native = read[0].native;
    let moon = natal.foundation.graha(Graha::Moon).unwrap().longitude_deg;
    let almanac = natal.panchanga.as_ref().unwrap();
    assert_eq!(native.nakshatra, StarPlace::at(moon).nakshatra);
    assert_eq!(native.rashi, sign(moon));
    assert_eq!(
        native.tithi,
        almanac.tithi_at(natal.foundation.instant).unwrap().member
    );
    assert_eq!(native.vara, almanac.vara());
    assert_eq!(native.name, None);
}

#[test]
fn each_reading_is_the_kernels_over_a_chart_founded_at_its_instant() {
    let sdk = sdk();
    let natal = natal(&sdk);
    let days = days();
    let read = sdk
        .chart()
        .sarvatobhadra(&natal, &SarvatobhadraRequest::over(days.clone()))
        .unwrap()
        .value;
    assert_eq!(read.len(), days.len());
    for (reading, transit) in read.iter().zip(transits(&sdk, &natal, &days)) {
        let mut sky = [Transit::at_longitude(0.0); 9];
        for (slot, graha) in sky.iter_mut().zip(Graha::ALL) {
            *slot = Transit::at_longitude(transit.foundation.graha(graha).unwrap().longitude_deg);
        }
        let motions = [
            Graha::Mars,
            Graha::Mercury,
            Graha::Jupiter,
            Graha::Venus,
            Graha::Saturn,
        ]
        .map(|graha| reading.grahas[graha as usize].motion);
        let again =
            sarvatobhadra(&sky, &motions, &reading.native, SarvatobhadraRules::TEXT).unwrap();
        assert_eq!(*reading, again);
    }
}

#[test]
fn the_skys_motion_is_the_speed_against_the_siddhantas_mean() {
    let sdk = sdk();
    let natal = natal(&sdk);
    let days = days();
    let read = sdk
        .chart()
        .sarvatobhadra(&natal, &SarvatobhadraRequest::over(days.clone()))
        .unwrap()
        .value;
    // The Surya Siddhanta's mean motions (I.29 to 34 over I.37), Mercury's and
    // Venus's the Sun's.
    let mean = |graha: Graha| match graha {
        Graha::Mars => 0.524,
        Graha::Jupiter => 0.0831,
        Graha::Saturn => 0.0334,
        _ => 0.9856,
    };
    let mut seen = Vec::new();
    for (reading, transit) in read.iter().zip(transits(&sdk, &natal, &days)) {
        for graha in [
            Graha::Mars,
            Graha::Mercury,
            Graha::Jupiter,
            Graha::Venus,
            Graha::Saturn,
        ] {
            let speed = transit.foundation.graha(graha).unwrap().speed_deg_per_day;
            let motion = reading.grahas[graha as usize].motion;
            // Away from the line, where the mean's fourth digit cannot
            // decide it.
            if (speed - mean(graha)).abs() > 1e-3 {
                assert_eq!(motion, Motion::by_speed(speed, mean(graha)), "{graha:?}");
            }
            seen.push((graha, motion));
        }
        // v. 19: the luminaries fast, the nodes retrograde, whatever the
        // source.
        assert_eq!(reading.grahas[Graha::Sun as usize].motion, Motion::Fast);
        assert_eq!(
            reading.grahas[Graha::Ketu as usize].motion,
            Motion::Retrograde
        );
    }
    // Two years of sky: every one of the five goes retrograde, and Mercury
    // and Venus are seen in all three motions.
    for graha in [
        Graha::Mars,
        Graha::Mercury,
        Graha::Jupiter,
        Graha::Venus,
        Graha::Saturn,
    ] {
        assert!(seen.contains(&(graha, Motion::Retrograde)), "{graha:?}");
    }
    for graha in [Graha::Mercury, Graha::Venus] {
        assert!(seen.contains(&(graha, Motion::Fast)), "{graha:?}");
        assert!(seen.contains(&(graha, Motion::Mean)), "{graha:?}");
    }
}

#[test]
fn the_suns_motion_is_vv_16_to_18s_and_the_skys_where_they_are_silent() {
    let sdk = sdk();
    let natal = natal(&sdk);
    let days = days();
    let asked = SarvatobhadraRequest::over(days.clone()).with_motion(SarvatobhadraMotion::Sun);
    let by_sun = sdk.chart().sarvatobhadra(&natal, &asked).unwrap().value;
    let by_sky = sdk
        .chart()
        .sarvatobhadra(&natal, &SarvatobhadraRequest::over(days.clone()))
        .unwrap()
        .value;
    let mut unseen = 0;
    for ((sun, sky), transit) in by_sun
        .iter()
        .zip(&by_sky)
        .zip(transits(&sdk, &natal, &days))
    {
        let sun_sign = sign(transit.foundation.graha(Graha::Sun).unwrap().longitude_deg);
        for graha in [
            Graha::Mars,
            Graha::Mercury,
            Graha::Jupiter,
            Graha::Venus,
            Graha::Saturn,
        ] {
            let own = sign(transit.foundation.graha(graha).unwrap().longitude_deg);
            let motion = sun.grahas[graha as usize].motion;
            if let Some(text) = Motion::by_the_sun(House::between(own, sun_sign).get()) {
                assert_eq!(motion, text, "{graha:?}");
                // §8.2: where the verses speak, Mercury and Venus are
                // never retrograde.
                if matches!(graha, Graha::Mercury | Graha::Venus) {
                    assert_ne!(motion, Motion::Retrograde, "{graha:?}");
                }
            } else {
                // With the Sun, unseen (v. 18): the sky's, which is
                // where an inferior planet's retrograde falls.
                unseen += 1;
                assert_eq!(motion, sky.grahas[graha as usize].motion, "{graha:?}");
            }
        }
    }
    assert!(unseen > 0, "two years with no graha in the Sun's sign");
}

#[test]
fn given_motions_and_rules_and_a_name_are_read_as_asked() {
    let sdk = sdk();
    let natal = natal(&sdk);
    let given = [
        Motion::Retrograde,
        Motion::Mean,
        Motion::Fast,
        Motion::Mean,
        Motion::Retrograde,
    ];
    let rules = SarvatobhadraRules {
        sight: Sight::AllThree,
        ..SarvatobhadraRules::TEXT
    };
    let name = NameSounds {
        letter: 'र',
        vowel: Vowel::A,
    };
    let asked = SarvatobhadraRequest::at(day(2_461_000.5))
        .with_motion(SarvatobhadraMotion::Given(given))
        .with_rules(rules)
        .named(name);
    let read = &sdk.chart().sarvatobhadra(&natal, &asked).unwrap().value[0];
    for (graha, motion) in [
        Graha::Mars,
        Graha::Mercury,
        Graha::Jupiter,
        Graha::Venus,
        Graha::Saturn,
    ]
    .into_iter()
    .zip(given)
    {
        assert_eq!(read.grahas[graha as usize].motion, motion);
        // Every graha along all three lines, its motion notwithstanding.
        assert_eq!(read.grahas[graha as usize].lines.len(), 3);
    }
    assert_eq!(read.rules, rules);
    assert_eq!(read.native.name, Some(name));
}

#[test]
fn a_refusal_names_the_field() {
    let sdk = sdk();
    let natal = natal(&sdk);
    let today = SarvatobhadraRequest::at(day(2_461_000.5));
    let none = sdk
        .chart()
        .sarvatobhadra(&natal, &SarvatobhadraRequest::over([]))
        .unwrap_err();
    assert_eq!(none.field(), Some("instants"));
    // A document without its panchanga has no tithi or weekday of birth.
    let bare = sdk
        .chart()
        .reading(
            natal.foundation.instant,
            &ChartRequest::at(kathmandu(), UtcOffset::literal(5, 45, 0)),
        )
        .unwrap()
        .value;
    let unread = sdk.chart().sarvatobhadra(&bare, &today).unwrap_err();
    assert_eq!(unread.field(), Some("panchanga"));
    let latin = today.named(NameSounds {
        letter: 'r',
        vowel: Vowel::A,
    });
    let letter = sdk.chart().sarvatobhadra(&natal, &latin).unwrap_err();
    assert_eq!(letter.field(), Some("native.name.letter"));
    // The record a binding writes, refused by its own names.
    let typo = SarvatobhadraRequest::from_json(r#"{"instants": [2460676.5], "motions": "SUN"}"#)
        .unwrap_err();
    assert_eq!(typo.field(), Some("sarvatobhadra.motions"));
    let given = SarvatobhadraRequest::from_json(
        r#"{"instants": [2460676.5], "motion": {"GIVEN": ["MEAN", "MEAN", "FAST", "MEAN", "RETROGRADE"]}, "name": {"letter": "र", "vowel": "A"}}"#,
    )
    .unwrap();
    assert_eq!(
        given.motion(),
        SarvatobhadraMotion::Given([
            Motion::Mean,
            Motion::Mean,
            Motion::Fast,
            Motion::Mean,
            Motion::Retrograde
        ])
    );
}
