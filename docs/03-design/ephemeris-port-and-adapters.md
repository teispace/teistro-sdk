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
  code and message carried. A single refused cell maps the same way
  (`CellStatus::status`): an instant past the coverage is `OUT_OF_RANGE`
  whether one cell or the whole request was refused for it.
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
| `ayanamsha_deg(jd, scale, ayanamsha, basis)` | override | a catalogued ayanamsha on the basis asked: `MEAN` without the nutation in longitude, `TRUE` with the engine's own nutation added (the basis every drik profile subtracts from apparent longitudes, SR-100); a basis the engine cannot answer is `UNSUPPORTED`, and the SDK's catalogue answers it under `PREFER_NATIVE` | native, both bases from its own nutation | `SEFLG_NONUT` for `MEAN`, without it for `TRUE` | the SDK's catalogue (Phase 2), on either basis |
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
`delta_t`, `ayanamsha`, `dut1`, `horizon_event`, `crossings`, `native_manifest` and `native_call`, the last eight nullable (ABI version 4). Requests, columns and
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
one tier per verify job). An adapter is run by the generic
`cargo xtask check-adapter --adapter DIR` (the kit under the published
bounds, then every binding's plugin test with the adapter loaded;
`06-cicd/adapter-gate.md`). Teimeris, the corpus's own ephemeris, is run
by it from the engine's private repository, against every SDK batch
before it merges; the corpus run is by hand
(`--corpus fixtures --class same-ephemeris`), and `--native-frame-only`
founds the charts over its native frame alone, which measures the SDK's
completion rather than the engine's. The adapter is its own Cargo
workspace with a committed `Cargo.lock`; every gate runs Cargo there with
`--locked`, so a crate of the SDK gaining a dependency fails the gate
until `cargo xtask relock-adapter --adapter DIR` writes the lock again (and
those of the crates its `locked-crates` lists).

**The DE440 check: geometric positions against JPL's own ephemeris.**
`kit::jpl` asks a provider for geocentric geometric positions on the
J2000 ecliptic at every instant of the corpus's `jpl/` recording (CSPICE
over NAIF's DE440, evidence rank 1) that it covers, each TDB stamp turned
to TT by the two leading terms of TDB − TT, and compares each body's
direction and distance under the `jpl.geometric.*` bands of its class.
The geometric rung has no light time, aberration, deflection or Delta T
in it, so what it measures is the provider's theory alone. Its own
`KNOWN` is held both ways like the charts'. Run first over the built-in
tiers it found that the full tier is not the arcsecond its class
promises away from 2000: ELP2000-82B and VSOP87 as published drift from
DE440, the Moon and Mars to about 3″ and Neptune to about 7″ by 2400, and
the standard tier misses by the same, so the drift is the theories' and
not their truncation. VSOP87's radius vectors part from DE440's by
hundreds of kilometres for Mars to Saturn and tens of thousands for
Uranus and Neptune. Each is a measured entry there, scoped to the tiers
it holds in; the reference tier (a DE440 refit, ADR-0021) is what closes
them.

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

Since the boundary audit (2026-11-14, rank 2) Delta T has **one owner per
question**, and every request says which model it is asked under
(§12.1): the engine when it declares the override, the policy lets it
answer and it implements the model asked; the SDK's routine otherwise,
which then carries the UT instant to TT itself and asks the engine on
TT. Until then a modern provider was handed UT and its own Delta T
answered, whatever `time.delta_t` said.

## 11. Localisation

None: the port emits keys and numbers; body and ayanamsha names are the
catalogue's.

## 12. The conventions every question states

The boundary audit of 2026-11-14 (`notes/boundary-audit-2026-11-14.md`,
ranks 2 to 7) found an engine answering under conventions nobody had
asked for: its own Delta T, its own precession and nutation, its own
refraction and solar radius. The port now states each one on every
question, an engine answers under it or refuses, and it never
substitutes its own. Vtable version 6, `TS_ABI_VERSION` 3.

### 12.1 What a request carries

`Conventions` (`port-ephemeris/src/conventions.rs`) rides on every
`PositionRequest`, `HorizonRequest`, `CrossingRequest` and
`AnglesRequest`, and is an argument of `obliquity` and `ayanamsha_deg`:

- `delta_t`, the `DeltaTModel` a UT instant is carried to TT by
  (`time.delta_t`; a custom value travels as its seconds);
- `models`, the precession, nutation, frame bias and sidereal time
  (`frame.precession`, `frame.nutation`, `frame.frame_bias`,
  `time.sidereal_time`), packed at the C boundary in one `u32`.

A request's conventions are the context's, so a per-call patch reaches
them like any other setting. A cache keys a cell by them (`bits()`), so
two models never share a cell.

`delta_t(jd_ut1, model)` replaces `delta_t_seconds(jd_ut1)` and answers a
`DeltaTEstimate`: the seconds, the one-sigma uncertainty when the model
publishes one, and the band it comes from (`MEASURED` inside the IERS
table, `STEPHENSON_MORRISON_HOHENKERK_2016`, `MORRISON_STEPHENSON_2004`,
`UNSTATED`).

### 12.2 Who owns Delta T

The engine answers Delta T only when three things hold: it declares the
`DELTA_T` override, the policy chooses it (`prefer-native` or
`native-only`), and it implements the model asked, which the completion
learns once by asking (`delta_t(jd, model)`; a refusal is remembered).
Then a UT instant goes to the engine as UT and the step is stamped
`delta-t:NATIVE`. Otherwise the SDK's routine owns it: the completion
carries the UT instant to TT with the model and asks the engine on TT,
stamped `delta-t:SDK`. A classical text is asked on UT as given, since it
reckons in civil time.

Why this rule and not "always the SDK" or "always the engine": the SDK
cannot make an engine's UT answers agree with a model the engine does
not implement, and it should not override an engine that implements the
model asked, since that engine also knows its own tidal acceleration.
The built-in implements no Delta T of its own, so the SDK's conversion
was already its conversion and its answers do not move; a modern engine
under the default `TABLE_THEN_MODEL` is now asked on TT, which is the
fix for `nepali-drik` ignoring `time.delta_t` (rank 2).

The answer's provenance states the value, the model, the uncertainty,
the band and the owner at the answer's first instant
(`provenance.time.delta_t_seconds`, `delta_t_model`,
`uncertainty_seconds`, `delta_t_band`, `delta_t_by`) and the models
(`provenance.time.models`).

### 12.3 The horizon

`day.horizon` (`HorizonModel`) states what a convention left implicit,
and `Horizon` carries it on every rise and set:

| field | values | default, and why |
|---|---|---|
| `refraction` | `BENNETT`, `SAEMUNDSSON`, `SINCLAIR` | `BENNETT` (Bennett 1982; Meeus, eq. 16.4): what the SDK has reckoned a named air by |
| `disc_size` | `ACTUAL`, `MEAN_DISTANCE`, `FIXED{semidiameter_arcmin}` | `ACTUAL`: the disc at the body's distance |
| `radii` | `IAU_2015`, `CUSTOM{sun_km, moon_km}` | `IAU_2015`: the Sun 695,700 km (IAU 2015 B3), the Moon 1,737.4 km |
| `latitude` | `BODY`, `ZERO` | `BODY`: the body where it is; `ZERO` is the Hindu rising of the ecliptic point it stands over |
| `dip` | `NONE`, `GEOMETRIC`, `REFRACTED` | `NONE`: the almanacs reckon from sea level |
| `altitude_deg` | −4 to 20 | 0 |

The formula applies to a named air (`Refraction::Atmosphere`); the
almanac's standard refraction stays the stated 34′, named as such. The
geometric dip is `−acos(R / (R + h))` on the WGS 84 radius; the refracted
one is Sinclair's through the near-ground refraction coefficient, as the
Swiss Ephemeris reckons it. Bennett's formula is refused below −4°, where
its pole at −4.4° makes it meaningless. A terrain profile stands in for
the model's altitude and dip.

There is one solar radius, 695,700 km: the eclipse module's 696,000 km is
retired (`eclipse::SUN_RADIUS_KM` is the rise-and-set constant).

### 12.4 DUT1

`time.dut1` is `NONE` (UT1 taken as UTC, within the 0.9 s a leap second
keeps them; the default, so no answer moves), `IERS` (the engine's,
asked through the `DUT1` override; refused when no engine declares it)
or `CUSTOM{seconds}` (within ±0.9 s). `time.convert` applies it between
UTC and UT1 and says what it applied.

### 12.5 Routed through the policy

Rise and set and crossings go through the completion's `choose`, as every
other override does: under `prefer-native` a modern engine that declares
`RISE_SET` answers the SDK's solver's question, at any altitude and under
any horizon model, and one that declares `CROSSINGS` answers a tropical
crossing search. An engine that refuses the convention hands the
question back to the SDK under `prefer-native` and refuses under
`native-only`. A sidereal crossing stays the SDK's: the request carries
no ayanamsha basis, and an engine's sidereal frame is the mean basis
alone. A terrain profile stays the SDK's too.

### 12.6 What the kit adds

| check | bound | what it holds |
|---|---:|---|
| `override_delta_t` | 1 s | Delta T under each model the engine answers (`TABLE_THEN_MODEL`, `ESPENAK_MEEUS_2006`, `STEPHENSON_MORRISON_HOHENKERK_2016`) against the SDK's routine for the same model at 1900, 2000 and 2050, and a custom value exactly; an engine that declares the override and answers no model fails |
| `override_ayanamsha_value` | 1e-4° | every listed member on both bases against the SDK's catalogue under the standard models |
| `override_rise_set_lower_limb`, `_atmosphere`, `_twilight`, `_moon` | 10 s, 10 s, 1 s, 10 s | the lower limb, Bennett through the standard air, the Sun's centre at −12°, and the Moon's upper limb, against the SDK's solver under `sdk-only` |
| `override_crossings_station` | 1 s | Mercury's stations over 240 days |
| `override_crossings_sidereal` | 1 s | the Sun's sidereal ingresses over a year, mean basis |
| `override_topocentric` | 0.1″ | the engine's topocentric Sun and Moon from Kathmandu at 1,400 m against the SDK's correction of its geocentric ones |
| `positions_reference` (`kit::check_reference`) | 1″ | a provider's positions against a reference provider's, the built-in for an engine |

The rise and set checks ask the SDK's side under `sdk-only`, since under
`prefer-native` the solver now hands the question to the engine it is
checking.

### 12.7 The Teimeris adapter, change by change

Each new question maps one to one onto a call the engine already has on
its `main` from f27f3bb (`teimeris/teistro/docs/engine-conventions-calls.md`,
header `teimeris/core/include/teimeris/events.h` and siblings). The change
is a separate Teimeris commit, in `teimeris/teistro/rust/src/`:

`lib.rs`

1. `TeimerisProvider::over`: keep the overrides; add `Overrides::DUT1`
   once item 8 lands.
2. `positions`: before the call, `stated(request.conventions)` (a new
   helper, below); when `request.scale` is `UT1`, pass the Delta T model
   through `tm_set_model(ctx, TM_MODEL_DELTA_T, model)` (or
   `tm_set_delta_t_override(ctx, s)` for `CUSTOM`), so the engine's UT
   conversion is the one asked. The completion only sends UT when item 4
   said yes.
3. `obliquity(jd, scale, conventions)`: `stated(conventions)`, then
   `ctx.obliquity` as today.
4. `delta_t(jd_ut1, model)` replaces `delta_t_seconds`:
   `tm_delta_t_estimate_calc(ctx, jd_ut1, model, &e)` with
   `STEPHENSON_MORRISON_HOHENKERK_2016` → `TM_DELTA_T_STEPHENSON_ETC_2016`
   (5), `ESPENAK_MEEUS_2006` → `TM_DELTA_T_ESPENAK_MEEUS_2006` (4),
   `CUSTOM(s)` → `tm_set_delta_t_override(ctx, s)` then model 0 (check
   `e.override_applied`), and `TABLE_THEN_MODEL` refused `UNSUPPORTED`:
   the engine's chain is its own table, not the SDK's, so the SDK owns
   the default. Answer `DeltaTEstimate { seconds: e.seconds,
   sigma_seconds: Some(e.sigma_seconds), band }` with `TM_DELTA_T_BAND_SMH_2016`
   → `StephensonMorrisonHohenkerk2016`, `TM_DELTA_T_BAND_MS_2004` →
   `MorrisonStephenson2004`.
5. `ayanamsha_deg(jd, scale, ayanamsha, basis, conventions)`:
   `stated(conventions)`, then `tm_ayanamsha_bases_calc(ctx, jd, scale,
   mode, 0, 0, &b)` and answer `b.mean_deg` or `b.true_deg` by `basis`;
   `set_ayanamsha` leaves this path.
6. `horizon_event`: today (f27f3bb) it builds `EventOptions` with the
   engine's defaults for the conventions the request does not name; it
   builds `tm_event_request` from the request's whole horizon instead,
   `engine_horizon_options` retiring its refusals:
   - `refraction`: `Refraction::None` → `NO_REFRACTION`;
     `Refraction::Standard` → `TM_EVENT_REFRACTION_FIXED` with
     `refraction_arcmin = 34`; `Refraction::Atmosphere(air)` → the model's
     formula, `BENNETT` → `_BENNETT`, `SAEMUNDSSON` → `_SAEMUNDSSON`,
     `SINCLAIR` → `_ENGINE`, with `atmosphere` the air resolved at the
     place (`engine_atmosphere`, unchanged);
   - `disc`: as today; `disc_size` `ACTUAL` / `MEAN_DISTANCE` /
     `FIXED{a}` → `TM_EVENT_DISC_SIZE_ACTUAL` / `_MEAN_DISTANCE` /
     `_FIXED_ANGLE` with `semidiameter_deg = a / 60`, and `ACTUAL` for the
     centre, which the engine refuses with a fixed disc;
   - `radii`: `IAU_2015` → `TM_EVENT_RADII_IAU_2015`; `CUSTOM` →
     `TM_EVENT_RADII_CUSTOM` with the body's `radius_km`;
   - `latitude`: `ZERO` → `TM_EVENT_HINDU_RISING`;
   - `dip`: `NONE` / `GEOMETRIC` / `REFRACTED` → `TM_EVENT_DIP_NONE` /
     `_GEOMETRIC` / `_REFRACTED`;
   - the altitude: the convention's altitude plus the model's
     `altitude_deg` → `horizon_height_deg`, the Sun's and the Moon's at
     any depression as the centre unrefracted (the engine's twilight
     rule), so the −6/−12/−18 table and the Moon's refusal go.
   The search asks with `request.conventions.delta_t` set as in item 2.
7. `crossings`: `stated(request.conventions)`; a sidereal frame keeps
   `set_ayanamsha` (mean basis), which the SDK never routes here.
8. `dut1_seconds(jd_utc)`, new: `tm_dut1_calc(ctx, jd_utc, TM_DUT1_IERS,
   0, &d)`, answering `d.seconds`; outside the data it answers 0 and the
   adapter refuses `OUT_OF_RANGE` when `!d.in_data`, so `time.dut1 = IERS`
   says so rather than applying a silent zero.
9. `stated(conventions) -> Result<(), ProviderError>`, new: for each of
   the four models, `tm_set_model(ctx, kind, engine_model(knob))` with
   `TM_MODEL_PRECESSION_LONG`/`_SHORT` (both from `frame.precession`),
   `_NUTATION`, `_FRAME_BIAS`, `_SIDEREAL_TIME`, remembering the last set
   so an unchanged request costs nothing; a model the engine lacks is
   `UNSUPPORTED`.
10. `over` and `Data::context`: after the models are set,
    `tm_identity_get(ctx, &id)`; `Identity.data_version` gains
    `id.fingerprint` (FNV-1a 64), so two runs under different models
    compare unequal. `$TEIMERIS_PROFILE` stays a default only.
11. `tm_set_nutation_fast_path(ctx, 1)` stays off; a dense scan
    (crossings) may turn it on around its own calls.

`dispatch.rs` and `passthrough.rs`: already regenerated at f27f3bb
(`teistro-gen engine`), so `engine.*` names the six new entry points
(146 of 169 functions) and needs no further change here. The engine
pass-through (`engine.call`, ADR-0030) builds `tm_event_request` in the
generated `read_tm_event_request`, reading every field through
`Within::number`/`Within::narrow`, which refuse a missing key
(`` `req.refraction_model` is required ``). The seven appended horizon
fields (`refraction_model`, `refraction_arcmin`, `disc_size`,
`semidiameter_deg`, `radii`, `radius_km`, `dip`) are therefore
**exposed, not defaulted**: a caller states them as it states every
other field, and the generated typed wrappers (`engine.d.ts`,
`engine.py`'s `TmEventRequest`, and the Dart and Java wrappers)
already list them. The SDK builds no engine request itself: `engine.call`
forwards the caller's JSON to the adapter unread, so the SDK cannot and
should not fill an engine's defaults (it is engine-agnostic, §12.5).
Filling them in the pass-through was rejected: the pass-through's one
rule is that a key left out is a mistake, and an old caller's request
with `struct_size` the full size would be answered as all-zero anyway,
which the engine documents as the default, so a silent fill would hide
the mistake without changing an answer. The change a caller needs is
named instead: a request written before f27f3bb adds the seven keys,
each `0` for the engine's default (its old behaviour, bit for bit), or
starts from `engine.call("tm_event_request_init_sized")`, which the
pass-through already offers and which answers the engine's default
request, the seven fields included, for the caller to edit. The
adapter's README gains that example; nothing else changes.

`ranges.rs`: nothing.

The solar eclipse's seen maximum (SR-106) reads
`tm_solar_eclipse_search_local`'s `seen_maximum`, `seen_magnitude` and
`seen_limited_by` once the port grows a local-eclipse override; until
then the SDK's own search answers.

The adapter's gate (`gate.sh`) relocks against the SDK revision that
carries vtable 6 and runs the kit, whose new rows (§12.6) are the
acceptance test.

## 13. Open questions

- Whether the SDK's context holds one provider or a pool for parallel
  requests, and how the Teimeris adapter maps onto that (Phase 1).
- The `reference` tier's JPL kernel adapter and its completion chain
  (ADR-0021, Phase 3).
- The refraction convention of the rise and set override against the
  SDK's (cruxes C34; `astro-events-and-crossings.md`). Settled
  2026-11-14 by naming it: the request carries the formula, the air and
  the rest of the horizon (§12.3).
- The Teimeris adapter as the Teimeris package's own crate. Done
  2026-11-12: it is private to Teispace, in the engine's own repository
  (`06-cicd/adapter-gate.md`).
