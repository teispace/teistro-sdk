# teistro-time

The time layer of the Teistro SDK (`docs/03-design/time-and-timezone.md`):
the civil side of time. The UTC conversions over the astronomy layer's
Delta T (re-exported here: the IERS table then a model, with an
uncertainty on every value), the leap-second table, civil time in any
calendar, zone resolution over the embedded tzdb with the metadata a
stored chart replays under a newer database (offset, source, era,
version, what the daylight-saving policy did), local mean time, the
sunrise-anchored local day under the profile's sunrise convention from
any solar model (the Surya Siddhanta or modern astronomy through the
ephemeris port), the `SUNRISE` fallback for a birth without a time, and
ghati-pala as exact integer arithmetic.

```rust
use teistro_calendar::CalendarDate;
use teistro_core::catalogue::Calendar;
use teistro_time::{CivilDateTime, CivilTime, EmbeddedTzdb, Policy, ZoneSpec, resolve};

let civil = CivilDateTime::at(
    CalendarDate::defined(Calendar::Gregorian, 1990, 4, 14),
    CivilTime::new(5, 30, 0).expect("a real time"),
);
let resolved = resolve(&civil, &ZoneSpec::iana("Asia/Kathmandu"), &Policy::default(), EmbeddedTzdb::shared())
    .expect("a zone the database knows");
assert_eq!(resolved.zone.offset.to_string(), "+05:45");
```

Data: `data/leap-seconds.json` (the IANA list), generated into
`src/generated.rs` by `cargo xtask gen time` and held by
`cargo xtask check-time`; the Delta T data lives with the astronomy
layer (`crates/astro/data/delta-t.json`).
