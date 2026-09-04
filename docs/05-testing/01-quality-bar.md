# The quality bar

Status: `accepted`, 2026-09-04 (the maintainer's instruction: the code must
be proper, well tested, validated, benchmarked, and measured for memory,
optimisation and leaks; everything must be). This page turns that into
gates. A module is done when every row below is green for it, not when
its code compiles.

## Correctness

| gate | what it proves | tool | where it runs |
|---|---|---|---|
| unit tests with classical examples | every formula reproduces a worked example from a text or a reference | `cargo test` | fast check |
| golden-vector conformance | every P0 computation reproduces the baseline engine, PyJHora or a printout within its stated tolerance | the conformance harness (`cargo xtask conformance`) | fast check (subset), nightly (full) |
| property tests | invariants hold for random inputs: normalised angles, sums, tree consistency, monotonic transitions, determinism | `proptest` | fast check |
| snapshot tests | serialised outputs and composed text per language do not change unnoticed | `insta` | fast check |
| cross-binding parity | the same inputs give byte-identical canonical JSON in every binding | parity harness | nightly, release |
| provider conformance kit | every adapter and every built-in tier meets its published bound | `teistro` CLI | nightly, release |
| doc examples | every example in the docs runs and prints what it claims | `cargo xtask doc-examples` | fast check |
| coverage floors | 90% line and branch on computation crates, 80% on binding ergonomic layers; the floor only goes up | `cargo-llvm-cov` | nightly, reported per crate |

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
| interleaved A/B against the baseline engine | the SDK through Node is not slower than the engine it replaces on the same inputs | the A/B harness | release |
| results schema | a claim is refused when it is smaller than its own spread, has no noise floor or no record of what was measured | `cargo xtask bench --check` | every benchmark run |
| FFI cost | the cost of a callback-based provider and of result marshalling per binding is measured and published, never guessed | binding benchmarks | nightly |
| allocation counts | hot paths (foundation, rule evaluation, dasha trees, panchanga day) allocate a fixed, documented number of times per call, asserted by a counting allocator in tests | test-only global allocator | fast check |
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
| format and lint | `rustfmt` clean; `clippy` with `all` and `pedantic` as errors | `cargo fmt --check`, `cargo clippy -D warnings` | fast check |
| unsafe confinement | `#![forbid(unsafe_code)]` everywhere except `ffi`; every `unsafe` block in `ffi` carries a `SAFETY:` comment reviewed | lint and review | fast check |
| documentation | every public item documented with a compiled example; no warnings from `cargo doc` | `cargo doc -D warnings` | fast check |
| dependencies | licences allowed, advisories none, duplicates justified, every dependency vetted | `cargo-deny`, `cargo-audit`, `cargo-vet` | fast check (deny), weekly (audit) |
| generated artefacts | regenerated output equals the committed output | `cargo xtask check-generated` | fast check |
| gates proven red | every new gate was broken once and observed failing before it was trusted | recorded in the pull request | review |

## The rule behind the rules

Nothing is asserted that is not measured, and nothing is measured once
that is not gated forever. The generated `ACCURACY`, `CONFORMANCE`,
`PERFORMANCE`, `MEMORY` and `SIZES` documents are the public record of the
bar being met, and the pull request template asks for the numbers first.
