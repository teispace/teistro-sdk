# teistro-siddhanta

The Surya Siddhanta as a computation model: the text's mean motions,
apsides and nodes, the twenty-four-entry sine table with its interpolation
rule, the manda and sighra equations with the four-step procedure for the
star planets, the true daily motions by the text's rules, the latitudes,
the declination, the ascensional difference that gives sunrise and
sunset, the text's precession, and the Lagna from the oblique ascensions.
Every parameter is cited by chapter and verse; the classical path uses
only the text's table and elementary arithmetic, so its results are
bit-identical on every platform. `SiddhantaProvider` presents the model
behind the ephemeris port as a classical astronomy, so a chart from the
text runs through the same trait, completion and solvers as an engine.

```rust
use teistro_core::quantity::{JulianDay, Ut1};
use teistro_siddhanta::SuryaSiddhanta;

let text = SuryaSiddhanta::text();
let instant = JulianDay::<Ut1>::try_new(2_460_413.5).expect("finite");
let sun = text.sun(instant);
println!("sidereal Sun {} moving {:.4}°/day", sun.longitude, sun.speed_deg_per_day);

use teistro_core::quantity::{Latitude, Longitude};
let lagna = text
    .lagna(instant, Latitude::literal(27.7172), Longitude::literal(85.324))
    .expect("a latitude where the Sun rises");
println!("the Lagna at Kathmandu: {:.2}° sidereal", lagna.sidereal_deg);

use teistro_port_ephemeris::EphemerisProvider;
use teistro_siddhanta::SiddhantaProvider;
let provider = SiddhantaProvider::text();
println!("{}", provider.capabilities().describe());
```

Burgess's worked computation for midnight of 1 January 1860 at Washington
is the crate's test vector for every step from the day count to the
horoscope point; the conformance kit runs over the provider in
`tests/kit.rs` and publishes the text's distance from modern astronomy.

Design: `docs/03-design/siddhanta.md`. Source: the Surya Siddhanta in the
translation of Ebenezer Burgess (1860), chapters I to III, cited per
constant in `params.rs`.
