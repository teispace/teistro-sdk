# Festival rules, measured

Status: `generated` by `cargo xtask festival`. Do not edit:
`check-festival` regenerates this page and fails on any difference.

It measures `festival-rules.md` §5.2, §5.3 and §9.3: the shipped rules
(`FestivalRule::dharmasindhu()`) through `sdk.almanac().festivals`, over
the built-in ephemeris at Delhi on India's clock, and at Kathmandu on
Nepal's (§4).

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
| HARITALIKA | LaterOnly 9, Neither 1 | otherwise: 9, guard 0: 1 |
| NAVARATRA_ARAMBHA | LaterOnly 9, Neither 1 | otherwise: 1, guard 1: 9 |
| YAMA_DWITIYA | EarlierOnly 1, LaterOnly 7, UnequalParts 2 | otherwise: 9, guard 0: 1 |
| SHIVARATRI | EarlierOnly 9, LaterOnly 1 | otherwise: 1, guard 0: 9 |

The guards no year reached, and why:

- VIJAYA_DASHAMI guard 1: the 10th beginning after the earlier day's aparahna with Shravana joining it only that evening, which no year of the decade had; the crate test `vijaya_dashami_held_later_alone_yields_to_shravana_on_the_earlier_evening` reaches it.
- VIJAYA_DASHAMI guard 6: the 10th holding both aparahnas, neither or each in part, with Shravana joining it on the later day only, which no year of the decade had; the crate test `vijaya_dashami_held_on_both_days_or_neither_goes_to_shravana_alone` reaches it.
- NAVARATRA_ARAMBHA guard 0: the 1st holding two sunrises, a vriddhi no Ashwina of the decade had; the crate test `navaratra_arambha_needs_a_muhurta_past_the_later_sunrise` reaches it.
- SHIVARATRI guard 1: the 14th holding the earlier night's eighth muhurta whole and the later's in part, which no year of the decade had; the crate test `shivaratri_takes_the_book_s_day_in_each_of_its_clauses` reaches it.

## 3. Ekadashi against a published almanac

Drik Panchang's New Delhi lists for 2023–2026: its Ekadashi list's day for
the Smarta householder, its Gauna day for the renunciant where that
differs, and its ISKCON list for the Vaishnava. The rules are
`EkadashiRule::dharmasindhu()` (`festival-rules.md` §8).

| proposed rule | verdict | measured |
|---|---|---|
| EKADASHI_SMARTA falls on the Smarta list's day | **holds** | 0 of 99 disagree |
| EKADASHI_SMARTA_RENUNCIANT falls on the Gauna list's day | **holds** | 0 of 99 disagree |
| EKADASHI_VAISHNAVA falls on the ISKCON list's day | falsified | 3 of 99 disagree; each parting is named below with its cause |

Where a rule parts from the record, and why:

- 2023 EKADASHI_VAISHNAVA: published 11 Sep, found 10 Sep: the ISKCON list's Paksha Vardhini Mahadvadashi: the fortnight's new or full moon holds two sunrises, which the pass finds so, and the fast moves to the 12th's day, a rule *Dharmasindhu* does not state
- 2024 EKADASHI_VAISHNAVA: published 30 Aug, found 29 Aug: the ISKCON list's Paksha Vardhini Mahadvadashi: the fortnight's new or full moon holds two sunrises, which the pass finds so, and the fast moves to the 12th's day, a rule *Dharmasindhu* does not state
- 2025 EKADASHI_VAISHNAVA: published 16 Dec, found 15 Dec: the ISKCON list's Paksha Vardhini Mahadvadashi: the fortnight's new or full moon holds two sunrises, which the pass finds so, and the fast moves to the 12th's day, a rule *Dharmasindhu* does not state

## 4. Against Nepal's national panchanga

The days the Nepal Panchanga Decision Committee printed in its national
panchanga for VS 2082 and 2083, read off the page images, beside each
shipped rule's day at Kathmandu on Nepal's clock: over the committee's
own sky (`nepali-committee`, the Surya Siddhanta with its bija, C187),
which the pass holds to the print, and over the modern sky
(`nepali-default`), counted beside it.

| VS | rule | printed | where | committee's sky | case | decided by | modern sky |
|---:|---|---|---|---|---|---|---|
| 2082 | JANMASHTAMI | 16 Aug | p. 10: श्रीकृष्णजन्माष्टमीव्रत | 16 Aug | Neither | otherwise | 15 Aug ✗ |
| 2082 | HARITALIKA | 26 Aug | p. 11: हरितालिकाव्रत (तीज) | 26 Aug | LaterOnly | otherwise | 26 Aug |
| 2082 | NAVARATRA_ARAMBHA | 22 Sep | p. 13: नवरात्रारम्भ | 22 Sep | LaterOnly | guard 1 | 22 Sep |
| 2082 | VIJAYA_DASHAMI | 2 Oct | p. 13: विजयादशमी, टीका | 2 Oct | UnequalParts | guard 6 | 2 Oct |
| 2082 | LAKSHMI_PUJA | 20 Oct | p. 14: लक्ष्मीपूजा, दीपमालिका | 20 Oct | EarlierOnly | otherwise | 21 Oct ✗ |
| 2082 | YAMA_DWITIYA | 23 Oct | p. 15: यमद्वितीया, भाइटीका | 23 Oct | LaterOnly | otherwise | 23 Oct |
| 2082 | SHIVARATRI | 15 Feb | p. 22: महाशिवरात्रिव्रत | 15 Feb | EarlierOnly | guard 0 | 15 Feb |
| 2082 | RAMA_NAVAMI | 27 Mar | p. 25: रामनवमीव्रत, श्रीरामजयन्ती | 27 Mar | LaterOnly | otherwise | 26 Mar ✗ |
| 2083 | JANMASHTAMI | 4 Sep | p. 12: श्रीकृष्णजन्माष्टमीव्रत | 4 Sep | Neither | otherwise | 4 Sep |
| 2083 | HARITALIKA | 14 Sep | p. 13: हरितालिकाव्रत (तीज) | 14 Sep | LaterOnly | otherwise | 14 Sep |
| 2083 | NAVARATRA_ARAMBHA | 11 Oct | p. 15: घटस्थापना, नवरात्रारम्भ | 11 Oct | LaterOnly | guard 1 | 11 Oct |
| 2083 | VIJAYA_DASHAMI | 21 Oct | p. 15: विजयादशमी, दशैंको टीका | 20 Oct ✗ | EarlierOnly | guard 5 | 20 Oct ✗ |
| 2083 | LAKSHMI_PUJA | 8 Nov | p. 16: लक्ष्मीपूजा, दीपमालिका | 8 Nov | EarlierOnly | otherwise | 8 Nov |
| 2083 | YAMA_DWITIYA | 11 Nov | p. 17: यमद्वितीया (किजापूजा) | 11 Nov | UnequalParts | otherwise | 11 Nov |
| 2083 | SHIVARATRI | 6 Mar | p. 24: महाशिवरात्रिव्रत | 6 Mar | EarlierOnly | guard 0 | 6 Mar |

| proposed rule | verdict | measured |
|---|---|---|
| each shipped rule over the committee's sky falls on the printed day | falsified | 1 of 15 disagree; each parting is named below with its cause |
| each shipped rule over the modern sky falls on the printed day | falsified | 4 of 15 disagree; the sky decides these: a tithi's end moves the day where the modern sky's and the text's part |

Where a rule over the committee's sky parts from the print, and why:

- VS 2083 VIJAYA_DASHAMI: the committee keeps the day whose sunrise the 10th holds, until 10:51 by its print, with Shravana joining the 10th on the earlier day only; p. 71 gives the earlier day, which alone holds aparahna, and moves to the later only with Shravana joined there alone (C197)
