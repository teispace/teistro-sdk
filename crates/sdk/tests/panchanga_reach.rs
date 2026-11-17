//! What reaches past the muhurta family (`03-design/panchanga-reach.md`):
//! the panchanga's own longitudes at instants (SR-137), the season's
//! blackouts by profile (SR-138), and an eclipse's vedha in every build
//! (SR-132), each read back against the almanac it must agree with.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    reason = "tests fail by panicking, index what they asked for and compare bits"
)]

use teistro::catalogue::{BlackoutKind, Calendar};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{CalendarDate, Context, Ephemeris, MonthKind, Panchanga, Status, UtcOffset};

/// A context on a profile, with a settings patch laid over it; the
/// committee's sky is the text's, opened over its own provider.
fn context(profile: &str, patch: Option<&str>) -> Context {
    let sky = if profile == "nepali-committee" {
        Ephemeris::SuryaSiddhanta
    } else {
        Ephemeris::Builtin
    };
    let builder = Context::builder().profile(profile).ephemeris([sky]);
    match patch {
        Some(json) => builder.settings_json(json),
        None => builder,
    }
    .build()
    .unwrap()
}

fn kathmandu() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    )
}

const NEPAL: UtcOffset = UtcOffset::literal(5, 45, 0);

fn date(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, year, month, day)
}

/// The search finds a crossing to a ten-millionth of a day
/// (`teistro_astro::events::TOLERANCE_DAYS`); the Moon less or plus the
/// Sun moves under sixteen degrees a day, so an instant the search found
/// stands within this of its line.
const ON_THE_GRID_DEG: f64 = 1e-7 * 16.0;

/// How far an angle stands from the nearest line of a grid of `step`.
fn off_grid(deg: f64, step: f64) -> f64 {
    let k = (deg / step).round();
    (deg - k * step).abs().min((deg - k * step - 360.0).abs())
}

/// Every limb end the days hold, with the grid its angle sits on and which
/// angle that is: the tithis' and karanas' ends on the elongation, the
/// nakshatras' on the Moon, the yogas' on the sum.
fn ends(days: &[Panchanga]) -> Vec<(JulianDay<Utc>, f64, &'static str)> {
    let mut found = Vec::new();
    for day in days {
        for span in &day.limbs.tithi {
            found.push((span.whole.to, 12.0, "elongation"));
        }
        for span in &day.limbs.karana {
            found.push((span.whole.to, 6.0, "elongation"));
        }
        for span in &day.limbs.nakshatra {
            found.push((span.whole.to, 360.0 / 27.0, "moon"));
        }
        for span in &day.limbs.yoga {
            found.push((span.whole.to, 360.0 / 27.0, "yoga"));
        }
    }
    found
}

/// At every limb end a month's days found, the angles sit on the limb's
/// grid: the worst distance, degrees.
fn worst_off_grid(sdk: &Context, from: &CalendarDate, to: &CalendarDate) -> f64 {
    let days = sdk
        .almanac()
        .of(from, to, &kathmandu(), NEPAL)
        .unwrap()
        .value;
    let ends = ends(&days);
    let instants: Vec<JulianDay<Utc>> = ends.iter().map(|(at, _, _)| *at).collect();
    let angles = sdk
        .almanac()
        .limb_angles(&instants, &kathmandu())
        .unwrap()
        .value;
    assert_eq!(angles.len(), ends.len());
    ends.iter()
        .zip(&angles)
        .map(|((at, step, which), read)| {
            assert_eq!(read.at, *at);
            let deg = match *which {
                "elongation" => read.elongation_deg,
                "moon" => read.moon_deg,
                _ => read.yoga_deg,
            };
            off_grid(deg, *step)
        })
        .fold(0.0, f64::max)
}

#[test]
fn every_limb_end_the_search_found_sits_on_the_grid_under_drik_and_the_committee() {
    for profile in ["nepali-drik", "nepali-committee"] {
        let sdk = context(profile, None);
        for (from, to) in [
            (date(2026, 11, 1), date(2026, 11, 30)),
            (date(1900, 3, 1), date(1900, 3, 30)),
        ] {
            let worst = worst_off_grid(&sdk, &from, &to);
            assert!(
                worst <= ON_THE_GRID_DEG,
                "{profile} from {from}: {worst}° off the grid"
            );
        }
    }
}

#[test]
fn every_knob_the_limbs_read_the_angles_read_too() {
    // Each knob the brief names, the one the old road through `positions`
    // could not express among them: a true basis, true places, a
    // topocentric panchanga and a custom ayanamsha.
    for patch in [
        r#"{"frame": {"ayanamsha_basis": "MEAN"}}"#,
        r#"{"frame": {"ayanamsha_basis": "TRUE"}}"#,
        r#"{"frame": {"positions": "TRUE"}}"#,
        r#"{"panchanga": {"centre": "TOPOCENTRIC"}}"#,
        r#"{"frame": {"ayanamsha": {"kind": "CUSTOM", "epoch_jd_tt": 2451545.0, "value_deg": 23.85, "rate_deg_per_year": 0.013969}}}"#,
    ] {
        let sdk = context("nepali-drik", Some(patch));
        let worst = worst_off_grid(&sdk, &date(2026, 6, 1), &date(2026, 6, 15));
        assert!(worst <= ON_THE_GRID_DEG, "{patch}: {worst}° off the grid");
    }
}

#[test]
fn the_angles_name_the_limb_the_day_holds() {
    let sdk = context("nepali-drik", None);
    let day = sdk
        .almanac()
        .of(
            &date(2026, 11, 25),
            &date(2026, 11, 25),
            &kathmandu(),
            NEPAL,
        )
        .unwrap()
        .value
        .remove(0);
    let tithi = &day.limbs.tithi[0];
    let yoga = &day.limbs.yoga[0];
    let star = &day.limbs.nakshatra[0];
    // The middle of each span, where the member is held whole.
    let middle = |from: JulianDay<Utc>, to: JulianDay<Utc>| {
        JulianDay::<Utc>::literal(f64::midpoint(from.get(), to.get()))
    };
    let angles = sdk
        .almanac()
        .limb_angles(
            &[
                middle(tithi.whole.from, tithi.whole.to),
                middle(yoga.whole.from, yoga.whole.to),
                middle(star.whole.from, star.whole.to),
            ],
            &kathmandu(),
        )
        .unwrap();
    assert_eq!(angles.value[0].tithi(), tithi.member);
    assert_eq!(angles.value[1].yoga(), yoga.member);
    assert_eq!(angles.value[2].nakshatra(), star.member);
    // The answer is sealed over its value.
    assert_eq!(
        angles.provenance.content_hash,
        teistro::content_hash(&angles.value)
    );
}

#[test]
fn the_angles_refuse_no_instants_and_a_context_with_no_sky() {
    let sdk = context("nepali-drik", None);
    let none = sdk.almanac().limb_angles(&[], &kathmandu()).unwrap_err();
    assert_eq!(
        (none.status, none.field()),
        (Status::InvalidArg, Some("instants"))
    );
    let skyless = Context::builder()
        .ephemeris([Ephemeris::None])
        .build()
        .unwrap();
    let at = JulianDay::<Utc>::literal(2_461_345.25);
    assert_eq!(
        skyless
            .almanac()
            .limb_angles(&[at], &kathmandu())
            .unwrap_err()
            .status,
        Status::Capability
    );
}

/// The intervals of a kind, as raw instants.
fn spans_of(found: &[teistro::Blackout], kind: BlackoutKind) -> Vec<(f64, f64)> {
    found
        .iter()
        .filter(|b| b.kind == kind)
        .map(|b| (b.at.from.get(), b.at.to.get()))
        .collect()
}

#[test]
fn the_malmas_is_the_run_of_days_the_almanac_marks_adhika() {
    for profile in ["nepali-drik", "nepali-committee"] {
        let sdk = context(profile, None);
        let (first, last) = (date(2026, 4, 1), date(2026, 7, 31));
        let season = sdk
            .almanac()
            .blackouts(&first, &last, NEPAL, &[BlackoutKind::AdhikaMasa])
            .unwrap()
            .value;
        // 2026's adhika Jyeshtha, one span.
        let malmas = spans_of(&season, BlackoutKind::AdhikaMasa);
        assert_eq!(malmas.len(), 1, "{profile}: {malmas:?}");
        let (from, to) = malmas[0];
        // Read back from the almanac: a day is adhika exactly when its
        // sunrise falls inside the span.
        let days = sdk
            .almanac()
            .of(&first, &last, &kathmandu(), NEPAL)
            .unwrap()
            .value;
        for day in &days {
            let sunrise = day.day.sunrise.get();
            let inside = from <= sunrise && sunrise < to;
            assert_eq!(
                day.month.kind == MonthKind::Adhika,
                inside,
                "{profile} {}",
                day.day.date
            );
        }
    }
}

#[test]
fn kharmas_is_the_sun_in_dhanu_and_meena_and_chaturmas_spans_its_eleventh_to_eleventh() {
    let sdk = context("nepali-drik", None);
    let (first, last) = (date(2026, 1, 1), date(2026, 12, 31));
    let year = sdk.almanac().blackouts(&first, &last, NEPAL, &[]).unwrap();
    let season = &year.value;
    // Every kind found is one the Sun and the Moon make, in order.
    assert!(season.iter().all(|b| teistro::SKY_KINDS.contains(&b.kind)));
    assert!(
        season
            .windows(2)
            .all(|pair| pair[0].at.from.get() <= pair[1].at.from.get())
    );
    // Kharmas: the Sun's Dhanu clipped at the year's start, its Meena, and
    // its Dhanu again in December.
    let kharmas = spans_of(season, BlackoutKind::Kharmas);
    assert_eq!(kharmas.len(), 3, "{kharmas:?}");
    // Chaturmas: Ashadha's bright eleventh to Kartika's, about four months.
    let chaturmas = spans_of(season, BlackoutKind::Chaturmas);
    assert_eq!(chaturmas.len(), 1);
    let (from, to) = chaturmas[0];
    assert!((115.0..150.0).contains(&(to - from)), "{chaturmas:?}");
    // Each end is where the day holding it opens or runs in the eleventh.
    let days = sdk
        .almanac()
        .of(&first, &last, &kathmandu(), NEPAL)
        .unwrap()
        .value;
    for at in [from, to] {
        let holding = days
            .iter()
            .flat_map(|day| &day.limbs.tithi)
            .find(|span| span.whole.from.get() <= at && at < span.whole.to.get())
            .unwrap();
        assert_eq!(holding.member, teistro::catalogue::Tithi::ShuklaEkadashi);
        assert!((holding.whole.from.get() - at).abs() < 1e-6);
    }
    // One kind asked, one kind answered; the same spans.
    let only = sdk
        .almanac()
        .blackouts(&first, &last, NEPAL, &[BlackoutKind::Chaturmas])
        .unwrap()
        .value;
    assert_eq!(spans_of(&only, BlackoutKind::Chaturmas), chaturmas);
    assert_eq!(only.len(), 1);
}

#[test]
fn a_kind_the_sun_and_the_moon_do_not_make_is_refused() {
    let sdk = context("nepali-drik", None);
    let asta = sdk
        .almanac()
        .blackouts(
            &date(2026, 1, 1),
            &date(2026, 1, 31),
            NEPAL,
            &[BlackoutKind::ShukraAsta],
        )
        .unwrap_err();
    assert_eq!(
        (asta.status, asta.field()),
        (Status::InvalidArg, Some("kinds"))
    );
    let reversed = sdk
        .almanac()
        .blackouts(&date(2026, 2, 1), &date(2026, 1, 1), NEPAL, &[])
        .unwrap_err();
    assert_eq!(reversed.status, Status::InvalidArg);
}

#[test]
fn the_blackouts_record_writes_each_kind_in_full_and_seals_a_zone_apart() {
    let sdk = context("nepali-drik", None);
    let record = teistro::records::record("almanac.blackouts").unwrap();
    let by_offset = record
        .answer(
            &sdk,
            r#"{"first": {"year": 2026, "month": 12, "day": 1}, "last": {"year": 2026, "month": 12, "day": 31},
                "utcOffsetSeconds": 20700, "kinds": ["KHARMAS"]}"#,
        )
        .unwrap();
    assert_eq!(by_offset.value[0]["kind"], "blackout_kind.KHARMAS");
    let by_zone = record
        .answer(
            &sdk,
            r#"{"first": {"year": 2026, "month": 12, "day": 1}, "last": {"year": 2026, "month": 12, "day": 31},
                "zone": {"kind": "IANA", "zone": "Asia/Kathmandu"}, "kinds": ["KHARMAS"]}"#,
        )
        .unwrap();
    assert_eq!(by_zone.value, by_offset.value);
    let hash =
        |answered: &teistro::records::Answered| answered.provenance.as_ref().unwrap().input_hash;
    assert_ne!(hash(&by_zone), hash(&by_offset));
}

#[test]
fn the_limb_angles_record_answers_each_instant_in_order() {
    let sdk = context("nepali-drik", None);
    let record = teistro::records::record("almanac.limbAngles").unwrap();
    let answered = record
        .answer(
            &sdk,
            r#"{"instants": [2461345.25, 2461345.75], "latitudeDeg": 27.7172, "longitudeDeg": 85.324}"#,
        )
        .unwrap();
    let list = answered.value.as_array().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[1]["at"], 2_461_345.75);
}

#[test]
fn every_seen_eclipse_carries_its_vedha_in_every_build() {
    // The vedha is the almanac's (SR-132): a build without the muhurta
    // family answers it as the full build does.
    let sdk = context("nepali-drik", None);
    let year = sdk
        .almanac()
        .eclipses(&date(2025, 1, 1), &date(2025, 12, 31), &kathmandu(), NEPAL)
        .unwrap()
        .value;
    let seen: Vec<_> = year
        .lunar
        .iter()
        .filter(|lunar| lunar.here.umbral_seen.is_some())
        .collect();
    assert_ne!(seen.len(), 0);
    for lunar in seen {
        let vedha = lunar.vedha.unwrap();
        let first = lunar.here.umbral_seen.unwrap().from.get();
        // Three or four praharas before the first moment seen.
        assert!(vedha.from.get() < first && first - vedha.from.get() < 1.0);
    }
}
