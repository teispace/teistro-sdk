# The ephemeris port and its adapters

Status: `draft`, written 2026-09-05 from spike 3
(`spikes/03-ephemeris-port/README.md`); revised in Phase 1 when the
`ephemeris` module is built. Derives from
`02-architecture/02-ephemeris-port.md`, ADR-0002, ADR-0009, ADR-0013,
ADR-0019, ADR-0020, ADR-0021 and ADR-0022. Type and function names are
the spike's; Phase 1 renames into the SDK's catalogue without changing
the shapes.

## 1. Purpose and scope

The port is the one boundary between the SDK and an ephemeris. It
requires positions over a grid and nothing else; every other astronomical
quantity is either computed by the SDK's `astro` layer from those
positions or supplied by the provider as a declared override, used under
the profile's policy and stamped in the result. This page settles the
port's data model, its C shape, the frame completion, the adapter rules
for licensed engines, and the conformance kit that every provider passes
before it is offered.

## 2. Inputs, settings and ports

Inputs are a request: instants (`f64` Julian Days with one time scale,
`UT1` or `TT`), bodies, a frame, an optional observer, and whether speeds
are wanted. The settings knob the port reads is the profile's
`provider_overrides` policy (`prefer-native`, `sdk-only`, `native-only`;
ADR-0013). The port needs no other port.

## 3. The data model

### The frame

A frame is five facts, each a closed enumeration:

| field | values | canonical |
|---|---|---|
| centre | geocentric, topocentric, heliocentric, barycentric | geocentric |
| equinox | of date, J2000 | of date |
| coordinates | ecliptic, equatorial | ecliptic |
| zodiac | tropical, sidereal with an ayanamsha id | tropical |
| corrections | light time, aberration, deflection, nutation, each on or off | all on (apparent) |

The canonical frame is the apparent geocentric ecliptic of date,
tropical, which is what both licensed engines return by default and what
every chart module consumes. A frame packs into 32 bits for the C
boundary: centre in bits 0 to 1, equinox in bit 2, coordinates in bit 3,
the four corrections in bits 4 to 7, the zodiac in bit 8 and the
ayanamsha id in bits 16 to 23; the packing is total in both directions.

### The response

Positions come back as columns, one `f64` vector per quantity
(longitude or right ascension, latitude or declination, distance, and
the three speeds), with a status and a source per cell. The layout is
instants outermost: cell `index = instant × bodies + body`. Distances are
in astronomical units, angles in degrees, speeds per day. A cell status
is one of `Ok`, `NotComputed`, `UnsupportedBody`, `OutOfRange`,
`DataMissing` or a provider's own code; the codes on the C side are 0,
-6, -1, -2, -3 and the provider's number, which must stay outside -1 to
-6. A source is the ephemeris kind (files, JPL kernel, analytic, test,
unknown) and the tier when the provider has tiers, packed into 32 bits.

A cell is never silently something other than what was asked: an
instant outside the declared coverage is `OutOfRange` whatever the engine
did with it, and an engine's fallback to an analytic model when a file is
missing is `DataMissing`. The response frame is the request frame, or the
provider's native frame when the provider refused the request and the
SDK completed it (section 5).

### Capabilities

A provider declares once: its identity (name, version, data version,
tier, and the SHA-256 and byte length of every data file it reads), its
coverage as a Julian Day range, its bodies, its native frame, whether it
computes speeds, its overrides as a bit set (obliquity, Delta T,
sidereal time, ayanamsha, topocentric, houses, rise and set, crossings,
stations, eclipses, stars), the ayanamshas it knows, and whether it is
deterministic. The SDK validates every request against the capabilities
before any call: a topocentric frame needs an observer, every body must
be offered, every instant must be finite.

### Errors

A whole-batch failure is an error; a per-cell failure is a status. The
error variants and their C codes: unsupported (-1), out of range (-2),
data missing (-3), refused (-4), invalid request (-5), and a provider's
own code with its message. The provider's own codes are offset by the
adapter so they never land in the reserved range; Teimeris's -2 arrives
as -102.

## 4. The operations

| operation | required | what it means | Teimeris | Swiss Ephemeris | SDK when absent |
|---|---|---|---|---|---|
| `positions(request)` | yes | columns over the grid in the requested frame, or `Unsupported` for a frame the provider cannot produce natively | one batch call, body-major grid transposed by the adapter | one call per cell under the process lock | none; a provider is required |
| `obliquity(jd, scale)` | override | mean and true obliquity, nutation in longitude and obliquity, degrees | native | `SE_ECL_NUT` | IAU 2006 and IAU 2000B, ported from ERFA |
| `delta_t_seconds(jd_ut1)` | override | Delta T in seconds | native (the engine answers in seconds) | native (the library answers in days; the adapter converts) | a table plus a model (section 10) |
| `ayanamsha_deg(jd, scale, id)` | override | the mean ayanamsha, the value sidereal longitudes subtract, without the nutation in longitude | native, no-nutation switch only | native with `SEFLG_NONUT` | the SDK's catalogue (Phase 2) |

The overrides that the spike did not build (sidereal time, nodes and
apsides, houses, events, crossings, stations, eclipses, stars) follow the
same pattern: a bit in the capabilities, a trait method with a default
that answers `Unsupported`, a vtable slot that may be null, and a kit
check that the declared override works and agrees with the SDK's own
implementation within the published bound.

## 5. Frame completion

The SDK asks the provider for the requested frame. If the provider
answers, the result passes through and every step is stamped `Native`.
If the provider refuses with `Unsupported`, the SDK asks for the
provider's native frame and completes the difference:

- coordinates: rotation between the ecliptic and the equator through the
  true obliquity, from the provider under `prefer-native` and
  `native-only` when it declares the override, from the SDK under
  `sdk-only`; speeds by central difference over 1e-3 day;
- zodiac: the sidereal shift through the ayanamsha, from the provider's
  override or, in Phase 2, the SDK's catalogue;
- centre, equinox and corrections: refused in the spike; Phase 2's
  `astro-timescales-and-frames.md` supplies light time, aberration,
  deflection, nutation and precession so that a J2000 geometric provider
  (a JPL kernel, the built-in ephemeris) completes to the canonical
  frame.

Every completed result carries its step list, each step stamped with the
implementation that did it, which is the provenance ADR-0020 requires.
Measured: 0.16 µs per cell with a native obliquity, 0.32 µs with the
SDK's, and the rotation reproduces the provider's own ecliptic output to
2e-10″ through its obliquity and 4e-4″ through the SDK's.

## 6. The C shape

The port is one `#[repr(C)]` vtable: `struct_size`, `abi_version`, and
function pointers for `capabilities`, `positions`, `obliquity`,
`delta_t` and `ayanamsha`, the last three nullable. Requests, columns and
capabilities cross as `#[repr(C)]` structs with their own `struct_size`
so either side can be older. Columns are caller-allocated: the SDK owns
the vectors and hands pointers with a capacity; a provider writes into
them and never allocates for the SDK. A Rust provider is exported into
this vtable and a vtable is bound back into a Rust provider; the round
trip is bit-identical (tested), and a binding's host-language provider
is the same vtable with typed-array grids (spike 2 measured the crossing:
0.5 µs into JavaScript, 0.1 µs into Dart). Measured in the spike: the
vtable costs 16 ns per cell over the trait.

## 7. Adapters

The rules, from ADR-0019, as built and tested in the spike:

- Adapters for licensed engines are separate packages outside the
  workspace; the dependency runs adapter to port. The workspace's fast
  check builds with the test provider and no adapter, which is the
  containment check.
- Every switch is explicit on every call: the ephemeris flag, the speed
  flag, the frame flags derived from the request's frame; an engine never
  chooses a model for the SDK.
- A fallback to an analytic model is read from the flags the engine
  returns and reported as `DataMissing`; an instant outside the declared
  coverage is `OutOfRange` without a call.
- Data files are content-hashed at open and the hashes enter the
  identity; coverage is declared from the files present (`sefile`).
- Engines with global state are serialised behind one lock, and the
  state-setting calls (sidereal mode, observer, path) and the
  computation happen under one hold. The Swiss adapter holds one static
  lock for the process, sets the path once and refuses a second
  directory. Its stress test interleaves sidereal modes and observers
  across eight threads and demands bits identical to the serial run.
- An engine's own status codes are carried with an offset out of the
  reserved range; its messages travel in the error, never in a cell.
- The raw binding is private; the adapter's public surface is the port
  and one direct-call entry used only by the benchmark.

The Teimeris adapter holds one context behind a mutex because the
sidereal mode is context state; a pool of contexts is a Phase 1 decision
for the SDK's own context. Teimeris returns the canonical frame and all
four overrides natively; its grid is body-major and the adapter
transposes, at a cost inside the measurement noise.

## 8. Performance budget and benchmark

Budget: the port and the vtable together add no more than 5 % to the
engine's own batch call on a 1000-cell grid; frame completion adds no
more than 0.5 µs per cell. The benchmark is the kit runner's standard
rows (the engine directly, the trait, the vtable, completion with native
and SDK obliquity) on a grid of 100 instants at 36.525-day steps from
J2000 over the ten classical bodies. Measured in the spike: 0.2 % and
1.8 % over Teimeris's own call, within noise over the Swiss library.

## 9. Tests and the conformance kit

The kit runs the same thirteen checks against every provider under one
published set of bounds, never per provider:

| check | bound |
|---|---:|
| capabilities well formed | |
| positions finite in range | |
| determinism: identical bits on a repeated request | |
| batch equals single calls, bit for bit | |
| reported speed against a central difference | 2e-3 °/day |
| continuity: longitude change over 1e-4 day against speed times the step | 5e-6 ° |
| out of range reported per cell | |
| unsupported body refused by name | |
| native obliquity against IAU 2006 and IAU 2000B | 0.01″ |
| native Delta T against the SDK's fit, 1900 to 2005 only | 5 s |
| native ayanamsha at J2000 against the published values | 0.1° |
| completion through the native obliquity against the provider's own ecliptic output | 1e-4″ |
| completion through the SDK's obliquity and nutation | 0.05″ |

Phase 1 adds the corpus checks (positions against fixtures per tier,
ADR-0022) and an `sdk-only` cross-provider byte-identity check. CI runs
the kit against the test provider on every change and against the
Teimeris adapter and the built-in provider at every tier when they exist
in the workspace; the Swiss adapter is run by hand before a release of
the port. Unit tests cover the bit packings, the ERFA ports against
ERFA's own reference values, the vtable round trip and the `.se1` name
decoding.

## 10. Delta T

The spike's finding: the Espenak and Meeus polynomial fit is built from
measurements to 2005 and extrapolates after; by 2025-01-01 it is 5.5 s
above the measured value both engines carry, and by 2100 two reasonable
extrapolations differ by 110 s. Five seconds of Delta T move the Moon by
2.7″. The SDK's Delta T is therefore a table of the published values to
the present (the IERS and USNO series, updated with the data packs), a
model for the future and the past (the long-term parabola with the
lunar-acceleration correction), and the fit only as the last resort; the
provider's native Delta T is an override under the same policy as the
others, bounded by the kit inside the measured era only.

## 11. Localisation

None: the port emits keys and numbers; body and ayanamsha names are the
catalogue's.

## 12. Open questions

- Whether the SDK's context holds one provider or a pool for parallel
  requests, and how the Teimeris adapter maps onto that (Phase 1).
- The `reference` tier's JPL kernel adapter and its completion chain
  (ADR-0021, Phase 3).
- Which Delta T series the data packs carry and how they are updated
  (Phase 1, with `time-and-timezone.md`).
