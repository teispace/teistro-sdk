# Examples

Every file here is a runnable program that `cargo xtask check-python`
runs, so none of them can drift from what the binding does. They are
meant to be read in order: each one assumes the one before it.

```sh
cargo build --release -p teistro-ffi
cd bindings/python
TEISTRO_LIBRARY=../../target/release/libteistro_ffi.dylib \
PYTHONPATH=. python3 example/birth_chart.py
```

| file | the scenario | what it is really teaching |
|---|---|---|
| [`quickstart.py`](quickstart.py) | the smallest thing that works | opening the library, a context, one call of each kind |
| [`birth_chart.py`](birth_chart.py) | a Nepali birth record to the nine grahas placed by sign, nakshatra and pada | the canonical frame is **tropical**; a Vedic chart asks for a sidereal one and the SDK completes it. Also: a zone's history matters, and Rahu is a body the chart calls a graha |
| [`panchanga.py`](panchanga.py) | the five limbs of a day — tithi, vara, nakshatra, yoga, karana | every limb but the weekday is a function of two longitudes, so a binding can compute a whole almanac from `positions`. The karana is not a cycle of eleven, and the rule is the SDK's own |
| [`calendar.py`](calendar.py) | a Bikram Sambat year and one month as a calendar page | month lengths are decided by the Sun and vary year to year, so they are asked for and never assumed; and every date says whether it came from the official table or the SDK's engine |
| [`ephemeris.py`](ephemeris.py) | a year of the sky in one call | the grid is one crossing, not 366; a column is a `memoryview` numpy wraps without copying; and the provenance's settings hash is the cache key |
| [`your_own_ephemeris.py`](your_own_ephemeris.py) | putting your own engine behind the SDK | the provider contract in full — one call per grid, refusing a frame so the SDK completes it, coverage checked before you are asked, and an exception that reaches the caller |

## What these examples do not do

They all use `test_provider=True`, the analytic provider the SDK carries,
so that they run anywhere with no ephemeris to install. It is a smooth
model: its positions are plausible and its motions are not. Nothing in it
turns retrograde except the lunar node, which always is, and its
positions are not an ephemeris's. Point the SDK at a real provider —
`05` shows how — and every one of these programs is unchanged.

They also stop where the C boundary does. Houses, divisional charts,
planetary states, aspects and dashas are computed by the SDK's Rust
crates and do not yet cross the boundary, so a binding cannot ask for
them. What a binding *can* do is everything above, which is most of what
an application actually shows.
