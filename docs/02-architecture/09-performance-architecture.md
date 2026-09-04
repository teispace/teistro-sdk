# Performance architecture

Status: `draft`, 2026-09-04.

## Principles in code

1. **Foundation once.** `chart::Foundation` memoised on the context keyed
   by (birth data hash, settings hash); slices keyed by (foundation hash,
   slice options); bounded memory with LRU.
2. **Indexed chart state.** After the foundation, an index is built once:
   bitsets of bodies per house, sign, nakshatra, dignity and state;
   lordship tables; aspect matrices. Rule predicates are compiled to
   operations over these bitsets; 624 rules should evaluate in tens of
   microseconds.
3. **Trees as arrays.** Dasha trees and rule results are arrays of nodes
   with parent indices and level markers; no per-node allocation; walking
   the active chain is index arithmetic.
4. **Batch at the port.** One `positions` grid per foundation (all bodies,
   one instant), one `houses_many` for candidate grids, one `crossings`
   per limb per day; the fallback searches are step-bounded.
5. **Arena allocation per request** with a reset; results copied out into
   caller buffers or blobs.
6. **No strings in hot paths.** Keys are integers until serialisation.
7. **Contexts per thread**, pools in bindings, no locks in the core.
8. **Floating point policy** fixed and documented; a cross-platform golden
   test asserts agreement to the stated tolerance.

## Budgets (proposed, to be measured then gated)

| operation | budget (native provider, laptop core) |
|---|---|
| foundation | 1 ms |
| full parity chart (all P0 slices, dashas to depth 3, 624 rules) | 10 ms |
| panchanga day | 2 ms with native crossings, 10 ms with the fallback |
| month grid (42 days) | 60 ms |
| muhurta search (90 days) | 200 ms |
| rectification (full-day window) | 1 s |
| N-API full chart round trip including blob decode | within 1.5x of the C path |
| wasm full chart | within 2x of native |

## Benchmark harness

Interleaved A/B against the baseline engine on the same inputs through the Node
binding, criterion micro-benchmarks per module, a results schema that
refuses claims smaller than their spread, and docs numbers generated from
the run. Baselines are checked in; a regression beyond the noise floor
fails the nightly.
