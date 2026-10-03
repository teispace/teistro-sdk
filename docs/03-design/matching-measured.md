# The Ashta Koota, measured

Status: `generated` by `cargo xtask matching` from the kernel alone,
2026-10-04. Do not edit: `check-matching` regenerates this page and
fails on any difference.

A pada fixes all a koota reads: the nakshatra, the sign and the
navamsha. So the 108 padas give 11 664 pairs, a bride's Moon in one and
a groom's in another, which is every pair the Ashta Koota can tell apart
(`matching.md`). Each is read under the verse's rules, the default, and
again under each knob's alternative.

| proposed rule | verdict | measured |
|---|---|---|
| every total is its kootas' points summed, within 0 to 36 | **holds** | 0 of 11664 disagree |
| the eight come in the verse's order, worth 1 to 8 | **holds** | 0 of 11664 disagree |
| Tara, Yoni, Graha Maitri, Bhakoot and Nadi give the same points when the bride and the groom swap | **holds** | 0 of 11664 disagree |
| a Moon matched with itself totals 28 | **holds** | 0 of 108 disagree |
| Bhakoot gives its 7 exactly when the signs stand without a dosha, a lifted dosha keeping its 0 | **holds** | 0 of 11664 disagree |

## How the totals spread

| total | pairs | share |
|---|---|---|
| 0 to under 6 | 124 | 1.1% |
| 6 to under 12 | 767 | 6.6% |
| 12 to under 18 | 2700 | 23.1% |
| 18 to under 24 | 4081 | 35.0% |
| 24 to under 30 | 3550 | 30.4% |
| 30 to 36 | 442 | 3.8% |

## Each koota

The mean of each koota's points over every pair, and how often it gives
all its points or none.

| koota | most | mean | all its points | none |
|---|---|---|---|---|
| VARNA | 1 | 0.62 | 62.5% | 37.5% |
| VASHYA | 2 | 0.82 | 18.1% | 27.8% |
| TARA | 3 | 2.00 | 33.3% | 0.0% |
| YONI | 4 | 2.00 | 7.3% | 7.1% |
| GRAHA_MAITRI | 5 | 3.00 | 38.9% | 5.6% |
| GANA | 6 | 3.33 | 33.3% | 22.2% |
| BHAKOOT | 7 | 3.50 | 50.0% | 50.0% |
| NADI | 8 | 5.33 | 66.7% | 33.3% |

## The bad Bhakoots and their exceptions

For each dosha, how often each of the five exceptions of VI.32–33
holds, and how often the dosha is lifted: by any one exception with the
nadi pure, the verse's reading, or by Garga's count, three for the 6/8
and two for the others (C263).

| dosha | pairs | one lord | lords friends | navamsha lords friends | tara pure | vashya | lifted by any one | lifted by Garga's count |
|---|---|---|---|---|---|---|---|---|
| 6/8 | 1944 | 16.7% | 8.3% | 38.6% | 22.8% | 83.3% | 54.8% | 1.6% |
| 5/9 | 1944 | 0.0% | 66.7% | 40.7% | 87.7% | 66.7% | 70.0% | 61.3% |
| 2/12 | 1944 | 8.3% | 41.7% | 38.6% | 22.8% | 58.3% | 70.2% | 34.1% |

## What each knob moves

How many pairs change when one reading is swapped for its alternative,
the others left at the verse's: in their totals, and in anything the
answer says. The Bhakoot's lift and the nadi's dosha are clauses beside
points the signs and the nadis fix, so those two knobs can change what
is said without moving a total.

| reading | totals it moves | share | most it moves | readings it changes | share |
|---|---|---|---|---|---|
| an equal varna gives half (C259) | 2916 | 25.0% | 0.5 | 2916 | 25.0% |
| a Deva bride with a Manushya groom gives 3 (C262) | 1296 | 11.1% | 1.0 | 1296 | 11.1% |
| a bad Bhakoot is lifted by Garga's count (C263) | 0 | 0.0% | 0.0 | 1904 | 16.3% |
| only the middle nadi is a dosha (C264) | 0 | 0.0% | 0.0 | 2592 | 22.2% |
