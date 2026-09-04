# Spike 3: the ephemeris port

**Question.** Does one port shape, with positions as the only required
operation and everything else an optional override (ADR-0002, ADR-0009,
ADR-0013), carry a native engine, a licensed host library and a
zero-setup test provider through the same C vtable, the same frame
completion and the same conformance kit, at a cost that disappears
against the ephemeris itself, and under the containment rules of
ADR-0019? **Result: yes, measured on three providers; the port's shape
goes into `docs/03-design/ephemeris-port-and-adapters.md`.** One finding
changed the SDK's plan: the SDK's Delta T fit is 5 s high by 2025 against
the tables both engines carry, so Phase 1's Delta T is a table plus a
model, not a fit.

## The slice

```text
port/                 teistro-spike-port          the port: model, trait, C vtable, ERFA-ported IAU routines,
                                                  frame completion with the override policy, the conformance
                                                  kit, the timing helper, the `.se1` scanner, the kit runner,
                                                  the spike-2 test provider behind the port, the kit binary
adapters/teimeris/    teistro-spike-adapter-teimeris   outside the workspace: the port over Teimeris's Rust
                                                  binding (one context behind a mutex, body-major grid transposed)
adapters/sweph/       teistro-spike-adapter-sweph      outside the workspace: the port over the Swiss Ephemeris
                                                  C library compiled from `SWEPH_SRC_DIR` (one process-wide
                                                  lock, explicit flags, fallback refused, hashed data)
results/              test-provider.json, teimeris.json, sweph.json: the kit reports and timings quoted below
```

The port (`port/src/`):

- `model.rs`: bodies, time scales, the observer, the frame (centre,
  equinox, coordinates, zodiac, four corrections; packed to 32 bits for
  the C boundary), the request over a grid of instants and bodies, the
  columnar response (instants outermost, `instant × bodies + body`, one
  status and one source per cell), overrides as a bit set, capabilities
  with identity and content hashes, the obliquity record, the override
  policy, and the error with its reserved codes.
- `provider.rs`: the trait. `positions` is required; `obliquity`,
  `delta_t_seconds` and `ayanamsha_deg` (the mean ayanamsha, the value
  sidereal longitudes subtract) are overrides a provider declares.
- `vtable.rs`: the same contract as a `#[repr(C)]` vtable with a
  `struct_size` handshake and an ABI version. A Rust provider is exported
  into a vtable and a vtable is bound back into a Rust provider; the
  round trip is bit-identical (tested).
- `astro/`: the IAU 2006 mean obliquity and the IAU 2000B nutation ported
  from ERFA's `eraObl06` and `eraNut00b` (BSD-3, `NOTICE`, the provenance
  table in the module), the Espenak and Meeus Delta T fit, and the
  ecliptic and equatorial rotations.
- `completion.rs`: the frame completion. A request the provider answers
  natively passes through; otherwise the provider is asked for its native
  frame and the SDK rotates coordinates through the obliquity (native or
  SDK by policy) and shifts the zodiac through the native ayanamsha;
  every step is stamped `Native`, `Sdk` or `PassThrough`.
- `kit.rs`: the conformance kit, thirteen checks under one published set
  of bounds (`Bounds::DEFAULT`), and its Markdown report.
- `runner.rs`: what every kit binary does, once; `bench.rs`: one timing
  helper; `sefile.rs`: the `.se1` family (coverage from the block names,
  SHA-256 of every file, the star catalogue's presence) shared by both
  adapters so they declare the same coverage and the same hashes.

## How to run

```sh
cargo run --release -p teistro-spike-port --bin teistro-spike-port-kit
TEIMERIS_LIB_DIR=../teimeris/build/release \
  cargo run --release --manifest-path spikes/03-ephemeris-port/adapters/teimeris/Cargo.toml
SWEPH_SRC_DIR=/path/to/swisseph/sources \
  cargo run --release --manifest-path spikes/03-ephemeris-port/adapters/sweph/Cargo.toml
```

The adapters read the `.se1` files from `TEIMERIS_DATA_DIR` or
`SWEPH_DATA_DIR`, defaulting to the Teimeris checkout beside this one.
They are standalone crates: the SDK workspace excludes them
(`Cargo.toml`), the dependency runs adapter to port and never back, and
the fast check builds the workspace with the test provider and no adapter
present, which is the containment check ADR-0019 asks for. Their tests
(`cargo test --manifest-path …`) include the Swiss stress test: eight
threads interleaving two sidereal modes and two observers over four
rounds must return bits identical to the serial run.

Measured on an Apple Silicon laptop, release builds, Teimeris 0.1.0
against its release library, Swiss Ephemeris 2.10.03 compiled from
source, the same twelve `.se1` files (years 600 to 2999) for both.

## Measurements

### The kit

| check | test provider | Teimeris | Swiss Ephemeris | bound |
|---|---:|---:|---:|---:|
| capabilities | 8 bodies, no overrides | 14 bodies, 4 overrides | 14 bodies, 4 overrides | |
| positions finite in range | 40 cells | 70 cells | 70 cells | |
| determinism (repeated request) | identical bits | identical bits | identical bits | |
| batch equals single calls | identical | identical | identical | |
| speed against a central difference, °/day | 3.2e-5 | 9.4e-5 | 9.4e-5 | 2e-3 |
| continuity over 1e-4 day, ° | 3.8e-9 | 8.1e-9 | 8.1e-9 | 5e-6 |
| out of range reported per cell | pass | pass | pass | |
| unsupported body reported | refused by name | every body offered | every body offered | |
| native obliquity against IAU 2006 and 2000B, ″ | not declared | 4.0e-4 | 4.0e-4 | 0.01 |
| native Delta T against the fit, 1900 to 2005, s | not declared | 0.83 | 0.83 | 5 |
| native ayanamsha at J2000, ° (Lahiri, Raman, Krishnamurti) | not declared | 23.8571, 22.4108, 23.7602 | the same | ±0.1 of the published values |
| completion, native obliquity, ″ | cannot return equatorial | 2.0e-10 | 2.0e-10 | 1e-4 |
| completion, SDK obliquity and nutation, ″ | | 3.9e-4 | 3.9e-4 | 0.05 |

All three pass. The two engines report the same worst values to every
digit printed: on this kit they are the same numbers.

### The cost of the port

A grid of 100 instants at 36.525-day steps from J2000 over the ten
classical bodies (eight for the test provider), speeds on, median of the
best of three rounds of 200 calls, microseconds.

| measurement | test provider | Teimeris | Swiss Ephemeris |
|---|---:|---:|---:|
| through the engine's own binding directly | | 788 | 2 256 |
| through the port trait | 18.5 | 790 | 2 273 |
| through the C vtable | 31.4 | 803 | 2 257 |
| completed to equatorial, native obliquity | 302 | 950 | 2 602 |
| completed to equatorial, SDK obliquity and nutation | 299 | 1 112 | 2 577 |

The trait costs 0.2 % over Teimeris's own batch call, including the
transpose from its body-major grid; the vtable 1.8 %. Against the Swiss
library the port is within noise. The test provider shows the crossing's
fixed cost: 13 µs per 800-cell grid, 16 ns per cell.

Completion to the equatorial frame costs 0.16 µs per cell with a native
obliquity and 0.32 µs with the SDK's, the second being one IAU 2000B
evaluation per instant; the equatorial rows of the test provider are the
SDK rotating its own canonical output.

## What the spike found

1. **The shape holds.** Positions required, overrides declared, frames on
   the request, statuses per cell: three providers of three natures
   (a table, a native engine, a host library with global state) sit
   behind the one trait and the one vtable, and the kit and the
   completion do not know which is which.
2. **The port is free against a real ephemeris.** Every cost above the
   engine's own call is bounded by tens of nanoseconds per cell; the
   transpose in the Teimeris adapter is a copy and does not show.
3. **Both engines agree on this kit** to every printed digit, which is
   evidence for Teimeris as the default provider and a warning about the
   kit: it measures self-consistency and capability honesty, not
   accuracy. Accuracy is the conformance corpus's job (ADR-0022).
4. **The SDK's Delta T fit is stale past 2005.** The Espenak and Meeus
   polynomial extrapolates after its last measured year and by
   2025-01-01 is 5.5 s above the measured value both engines carry; by
   2100 two reasonable extrapolations differ by 110 s. The kit therefore
   bounds a native Delta T to the fit only in the fit's measured era, and
   Phase 1's Delta T is a table (the published values to the present)
   plus a model for the future and the past, with the fit as the last
   resort. Five seconds of Delta T move the Moon by 2.7″.
5. **Two engine conventions the port has to absorb.** Teimeris's grid is
   body-major (`body × instants + instant`); the port's is instants
   outermost, so the adapter transposes. Teimeris's ayanamsha call takes
   no ephemeris flag, only the no-nutation switch; passing the file flag
   is an error. Both are adapter facts, not port facts.
6. **The Swiss adapter's containment rules are cheap.** One static lock,
   the state-setting calls and the computation under one hold, the path
   set once and a second directory refused, the ephemeris flag explicit,
   a fallback to the analytic model reported as missing data from the
   returned flags, coverage from the file names, SHA-256 of every file in
   the identity: 450 lines, and the stress test passes.
7. **The mean ayanamsha is the value to expose.** Both engines offer the
   ayanamsha with and without the nutation in longitude; sidereal
   longitudes subtract the mean one, so that is what the override means,
   and the port's documentation says so.
8. **Frame completion is exact where it should be.** Rotating a
   provider's equatorial output back to its ecliptic through the
   provider's own obliquity reproduces its ecliptic output to 2e-10″;
   through the SDK's IAU 2006 obliquity and IAU 2000B nutation to 4e-4″,
   which is the difference between the models, and inside the published
   bound for `sdk-only`.

## What is not covered

- A host-language provider through the vtable: spike 2 measured the
  callback cost (0.5 µs into JavaScript, 0.1 µs into Dart) and the vtable
  here is the same shape; the binding glue is Phase 1.
- The nodes and apsides, houses, events, eclipses and star overrides, and
  the built-in provider: the override mechanism is the same bit set and
  policy; each operation is its own design page.
- A JPL kernel adapter and the `reference` tier (ADR-0021).
- Parallel requests to one Teimeris context: the adapter serialises on a
  mutex because the sidereal mode is context state; a pool of contexts is
  a Phase 1 question for the SDK's own context.
- The adapters do not run in CI: they need the engines. The workspace
  job is the containment check; the adapters are run by hand before a
  release of the port.

## What changes in Phase 1

- The ephemeris port design page carries the model, the vtable, the
  statuses, the codes, the bit layouts and the kit's checks and bounds
  from here, with the Delta T decision above.
- The kit gains the corpus checks (positions against fixtures per tier)
  and a `sdk-only` cross-provider byte-identity check.
- The Teimeris adapter becomes the Teimeris package's own crate; the
  Swiss adapter stays a separate package under its own terms.
