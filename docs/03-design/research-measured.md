# Research, measured

Status: `generated` by `cargo xtask research`. Do not edit: `cargo xtask check-research` fails on any difference.

Whether the numbers `sdk.research()` answers mean what they say (`research.md` §3). Every batch is seeded, so the page is the same on every run; a rate is judged against the upper end of a 99% interval on its level, since a seeded run is one draw of the sampling noise.

## 1. Calibration (tests 1 and 3)

A synthetic batch of 200 charts carries eight predicates: four independent, of prevalence 5% to 30%, and two nested pairs read off one draw each, as yogas sharing a condition are. 1000 labellings draw each chart a case with probability 0.3, independently of every predicate, and each is tested case against rest over 499 permutations. A rate is judged against 0.0678, the upper end of a 99% interval on 0.05 over 1000 studies.

| proposed rule | verdict | measured |
|---|---|---|
| on labels that mean nothing, a raw p is under 0.05 no more often than 5% | **holds** | 0 of 8 disagree |
| every correction holds the familywise rate at 0.05 under the complete null | **holds** | 0 of 5 disagree |
| a raw p is valid at every level: under 0.01, 0.05, 0.10, 0.25 and 0.50 no more often than the level | **holds** | 0 of 40 disagree |
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

On the effect's own scale, the share of cases the planted predicate holds on against the share of the rest, the planted effect over this batch is a risk ratio of 1.739 and a difference of 0.209, from the counts the case shares give. A coverage is judged against 0.9103, the lower end of a 99% interval on 0.95 over 200 studies.

| proposed rule | verdict | measured |
|---|---|---|
| a predicate that shares nothing with the planted one stays at the null rate | **holds** | 0 of 3 disagree |
| the risk ratio's 95% interval (Katz) covers the planted value at least 95% of the time | **holds** | 190 of 200 studies, 0.950 |
| the risk difference's 95% interval (Newcombe) covers the planted value at least 95% of the time | **holds** | 190 of 200 studies, 0.950 |

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

## 4. The event nulls (test 5)

20 studies under each shuffle, each of 60 subjects born at Kathmandu at any hour over 1950 to 1990 with one event between the ages of 25 and 32, placed with no regard to the chart; the shipped yogas are each tested as delivered by the Vimshottari to the antardasha, over 199 permutations, a pairing before a birth never drawn. A row counts where the rule is delivered on some pairings and not all. The ages are narrow and the births forty years wide, so a date shuffle reads each person at ages far from any event's (C364). Self-matched, each person is read at their event and every 30 days of the 365.25-day window from birth that holds it, and nobody's event moves (C367).

| proposed rule | verdict | measured |
|---|---|---|
| under `AGES_AT_EVENT`, a null event study's raw p is under 0.05 no more often than 5% | **holds** | 6 of 154 rows, 0.039 |
| under `EVENT_DATES`, a null event study's raw p is under 0.05 no more often than 5% | **holds** | 6 of 158 rows, 0.038 |
| under `SELF_MATCHED`, a null event study's raw p is under 0.05 no more often than 5% | **holds** | 2 of 136 rows, 0.015 |

## 5. An uncertain birth's window

60 births at Kathmandu at any hour over 1950 to 1990, each uncertain by the minutes either side named, and the shipped yogas counted on them: read across the window every 2 minutes (the default step), and read at its two edges alone, as every window was before. A cell is one rule on one chart; it is unstable when its answer differs anywhere it is read from the answer at the birth.

| proposed rule | verdict | measured |
|---|---|---|
| within 30 minutes either side, the edges alone find every unstable cell | **holds** | 13 of 480 cells unstable across the window, 13 at its edges |
| within 180 minutes either side, the edges alone find every unstable cell | falsified | 40 of 480 cells unstable across the window, 30 at its edges |

## 6. Strata (C369)

400 charts in two strata of 200, interleaved, whose case shares differ (0.15 and 0.45) as a confounder's would. Three predicates hold equally often in both strata and three do not. 400 labellings draw each chart a case with its stratum's share, independently of every predicate, and each is tested twice over 499 permutations: with its strata, which shuffles inside them and ranks by the Cochran–Mantel–Haenszel statistic, and without them. A rate is judged against 0.0781.

| proposed rule | verdict | measured |
|---|---|---|
| with its strata, a raw p is under 0.05 no more often than 5% | **holds** | 0 of 6 disagree |
| with its strata, every correction holds the familywise rate at 0.05 | **holds** | 0 of 5 disagree |
| with its strata, the exact p lies inside the permutation p's 95% interval at least 95% of the time | **holds** | 111 of 2400 rows outside |
| without its strata, every predicate whose prevalence differs between them reads as an effect more often than 5% | **holds** | 0 of 3 disagree |

| predicate | prevalence by stratum | raw p under 0.05, with strata | without |
|---|--:|--:|--:|
| `E10` | 0.1 / 0.1 | 0.030 | 0.015 |
| `E30` | 0.3 / 0.3 | 0.028 | 0.058 |
| `E50` | 0.5 / 0.5 | 0.037 | 0.028 |
| `U10_40` | 0.1 / 0.4 | 0.043 | 0.625 |
| `U40_10` | 0.4 / 0.1 | 0.052 | 0.655 |
| `U05_60` | 0.05 / 0.6 | 0.037 | 0.968 |

The cases crowd into the second stratum, so a predicate commoner there is commoner among the cases, and one rarer there rarer, with no effect at all: read without its strata, the study finds both. Given its strata, it compares each case only with the charts of its own stratum, and finds neither.

## 7. A partial null (test 1, the false discovery rate)

400 labellings of the first batch plant two predicates, `P20` and `P30`, which share nothing with the other six: a chart is a case with probability 0.22, raised by 0.18 for each planted predicate it holds. The other six are true nulls, and a discovery among them is false. BH and BY promise the false discovery rate, the mean share of false discoveries among a study's discoveries; max-T, Holm and Bonferroni promise the familywise rate over the true nulls. A mean is judged by its 99% interval: it fails only when the whole interval lies above 0.05.

| proposed rule | verdict | measured |
|---|---|---|
| under BH, the mean false discovery proportion is at most 0.05 | **holds** | 0.0300, its 99% interval reaching 0.0481 |
| under BY, the mean false discovery proportion is at most 0.05 | **holds** | 0.0025, its 99% interval reaching 0.0089 |
| under max-T, a study rejects any true null no more often than 5% | **holds** | 19 of 400 studies, 0.048 |
| under Holm, a study rejects any true null no more often than 5% | **holds** | 6 of 400 studies, 0.015 |
| under Bonferroni, a study rejects any true null no more often than 5% | **holds** | 6 of 400 studies, 0.015 |

## 8. A small reference set (`research/shuffle/3`)

200 studies of the first batch's eight predicates on 12 of its charts drawn at random, the first 4 the cases: 495 labellings, fewer than the 999 permutations asked, so `/3` reads each once and `/2` draws 999 with replacement. A row counts where its exact p exists, every chart read. Drawn, `(b + 1)/(m + 1)` is a valid p but carries Monte Carlo noise a walk does not (C368).

| proposed rule | verdict | measured |
|---|---|---|
| walked whole, a small study's raw p is the exact hypergeometric p on every row | **holds** | 0 of 1600 rows differ |
| drawn 999 times instead, a small study's raw p is within 0.01 of the exact p on every row | falsified | 318 of 1600 rows further, the worst 0.0467; the mean departure +0.0004 |

