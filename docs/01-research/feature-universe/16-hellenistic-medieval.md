# Hellenistic and medieval astrology

Status: `research`, 2026-09-04. Checked against Delphic Oracle (Robert
Schmidt's reconstruction; time lords from Valens, Antiochus, Hephaistio,
Dorotheus, Porphyry, Rhetorius), Solar Fire's traditional features (Lilly
horary, dignity tables, almutens, firdaria, profections, zodiacal releasing,
primary directions) and the Tajika page, since Tajika is the Indian
reception of the same Perso-Arabic material.

| feature | inputs | variants | field | tier | module |
|---|---|---|---|---|---|
| sect (diurnal or nocturnal chart; planets in sect and contrary; sect light) | Sun above horizon | | Solar Fire, Delphic Oracle | P1 | `hellenistic` |
| essential dignities: domicile, exaltation, triplicity (Dorothean, Ptolemaic, Lilly), terms (Egyptian, Ptolemaic, Chaldean), faces (decans); scores and almutens (of a degree, of a house, of the chart, Ibn Ezra) | tables | | Solar Fire | P1 | `dignity` (tables) |
| accidental dignities and debilities (angularity, joy, combustion, cazimi, under the beams, oriental, occidental, hayz) | | | Solar Fire | P1 | `state` |
| lots (Arabic parts) with Hellenistic day/night reversal: Fortune, Spirit, Eros, Necessity, Courage, Victory, Nemesis, and the extended set | | | Delphic Oracle, Solar Fire | P1 | `western.parts` |
| whole-sign houses, Porphyry, Alcabitius, Regiomontanus as period-appropriate systems | | | | P0 (houses exist) | `houses` |
| annual profections (from lagna, from lots, monthly and daily profections) | | | Solar Fire, Delphic Oracle | P1 | `timelords` |
| zodiacal releasing from Fortune and Spirit (Valens): periods, sub-periods, loosing of the bond, peak periods, angular from Fortune | lots | | Solar Fire, Delphic Oracle | P1 | `timelords` |
| firdaria (Schoener, nodal variants; day and night order) | | | Solar Fire | P1 | `timelords` |
| decennials (129-month system), circumambulations (directing through the bounds), Balbillus, planetary years | | | Delphic Oracle | P2 | `timelords` |
| planetary hours and days | sunrise, sunset | | all | P0 (hora exists) | `panchanga` |
| primary directions (see Western predictive) | | | Solar Fire | P1 | `western.directions` |
| horary (Lilly): considerations, receptions, translation and collection of light, refranation, frustration, prohibition, Moon's last and next aspects, planetary hour agreement | | | Solar Fire | P1 | `horary` |
| temperament (Lilly weighting) | | | Solar Fire | P2 | `western.analysis` |
| hyleg and alcocoden (Ptolemy, Omar, Bonatti variants), longevity by directions | | | Solar Fire | P2 | `longevity` |

## Why this matters for a Vedic-first SDK

Tajika is medieval Perso-Arabic astrology in Sanskrit dress: Ithasala is the
applying aspect, Ishrafa the separating one, Hadda are the Egyptian terms,
Muntha is the profected ascendant, Sahamas are the lots. Building the term
tables, the applying-separating engine and the lot formulas once serves
Tajika (P0) and the whole Hellenistic and medieval family (P1). The time-lord
registry from `03-dashas.md` holds profections, releasing and firdaria as
event-seeded systems without change.
