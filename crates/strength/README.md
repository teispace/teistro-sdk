# `teistro-strength`

Status: `building`, 2026-09-15: the Ashtakavarga and the Vimshopaka built and
measured. The design is [`docs/03-design/strength-schemes.md`](../../docs/03-design/strength-schemes.md),
measured in [`docs/03-design/ashtakavarga-measured.md`](../../docs/03-design/ashtakavarga-measured.md)
and [`docs/03-design/vimshopaka-measured.md`](../../docs/03-design/vimshopaka-measured.md).

Strength measures over a chart.

| module | what it settles |
|---|---|
| [`ashtakavarga`](src/ashtakavarga.rs) | each graha's bindus by sign from BPHS ch. 66's tables, the sarvashtakavarga, the trine and Ekadhipatya reductions and the pindas, under BPHS chs. 67 to 69 or the conformance corpus's engine's reading |
| [`vimshopaka`](src/vimshopaka.rs) | each graha's strength out of 20 across the sixteen vargas under the shadvarga, saptavarga, dashavarga and shodashavarga, with BPHS ch. 7's weights, by the text's points or the conformance corpus's engine's virupas |

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
- **The Vimshopaka's weights are the text's; its points are not.** The engine
  scores a varga by the Saptavargaja virupas over 45 and natural friendship
  alone; the text by 20, 18, 15, 10, 7 or 5 and the compound relationship.
  `strength.vimshopaka` chooses (`BPHS`, the default, or
  `SAPTAVARGAJA_VIRUPAS`), crux C63.

## What proves it

- The engine's reading reproduces every recorded Ashtakavarga of the corpus,
  bindus, sums, reductions and pindas, on all 77 charts
  (`tests/baseline.rs`).
- The engine's reading reproduces every recorded Vimshopaka, all four scores
  of all seven grahas, on all 93 charts (`tests/baseline.rs`).
- Every chart from every lagna holds the classical totals; the text's
  Ekadhipatya rules each have a case; each reading's sums are its own
  reductions' (unit tests).
- Every scheme weighs 20; a graha exalted in every varga scores 20 under both
  readings; a friend's varga is a third of the text's under the engine, and
  the rasi chart decides the text's temporary half (unit tests).
