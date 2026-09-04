# Performance and latency

Status: `research`, 2026-09-04. Feeds
`02-architecture/09-performance-architecture.md`. Numbers cited are from
Teimeris's measurements of the baseline engine and its own benchmarks.

## Where the time goes today

- A full chart in the baseline engine makes 703 ephemeris calls, 593 of them one scalar
  method; the ephemeris is 8.2% of the chart. The other 92% is the baseline engine's own
  loops: rule evaluation over 624 rules, dasha trees, vargas, strengths,
  serialisation.
- Panchanga boundary finding by sample-and-bisect against the ephemeris is
  where 5–15x was measured available through a composite-angle crossing
  search.
- The N-API boundary costs about 70 ns per field read; a columnar path is
  within 5% of C.
- the baseline engine's backend spends most of its wall-clock in caching layers, HTTP and
  serialisation, not the engine (chart compute about 75 ms including
  serialisation; foundation 1–5 ms).

## Targets to set (to be measured, then gated)

| workload | target | rationale |
|---|---|---|
| chart foundation (positions, cusps, vargas, dignities, state) | under 1 ms on a laptop core with a native provider | The baseline engine's foundation is 1–5 ms including Node overhead |
| full natal chart with 624 rules, Shadbala, Ashtakavarga, 18 dasha trees to depth 3 | under 10 ms | The baseline engine about 75 ms including serialisation; the rule engine and dashas are the bulk |
| daily panchanga with all limbs and timings | under 2 ms with a crossing-capable provider | Teimeris measured 14x on the boundary search |
| 42-day month grid | under 50 ms | 42 days sequential |
| muhurta search over 90 days | under 200 ms | The baseline engine: under a second after its optimisations |
| rectification full-day search | under 1 s | The baseline engine: refinement keeps the candidate count constant |
| batch of 1,000 charts (research) | linear, parallelisable across contexts | |
| FFI: callback-based provider versus native vtable | measured ratio published, not a target | consumers choose |

## Techniques

1. **Batch and grid shapes** at the port and in the SDK's own loops
   (positions for all bodies in one call; houses for many instants).
2. **Foundation once, slices on demand** (the baseline engine's design, kept): every
   computation takes a foundation handle and computes its slice; a memo on
   the context caches slices by settings hash.
3. **Rule engine compilation**: rules compiled once per context into a
   predicate tree over an indexed chart state (bitsets of bodies per house,
   per sign, per dignity) so 624 rules evaluate in microseconds; no string
   comparison at evaluation time.
4. **Allocation discipline**: arena allocation per computation, no
   per-planet heap allocation in hot loops, results in flat arrays with
   index tables; trees as arrays of nodes with parent indices.
5. **No global state, one context per thread**, so parallelism is free
   (Teimeris measured 92–95% of ideal to four threads).
6. **Caching keyed by inputs plus settings hash**, opt-in per context, with
   bounded memory.
7. **Crossing search delegation** to the provider when capable; the SDK
   fallback uses a step bounded by the fastest body's speed and bisection
   to a stated tolerance in time, gated for agreement with the native search.
8. **Floating point policy**: no fast-math; contraction policy stated;
   deterministic across platforms to a stated tolerance, with a
   cross-platform golden test.

## Benchmark method (adopted from Teimeris)

Interleaved A/B against the baseline (the baseline engine through Node, then
the SDK through the same binding), median of repeated rounds, noise floor
reported, a benchmark that refuses to run unless both sides agree on the
answer first, and a results schema that rejects a claim smaller than its
own spread. Performance claims in docs are generated from the benchmark
output.
