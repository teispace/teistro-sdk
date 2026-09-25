# The ephemeris port and its adapters

Status: `draft`, written 2026-09-05 from spike 3
(`spikes/03-ephemeris-port/README.md`); revised the same day when the
port was promoted into `crates/port-ephemeris`, the completion and the
IAU routines into `crates/astro`, the kit into `crates/ephemeris-kit`
and the adapters under `adapters/`. Derives from
`02-architecture/02-ephemeris-port.md`, ADR-0002, ADR-0009, ADR-0013,
ADR-0019, ADR-0020, ADR-0021 and ADR-0022.

What the built crates differ in from the page as first written, each
with its reason:

- Names follow the SDK's catalogue: `Centre` (British spelling, as the
  settings knob), the sidereal zodiac names a catalogued `Ayanamsha`
  (the engines' mode number is the catalogue's `swiss_mode` attribute,
  so an adapter maps by attribute and the port never carries an engine's
  numbering), the observer is a validated `Place`, the tier and the
  override policy are the settings' own enumerations.
- The rise and set override was added (`horizon_event` on the trait, a
  vtable slot, two kit checks), with the horizon convention as port
  vocabulary so an adapter maps it onto its engine's options
  (`astro-events-and-crossings.md`).
- The conformance kit lives in its own crate above `astro`, because its
  checks compare a provider's overrides with the SDK's own routines,
  which the port cannot depend on; the test provider lives in the port,
  since it is the port's own zero-setup instance.
- Delta T lives in `crates/astro` (the IERS table then Espenak and
  Meeus, `time-and-timezone.md`), which the completion reads for the
  SDK's obliquity; the spike's polynomial stand-in is gone.
- The completion asks the provider for the requested frame first and
  passes a native answer through; it rotates only when the provider
  refuses, so an engine that returns equatorial coordinates itself is
  never second-guessed. The kit and the runner measure the rotation
  through a proxy that refuses every frame but its native one.
- The port's C codes and the SDK's status codes are two numberings at
  two boundaries: a `ProviderError` maps to `UNSUPPORTED`,
  `OUT_OF_RANGE`, `INVALID_ARG` or `PROVIDER`, with the provider's own
  code and message carried.
- The vtable module is the one place outside the future `ffi` crate
  that holds `unsafe` code, `deny` rather than `forbid` at the crate
  with one SAFETY comment per block (`04-implementation/README.md`).

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

A frame is five facts, each a closed enumeration
(`teistro_port_ephemeris::Frame`, `Frame::key()` is the stamp):

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
ayanamsha's catalogue id in bits 16 to 31; packing is total, unpacking
refuses a reserved bit or an id the catalogue does not have.

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
coverage as a Julian Day range, its bodies, its native frame, which
astronomy it computes (`MODERN`, the sky as observed; `CLASSICAL`, a
text's model whose obliquity, precession, motions and sunrise are its
own definitions), whether it computes speeds and how (`DERIVATIVE` of
its places, or a text's `RULE`), its distance unit (astronomical units,
or a classical model's hypotenuse on its radius, `MEAN_DISTANCES`), its
overrides as a bit set (obliquity, Delta T, sidereal time, ayanamsha,
topocentric, houses, rise and set, crossings, stations, eclipses, stars,
DUT1), the ayanamshas it knows, and whether it is deterministic. The SDK validates every request against the capabilities
before any call: a topocentric frame needs an observer, every body must
be offered, every instant must be finite.

### Errors

A whole-batch failure is an error; a per-cell failure is a status. The
error variants and their C codes: unsupported (-1), out of range (-2),
data missing (-3), refused (-4), invalid request (-5), and a provider's
own code with its message. The provider's own codes are offset by the
adapter so they never land in the reserved range; Teimeris's -2 arrives
as -102. As the SDK's error, unsupported is `UNSUPPORTED`, out of range
`OUT_OF_RANGE`, an invalid request `INVALID_ARG`, and the rest
`PROVIDER` with the code and the message.

## 4. The operations

| operation | required | what it means | Teimeris | Swiss Ephemeris | SDK when absent |
|---|---|---|---|---|---|
| `positions(request)` | yes | columns over the grid in the requested frame, or `Unsupported` for a frame the provider cannot produce natively | one batch call, body-major grid transposed by the adapter | one call per cell under the process lock | none; a provider is required |
| `obliquity(jd, scale)` | override | mean and true obliquity, nutation in longitude and obliquity, degrees | native | `SE_ECL_NUT` | IAU 2006 and IAU 2000B, ported from ERFA |
| `delta_t_seconds(jd_ut1)` | override | Delta T in seconds | native (the engine answers in seconds) | native (the library answers in days; the adapter converts) | a table plus a model (section 10) |
| `ayanamsha_deg(jd, scale, ayanamsha)` | override | the mean ayanamsha, the value sidereal longitudes subtract, without the nutation in longitude, for a catalogued ayanamsha | native, no-nutation switch only | native with `SEFLG_NONUT` | the SDK's catalogue (Phase 2) |
| `horizon_event(request)` | override | the next rise, set, transit or antitransit of a body at a place inside a window, under a horizon convention, or `None` when it does not happen | native, through its event search; the convention mapped onto its options, a custom altitude refused unless a twilight | not declared | the SDK's solver (`astro-events-and-crossings.md`) |

| `crossings(request)` | override | every crossing of a quantity (a body's longitude, its speed, or a composite angle of two bodies) over a lattice of boundaries inside a window, in the frame requested, in time order, with each boundary and direction | native, through its crossing search (the quantity and lattice mapped onto its options, the direction read from the quantity's rate at the instant); a topocentric or equatorial request refused, and the SDK's kernel answers | not declared | the SDK's kernel (`astro-events-and-crossings.md`), which `Completion::crossings` runs under the policy |

The overrides not built yet (sidereal time, nodes and apsides, houses,
stations as a search of their own, eclipses, stars) follow the same pattern: a bit in
the capabilities, a trait method with a default that answers
`Unsupported`, a vtable slot that may be null, and a kit check that the
declared override works and agrees with the SDK's own implementation
within the published bound.

## 5. Frame completion

The SDK asks the provider for the requested frame. If the provider
answers, the result passes through and the step is stamped `Native`
(`PassThrough` when the request was the native frame itself). If the
provider refuses with `Unsupported`, the SDK asks for the provider's
native frame and completes the difference:

- coordinates: rotation between the ecliptic and the equator through the
  true obliquity, from the provider under `prefer-native` and
  `native-only` when it declares the override, from the SDK under
  `sdk-only`; speeds by central difference over 1e-3 day;
- zodiac: the sidereal shift through the ayanamsha, from the provider's
  override under `prefer-native` and `native-only` when it declares one,
  otherwise from the SDK's own catalogue (`astro-ayanamsha-catalogue.md`:
  every epoch-defined member, the mean value carried by the precession
  model in force, within 1e-7″ of Teimeris; the twelve star-anchored
  members refused by name until the star table); the shift is applied
  while the columns are ecliptic (before a rotation out of the ecliptic,
  after one into it), so a sidereal ecliptic native frame completes to
  tropical equatorial coordinates, which is what the rise and set solver
  asks for: the provider's own centre, equinox and corrections with
  equatorial coordinates in the tropical zodiac, so an engine answers in
  the apparent frame and a classical text in its own;
- the centre, geocentric to topocentric: built
  (`astro::topocentric`, designed from `topocentric-measured.md`), the
  station on WGS84 at the place's own height with the annual aberration
  taken off and put back around the displacement and the light time it
  saves, applied to every body that is somewhere and to no point that is
  a direction; a heliocentric or barycentric centre, the equinox and the
  corrections are still refused with the step named, and Phase 2's
  `astro-timescales-and-frames.md` supplies light time, aberration,
  deflection, nutation and precession so that a J2000 geometric provider
  (a JPL kernel, the built-in ephemeris) completes to the canonical
  frame.

Every completed result carries its step list, each step stamped with the
implementation that did it (`Completed::step_keys`), which is the
provenance ADR-0020 requires. Measured after the promotion over
Teimeris: 0.16 µs per cell with a native obliquity, 0.30 µs with the
SDK's (one IAU 2000B evaluation and one Delta T lookup per instant), and
the rotation reproduces the provider's own ecliptic output to 2.0e-10″
through its obliquity and 3.9e-4″ through the SDK's, the difference
between the engines' nutation model and IAU 2000B.

## 6. The C shape

The port is one `#[repr(C)]` vtable: `struct_size`, `abi_version`, and
function pointers for `capabilities`, `positions`, `obliquity`,
`delta_t`, `ayanamsha`, `dut1`, `horizon_event` and `crossings`, the last six nullable (ABI version 2). Requests, columns and
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
and SDK obliquity through the refusing proxy) on a grid of 100 instants
at 36.525-day steps from J2000 over the ten classical bodies. Measured
after the promotion (release, Apple Silicon, medians of the best of
three rounds of 200 calls): Teimeris 797 µs directly, 824 µs through the
trait and 809 µs through the vtable (3 % and 1.5 %, inside the
run-to-run spread of about 4 %), 958 µs completed to equatorial with the
native obliquity and 1102 µs with the SDK's; the Swiss library 2234 µs
directly, 2248 µs and 2260 µs through the trait and the vtable, 2604 µs
and 2565 µs completed.

### Threads: one provider per thread, and the SDK stays out of it

A provider is `Send + Sync`, so the SDK never stops a caller threading
its own work. What a caller should thread, and what the SDK deliberately
does **not** do for them, is a measurement rather than a preference.

An engine's context is usually not shareable. Teimeris says so directly
— *"N contexts may be used concurrently from N threads with no locking,
which is a statement about N contexts, not about one shared between
them"* — so its adapter holds `Mutex<Context>` and every call queues.
Measured over a year of the Moon's sign ingresses, **46–48% of a batch
is inside that lock** and the rest is the SDK's arithmetic, so threads
that share one provider can overlap only the rest: 1.7× at four threads
and 2.2× at any number of cores.

The way out is a context each, and its price decides everything:
**opening one costs 43.8 ms, which is 19 491 `positions` calls, and the
largest batch this project measures is 8 174**. A thread that opened its
own context for one batch would pay more for it than the batch costs.

So the shape that wins is a **pool of providers that outlives many
batches**, and it belongs to the caller, who knows how long their process
lives and how many cores they mean to spend:

```rust
// One provider, one almanac, one thread — as many as you have work for.
std::thread::scope(|scope| {
    for (provider, range) in providers.iter().zip(ranges) {
        scope.spawn(move || {
            let model = DrikSun::new(provider, /* … */);
            let almanac = Almanac::new(provider, /* … */);
            almanac.between(range.from, range.to, &place)
        });
    }
});
```

Nothing in the SDK has to change for that, and nothing in the SDK does
it for you. Threading a single batch internally would buy at most 2.2×
and cost a knob, a memo that is either per-thread or contended, and the
determinism of the counts the measured page gates — a bad trade, and the
numbers rather than the taste say so
(`07-roadmap/02-plan-performance-and-passthrough.md`, A3).

## 9. Tests and the conformance kit

The kit (`crates/ephemeris-kit`) runs the same eighteen checks against
every provider under one published set of bounds (`Bounds::DEFAULT`),
never per provider; measured on 2026-09-05 against the test provider,
Teimeris 0.1.0 and the Swiss Ephemeris 2.10.03 over the same twelve
`.se1` files, and against the Surya Siddhanta provider
(`crates/siddhanta`, `tests/kit.rs`):

| check | bound | test provider | Teimeris | Swiss Ephemeris | Surya Siddhanta |
|---|---:|---:|---:|---:|---:|
| capabilities well formed | | 8 bodies, no overrides | 14 bodies, 5 overrides, 47 ayanamshas | 14 bodies, 4 overrides | 9 bodies, 3 overrides, classical, speeds by rule |
| positions finite in range | | 40 cells | 70 cells | 70 cells | 45 cells |
| determinism: identical bits on a repeated request | | identical | identical | identical | identical |
| batch equals single calls, bit for bit | | identical | identical | identical | identical |
| reported speed against a central difference (published, not gated, for speeds by rule) | 2e-3 °/day | 3.2e-5 | 9.4e-5 | 9.4e-5 | 0.23 °/day, Mars (C36) |
| continuity: longitude change over 1e-4 day against speed times the step (the second difference for speeds by rule) | 5e-6 ° | 3.8e-9 | 8.1e-9 | 8.1e-9 | 4.9e-10 |
| out of range reported per cell | | pass | pass | pass | pass |
| unsupported body refused by name | | refused | every body offered | every body offered | refused |
| native obliquity against IAU 2006 and IAU 2000B (published, not gated, for a classical astronomy) | 0.01″ | not declared | 4.0e-4″ | 4.0e-4″ | 2065″: the text's 24° |
| native Delta T against the IERS table, inside the table's span | 1 s | not declared | 0.33 s, at the table's last row | 0.33 s | not declared |
| native ayanamsha against the published values (at J2000; the Surya Siddhanta member at Burgess's 1860 instant against the text's own value for a classical astronomy and against the SDK's epoch construction for a modern one) | 0.1° | not declared | 0.011° (Lahiri 23.8571, Raman 22.4108, Krishnamurti 23.7602, Surya Siddhanta 18.9400) | the same | 7.4e-5° (20.4108° in 1860) |
| native DUT1 within the 0.9 s bound | 0.9 s | not declared | not declared | not declared | not declared |
| native rise and set of the Sun against the SDK's solver, the geometric convention, three sea-level places (published, not gated, for a classical astronomy) | 1 s | not declared | 0.13 s | not declared | 250 s: no equation of time (C37) |
| the same under the almanac's convention (the upper limb with refraction; skipped when the provider refuses the convention) | 10 s | not declared | 7.3 s, at Reykjavík (C34) | not declared | refused: the text gives the centre on the geometric horizon |
| native crossings of Mercury's longitude over the signs, 120 days from the first instant, against the SDK's kernel over the same provider | 1 s | not declared | 0.0034 s over 6 events | not declared | not declared |
| native crossings of the elongation over the tithis, 30 days, against the kernel | 1 s | not declared | 0.0039 s over 30 events | not declared | not declared |
| completion through the native obliquity against the provider's own ecliptic output | 1e-4″ | cannot return equatorial | 2.0e-10″ | 2.0e-10″ | cannot return equatorial |
| completion through the SDK's obliquity and nutation | 0.05″ | | 3.9e-4″ | 3.9e-4″ | |

A classical provider is held to what modern astronomy can check
(determinism, batches, continuity, the refusals, its own ayanamsha
against a published figure) and its definitions are measured against
the IAU routines and published in the report rather than gated: the
text's obliquity, its sunrise and its speed rule are the tradition's
answers, not approximations of the sky's.

The Delta T row is the engines' own table against the IERS series the
SDK carries: they agree to a few hundredths of a second inside the
engines' table and part by a third of a second at the series' last rows
(2026), where the engines extrapolate. CI runs the kit against the test
provider on every change (`cargo test -p teistro-ephemeris-kit`) and
against the built-in provider at every tier; the adapters are run by
hand with the engines present. Unit tests cover the bit packings, the
ERFA ports against ERFA's own reference values, the vtable round trip
and the `.se1` name decoding.

### 9a. Two checks made through the façade

What these two measure is a **chart**, not a position, so they found one
through the façade (`teistro` without its default features, so a kit run
never brings a built-in tier with it) and every kit binary runs both
(`runner::charts`).

**`sdk-only`: a chart is its native positions.** ADR-0009 promises
cross-provider byte identity under `sdk-only`: two providers that agree
on their native positions give the same bytes whatever else each
declares. `kit::sdk_only` holds a provider to it by founding every
section the façade makes twice under that policy, once over the provider
and once over `NativeFrameOnly` of it — the same provider reduced to its
native frame, every override, ayanamsha and engine operation taken away
— and requiring the sealed envelopes identical, provenance included, a
difference named by the first field it reaches. A check that cannot fail
proves nothing, so the same comparison runs under `prefer-native` and
the report says where the provider's overrides part the two; a test
provider declaring four overrides that disagree with the SDK on purpose
holds both halves (`crates/ephemeris-kit/tests/sdk_only.rs`). It holds
over the test provider, the built-in (trivially: no overrides) and the
Surya Siddhanta provider, whose sunrise reaches a chart under
`prefer-native` and none under `sdk-only`. Writing it found that
**`prefer-native` refused a whole chart** when a provider declared the
ayanamsha override and did not list the member asked for: the text's
provider knows only its own, and a chart in Lahiri was refused where the
policy means the SDK's value when there is no native one. The choice is
now made per member, against `Capabilities::ayanamshas`, and each side of
a shift between two sidereal zodiacs is chosen for itself.

**The corpus check: positions against the recorded charts, per class.**
`kit::corpus` founds each of the corpus's 55 recorded births under the
`conformance-baseline` profile over the provider, compares every
position both carry — each graha's sidereal and tropical longitude,
latitude, speed and distance, and the lagna — under the band
`tolerances.json` gives the provider's class, and writes the corpus's
own report (`schema/report.schema.json`), every field listed with how far
it missed. The nodes' distance is not compared: the port calls a node a
direction, and a distance an engine gives one is a convention. The kit's
`KNOWN` lists every way the SDK parts from corpus 0.11.0, each measured
and explained, and `CorpusReport::against` holds it both ways — a miss no
entry explains fails, and so does an entry that explains nothing in a run
it applies to. The built-in tiers run it in CI (`tests/conformance.rs`,
one tier per verify job); Teimeris, the corpus's own ephemeris, by hand
(`--corpus fixtures --class same-ephemeris`), and `--native-frame-only`
founds the charts over its native frame alone, which measures the SDK's
completion rather than the engine's.

Run first, the check failed all 55 charts at every tier, and the tier
made no difference to most of what failed — a residual that does not
scale with what should scale it is a different kind of term. What it
found, each now fixed and held by a test:

| found | how far | the cause |
|---|---:|---|
| every speed in a sidereal chart | 1e-4 °/day, the same in every body | a graha's speed was the tropical rate beside a sidereal longitude; it is now the rate of the longitude it sits beside, the ayanamsha's own rate taken out (`ChartZodiac::rate_deg_per_day`) |
| Ketu's distance | 0 against Rahu's | Ketu is the node line's other end and now takes Rahu's distance |
| the built-in's nodes | 0.017° of latitude in 1800, 0.04° in 2399 | the node's J2000 direction was kept without its latitude, so it came back off the ecliptic of date |
| every planet's distance | 1e-4 AU; the Sun's within 6e-8 | the light time moved the direction and kept the geometric distance |
| every speed over a geometric provider | ±5e-4 °/day, body by body | the centre step was told the provider's native frame after the corrections had made the columns apparent, and dropped the diurnal aberration, 0.3″ turning once a day |
| Mercury's speed near a station | 7e-4 °/day | a speed carried through the equinox, the light's path and the observer's place missed each step's own rate; a reported speed is now the derivative of the completed places (`Completion::deriving_speeds`), which a search, stepping by a rate, does not pay for |

After them, measured 2026-09-25 (outside the two far-future charts):

| run | charts passing | what is left, and why |
|---|---:|---|
| built-in, compact | 53 of 55 | the two charts of 2350 and 2399, whose Delta T the SDK and the engine extrapolate minutes apart |
| built-in, standard | 32 of 55 | the Moon's topocentric speed, up to 3e-3 °/day: the engine's is not the derivative of its own topocentric places (1.1e-3 to 1.6e-3 apart, measured over Teimeris), and the SDK's is |
| built-in, full | 1 of 55 | the same, the outer planets' distances to 4e-6 AU (0.09″ at Saturn) against a band of 1e-6, and the Moon in 1800 to 1850, where the SDK's Delta T is 1.8 to 5.2 seconds below the engine's |
| Teimeris, `same-ephemeris` | 50 of 55 | the far-future pair, the lagna of 1800 and 1830 (the Earth's rotation before the modern era, 0.8″) and at 65°N (1.08e-6° against 1e-6) |

The corpus calls its bands provisional until a harness measures them;
this is that measurement, and where a band is tighter than any
implementation's own consistency — the Moon's topocentric speed, the full
tier's distances — the corpus is the place that changes.

## 10. Delta T

The spike's finding: the Espenak and Meeus polynomial fit is built from
measurements to 2005 and extrapolates after; by 2025-01-01 it is 5.5 s
above the measured value both engines carry, and by 2100 two reasonable
extrapolations differ by 110 s. Five seconds of Delta T move the Moon by
2.7″. The SDK's Delta T is therefore the IERS table where measured (1956
to the present, `crates/astro/data/delta-t.json`, updated with the data
packs) with a cited model either side and an uncertainty on every value
(`time-and-timezone.md`, §3.1); the provider's native Delta T is an
override under the same policy as the others, bounded by the kit inside
the table's span.

## 11. Localisation

None: the port emits keys and numbers; body and ayanamsha names are the
catalogue's.

## 12. Open questions

- Whether the SDK's context holds one provider or a pool for parallel
  requests, and how the Teimeris adapter maps onto that (Phase 1).
- The `reference` tier's JPL kernel adapter and its completion chain
  (ADR-0021, Phase 3).
- The refraction convention of the rise and set override against the
  SDK's (cruxes C34; `astro-events-and-crossings.md`).
- The Teimeris adapter as the Teimeris package's own crate; until then
  it lives under `adapters/ephemeris-teimeris/rust` here.
