# Festival rules, measured

Status: `generated` by `cargo xtask festival`. Do not edit:
`check-festival` regenerates this page and fails on any difference.

It measures `festival-rules.md` §5.2 and §5.3: the shipped rules
(`FestivalRule::dharmasindhu()`) through `sdk.almanac().festivals`, over
the built-in ephemeris at Delhi on India's clock.

## 1. Against the published days

The days the Government of India listed as holidays for its offices in Delhi,
read off copies of each year's office memorandum (the pass's source names them).
Janmashtami is listed twice where the restricted list gives a Smarta day apart
from the Vaishnava one; *Dharmasindhu* is a Smarta text.

| year | rule | list | published | found | case | decided by |
|---:|---|---|---|---|---|---|
| 2021 | JANMASHTAMI | Smarta | 30 Aug | 30 Aug | Both | guard 0 |
| 2021 | JANMASHTAMI | Vaishnava | 30 Aug | 30 Aug | Both | guard 0 |
| 2021 | LAKSHMI_PUJA | compulsory | 4 Nov | 4 Nov | LaterOnly | guard 0 |
| 2021 | RAMA_NAVAMI | compulsory | 21 Apr | 21 Apr | LaterOnly | otherwise |
| 2021 | VIJAYA_DASHAMI | compulsory | 15 Oct | 15 Oct | LaterOnly | guard 0 |
| 2023 | JANMASHTAMI | Smarta | 6 Sep | 6 Sep | EarlierOnly | guard 1 |
| 2023 | JANMASHTAMI | Vaishnava | 7 Sep | 6 Sep ✗ | EarlierOnly | guard 1 |
| 2023 | LAKSHMI_PUJA | compulsory | 12 Nov | 12 Nov | EarlierOnly | otherwise |
| 2023 | RAMA_NAVAMI | compulsory | 30 Mar | 30 Mar | LaterOnly | otherwise |
| 2023 | VIJAYA_DASHAMI | compulsory | 24 Oct | 24 Oct | LaterOnly | guard 2 |
| 2024 | JANMASHTAMI | Vaishnava | 26 Aug | 26 Aug | LaterOnly | guard 0 |
| 2024 | LAKSHMI_PUJA | compulsory | 31 Oct | 1 Nov ✗ | UnequalParts | guard 0 |
| 2024 | RAMA_NAVAMI | compulsory | 17 Apr | 17 Apr | UnequalParts | otherwise |
| 2024 | VIJAYA_DASHAMI | compulsory | 12 Oct | 12 Oct | EarlierOnly | guard 5 |
| 2025 | JANMASHTAMI | Smarta | 15 Aug | 15 Aug | EarlierOnly | guard 2 |
| 2025 | JANMASHTAMI | Vaishnava | 16 Aug | 15 Aug ✗ | EarlierOnly | guard 2 |
| 2025 | LAKSHMI_PUJA | compulsory | 20 Oct | 20 Oct | UnequalParts | otherwise |
| 2025 | VIJAYA_DASHAMI | compulsory | 2 Oct | 2 Oct | LaterOnly | guard 0 |
| 2027 | JANMASHTAMI | Vaishnava | 25 Aug | 24 Aug ✗ | EarlierOnly | guard 2 |
| 2027 | LAKSHMI_PUJA | compulsory | 29 Oct | 29 Oct | LaterOnly | guard 0 |
| 2027 | RAMA_NAVAMI | compulsory | 15 Apr | 15 Apr | LaterOnly | otherwise |
| 2027 | VIJAYA_DASHAMI | compulsory | 9 Oct | 9 Oct | EarlierOnly | guard 3 |

| proposed rule | verdict | measured |
|---|---|---|
| each shipped rule falls on the day the published list gives | falsified | 4 of 22 disagree; each parting is named below with its cause |

Where the rule parts from the published day, and why:

- 2023 JANMASHTAMI (Vaishnava): the Vaishnava day, the later one, whose sunrise the 8th holds; the shipped rule is the Smarta one of p. 49, which takes the earlier day when it alone holds nishitha, and *Dharmasindhu* states no Vaishnava rule for Janmashtami (C175)
- 2024 LAKSHMI_PUJA (compulsory): the new moon lasts more than a ghati past the later day's sunset, which p. 77 calls beyond doubt; the list takes the earlier day, over which the calendars of that year were divided
- 2025 JANMASHTAMI (Vaishnava): the Vaishnava day, the later one, whose sunrise the 8th holds; the shipped rule is the Smarta one of p. 49, which takes the earlier day when it alone holds nishitha, and *Dharmasindhu* states no Vaishnava rule for Janmashtami (C175)
- 2027 JANMASHTAMI (Vaishnava): the Vaishnava day, the later one, whose sunrise the 8th holds; the shipped rule is the Smarta one of p. 49, which takes the earlier day when it alone holds nishitha, and *Dharmasindhu* states no Vaishnava rule for Janmashtami (C175)

## 2. What a decade reaches

Each rule over 2021–2030: the case each year met between the tithi's two
days, and the guard that decided. A guard no year reached is listed
with its reason; the pass fails on one that is not, and on a listed
one a year did reach.

| rule | cases met | decided by |
|---|---|---|
| RAMA_NAVAMI | EarlierOnly 2, LaterOnly 6, UnequalParts 2 | otherwise: 8, guard 0: 2 |
| JANMASHTAMI | Both 1, EarlierOnly 6, LaterOnly 2, Neither 1 | otherwise: 2, guard 0: 2, guard 1: 1, guard 2: 5 |
| VIJAYA_DASHAMI | EarlierOnly 4, LaterOnly 5, UnequalParts 1 | otherwise: 1, guard 0: 4, guard 2: 1, guard 3: 2, guard 4: 1, guard 5: 1 |
| LAKSHMI_PUJA | EarlierOnly 5, LaterOnly 3, UnequalParts 2 | otherwise: 6, guard 0: 4 |

The guards no year reached, and why:

- VIJAYA_DASHAMI guard 1: the 10th beginning after the earlier day's aparahna with Shravana joining it only that evening, which no year of the decade had; the crate test `vijaya_dashami_held_later_alone_yields_to_shravana_on_the_earlier_evening` reaches it.
- VIJAYA_DASHAMI guard 6: the 10th holding both aparahnas, neither or each in part, with Shravana joining it on the later day only, which no year of the decade had; the crate test `vijaya_dashami_held_on_both_days_or_neither_goes_to_shravana_alone` reaches it.
