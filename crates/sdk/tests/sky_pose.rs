//! The sky pose through the façade (`03-design/sky-pose.md`): its sky is
//! the sky view's value bit for bit, each placement is the pose's own
//! projection of that sky's point, and the record and its refusals.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    reason = "tests fail by panicking, index what they asked for and compare what is copied"
)]

use teistro::catalogue::Graha;
use teistro::sky_pose::Pose;
use teistro::{Context, Ephemeris, SkyPoseRequest, SkyViewRequest};

const VIEW: &str =
    r#""latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400, "at": 2461345.25"#;

fn context() -> Context {
    Context::builder()
        .profile("nepali-drik")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

fn posed(pose: &str) -> SkyPoseRequest {
    SkyPoseRequest::from_json(&format!(r#"{{{VIEW}, "pose": {pose}}}"#)).unwrap()
}

#[test]
fn the_sky_is_the_sky_views_and_each_point_is_the_poses_projection() {
    let sdk = context();
    let asked = posed(
        r#"{"azimuthDeg": 100, "altitudeDeg": 25, "rollDeg": -12, "horizontalFovDeg": 66, "aspectRatio": 0.5625}"#,
    );
    let seen = sdk.almanac().sky_pose(&asked).unwrap();
    let view = sdk
        .almanac()
        .sky_view(&SkyViewRequest::from_json(&format!("{{{VIEW}}}")).unwrap())
        .unwrap();
    let value = &seen.value;
    assert_eq!(value.sky, view.value);
    assert_eq!(value.pose, asked.pose);
    let sky = &value.sky;
    assert_eq!(value.screen.grahas.len(), sky.grahas.len());
    assert_eq!(value.screen.stars.len(), sky.stars.len());
    assert_eq!(value.screen.ecliptic.len(), sky.ecliptic.len());
    assert_eq!(value.screen.horizon.len(), sky.horizon.len());
    for (graha, placed) in sky.grahas.iter().zip(&value.screen.grahas) {
        assert_eq!(
            *placed,
            asked
                .pose
                .place(graha.azimuth_deg, graha.refracted_altitude_deg)
        );
    }
    for (point, placed) in sky.horizon.iter().zip(&value.screen.horizon) {
        assert_eq!(*placed, asked.pose.place(point.azimuth_deg, 0.0));
    }
    let (rising, placed) = value.ascendant().unwrap();
    assert_eq!(
        *placed,
        asked
            .pose
            .place(rising.azimuth_deg, rising.refracted_altitude_deg)
    );
    assert!((value.vertical_fov_deg - asked.pose.vertical_fov_deg()).abs() < 1e-12);
    // The pose is in the input and the projection is named; the sky
    // view's hash and conventions are its own.
    assert_ne!(seen.provenance.input_hash, view.provenance.input_hash);
    assert!(
        seen.provenance
            .applied_conventions
            .iter()
            .any(|convention| convention.knob == "sky.pose")
    );
    assert!(
        !view
            .provenance
            .applied_conventions
            .iter()
            .any(|convention| convention.knob == "sky.pose")
    );
}

/// Pointed at the Moon's refracted place, the Moon is the picture's
/// centre and in view, whatever the roll.
#[test]
fn pointed_at_the_moon_the_moon_is_the_centre() {
    let sdk = context();
    let probe = sdk
        .almanac()
        .sky_view(&SkyViewRequest::from_json(&format!("{{{VIEW}}}")).unwrap())
        .unwrap()
        .value;
    let moon = probe.graha(Graha::Moon).unwrap();
    for roll in [-170.0, 0.0, 45.0] {
        let asked = posed(&format!(
            r#"{{"azimuthDeg": {}, "altitudeDeg": {}, "rollDeg": {roll}, "horizontalFovDeg": 40, "aspectRatio": 1.5}}"#,
            moon.azimuth_deg, moon.refracted_altitude_deg
        ));
        let seen = sdk.almanac().sky_pose(&asked).unwrap().value;
        let placed = seen.screen.grahas[1];
        assert!(placed.in_view, "{placed:?}");
        assert!((placed.x.unwrap() - 0.5).abs() < 1e-9, "{placed:?}");
        assert!((placed.y.unwrap() - 0.5).abs() < 1e-9, "{placed:?}");
    }
}

#[test]
fn the_record_answers_the_area_with_its_members_in_full() {
    let sdk = context();
    let text = format!(
        r#"{{{VIEW}, "grahas": ["MOON"], "pose": {{"azimuthDeg": 0, "altitudeDeg": 90, "rollDeg": 0, "horizontalFovDeg": 90, "aspectRatio": 1}}}}"#
    );
    let record = teistro::records::record("almanac.skyPose").unwrap();
    let answered = record.answer(&sdk, &text).unwrap();
    let written = sdk
        .almanac()
        .sky_pose_query(&SkyPoseRequest::from_json(&text).unwrap())
        .unwrap();
    assert_eq!(answered.value, written.value);
    assert_eq!(answered.value["sky"]["grahas"][0]["graha"], "graha.MOON");
    assert_eq!(
        answered.value["sky"]["stars"][13]["nakshatra"],
        "nakshatra.CHITRA"
    );
    assert_eq!(answered.value["pose"]["altitudeDeg"], 90.0);
    // A camera at the zenith sees no point of the horizon: each is 90°
    // off its line of sight.
    for placed in answered.value["screen"]["horizon"].as_array().unwrap() {
        assert_eq!(placed["inView"], false);
    }
    // The sky view still refuses a pose by name, and the sky pose needs one.
    assert_eq!(
        SkyViewRequest::from_json(&format!(r#"{{{VIEW}, "pose": {{}}}}"#))
            .unwrap_err()
            .field(),
        Some("skyView.pose")
    );
    assert_eq!(
        SkyPoseRequest::from_json(&format!("{{{VIEW}}}"))
            .unwrap_err()
            .field(),
        Some("skyPose.pose")
    );
    let none = Context::builder()
        .ephemeris([Ephemeris::None])
        .build()
        .unwrap();
    let asked = posed(
        r#"{"azimuthDeg": 0, "altitudeDeg": 0, "rollDeg": 0, "horizontalFovDeg": 60, "aspectRatio": 1}"#,
    );
    assert_eq!(
        none.almanac().sky_pose(&asked).unwrap_err().status,
        teistro_core::error::Status::Capability
    );
    assert_eq!(Pose::new(0.0, 0.0, 0.0, 60.0, 1.0).unwrap(), asked.pose);
}
