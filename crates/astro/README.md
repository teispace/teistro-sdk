# teistro-astro

The astronomy layer of the Teistro SDK: everything above raw positions
(`docs/02-architecture/01-module-catalog.md`). Built so far:

- Delta T as the IERS table where measured (1956 to the present) and a
  cited model either side, with an uncertainty on every value, and the
  conversions between UT1 and TT (`delta_t`, `scale`; the data file is
  `data/delta-t.json`, generated into `src/generated.rs` by
  `cargo xtask gen time` and held by `cargo xtask check-time`);
- the IAU routines ported from ERFA with a provenance table (`iau`,
  ADR-0021): the Earth rotation angle, mean and apparent sidereal time,
  the IAU 1980 and 2006 obliquities, the IAU 2000B nutation, the
  fundamental arguments, the equation of the equinoxes, the refraction
  constants;
- the obliquity record, the rotation between the ecliptic and the
  equator, and apparent sidereal time at a place (`sky`);
- frame completion over the ephemeris port (`completion`): from the
  frame a provider returns to the frame a caller asks for, the override
  policy deciding who does each step, every step stamped;
- the shared boundary solver (`solve`): one root finder every event
  search in the SDK goes through;
- the rise and set solver (`rise_set`) under a horizon convention, with
  polar days and nights reported as absences;
- precession as a catalogue of models (`precession`: Vondrák 2011 the
  default, IAU 2006, IAU 1976, Newcomb) over the ported IAU 2006 and
  long-term routines, with the mean obliquity each is consistent with;
- the ayanamsha catalogue (`ayanamsha`): every epoch-defined member
  computed from its published epoch and value carried by precession,
  mean or with nutation, within 1e-7″ of Teimeris, so a sidereal zodiac
  needs no provider override;
- the twenty-two house systems (`houses`) as one construction with
  twenty-two choices of circles, the auxiliary points and the polar
  policy, within 5e-6° of Teimeris at ten latitudes.

```rust
use teistro_astro::rise_set::Solver;
use teistro_astro::{Completion, DeltaTModel};
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
use teistro_core::settings::OverridePolicy;
use teistro_port_ephemeris::{Body, Horizon, TestProvider};

let provider = TestProvider::new();
let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
let solver = Solver::new(&sky, Body::Sun, kathmandu, Horizon::CENTRE_NO_REFRACTION, DeltaTModel::TableThenModel);
let day = solver.day(JulianDay::literal(2_460_482.5 - 85.324 / 360.0)).expect("a day");
assert!(day.arc().is_some());
```

Design: `docs/03-design/ephemeris-port-and-adapters.md` (the port and
the completion), `astro-events-and-crossings.md` (the solvers),
`time-and-timezone.md` (Delta T). The precession and nutation models,
the ayanamsha catalogue, house systems, crossings and stations follow in
Phase 2.
