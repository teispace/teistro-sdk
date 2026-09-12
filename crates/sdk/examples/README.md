# Examples

Every file here is a runnable program that `cargo xtask check-rust`
runs, so none of them can drift from what the crate does. They are meant
to be read in order: each one assumes the one before it.

```sh
cargo run --release -p teistro --example birth_chart
```

Release, not debug: the built-in ephemeris is a truncated VSOP87 and
ELP2000, and a debug build of it computes a year of the sky slowly
enough to notice.

| file | the scenario | what it is really teaching |
|---|---|---|
| [`quickstart.rs`](quickstart.rs) | the smallest thing that works | a context is a **value** — no handle to open, no blob to decode, no `dispose` — and one call of each kind |
| [`birth_chart.rs`](birth_chart.rs) | a Nepali birth record to the nine grahas placed by sign, nakshatra and pada | the canonical frame is **tropical**; a Vedic chart asks for a sidereal one and the founder completes it, stamping every step. Also: a zone's history matters, Rahu is a body the chart calls a graha, and an absent ayanamsha is `None` rather than nought |
| [`panchanga.rs`](panchanga.rs) | the five limbs of a day — tithi, vara, nakshatra, yoga, karana | every limb but the weekday is a function of two longitudes, so a consumer can compute a whole almanac from `positions`. The karana is not a cycle of eleven, and the rule is the SDK's own. Each limb is a **catalogue member**, so one that does not exist cannot be constructed |
| [`calendar.rs`](calendar.rs) | a Bikram Sambat year and one month as a calendar page | month lengths are decided by the Sun and vary year to year, so they are asked for and never assumed; and `CalendarResolution` is an **enum a `match` must cover**, where the other three bindings hand over a string |
| [`ephemeris.rs`](ephemeris.rs) | a year of the sky in one call | the grid is one crossing, not 366; a column is the astronomy crate's own `Vec<f64>` with no blob in between; and there is no `buildInfo` to ask for, because Cargo fixed the versions — what is worth logging is the provider's `capabilities` |
| [`rectification.rs`](rectification.rs) | a birth time known only to the hour, narrowed by lagna | `found_many` founds eighteen candidate charts in **one crossing**, sharing the settings, the solar model and the day's sunrise; and `found(one)` is the same crossing unwrapped, agreeing bit for bit |
| [`almanac.rs`](almanac.rs) | a week's panchangam: the five limbs of each day, and its periods | a limb is a **span**, not a name — most days have two tithis, and the SDK gives both with the instant each gives way; a span carries its own bounds as well as the clipped ones; and a value a day may not have is `Option`, which the compiler will not let you ignore |
| [`your_own_ephemeris.rs`](your_own_ephemeris.rs) | putting your own engine behind the SDK | the port in full — one call per grid, refusing a frame so the SDK completes it, coverage marked **per cell** rather than refused, a `ProviderError` whose sentence reaches the caller, an `Arc` newtype for a provider you keep a handle on, and a chain whose first entry is a recipe so a later one can be its fallback |

There is a ninth file, [`parity.rs`](parity.rs), and it is not one of
these: it prints one scenario as `key<TAB>value` lines so that
`cargo xtask check-parity` can compare the four bindings value for
value. `check-rust` leaves it to that gate rather than running it twice.

## Why they repeat themselves

Each file is a program a reader is invited to **copy**, so each carries
its own small helpers — a clock formatter, a sign lookup, an
entity-name-or-key fallback. A shared `support` module would make every
one of them un-copyable, which is the wrong trade here; the DRY rule
applies to what ships, and what ships is the crate.

## What these examples do not do

They all name `Ephemeris::Builtin`, the analytic ephemeris the SDK
carries, so that they run anywhere with nothing to install. It is the
fallback and not the intended path: in most cases a consumer belongs on
a real engine — Teimeris, Swiss Ephemeris — linked as its own crate
under its own licence (ADR-0029). Do that, or hand in an ephemeris of
your own as [`your_own_ephemeris.rs`](your_own_ephemeris.rs) shows, and
every one of these programs is unchanged, which is what the port is for.

They also stop where the façade does. Houses, divisional charts,
planetary states, aspects and dashas are computed by the SDK's Rust
crates and are not on the areas yet, so this surface cannot ask for
them — a Rust consumer *can* depend on those crates directly, which the
other three bindings cannot, and
[`03-design/rust-consumer-surface.md`](../../../docs/03-design/rust-consumer-surface.md)
§6 says what that surface deliberately leaves out.
