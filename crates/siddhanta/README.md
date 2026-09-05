# teistro-siddhanta

The Surya Siddhanta as a computation model: the text's mean motions,
apsides and nodes, the twenty-four-entry sine table with its interpolation
rule, the manda and sighra equations with the four-step procedure for the
star planets, the true daily motion, the declination, the ascensional
difference that gives sunrise and sunset, and the text's precession. Every
parameter is cited by chapter and verse; the classical path uses only the
text's table and elementary arithmetic, so its results are bit-identical
on every platform.

```rust
use teistro_core::quantity::{JulianDay, Ut1};
use teistro_siddhanta::SuryaSiddhanta;

let text = SuryaSiddhanta::text();
let instant = JulianDay::<Ut1>::try_new(2_460_413.5).expect("finite");
let sun = text.sun(instant);
println!("sidereal Sun {} moving {:.4}°/day", sun.longitude, sun.speed_deg_per_day);
```

Design: `docs/03-design/siddhanta.md`. Source: the Surya Siddhanta in the
translation of Ebenezer Burgess (1860), chapters I to III, cited per
constant in `params.rs`.
