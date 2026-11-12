//! The sunrise behind the terrain (SR-91, `03-design/terrain-horizon.md`)
//! through `almanac.days`, as a consumer asks it: a 2° skyline at the
//! rise's azimuth delays a real sunrise by the amount the Sun's own climb
//! and the refraction's fall give, derived here from a low-precision Sun
//! that shares nothing with the SDK; a level skyline of zero is the level
//! horizon's sunrise; the days themselves do not move; and every refusal
//! names its field.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    clippy::float_cmp,
    reason = "tests fail by panicking, index what they asked for and print the measurement under --nocapture"
)]

use serde_json::{Value, json};
use teistro::records::Answered;
use teistro::{Context, Ephemeris};

/// Kathmandu, where Tithipatra's days are reckoned.
const LATITUDE_DEG: f64 = 27.7172;
const LONGITUDE_DEG: f64 = 85.324;

/// How far the measured delay may stand from the derived one, seconds.
/// The derivation's Sun is the *Astronomical Almanac*'s low-precision one
/// (0.01° in declination), and its hour-angle rate a day's mean; each
/// moves a ten-minute delay by well under a tenth of a second, and the
/// three days measured stand 0.016 to 0.020 s apart.
const DELAY_TOLERANCE_S: f64 = 0.1;

/// The level horizon's sunrise and the skyline's, to the solver's
/// tolerance each (`rise_set::TOLERANCE_DAYS`, 1e-7 d) from either side.
const SAME_RISE_S: f64 = 0.02;

fn context() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

fn days(sdk: &Context, profile: Option<Value>) -> Result<Answered, teistro::Error> {
    let mut request = json!({
        "first": {"year": 2026, "month": 11, "day": 12},
        "last": {"year": 2026, "month": 11, "day": 14},
        "latitudeDeg": LATITUDE_DEG,
        "longitudeDeg": LONGITUDE_DEG,
        "altitudeM": 1400,
        "utcOffsetSeconds": 20_700,
    });
    if let Some(profile) = profile {
        request["horizonProfile"] = profile;
    }
    teistro::records::record("almanac.days")
        .unwrap()
        .answer(sdk, &request.to_string())
}

fn instant(value: &Value) -> f64 {
    value.as_f64().expect("an instant")
}

/// The *Astronomical Almanac*'s low-precision Sun (section C, "Low
/// precision formulas for the Sun", good to 0.01° from 1950 to 2050):
/// the declination, degrees, and the distance, au, at a Julian day.
fn low_precision_sun(jd: f64) -> (f64, f64, f64) {
    let n = jd - 2_451_545.0;
    let mean = (280.460 + 0.985_647_4 * n).to_radians();
    let anomaly = (357.528 + 0.985_600_3 * n).to_radians();
    let longitude = mean + (1.915 * anomaly.sin() + 0.020 * (2.0 * anomaly).sin()).to_radians();
    let obliquity = (23.439 - 0.000_000_4 * n).to_radians();
    let declination = (obliquity.sin() * longitude.sin()).asin().to_degrees();
    let right_ascension = (obliquity.cos() * longitude.sin())
        .atan2(longitude.cos())
        .to_degrees();
    let distance = 1.000_14 - 0.016_71 * anomaly.cos() - 0.000_14 * (2.0 * anomaly).cos();
    (declination, right_ascension, distance)
}

/// Bennett's refraction at an apparent altitude, arcminutes (Meeus, eq. 16.4).
fn bennett(apparent_deg: f64) -> f64 {
    1.0 / (apparent_deg + 7.31 / (apparent_deg + 4.4))
        .to_radians()
        .tan()
}

/// The rising hour angle at which the Sun's centre stands at a geocentric
/// altitude, degrees, negative.
fn rising_hour_angle(altitude_deg: f64, declination_deg: f64) -> f64 {
    let (phi, dec) = (LATITUDE_DEG.to_radians(), declination_deg.to_radians());
    let cos_h = (altitude_deg.to_radians().sin() - phi.sin() * dec.sin()) / (phi.cos() * dec.cos());
    -cos_h.acos().to_degrees()
}

/// The delay of the upper limb's rise from the level horizon to a level
/// skyline of `skyline_deg`, days, under the almanac's 34′ carried up by
/// Bennett's formula: the hour angle the Sun must still turn, at the
/// declination it has when it gets there, over the hour angle's rate.
fn derived_delay(rise_jd: f64, skyline_deg: f64) -> f64 {
    let (_, ra_before, _) = low_precision_sun(rise_jd - 0.5);
    let (_, ra_after, _) = low_precision_sun(rise_jd + 0.5);
    let ra_rate = (ra_after - ra_before).rem_euclid(360.0);
    // The Earth's turn against the stars (*Astronomical Almanac*, B), less
    // the Sun's own motion in right ascension.
    let rate = 360.985_647 - ra_rate;
    let target = |at: f64, skyline: f64, refraction_arcmin: f64| {
        let (_, _, distance) = low_precision_sun(at);
        let semidiameter = 0.266_6 / distance;
        let parallax = 8.794 / 3600.0 / distance;
        skyline - refraction_arcmin / 60.0 - semidiameter + parallax
    };
    let (dec_level, _, _) = low_precision_sun(rise_jd);
    let level = rising_hour_angle(target(rise_jd, 0.0, 34.0), dec_level);
    let refracted = 34.0 * bennett(skyline_deg) / bennett(0.0);
    let mut delay = 0.0;
    for _ in 0..4 {
        let at = rise_jd + delay;
        let (dec, _, _) = low_precision_sun(at);
        let seen = rising_hour_angle(target(at, skyline_deg, refracted), dec);
        delay = (seen - level) / rate;
    }
    delay
}

#[test]
fn a_two_degree_skyline_delays_the_sunrise_by_the_derived_amount() {
    let sdk = context();
    // A ridge two degrees high from 80° to 140°, the November Sun rising
    // inside it, and the rest of the skyline level.
    let ridge = json!([
        {"azimuthDeg": 60, "elevationDeg": 0},
        {"azimuthDeg": 80, "elevationDeg": 2},
        {"azimuthDeg": 140, "elevationDeg": 2},
        {"azimuthDeg": 160, "elevationDeg": 0},
    ]);
    let answer = days(&sdk, Some(ridge)).unwrap();
    let visible = &answer.value["visible"]["value"];
    assert_eq!(visible.as_array().map(Vec::len), Some(3));
    for day in visible.as_array().unwrap() {
        let sun = &day["sun"];
        let rise = instant(&sun["rise"]);
        let seen = &sun["visibleRise"];
        assert_eq!(seen["terrainDeg"], 2.0, "{seen}");
        let azimuth = seen["azimuthDeg"].as_f64().unwrap();
        assert!((100.0..120.0).contains(&azimuth), "{azimuth}");
        let measured = (instant(&seen["at"]) - rise) * 86_400.0;
        let derived = derived_delay(rise, 2.0) * 86_400.0;
        println!("delay at JD {rise:.5}: measured {measured:.3} s, derived {derived:.3} s");
        assert!(
            (measured - derived).abs() < DELAY_TOLERANCE_S,
            "measured {measured} s, derived {derived} s"
        );
        // Some eleven minutes: two degrees at about ten degrees an hour,
        // and the refraction a quarter of a degree less behind the ridge.
        assert!((600.0..800.0).contains(&measured), "{measured}");
        // The visible set is earlier than the level one, the ridge only in
        // the east: the western skyline is level, and the set there the
        // level horizon's.
        let set = instant(&sun["set"]);
        let seen_set = instant(&sun["visibleSet"]["at"]);
        assert!(
            (seen_set - set).abs() * 86_400.0 < SAME_RISE_S,
            "{seen_set} {set}"
        );
        assert_eq!(sun["visibleState"], "CROSSES");
    }
    // A skyline two degrees high all round finds the same rise to the bit:
    // the Sun only ever stood on the ridge's flat top.
    let flat = days(&sdk, Some(json!([{"azimuthDeg": 0, "elevationDeg": 2}]))).unwrap();
    for (ridge, flat) in visible
        .as_array()
        .unwrap()
        .iter()
        .zip(flat.value["visible"]["value"].as_array().unwrap())
    {
        assert_eq!(ridge["sun"]["visibleRise"], flat["sun"]["visibleRise"]);
    }
}

#[test]
fn a_level_skyline_of_zero_is_the_level_horizons_rise_and_set() {
    let sdk = context();
    let answer = days(&sdk, Some(json!([{"azimuthDeg": 0, "elevationDeg": 0}]))).unwrap();
    let mut compared = 0;
    for day in answer.value["visible"]["value"].as_array().unwrap() {
        for body in ["sun", "moon"] {
            let body = &day[body];
            assert_eq!(body["visibleState"], "CROSSES");
            for (level, seen) in [("rise", "visibleRise"), ("set", "visibleSet")] {
                assert_eq!(body[level].is_null(), body[seen].is_null(), "{body}");
                if body[level].is_null() {
                    continue;
                }
                let gap = (instant(&body[seen]["at"]) - instant(&body[level])) * 86_400.0;
                assert!(gap.abs() < SAME_RISE_S, "{gap} s: {body}");
                compared += 1;
            }
        }
    }
    // Three days' sunrises and sunsets, and the Moon's that fell in them.
    assert!(compared >= 9, "{compared}");
}

#[test]
fn the_days_open_at_the_astronomical_sunrise_whatever_the_skyline() {
    let sdk = context();
    let level = days(&sdk, None).unwrap();
    let hills = days(&sdk, Some(json!([{"azimuthDeg": 0, "elevationDeg": 5}]))).unwrap();
    assert!(level.value.get("visible").is_none());
    assert_eq!(level.value["days"], hills.value["days"]);
    assert_eq!(level.value["dayHashes"], hills.value["dayHashes"]);
    // The section says what it read the skyline as.
    let conventions = &hills.value["visible"]["provenance"]["applied_conventions"];
    assert!(
        conventions
            .as_array()
            .unwrap()
            .iter()
            .any(|convention| convention["knob"] == "visible.horizon"
                && convention["value"] == "1 sample, linear in azimuth"),
        "{conventions}"
    );
    // Five degrees all round: the Moon too clears it later than it rises,
    // where both are of one passage (a rise just before midnight leaves
    // the day's first visible rise to that passage, not the day's).
    for day in hills.value["visible"]["value"].as_array().unwrap() {
        let moon = &day["moon"];
        let pair = (moon["rise"].as_f64(), moon["visibleRise"]["at"].as_f64());
        if let (Some(rise), Some(seen)) = pair {
            assert!(seen > rise || rise - seen > 0.25, "{moon}");
        }
    }
}

#[test]
fn a_skyline_the_sun_never_tops_answers_no_rise_and_says_why() {
    // Oslo at midwinter: the Sun culminates under 7°, so a 20° wall
    // hides the whole day, though the level horizon has its sunrise.
    let sdk = context();
    let request = json!({
        "first": {"year": 2026, "month": 12, "day": 21},
        "latitudeDeg": 59.91, "longitudeDeg": 10.75, "utcOffsetSeconds": 3600,
        "horizonProfile": [{"azimuthDeg": 0, "elevationDeg": 20}],
    });
    let answer = teistro::records::record("almanac.days")
        .unwrap()
        .answer(&sdk, &request.to_string())
        .unwrap();
    let sun = &answer.value["visible"]["value"][0]["sun"];
    assert!(sun["rise"].is_number() && sun["set"].is_number(), "{sun}");
    assert!(
        sun["visibleRise"].is_null() && sun["visibleSet"].is_null(),
        "{sun}"
    );
    assert_eq!(sun["visibleState"], "ALWAYS_BELOW");
}

#[test]
fn every_refusal_names_its_field() {
    let sdk = context();
    let refused = |profile: Value| {
        let error = days(&sdk, Some(profile)).unwrap_err();
        (error.status, error.field().map(str::to_owned))
    };
    let invalid = teistro::Status::InvalidArg;
    let field = |name: &str| (invalid, Some(name.to_owned()));
    assert_eq!(refused(json!([])), field("horizonProfile"));
    assert_eq!(refused(Value::Null), field("horizonProfile"));
    assert_eq!(
        refused(json!([{"azimuthDeg": 360, "elevationDeg": 0}])),
        field("horizonProfile[0].azimuthDeg")
    );
    assert_eq!(
        refused(json!([{"azimuthDeg": -1, "elevationDeg": 0}])),
        field("horizonProfile[0].azimuthDeg")
    );
    assert_eq!(
        refused(json!([
            {"azimuthDeg": 10, "elevationDeg": 0},
            {"azimuthDeg": 10, "elevationDeg": 1},
        ])),
        field("horizonProfile[1].azimuthDeg")
    );
    assert_eq!(
        refused(json!([
            {"azimuthDeg": 20, "elevationDeg": 0},
            {"azimuthDeg": 10, "elevationDeg": 1},
        ])),
        field("horizonProfile[1].azimuthDeg")
    );
    assert_eq!(
        refused(json!([{"azimuthDeg": 0, "elevationDeg": 45}])),
        field("horizonProfile[0].elevationDeg")
    );
    assert_eq!(
        refused(json!([{"azimuthDeg": 0, "elevationDeg": -5}])),
        field("horizonProfile[0].elevationDeg")
    );
    let unknown = refused(json!([{"azimuthDeg": 0, "elevationDeg": 0, "distanceM": 900}]));
    assert_eq!(unknown.0, invalid);
    assert!(
        unknown
            .1
            .as_deref()
            .is_some_and(|field| field.starts_with("horizonProfile[0]")),
        "{unknown:?}"
    );
    let missing = refused(json!([{"azimuthDeg": 0}]));
    assert_eq!(missing.0, invalid);
    assert!(
        missing
            .1
            .as_deref()
            .is_some_and(|field| field.starts_with("horizonProfile")),
        "{missing:?}"
    );
    assert_eq!(
        refused(json!([{"azimuthDeg": "east", "elevationDeg": 0}])),
        field("horizonProfile[0].azimuthDeg")
    );
}
