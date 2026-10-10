# The quality bar

Status: `accepted`, 2026-09-04; extended the same day by ADR-0022 (the
determinism contract and the conformance repository) and ADR-0023 (type
safety in every binding); each requirement marked enforced or not,
2026-10-11. The maintainer's instruction: the code must be proper, well
tested, validated, benchmarked, and measured for memory, optimisation and
leaks; everything must be. This page turns that into gates. A module is
done when every row below is green for it, not when its code compiles.

## How to read this page

Every row is a requirement, and none is withdrawn because nothing holds it
yet. The last column says which of two states it is in:

- **enforced**, naming the gate that holds it: a `cargo xtask` command or
  a `cargo` step, and the workflow job that runs it. `fast-check.yml` runs
  on every push to `main` and every pull request; `verify.yml` nightly, on
  demand and inside every release (`release.yml` calls it, and its publish
  job waits for it); `hash-matrix.yml` nightly and on demand;
  `benchmarks.yml` on every pull request. A requirement held only in part
  says which part.
- **not yet enforced: taken up in the hardening pass** (the completion
  plan's week of 31 October, `07-roadmap/00-roadmap.md`). Nothing in the
  repository holds the requirement today, so a module can be merged
  without meeting it.

A gate is named here only if `xtask/src/main.rs` dispatches it or a
workflow runs it; `gate-has-a-runner` (below) holds the second half.

## Correctness

| gate | what it proves | status |
|---|---|---|
| unit tests with classical examples | every formula reproduces a worked example from a text or a reference | **enforced**: `cargo test --workspace`, fast check (`checks`) |
| golden-vector conformance | every P0 computation reproduces the baseline engine, PyJHora or a printout within its stated tolerance; tolerances live in one central file keyed by field and provider class, never per fixture; every fixture asserts the settings hash; every expected value cites its source | **enforced**: `cargo xtask check-fixtures` (schema, manifest, settings hashes) and `cargo xtask check-conformance` (the corpus tests reproduce `CONFORMANCE.md`, no unexplained miss), fast check; verify's `conformance` job holds every recorded tier's score with `check-conformance --from` |
| whole-table invariants | every kernel table passes its invariant list in one test: exact totals, every seed maps to a lord, orders visit every sign once, spans sum to thirty degrees, six bala groups exactly, children sum to parents exactly (`03-design/`) | **enforced** where a crate's tests carry the list: `cargo test --workspace`, fast check |
| rule fixtures | every rule has a positive and a negative fixture before it is marked stable; rules that always co-fire across the corpus are flagged as probable duplicates | **not yet enforced: taken up in the hardening pass**. `check-yogas` and `check-doshas` measure every shipped rule over the corpus, but nothing refuses a rule without both fixtures or flags co-firing pairs |
| a fixture before a fix | a defect report becomes a failing fixture before the fix lands | **enforced by review**, not by a gate |
| property tests | invariants hold for random inputs: exact partition of the circle for every divisor and varga, sums, tree consistency, `dasha_at` finds every instant, monotonic transitions, serialise-deserialise fixed points, calendar round trips; generators deliberately produce values at and one microarcsecond either side of every classification boundary, because that is where defects live | **enforced in part**: the `proptest` suites that exist run under `cargo test --workspace`, fast check. **The rest not yet enforced: taken up in the hardening pass**: the list above is not covered crate by crate, and no gate requires a boundary generator |
| snapshot tests | serialised outputs and composed text per language do not change unnoticed | **enforced** by generated-page gates rather than a snapshot library: `check-render` (golden drawings byte for byte), `check-interpret` and `check-interpretations` (composed text in every strict locale), `check-serial` and `check-schema` (the canonical form and the chart document), fast check |
| cross-binding parity | the same inputs give byte-identical canonical JSON in every binding; the generated type surfaces match the API description | **enforced**: `cargo xtask check-parity` (the bindings and the shared examples, value by value), verify; `cargo xtask check-ffi` (every binding's generated layer equals what the API description emits), fast check |
| cross-architecture determinism | the same scenario gives identical output hashes on every platform ADR-0022 names; a divergence names the section, how many values moved and by how many places | **enforced**: `cargo xtask hashes` and `compare-hashes`, `hash-matrix.yml`, nightly and on demand, failing on any platform that differs from Linux x86-64. A release does not run it |
| type safety per binding | swapped newtypes and incomplete builders do not compile (`trybuild`); a consumer project per binding type-checks at maximum strictness; one shared corpus of valid and invalid inputs agrees across every binding's validators and the Rust constructors | **enforced in part**: `crates/core/tests/compile_fail.rs` (`trybuild`) under `cargo test --workspace`, fast check; the strict consumers and the files of wrong usages in `check-node`, `check-python` and `check-dart`, verify. **Not yet enforced: taken up in the hardening pass**: a wrong-usage check for Java, and the shared validator corpus |
| provider conformance kit | every adapter and every built-in tier meets its published bound | **enforced** for the built-in tiers: verify's `ephemeris tier` job runs the kit and the crate's tests at each tier (`cargo test -p teistro-ephemeris-builtin`). **Not yet enforced: taken up in the hardening pass**: the Teimeris adapter, which no workflow builds (roadmap, Track A step 8) |
| doc examples | every example in the docs runs and prints what it claims | **enforced in part**: rustdoc examples run as doctests under `cargo test --workspace`, fast check; the shared examples run in `check-rust` and each binding's check and are compared line by line by `check-parity`, verify. **Not yet enforced: taken up in the hardening pass**: the examples in the Markdown pages and the site's guides |
| coverage floors | 90% line and branch on computation crates, 80% on binding ergonomic layers; the floor only goes up | **not yet enforced: taken up in the hardening pass** |
| mutation testing | at least 80% of mutants caught on kernel crates; a surviving mutant is a missing test | **not yet enforced: taken up in the hardening pass** |
| feature matrix | every feature builds and tests alone and with no default features; no feature silently requires another | **enforced in part**: the fast check's `families` job lints the façade, the boundary and the wasm module with no module family and with each family that shares the request parts, and runs the boundary's family test; verify's `ephemeris tier` job builds the boundary with no built-in ephemeris; `check-lints` holds that families are forwarded and that dependents name them. **Not yet enforced: taken up in the hardening pass**: every feature of every crate alone |
| public API stability | no unintended breaking change in a minor release | **not yet enforced: taken up in the hardening pass** |

### What the determinism matrix measured first

Measured 2026-09-06 over 100,236 values (the calendars, the astronomy,
the house systems and the classical model; `cargo xtask hashes`):

| pair | outcome |
|---|---|
| Linux x86-64 against Linux aarch64 | every value bit for bit the same |
| Linux aarch64 against macOS aarch64 | the calendars and the classical model bit for bit the same; 139 of the 16,060 astronomy values and 1,883 of the 58,240 house values differ, none by more than a thousand places and most by one |

The architectures agreed; the C libraries did not. The two Linux runners
share glibc and computed the same numbers on different hardware, which was
Phase 1's exit criterion. macOS is a different maths library, and the
functions the astronomy layer called rounded differently there.

How differently, measured: of the 2,022 values that differed, 1,534
differed by one place and 488 by up to a thousand; none by more. The worst
relative difference was 4.8e-14 in the house cusps and 3.7e-15 in the
astronomy, which on a cusp is under a nanodegree. The calendars and the
classical model never call those functions and agreed everywhere.

**Decided and built on 2026-09-25** (`03-design/wasm-binding.md` §2,
"Determinism first"): the SDK takes its transcendental functions from one
pure-Rust `libm` on every target (`teistro_core::math`, held by the
`uses-one-libm` lint in `check-lints`). Since then the matrix **fails** on
any platform that differs from Linux x86-64, macOS, Windows and wasm32
included, because a difference is now a defect and not a property of a C
library.

## Robustness

| gate | what it proves | status |
|---|---|---|
| fuzzing | pack parsers, the MF2 engine, blob decoders and every C ABI entry point survive arbitrary bytes, with a committed corpus, a nightly smoke run and a weekly long run | **not yet enforced: taken up in the hardening pass**. The repository has no fuzz targets |
| sanitizers | no memory errors, no undefined behaviour, no data races in the `ffi` crate and the bindings' native layers (ASan, UBSan, TSan builds, nightly) | **not yet enforced: taken up in the hardening pass**. No workflow builds with a sanitizer |
| Miri | no undefined behaviour in the safe crates' tests (`cargo miri test`, nightly) | **not yet enforced: taken up in the hardening pass** |
| input validation | every public entry point rejects out-of-range, non-finite and oversized inputs with a structured error, tested per parameter | **enforced in part**: the refusal tests written by hand in the crates (`cargo test --workspace`, fast check) and in each binding's suite (verify). **Not yet enforced: taken up in the hardening pass**: tests generated from the API description, so that no parameter goes untested |
| iteration caps | every search terminates within its cap on adversarial inputs | **enforced in part**: the cap is a context limit (`Limits::max_iterations`) and the solvers' own tests run under `cargo test --workspace`. **Not yet enforced: taken up in the hardening pass**: property tests that drive every search with adversarial inputs |
| panic-free boundary | no panic escapes the C ABI; a forced panic inside becomes `INTERNAL` | **enforced**: every entry point runs under the boundary's panic guard (`crates/ffi/src/support.rs`), whose test forces a panic and reads `INTERNAL`, under `cargo test --workspace`, fast check; `clippy::panic`, `unwrap_used` and `expect_used` refuse a panic in library code, fast check (`Lint`) |

## Performance

| gate | what it proves | status |
|---|---|---|
| micro-benchmarks per module with budgets | each operation stays within its budget (`02-architecture/09-performance-architecture.md`), with `criterion` baselines checked in and a regression beyond the noise floor failing | **not yet enforced: taken up in the hardening pass**. Several crates carry `criterion` benches, and no workflow runs them |
| instruction-count regression | a pull request does not raise the instruction count of any section of the fixed scenario by more than 3% (fail) or 1% (warn) against its base commit, both measured in the same job on the same machine; deterministic on shared runners where wall clock is not; a cost a change means to pay is accepted only by an `Instruction-cost` trailer on its commits | **enforced**: `cargo xtask bench` and `cargo xtask compare-bench` over callgrind, `benchmarks.yml`, every pull request |
| wasm size | no profile's wasm module grows past its budget, and every shipped artefact stays within 2% of the recorded figure | **enforced**: `cargo xtask check-wasm` holds each module to its budget in `05-testing/sizes.json`, and `cargo xtask check-sizes --from` holds every artefact a run built to the record, verify (nightly, on demand, every release); `check-sizes` holds `SIZES.md` to the record, fast check. A pull request is not measured, so a growth is caught the following night rather than on the pull request |
| interleaved A/B against the baseline engine | the SDK through Node is not slower than the engine it replaces on the same inputs | **not yet enforced: taken up in the hardening pass** |
| results schema | a claim is refused when it is smaller than its own spread, has no noise floor or no record of what was measured | **not yet enforced: taken up in the hardening pass** |
| FFI cost | the cost of a callback-based provider and of result marshalling per binding is measured and published, never guessed | **not yet enforced: taken up in the hardening pass**. The engine passthrough's relay was measured once by hand, and nothing republishes the figure |
| allocation counts | hot paths allocate a fixed, documented number of times per call, asserted by a counting allocator in tests, and the count does not grow with the work: a completion allocates the same for a grid of 10 cells and of 1000; a calendar conversion allocates nothing; a render parses once and the second render of a key allocates less than the first. The modules of Phase 4 (foundation, rule evaluation, dasha trees, panchanga day) join as they are built | **enforced** for `astro`, `calendar`, `dasha` and `intl`: `teistro-test-allocator` in `crates/*/tests/allocations.rs`, under `cargo test --workspace`, fast check. **Not yet enforced: taken up in the hardening pass**: the foundation, rule evaluation and the panchanga day |
| profiles on file | every optimisation is preceded by a profile and followed by a measurement; the numbers go in the commit body | **enforced by review**, not by a gate |

### What the counting allocator measured

Measured 2026-09-06 (`crates/*/tests/allocations.rs`):

| path | allocations | what they are |
|---|---:|---|
| a completion over 10 cells | 13 | the eight columns, the steps and the provider's own vectors |
| a completion over 1000 cells | 13 | the same: the columns are allocated once each, whatever the grid |
| Delta T, the obliquity, the nutation | 0 | a table and two series |
| a date in any shipped calendar, read or written | 0 | integer arithmetic over a fixed day |
| thirty Bikram Sambat dates | 0 | the same, thirty times |
| a render, first time | 126 | the message parsed and the text built |
| the same render again | 58 | the parse is cached; the text is built again |

The calendar's zero is new: reading a Bikram Sambat date allocated twice
per call for the authority and the edition of the table it came from,
which are static text. `CalendarResolution` now borrows them, so a date
on the path every chart takes for every date it shows allocates nothing.
The counting allocator found that on its first run.

## Memory

| gate | what it proves | status |
|---|---|---|
| peak memory per operation | a context, a foundation, a full chart, a month grid and a muhurta search each stay within a documented peak, profiled in a nightly job and published | **not yet enforced: taken up in the hardening pass** |
| leak checks in Rust | no leak across 10,000 iterations of every public operation, under a LeakSanitizer build and `valgrind --leak-check=full` on Linux | **not yet enforced: taken up in the hardening pass**. The only valgrind run is callgrind's, in `benchmarks.yml` |
| leak checks per binding | a binding's objects release their native memory: 10,000 create-use-drop cycles in Node (with forced GC and heap measurement), Python (`tracemalloc` and RSS), Dart and Java, with a flat RSS curve | **not yet enforced: taken up in the hardening pass**. No binding's suite measures memory |
| cache bounds | every cache respects its byte limit under sustained load | **enforced in part**: the memo's budget is a context limit (`Limits::cache_bytes`), and its tests hold that a full cache stops admitting and keeps answering (`crates/port-ephemeris/src/caching.rs`), under `cargo test --workspace`, fast check. **Not yet enforced: taken up in the hardening pass**: a test under sustained load |
| size | the binary size per profile and per platform, and the pack size per locale, stay within budget | **enforced** for the shipped artefacts: `cargo xtask check-sizes --from` and `check-wasm` against `05-testing/sizes.json`, verify (the wasm size row above). **Not yet enforced: taken up in the hardening pass**: a budget per locale's pack |
| no global state | contexts on N threads scale near-linearly and share nothing; TSan clean | **enforced in part**: `check-lints` refuses reads of the clock, the environment and the process in a computation crate, fast check, and the C ABI has no global state by design (a constructor's failure crosses as an owned record). **Not yet enforced: taken up in the hardening pass**: the thread benchmark and a TSan build |

## Code quality

| gate | what it proves | status |
|---|---|---|
| format and lint | `rustfmt` clean; `clippy` with `all` and `pedantic` as errors; in library crates no `unwrap`, `expect`, `panic`, `todo`, `dbg`, printing or slice indexing (workspace lints) | **enforced**: `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`, fast check (`checks`), with the same clippy for `wasm32-unknown-unknown` beside it |
| determinism lints | no unordered collection in a computation crate unless the file says why it is safe; no reads of the clock, the environment or the process in one; only the port, the boundary and the addon may hold unsafe code; the classification functions are `const fn`, which stable Rust cannot compute in floating point; one `libm` on every target; **every settings knob has a reader outside the settings layer**, or says at its declaration which module will read it; **every workflow file parses**; **every gate is run by a workflow**, or says on its arm that it is run by hand; **every entry point of the boundary is placed by an emitter rule**, or is declared as awaiting one; **Python is spawned in one place**, which puts it in UTF-8 mode, because every one of these programs prints Devanagari and the Windows console's default encoding cannot; **every platform row of a workflow matrix runs on the runner the platform table names**, because a retired runner label does not fail, it queues | **enforced**: `cargo xtask check-lints`, fast check |
| unsafe confinement | `#![forbid(unsafe_code)]` everywhere except `ffi`; every `unsafe` block in `ffi` carries a `SAFETY:` comment reviewed | **enforced as an inventory**: the workspace lint `unsafe_code = "forbid"`, and `check-lints`'s unsafe inventory, which refuses a crate that downgrades it without being listed, and a listed crate that no longer needs to, fast check. The list is wider than `ffi` alone: it names the ephemeris port, the MCP server, the test allocator, the test adapter, the Node and wasm addons and the adapters beside the boundary. The `SAFETY:` comments are held by review |
| documentation | every public item documented with a compiled example; no warnings from `cargo doc` | **enforced in part**: `missing_docs` is a workspace lint, an error under the fast check's clippy; every rustdoc example compiles and runs as a doctest under `cargo test --workspace`. **Not yet enforced: taken up in the hardening pass**: an example on every public item, and a `cargo doc` run with warnings as errors, which no workflow makes |
| dependencies | licences on the allow list (`deny.toml`; copyleft and MPL denied everywhere), no oracle or ephemeris adapter in a publishable crate's graph, advisories none, duplicates justified, every dependency vetted | **enforced in part**: `cargo deny check` (licences, bans, advisories from the RustSec database, sources), fast check (`Dependency policy`); the adapters sit outside the workspace, and `deny.toml` bans the Swiss Ephemeris crates. **Not yet enforced: taken up in the hardening pass**: duplicates are a warning rather than a refusal, and no dependency is vetted (`cargo-vet`) |
| containment | the workspace builds and passes its tests with the test provider only and no adapter present | **enforced**: the adapters are outside the workspace (ADR-0019), so the fast check's `cargo test --workspace` builds and tests with none; verify's `ephemeris tier` job also builds the boundary with no built-in ephemeris |
| generated artefacts | regenerated output equals the committed output -- the emitted bindings and headers, and every **measured design page**, each of which is regenerated in memory from its own sources and compared | **enforced**: `cargo xtask check-ffi` (the API description, the C header and every binding's generated layer), `check-catalogue`, `check-calendars`, `check-time`, `check-intl`, `check-accuracy`, `check-conformance`, `check-sizes`, and `check-<pass>` for each measured page, fast check; `check-lints` refuses a measured page no gate regenerates |
| gates proven red | every new gate was broken once and observed failing before it was trusted | **enforced by review**: recorded in the pull request |

### Why a knob needs a reader

A settings knob that ships, resolves and is read by nobody is a bug
whether or not anything crashes, and three were found by hand in as many
modules before the rule existed:

| knob | how it failed |
|---|---|
| `state.combustion_orbs` | **loudly**: a chart founded on the SDK's own default profile returned `UNSUPPORTED` (`01-golden-vectors.md`, entry 23) |
| `houses.module_overrides` | **quietly**: every shipped profile says `kp` takes Placidus, and the KP reading got whole-sign houses — not an error, the wrong chart |
| `output.precision` | **silently**: the knob did nothing at all |

Three in three is a pattern rather than an accident, so it is gated. The
knob list comes from `core` itself (`Settings::knob_paths`, held to the
settings document by its own test), so a group added to the document is
watched without a second list to remember.

A knob whose module is not written yet says so where it is declared,
with a `lint: knob-has-a-reader` marker naming what will read it. The
gate prints those, so a deferral is an inventory rather than a silence —
and **an allowance that is no longer needed is itself a failure**, so
the inventory cannot rot. `check-lints` prints the deferrals on every
run, so how many there are is read there rather than here.

### A gate nothing runs

The same argument holds one level up, and cost a run to learn. A gate
regenerates a page in memory and fails on any difference; it holds that
page for exactly as long as something runs it. `check-surface` was
declared in `xtask`, named in this document and named on the page it
was said to hold — and no workflow ran it, so
`03-design/binding-surface-measured.md` had been ungated since it was
written. Nothing compared the list of gates against the list of things
that run them, which is `knob-has-a-reader` with the knob one level up,
so `gate-has-a-runner` now does. A gate meant to be run by hand says so
on its arm and appears in the inventory.

Its neighbour is smaller and sharper. GitHub answers a workflow file it
cannot parse by failing the run **in zero seconds**, without starting a
step and without naming what is wrong: a step name with an unquoted
colon in it stopped every gate in the fast check at once, and the
only sign was a red tick. Nothing in the repository parses those files —
`rustc` never reads them — so `workflow-parses` does, in the pass that
they themselves run.

## The rule behind the rules

Nothing is asserted that is not measured, and nothing is measured once
that is not gated forever. The generated `ACCURACY`, `CONFORMANCE` and
`SIZES` documents are the public record of the bar being met, and the pull
request template asks for the numbers first. A generated performance and
memory record does not exist yet: it waits on the benchmark and memory
rows above, taken up in the hardening pass.
