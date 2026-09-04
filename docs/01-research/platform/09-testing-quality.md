# Testing and quality

Status: `research`, 2026-09-04. Feeds `05-testing/`. The model is Teimeris:
a conformance oracle, golden vectors, generated accuracy documents, gates
proven red, and parity across bindings.

## The oracles

| oracle | what it provides | how it is used |
|---|---|---|
| baseline engine (the existing TypeScript packages) | every P0 computation on a regression set of charts, days and matches | exported once as golden vectors (JSON with settings and versions); the SDK must reproduce them within stated tolerances; differences are recorded as deliberate with a reason (defect fixed, variant renamed) |
| PyJHora | independent implementation of most Vedic features with 5,600 tests | second opinion for dashas, vargas, chakras and balas; used to adjudicate when the SDK and the baseline engine differ |
| JHora and Parashara's Light exports | printouts for a fixed set of charts | manual golden values for the classical calculations (Shadbala, Ashtakavarga, dashas) |
| Teimeris | positions, houses, crossings, rise and set | the provider conformance kit's reference; the SDK's own fallbacks (rise/set solver, sample-and-bisect crossings) are measured against it |
| classical texts | formulas and tables | citations on rules and tables; hand-computed examples from the texts as unit tests |

## Test kinds

| kind | tool (Rust) | scope |
|---|---|---|
| unit tests per module with hand-computed classical examples | `cargo test` | every formula |
| golden-vector conformance | a harness reading the exported vectors, tolerance per field | every P0 computation |
| property-based tests | `proptest` | invariants: longitudes normalised, sums (Ashtakavarga totals 337), tree consistency (dasha children span the parent), determinism, monotonic transitions |
| snapshot tests | `insta` | serialised outputs, composed text per language |
| cross-binding parity | a canonical JSON emitted by every binding for the same inputs, diffed byte for byte | every binding, every release |
| provider conformance kit | fixed instants and expected values per precision profile | every adapter |
| fuzzing | `cargo-fuzz` | pack parsers, MF2 engine, C ABI entry points, JSON decoders |
| sanitizers and Miri | CI configurations | FFI crate and safe crates |
| benchmarks | `criterion` and an interleaved A/B harness against the baseline engine | performance claims |
| docs examples | a runner that executes every example in the docs | documentation |
| install checks | pack, install elsewhere, run | every binding package |
| size gates | per profile per platform | modularity claims |

## Coverage policy

- Line and branch coverage measured (`cargo-llvm-cov`) and published per
  module; a floor per module (proposed 90% for computation crates, 80% for
  bindings' ergonomic layers), enforced as a gate that can only go up.
- Every rule in a rule pack has at least one positive and one negative
  chart in the golden set (624 rules means at least 1,248 cases, generated
  from the baseline engine's corpus by searching its regression charts).
- Every locale pack key is exercised by the pack validator; every composer
  has a snapshot per language.

## Accuracy and conformance documents

Generated from the test run, never written by hand: per-computation
tolerance and worst observed difference against each oracle, per-provider
conformance results, per-binding parity results, per-profile sizes. This is
`ACCURACY.md` and `CONFORMANCE.md` in Teimeris, adopted whole.

## Gates proven red

Every new check is broken deliberately once and the failure observed before
it is trusted (Teimeris rule 4). The CI configuration keeps a record of the
red run for each gate.
