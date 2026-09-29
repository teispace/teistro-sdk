//! Gochar through the façade: the reference is the natal chart's, each
//! house is the transit counted from it, the batch is the calls one by
//! one, and the settings reach the judgement (`docs/03-design/gochar.md`).

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "tests fail by panicking and index what they found"
)]

use teistro::catalogue::{Graha, Rashi};
use teistro::gochar::{GRAHAS, GocharRules, Transit};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::settings::NodeObstruction;
use teistro::{ChartRequest, Context, Document, Ephemeris, GocharFrom, GocharRequest, UtcOffset};

/// A Kathmandu birth, 14 April 1990, the corpus's first.
const BIRTH: f64 = 2_447_995.489_583_333_5;

fn context(patch: &str) -> Context {
    Context::builder()
        .settings_json(patch)
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

fn place() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    )
}

fn natal(sdk: &Context) -> Document {
    sdk.chart()
        .reading(
            JulianDay::<Utc>::literal(BIRTH),
            &ChartRequest::at(place(), UtcOffset::literal(5, 45, 0)),
        )
        .unwrap()
        .value
}

fn sign_of(longitude_deg: f64) -> Rashi {
    Rashi::ALL[(longitude_deg.rem_euclid(360.0) / 30.0) as usize % 12]
}

#[test]
fn every_house_is_the_transit_counted_from_the_natal_reference() {
    let sdk = context("{}");
    let natal = natal(&sdk);
    let moon = sign_of(natal.foundation.graha(Graha::Moon).unwrap().longitude_deg);
    let lagna = Rashi::ALL[usize::from(natal.foundation.lagna_sign_index())];
    let instants: Vec<_> = (0..24)
        .map(|k| JulianDay::<Utc>::literal(2_460_676.5 + 15.25 * f64::from(k)))
        .collect();
    for (from, reference) in [(GocharFrom::Moon, moon), (GocharFrom::Lagna, lagna)] {
        let batch = sdk
            .chart()
            .gochar(
                &natal,
                &GocharRequest::over(instants.clone()).counted_from(from),
            )
            .unwrap()
            .value;
        assert_eq!(batch.len(), instants.len());
        for (reading, at) in batch.iter().zip(&instants) {
            assert_eq!(reading.reference.sign, reference, "{from:?}");
            assert_eq!(reading.reference.from, from);
            // The transit chart founded on its own, and each graha counted
            // from the reference by hand.
            let transit = sdk
                .chart()
                .reading(*at, &ChartRequest::at(place(), UtcOffset::UTC))
                .unwrap()
                .value;
            for (read, graha) in reading.grahas.iter().zip(GRAHAS) {
                let sign = sign_of(transit.foundation.graha(graha).unwrap().longitude_deg);
                assert_eq!(read.transit.sign, sign, "{graha:?}");
                let house = (sign as usize + 12 - reference as usize) % 12 + 1;
                assert_eq!(usize::from(read.house), house, "{graha:?}");
            }
            // The batch is the calls one by one.
            let single = sdk
                .chart()
                .gochar(&natal, &GocharRequest::at(*at).counted_from(from))
                .unwrap()
                .value;
            assert_eq!(single, vec![reading.clone()]);
        }
    }
}

#[test]
fn the_settings_reach_the_judgement() {
    let text = context("{}");
    let natal = natal(&text);
    let at = GocharRequest::at(JulianDay::<Utc>::literal(2_460_676.5));
    let reading = &text.chart().gochar(&natal, &at).unwrap().value[0];
    assert_eq!(reading.rules, GocharRules::TEXT);
    let seven = context(r#"{"gochar": {"node_obstruction": "NONE"}}"#);
    let reading = &seven.chart().gochar(&natal, &at).unwrap().value[0];
    assert_eq!(reading.rules.node_obstruction, NodeObstruction::None);
    // Under it no node obstructs anyone.
    for read in &reading.grahas {
        assert!(
            read.obstructed_by
                .iter()
                .all(|by| !matches!(by, Graha::Rahu | Graha::Ketu)),
            "{read:?}"
        );
    }
}

/// On the real sky the nodes stand opposite, in each other's vedha house
/// whenever one is in a good house: the text's reading spares them each
/// other (C140), and the literal one leaves them never good.
#[test]
fn a_node_in_a_good_house_is_spared_the_other_node() {
    let text = context("{}");
    let literal = context(r#"{"gochar": {"node_obstruction": "EACH_OTHER_TOO"}}"#);
    let natal = natal(&text);
    let request = GocharRequest::over(
        (0_u32..36).map(|month| JulianDay::<Utc>::literal(2_460_676.5 + 30.0 * f64::from(month))),
    );
    let spared = text.chart().gochar(&natal, &request).unwrap().value;
    let read = literal.chart().gochar(&natal, &request).unwrap().value;
    let mut good = 0;
    for (spared, read) in spared.iter().zip(&read) {
        for (node, other) in [(Graha::Rahu, Graha::Ketu), (Graha::Ketu, Graha::Rahu)] {
            let (spared, read) = (&spared.grahas[node as usize], &read.grahas[node as usize]);
            assert!(!spared.obstructed_by.contains(&other), "{spared:?}");
            if read.good_house {
                good += 1;
                assert!(read.obstructed_by.contains(&other), "{read:?}");
            }
        }
    }
    assert!(good > 0, "no node stood in a good house over three years");
}

/// The batch places the grahas without founding the transit charts, and
/// each place must be the founded chart's to the bit: under the default,
/// and under settings that move every step it shares — the conformance
/// profile's topocentric centre and nutated ayanamsha, the true node.
#[test]
fn every_transit_is_the_founded_chart_s_graha_to_the_bit() {
    let instants: Vec<_> = (0_u32..12)
        .map(|month| JulianDay::<Utc>::literal(2_451_545.0 + 30.4375 * f64::from(month)))
        .collect();
    let conformance = Context::builder()
        .profile("conformance-baseline")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap();
    for sdk in [
        context("{}"),
        conformance,
        context(r#"{"frame": {"node": "TRUE"}}"#),
    ] {
        let natal = natal(&sdk);
        let read = sdk
            .chart()
            .gochar(&natal, &GocharRequest::over(instants.clone()))
            .unwrap()
            .value;
        let founded = sdk
            .chart()
            .found_many(
                &instants,
                &natal.foundation.place,
                UtcOffset::UTC,
                teistro::catalogue::ChartKind::Natal,
            )
            .unwrap()
            .value;
        for (reading, chart) in read.iter().zip(&founded) {
            for (graha, read) in GRAHAS.into_iter().zip(&reading.grahas) {
                let place = chart.graha(graha).unwrap().longitude_deg;
                assert_eq!(read.transit, Transit::at_longitude(place), "{graha:?}");
            }
        }
    }
}

/// Asked for, each reading judges the seven by the natal Ashtakavarga: the
/// bindus of the sign transited are the natal section's, and so is its
/// sarvashtakavarga; not asked for, a reading carries none; and the
/// settings' threshold is the one applied (`gochar-ashtakavarga.md`).
#[test]
fn the_ashtakavarga_reading_is_the_natal_bindus_of_the_sign_transited() {
    use teistro::gochar::ashtakavarga::{Kakshya, SarvaStanding};

    let sdk = context("{}");
    let natal = sdk
        .chart()
        .reading(
            JulianDay::<Utc>::literal(BIRTH),
            &ChartRequest::at(place(), UtcOffset::literal(5, 45, 0)).with_ashtakavarga(),
        )
        .unwrap()
        .value;
    let av = natal.ashtakavarga.as_ref().unwrap();
    let instants =
        (0_u32..24).map(|k| JulianDay::<Utc>::literal(2_460_676.5 + 30.0 * f64::from(k)));
    let plain = GocharRequest::over(instants);
    let asked = plain.clone().with_ashtakavarga();
    assert!(
        sdk.chart()
            .gochar(&natal, &plain)
            .unwrap()
            .value
            .iter()
            .all(|reading| reading.ashtakavarga.is_none())
    );
    let four = context(r#"{"gochar": {"ashtakavarga_good_from": "FOUR"}}"#);
    let by_four = four.chart().gochar(&natal, &asked).unwrap().value;
    let mut fours = 0;
    for (reading, other) in sdk
        .chart()
        .gochar(&natal, &asked)
        .unwrap()
        .value
        .iter()
        .zip(&by_four)
    {
        let seven = reading.ashtakavarga.unwrap();
        for (read, (moving, four)) in seven
            .iter()
            .zip(reading.grahas.iter().zip(other.ashtakavarga.unwrap()))
        {
            let sign = moving.transit.sign as usize;
            let own = av
                .grahas
                .iter()
                .find(|row| row.graha == read.graha)
                .unwrap();
            assert_eq!(read.graha, moving.graha);
            assert_eq!(read.bindus, own.bindus[sign], "{:?}", read.graha);
            assert_eq!(read.sarva, av.sarva[sign]);
            assert_eq!(read.sarva_standing, SarvaStanding::of(read.sarva));
            assert_eq!(read.kakshya, Kakshya::at(moving.transit.degrees));
            assert_eq!(read.good, read.bindus >= 5);
            assert_eq!(four.good, read.bindus >= 4);
            fours += usize::from(read.bindus == 4);
        }
    }
    assert!(
        fours > 0,
        "no transit stood on four bindus, so the knob went untested"
    );
}

/// Where Jupiter gives the Moon a bindu is one knob for the chart and its
/// transits (C144): under the 2nd, the natal Moon's row loses the bindu
/// the 12th from Jupiter held and gains one in the 2nd, two signs on,
/// nothing else in any row moves, the reading says which it was counted
/// under, and a transit reads the same row the chart does.
#[test]
fn where_jupiter_gives_the_moon_a_bindu_is_one_knob_for_the_chart_and_its_transits() {
    use teistro::settings::MoonBinduFromJupiter;

    let request = ChartRequest::at(place(), UtcOffset::literal(5, 45, 0)).with_ashtakavarga();
    let natal = |sdk: &Context| {
        sdk.chart()
            .reading(JulianDay::<Utc>::literal(BIRTH), &request)
            .unwrap()
            .value
    };
    let (bphs, phaladeepika) = (
        context("{}"),
        context(r#"{"strength": {"moon_bindu_from_jupiter": "SECOND"}}"#),
    );
    let (twelfth, second) = (natal(&bphs), natal(&phaladeepika));
    let (a, b) = (
        twelfth.ashtakavarga.as_ref().unwrap(),
        second.ashtakavarga.as_ref().unwrap(),
    );
    assert_eq!(
        (
            a.rules.moon_bindu_from_jupiter,
            b.rules.moon_bindu_from_jupiter
        ),
        (MoonBinduFromJupiter::Twelfth, MoonBinduFromJupiter::Second)
    );
    assert_eq!(a.sarva.iter().sum::<u16>(), b.sarva.iter().sum::<u16>());
    for (x, y) in a.grahas.iter().zip(&b.grahas) {
        let moved: Vec<usize> = (0..12).filter(|s| x.bindus[*s] != y.bindus[*s]).collect();
        if x.graha != Graha::Moon {
            assert!(moved.is_empty(), "{:?} moved", x.graha);
            continue;
        }
        assert_eq!(moved.len(), 2, "the Moon's row moved at {moved:?}");
        let (lost, gained) = (moved[0], moved[1]);
        let (lost, gained) = if x.bindus[lost] > y.bindus[lost] {
            (lost, gained)
        } else {
            (gained, lost)
        };
        assert_eq!(x.bindus[lost], y.bindus[lost] + 1);
        assert_eq!(y.bindus[gained], x.bindus[gained] + 1);
        assert_eq!(
            (gained + 12 - lost) % 12,
            2,
            "the 12th and the 2nd from one sign"
        );
    }
    let asked = GocharRequest::over([JulianDay::<Utc>::literal(2_460_676.5)]).with_ashtakavarga();
    let moon_row = b
        .grahas
        .iter()
        .find(|row| row.graha == Graha::Moon)
        .unwrap();
    for reading in phaladeepika.chart().gochar(&second, &asked).unwrap().value {
        let (read, moving) = reading
            .ashtakavarga
            .unwrap()
            .into_iter()
            .zip(&reading.grahas)
            .find(|(read, _)| read.graha == Graha::Moon)
            .unwrap();
        assert_eq!(read.bindus, moon_row.bindus[moving.transit.sign as usize]);
    }
}
