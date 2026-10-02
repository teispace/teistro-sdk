# Accidental fortitudes, measured

Status: `generated` by `cargo xtask fortitudes` from the corpus's
recorded births, 2026-10-02. Do not edit: `check-fortitudes` regenerates
this page and fails on any difference.

Lilly's accidental fortitudes (`essential-dignities.md` §Accidental
fortitudes) are held to two of his printed figures by the unit tests of
`crates/hellenistic`. What those figures cannot decide is how the
shipped readings fall on other skies, so this page counts it. It reads
each of the corpus's 55 births in the tropical zodiac, Lilly's, through
`ChartArea::fortitudes` under the default request (Regiomontanus houses,
Lilly's orbs and scores), and again under the rival each crux weighed.

| proposed rule | verdict | measured |
|---|---|---|
| every planet but the Sun stands in exactly one relation to him (cazimi, combust, under the beams or free), and the Sun in none | **holds** | 0 of 385 disagree |
| the five-degree rule moves a planet into the next house or leaves it (p. 33) | **holds** | 0 of 385 disagree; 75 planets moved by it |
| C214: the text's nearest cusp, before or after it, places every planet as "five degrees before the cusp" does | **holds** | 0 of 385 disagree; they part only in a house narrower than the two orbs together |
| C211: what the sign clause decides — planets within 8°30′ of the Sun but in another sign, read under the beams rather than combust | **holds** | 30 combust in the Sun's sign; 3 more would be combust whatever the sign |
| C212: how many planets the stated beams (17°) take, which the Chapter XXVIII tally never scores | **holds** | 41 planets under the beams |
| C216: partile lines read by the same degree, against within a degree of the exact aspect | **holds** | 17 by the same degree; 33 within a degree |

## How often each line holds

| line | planets |
|---|---|
| `Direct` | 229 |
| `Retrograde` | 46 |
| `Swift` | 275 |
| `Slow` | 110 |
| `Oriental` | 146 |
| `Occidental` | 129 |
| `Increasing` | 28 |
| `Decreasing` | 27 |
| `FreeFromCombustion` | 257 |
| `Cazimi` | 2 |
| `Combust` | 30 |
| `UnderBeams` | 41 |
| `ConjunctBenefic` | 3 |
| `ConjunctNorthNode` | 1 |
| `TrineBenefic` | 2 |
| `SextileBenefic` | 5 |
| `ConjunctMalefic` | 2 |
| `ConjunctSouthNode` | 1 |
| `OpposedMalefic` | 0 |
| `SquareMalefic` | 3 |
| `Besieged` | 2 |
| `Regulus` | 8 |
| `Spica` | 4 |
| `Algol` | 9 |

## What it means

The first three rows hold the shipped rules to the shape the text gives
them. The last three weigh each crux's rival on these skies: their
counts are how many lines the choice moves, and each rival is a knob of
`AccidentalRules` for a reader who decides the other way.
