# teistro-port-ephemeris

The ephemeris port of the Teistro SDK: the one boundary between the SDK
and an ephemeris (`docs/03-design/ephemeris-port-and-adapters.md`). A
provider must answer positions over a grid of instants and bodies in the
frame it names; everything else (the obliquity and nutation, Delta T,
the ayanamsha, rise and set) is an override it may declare, used under
the profile's policy and stamped in the result. This crate holds the
vocabulary (bodies, frames, requests, columnar responses, capabilities,
errors), the trait, the same contract as a `#[repr(C)]` vtable, the
`.se1` file family the licensed engines read, and the analytic test
provider the SDK builds and tests against with no engine present.

```rust
use teistro_port_ephemeris::{Body, EphemerisProvider, Frame, PositionRequest, TestProvider, TimeScale};

let provider = TestProvider::new();
let request = PositionRequest::new(&[2_451_545.0], TimeScale::Ut1, &[Body::Sun, Body::Moon], Frame::CANONICAL);
let columns = provider.positions(&request).expect("the canonical frame");
assert!(columns.all_ok());
assert!((0.0..360.0).contains(&columns.at(0, 0).expect("the Sun").lon));
```

The frame completion, the IAU routines and the rise and set solver live
in `teistro-astro`; the conformance kit in `teistro-ephemeris-kit`; the
adapters for Teimeris and the Swiss Ephemeris outside the workspace
under `adapters/` (ADR-0019).
