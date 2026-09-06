# The quality bar

Status: `accepted`, 2026-09-04; extended the same day by ADR-0022 (the
determinism contract and the conformance repository) and ADR-0023 (type
safety in every binding). The maintainer's instruction: the code must be
proper, well tested, validated, benchmarked, and measured for memory,
optimisation and leaks; everything must be. This page turns that into
gates. A module is done when every row below is green for it, not when
its code compiles.

## Correctness

| gate | what it proves | tool | where it runs |
|---|---|---|---|
| unit tests with classical examples | every formula reproduces a worked example from a text or a reference | `cargo test` | fast check |
| golden-vector conformance | every P0 computation reproduces the baseline engine, PyJHora or a printout within its stated tolerance; tolerances live in one central file keyed by field and provider class, never per fixture; every fixture asserts the settings hash; every expected value cites its source | the conformance harness (`cargo xtask conformance`) over the conformance repository (ADR-0022, a pinned submodule) | fast check (subset), nightly (full) |
| whole-table invariants | every kernel table passes its invariant list in one test: exact totals, every seed maps to a lord, orders visit every sign once, spans sum to thirty degrees, six bala groups exactly, children sum to parents exactly (`03-design/`) | `cargo test` | fast check |
| rule fixtures | every rule has a positive and a negative fixture before it is marked stable; rules that always co-fire across the corpus are flagged as probable duplicates | pack validation | fast check |
| a fixture before a fix | a defect report becomes a failing fixture before the fix lands | review | every bug fix |
| property tests | invariants hold for random inputs: exact partition of the circle for every divisor and varga, sums, tree consistency, `dasha_at` finds every instant, monotonic transitions, serialise-deserialise fixed points, calendar round trips; generators deliberately produce values at and one microarcsecond either side of every classification boundary, because that is where defects live | `proptest` | fast check |
| snapshot tests | serialised outputs and composed text per language do not change unnoticed | `insta` | fast check |
| cross-binding parity | the same inputs give byte-identical canonical JSON in every binding; the generated type surfaces match the API description | parity harness | nightly, release |
| cross-architecture determinism | the same scenario gives identical output hashes on x86-64 and aarch64 Linux, and the wider matrix (macOS, Windows, Node and wasm32) is measured and published; a divergence names the section, how many values moved and by how many places | `cargo xtask hashes` and `compare-hashes`, the `hash-matrix` workflow (ADR-0022) | nightly, release |
| type safety per binding | swapped newtypes and incomplete builders do not compile (`trybuild`); a consumer project per binding type-checks at maximum strictness; one shared corpus of valid and invalid inputs agrees across every binding's validators and the Rust constructors | `trybuild`, per-binding consumer projects, the validator corpus (ADR-0023) | fast check (Rust), nightly (bindings) |
| provider conformance kit | every adapter and every built-in tier meets its published bound | `teistro` CLI | nightly, release |
| doc examples | every example in the docs runs and prints what it claims | `cargo xtask doc-examples` | fast check |
| coverage floors | 90% line and branch on computation crates, 80% on binding ergonomic layers; the floor only goes up | `cargo-llvm-cov` | nightly, reported per crate |
| mutation testing | at least 80% of mutants caught on kernel crates; a surviving mutant is a missing test | `cargo-mutants` | nightly |
| feature matrix | every feature builds and tests alone and with no default features; no feature silently requires another | `cargo-hack --each-feature`, powerset sampled | fast check (sample), nightly (full) |
| public API stability | no unintended breaking change in a minor release | `cargo-semver-checks` | fast check on release branches |

### What the determinism matrix measures today

Measured 2026-09-06 over 100,236 values (the calendars, the astronomy,
the house systems and the classical model; `cargo xtask hashes`):

| pair | outcome |
|---|---|
| Linux x86-64 against Linux aarch64 | every value bit for bit the same |
| Linux aarch64 against macOS aarch64 | the calendars and the classical model bit for bit the same; 139 of the 16,060 astronomy values and 1,883 of the 58,240 house values differ |

The architectures agree; the C libraries do not. The two Linux runners
share glibc and compute the same numbers on different hardware, which is
Phase 1's exit criterion. macOS is a different maths library, and the
functions the astronomy layer calls round differently there; the
calendars and the classical model do not call them, and agree everywhere.
The nightly run publishes how far apart the differing values are and
prints the first few of each section, so the next reader sees whether a
difference is a rounding or a formula.

So the matrix fails on an architecture difference and reports a C library
difference. Making a chart bit-identical across operating systems as well
would mean the astronomy layer carrying its own maths functions rather
than the platform's, which is a decision for its own ADR: it buys
reproducibility across platforms and costs agreement with ERFA, which
uses the platform's.

## Robustness

| gate | what it proves | tool | where it runs |
|---|---|---|---|
| fuzzing | pack parsers, the MF2 engine, blob decoders and every C ABI entry point survive arbitrary bytes | `cargo-fuzz` with a committed corpus | nightly smoke (minutes), weekly long run |
| sanitizers | no memory errors, no undefined behaviour, no data races in the `ffi` crate and the bindings' native layers | ASan, UBSan, TSan builds | nightly |
| Miri | no undefined behaviour in the safe crates' tests | `cargo miri test` | nightly |
| input validation | every public entry point rejects out-of-range, non-finite and oversized inputs with a structured error, tested per parameter | unit tests generated from the API description | fast check |
| iteration caps | every search terminates within its cap on adversarial inputs | property tests | fast check |
| panic-free boundary | no panic escapes the C ABI; a forced panic inside becomes `INTERNAL` | `ffi` tests | fast check |

## Performance

| gate | what it proves | tool | where it runs |
|---|---|---|---|
| micro-benchmarks per module with budgets | each operation stays within its budget (`02-architecture/09-performance-architecture.md`) | `criterion`, baselines checked in | nightly, regression beyond the noise floor fails |
| instruction-count regression | a pull request does not raise the instruction count of any benchmark by more than 3% (fail) or 1% (warn) against its base commit; deterministic on shared runners where wall clock is not | `iai-callgrind` | every pull request |
| wasm size delta | a pull request does not grow any profile's gzipped wasm by more than 2% without a note | `cargo xtask size` | every pull request |
| interleaved A/B against the baseline engine | the SDK through Node is not slower than the engine it replaces on the same inputs | the A/B harness | release |
| results schema | a claim is refused when it is smaller than its own spread, has no noise floor or no record of what was measured | `cargo xtask bench --check` | every benchmark run |
| FFI cost | the cost of a callback-based provider and of result marshalling per binding is measured and published, never guessed | binding benchmarks | nightly |
| allocation counts | hot paths (foundation, rule evaluation, dasha trees, panchanga day) allocate a fixed, documented number of times per call, asserted by a counting allocator in tests; `dasha_at` allocates zero times after warm-up | test-only global allocator | fast check |
| profiles on file | every optimisation is preceded by a profile and followed by a measurement; the numbers go in the commit body | `cargo flamegraph`, `perf`, `dhat` | before any optimisation lands |

## Memory

| gate | what it proves | tool | where it runs |
|---|---|---|---|
| peak memory per operation | a context, a foundation, a full chart, a month grid and a muhurta search each stay within a documented peak | `dhat`-style heap profiling in a nightly job | nightly, published |
| leak checks in Rust | no leak across 10,000 iterations of every public operation | LeakSanitizer build, `valgrind --leak-check=full` on Linux | nightly |
| leak checks per binding | a binding's objects release their native memory: 10,000 create-use-drop cycles in Node (with forced GC and heap measurement), Python (`tracemalloc` and RSS), Dart and Java, with a flat RSS curve | binding test suites | nightly |
| cache bounds | every cache respects its byte limit under sustained load | tests | fast check |
| size | the binary size per profile and per platform, and the pack size per locale, stay within budget | `cargo xtask size`, `cargo-bloat` | nightly, release |
| no global state | contexts on N threads scale near-linearly and share nothing; TSan clean | thread benchmark, TSan | nightly |

## Code quality

| gate | what it proves | tool | where it runs |
|---|---|---|---|
| format and lint | `rustfmt` clean; `clippy` with `all` and `pedantic` as errors; in library crates no `unwrap`, `expect`, `panic`, `todo`, `dbg`, printing or slice indexing (workspace lints) | `cargo fmt --check`, `cargo clippy -D warnings` | fast check |
| determinism lints | no `HashMap` in an output-producing path; no reads of the clock, the environment or the locale in computation crates; no `f64` division or `floor` in a classification path; no user-facing string literal in a computation crate | `cargo xtask check-lints` | fast check |
| unsafe confinement | `#![forbid(unsafe_code)]` everywhere except `ffi`; every `unsafe` block in `ffi` carries a `SAFETY:` comment reviewed | lint and review | fast check |
| documentation | every public item documented with a compiled example; no warnings from `cargo doc` | `cargo doc -D warnings` | fast check |
| dependencies | licences on the allow list (`deny.toml`; copyleft and MPL denied everywhere), no oracle or ephemeris adapter in a publishable crate's graph, advisories none, duplicates justified, every dependency vetted | `cargo deny check`, `cargo-audit`, `cargo-vet` | fast check (deny), weekly (audit) |
| containment | the workspace builds and passes its tests with the test provider only and no adapter present | a CI job (ADR-0019) | fast check once an adapter exists |
| generated artefacts | regenerated output equals the committed output | `cargo xtask check-generated` | fast check |
| gates proven red | every new gate was broken once and observed failing before it was trusted | recorded in the pull request | review |

## The rule behind the rules

Nothing is asserted that is not measured, and nothing is measured once
that is not gated forever. The generated `ACCURACY`, `CONFORMANCE`,
`PERFORMANCE`, `MEMORY` and `SIZES` documents are the public record of the
bar being met, and the pull request template asks for the numbers first.
