//! Every wrapper that is itself a provider forwards every method: a
//! borrow, a box and the counting wrapper each answer every question as
//! the provider inside does. A method left out of a wrapper's impl
//! silently answers the trait's default ("unsupported") instead, and
//! the provider it wraps is never asked; type erasure once made every
//! grid 421 calls that way.

#![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

use teistro_core::catalogue::Ayanamsha;
use teistro_core::quantity::{JulianDay, Place, Ut1};
use teistro_port_ephemeris::{
    Angles, AnglesRequest, Body, Capabilities, CountingProvider, CrossingEvent, CrossingRequest,
    EphemerisProvider, Frame, Horizon, HorizonEventKind, HorizonRequest, Lattice, Obliquity,
    PositionColumns, PositionRequest, ProviderError, Quantity, TestProvider, TimeScale,
};

/// Answers every method with an error naming it, so a forwarded answer
/// is told from a default one.
struct Marked;

fn marked(method: &str) -> ProviderError {
    ProviderError::invalid(format!("marked {method}"))
}

impl EphemerisProvider for Marked {
    fn capabilities(&self) -> Capabilities {
        TestProvider::new().capabilities()
    }

    fn positions(&self, _: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
        Err(marked("positions"))
    }

    fn obliquity(&self, _: f64, _: TimeScale) -> Result<Obliquity, ProviderError> {
        Err(marked("obliquity"))
    }

    fn delta_t_seconds(&self, _: f64) -> Result<f64, ProviderError> {
        Err(marked("delta_t"))
    }

    fn ayanamsha_deg(&self, _: f64, _: TimeScale, _: Ayanamsha) -> Result<f64, ProviderError> {
        Err(marked("ayanamsha"))
    }

    fn dut1_seconds(&self, _: f64) -> Result<f64, ProviderError> {
        Err(marked("dut1"))
    }

    fn horizon_event(&self, _: &HorizonRequest) -> Result<Option<JulianDay<Ut1>>, ProviderError> {
        Err(marked("horizon"))
    }

    fn angles(&self, _: &AnglesRequest) -> Result<Angles, ProviderError> {
        Err(marked("angles"))
    }

    fn crossings(&self, _: &CrossingRequest) -> Result<Vec<CrossingEvent>, ProviderError> {
        Err(marked("crossings"))
    }

    fn native_manifest(&self) -> Result<String, ProviderError> {
        Err(marked("native_manifest"))
    }

    fn native_call(&self, _: &str, _: &str) -> Result<String, ProviderError> {
        Err(marked("native_call"))
    }
}

/// Every answer a provider gives, one question per method.
fn answers(provider: &impl EphemerisProvider) -> Vec<String> {
    let place = Place::try_from_degrees(27.7, 85.3, 0.0).unwrap();
    let jd = JulianDay::<Ut1>::J2000;
    let jds = [jd.get()];
    let request = PositionRequest::new(&jds, TimeScale::Ut1, &[Body::Sun], Frame::CANONICAL);
    let horizon = HorizonRequest {
        body: Body::Sun,
        kind: HorizonEventKind::Rise,
        place,
        from: jd,
        window_days: 1.0,
        horizon: Horizon::CENTRE_NO_REFRACTION,
    };
    let angles = AnglesRequest {
        jd: jd.get(),
        scale: TimeScale::Ut1,
        place,
    };
    let crossing = CrossingRequest {
        quantity: Quantity::Longitude(Body::Sun),
        lattice: Lattice::SIGNS,
        from: jd,
        to: JulianDay::<Ut1>::literal(jd.get() + 31.0),
        tolerance_days: 1e-7,
        frame: Frame::CANONICAL,
        observer: None,
    };
    vec![
        format!("{:?}", provider.capabilities()),
        format!("{:?}", provider.positions(&request).err()),
        format!("{:?}", provider.obliquity(jd.get(), TimeScale::Ut1)),
        format!("{:?}", provider.delta_t_seconds(jd.get())),
        format!(
            "{:?}",
            provider.ayanamsha_deg(jd.get(), TimeScale::Ut1, Ayanamsha::Lahiri)
        ),
        format!("{:?}", provider.dut1_seconds(jd.get())),
        format!("{:?}", provider.horizon_event(&horizon)),
        format!("{:?}", provider.angles(&angles)),
        format!("{:?}", provider.crossings(&crossing)),
        format!("{:?}", provider.native_manifest()),
        format!("{:?}", provider.native_call("f", "{}")),
    ]
}

#[test]
fn every_wrapper_answers_as_the_provider_inside() {
    let direct = answers(&Marked);
    assert!(
        direct
            .iter()
            .skip(1)
            .all(|answer| answer.contains("marked")),
        "{direct:#?}"
    );
    assert_eq!(answers(&&Marked), direct, "a borrow");
    let boxed: Box<dyn EphemerisProvider> = Box::new(Marked);
    assert_eq!(answers(&boxed), direct, "a box");
    let counting = CountingProvider::new(Marked);
    assert_eq!(answers(&counting), direct, "the counting wrapper");
    let calls = counting.calls();
    // One of each counted kind: positions, obliquity, delta T, the
    // ayanamsha, DUT1, a horizon event, the angles and a crossing.
    assert_eq!(calls.total(), 8, "{calls:?}");
}
