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
| RAKSHABANDHAN | LaterOnly 10 | otherwise: 3, guard 0: 7 |
| BALI_PRATIPADA | LaterOnly 8, Neither 2 | otherwise: 2, guard 0: 8 |
| HOLIKA | EarlierOnly 7, LaterOnly 3 | otherwise: 3, guard 0: 7 |
| UPAKARMA_MADHYANDINA | LaterOnly 10 | otherwise: 4, guard 1: 6 |
| UPAKARMA_TAITTIRIYA | LaterOnly 10 | otherwise: 3, guard 1: 7 |

The guards no year reached, and why:

- VIJAYA_DASHAMI guard 1: the 10th beginning after the earlier day's aparahna with Shravana joining it only that evening, which no year of the decade had; the crate test `vijaya_dashami_held_later_alone_yields_to_shravana_on_the_earlier_evening` reaches it.
- VIJAYA_DASHAMI guard 6: the 10th holding both aparahnas, neither or each in part, with Shravana joining it on the later day only, which no year of the decade had; the crate test `vijaya_dashami_held_on_both_days_or_neither_goes_to_shravana_alone` reaches it.
- NAVARATRA_ARAMBHA guard 0: the 1st holding two sunrises, a vriddhi no Ashwina of the decade had; the crate test `navaratra_arambha_needs_a_muhurta_past_the_later_sunrise` reaches it.
- SHIVARATRI guard 1: the 14th holding the earlier night's eighth muhurta whole and the later's in part, which no year of the decade had; the crate test `shivaratri_takes_the_book_s_day_in_each_of_its_clauses` reaches it.
- HOLIKA guard 1: the full moon beginning after the earlier day's pradosha and ending before the later's, a tithi shorter than the day between them, which no Phalguna of the decade had; the crate test `holika_takes_the_later_day_whenever_its_pradosha_holds_the_full_moon` reaches it.
- UPAKARMA_MADHYANDINA guard 0: Shravana's full moon holding two sunrises, a vriddhi no year of the decade had; the crate test `upakarma_parts_the_yajurvedis_on_how_long_the_later_day_holds_the_full_moon` reaches it.
- UPAKARMA_TAITTIRIYA guard 0: Shravana's full moon holding two sunrises, a vriddhi no year of the decade had; the crate test `upakarma_parts_the_yajurvedis_on_how_long_the_later_day_holds_the_full_moon` reaches it.

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
panchanga for VS 2082 and 2083, read off the page images, at Kathmandu on
Nepal's clock, beside four readings of each observance (Holi in the hills
and the Terai counted from the Holika day, §9.4): the text's rules over
the committee's own sky (`nepali-committee`, the Surya Siddhanta with its
bija, C187), each parting named with its cause; the same over the modern
sky (`nepali-default`); every rule read as the day whose sunrise holds
its tithi, C197's rival; and the `NEPAL` pack, the text for a rite of the
night and the tithi at sunrise for one of the daylight (§9.5), which the
pass holds to every printed day.

| VS | rule | printed | where | the text | case | decided by | modern sky | udaya | `NEPAL` |
|---:|---|---|---|---|---|---|---|---|---|
| 2082 | RAKSHABANDHAN | 9 Aug | p. 9: रक्षाबन्धन, जनैपूर्णिमा | 9 Aug | LaterOnly | guard 0 | 9 Aug | 9 Aug | 9 Aug |
| 2082 | UPAKARMA_MADHYANDINA | 9 Aug | p. 9: जनैपूर्णिमा | 9 Aug | LaterOnly | guard 1 | 9 Aug | 9 Aug | 9 Aug |
| 2082 | JANMASHTAMI | 16 Aug | p. 10: श्रीकृष्णजन्माष्टमीव्रत | 16 Aug | Neither | otherwise | 15 Aug ✗ | 16 Aug | 16 Aug |
| 2082 | HARITALIKA | 26 Aug | p. 11: हरितालिकाव्रत (तीज) | 26 Aug | LaterOnly | otherwise | 26 Aug | 26 Aug | 26 Aug |
| 2082 | NAVARATRA_ARAMBHA | 22 Sep | p. 13: नवरात्रारम्भ | 22 Sep | LaterOnly | guard 1 | 22 Sep | 22 Sep | 22 Sep |
| 2082 | VIJAYA_DASHAMI | 2 Oct | p. 13: विजयादशमी, टीका | 2 Oct | UnequalParts | guard 6 | 2 Oct | 2 Oct | 2 Oct |
| 2082 | LAKSHMI_PUJA | 20 Oct | p. 14: लक्ष्मीपूजा, दीपमालिका | 20 Oct | EarlierOnly | otherwise | 21 Oct ✗ | 21 Oct ✗ | 20 Oct |
| 2082 | BALI_PRATIPADA | 22 Oct | p. 15: गोवर्धनपूजा, म्हपूजा, बलिपूजा | 22 Oct | LaterOnly | guard 0 | 22 Oct | 22 Oct | 22 Oct |
| 2082 | YAMA_DWITIYA | 23 Oct | p. 15: यमद्वितीया, भाइटीका | 23 Oct | LaterOnly | otherwise | 23 Oct | 23 Oct | 23 Oct |
| 2082 | SHIVARATRI | 15 Feb | p. 22: महाशिवरात्रिव्रत | 15 Feb | EarlierOnly | guard 0 | 15 Feb | 16 Feb ✗ | 15 Feb |
| 2082 | HOLIKA | 2 Mar | p. 23: राति भद्रान्तमा चिरदाह | 2 Mar | EarlierOnly | guard 0 | 2 Mar | 3 Mar ✗ | 2 Mar |
| 2082 | HOLI_HILLS | 2 Mar | p. 23: पहाडी जिल्लामा होली | 2 Mar | EarlierOnly | 0 after HOLIKA | 2 Mar | 3 Mar ✗ | 2 Mar |
| 2082 | HOLI_TERAI | 3 Mar | p. 23: तराईमा होली | 3 Mar | EarlierOnly | 1 after HOLIKA | 3 Mar | 4 Mar ✗ | 3 Mar |
| 2082 | RAMA_NAVAMI | 27 Mar | p. 25: रामनवमीव्रत, श्रीरामजयन्ती | 27 Mar | LaterOnly | otherwise | 26 Mar ✗ | 27 Mar | 27 Mar |
| 2083 | RAKSHABANDHAN | 28 Aug | p. 11: रक्षाबन्धन, जनैपूर्णिमा | 28 Aug | LaterOnly | guard 0 | 28 Aug | 28 Aug | 28 Aug |
| 2083 | UPAKARMA_MADHYANDINA | 28 Aug | p. 11: जनैपूर्णिमा | 27 Aug ✗ | LaterOnly | otherwise | 27 Aug ✗ | 28 Aug | 28 Aug |
| 2083 | JANMASHTAMI | 4 Sep | p. 12: श्रीकृष्णजन्माष्टमीव्रत | 4 Sep | Neither | otherwise | 4 Sep | 4 Sep | 4 Sep |
| 2083 | HARITALIKA | 14 Sep | p. 13: हरितालिकाव्रत (तीज) | 14 Sep | LaterOnly | otherwise | 14 Sep | 14 Sep | 14 Sep |
| 2083 | NAVARATRA_ARAMBHA | 11 Oct | p. 15: घटस्थापना, नवरात्रारम्भ | 11 Oct | LaterOnly | guard 1 | 11 Oct | 11 Oct | 11 Oct |
| 2083 | VIJAYA_DASHAMI | 21 Oct | p. 15: विजयादशमी, दशैंको टीका | 20 Oct ✗ | EarlierOnly | guard 5 | 20 Oct ✗ | 21 Oct | 21 Oct |
| 2083 | LAKSHMI_PUJA | 8 Nov | p. 16: लक्ष्मीपूजा, दीपमालिका | 8 Nov | EarlierOnly | otherwise | 8 Nov | 9 Nov ✗ | 8 Nov |
| 2083 | BALI_PRATIPADA | 10 Nov | p. 17: गोवर्धनपूजा, म्हपूजा, बलिपूजा | 9 Nov ✗ | LaterOnly | otherwise | 10 Nov | 10 Nov | 10 Nov |
| 2083 | YAMA_DWITIYA | 11 Nov | p. 17: यमद्वितीया (किजापूजा) | 11 Nov | UnequalParts | otherwise | 11 Nov | 11 Nov | 11 Nov |
| 2083 | SHIVARATRI | 6 Mar | p. 24: महाशिवरात्रिव्रत | 6 Mar | EarlierOnly | guard 0 | 6 Mar | 7 Mar ✗ | 6 Mar |
| 2083 | HOLIKA | 21 Mar | p. 25: राति भद्रान्तमा चिरदाह | 21 Mar | EarlierOnly | guard 0 | 21 Mar | 22 Mar ✗ | 21 Mar |
| 2083 | HOLI_HILLS | 21 Mar | p. 25: पहाडी जिल्लामा होली | 21 Mar | EarlierOnly | 0 after HOLIKA | 21 Mar | 22 Mar ✗ | 21 Mar |
| 2083 | HOLI_TERAI | 22 Mar | p. 25: तराईमा होली | 22 Mar | EarlierOnly | 1 after HOLIKA | 22 Mar | 23 Mar ✗ | 22 Mar |

| proposed rule | verdict | measured |
|---|---|---|
| the text's rules over the committee's sky fall on the printed day | falsified | 3 of 27 disagree; each parting is named below with its cause |
| the text's rules over the modern sky fall on the printed day | falsified | 5 of 27 disagree; the sky decides these: a tithi's end moves the day where the modern sky's and the text's part |
| every rule read at sunrise, over the committee's sky, falls on the printed day | falsified | 10 of 27 disagree; each parting is a rite of the night or the evening, where the committee keeps the text's window |
| the `NEPAL` pack over the committee's sky falls on the printed day | **holds** | 0 of 27 disagree; C197: the text for a rite of the night, the tithi at sunrise for one of the daylight |

Where the text's rules over the committee's sky part from the print, and why:

- VS 2083 VIJAYA_DASHAMI: the committee keeps the day whose sunrise the 10th holds, until 10:51 by its print, with Shravana joining the 10th on the earlier day only; p. 71 gives the earlier day, which alone holds aparahna, and moves to the later only with Shravana joined there alone (C197)
- VS 2083 BALI_PRATIPADA: the committee keeps the day whose sunrise the 1st holds, though by its print the 1st lasts only 15 ghatis 49 palas past it, until 12:41; p. 78 keeps that day only when the 1st lasts nine muhurtas (18 ghatis) past sunrise, and otherwise the earlier day the new moon pierces (C197)
- VS 2083 UPAKARMA_MADHYANDINA: the committee prints Janai purnima on the day whose sunrise the full moon holds, until 9:16 by its print after a 5:41 sunrise, about nine ghatis; p. 47 gives the Madhyandina the later day only past six muhurtas (12 ghatis), and the earlier when less (C197)

## 5. Nepal's monthly full-moon fast

The committee prints पूर्णिमाव्रत every month of VS 2082 and 2083, the
adhika Jyeshtha too; the 24 days were read off the page images, each
from its row's own Gregorian column. No text in hand states the rule:
*Dharmasindhu* p. 20 gives the full moon the later day. The `NEPAL`
pack's `PURNIMA_VRATA` takes the day whose sunset the full moon holds,
the later when both or neither do (§9.6, C200). It is found at
Kathmandu on Nepal's clock over the committee's own sky, which the pass
holds to every printed day, and beside it the modern sky and five rival
readings. The case is the shipped rule's, at the two sunsets.

| VS | page | printed | month | case | `NEPAL` | modern sky | sunrise day | evening, earlier first | pradosha | 18 nadis | moonrise |
|---:|---:|---|---|---|---|---|---|---|---|---|---|
| 2082 | 3 | 12 May | VAISHAKHA | LaterOnly | 12 May | 12 May | 12 May | 12 May | 12 May | 12 May | 12 May |
| 2082 | 5 | 10 Jun | JYESHTHA | EarlierOnly | 10 Jun | 10 Jun | 11 Jun ✗ | 10 Jun | 10 Jun | 10 Jun | 10 Jun |
| 2082 | 7 | 10 Jul | ASHADHA | LaterOnly | 10 Jul | 10 Jul | 10 Jul | 10 Jul | 10 Jul | 10 Jul | 10 Jul |
| 2082 | 9 | 8 Aug | SHRAVANA | EarlierOnly | 8 Aug | 8 Aug | 9 Aug ✗ | 8 Aug | 8 Aug | 9 Aug ✗ | 8 Aug |
| 2082 | 11 | 7 Sep | BHADRAPADA | LaterOnly | 7 Sep | 7 Sep | 7 Sep | 7 Sep | 7 Sep | 7 Sep | 7 Sep |
| 2082 | 13 | 6 Oct | ASHWINA | EarlierOnly | 6 Oct | 6 Oct | 7 Oct ✗ | 6 Oct | 6 Oct | 6 Oct | 6 Oct |
| 2082 | 15 | 5 Nov | KARTIKA | LaterOnly | 5 Nov | 5 Nov | 5 Nov | 5 Nov | 5 Nov | 5 Nov | 5 Nov |
| 2082 | 17 | 4 Dec | MARGASHIRSHA | EarlierOnly | 4 Dec | 4 Dec | 4 Dec | 4 Dec | 4 Dec | 4 Dec | 4 Dec |
| 2082 | 19 | 3 Jan | PAUSHA | Neither | 3 Jan | 3 Jan | 3 Jan | 3 Jan | 2 Jan ✗ | 3 Jan | 3 Jan |
| 2082 | 21 | 1 Feb | MAGHA | LaterOnly | 1 Feb | 1 Feb | 1 Feb | 1 Feb | 1 Feb | 1 Feb | 1 Feb |
| 2082 | 23 | 2 Mar | PHALGUNA | EarlierOnly | 2 Mar | 3 Mar ✗ | 3 Mar ✗ | 2 Mar | 2 Mar | 3 Mar ✗ | 3 Mar ✗ |
| 2082 | 25 | 1 Apr | CHAITRA | EarlierOnly | 1 Apr | 1 Apr | 2 Apr ✗ | 1 Apr | 1 Apr | 1 Apr | 1 Apr |
| 2083 | 3 | 1 May | VAISHAKHA | LaterOnly | 1 May | 1 May | 1 May | 1 May | 1 May | 1 May | 1 May |
| 2083 | 5 | 30 May | JYESHTHA (adhika) | EarlierOnly | 30 May | 30 May | 31 May ✗ | 30 May | 30 May | 30 May | 30 May |
| 2083 | 7 | 29 Jun | JYESHTHA | LaterOnly | 29 Jun | 29 Jun | 29 Jun | 29 Jun | 29 Jun | 29 Jun | 29 Jun |
| 2083 | 9 | 29 Jul | ASHADHA | Both | 29 Jul | 29 Jul | 29 Jul | 28 Jul ✗ | 29 Jul | 29 Jul | 29 Jul |
| 2083 | 11 | 27 Aug | SHRAVANA | EarlierOnly | 27 Aug | 27 Aug | 28 Aug ✗ | 27 Aug | 27 Aug | 27 Aug | 27 Aug |
| 2083 | 13 | 26 Sep | BHADRAPADA | LaterOnly | 26 Sep | 26 Sep | 26 Sep | 26 Sep | 26 Sep | 26 Sep | 26 Sep |
| 2083 | 15 | 25 Oct | ASHWINA | EarlierOnly | 25 Oct | 25 Oct | 26 Oct ✗ | 25 Oct | 25 Oct | 25 Oct | 25 Oct |
| 2083 | 17 | 24 Nov | KARTIKA | LaterOnly | 24 Nov | 24 Nov | 24 Nov | 24 Nov | 24 Nov | 24 Nov | 24 Nov |
| 2083 | 19 | 23 Dec | MARGASHIRSHA | EarlierOnly | 23 Dec | 23 Dec | 24 Dec ✗ | 23 Dec | 23 Dec | 23 Dec | 23 Dec |
| 2083 | 21 | 22 Jan | PAUSHA | LaterOnly | 22 Jan | 22 Jan | 22 Jan | 22 Jan | 22 Jan | 22 Jan | 22 Jan |
| 2083 | 23 | 20 Feb | MAGHA | EarlierOnly | 20 Feb | 20 Feb | 20 Feb | 20 Feb | 20 Feb | 20 Feb | 20 Feb |
| 2083 | 25 | 21 Mar | PHALGUNA | EarlierOnly | 21 Mar | 22 Mar ✗ | 22 Mar ✗ | 21 Mar | 21 Mar | 22 Mar ✗ | 22 Mar ✗ |

| proposed rule | verdict | measured |
|---|---|---|
| the `NEPAL` pack's PURNIMA_VRATA over the committee's sky falls on the printed day | **holds** | 0 of 24 disagree |
| the same over the modern sky falls on the printed day | falsified | 2 of 24 disagree |
| the full moon's sunrise day (Dharmasindhu p. 20's "the later") is the printed day | falsified | 10 of 24 disagree |
| the evening the full moon holds, the earlier when both do, is the printed day | falsified | 1 of 24 disagree |
| the pradosha the full moon touches, the later when both do, is the printed day | falsified | 1 of 24 disagree |
| the earlier day when the 14th lasts under eighteen nadis past its sunrise (p. 20's allowance for family rites) is the printed day | falsified | 3 of 24 disagree |
| the moonrise the full moon holds, the later when both do, is the printed day | falsified | 2 of 24 disagree |
