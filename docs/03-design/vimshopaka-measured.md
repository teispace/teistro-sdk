# The Vimshopaka, measured

Status: `generated` by `cargo xtask vimshopaka` over the conformance
corpus's `baseline/vimshopaka`, 2026-09-15. Do not edit:
`check-vimshopaka` regenerates this page and fails on any difference.
The design it measures is [`strength-schemes.md`](strength-schemes.md).

The corpus records the recording engine's Vimshopaka for 93 charts,
computed from the seven grahas' recorded signs in the sixteen vargas,
under the shadvarga, saptavarga, dashavarga and shodashavarga schemes.
BPHS ch. 7 was read beside it.

## What the corpus decides

| proposed rule | verdict | measured |
|---|---|---|
| the four schemes' weights are BPHS ch. 7 vv. 17 to 25's, each summing to 20 | **holds** | 0 of 4 disagree |
| the engine scores a varga by exaltation, debilitation, moolatrikona and own sign by the sign alone, then by **natural** friendship with the sign's lord | **holds** | 0 of 10416 disagree |
| a scheme's score is each varga's Saptavargaja virupas over 45 times its weight, summed and rounded to hundredths | **holds** | 0 of 93 disagree; worst 0.0e0 |
| the text's: 20 in exaltation or the own sign, else 18, 15, 10, 7 or 5 by the compound relationship in the rasi chart, times the weight over 20 | falsified | 2604 of 2604 disagree; worst gap 12.88 of 20 |

## What it means for the module

**The weights are settled**: the engine's are the text's.

**The scoring is not the text's.** The engine reuses the Saptavargaja
virupas and divides by 45, so a varga in a friend's sign earns 6.67 of
20 where the text gives 15, and it takes natural friendship alone, so no
varga is ever a great friend's or a great enemy's. A rank-1 text
corrects a rank-2 value, so the module's default is the text's points by
the compound relationship and the engine's scale is a setting the
conformance profile takes. Which chart the compound relationship's
temporary half is taken in, and whether debilitation takes anything from
a varga, the text does not settle (crux C63).
