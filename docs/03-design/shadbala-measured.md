# The Shadbala, measured

Status: `generated` by `cargo xtask shadbala` over the conformance
corpus's `baseline/shadbala`, 2026-09-15. Do not edit: `check-shadbala`
regenerates this page and fails on any difference. The design it
measures is [`strength-schemes.md`](strength-schemes.md).

The corpus records the recording engine's Shadbala for 71 charts, every
component of each of the seven grahas, computed from the recorded chart,
day and houses the files repeat. BPHS ch. 27 was read beside it.

## The engine's reading, component by component

Each rule is reproduced from the inputs alone and compared with every
recorded cell, within 1e-9 of a virupa.

| proposed rule | verdict | measured |
|---|---|---|
| `sthana.uccha` — Uchcha: a third of the arc from the debilitation point (BPHS v. 1) | **holds** | 0 of 497 disagree; worst 1.4e-14 |
| `sthana.saptavargiya` — Saptavargaja: the Saptavargaja virupas of the seven vargas, 45 in exaltation, 30 in moolatrikona or the own sign, and 15, 7.5 or 3.75 by **natural** friendship, 0 in debilitation | **holds** | 0 of 497 disagree; worst 0.0e0 |
| `sthana.ojayugma` — Ojayugma: 15 each for an odd rasi and navamsha, even for the Moon and Venus (v. 4½) | **holds** | 0 of 497 disagree; worst 0.0e0 |
| `sthana.kendra` — Kendradi: 60, 30 or 15 in a kendra, panaphara or apoklima house (v. 5) | **holds** | 0 of 497 disagree; worst 0.0e0 |
| `sthana.drekkana` — Drekkana: 15 to a male graha in the first decanate, a female in the second, a neuter in the third (v. 6) | **holds** | 0 of 497 disagree; worst 0.0e0 |
| `dig` — Dig: a third of the arc from the powerless kendra, the kendras projected from the lagna's degree 30° a house | **holds** | 0 of 497 disagree; worst 2.1e-14 |
| `kaala.nathonnatha` — Nathonnatha: Mercury 60; the Sun, Jupiter and Venus by day, and the Moon, Mars and Saturn by night, rising from 0 at the arc's ends to 60 at its middle, measuring a night from the day's own sunset | **holds** | 0 of 497 disagree; worst 0.0e0 |
| `kaala.paksha` — Paksha: a third of the Moon's elongation for the Moon, Mercury, Jupiter and Venus, 60 less that for the rest, the Moon's doubled | **holds** | 0 of 497 disagree; worst 4.3e-14 |
| `kaala.tribhaga` — Tribhaga: 60 to the lord of the third of the day or night (Mercury, Sun, Saturn; Moon, Venus, Mars), measuring a night from the day's own sunset, and always to Jupiter | **holds** | 0 of 497 disagree; worst 0.0e0 |
| `kaala.vara` — Vara: 45 to the weekday's lord | **holds** | 0 of 497 disagree; worst 0.0e0 |
| `kaala.hora` — Hora: 60 to the recorded hora lord | **holds** | 0 of 497 disagree; worst 0.0e0 |
| `kaala.abda` — Abda: 15 to the recorded year lord (the weekday of Mesha sankranti) | **holds** | 0 of 497 disagree; worst 0.0e0 |
| `kaala.masa` — Masa: 30 to the lord of the Sun's sign | **holds** | 0 of 497 disagree; worst 0.0e0 |
| `kaala.ayana` — Ayana: 60 × (ε ± δ) / 2ε with ε = 23.4393° and δ the declination of the tropical longitude at zero latitude; north gains for the Sun, Mars, Jupiter and Venus, south for the Moon and Saturn, either for Mercury; the Sun's 0 | **holds** | 0 of 497 disagree; worst 2.8e-14 |
| `cheshta` — Cheshta: the Sun's Ayana, a third of the Moon's elongation, and for the rest a third of the seeghra kendra from the engine's J2000 mean elements | **holds** | 0 of 497 disagree; worst 3.5e-12 |
| `naisargika` — Naisargika: multiples of 60/7 from Saturn to the Sun, rounded to hundredths | **holds** | 0 of 497 disagree; worst 0.0e0 |
| `drik` — Drik: 60 for each full whole-sign glance on the graha's house (7th; Mars 4th and 8th, Jupiter 5th and 9th, Saturn 3rd and 10th; else ¾, ½ and ¼), added from the Moon, Jupiter and Venus and taken away from the rest, Mercury included, within ±60 | **holds** | 0 of 497 disagree; worst 0.0e0 |
| the whole: the six strengths' sum, Ayana inside Kaala, no Yuddha | **holds** | 0 of 497 disagree; worst 3.5e-12 |

## The text's reading where the inputs decide it

Each is compared with the engine's reproduction; a cell differs when the
two stand more than half a virupa apart, the natural strengths when they
differ at all.

| proposed rule | verdict | measured |
|---|---|---|
| Saptavargaja by the compound relationship (the moolatrikona rasi 45, own 30, 22.5, 15, 7.5, 3.75, 1.875), exaltation not counted (B.V. Raman, after Sripati) | falsified | 477 of 497 disagree; worst gap 135.00 |
| Nathonnatha from midnight at any hour: the night grahas twice the nata in ghatis, the day grahas 60 less that (vv. 8 and 9) | falsified | 402 of 497 disagree; worst gap 58.90 |
| a birth before sunrise is measured in the night it falls in | falsified | the engine measures 24 of 71 charts' nights from the day's own sunset, after the birth: every night graha's Nathonnatha and the Tribhaga lord are lost |
| the Sun's Ayana counted in Kaala, doubled (v. 17) | falsified | 68 of 497 disagree; worst gap 120.00 |
| the Moon's Cheshta is her Paksha (v. 18) | falsified | 70 of 497 disagree; worst gap 57.36 |
| Naisargika exactly one seventh of a rupa times 1 to 7 (v. 14) | falsified | 426 of 497 disagree; worst gap 0.0043 |
| Dig from the true kendras — the ascendant, the nadir, the descendant and the midheaven (v. 7) | falsified | 230 of 497 disagree; worst gap 22.17 |
| Mercury's glance added rather than taken away (v. 19 adds Mercury's and Jupiter's in full) | falsified | 147 of 497 disagree; worst gap 105.00 |
| the ahargana from Burgess's figure for 1 January 1860, taken for the Hindu day, falls on the recorded weekday (v. 13) | falsified | 4 of 71 disagree; each of `c022-honolulu-1941-12-07`, `c027-reykjavik-1975-06-21`, `c039-mexico-city-1985-09-19`, `c043-fairbanks-2015-06-21` is a birth the engine placed in the wrong Hindu day, its recorded sunrise a day early or its pre-sunrise night given the next day's weekday |
| the Abda lord is the weekday lord of the ahargana's 360-day year: its completed years to and including the day, times 3, from Sunday (v. 13) | falsified | 62 of 71 disagree |
| the Masa lord is the weekday lord of the ahargana's 30-day month: its completed months to and including the day, times 2, from Sunday (v. 13) | falsified | 67 of 71 disagree |
| BPHS vv. 32 and 33's requirements, 390, 360, 300, 420, 390, 330 and 300 virupas, the Sun's 6.5 rupas where the engine asks 5 | falsified | 22 of 497 disagree |

## What it means for the module

**The engine's arithmetic is settled**: every component reproduces from
the recorded inputs, so the conformance profile can take it whole.

**The text is not the engine**, and a rank-1 text corrects a rank-2
value, so the module's defaults follow BPHS wherever the chapter
decides, each fork a setting with the engine's reading as its other
value: the Saptavargaja by the compound relationship (C64), Nathonnatha
from midnight and a pre-dawn birth's night from the previous evening
(C65), the Sun's Ayana doubled, the Moon's Cheshta her Paksha and the
kranti from the ephemeris (C66), the Abda and Masa lords from the
ahargana (C67), Dig from the true angles (C68), Drik by the chapter's
quarters (C69), and the chapter's requirements and exact natural
strengths (C71).

**A third reading settles what the chapter leaves open.** B.V. Raman's
*Graha and Bhava Balas* works Sripati's method on one horoscope number
by number: the sphuta drishti Drik reads (C45), the benefics by the
Moon's phase and Mercury's company (C69), Kedarnath Dutt's mean elements
for the Cheshta and the Yuddha bala (C70), and a male, neuter, female
order for the Drekkana (C72). The module ships it as the `SRIPATI`
reading, reproduces his worked Shadbala within his rounding
(`crates/strength/tests/sripati.rs`), and takes its Cheshta elements and
Yuddha rule for the chapter's reading too, which gives neither.
