# Gochar, measured

Status: `generated` by `cargo xtask gochar`. Do not edit: `check-gochar`
regenerates this page and fails on any difference.

The design this measures is `gochar.md`. **Nothing records a transit's
verdict** — the corpus has no gochar and PyJHora no vedha — so the
table is held cell by cell by the crate's tests, and this page measures
what it does over the real sky: how the verdicts fall, how many each
fork moves, and who obstructs whom.

## 1. Over the recorded births

Each of the 55 recorded births, founded under `conformance-baseline`, read
through `sdk.chart().gochar` on the first of each month of 2026:
660 readings of nine grahas, counted from the natal Moon (v. 1).

| graha | good | obstructed | not good | obstructed, of the good houses |
|---|---:|---:|---:|---:|
| Sun | 136 | 84 | 440 | 38.2% |
| Moon | 155 | 190 | 315 | 55.1% |
| Mars | 99 | 68 | 493 | 40.7% |
| Mercury | 197 | 130 | 333 | 39.8% |
| Jupiter | 102 | 185 | 373 | 64.5% |
| Venus | 261 | 249 | 150 | 48.8% |
| Saturn | 68 | 40 | 552 | 37.0% |
| Rahu | 136 | 68 | 456 | 33.3% |
| Ketu | 171 | 57 | 432 | 25.0% |

What each fork moves, over the same readings:

| counted or read otherwise | verdicts moved, of 5940 |
|---|---:|
| from the lagna instead of the Moon (C139) | 3146 |
| `node_vedha = NONE` (C136) | 125 |
| `node_obstruction = EACH_OTHER_TOO` (C140) | 307 |
| `node_obstruction = NONE` (C137) | 238 |

## 2. Who obstructs whom, over sixty years of sky

Every day from 1960 to 2020 (21 915 days), each read from all twelve
reference signs: 262 980 readings. The verdicts:

| graha | good | obstructed | not good | obstructed, of the good houses |
|---|---:|---:|---:|---:|
| Sun | 60 252 | 27 408 | 175 320 | 31.3% |
| Moon | 73 511 | 57 979 | 131 490 | 44.1% |
| Mars | 35 813 | 29 932 | 197 235 | 45.5% |
| Mercury | 70 119 | 61 371 | 131 490 | 46.7% |
| Jupiter | 59 060 | 50 515 | 153 405 | 46.1% |
| Venus | 116 077 | 81 158 | 65 745 | 41.1% |
| Saturn | 36 533 | 29 212 | 197 235 | 44.4% |
| Rahu | 51 164 | 36 496 | 175 320 | 41.6% |
| Ketu | 51 720 | 35 940 | 175 320 | 41.0% |

Each cell counts the times the column's graha **obstructed** the row's,
standing in its vedha house while the row's graha stood in a good house.
`spared` is a pair that stood there and never obstructed, with the times
it stood; `never` is a pair the sky never put there.

| obstructed ↓ by → | Sun | Moon | Mars | Mercury | Jupiter | Venus | Saturn | Rahu | Ketu |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Sun | — | 7132 | 2668 | never | 6356 | never | spared (6600) | 7352 | 7240 |
| Moon | 10 879 | — | 10 993 | spared (10 948) | 10 888 | 10 926 | 10 958 | 10 989 | 10 963 |
| Mars | 4280 | 5477 | — | 4549 | 5940 | 4531 | 5519 | 5759 | 5917 |
| Mercury | 10 697 | spared (11 020) | 12 893 | — | 11 295 | 13 602 | 11 229 | 9874 | 11 740 |
| Jupiter | 9581 | 9228 | 8741 | 9553 | — | 9340 | 8602 | 9114 | 9269 |
| Venus | 3916 | 16 399 | 12 415 | 5573 | 15 889 | — | 15 836 | 16 291 | 16 428 |
| Saturn | spared (5308) | 5482 | 5519 | 5182 | 5375 | 5263 | — | 5856 | 5477 |
| Rahu | 7352 | 7352 | 7340 | 6796 | 7180 | 6808 | 8616 | — | spared (87 660) |
| Ketu | 7240 | 7336 | 7972 | 7748 | 6948 | 7528 | 7100 | spared (87 660) | — |

The pairs spared, as the verses name them:

- Sun by Saturn: v. 3: no vedha between father and son
- Saturn by Sun: v. 5: the Sun does not obstruct Saturn
- Moon by Mercury: v. 4: Mercury does not obstruct the Moon
- Mercury by Moon: v. 6: the Moon does not obstruct Mercury
- Rahu by Ketu: C140: the nodes do not obstruct each other
- Ketu by Rahu: C140: the nodes do not obstruct each other

The pairs the sky never puts there, each by the elongation it never
exceeds against every vedha offset of the obstructed graha:

- Sun by Mercury: Mercury stays within 28° of the Sun
- Sun by Venus: Venus stays within 47° of the Sun

## 3. The nodes read literally (C140)

Read with every graha obstructing, the nodes each other too, a node
stood in a good house 175 320 times over the sky, and the other node
obstructed it 175 320 times: the nodes always stand opposite and the
Sun's vedha pairs, theirs under `LIKE_THE_SUN`, are opposite houses, so
the literal reading leaves v. 2's good houses for the nodes never good.
The default spares them each other.

## 4. What this pass decides

| proposed rule | verdict | measured |
|---|---|---|
| a pair is spared exactly where the verses or C140 name it | **holds** | 0 of 72 disagree |
| a pair never stands in the other's vedha house exactly where an elongation bound forbids it | **holds** | 0 of 72 disagree |
| a graha standing in another's vedha house obstructs it every time or never | **holds** | 0 of 72 disagree |
| under the literal reading the other node obstructs every node in a good house | **holds** | 0 of 175320 disagree |

None of the four claims is falsified. The first three hold every ordered
pair of grahas to one of three lists written in this pass and not read
back from the code, both ways; the fourth is the measurement C140's
default rests on.
