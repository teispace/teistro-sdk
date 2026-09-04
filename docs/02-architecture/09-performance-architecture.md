# Performance architecture

Status: `draft`, revised 2026-09-04 (ADR-0017 lazy cursor, ADR-0022
instruction-count gating).

## Principles in code

1. **Foundation once.** `chart::Foundation` memoised on the context keyed
   by (birth data hash, settings hash); slices keyed by (foundation hash,
   slice options); bounded memory with LRU.
2. **Indexed chart state.** After the foundation, an index is built once:
   bitsets of bodies per house, sign, nakshatra, dignity and state;
   lordship tables; aspect matrices. Rule predicates are compiled to
   operations over these bitsets; 624 rules should evaluate in tens of
   microseconds.
3. **Trees are lazy; materialised trees are arrays.** The dasha layer is
   a cursor over `roots` and `children`: `dasha_at(instant, depth)` walks
   one branch in O(children × depth) with zero allocations after warm-up,
   range iteration prunes subtrees, and a materialised tree takes an
   explicit depth and window and is an array of nodes with parent indices
   (`03-design/dasha-kernels.md`). The baseline engine defaults to depth 3
   because it materialises; here depth is free.
4. **Batch at the port.** One `positions` grid per foundation (all bodies,
   one instant), one `houses_many` for candidate grids, one `crossings`
   per limb per day; the fallback searches are step-bounded.
5. **Arena allocation per request** with a reset; results copied out into
   caller buffers or blobs.
6. **No strings in hot paths.** Keys are integers until serialisation.
7. **Contexts per thread**, pools in bindings, no locks in the core.
8. **Floating point policy** fixed and documented; classification is
   integer arithmetic (ADR-0016); the cross-architecture matrix compares
   output hashes, never tolerances (ADR-0022).
9. **Incremental recompute for the rectification stepper.** Foundation
   intermediates are tagged by what they depend on (time, place,
   settings, nothing); `with_time_delta(seconds)` returns a context sharing
   everything time-invariant, recomputes cusps and the Moon, reuses the
   slow bodies within tolerance and the day's sunrise and ayanamsha, and
   re-evaluates only the rules whose inputs moved. Target: under one
   millisecond per one-second step, leaving the rest of a frame to the
   consumer.

## Budgets (proposed, to be measured then gated)

| operation | budget (native provider, laptop core) |
|---|---|
| foundation | 1 ms |
| `dasha_at(instant, depth 5)` | 20 µs, zero allocations |
| one varga chart; all standard vargas | 5 µs; 100 µs |
| 900 rules | 2 ms |
| rectification step (Δt = 1 s) | 1 ms |
| full parity chart (all P0 slices, dashas to depth 3, 624 rules) | 10 ms |
| panchanga day | 2 ms with native crossings, 10 ms with the fallback |
| month grid (42 days) | 60 ms |
| muhurta search (90 days) | 200 ms |
| rectification (full-day window) | 1 s |
| N-API full chart round trip including blob decode | within 1.5x of the C path |
| wasm full chart | within 2x of native |

## Size budgets per profile (gzipped wasm, gated)

| profile | modules | budget |
|---|---|---|
| `calendar` | core, calendar, intl (date namespaces) | 60 KB |
| `panchanga` | plus astro, ephemeris-builtin (Sun and Moon, standard tier), time, panchanga | 350 KB |
| `kundali` | plus chart, houses, vargas, state, aspect, points, strength, dasha, rules, jaimini, interpret | 1.5 MB |
| `full` | everything | 3 MB |

Dependency assertions back the budgets: `panchanga` never depends on
`chart`, `numerology` on nothing astronomical, calendars individually
selectable.

## Memory budgets (gated)

| item | budget |
|---|---|
| a context without ephemeris data | 2 MB |
| a foundation with full memoisation | 256 KB |
| peak during a full chart | 4 MB |
| allocations per full chart | 5 000 |
| allocations per `dasha_at` after warm-up | 0 |

## Benchmark harness

Two harnesses, deliberately: `criterion` for wall-clock and allocations
(what users feel; noisy on shared runners) and `iai-callgrind` for
instruction counts (deterministic; what a pull request is gated on: fail
above 3%, warn above 1%). Interleaved A/B against the baseline engine on
the same inputs through the Node binding, a results schema that refuses
claims smaller than their spread, and docs numbers generated from the
run. Baselines are checked in; a wall-clock regression beyond the noise
floor fails the nightly. Findings from a profile are recorded in
`docs/05-testing/perf/` so an investigation is not repeated.
