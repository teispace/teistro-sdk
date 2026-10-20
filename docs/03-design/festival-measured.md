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
| UPAKARMA_SAMAVEDI | EarlierOnly 5, LaterOnly 4, UnequalParts 1 | otherwise: 5, guard 0: 5 |
| SANKASHTA_CHATURTHI | Both 2, EarlierOnly 57, LaterOnly 22, Neither 10 | otherwise: 32, guard 0: 57, guard 1: 2 |
| KARAKA_CHATURTHI | EarlierOnly 2, LaterOnly 7, Neither 1 | otherwise: 8, guard 0: 2 |

The guards no year reached, and why:

- VIJAYA_DASHAMI guard 1: the 10th beginning after the earlier day's aparahna with Shravana joining it only that evening, which no year of the decade had; the crate test `vijaya_dashami_held_later_alone_yields_to_shravana_on_the_earlier_evening` reaches it.
- VIJAYA_DASHAMI guard 6: the 10th holding both aparahnas, neither or each in part, with Shravana joining it on the later day only, which no year of the decade had; the crate test `vijaya_dashami_held_on_both_days_or_neither_goes_to_shravana_alone` reaches it.
- NAVARATRA_ARAMBHA guard 0: the 1st holding two sunrises, a vriddhi no Ashwina of the decade had; the crate test `navaratra_arambha_needs_a_muhurta_past_the_later_sunrise` reaches it.
- SHIVARATRI guard 1: the 14th holding the earlier night's eighth muhurta whole and the later's in part, which no year of the decade had; the crate test `shivaratri_takes_the_book_s_day_in_each_of_its_clauses` reaches it.
- HOLIKA guard 1: the full moon beginning after the earlier day's pradosha and ending before the later's, a tithi shorter than the day between them, which no Phalguna of the decade had; the crate test `holika_takes_the_later_day_whenever_its_pradosha_holds_the_full_moon` reaches it.
- UPAKARMA_MADHYANDINA guard 0: Shravana's full moon holding two sunrises, a vriddhi no year of the decade had; the crate test `upakarma_parts_the_yajurvedis_on_how_long_the_later_day_holds_the_full_moon` reaches it.
- UPAKARMA_TAITTIRIYA guard 0: Shravana's full moon holding two sunrises, a vriddhi no year of the decade had; the crate test `upakarma_parts_the_yajurvedis_on_how_long_the_later_day_holds_the_full_moon` reaches it.
- KARAKA_CHATURTHI guard 1: the 4th holding both moonrises, which needs it to outlast the 24 h 50 m between them, and no Ashvina of the decade had; the crate test `a_rite_at_moonrise_reads_each_days_first_moonrise` reaches it.

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
| 2082 | UPAKARMA_SAMAVEDI | 26 Aug | p. 11: सामवेदीहरूको उपाकर्म | 26 Aug | EarlierOnly | guard 0 | 26 Aug | 26 Aug | 26 Aug |
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
| 2083 | UPAKARMA_SAMAVEDI | 13 Sep | p. 13: सामवेदीहरूको उपाकर्म | 13 Sep | UnequalParts | otherwise | 13 Sep | 13 Sep | 13 Sep |
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
| the text's rules over the committee's sky fall on the printed day | falsified | 3 of 29 disagree; each parting is named below with its cause |
| the text's rules over the modern sky fall on the printed day | falsified | 5 of 29 disagree; the sky decides these: a tithi's end moves the day where the modern sky's and the text's part |
| every rule read at sunrise, over the committee's sky, falls on the printed day | falsified | 10 of 29 disagree; each parting is a rite of the night or the evening, where the committee keeps the text's window |
| the `NEPAL` pack over the committee's sky falls on the printed day | **holds** | 0 of 29 disagree; C197: the text for a rite of the night, the tithi at sunrise for one of the daylight |

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

## 6. The fitted rules against the print

The days the committee printed for VS 2082 and 2083 that the
`NEPAL_PANCHANGA` pack adds to `NEPAL` (`festival-rules.md` §10), at
Kathmandu on Nepal's clock, beside the day the pack gives over the
committee's own sky (`nepali-committee`), each parting named with its
cause, and over the modern sky (`nepali-default`). Each rule is fitted:
its window and table were chosen to give these days, so the agreement
measures the fit and not a text (C600).

| VS | rule | printed | where | committee sky | modern sky |
|---:|---|---|---|---|---|
| 2082 | PARASHURAMA_JAYANTI | 29 Apr | p. 3: परशुरामजयन्ती | 29 Apr | 29 Apr |
| 2082 | SHANKARACHARYA_JAYANTI | 2 May | p. 3: आद्यजगत्गुरु शङ्कराचार्यजयन्ती | 2 May | 1 May ✗ |
| 2082 | GANGA_SAPTAMI | 4 May | p. 3: गंगासप्तमी | 4 May | 3 May ✗ |
| 2082 | SITA_NAVAMI | 6 May | p. 3: सीताजयन्ती | 6 May | 5 May ✗ |
| 2082 | KURMA_JAYANTI | 12 May | p. 3: कूर्मजयन्ती | 12 May | 12 May |
| 2082 | DASHAHARA_SNANA_ARAMBHA | 28 May | p. 5: दशहरास्नानारम्भ | 28 May | 28 May |
| 2082 | GOSAIKUNDA_SNANA_ARAMBHA | 28 May | p. 5: गोसाइकुण्डस्नानारम्भ | 28 May | 28 May |
| 2082 | KUMARA_JATRA | 2 Jun | p. 5: कुमारयात्रा | 2 Jun | 2 Jun |
| 2082 | GANGA_DASHAHARA | 5 Jun | p. 5: गङ्गादशहरा | 5 Jun | 5 Jun |
| 2082 | GOSAIKUNDA_SNANA_SAMAPTI | 5 Jun | p. 5: गोसाइँकुण्ड स्नानसमाप्ति | 5 Jun | 5 Jun |
| 2082 | HATYAMOCHANA_SNANA | 5 Jun | p. 5: हत्यामोचन तीर्थस्नान | 5 Jun | 5 Jun |
| 2082 | NIRJALA_EKADASHI | 7 Jun | p. 5: निर्जला एकादशीव्रत | 7 Jun | 6 Jun ✗ |
| 2082 | DILA_CHARHE | 24 Jun | p. 6: (दिल्ला चःह्रे पूजा) | 24 Jun | 24 Jun |
| 2082 | HILE_JATRA | 17 Aug | p. 10: रामेछाप चिसापानी हिलेजात्रा | 1 Sep ✗ | 1 Sep ✗ |
| 2082 | AJA_EKADASHI | 19 Aug | p. 10: अजाएकादशीव्रत | 19 Aug | 19 Aug |
| 2082 | CHATHA | 26 Aug | p. 11: चन्द्रदोषदर्शन (चथा) | 26 Aug | 27 Aug ✗ |
| 2082 | GANESHA_CHATURTHI | 27 Aug | p. 11: गणेशचतुर्थीव्रत | 27 Aug | 27 Aug |
| 2082 | RISHI_PANCHAMI | 28 Aug | p. 11: ऋषिपञ्चमीव्रत | 28 Aug | 28 Aug |
| 2082 | GAURA_SAPTAMI | 30 Aug | p. 11: गौरासप्तमी | 30 Aug | 30 Aug |
| 2082 | AMUKTABHARANA_SAPTAMI | 30 Aug | p. 11: अमुक्ताभरणसप्तमी | 30 Aug | 30 Aug |
| 2082 | DURVA_ASHTAMI | 31 Aug | p. 11: दूर्वाष्टमीव्रत | 31 Aug | 31 Aug |
| 2082 | VAMANA_DWADASHI | 4 Sep | p. 11: वामनद्वादशी | 4 Sep | 4 Sep |
| 2082 | PRATIPADA_SHRADDHA | 8 Sep | p. 12: प्रतिपदाश्राद्ध | 8 Sep | 8 Sep |
| 2082 | SORHA_SHRADDHA_ARAMBHA | 8 Sep | p. 12: सोह्रश्राद्ध सुरु | 8 Sep | 8 Sep |
| 2082 | DWITIYA_SHRADDHA | 9 Sep | p. 12: द्वितीयाश्राद्ध | 9 Sep | 9 Sep |
| 2082 | TRITIYA_SHRADDHA | 10 Sep | p. 12: तृतीयाश्राद्ध (गातिला) | 10 Sep | 10 Sep |
| 2082 | CHATURTHI_SHRADDHA | 11 Sep | p. 12: चतुर्थीश्राद्ध | 11 Sep | 10 Sep ✗ |
| 2082 | PANCHAMI_SHRADDHA | 12 Sep | p. 12: पञ्चमीश्राद्ध | 12 Sep | 11 Sep ✗ |
| 2082 | SHASHTHI_SHRADDHA | 12 Sep | p. 12: षष्ठीश्राद्ध | 12 Sep | 12 Sep |
| 2082 | SAPTAMI_SHRADDHA | 13 Sep | p. 12: सप्तमीश्राद्ध | 13 Sep | 13 Sep |
| 2082 | ASHTAMI_SHRADDHA | 14 Sep | p. 12: अष्टमीश्राद्ध | 14 Sep | 14 Sep |
| 2082 | NAVAMI_SHRADDHA | 15 Sep | p. 12: नवमीश्राद्ध, मातृनवमीश्राद्ध | 15 Sep | 15 Sep |
| 2082 | DASHAMI_SHRADDHA | 16 Sep | p. 12: दशमीश्राद्ध | 16 Sep | 16 Sep |
| 2082 | EKADASHI_SHRADDHA | 17 Sep | p. 12: एकादशीश्राद्ध | 17 Sep | 17 Sep |
| 2082 | DWADASHI_SHRADDHA | 18 Sep | p. 12: द्वादशीश्राद्ध | 18 Sep | 18 Sep |
| 2082 | TRAYODASHI_SHRADDHA | 19 Sep | p. 12: त्रयोदशीश्राद्ध | 19 Sep | 19 Sep |
| 2082 | KALI_YUGADI | 19 Sep | p. 12: कलियुगादि | 19 Sep | 19 Sep |
| 2082 | CHATURDASHI_SHRADDHA | 20 Sep | p. 12: शस्त्रादिबाट निधन भएकाहरूको विश्वेदेवासहित एकोदिष्ट विधिले चतुर्दशीश्राद्ध | 20 Sep | 20 Sep |
| 2082 | AMAVASYA_SHRADDHA | 21 Sep | p. 12: औंसीश्राद्ध, पितृविसर्जन | 21 Sep | 21 Sep |
| 2082 | UPANGA_LALITA_VRATA | 26 Sep | p. 13: उपाङ्गललिताव्रत | 26 Sep | 27 Sep ✗ |
| 2082 | KUCHHI_BHOYE | 30 Sep | p. 13: (कूछि भ्वय्) | 30 Sep | 30 Sep |
| 2082 | DHANVANTARI_JAYANTI | 19 Oct | p. 14: धन्वन्तरी जयन्ती | 19 Oct | 19 Oct |
| 2082 | MHA_PUJA | 22 Oct | p. 15: म्हपूजा | 22 Oct | 22 Oct |
| 2082 | NEPAL_SAMBAT_NEW_YEAR | 22 Oct | p. 15: ने.सं.११४६ प्रारम्भ | 22 Oct | 22 Oct |
| 2082 | HARIBODHINI_EKADASHI | 1 Nov | p. 15: हरिबोधिनीएकादशीव्रत | 1 Nov | 2 Nov ✗ |
| 2082 | BHISHMA_PANCHAKA_ARAMBHA | 1 Nov | p. 15: भीष्मपञ्चकारम्भ | 1 Nov | 2 Nov ✗ |
| 2082 | TULASI_VIVAHA | 1 Nov | p. 15: तुलसीविवाह | 1 Nov | 2 Nov ✗ |
| 2082 | PASHUPATI_HALESI_MELA | 18 Nov | p. 16: हलेसीमहादेवमेला | 18 Nov | 18 Nov |
| 2082 | BALA_CHATURDASHI | 19 Nov | p. 16: बालाचतुर्दशी | 19 Nov | 19 Nov |
| 2082 | PISHACHAMOCHANA_SHRADDHA | 3 Dec | p. 17: काशीमा पिशाचमोचन तीर्थश्राद्ध | 3 Dec | 3 Dec |
| 2082 | DATTATREYA_JAYANTI | 4 Dec | p. 17: दत्तात्रेयजयन्ती | 4 Dec | 4 Dec |
| 2082 | NUWAKOT_DUPCHESHWAR_MELA | 4 Dec | p. 17: नुवाकोट दुप्चेश्वरमेला | 4 Dec | 4 Dec |
| 2082 | GAIDU_PUJA | 4 Dec | p. 17: गैडुपूजा | 4 Dec | 4 Dec |
| 2082 | PAUSHA_PUTRADA_EKADASHI | 30 Dec | p. 19: पुत्रदाएकादशीव्रत | 30 Dec | 30 Dec |
| 2082 | LAI_CHARHE | 17 Jan | p. 20: (लैचःह्रेपूजा) | 17 Jan | 17 Jan |
| 2082 | TILAKUNDA_CHATURTHI | 22 Jan | p. 21: तिलकुन्द चौथी | 22 Jan | 22 Jan |
| 2082 | BHIMA_EKADASHI | 29 Jan | p. 21: भीमा एकादशीव्रत | 29 Jan | 29 Jan |
| 2082 | VIJAYA_EKADASHI | 13 Feb | p. 22: विजया एकादशीव्रत | 13 Feb | 13 Feb |
| 2082 | SHILA_CHARHE | 15 Feb | p. 22: (शिलाचःह्रेपूजा) | 15 Feb | 15 Feb |
| 2082 | DVAPARA_YUGADI | 17 Feb | p. 22: द्वापरयुगादि | 17 Feb | 17 Feb |
| 2082 | PAHAN_CHARHE | 17 Mar | p. 24: (पाशाचःह्रेपूजा) | 17 Mar | 17 Mar |
| 2082 | GHODE_JATRA | 18 Mar | p. 24: घोडेजात्रा | 18 Mar | 18 Mar |
| 2082 | MATSYANARAYANA_MELA | 21 Mar | p. 25: मत्स्यनारायणमेला | 21 Mar | 21 Mar |
| 2082 | MATSYA_JAYANTI | 21 Mar | p. 25: मत्स्यजयन्ती | 21 Mar | 21 Mar |
| 2082 | CHAITE_DASHAIN | 26 Mar | p. 25: चैतेदशैं | 26 Mar | 26 Mar |
| 2082 | SETO_MACHHINDRANATH_JATRA | 26 Mar | p. 25: श्रीश्वेतमत्स्येन्द्रनाथ रथयात्रा | 26 Mar | 26 Mar |
| 2082 | VAISHAKHA_SNANA_ARAMBHA | 2 Apr | p. 25: वैशाखस्नान सुरु | 2 Apr | 2 Apr |
| 2082 | RAVI_SAPTAMI | 20 Apr | p. 2: रविसप्तमीव्रत | 20 Apr | 20 Apr |
| 2082 | PRADOSHA_VRATA | 25 Apr | p. 2: प्रदोषव्रत | 25 Apr | 25 Apr |
| 2082 | RAVI_SAPTAMI | 4 May | p. 3: रविसप्तमीव्रत | 4 May | 4 May |
| 2082 | PRADOSHA_VRATA | 9 May | p. 3: प्रदोषव्रत | 9 May | 9 May |
| 2082 | BHAUMASHTAMI | 20 May | p. 4: भौमाष्टमीव्रत | 20 May | 20 May |
| 2082 | PRADOSHA_VRATA | 24 May | p. 4: शनिप्रदोषव्रत | 24 May | 24 May |
| 2082 | BHAUMASHTAMI | 3 Jun | p. 5: भौमाष्टमीव्रत | 3 Jun | 3 Jun |
| 2082 | PRADOSHA_VRATA | 8 Jun | p. 5: प्रदोषव्रत | 8 Jun | 8 Jun |
| 2082 | PRADOSHA_VRATA | 23 Jun | p. 6: सोमप्रदोषव्रत | 23 Jun | 23 Jun |
| 2082 | PRADOSHA_VRATA | 8 Jul | p. 7: प्रदोषव्रत | 8 Jul | 8 Jul |
| 2082 | PRADOSHA_VRATA | 22 Jul | p. 8: प्रदोषव्रत | 22 Jul | 22 Jul |
| 2082 | PRADOSHA_VRATA | 6 Aug | p. 9: प्रदोषव्रत | 6 Aug | 6 Aug |
| 2082 | PRADOSHA_VRATA | 20 Aug | p. 10: प्रदोषव्रत | 20 Aug | 20 Aug |
| 2082 | PRADOSHA_VRATA | 5 Sep | p. 11: प्रदोषव्रत | 5 Sep | 5 Sep |
| 2082 | RAVI_SAPTAMI | 14 Sep | p. 12: रविसप्तमीव्रत | 14 Sep | none ✗ |
| 2082 | PRADOSHA_VRATA | 19 Sep | p. 12: प्रदोषव्रत | 19 Sep | 19 Sep |
| 2082 | BHAUMASHTAMI | 30 Sep | p. 13: भौमाष्टमीव्रत | 30 Sep | 30 Sep |
| 2082 | PRADOSHA_VRATA | 4 Oct | p. 13: शनिप्रदोषव्रत | 4 Oct | 4 Oct |
| 2082 | BHAUMASHTAMI | 14 Oct | p. 14: भौमाष्टमीव्रत | 14 Oct | 14 Oct |
| 2082 | PRADOSHA_VRATA | 18 Oct | p. 14: शनिप्रदोषव्रत | 18 Oct | 18 Oct |
| 2082 | BUDHASHTAMI | 29 Oct | p. 15: बुधाष्टमीव्रत | 29 Oct | 12 Nov ✗ |
| 2082 | PRADOSHA_VRATA | 3 Nov | p. 15: सोमप्रदोषव्रत | 3 Nov | 3 Nov |
| 2082 | BUDHASHTAMI | 12 Nov | p. 16: बुधाष्टमीव्रत | 12 Nov | 12 Nov |
| 2082 | PRADOSHA_VRATA | 17 Nov | p. 16: सोमप्रदोषव्रत | 17 Nov | 17 Nov |
| 2082 | PRADOSHA_VRATA | 2 Dec | p. 17: प्रदोषव्रत | 2 Dec | 2 Dec |
| 2082 | PRADOSHA_VRATA | 17 Dec | p. 18: प्रदोषव्रत | 17 Dec | 17 Dec |
| 2082 | PRADOSHA_VRATA | 1 Jan | p. 19: प्रदोषव्रत | 1 Jan | 1 Jan |
| 2082 | PRADOSHA_VRATA | 16 Jan | p. 20: प्रदोषव्रत | 16 Jan | 16 Jan |
| 2082 | RAVI_SAPTAMI | 25 Jan | p. 21: रविसप्तमीव्रत | 25 Jan | 25 Jan |
| 2082 | PRADOSHA_VRATA | 30 Jan | p. 21: प्रदोषव्रत | 30 Jan | 30 Jan |
| 2082 | RAVI_SAPTAMI | 8 Feb | p. 22: रविसप्तमीव्रत | 8 Feb | 8 Feb |
| 2082 | PRADOSHA_VRATA | 14 Feb | p. 22: शनिप्रदोषव्रत | 14 Feb | 14 Feb |
| 2082 | BHAUMASHTAMI | 24 Feb | p. 23: भौमाष्टमीव्रत | 24 Feb | 24 Feb |
| 2082 | PRADOSHA_VRATA | 1 Mar | p. 23: प्रदोषव्रत | 1 Mar | 1 Mar |
| 2082 | BUDHASHTAMI | 11 Mar | p. 24: बुधाष्टमीव्रत | 11 Mar | 11 Mar |
| 2082 | PRADOSHA_VRATA | 16 Mar | p. 24: सोमप्रदोषव्रत | 16 Mar | 16 Mar |
| 2082 | PRADOSHA_VRATA | 30 Mar | p. 25: सोमप्रदोषव्रत | 30 Mar | 30 Mar |
| 2083 | PARASHURAMA_JAYANTI | 19 Apr | p. 3: परशुरामजयन्ती | 19 Apr | 19 Apr |
| 2083 | SHANKARACHARYA_JAYANTI | 21 Apr | p. 3: आद्यजगत्गुरु शङ्कराचार्यजयन्ती | 21 Apr | 21 Apr |
| 2083 | GANGA_SAPTAMI | 23 Apr | p. 3: गंगासप्तमी | 23 Apr | 23 Apr |
| 2083 | SITA_NAVAMI | 25 Apr | p. 3: सीताजयन्ती | 25 Apr | 25 Apr |
| 2083 | KURMA_JAYANTI | 1 May | p. 3: कूर्मजयन्ती | 1 May | 1 May |
| 2083 | DASHAHARA_SNANA_ARAMBHA | 17 May | p. 5: दशहरास्नानारम्भ | 17 May | 17 May |
| 2083 | GOSAIKUNDA_SNANA_ARAMBHA | 17 May | p. 5: गोसाइकुण्डस्नानारम्भ | 17 May | 17 May |
| 2083 | GANGA_DASHAHARA | 26 May | p. 5: गङ्गादशहरा | 26 May | 25 May ✗ |
| 2083 | GOSAIKUNDA_SNANA_SAMAPTI | 26 May | p. 5: गोसाइँकुण्ड स्नानसमाप्ति | 26 May | 25 May ✗ |
| 2083 | HATYAMOCHANA_SNANA | 26 May | p. 5: हत्यामोचन तीर्थस्नान | 26 May | 25 May ✗ |
| 2083 | ANALA_PUNHI | 31 May | p. 5: अनलापुन्हि | 31 May | 31 May |
| 2083 | KUMARA_JATRA | 21 Jun | p. 7: कुमारयात्रा | 21 Jun | 21 Jun |
| 2083 | NIRJALA_EKADASHI | 25 Jun | p. 7: निर्जला एकादशीव्रत | 25 Jun | 25 Jun |
| 2083 | DILA_CHARHE | 13 Jul | p. 8: (दिल्ला चःह्रे पूजा) | 13 Jul | 13 Jul |
| 2083 | GAURA_SAPTAMI | 3 Sep | p. 12: गौरासप्तमी | 18 Sep ✗ | 18 Sep ✗ |
| 2083 | AMUKTABHARANA_SAPTAMI | 3 Sep | p. 12: अमुक्ताभरणसप्तमी | 18 Sep ✗ | 18 Sep ✗ |
| 2083 | DURVA_ASHTAMI | 4 Sep | p. 12: दूर्वाष्टमीव्रत | 19 Sep ✗ | 19 Sep ✗ |
| 2083 | HILE_JATRA | 5 Sep | p. 12: रामेछाप चिसापानी हिलेजात्रा | 20 Sep ✗ | 20 Sep ✗ |
| 2083 | AJA_EKADASHI | 7 Sep | p. 12: अजाएकादशीव्रत | 7 Sep | 7 Sep |
| 2083 | CHATHA | 14 Sep | p. 13: चन्द्रदर्शनदोष (चथा) | 14 Sep | 14 Sep |
| 2083 | GANESHA_CHATURTHI | 14 Sep | p. 13: गणेशचतुर्थीव्रत | 14 Sep | 14 Sep |
| 2083 | RISHI_PANCHAMI | 15 Sep | p. 13: ऋषिपञ्चमीव्रत | 15 Sep | 15 Sep |
| 2083 | VAMANA_DWADASHI | 23 Sep | p. 13: वामनजयन्ती | 23 Sep | 23 Sep |
| 2083 | PRATIPADA_SHRADDHA | 27 Sep | p. 14: प्रतिपदाश्राद्ध | 27 Sep | 27 Sep |
| 2083 | SORHA_SHRADDHA_ARAMBHA | 27 Sep | p. 14: सोह्रश्राद्ध सुरु | 27 Sep | 27 Sep |
| 2083 | DWITIYA_SHRADDHA | 28 Sep | p. 14: द्वितीयाश्राद्ध | 28 Sep | 28 Sep |
| 2083 | TRITIYA_SHRADDHA | 29 Sep | p. 14: तृतीयाश्राद्ध | 29 Sep | 29 Sep |
| 2083 | CHATURTHI_SHRADDHA | 30 Sep | p. 14: चतुर्थीश्राद्ध | 30 Sep | 30 Sep |
| 2083 | PANCHAMI_SHRADDHA | 1 Oct | p. 14: पञ्चमीश्राद्ध | 1 Oct | 30 Sep ✗ |
| 2083 | SHASHTHI_SHRADDHA | 1 Oct | p. 14: षष्ठीश्राद्ध | 1 Oct | 1 Oct |
| 2083 | SAPTAMI_SHRADDHA | 2 Oct | p. 14: सप्तमीश्राद्ध | 2 Oct | 2 Oct |
| 2083 | ASHTAMI_SHRADDHA | 3 Oct | p. 14: अष्टमीश्राद्ध | 3 Oct | 3 Oct |
| 2083 | NAVAMI_SHRADDHA | 4 Oct | p. 14: नवमीश्राद्ध, मातृनवमीश्राद्ध | 4 Oct | 4 Oct |
| 2083 | DASHAMI_SHRADDHA | 5 Oct | p. 14: दशमीश्राद्ध | 5 Oct | 5 Oct |
| 2083 | EKADASHI_SHRADDHA | 6 Oct | p. 14: एकादशीश्राद्ध | 6 Oct | 6 Oct |
| 2083 | DWADASHI_SHRADDHA | 7 Oct | p. 14: द्वादशीश्राद्ध | 7 Oct | 7 Oct |
| 2083 | TRAYODASHI_SHRADDHA | 8 Oct | p. 14: त्रयोदशीश्राद्ध | 8 Oct | 8 Oct |
| 2083 | KALI_YUGADI | 8 Oct | p. 14: कलियुगादि | 8 Oct | 8 Oct |
| 2083 | CHATURDASHI_SHRADDHA | 9 Oct | p. 14: शस्त्रादिबाट निधन भएकाहरूको विश्वेदेवासहित एकोदिष्ट विधिले चतुर्दशीश्राद्ध | 9 Oct | 9 Oct |
| 2083 | AMAVASYA_SHRADDHA | 10 Oct | p. 14: औंसीश्राद्ध, पितृविसर्जन | 10 Oct | 10 Oct |
| 2083 | UPANGA_LALITA_VRATA | 15 Oct | p. 15: उपाङ्गललिताव्रत | 15 Oct | 15 Oct |
| 2083 | KUCHHI_BHOYE | 18 Oct | p. 15: (कूछि भ्वय्) | 18 Oct | 19 Oct ✗ |
| 2083 | DHANVANTARI_JAYANTI | 7 Nov | p. 16: धन्वन्तरी जयन्ती | 7 Nov | 7 Nov |
| 2083 | MHA_PUJA | 10 Nov | p. 17: म्हपूजा | 10 Nov | 10 Nov |
| 2083 | NEPAL_SAMBAT_NEW_YEAR | 10 Nov | p. 17: ने.सं.११४७ प्रारम्भ | 10 Nov | 10 Nov |
| 2083 | HARIBODHINI_EKADASHI | 20 Nov | p. 17: हरिबोधिनीएकादशीव्रत | 20 Nov | 21 Nov ✗ |
| 2083 | BHISHMA_PANCHAKA_ARAMBHA | 20 Nov | p. 17: भीष्मपञ्चकारम्भ | 20 Nov | 21 Nov ✗ |
| 2083 | TULASI_VIVAHA | 20 Nov | p. 17: तुलसीविवाह | 20 Nov | 21 Nov ✗ |
| 2083 | PASHUPATI_HALESI_MELA | 6 Dec | p. 18: हलेसीमहादेव मेला | 6 Dec | 6 Dec |
| 2083 | BALA_CHATURDASHI | 7 Dec | p. 18: बालाचतुर्दशी | 7 Dec | 7 Dec |
| 2083 | PISHACHAMOCHANA_SHRADDHA | 22 Dec | p. 19: काशीमा पिशाचमोचन तीर्थश्राद्ध | 22 Dec | 22 Dec |
| 2083 | DATTATREYA_JAYANTI | 23 Dec | p. 19: दत्तात्रेयजयन्ती | 23 Dec | 23 Dec |
| 2083 | NUWAKOT_DUPCHESHWAR_MELA | 24 Dec | p. 19: नुवाकोट दुप्चेश्वरमेला | 23 Dec ✗ | 23 Dec ✗ |
| 2083 | GAIDU_PUJA | 24 Dec | p. 19: गैडुपूजा | 24 Dec | 24 Dec |
| 2083 | PAUSHA_PUTRADA_EKADASHI | 18 Jan | p. 21: स्मार्तहरुको पुत्रदाएकादशीव्रत | 18 Jan | 19 Jan ✗ |
| 2083 | LAI_CHARHE | 5 Feb | p. 22: (लैचःह्रेपूजा) | 5 Feb | 5 Feb |
| 2083 | TILAKUNDA_CHATURTHI | 10 Feb | p. 23: तिलकुन्द चौथी | 10 Feb | 10 Feb |
| 2083 | BHIMA_EKADASHI | 17 Feb | p. 23: भीमा एकादशीव्रत | 17 Feb | 17 Feb |
| 2083 | VIJAYA_EKADASHI | 4 Mar | p. 24: विजया एकादशीव्रत | 4 Mar | 4 Mar |
| 2083 | SHILA_CHARHE | 6 Mar | p. 24: (शिलाचःह्रेपूजा) | 6 Mar | 6 Mar |
| 2083 | DVAPARA_YUGADI | 8 Mar | p. 24: द्वापरयुगादि | 8 Mar | 8 Mar |
| 2083 | PAHAN_CHARHE | 5 Apr | p. 26: (पाशाचःह्रेपूजा) | 5 Apr | 5 Apr |
| 2083 | GHODE_JATRA | 6 Apr | p. 26: घोडेजात्रा | 6 Apr | 6 Apr |
| 2083 | MATSYANARAYANA_MELA | 9 Apr | p. 27: मत्स्यनारायणमेला | 9 Apr | 9 Apr |
| 2083 | MATSYA_JAYANTI | 9 Apr | p. 27: मत्स्यजयन्ती | 9 Apr | 9 Apr |
| 2083 | PRADOSHA_VRATA | 15 Apr | p. 2: प्रदोषव्रत | 15 Apr | 15 Apr |
| 2083 | MANGALA_CHATURTHI | 21 Apr | p. 3: मंगलचौथीव्रत | 21 Apr | none ✗ |
| 2083 | PRADOSHA_VRATA | 29 Apr | p. 3: प्रदोषव्रत | 29 Apr | 28 Apr ✗ |
| 2083 | MANGALA_CHATURTHI | 5 May | p. 4: मंगलचौथीव्रत | 5 May | none ✗ |
| 2083 | PRADOSHA_VRATA | 14 May | p. 4: प्रदोषव्रत | 14 May | 14 May |
| 2083 | PRADOSHA_VRATA | 28 May | p. 5: प्रदोषव्रत | 28 May | 28 May |
| 2083 | RAVI_SAPTAMI | 7 Jun | p. 6: रविसप्तमीव्रत | 7 Jun | 7 Jun |
| 2083 | PRADOSHA_VRATA | 12 Jun | p. 6: प्रदोषव्रत | 12 Jun | 12 Jun |
| 2083 | SOMAVATI_AMAVASYA | 15 Jun | p. 6: सोमबारे औसी | 15 Jun | 15 Jun |
| 2083 | RAVI_SAPTAMI | 21 Jun | p. 7: रविसप्तमीव्रत | 21 Jun | 21 Jun |
| 2083 | PRADOSHA_VRATA | 27 Jun | p. 7: शनिप्रदोषव्रत | 27 Jun | 27 Jun |
| 2083 | BUDHASHTAMI | 8 Jul | p. 8: बुधाष्टमीव्रत | 8 Jul | 8 Jul |
| 2083 | PRADOSHA_VRATA | 12 Jul | p. 8: प्रदोषव्रत | 12 Jul | 12 Jul |
| 2083 | BUDHASHTAMI | 22 Jul | p. 9: बुधाष्टमीव्रत | 22 Jul | 8 Jul ✗ |
| 2083 | PRADOSHA_VRATA | 26 Jul | p. 9: प्रदोषव्रत | 26 Jul | 26 Jul |
| 2083 | PRADOSHA_VRATA | 10 Aug | p. 10: सोमप्रदोषव्रत | 10 Aug | 10 Aug |
| 2083 | PRADOSHA_VRATA | 25 Aug | p. 11: प्रदोषव्रत | 25 Aug | 25 Aug |
| 2083 | MANGALA_CHATURTHI | 1 Sep | p. 12: मंगलचौथीव्रत | 1 Sep | 1 Sep |
| 2083 | PRADOSHA_VRATA | 8 Sep | p. 12: प्रदोषव्रत | 8 Sep | 8 Sep |
| 2083 | MANGALA_CHATURTHI | 15 Sep | p. 13: मंगलचौथीव्रत | 15 Sep | 15 Sep |
| 2083 | PRADOSHA_VRATA | 24 Sep | p. 13: प्रदोषव्रत | 24 Sep | 24 Sep |
| 2083 | PRADOSHA_VRATA | 8 Oct | p. 14: प्रदोषव्रत | 8 Oct | 8 Oct |
| 2083 | PRADOSHA_VRATA | 23 Oct | p. 15: प्रदोषव्रत | 23 Oct | 23 Oct |
| 2083 | RAVI_SAPTAMI | 1 Nov | p. 16: रविसप्तमीव्रत | 1 Nov | 1 Nov |
| 2083 | PRADOSHA_VRATA | 6 Nov | p. 16: प्रदोषव्रत | 6 Nov | 6 Nov |
| 2083 | SOMAVATI_AMAVASYA | 9 Nov | p. 16: सोमबारे औसी | 9 Nov | 9 Nov |
| 2083 | BHAUMASHTAMI | 17 Nov | p. 17: भौमाष्टमीव्रत | 17 Nov | 17 Nov |
| 2083 | PRADOSHA_VRATA | 22 Nov | p. 17: प्रदोषव्रत | 22 Nov | 22 Nov |
| 2083 | BHAUMASHTAMI | 1 Dec | p. 18: भौमाष्टमीव्रत | 1 Dec | 1 Dec |
| 2083 | PRADOSHA_VRATA | 6 Dec | p. 18: प्रदोषव्रत | 6 Dec | 6 Dec |
| 2083 | PRADOSHA_VRATA | 21 Dec | p. 19: सोमप्रदोषव्रत | 21 Dec | 21 Dec |
| 2083 | PRADOSHA_VRATA | 5 Jan | p. 20: प्रदोषव्रत | 5 Jan | 5 Jan |
| 2083 | MANGALA_CHATURTHI | 12 Jan | p. 21: मंगलचौथीव्रत | 12 Jan | 12 Jan |
| 2083 | PRADOSHA_VRATA | 20 Jan | p. 21: प्रदोषव्रत | 20 Jan | 20 Jan |
| 2083 | MANGALA_CHATURTHI | 26 Jan | p. 22: मंगलचौथीव्रत | 26 Jan | 12 Jan ✗ |
| 2083 | PRADOSHA_VRATA | 3 Feb | p. 22: प्रदोषव्रत | 3 Feb | 3 Feb |
| 2083 | PRADOSHA_VRATA | 18 Feb | p. 23: प्रदोषव्रत | 18 Feb | 18 Feb |
| 2083 | PRADOSHA_VRATA | 5 Mar | p. 24: प्रदोषव्रत | 5 Mar | 5 Mar |
| 2083 | BHAUMASHTAMI | 16 Mar | p. 25: भौमाष्टमीव्रत | 16 Mar | 16 Mar |
| 2083 | PRADOSHA_VRATA | 20 Mar | p. 25: शनिप्रदोषव्रत | 20 Mar | 20 Mar |
| 2083 | BHAUMASHTAMI | 30 Mar | p. 26: भौमाष्टमीव्रत | 30 Mar | 30 Mar |
| 2083 | PRADOSHA_VRATA | 4 Apr | p. 26: प्रदोषव्रत | 4 Apr | 4 Apr |
| 2082 | CHANDRODAYA | 29 Apr | p. 3: चन्द्रोदय | 29 Apr | 29 Apr |
| 2082 | CHANDRODAYA | 28 May | p. 5: चन्द्रोदय | 28 May | 28 May |
| 2082 | CHANDRODAYA | 26 Jun | p. 7: चन्द्रोदय | 26 Jun | 26 Jun |
| 2082 | CHANDRODAYA | 26 Jul | p. 9: चन्द्रोदय | 26 Jul | 26 Jul |
| 2082 | CHANDRODAYA | 24 Aug | p. 11: चन्द्रोदय | 24 Aug | 24 Aug |
| 2082 | CHANDRODAYA | 23 Sep | p. 13: चन्द्रोदय | 23 Sep | 23 Sep |
| 2082 | CHANDRODAYA | 23 Oct | p. 15: चन्द्रोदय | 23 Oct | 23 Oct |
| 2082 | CHANDRODAYA | 21 Nov | p. 17: चन्द्रोदय | 21 Nov | 21 Nov |
| 2082 | CHANDRODAYA | 21 Dec | p. 19: चन्द्रोदय | 21 Dec | 21 Dec |
| 2082 | CHANDRODAYA | 20 Jan | p. 21: चन्द्रोदय | 20 Jan | 20 Jan |
| 2082 | CHANDRODAYA | 19 Feb | p. 23: चन्द्रोदय | 19 Feb | 19 Feb |
| 2082 | CHANDRODAYA | 20 Mar | p. 25: चन्द्रोदय | 20 Mar | 20 Mar |
| 2083 | CHANDRODAYA | 18 Apr | p. 3: चन्द्रोदय | 18 Apr | 18 Apr |
| 2083 | CHANDRODAYA | 18 May | p. 5: चन्द्रोदय | 18 May | 18 May |
| 2083 | CHANDRODAYA | 16 Jun | p. 7: चन्द्रोदय | 16 Jun | 16 Jun |
| 2083 | CHANDRODAYA | 15 Jul | p. 9: चन्द्रोदय | 15 Jul | 15 Jul |
| 2083 | CHANDRODAYA | 14 Aug | p. 11: चन्द्रोदय | 14 Aug | 14 Aug |
| 2083 | CHANDRODAYA | 12 Sep | p. 13: चन्द्रोदय | 12 Sep | 12 Sep |
| 2083 | CHANDRODAYA | 12 Oct | p. 15: चन्द्रोदय | 12 Oct | 12 Oct |
| 2083 | CHANDRODAYA | 10 Nov | p. 17: चन्द्रोदय | 10 Nov | 10 Nov |
| 2083 | CHANDRODAYA | 10 Dec | p. 19: चन्द्रोदय | 10 Dec | 10 Dec |
| 2083 | CHANDRODAYA | 9 Jan | p. 21: चन्द्रोदय | 9 Jan | 9 Jan |
| 2083 | CHANDRODAYA | 8 Feb | p. 23: चन्द्रोदय | 8 Feb | 8 Feb |
| 2083 | CHANDRODAYA | 9 Mar | p. 25: चन्द्रोदय | 9 Mar | 9 Mar |
| 2083 | CHANDRODAYA | 8 Apr | p. 27: चन्द्रोदय | 8 Apr | 8 Apr |

| proposed rule | verdict | measured |
|---|---|---|
| the `NEPAL_PANCHANGA` pack over the committee's sky falls on the printed day | falsified | 6 of 236 disagree; each parting is named below with its cause |
| the same over the modern sky falls on the printed day | falsified | 33 of 236 disagree; the sky decides these: a tithi's end moves the day where the two skies part |

Where the pack over the committee's sky parts from the print, and why:

- VS 2082 HILE_JATRA (1 Sep): the print's हिलेजात्रा is Ramechhap's, on Shravana's dark 9th; the rule's Bhadrapada bright 9th prints no Hile jatra (C609)
- VS 2083 AMUKTABHARANA_SAPTAMI (18 Sep): counted from GAURA_SAPTAMI (C608)
- VS 2083 DURVA_ASHTAMI (19 Sep): the committee prints Shravana's dark 8th, inside the solar Bhadra, where Bhadrapada's bright 8th falls in the solar Ashwin (C608)
- VS 2083 GAURA_SAPTAMI (18 Sep): the committee prints Shravana's dark 7th, inside the solar Bhadra, where Bhadrapada's bright 7th falls in the solar Ashwin (C608)
- VS 2083 HILE_JATRA (20 Sep): the print's हिलेजात्रा is Ramechhap's, on Shravana's dark 9th; the rule's Bhadrapada bright 9th prints no Hile jatra (C609)
- VS 2083 NUWAKOT_DUPCHESHWAR_MELA (23 Dec): the committee prints the mela on the full moon, a day after Dattatreya's 14th (C611)
