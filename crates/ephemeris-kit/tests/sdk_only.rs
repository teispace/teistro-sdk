//! The `sdk-only` check against a provider whose overrides disagree with
//! the SDK on purpose.
//!
//! The test provider declares no override, so over it the check holds
//! trivially and could not tell a policy that works from one that is
//! ignored. This provider declares four and answers each of them wrongly
//! enough to move a chart — a Delta T of a hundred seconds, an ayanamsha
//! of 23°, an obliquity a second of arc off. Under `prefer-native` those
//! must reach the documents, which proves the comparison sees them; under
//! `sdk-only` none may, which is the promise.

#![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

use teistro_core::catalogue::Ayanamsha;
use teistro_core::settings::OverridePolicy;
use teistro_ephemeris_kit::sdk_only::{self, NativeFrameOnly};
use teistro_port_ephemeris::{
    Capabilities, EphemerisProvider, Obliquity, Overrides, PositionColumns, PositionRequest,
    ProviderError, TestProvider, TimeScale,
};

/// The test provider, with four overrides that disagree with the SDK.
struct Overriding(TestProvider);

impl EphemerisProvider for Overriding {
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            overrides: Overrides::OBLIQUITY
                .with(Overrides::DELTA_T)
                .with(Overrides::AYANAMSHA),
            ayanamshas: vec![Ayanamsha::Lahiri],
            ..self.0.capabilities()
        }
    }

    fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
        self.0.positions(request)
    }

    fn obliquity(&self, _jd: f64, _scale: TimeScale) -> Result<Obliquity, ProviderError> {
        let arcsecond = 1.0 / 3600.0;
        Ok(Obliquity {
            mean_deg: 23.4 + arcsecond,
            true_deg: 23.4 + arcsecond,
            nutation_lon_deg: 0.0,
            nutation_obl_deg: 0.0,
        })
    }

    fn delta_t_seconds(&self, _jd_ut1: f64) -> Result<f64, ProviderError> {
        Ok(100.0)
    }

    fn ayanamsha_deg(
        &self,
        _jd: f64,
        _scale: TimeScale,
        _ayanamsha: Ayanamsha,
    ) -> Result<f64, ProviderError> {
        Ok(23.0)
    }
}

fn open() -> Box<dyn EphemerisProvider> {
    Box::new(Overriding(TestProvider::new()))
}

#[test]
fn under_sdk_only_a_chart_is_its_native_positions_and_nothing_else() {
    let check = sdk_only::check(&open);
    assert!(check.passed, "{}", check.detail);
    assert!(
        check.detail.contains("overrides make them part"),
        "the overrides must be seen to reach a chart, or the pass is vacuous: {}",
        check.detail
    );
}

/// The comparison the check rests on, shown failing where it must: under
/// `prefer-native` the provider's own ayanamsha reaches every sidereal
/// longitude, and the parting is named by the first field it reaches.
#[test]
fn under_prefer_native_the_overrides_part_the_charts_by_a_named_field() {
    let parting = sdk_only::parting(&open, OverridePolicy::PreferNative).unwrap();
    assert!(parting.founded > 0, "{parting:?}");
    let at = parting.at.unwrap();
    assert!(at.starts_with("readings[0]."), "{at}");
}

/// The reduction takes away every frame but the native one, so a chart
/// over it is completed by the SDK or not made at all.
#[test]
fn the_native_frame_alone_answers_nothing_else() {
    let reduced = NativeFrameOnly::new(Overriding(TestProvider::new()));
    let capabilities = reduced.capabilities();
    assert_eq!(capabilities.overrides, Overrides::NONE);
    assert!(capabilities.ayanamshas.is_empty() && !capabilities.native);
    assert!(reduced.delta_t_seconds(2_451_545.0).is_err());
}
