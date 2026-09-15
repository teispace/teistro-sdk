# `teistro-strength`

Status: `building`, 2026-09-15: the Ashtakavarga built and measured. The design
is [`docs/03-design/strength-schemes.md`](../../docs/03-design/strength-schemes.md),
measured in [`docs/03-design/ashtakavarga-measured.md`](../../docs/03-design/ashtakavarga-measured.md).

Strength measures over a chart.

| module | what it settles |
|---|---|
| [`ashtakavarga`](src/ashtakavarga.rs) | each graha's bindus by sign from BPHS ch. 66's tables, the sarvashtakavarga, the trine and Ekadhipatya reductions and the pindas, under BPHS chs. 67 to 69 or the conformance corpus's engine's reading |

## What the corpus and the text settled

- **The bindu tables are the text's**, and every chart holds 337.
- **The engine reduces the sum; the text reduces each graha's own
  Ashtakavarga.** `strength.shodhana` chooses (`EACH_GRAHA`, the default, or
  `SARVA`), crux C59.
- **The text keeps a difference the engine zeroes** when an empty co-ruled
  sign has more bindus than the occupied one. `strength.ekadhipatya` chooses
  (`BPHS`, the default, or `EMPTY_TO_ZERO`), crux C60.
- **Virgo's measure is 6 in the text and 8 in the engine**, crux C61; each
  reading uses its own.

## What proves it

- The engine's reading reproduces every recorded Ashtakavarga of the corpus,
  bindus, sums, reductions and pindas, on all 77 charts
  (`tests/baseline.rs`).
- Every chart from every lagna holds the classical totals; the text's
  Ekadhipatya rules each have a case; each reading's sums are its own
  reductions' (unit tests).
