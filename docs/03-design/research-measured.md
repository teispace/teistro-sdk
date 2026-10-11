# Research, measured

Status: `generated` by `cargo xtask research`. Do not edit: `cargo xtask check-research` fails on any difference.

Whether the numbers `sdk.research()` answers mean what they say (`research.md` §3). Every batch is seeded, so the page is the same on every run; a rate is judged against the upper end of a 99% interval on its level, since a seeded run is one draw of the sampling noise.

## 1. Calibration (tests 1 and 3)

A synthetic batch of 200 charts carries eight predicates: four independent, of prevalence 5% to 30%, and two nested pairs read off one draw each, as yogas sharing a condition are. 1000 labellings draw each chart a case with probability 0.3, independently of every predicate, and each is tested case against rest over 499 permutations. A rate is judged against 0.0678, the upper end of a 99% interval on 0.05 over 1000 studies.

| proposed rule | verdict | measured |
|---|---|---|
| on labels that mean nothing, a raw p is under 0.05 no more often than 5% | **holds** | 0 of 8 disagree |
| every correction holds the familywise rate at 0.05 under the complete null | **holds** | 0 of 5 disagree |
| the exact p lies inside the permutation p's 95% interval at least 95% of the time | **holds** | 298 of 8000 rows outside |
| Holm never adjusts above Bonferroni | **holds** | 0 of 8000 disagree |

| predicate | raw p under 0.05 |
|---|--:|
| `P05` | 0.033 |
| `P10` | 0.027 |
| `P20` | 0.039 |
| `P30` | 0.049 |
| `A30` | 0.038 |
| `A40` | 0.033 |
| `B50` | 0.038 |
| `B60` | 0.044 |

| method | studies with any predicate under 0.05 |
|---|--:|
| max-T | 0.044 |
| Holm | 0.036 |
| Bonferroni | 0.036 |
| BH | 0.037 |
| BY | 0.008 |

A permutation test conditions on the margins, so a rare predicate cannot reach 0.05 as often as a common one: its rate sits under the level, which is the conservative side.

## 2. Power (test 2)

200 labellings over the same batch draw a chart a case with probability 0.45 where `P30` holds and 0.25 where it does not, a risk ratio of 1.8. What each method finds of the planted predicate:

| method | share of studies under 0.05 |
|---|--:|
| raw | 0.750 |
| max-T | 0.565 |
| Holm | 0.500 |
| Bonferroni | 0.500 |
| BH | 0.500 |
| BY | 0.335 |

| proposed rule | verdict | measured |
|---|---|---|
| a predicate that shares nothing with the planted one stays at the null rate | **holds** | 0 of 3 disagree |

## 3. The Gauquelin artefact (test 4)

300 births at Paris over 1950 to 1990, no effect planted, but born as births were before induction: half between 02:00 and 07:00 local time, the rest at any hour. Each sector is a whole-sign pair of houses standing in for Gauquelin's: rising (the 12th and 1st) and culminating (the 9th and 10th). The uniform column is what a study that expects two houses in twelve would report; the recombined column is `expected` over 49 replicates, each birth keeping its date and place and taking another's clock time. A uniform expectation is not offered (C365), and this is why.

| proposed rule | verdict | measured |
|---|---|---|
| a uniform expectation reads a sector of a sample with no effect as beyond chance, at a two-sided p of 0.001 (z beyond ±3.29) | **holds** | `SUN_CULMINATING` at z -3.6 against 2 houses in 12 |
| the sample's own recombined population finds no sector under 0.05 by max-T | **holds** | 0 of 4 disagree |

| sector | observed | z against uniform | recombined expectation | ratio | max-T |
|---|--:|--:|--:|--:|--:|
| `SUN_RISING` | 0.227 | 2.8 | 0.220 | 1.03 | 0.860 |
| `SUN_CULMINATING` | 0.090 | -3.6 | 0.098 | 0.92 | 0.800 |
| `MARS_RISING` | 0.200 | 1.5 | 0.208 | 0.96 | 0.860 |
| `MARS_CULMINATING` | 0.120 | -2.2 | 0.131 | 0.92 | 0.860 |

The rising Sun, the sector the early hours fill, reads z 2.8 alone: the 12th and 1st hold the Sun for about four hours around sunrise, and at Paris sunrise moves through the year by more than the early peak is wide. The culminating Sun, which the same hours empty, departs further (z -3.6), and a study reading one sector would have reported whichever it looked at.

## 4. The two event shuffles (test 5)

20 studies under each shuffle, each of 60 subjects born at Kathmandu at any hour over 1950 to 1990 with one event between the ages of 25 and 32, placed with no regard to the chart; the shipped yogas are each tested as delivered by the Vimshottari to the antardasha, over 199 permutations, a pairing before a birth never drawn. A row counts where the rule is delivered on some pairings and not all. The ages are narrow and the births forty years wide, so a date shuffle reads each person at ages far from any event's (C364).

| proposed rule | verdict | measured |
|---|---|---|
| under `AGES_AT_EVENT`, a null event study's raw p is under 0.05 no more often than 5% | **holds** | 6 of 154 rows, 0.039 |
| under `EVENT_DATES`, a null event study's raw p is under 0.05 no more often than 5% | **holds** | 5 of 158 rows, 0.032 |

