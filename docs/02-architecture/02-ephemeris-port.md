# The ephemeris port

Status: `draft`, revised 2026-09-05 after spike 3. The required surface
is positions only; the SDK's `astro` layer computes everything above
them; a provider may declare native overrides; the built-in ephemeris is
one provider among others. Signatures are modelled on Teimeris's headers.
Rust sketches are illustrative; the settled shapes, bit layouts, codes,
adapter rules and kit bounds are in
`03-design/ephemeris-port-and-adapters.md`, written from the spike's
measurements (`spikes/03-ephemeris-port/README.md`): the port costs 0.2 %
over Teimeris's own batch call and 1.8 % through the C vtable, frame
completion 0.16 to 0.32 µs per cell, and both licensed engines pass the
same thirteen checks with the same numbers.

## Capabilities

```rust
pub struct EphemerisIdentity { name: String, version: String, data_version: String, tier: Option<String> }

pub struct EphemerisCapabilities {
    identity: EphemerisIdentity,
    jd_range: (f64, f64),                  // UT1 coverage
    bodies: BodySet,                       // Sun..Pluto, nodes mean/true, apogees, Chiron, asteroids...
    native_frame: PositionFrame,           // what `positions` returns: J2000 or of-date; geometric or apparent;
                                           // heliocentric, barycentric or geocentric; which corrections applied
    speeds: bool,                          // analytic speeds available
    overrides: OverrideSet,                // NUTATION, OBLIQUITY, SIDEREAL_TIME, DELTA_T, AYANAMSHA, HOUSES,
                                           // RISE_SET, CROSSINGS, STATIONS, ECLIPSES, STARS, TOPOCENTRIC
    ayanamshas: AyanamshaSet,              // for the AYANAMSHA override
    house_systems: HouseSystemSet,         // for the HOUSES override
    deterministic: bool,
}
```

The context validates: every module needs only `positions` for the bodies
it uses within the jd range; overrides are used when declared and when the
profile's `provider_overrides` policy allows (`prefer-native` default,
`sdk-only` for byte-identical cross-provider results, `native-only` to
refuse SDK fallbacks). Every result stamps which implementation computed
each part.

## Operations

| operation | required | signature sketch | Teimeris | Swiss | SDK default when absent |
|---|---|---|---|---|---|
| `positions(grid) -> PositionColumns` | **yes** | `jds[]` with timescale, `bodies[]`, requested frame → columns lon, lat, dist and speeds (or rectangular vectors), status per cell, `frame_returned`, `source` | `tm_position_calc_grid_columns` | loop over `swe_calc_ut` | none; a provider is required, and the built-in provider is the zero-setup one |
| `nodes_apsides(grid)` | no | mean and true node, mean and osculating apogee | `tm_nodes_apsides_calc_many` | `swe_nod_aps_ut` | derived by `astro` from the provider's Moon positions and elements when the provider exposes them, else mean elements for the mean node and a derivation for the true node |
| `delta_t`, `sidereal_time`, `obliquity`, `nutation` | no | overrides | Teimeris exposes all | Swiss exposes all | `astro` |
| `ayanamsha(jd, id, flags)` | no | override | `tm_ayanamsha_value` | `swe_get_ayanamsa_ex_ut` | `astro` catalogue |
| `houses`, `houses_many` | no | override | `tm_houses_calc_many` | `swe_houses_ex` | `astro` house systems |
| `rise_set(req)` | no | override | `tm_event_search` | `swe_rise_trans_true_hor` | `astro` solver |
| `crossings(req)`, `stations` | no | override | `tm_crossing_search`, `tm_scan_grid` | none | `astro` search |
| `eclipses(req)` | no | override | `tm_solar_eclipse_search` and kin | `swe_sol_eclipse_when_glob` and kin | `astro` (v1.x) |
| `stars(query)` | no | override | `tm_star_calc` | `swe_fixstar2` | `astro` star table |

Batch is the shape of every operation; scalars exist only in ergonomic
layers.

## Frame completion

The provider declares its native frame; `astro` completes the chain to the
SDK's canonical frame (apparent geocentric ecliptic of date, with the
optional flags true, no-nutation, no-aberration, J2000, heliocentric,
topocentric) and stamps every position with the corrections applied. A
provider that already returns the canonical frame (Teimeris, Swiss) passes
through untouched. This is what makes a JPL-kernel adapter or the built-in
analytic provider produce charts that mean the same thing as a Swiss chart.

## Frames on the request

Ayanamsha id or custom parameters, sidereal flag, observer, node type and
correction flags travel with every request; providers with global state
(Swiss) serialise inside the adapter under one lock that covers the
state-setting calls and the computation, and are documented as
non-reentrant. The ayanamsha override means the mean ayanamsha, the
value sidereal longitudes subtract; both engines also offer the value
with the nutation in longitude, which the house circle needs.

## Two adapter shapes, one API

Native vtable (Teimeris, Swiss shim, the built-in provider) or a
host-language object wrapped by the binding into a vtable with typed-array
grids. The conformance kit runs against both; the benchmark publishes the
cost ratio.

## Provenance

Provider identity, tier, native frame, per-position source and corrections,
and which of the higher operations ran natively or in `astro`.

## Conformance kit

Instants, places and bodies with expected values and tolerance bands per
tier; determinism; capability honesty (declared overrides must work and
agree with `astro` within the published bound); a report. CI runs it
against the Teimeris adapter, the built-in provider at every tier, and the
test provider. The spike's kit has thirteen checks under one published
set of bounds; a native Delta T is held to the SDK's fit only inside the
fit's measured era (1900 to 2005), because the fit is 5 s high by 2025
and Phase 1's Delta T is a table plus a model.
