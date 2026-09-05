# teistro-core

The core of the Teistro SDK: the vocabulary every other crate speaks.

- **The catalogue.** Every kind of entity (grahas, signs, nakshatras,
  tithis, karanas, yogas, weekdays, ayanamshas, house systems, dasha
  systems and the rest, fifty-three kinds) as an enum whose discriminants
  are stable ids, with cited attributes and confidence marks, generated
  from `catalogue/*.yaml` and held to its sources by a gate.
- **Keys and ids.** `graha.SUN` as text, `(kind, id)` packed for the C
  boundary, resolution with a suggestion when a key is one edit off.
- **Quantities.** Validated newtypes for latitude, longitude, altitude,
  angles, Julian days on a typed time scale, sign and house indices and
  the rest, so a swapped argument is a compile error.
- **Exact arithmetic.** The canonical nanoarcsecond angle whose
  classification into signs, nakshatras, padas and varga parts is integer
  arithmetic on every platform, and exact rationals for period spans.
- **Settings and profiles.** One complete, hashed settings value built by
  applying a patch to a cited profile, with coherence checked once.
- **The envelope.** The provenance every result carries, and the error
  model with stable status codes.

```rust
use teistro_core::angle::Nas;
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::quantity::Degrees;
use teistro_core::settings::{Profile, SettingsPatch};

let mars = Graha::from_key("MARS").expect("a catalogued key");
assert_eq!(mars.attributes().exaltation.map(|e| e.sign), Some(Rashi::Capricorn));

let longitude = Nas::from_degrees(Degrees::try_new(222.5763).expect("finite"));
assert_eq!(longitude.sign(), Rashi::Scorpio);

let settings = Profile::shipped("nepali-default")
    .expect("shipped")
    .resolve(&SettingsPatch::default())
    .expect("coherent");
println!("{}", settings.settings.hash());
```

Design: `docs/03-design/core-types-and-catalogue.md`,
`settings-and-profiles.md`, `exact-arithmetic.md`. Sources:
`catalogue/README.md`.
