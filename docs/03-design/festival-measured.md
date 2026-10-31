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
| SANKASHTA_CHATURTHI | Both 2, EarlierOnly 57, LaterOnly 23, Neither 9 | otherwise: 32, guard 0: 57, guard 1: 2 |
| KARAKA_CHATURTHI | EarlierOnly 2, LaterOnly 7, Neither 1 | otherwise: 8, guard 0: 2 |
| MAHA_NAVAMI | EarlierOnly 5, LaterOnly 4, UnequalParts 1 | otherwise: 6, guard 0: 4 |
| DEVI_VISARJAN | LaterOnly 5, UnequalParts 5 | otherwise: 9, guard 0: 1 |

The guards no year reached, and why:

- VIJAYA_DASHAMI guard 1: the 10th beginning after the earlier day's aparahna with Shravana joining it only that evening, which no year of the decade had; the crate test `vijaya_dashami_held_later_alone_yields_to_shravana_on_the_earlier_evening` reaches it.
- VIJAYA_DASHAMI guard 6: the 10th holding both aparahnas, neither or each in part, with Shravana joining it on the later day only, which no year of the decade had; the crate test `vijaya_dashami_held_on_both_days_or_neither_goes_to_shravana_alone` reaches it.
- DEVI_VISARJAN guard 1: the 10th holding the earlier morning and ending before the later sunrise, a tithi shorter than a day begun in daylight, which no Ashvina of the decade had; the crate test `devi_visarjan_keeps_a_tenth_that_never_reaches_the_later_morning` reaches it.
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
`EkadashiRule::dharmasindhu()` (`festival-rules.md` §8) and
`EkadashiRule::hari_bhakti_vilasa()` (§8.9), the Gaudiya rule with its
mahadvadashis, which the Vaishnava's partings are.

| proposed rule | verdict | measured |
|---|---|---|
| EKADASHI_SMARTA falls on the Smarta list's day | **holds** | 0 of 99 disagree |
| EKADASHI_SMARTA_RENUNCIANT falls on the Gauna list's day | **holds** | 0 of 99 disagree |
| EKADASHI_VAISHNAVA falls on the ISKCON list's day | falsified | 3 of 99 disagree; each parting is named below with its cause |
| EKADASHI_GAUDIYA falls on the ISKCON list's day | falsified | 1 of 99 disagree; each parting is named below with its cause |

Where a rule parts from the record, and why:

- 2023 EKADASHI_VAISHNAVA: published 11 Sep, found 10 Sep: the ISKCON list's Paksha Vardhini Mahadvadashi: the fortnight's new or full moon holds two sunrises, which the pass finds so, and the fast moves to the 12th's day, a rule *Dharmasindhu* does not state
- 2024 EKADASHI_VAISHNAVA: published 30 Aug, found 29 Aug: the ISKCON list's Paksha Vardhini Mahadvadashi: the fortnight's new or full moon holds two sunrises, which the pass finds so, and the fast moves to the 12th's day, a rule *Dharmasindhu* does not state
- 2025 EKADASHI_VAISHNAVA: published 16 Dec, found 15 Dec: the ISKCON list's Paksha Vardhini Mahadvadashi: the fortnight's new or full moon holds two sunrises, which the pass finds so, and the fast moves to the 12th's day, a rule *Dharmasindhu* does not state
- 2026 EKADASHI_GAUDIYA: published 25 Jun, found 26 Jun: the day is decided within a minute of an edge it reads, which the pass finds so, finer than the minute the list prints its tithis to

## 4. Against Nepal's national panchanga

The days the Nepal Panchanga Decision Committee printed in its national
panchanga for VS 2082 and 2083, read off the page images, at Kathmandu on
Nepal's clock, beside four readings of each observance (Holi in the hills
and the Terai counted from the Holika day, §9.4): the text's rules over
the committee's own sky (`nepali-committee`, the Surya Siddhanta with its
bija, C187), each parting named with its cause; the same over the modern
sky (`nepali-drik`); every rule read as the day whose sunrise holds
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
cause, and over the modern sky (`nepali-drik`). Each rule is fitted:
its window and table were chosen to give these days, so the agreement
measures the fit and not a text (C600).

| VS | rule | printed | where | committee sky | modern sky |
|---:|---|---|---|---|---|
| 2082 | VARUTHINI_EKADASHI | 24 Apr | p. 2: बरुथिनीएकादशीव्रत | 24 Apr | 24 Apr |
| 2082 | VARUTHINI_EKADASHI | 13 Apr | p. 26: बरुथिनी एकादशीव्रत | 13 Apr | 13 Apr |
| 2082 | YOGINI_EKADASHI | 21 Jun | p. 6: स्मार्तहरूको योगिनी एकादशीव्रत | 21 Jun | 21 Jun |
| 2083 | YOGINI_EKADASHI | 10 Jul | p. 8: स्मार्तहरुको योगिनी एकादशीव्रत | 10 Jul | 11 Jul ✗ |
| 2082 | GYALPO_LHOSAR | 18 Feb | p. 23: ग्याल्पो ल्होसार | 18 Feb | 18 Feb |
| 2083 | GYALPO_LHOSAR | 9 Mar | p. 25: चन्द्रोदय, ग्याल्पो ल्होसार | 9 Mar | 9 Mar |
| 2082 | SONAM_LHOSAR | 19 Jan | p. 21: सोनम ल्होसार/तामाङ ल्होसार | 19 Jan | 19 Jan |
| 2083 | SONAM_LHOSAR | 7 Feb | p. 23: सोनम ल्होसार/तामाङ ल्होछार | 7 Feb | 7 Feb |
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
| 2083 | PADMINI_EKADASHI | 27 May | p. 5: पद्मिनी एकादशीव्रत | 27 May | 27 May |
| 2083 | ANALA_PUNHI | 31 May | p. 5: अनलापुन्हि | 31 May | 31 May |
| 2083 | PARAMA_EKADASHI | 11 Jun | p. 6: परमा एकादशीव्रत | 11 Jun | 11 Jun |
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
| 2082 | SAMVATSARA_ARAMBHA | 19 Mar | p. 24: वत्सरारम्भ | 19 Mar | 19 Mar |
| 2083 | SAMVATSARA_ARAMBHA | 7 Apr | p. 27: वत्सरारम्भ | 7 Apr | 7 Apr |
| 2082 | GAURI_VRAT | 21 Mar | p. 25: गौरीव्रत | 21 Mar | 21 Mar |
| 2083 | GAURI_VRAT | 9 Apr | p. 27: गौरीव्रत | 9 Apr | 9 Apr |
| 2082 | MAHAVIR_JAYANTI | 31 Mar | p. 25: महावीरजयन्ती | 31 Mar | 31 Mar |
| 2082 | BALAJU_BAISDHARA_MELA | 2 Apr | p. 25: बालाजु बाइसधारा मेला | 2 Apr | 2 Apr |
| 2082 | HANUMAN_JAYANTI | 2 Apr | p. 25: हनुमानजयन्ती | 2 Apr | 2 Apr |
| 2082 | BISHALNAGAR_VAISHNAVI_DEVI_JATRA | 3 Apr | p. 26: विशालनगरमा वैष्णवीदेवीयात्रा | 3 Apr | 3 Apr |
| 2082 | CHANGUNARAYAN_CHHINNAMASTA_KILESHWOR_MULRATH_JATRA | 27 Apr | p. 2: चाँगुनारायण छिन्नमस्ता कीलेश्वर मूलरथयात्रा | 27 Apr | 27 Apr |
| 2083 | CHANGUNARAYAN_CHHINNAMASTA_KILESHWOR_MULRATH_JATRA | 17 Apr | p. 2: चाँगुनारायण छिन्नमस्ता कीलेश्वर मूलरथयात्रा | 17 Apr | 17 Apr |
| 2082 | MATA_TIRTHA_AUNSI | 27 Apr | p. 2: मातातीर्थस्नान | 27 Apr | 27 Apr |
| 2083 | MATA_TIRTHA_AUNSI | 17 Apr | p. 2: मातातीर्थस्नान | 17 Apr | 17 Apr |
| 2082 | RATO_MACHHINDRANATH_RATHAROHAN | 28 Apr | p. 3: ल.पु.मत्स्येन्द्रनाथ रथारोहण | 28 Apr | 28 Apr |
| 2083 | RATO_MACHHINDRANATH_RATHAROHAN | 18 Apr | p. 3: ल.पु.मत्स्येन्द्रनाथ रथारोहण | 18 Apr | 18 Apr |
| 2082 | AKSHAYA_TRITIYA | 30 Apr | p. 3: अक्षयतृतीया | 30 Apr | 30 Apr |
| 2083 | AKSHAYA_TRITIYA | 20 Apr | p. 3: अक्षयतृतीया | 20 Apr | 20 Apr |
| 2082 | TRETAYUGADI | 30 Apr | p. 3: त्रेतायुगादि | 30 Apr | 30 Apr |
| 2083 | TRETAYUGADI | 20 Apr | p. 3: त्रेतायुगादि | 20 Apr | 20 Apr |
| 2082 | RATO_MACHHINDRANATH_RATH_JATRA_BEGINS | 1 May | p. 3: ल.पु.श्रीमत्स्येन्द्रनाथ रथयात्रारम्भ | 1 May | 1 May |
| 2083 | RATO_MACHHINDRANATH_RATH_JATRA_BEGINS | 21 Apr | p. 3: ल.पु.श्रीमत्स्येन्द्रनाथ रथयात्रारम्भ | 21 Apr | 20 Apr ✗ |
| 2082 | PASHUPATINATH_DAMAN_PUSHPARPAN | 5 May | p. 3: श्रीपशुपतिनाथमा दमनार्पण | 5 May | 5 May |
| 2083 | PASHUPATINATH_DAMAN_PUSHPARPAN | 24 Apr | p. 3: श्रीपशुपतिनाथमा दमनार्पण | 24 Apr | 24 Apr |
| 2082 | MOHINI_EKADASHI | 8 May | p. 3: मोहिनी एकादशीव्रत | 8 May | 8 May |
| 2083 | MOHINI_EKADASHI | 27 Apr | p. 3: मोहिनीएकादशीव्रत | 27 Apr | 27 Apr |
| 2082 | BAISAKH_SNAN_ENDS | 12 May | p. 3: वैशाखस्नानसमाप्ति | 12 May | 12 May |
| 2083 | BAISAKH_SNAN_ENDS | 1 May | p. 3: वैशाखस्नानसमाप्ति | 1 May | 1 May |
| 2082 | BUDDHA_JAYANTI | 12 May | p. 3: गौतमबुद्धजयन्ती | 12 May | 12 May |
| 2083 | BUDDHA_JAYANTI | 1 May | p. 3: गौतमबुद्धजयन्ती | 1 May | 1 May |
| 2082 | CHANDI_PURNIMA | 12 May | p. 3: चण्डीपूर्णिमा | 12 May | 12 May |
| 2083 | CHANDI_PURNIMA | 1 May | p. 3: चण्डीपूर्णिमा | 1 May | 1 May |
| 2082 | GORAKHNATH_JAYANTI | 12 May | p. 3: गोरखनाथजयन्ती | 12 May | 12 May |
| 2083 | GORAKHNATH_JAYANTI | 1 May | p. 3: गोरखनाथजयन्ती | 1 May | 1 May |
| 2082 | UBHAULI | 12 May | p. 3: उभौलीपर्व | 12 May | 12 May |
| 2083 | UBHAULI | 1 May | p. 3: उभौलीपर्व | 1 May | 1 May |
| 2082 | APARA_EKADASHI | 23 May | p. 4: अपराएकादशीव्रत | 23 May | 23 May |
| 2083 | APARA_EKADASHI | 13 May | p. 4: अपराएकादशीव्रत | 13 May | 13 May |
| 2082 | SHANI_JAYANTI | 27 May | p. 4: शनिजयन्ती | 27 May | 27 May |
| 2083 | SHANI_JAYANTI | 16 May | p. 4: शनिजयन्ती | 16 May | 16 May |
| 2082 | BHAKTAPUR_CHANDI_BHAGAWATI_JATRA | 1 Jun | p. 5: भ.पु.चण्डीभगवतीयात्रा | 1 Jun | 1 Jun |
| 2083 | BHAKTAPUR_CHANDI_BHAGAWATI_JATRA | 20 Jun | p. 7: भ.पु.चण्डीभगवतीयात्रा | 20 Jun | 20 Jun |
| 2082 | KUMAR_SHASHTHI | 1 Jun | p. 5: कुमारषष्ठी (सिथिनख) | 1 Jun | 1 Jun |
| 2083 | KUMAR_SHASHTHI | 20 Jun | p. 7: कुमारषष्ठी | 20 Jun | 20 Jun |
| 2082 | SITHI_NAKHA | 1 Jun | p. 5: कुमारषष्ठी (सिथिनख) | 1 Jun | 1 Jun |
| 2083 | SITHI_NAKHA | 20 Jun | p. 7: (सिथिनख) | 20 Jun | 20 Jun |
| 2082 | VAYU_ASHTAMI | 3 Jun | p. 5: वायुअष्टमी | 3 Jun | 3 Jun |
| 2083 | VAYU_ASHTAMI | 22 Jun | p. 7: वायु अष्टमी | 22 Jun | 22 Jun |
| 2082 | GANGA_PUJA | 5 Jun | p. 5: रामेश्वरगङ्गापूजा | 5 Jun | 5 Jun |
| 2083 | GANGA_PUJA | 26 May | p. 5: रामेश्वरगङ्गापूजा | 26 May | 25 May ✗ |
| 2082 | GURU_PADMASAMBHAVA_JAYANTI | 5 Jun | p. 5: वज्रगुरु पद्मसम्भवजयन्ती | 5 Jun | 5 Jun |
| 2083 | GURU_PADMASAMBHAVA_JAYANTI | 26 May | p. 5: वज्रगुरु पद्मसम्भवजयन्ती | 26 May | 25 May ✗ |
| 2082 | RAMESHWAR_PUJA | 5 Jun | p. 5: रामेश्वरगङ्गापूजा | 5 Jun | 5 Jun |
| 2083 | RAMESHWAR_PUJA | 26 May | p. 5: रामेश्वरगङ्गापूजा | 26 May | 25 May ✗ |
| 2082 | JYA_PUNHI | 11 Jun | p. 5: (ज्याः पुन्हि) | 11 Jun | 11 Jun |
| 2083 | JYA_PUNHI | 29 Jun | p. 7: (ज्याः पुन्हि) | 29 Jun | 29 Jun |
| 2082 | KABIR_JAYANTI | 11 Jun | p. 5: कवीरजयन्ती | 11 Jun | 11 Jun |
| 2083 | KABIR_JAYANTI | 29 Jun | p. 7: कवीरजयन्ती | 29 Jun | 29 Jun |
| 2082 | MASTA_PURNIMA | 11 Jun | p. 5: मष्टपूर्णिमा | 11 Jun | 11 Jun |
| 2083 | MASTA_PURNIMA | 29 Jun | p. 7: मष्टपूर्णिमा | 29 Jun | 29 Jun |
| 2082 | PANAUTI_SNAN_MELA | 11 Jun | p. 5: पनौतीस्नान | 11 Jun | 11 Jun |
| 2083 | PANAUTI_SNAN_MELA | 29 Jun | p. 7: पनौतीस्नान | 29 Jun | 29 Jun |
| 2082 | PANAUTI_JATRA | 11 Jun | p. 5: पनौती भैरवभद्रकाली इन्द्रेश्वर रथयात्रा | 11 Jun | 11 Jun |
| 2083 | PANAUTI_JATRA | 29 Jun | p. 7: Ⓑ पनौती भैरवभद्रकाली इन्द्रेश्वररथयात्रा | 29 Jun | 29 Jun |
| 2082 | BHAL_BHAL_ASHTAMI | 19 Jun | p. 6: भलभलअष्टमी | 19 Jun | 19 Jun |
| 2083 | BHAL_BHAL_ASHTAMI | 8 Jul | p. 8: भलभलअष्टमी | 8 Jul | 8 Jul |
| 2082 | DEOPATAN_TRISHUL_JATRA | 19 Jun | p. 6: देउपाटनमा त्रिशूलयात्रा | 19 Jun | 19 Jun |
| 2083 | DEOPATAN_TRISHUL_JATRA | 8 Jul | p. 8: देउपाटनमा त्रिशूलयात्रा | 8 Jul | 8 Jul |
| 2082 | NAKWA_DISHI | 20 Jun | p. 6: नक्वादिशी | 20 Jun | 21 Jun ✗ |
| 2083 | NAKWA_DISHI | 9 Jul | p. 8: नक्वादिशी | 9 Jul | 10 Jul ✗ |
| 2082 | JAGANNATH_RATH_JATRA | 27 Jun | p. 7: श्रीजगन्नाथरथयात्रा | 27 Jun | 27 Jun |
| 2083 | JAGANNATH_RATH_JATRA | 16 Jul | p. 9: श्रीजगन्नाथरथयात्रा | 16 Jul | 16 Jul |
| 2082 | DEOPATAN_GANGAMAI_RATH_JATRA | 2 Jul | p. 7: देउपाटनमा गङ्गामाई रथयात्रा | 2 Jul | 2 Jul |
| 2083 | DEOPATAN_GANGAMAI_RATH_JATRA | 21 Jul | p. 9: देउपाटनमा गङ्गामाई रथयात्रा | 21 Jul | 20 Jul ✗ |
| 2082 | SURYA_PUJA | 2 Jul | p. 7: सूर्यपूजा | 2 Jul | 2 Jul |
| 2083 | SURYA_PUJA | 21 Jul | p. 9: सूर्यपूजा | 21 Jul | 20 Jul ✗ |
| 2082 | CHATURMAS_BEGINS | 6 Jul | p. 7: चतुर्मासव्रत सुरु | 6 Jul | 6 Jul |
| 2083 | CHATURMAS_BEGINS | 25 Jul | p. 9: चतुर्मासव्रत सुरु | 25 Jul | 25 Jul |
| 2082 | HARISHAYANI_EKADASHI | 6 Jul | p. 7: हरिशयनीएकादशीव्रत | 6 Jul | 6 Jul |
| 2083 | HARISHAYANI_EKADASHI | 25 Jul | p. 9: हरिशयनी एकादशीव्रत | 25 Jul | 25 Jul |
| 2082 | DAKSHINAMURTI_PUJA | 10 Jul | p. 7: दक्षिणामूर्तिपूजा | 10 Jul | 10 Jul |
| 2083 | DAKSHINAMURTI_PUJA | 29 Jul | p. 9: दक्षिणामूर्तिपूजा | 29 Jul | 29 Jul |
| 2082 | DILLA_PUNHI | 10 Jul | p. 7: (दिल्लापून्हि) | 10 Jul | 10 Jul |
| 2083 | DILLA_PUNHI | 29 Jul | p. 9: (दिल्लापून्हि) | 29 Jul | 29 Jul |
| 2082 | GURU_PURNIMA | 10 Jul | p. 7: गुरुपूर्णिमा | 10 Jul | 10 Jul |
| 2083 | GURU_PURNIMA | 29 Jul | p. 9: गुरुपूर्णिमा | 29 Jul | 29 Jul |
| 2082 | KAMIKA_EKADASHI | 21 Jul | p. 8: कामिकाएकादशीव्रत | 21 Jul | 21 Jul |
| 2083 | KAMIKA_EKADASHI | 9 Aug | p. 10: कामिकाएकादशीव्रत | 9 Aug | 9 Aug |
| 2082 | GUNLA_BEGINS | 25 Jul | p. 9: गुँलाधर्मारम्भ | 25 Jul | 25 Jul |
| 2083 | GUNLA_BEGINS | 13 Aug | p. 11: गुँलाधर्मारम्भ | 13 Aug | 13 Aug |
| 2082 | DHAPAKHEL_NAGDAHA | 29 Jul | p. 9: धापाखेल नागदह | 29 Jul | 29 Jul |
| 2083 | DHAPAKHEL_NAGDAHA | 17 Aug | p. 11: धापाखेल नागदह | 17 Aug | 17 Aug |
| 2082 | NAG_PANCHAMI | 29 Jul | p. 9: नागपञ्चमी | 29 Jul | 29 Jul |
| 2083 | NAG_PANCHAMI | 17 Aug | p. 11: नागपञ्चमी | 17 Aug | 17 Aug |
| 2082 | TULSIDAS_JAYANTI | 31 Jul | p. 9: गोस्वामी तुलसीदासजयन्ती | 31 Jul | 31 Jul |
| 2083 | TULSIDAS_JAYANTI | 19 Aug | p. 11: गोस्वामी तुलसीदास जयन्ती | 19 Aug | 19 Aug |
| 2082 | YALA_PANCHADAN | 1 Aug | p. 9: (यल पञ्चदान) | 1 Aug | 1 Aug |
| 2083 | YALA_PANCHADAN | 20 Aug | p. 11: (यँलपञ्चदान) | 20 Aug | 20 Aug |
| 2082 | SHRAWAN_PUTRADA_EKADASHI | 5 Aug | p. 9: पुत्रदाएकादशीव्रत | 5 Aug | 5 Aug |
| 2083 | SHRAWAN_PUTRADA_EKADASHI | 23 Aug | p. 11: पुत्रदाएकादशीव्रत | 23 Aug | 23 Aug |
| 2082 | KWATI_PUNHI | 9 Aug | p. 9: व्याँजानके गुँ पुन्हि | 9 Aug | 9 Aug |
| 2083 | KWATI_PUNHI | 28 Aug | p. 11: व्याँजानके गुँ पुन्हि | 28 Aug | 28 Aug |
| 2082 | SANSKRIT_DAY | 9 Aug | p. 9: संस्कृतदिवस | 9 Aug | 9 Aug |
| 2083 | SANSKRIT_DAY | 28 Aug | p. 11: संस्कृतदिवस | 28 Aug | 28 Aug |
| 2082 | GAI_JATRA | 10 Aug | p. 10: गाईजात्रा | 10 Aug | 10 Aug |
| 2083 | GAI_JATRA | 29 Aug | p. 12: गाईजात्रा | 29 Aug | 29 Aug |
| 2082 | ROPAI_JATRA | 11 Aug | p. 10: रोपाईजात्रा | 11 Aug | 11 Aug |
| 2083 | ROPAI_JATRA | 30 Aug | p. 12: रोपाईजात्रा | 30 Aug | 30 Aug |
| 2083 | CHANDRA_SHASHTHI | 2 Sep | p. 12: चन्द्रषष्ठी | 2 Sep | 2 Sep |
| 2082 | GUGA_NAVAMI | 17 Aug | p. 10: गूँगानवमी | 17 Aug | 17 Aug |
| 2083 | GUGA_NAVAMI | 5 Sep | p. 12: गूँगानवमी | 5 Sep | 5 Sep |
| 2082 | KUSHE_AUNSI | 23 Aug | p. 10: कुशेऔंसी, बाबुको मुख हेर्नेदिन | 23 Aug | 23 Aug |
| 2083 | KUSHE_AUNSI | 11 Sep | p. 12: कुशेऔंसी, बाबुको मुख हेर्नेदिन | 11 Sep | 11 Sep |
| 2082 | MOTIRAM_JAYANTI | 23 Aug | p. 10: मोतिजयन्ती | 23 Aug | 23 Aug |
| 2083 | MOTIRAM_JAYANTI | 11 Sep | p. 12: मोतीजयन्ती | 11 Sep | 11 Sep |
| 2082 | GUNLA_ENDS | 24 Aug | p. 11: गुँलाधर्मसमाप्ति | 24 Aug | 24 Aug |
| 2083 | GUNLA_ENDS | 12 Sep | p. 13: गुँलाधर्मसमाप्ति | 12 Sep | 12 Sep |
| 2082 | DAR_KHANE_DIN | 25 Aug | p. 11: दरखाने दिन | 25 Aug | 25 Aug |
| 2083 | DAR_KHANE_DIN | 13 Sep | p. 13: दरखाने दिन | 13 Sep | 13 Sep |
| 2082 | KOKHAJA_BIYE | 28 Aug | p. 11: (क्वखजाविय) | 28 Aug | 28 Aug |
| 2083 | KOKHAJA_BIYE | 16 Sep | p. 13: (क्वखजाविय) | 16 Sep | 16 Sep |
| 2082 | SURYA_SHASHTHI | 29 Aug | p. 11: सूर्यषष्ठी | 29 Aug | 29 Aug |
| 2083 | SURYA_SHASHTHI | 17 Sep | p. 13: सूर्यषष्ठी | 17 Sep | 17 Sep |
| 2082 | KAYA_ASHTAMI | 31 Aug | p. 11: काय अष्टमी | 31 Aug | 31 Aug |
| 2083 | KAYA_ASHTAMI | 19 Sep | p. 13: काय अष्टमी | 19 Sep | 19 Sep |
| 2082 | ADUKHA_NAVAMI | 1 Sep | p. 11: अदुःखनवमीव्रत | 1 Sep | 1 Sep |
| 2083 | ADUKHA_NAVAMI | 20 Sep | p. 13: अदुःखनवमीव्रत | 20 Sep | 20 Sep |
| 2082 | HARIPARIVARTINI_EKADASHI | 3 Sep | p. 11: हरिपरिवर्तिनी एकादशीव्रत | 3 Sep | 3 Sep |
| 2083 | HARIPARIVARTINI_EKADASHI | 22 Sep | p. 13: हरिपरिवर्तिनी एकादशीव्रत | 22 Sep | 22 Sep |
| 2082 | INDRADHWAJA_UTTHAN | 4 Sep | p. 11: इन्द्रध्वजोत्थान (मतविए) | 4 Sep | 4 Sep |
| 2083 | INDRADHWAJA_UTTHAN | 23 Sep | p. 13: इन्द्रध्वजोत्थान (मतविए) | 23 Sep | 23 Sep |
| 2082 | ANANTA_CHATURDASHI | 6 Sep | p. 11: अनन्त चतुर्दशीव्रत | 6 Sep | 6 Sep |
| 2083 | ANANTA_CHATURDASHI | 25 Sep | p. 13: अनन्त चतुर्दशीव्रत | 25 Sep | 25 Sep |
| 2082 | INDRA_JATRA | 6 Sep | p. 11: कुमारीइन्द्रयात्रा | 6 Sep | 6 Sep |
| 2083 | INDRA_JATRA | 25 Sep | p. 13: कुमारीइन्द्रयात्रा | 25 Sep | 25 Sep |
| 2082 | BHAKTAPUR_SIDDHA_POKHARI_MELA | 7 Sep | p. 11: भ.पु.सिद्धपोखरीमेला | 7 Sep | 7 Sep |
| 2083 | BHAKTAPUR_SIDDHA_POKHARI_MELA | 26 Sep | p. 13: भ.पु.सिद्धपोखरीमेला | 26 Sep | 26 Sep |
| 2082 | CHONAM_PARVA | 7 Sep | p. 11: चेपाङचोनामपर्व | 7 Sep | 7 Sep |
| 2083 | CHONAM_PARVA | 26 Sep | p. 13: चेपाङ-चोनामपर्व | 26 Sep | 26 Sep |
| 2082 | INDRADAHA_SNAN | 7 Sep | p. 11: इन्द्रदहस्नान | 7 Sep | 7 Sep |
| 2083 | INDRADAHA_SNAN | 26 Sep | p. 13: इन्द्रदहस्नान | 26 Sep | 26 Sep |
| 2082 | PUNCHALI_BHUJA | 8 Sep | p. 12: पुञ्चलीभूजा | 8 Sep | 8 Sep |
| 2083 | PUNCHALI_BHUJA | 27 Sep | p. 14: पुञ्चलीभूजा | 27 Sep | 27 Sep |
| 2082 | GATILA | 10 Sep | p. 12: (गातिला) | 10 Sep | 10 Sep |
| 2083 | GATILA | 29 Sep | p. 14: (गातिला) | 29 Sep | 29 Sep |
| 2082 | JITIYA | 15 Sep | p. 12: (जितियापर्व) | 15 Sep | 14 Sep ✗ |
| 2083 | JITIYA | 4 Oct | p. 14: (जितियापर्व) | 4 Oct | 4 Oct |
| 2082 | INDIRA_EKADASHI | 17 Sep | p. 12: इन्दिराएकादशीव्रत | 17 Sep | 17 Sep |
| 2083 | INDIRA_EKADASHI | 6 Oct | p. 14: इन्दिराएकादशीव्रत | 6 Oct | 6 Oct |
| 2082 | MAGHA_SHRADDHA | 19 Sep | p. 12: मघाश्राद्ध | 19 Sep | 19 Sep |
| 2083 | MAGHA_SHRADDHA | 8 Oct | p. 14: मघाश्राद्ध | 8 Oct | 8 Oct |
| 2082 | PACHALI_BHAIRAV_JATRA | 27 Sep | p. 13: पचलीभैरवयात्रा | 27 Sep | 27 Sep |
| 2083 | PACHALI_BHAIRAV_JATRA | 15 Oct | p. 15: पचलीभैरवयात्रा | 15 Oct | 15 Oct |
| 2082 | BILVA_NIMANTRAN | 28 Sep | p. 13: विल्व निमन्त्रण | 28 Sep | 28 Sep |
| 2083 | BILVA_NIMANTRAN | 16 Oct | p. 15: बिल्वनिमन्त्रण | 16 Oct | 16 Oct |
| 2082 | PHULPATI | 29 Sep | p. 13: फूलपाती | 29 Sep | 29 Sep |
| 2083 | PHULPATI | 17 Oct | p. 15: फूलपाती | 17 Oct | 18 Oct ✗ |
| 2082 | MAHA_ASHTAMI | 30 Sep | p. 13: महाअष्टमीव्रत | 30 Sep | 30 Sep |
| 2083 | MAHA_ASHTAMI | 18 Oct | p. 15: महाष्टमीव्रत | 18 Oct | 19 Oct ✗ |
| 2082 | BUDDHAVATAR | 1 Oct | p. 13: बौद्धावतार | 1 Oct | 1 Oct |
| 2083 | BUDDHAVATAR | 20 Oct | p. 15: बौद्धावतार | 20 Oct | 20 Oct |
| 2082 | MAHA_NAVAMI | 1 Oct | p. 13: महानवमी | 1 Oct | 1 Oct |
| 2083 | MAHA_NAVAMI | 20 Oct | p. 15: महानवमी | 20 Oct | 20 Oct |
| 2082 | SYAKWA_TYAKWA | 1 Oct | p. 13: (स्याक्वत्याक्व) | 1 Oct | 1 Oct |
| 2083 | SYAKWA_TYAKWA | 20 Oct | p. 15: (स्याक्वत्याक्व) | 20 Oct | 20 Oct |
| 2083 | CHAALA | 21 Oct | p. 15: (चालं) | 21 Oct | 21 Oct |
| 2082 | DEVI_VISARJAN | 2 Oct | p. 13: देवी विसर्जन | 2 Oct | 2 Oct |
| 2083 | DEVI_VISARJAN | 21 Oct | p. 15: देवी विसर्जन | 21 Oct | 21 Oct |
| 2082 | KHADGA_JATRA | 2 Oct | p. 13: खड्गयात्रा | 2 Oct | 2 Oct |
| 2083 | KHADGA_JATRA | 21 Oct | p. 15: खड्गयात्रा | 21 Oct | 21 Oct |
| 2082 | ANNAPURNA_JATRA | 3 Oct | p. 13: अन्नपूर्णयात्रा | 3 Oct | 3 Oct |
| 2083 | ANNAPURNA_JATRA | 22 Oct | p. 15: अन्नपूर्णायात्रा | 22 Oct | 22 Oct |
| 2082 | PAPANKUSHA_EKADASHI | 3 Oct | p. 13: पापाङ्कुशा एकादशीव्रत | 3 Oct | 3 Oct |
| 2083 | PAPANKUSHA_EKADASHI | 22 Oct | p. 15: पापाङ्कुशा एकादशीव्रत | 22 Oct | 22 Oct |
| 2082 | KHANCHI_SARAI_PARVA | 5 Oct | p. 13: खाँचीमा सरायपर्व | 5 Oct | 5 Oct |
| 2083 | KHANCHI_SARAI_PARVA | 24 Oct | p. 15: खाँचीमा सरायपर्व | 24 Oct | 24 Oct |
| 2082 | AKHILBALI_PURTI | 6 Oct | p. 13: अखिलबलिपूर्ति (सिघया) | 6 Oct | 6 Oct |
| 2083 | AKHILBALI_PURTI | 25 Oct | p. 15: अखिलबलिपूर्ति | 25 Oct | 25 Oct |
| 2082 | SHREEGHA_JATRA | 6 Oct | p. 13: (सिघया) | 6 Oct | 6 Oct |
| 2083 | SHREEGHA_JATRA | 25 Oct | p. 15: (सिघया) | 25 Oct | 25 Oct |
| 2082 | ARGHA_SARAI_PARVA | 7 Oct | p. 13: अर्घासरायपर्व | 7 Oct | 7 Oct |
| 2083 | ARGHA_SARAI_PARVA | 26 Oct | p. 15: अर्घामा सरायपर्व | 26 Oct | 26 Oct |
| 2082 | KARTIK_SNAN_BEGINS | 7 Oct | p. 13: कार्तिकस्नान | 7 Oct | 7 Oct |
| 2083 | KARTIK_SNAN_BEGINS | 26 Oct | p. 15: कार्तिकस्नान आरम्भ | 26 Oct | 26 Oct |
| 2082 | KATIN_PUNHI | 7 Oct | p. 13: कतिं पुन्हि | 7 Oct | 7 Oct |
| 2083 | KATIN_PUNHI | 26 Oct | p. 15: कतिं पुन्हि | 26 Oct | 26 Oct |
| 2082 | RADHASHTAMI | 14 Oct | p. 14: राधाष्टमी | 14 Oct | 14 Oct |
| 2083 | RADHASHTAMI | 2 Nov | p. 16: राधाष्टमी | 2 Nov | 2 Nov |
| 2082 | RAMA_EKADASHI | 17 Oct | p. 14: रमाएकादशीव्रत | 17 Oct | 17 Oct |
| 2083 | RAMA_EKADASHI | 5 Nov | p. 16: रमाएकादशीव्रत | 5 Nov | 5 Nov |
| 2082 | DHANTERAS | 18 Oct | p. 14: धन्तेरस | 18 Oct | 18 Oct |
| 2083 | DHANTERAS | 6 Nov | p. 16: धन्तेरस | 6 Nov | 6 Nov |
| 2082 | KAG_TIHAR | 19 Oct | p. 14: कागतिहार | 19 Oct | 19 Oct |
| 2083 | KAG_TIHAR | 7 Nov | p. 16: कागतिहार | 7 Nov | 7 Nov |
| 2082 | SWONTI_CHARHE | 19 Oct | p. 14: (स्वन्तिचह्रेपूजा) | 19 Oct | 19 Oct |
| 2083 | SWONTI_CHARHE | 7 Nov | p. 16: (स्वन्तिचह्रेपूजा) | 7 Nov | 7 Nov |
| 2082 | KUKUR_TIHAR | 20 Oct | p. 14: कुकुरतिहार | 20 Oct | 20 Oct |
| 2083 | KUKUR_TIHAR | 8 Nov | p. 16: कुकुरतिहार | 8 Nov | 8 Nov |
| 2082 | NARAK_CHATURDASHI | 20 Oct | p. 14: नरक चतुर्दशी | 20 Oct | 20 Oct |
| 2083 | NARAK_CHATURDASHI | 8 Nov | p. 16: नरक चतुर्दशी | 8 Nov | 8 Nov |
| 2082 | GOVARDHAN_PUJA | 22 Oct | p. 15: गोवर्धनपूजा | 22 Oct | 22 Oct |
| 2083 | GOVARDHAN_PUJA | 10 Nov | p. 17: गोवर्धनपूजा | 10 Nov | 10 Nov |
| 2082 | HALI_TIHAR | 22 Oct | p. 15: हलितिहार | 22 Oct | 22 Oct |
| 2083 | HALI_TIHAR | 10 Nov | p. 17: हलितिहार | 10 Nov | 10 Nov |
| 2082 | CHHATH | 27 Oct | p. 15: छठपर्व | 27 Oct | 28 Oct ✗ |
| 2083 | CHHATH | 15 Nov | p. 17: छठपर्व | 15 Nov | 15 Nov |
| 2082 | GOPASHTAMI | 29 Oct | p. 15: गोपअष्टमी | 29 Oct | 30 Oct ✗ |
| 2083 | GOPASHTAMI | 17 Nov | p. 17: गोपाष्टमी | 17 Nov | 17 Nov |
| 2082 | MUKHA_ASHTAMI | 29 Oct | p. 15: मुखअष्टमी | 29 Oct | 30 Oct ✗ |
| 2083 | MUKHA_ASHTAMI | 17 Nov | p. 17: मुखाष्टमी | 17 Nov | 17 Nov |
| 2082 | SILANGA_MAHARUDRA_KOTIHOM_MELA | 29 Oct | p. 15: बैतडी सिलङ्गामा महारुद्र कोटीहोममेला | 29 Oct | 30 Oct ✗ |
| 2083 | SILANGA_MAHARUDRA_KOTIHOM_MELA | 17 Nov | p. 17: बैतडी सिलङ्गामा महारुद्र कोटीहोममेला | 17 Nov | 17 Nov |
| 2082 | HETAUDA_KUSHMANDA_SAROBAR_MELA | 30 Oct | p. 15: हेटौडा कुष्माण्ड सरोवरमेला | 30 Oct | 31 Oct ✗ |
| 2083 | HETAUDA_KUSHMANDA_SAROBAR_MELA | 18 Nov | p. 17: हेटौडा कुष्माण्ड सरोवरमेला | 18 Nov | 18 Nov |
| 2082 | KUSHMANDA_NAVAMI | 30 Oct | p. 15: कुष्माण्डनवमी | 30 Oct | 31 Oct ✗ |
| 2083 | KUSHMANDA_NAVAMI | 18 Nov | p. 17: कुष्माण्डनवमी | 18 Nov | 18 Nov |
| 2082 | SAKWO_CHANGU_WONE | 30 Oct | p. 15: साँखुचाँगुमेला (सकोचंगुवने) | 30 Oct | 31 Oct ✗ |
| 2083 | SAKWO_CHANGU_WONE | 18 Nov | p. 17: साँखुचाँगुमेला(सकोचंगुवने) | 18 Nov | 18 Nov |
| 2082 | SATYAYUGADI | 30 Oct | p. 15: सत्ययुगादि | 30 Oct | 31 Oct ✗ |
| 2083 | SATYAYUGADI | 18 Nov | p. 17: सत्ययुगादि | 18 Nov | 18 Nov |
| 2082 | BALAMBU_MAHALAXMI_JATRA | 31 Oct | p. 15: बलम्बु महालक्ष्मीयात्रा | 31 Oct | 1 Nov ✗ |
| 2083 | BALAMBU_MAHALAXMI_JATRA | 19 Nov | p. 17: बलम्बु महालक्ष्मीयात्रा | 19 Nov | 20 Nov ✗ |
| 2082 | FULKHARKA_GANGAJAMUNA_MELA | 31 Oct | p. 15: धादिङ फूलखर्क गङ्गाजमुनामेला | 31 Oct | 1 Nov ✗ |
| 2083 | FULKHARKA_GANGAJAMUNA_MELA | 19 Nov | p. 17: धादिङ फूलखर्क गङ्गाजमुनामेला | 19 Nov | 20 Nov ✗ |
| 2082 | CHANGUNARAYAN_AKHANDA_DEEP_DARSHAN | 2 Nov | p. 15: चाँगुनारायण अखण्डदीपदर्शन | 2 Nov | 2 Nov |
| 2083 | CHANGUNARAYAN_AKHANDA_DEEP_DARSHAN | 21 Nov | p. 17: चाँगुनारायण अखण्डदीपदर्शन | 21 Nov | 21 Nov |
| 2082 | SHIVA_BAIKUNTHA_CHATURDASHI | 4 Nov | p. 15: बैकुण्ठ चतुर्दशीव्रत | 4 Nov | 4 Nov |
| 2083 | SHIVA_BAIKUNTHA_CHATURDASHI | 23 Nov | p. 17: बैकुण्ठ चतुर्दशीव्रत | 23 Nov | 23 Nov |
| 2082 | CHATURMAS_ENDS | 5 Nov | p. 15: चतुर्मासव्रत समाप्ति | 5 Nov | 5 Nov |
| 2083 | CHATURMAS_ENDS | 24 Nov | p. 17: कार्तिकस्नान चतुर्मासव्रत समाप्ति | 24 Nov | 24 Nov |
| 2082 | GURU_NANAK_JAYANTI | 5 Nov | p. 15: गुरुनानकजयन्ती | 5 Nov | 5 Nov |
| 2083 | GURU_NANAK_JAYANTI | 24 Nov | p. 17: गुरुनानकजयन्ती | 24 Nov | 24 Nov |
| 2082 | KARTIK_SNAN_ENDS | 5 Nov | p. 15: कार्तिकस्नान चतुर्मासव्रत समाप्ति | 5 Nov | 5 Nov |
| 2083 | KARTIK_SNAN_ENDS | 24 Nov | p. 17: कार्तिकस्नान चतुर्मासव्रत समाप्ति | 24 Nov | 24 Nov |
| 2082 | THANKOT_MAHALAXMI_JATRA | 6 Nov | p. 16: थानकोट महालक्ष्मीयात्रा | 6 Nov | 6 Nov |
| 2083 | THANKOT_MAHALAXMI_JATRA | 25 Nov | p. 18: थानकोट महालक्ष्मीयात्रा | 25 Nov | 25 Nov |
| 2082 | GUHYESHWARI_JATRA | 14 Nov | p. 16: गुह्येश्वरीयात्रा | 14 Nov | 14 Nov |
| 2083 | GUHYESHWARI_JATRA | 3 Dec | p. 18: गुह्येश्वरीयात्रा | 3 Dec | 3 Dec |
| 2082 | UTPATTIKA_EKADASHI | 15 Nov | p. 16: उत्पत्तिका एकादशीव्रत | 15 Nov | 15 Nov |
| 2083 | UTPATTIKA_EKADASHI | 4 Dec | p. 18: उत्पत्तिका एकादशीव्रत | 4 Dec | 4 Dec |
| 2082 | BALA_CHATURDASHI_BALA | 19 Nov | p. 16: बालाचतुर्दशी | 19 Nov | 19 Nov |
| 2083 | BALA_CHATURDASHI_BALA | 7 Dec | p. 18: बालाचतुर्दशी | 7 Dec | 7 Dec |
| 2082 | SHADANANDA_JAYANTI | 25 Nov | p. 17: आबाल ब्रह्मचारी षडानन्दजयन्ती | 25 Nov | 25 Nov |
| 2083 | SHADANANDA_JAYANTI | 14 Dec | p. 19: आबाल ब्रह्मचारी षडानन्दजयन्ती | 14 Dec | 14 Dec |
| 2082 | VIVAH_PANCHAMI | 25 Nov | p. 17: विवाहपञ्चमी | 25 Nov | 25 Nov |
| 2083 | VIVAH_PANCHAMI | 14 Dec | p. 19: विवाहपञ्चमी | 14 Dec | 14 Dec |
| 2082 | BAKHUMADA | 28 Nov | p. 17: (वखुमद) | 28 Nov | 28 Nov |
| 2083 | BAKHUMADA | 17 Dec | p. 19: (वखुमद) | 17 Dec | 17 Dec |
| 2082 | PANGA_VAISHNAVI_DEVI_JATRA | 29 Nov | p. 17: पाङ्गा वैष्णवदेवीयात्रा | 29 Nov | 29 Nov |
| 2083 | PANGA_VAISHNAVI_DEVI_JATRA | 18 Dec | p. 19: पाङ्गा वैष्णवदेवीयात्रा | 18 Dec | 18 Dec |
| 2082 | KIRTIPUR_INDRAYANI_JATRA | 30 Nov | p. 17: कीर्तिपुरमा इन्द्राणीयात्रा | 30 Nov | 30 Nov |
| 2083 | KIRTIPUR_INDRAYANI_JATRA | 19 Dec | p. 19: कीर्तिपुरमा इन्द्राणीयात्रा | 19 Dec | 19 Dec |
| 2082 | GITA_JAYANTI | 1 Dec | p. 17: गीताजयन्ती | 1 Dec | 1 Dec |
| 2083 | GITA_JAYANTI | 20 Dec | p. 19: गीताजयन्ती | 20 Dec | 20 Dec |
| 2082 | MOKSHADA_EKADASHI | 1 Dec | p. 17: मोक्षदा एकादशीव्रत | 1 Dec | 1 Dec |
| 2083 | MOKSHADA_EKADASHI | 20 Dec | p. 19: मोक्षदा एकादशीव्रत | 20 Dec | 20 Dec |
| 2082 | DHANYA_PURNIMA | 4 Dec | p. 17: धान्यपूर्णिमा | 4 Dec | 4 Dec |
| 2083 | DHANYA_PURNIMA | 24 Dec | p. 19: धान्यपूर्णिमा | 24 Dec | 24 Dec |
| 2082 | UDHAULI | 4 Dec | p. 17: उधौलीपर्व | 4 Dec | 4 Dec |
| 2083 | UDHAULI | 24 Dec | p. 19: उधौलीपर्व | 24 Dec | 24 Dec |
| 2082 | YOMARI_PUNHI | 4 Dec | p. 17: (योमरीपून्हि, थिंलापून्हि) | 4 Dec | 4 Dec |
| 2083 | YOMARI_PUNHI | 24 Dec | p. 19: (योमरीपून्हि) | 24 Dec | 24 Dec |
| 2082 | SAFALA_EKADASHI | 15 Dec | p. 18: सफला एकादशीव्रत | 15 Dec | 15 Dec |
| 2083 | SAFALA_EKADASHI | 3 Jan | p. 20: सफला एकादशीव्रत | 3 Jan | 3 Jan |
| 2082 | GAYA_MAUNI_AUNSI | 19 Dec | p. 18: गयामौनी औसी | 19 Dec | 19 Dec |
| 2083 | GAYA_MAUNI_AUNSI | 7 Jan | p. 20: गयामौनी औंसी | 7 Jan | 7 Jan |
| 2082 | TOL_LHOSAR | 20 Dec | p. 19: तोल ल्होसार | 20 Dec | 21 Dec ✗ |
| 2083 | TOL_LHOSAR | 8 Jan | p. 21: तोल ल्होसार | 8 Jan | 8 Jan |
| 2082 | SETO_MACHHINDRANATH_SNAN | 28 Dec | p. 19: श्रीश्वेतमत्स्येन्द्रनाथस्नान (जनबहाद्यन्हव) | 28 Dec | 28 Dec |
| 2083 | SETO_MACHHINDRANATH_SNAN | 16 Jan | p. 21: श्रीश्वेतमत्स्येन्द्रनाथस्नान (जनबाद्यन्हव) | 16 Jan | 16 Jan |
| 2082 | MUSYADULI | 31 Dec | p. 19: मुस्याडुली | 31 Dec | 31 Dec |
| 2083 | MUSYADULI | 19 Jan | p. 21: मुस्याडुली | 19 Jan | 19 Jan |
| 2082 | CHANGUNARAYAN_KALASH_JATRA | 3 Jan | p. 19: (चाँगुनारायण कलशयात्रा) | 3 Jan | 3 Jan |
| 2083 | CHANGUNARAYAN_KALASH_JATRA | 22 Jan | p. 21: (चाँगुनारायण कलशयात्रा) | 22 Jan | 22 Jan |
| 2082 | MAGH_SNAN_BEGINS | 3 Jan | p. 19: माघस्नानसुरु | 3 Jan | 3 Jan |
| 2083 | MAGH_SNAN_BEGINS | 22 Jan | p. 21: माघस्नानसुरु | 22 Jan | 22 Jan |
| 2082 | SWASTHANI_VRAT_BEGINS | 3 Jan | p. 19: श्रीस्वस्थानीव्रत सुरु | 3 Jan | 3 Jan |
| 2083 | SWASTHANI_VRAT_BEGINS | 22 Jan | p. 21: श्रीस्वस्थानीव्रत सुरु | 22 Jan | 22 Jan |
| 2082 | SHATTILA_EKADASHI | 14 Jan | p. 20: षट्तिला एकादशीव्रत | 14 Jan | 14 Jan |
| 2083 | SHATTILA_EKADASHI | 2 Feb | p. 22: षट्तिला एकादशीव्रत | 2 Feb | 2 Feb |
| 2082 | ARYAGHAT_MADHAV_NARAYAN_MELA | 18 Jan | p. 20: आर्यघाटमा माधवनारायण मेला | 18 Jan | 18 Jan |
| 2083 | ARYAGHAT_MADHAV_NARAYAN_MELA | 6 Feb | p. 22: आर्यघाटमा माधवनारायण मेला | 6 Feb | 6 Feb |
| 2082 | SHREE_BALLABH_JAYANTI | 19 Jan | p. 21: श्रीवल्लभजयन्ती | 19 Jan | 19 Jan |
| 2083 | SHREE_BALLABH_JAYANTI | 7 Feb | p. 23: श्रीवल्लभजयन्ती | 7 Feb | 7 Feb |
| 2082 | BASANTA_PANCHAMI | 23 Jan | p. 21: वसन्तपञ्चमी | 23 Jan | 23 Jan |
| 2083 | BASANTA_PANCHAMI | 11 Feb | p. 23: वसन्तपञ्चमी | 11 Feb | 11 Feb |
| 2082 | SKANDA_SHASHTHI | 24 Jan | p. 21: स्कन्दषष्ठी | 24 Jan | 24 Jan |
| 2083 | SKANDA_SHASHTHI | 12 Feb | p. 23: स्कन्दषष्ठी | 12 Feb | 12 Feb |
| 2082 | ACHALA_SAPTAMI | 25 Jan | p. 21: अचलासप्तमी | 25 Jan | 25 Jan |
| 2083 | ACHALA_SAPTAMI | 13 Feb | p. 23: अचलासप्तमी | 13 Feb | 13 Feb |
| 2082 | BHISHMASHTAMI | 26 Jan | p. 21: भीष्माष्टमी | 26 Jan | 26 Jan |
| 2083 | BHISHMASHTAMI | 14 Feb | p. 23: भीष्माष्टमी | 14 Feb | 14 Feb |
| 2082 | DRONA_NAVAMI | 27 Jan | p. 21: द्रोणनवमी | 27 Jan | 27 Jan |
| 2083 | DRONA_NAVAMI | 15 Feb | p. 23: द्रोणनवमी | 15 Feb | 15 Feb |
| 2082 | SHALYA_DASHAMI | 28 Jan | p. 21: शल्यदशमी | 28 Jan | 28 Jan |
| 2083 | SHALYA_DASHAMI | 16 Feb | p. 23: शल्यदशमी | 16 Feb | 16 Feb |
| 2082 | BHISHMA_DWADASHI | 30 Jan | p. 21: भीष्मद्वादशी | 30 Jan | 30 Jan |
| 2083 | BHISHMA_DWADASHI | 18 Feb | p. 23: भीष्मद्वादशी | 18 Feb | 18 Feb |
| 2082 | CHANGU_MADHAV_NARAYAN_MELA | 30 Jan | p. 21: चाँगुमाधवनारायणमेला | 30 Jan | 30 Jan |
| 2083 | CHANGU_MADHAV_NARAYAN_MELA | 18 Feb | p. 23: चाँगुनारायणमेला | 18 Feb | 18 Feb |
| 2082 | WATU_MAGH_JATRA | 30 Jan | p. 21: वटुमाघयात्रा | 30 Jan | 30 Jan |
| 2083 | WATU_MAGH_JATRA | 18 Feb | p. 23: वटुमाघयात्रा | 18 Feb | 18 Feb |
| 2082 | MAGH_SNAN_ENDS | 1 Feb | p. 21: माघस्नान समाप्ति | 1 Feb | 1 Feb |
| 2083 | MAGH_SNAN_ENDS | 20 Feb | p. 23: माघस्नान समाप्ति | 20 Feb | 20 Feb |
| 2082 | SWASTHANI_VRAT_ENDS | 1 Feb | p. 21: श्रीस्वस्थानीव्रतसमाप्ति | 1 Feb | 1 Feb |
| 2083 | SWASTHANI_VRAT_ENDS | 20 Feb | p. 23: श्रीस्वस्थानीव्रतसमाप्ति | 20 Feb | 20 Feb |
| 2082 | CHIR_SWAYEGU | 24 Feb | p. 23: चिरोत्थान | 24 Feb | 24 Feb |
| 2083 | CHIR_SWAYEGU | 16 Mar | p. 25: चिरस्वाय | 16 Mar | 16 Mar |
| 2082 | AMALAKI_EKADASHI | 27 Feb | p. 23: आमलकी एकादशीव्रत | 27 Feb | 27 Feb |
| 2083 | AMALAKI_EKADASHI | 18 Mar | p. 25: आमलकी एकादशीव्रत | 18 Mar | 18 Mar |
| 2082 | GOVINDA_DWADASHI | 28 Feb | p. 23: गोविन्दद्वादशी | 28 Feb | 28 Feb |
| 2083 | GOVINDA_DWADASHI | 19 Mar | p. 25: गोविन्दद्वादशी | 19 Mar | 19 Mar |
| 2082 | NALA_MACHHINDRANATH_RATH_JATRA | 6 Mar | p. 24: नालायात्रा | 6 Mar | 6 Mar |
| 2083 | NALA_MACHHINDRANATH_RATH_JATRA | 25 Mar | p. 26: नालायात्रा, नाला मत्स्येन्द्रनाथ यात्रा | 25 Mar | 25 Mar |
| 2082 | SHITALASHTAMI | 11 Mar | p. 24: शीतलाष्टमी | 11 Mar | 11 Mar |
| 2083 | SHITALASHTAMI | 30 Mar | p. 26: शीतलाष्टमी | 30 Mar | 30 Mar |
| 2082 | PAPMOCHANI_EKADASHI | 15 Mar | p. 24: पापमोचनी एकादशीव्रत | 15 Mar | 15 Mar |
| 2083 | PAPMOCHANI_EKADASHI | 2 Apr | p. 26: पापमोचनी एकादशीव्रत | 2 Apr | 2 Apr |
| 2082 | KOKA_KAUSHIKI_SANGAM_MELA | 5 Dec | p. 18: कोशिकोका सङ्गममा मेला | 5 Dec | 5 Dec |
| 2083 | KOKA_KAUSHIKI_SANGAM_MELA | 25 Dec | p. 20: कोशीकोका सङ्गममा मेला | 25 Dec | 25 Dec |
| 2082 | NARASIMHA_JAYANTI | 11 May | p. 3: नृसिंहजयन्ती | 11 May | 11 May |
| 2083 | NARASIMHA_JAYANTI | 30 Apr | p. 3: नृसिंहजयन्ती | 30 Apr | 30 Apr |
| 2082 | VARAHA_JAYANTI | 27 Jul | p. 9: वराहजयन्ती | 27 Jul | 27 Jul |
| 2082 | KALKI_JAYANTI | 30 Jul | p. 9: कल्कीजयन्ती | 30 Jul | 30 Jul |
| 2082 | BAHIDYO_BOYEGU | 6 Aug | p. 9: बहिद्यब्वोय | 6 Aug | 6 Aug |
| 2082 | SHIVA_PAVITRAROPAN | 8 Aug | p. 9: श्रीशिवजीमा पवित्रारोपण | 8 Aug | 8 Aug |
| 2082 | HAYAGRIVA_JAYANTI | 8 Aug | p. 9: हयग्रीवोत्पत्ति | 8 Aug | 8 Aug |
| 2082 | LALITPUR_NARSINGH_JATRA | 14 Aug | p. 10: ल.पु. नृसिंहयात्रा (उपाकलं सायाः) | 14 Aug | 13 Aug ✗ |
| 2082 | KOJAGRAT_PURNIMA | 6 Oct | p. 13: कोजाग्रतव्रत | 6 Oct | 6 Oct |
| 2082 | GOVATSA_DWADASHI | 17 Oct | p. 14: गोवत्सद्वादशी | 17 Oct | 17 Oct |
| 2082 | ARJUNDHARA_MELA | 18 Nov | p. 16: सताक्षीधाम तथा अर्जुनधारा मेला | 18 Nov | 18 Nov |
| 2082 | KASHI_BADA_GANESH_UTSAV | 6 Jan | p. 20: काशीमा बडागणेश उत्सव | 6 Jan | 6 Jan |
| 2082 | SI_PUNHI | 1 Feb | p. 21: (सिपून्हि) | 1 Feb | 1 Feb |
| 2082 | KAMADA_EKADASHI | 29 Mar | p. 25: कामदाएकादशीव्रत | 29 Mar | 29 Mar |
| 2083 | VARAHA_JAYANTI | 15 Aug | p. 11: वराहजयन्ती | 15 Aug | 15 Aug |
| 2083 | KALKI_JAYANTI | 18 Aug | p. 11: कल्कीजयन्ती | 18 Aug | 18 Aug |
| 2083 | BAHIDYO_BOYEGU | 24 Aug | p. 11: बहिद्यब्बोय | 24 Aug | 24 Aug |
| 2083 | SHIVA_PAVITRAROPAN | 27 Aug | p. 11: श्रीशिवजीमा पवित्रारोपण | 27 Aug | 27 Aug |
| 2083 | HAYAGRIVA_JAYANTI | 27 Aug | p. 11: हयग्रीवोत्पत्ति | 27 Aug | 27 Aug |
| 2083 | LALITPUR_NARSINGH_JATRA | 1 Sep | p. 12: ल.पु. नृसिंहयात्रा (उपाकलं सायाः) | 1 Sep | 2 Sep ✗ |
| 2083 | KOJAGRAT_PURNIMA | 25 Oct | p. 15: कोजाग्रतव्रत | 25 Oct | 25 Oct |
| 2083 | GOVATSA_DWADASHI | 5 Nov | p. 16: गोवत्सद्वादशी | 5 Nov | 5 Nov |
| 2083 | ARJUNDHARA_MELA | 6 Dec | p. 18: झापा सताक्षीधाम तथा अर्जुनधारामेला | 6 Dec | 6 Dec |
| 2083 | KASHI_BADA_GANESH_UTSAV | 25 Jan | p. 22: काशीमा बडागणेश उत्सव | 25 Jan | 25 Jan |
| 2083 | SI_PUNHI | 20 Feb | p. 23: (सिपून्हि) | 20 Feb | 20 Feb |

| proposed rule | verdict | measured |
|---|---|---|
| the `NEPAL_PANCHANGA` pack over the committee's sky falls on the printed day | falsified | 6 of 589 disagree; each parting is named below with its cause |
| the same over the modern sky falls on the printed day | falsified | 60 of 589 disagree; the sky decides these: a tithi's end moves the day where the two skies part |

Where the pack over the committee's sky parts from the print, and why:

- VS 2082 HILE_JATRA (1 Sep): the print's हिलेजात्रा is Ramechhap's, on Shravana's dark 9th; the rule's Bhadrapada bright 9th prints no Hile jatra (C609)
- VS 2083 AMUKTABHARANA_SAPTAMI (18 Sep): counted from GAURA_SAPTAMI (C608)
- VS 2083 DURVA_ASHTAMI (19 Sep): the committee prints Shravana's dark 8th, inside the solar Bhadra, where Bhadrapada's bright 8th falls in the solar Ashwin (C608)
- VS 2083 GAURA_SAPTAMI (18 Sep): the committee prints Shravana's dark 7th, inside the solar Bhadra, where Bhadrapada's bright 7th falls in the solar Ashwin (C608)
- VS 2083 HILE_JATRA (20 Sep): the print's हिलेजात्रा is Ramechhap's, on Shravana's dark 9th; the rule's Bhadrapada bright 9th prints no Hile jatra (C609)
- VS 2083 NUWAKOT_DUPCHESHWAR_MELA (23 Dec): the committee prints the mela on the full moon, a day after Dattatreya's 14th (C611)

## 7. The print as a published set

The same printed rows written as one published set
(`published-overrides.md` §6) over the `NEPAL_PANCHANGA` pack on the
modern sky, as an app showing the print over its own reckoning would:
each row a `SET` on its printed day, each Vikram year asked as one
range. The pass holds every printed day to the answer's. Each day the
set moved is then explained (`published_reasons`) with the committee's
sky tried, then each shipped pack's rule of the same key; a day neither
gives is the publisher's own.

| proposed rule | verdict | measured |
|---|---|---|
| the set laid over the modern sky gives every printed day it places | **holds** | 0 of 578 disagree |
| every entry is placed on its rule's occurrence | falsified | 11 of 589 disagree |
| the committee's sky gives every day the set moved | falsified | 1 of 49 disagree; the others' reasons are below |

Of the 49 days the set moved, the committee's sky gives 48, and no change tried 1.

The publisher's own days, which no change tried gives:

- NUWAKOT_DUPCHESHWAR_MELA: printed 24 Dec, the modern sky 23 Dec

The entries no occurrence took, the rule giving no day within a week
of the print under the modern sky (a weekday the sky does not meet, or
a month the committee reckons otherwise, §6):

- VS 2082 HILE_JATRA, printed 17 Aug
- VS 2082 RAVI_SAPTAMI, printed 14 Sep
- VS 2082 BUDHASHTAMI, printed 29 Oct
- VS 2083 GAURA_SAPTAMI, printed 3 Sep
- VS 2083 AMUKTABHARANA_SAPTAMI, printed 3 Sep
- VS 2083 DURVA_ASHTAMI, printed 4 Sep
- VS 2083 HILE_JATRA, printed 5 Sep
- VS 2083 MANGALA_CHATURTHI, printed 21 Apr
- VS 2083 MANGALA_CHATURTHI, printed 5 May
- VS 2083 BUDHASHTAMI, printed 22 Jul
- VS 2083 MANGALA_CHATURTHI, printed 26 Jan

## 8. Saits against the print

Each printed sait (`festival-sait.md` §1) is the rite's day asked of
the `NEPAL_PANCHANGA` pack, whose saits elect a navamsa of a dual sign
in a lagna the Sun does not stand in, inside the text's window less
what it avoids. The pass holds every printed minute to the candidate
the committee's sky elects, within a printed minute; the elected
middle's distance from the print is measured, not held.

| proposed rule | verdict | measured |
|---|---|---|
| the committee's sky elects every printed sait | **holds** | 0 of 6 disagree |
| the modern sky elects every printed sait | **holds** | 0 of 6 disagree |
| C747: without the Sun's lagna barred, the print is still elected | falsified | 1 of 6 disagree |
| C747: with every malefic's lagna barred, the print is still elected | falsified | 1 of 6 disagree |
| C746: with each rule's choice turned round, the print is still elected | falsified | 2 of 6 disagree |

| rite | day | printed | the committee's sky | lagna / navamsa | middle | the modern sky's middle |
|---|---|---|---|---|---|---|
| NAVARATRA_ARAMBHA | 2023-10-15 | 11:29 | 11:26–11:40 | Sagittarius / Gemini | +4.4 min | +4.4 min |
| NAVARATRA_ARAMBHA | 2024-10-03 | 09:16 | 09:07–09:23 | Libra / Gemini | -0.8 min | -0.8 min |
| NAVARATRA_ARAMBHA | 2025-09-22 | 09:13 | 09:05–09:21 | Libra / Pisces | -0.1 min | -0.1 min |
| NAVARATRA_ARAMBHA | 2026-10-11 | 11:47 | 11:41–11:55 | Sagittarius / Gemini | +1.1 min | +1.1 min |
| PHULPATI | 2024-10-10 | 07:11 | 07:07–07:23 | Libra / Sagittarius | +3.9 min | +3.9 min |
| YAMA_DWITIYA | 2025-10-23 | 11:39 | 11:35–11:49 | Sagittarius / Virgo | +2.7 min | +2.7 min |
## 9. India's central holidays against the notices

`INDIA_CENTRAL` at Delhi on India's clock, against each year's Annexure I of the Department of Personnel and Training's office memorandum. 86 of 91 rows agree.

- 2021: F.No.12/9/2020-JCA-2, 10 June 2020
- 2022: F.No.12/5/2021-JCA-2, 8 June 2021
- 2023: F.No.12/5/2022-JCA, 16 June 2022
- 2024: F.No.12/2/2023-JCA, 3 July 2023
- 2025: F.No.12/2/2023-JCA, 9 July 2024
- 2026: F.No.12/2/2023-JCA, 3 July 2025
- 2027: F.No.12/2/2023-JCA, 16 July 2026

| year | rule | listed | found |
| --- | --- | --- | --- |
| 2021 | HOLI | 3/29 | 3/29 |
| 2021 | GOOD_FRIDAY | 4/2 | 4/2 |
| 2021 | RAMA_NAVAMI | 4/21 | 4/21 |
| 2021 | MAHAVIR_JAYANTI | 4/25 | 4/25 |
| 2021 | BUDDHA_JAYANTI | 5/26 | 5/26 |
| 2021 | JANMASHTAMI | 8/30 | 8/30 |
| 2021 | VIJAYA_DASHAMI | 10/15 | 10/15 |
| 2021 | LAKSHMI_PUJA | 11/4 | 11/4 |
| 2021 | GURU_NANAK_JAYANTI | 11/19 | 11/19 |
| 2022 | SHIVARATRI | 3/1 | 3/1 |
| 2022 | HOLI | 3/18 | 3/18 |
| 2022 | MAHAVIR_JAYANTI | 4/14 | 4/14 |
| 2022 | GOOD_FRIDAY | 4/15 | 4/15 |
| 2022 | BUDDHA_JAYANTI | 5/16 | 5/16 |
| 2022 | JANMASHTAMI | 8/19 | 8/19 |
| 2022 | VIJAYA_DASHAMI | 10/5 | 10/4 ✗ |
| 2022 | LAKSHMI_PUJA | 10/24 | 10/24 |
| 2022 | GURU_NANAK_JAYANTI | 11/8 | 11/8 |
| 2023 | HOLI | 3/8 | 3/7 ✗ |
| 2023 | RAMA_NAVAMI | 3/30 | 3/30 |
| 2023 | MAHAVIR_JAYANTI | 4/4 | 4/4 |
| 2023 | GOOD_FRIDAY | 4/7 | 4/7 |
| 2023 | BUDDHA_JAYANTI | 5/5 | 5/5 |
| 2023 | JANMASHTAMI | 9/7 | 9/7 |
| 2023 | VIJAYA_DASHAMI | 10/24 | 10/24 |
| 2023 | LAKSHMI_PUJA | 11/12 | 11/12 |
| 2023 | GURU_NANAK_JAYANTI | 11/27 | 11/27 |
| 2024 | HOLI | 3/25 | 3/25 |
| 2024 | GOOD_FRIDAY | 3/29 | 3/29 |
| 2024 | RAMA_NAVAMI | 4/17 | 4/17 |
| 2024 | MAHAVIR_JAYANTI | 4/21 | 4/21 |
| 2024 | BUDDHA_JAYANTI | 5/23 | 5/23 |
| 2024 | JANMASHTAMI | 8/26 | 8/26 |
| 2024 | VIJAYA_DASHAMI | 10/12 | 10/12 |
| 2024 | LAKSHMI_PUJA | 10/31 | 11/1 ✗ |
| 2024 | GURU_NANAK_JAYANTI | 11/15 | 11/15 |
| 2025 | SHIVARATRI | 2/26 | 2/26 |
| 2025 | HOLI | 3/14 | 3/14 |
| 2025 | MAHAVIR_JAYANTI | 4/10 | 4/10 |
| 2025 | GOOD_FRIDAY | 4/18 | 4/18 |
| 2025 | BUDDHA_JAYANTI | 5/12 | 5/12 |
| 2025 | JANMASHTAMI | 8/16 | 8/16 |
| 2025 | VIJAYA_DASHAMI | 10/2 | 10/2 |
| 2025 | LAKSHMI_PUJA | 10/20 | 10/20 |
| 2025 | GURU_NANAK_JAYANTI | 11/5 | 11/5 |
| 2026 | HOLI | 3/4 | 3/3 ✗ |
| 2026 | RAMA_NAVAMI | 3/26 | 3/26 |
| 2026 | MAHAVIR_JAYANTI | 3/31 | 3/31 |
| 2026 | GOOD_FRIDAY | 4/3 | 4/3 |
| 2026 | BUDDHA_JAYANTI | 5/1 | 5/1 |
| 2026 | JANMASHTAMI | 9/4 | 9/4 |
| 2026 | VIJAYA_DASHAMI | 10/20 | 10/20 |
| 2026 | LAKSHMI_PUJA | 11/8 | 11/8 |
| 2026 | GURU_NANAK_JAYANTI | 11/24 | 11/24 |
| 2027 | HOLI | 3/23 | 3/22 ✗ |
| 2027 | GOOD_FRIDAY | 3/26 | 3/26 |
| 2027 | RAMA_NAVAMI | 4/15 | 4/15 |
| 2027 | MAHAVIR_JAYANTI | 4/19 | 4/19 |
| 2027 | BUDDHA_JAYANTI | 5/20 | 5/20 |
| 2027 | JANMASHTAMI | 8/25 | 8/25 |
| 2027 | VIJAYA_DASHAMI | 10/9 | 10/9 |
| 2027 | LAKSHMI_PUJA | 10/29 | 10/29 |
| 2027 | GURU_NANAK_JAYANTI | 11/14 | 11/14 |

The four civil days stand on their date in all 7 years.

- 2022 VIJAYA_DASHAMI: the 10th holds only the earlier aparahna, and Shravana begins that night inside the 10th, so it joins both days and the rule stays on the earlier; the Department prints the later, where Shravana stands in the aparahna (C855)
- 2023 HOLI: bhadra covers the earlier pradosha and the later has no full moon; the rule takes the earlier, the Department a Holi a day later (C854)
- 2024 LAKSHMI_PUJA: the new moon lasts past a ghati after the later sunset, so the text takes the later; the Department, printing 16 months ahead, the earlier (C856)
- 2026 HOLI: bhadra covers the earlier pradosha and the later has no full moon; the rule takes the earlier, the Department a Holi a day later (C854)
- 2027 HOLI: bhadra covers the earlier pradosha and the later has no full moon; the rule takes the earlier, and Annexure II prints Holika Dahan on the later (C854)

The days set by the moon are not reckoned. Para 5.1 leaves each to the sighting, so the pack carries the notice's day, marked; two the Department moved stand on the day it declared; 2031, with no notice, holds none.

| year | key | day | stands as |
| --- | --- | --- | --- |
| 2021 | EID_AL_FITR | 5/14 | subject to sighting |
| 2021 | EID_AL_ADHA | 7/21 | subject to sighting |
| 2021 | MUHARRAM | 8/20 | declared |
| 2021 | MAWLID | 10/19 | subject to sighting |
| 2022 | EID_AL_FITR | 5/3 | subject to sighting |
| 2022 | EID_AL_ADHA | 7/10 | subject to sighting |
| 2022 | MUHARRAM | 8/9 | subject to sighting |
| 2022 | MAWLID | 10/9 | subject to sighting |
| 2023 | EID_AL_FITR | 4/22 | subject to sighting |
| 2023 | EID_AL_ADHA | 6/29 | subject to sighting |
| 2023 | MUHARRAM | 7/29 | subject to sighting |
| 2023 | MAWLID | 9/28 | subject to sighting |
| 2024 | EID_AL_FITR | 4/11 | subject to sighting |
| 2024 | EID_AL_ADHA | 6/17 | subject to sighting |
| 2024 | MUHARRAM | 7/17 | subject to sighting |
| 2024 | MAWLID | 9/16 | subject to sighting |
| 2025 | EID_AL_FITR | 3/31 | subject to sighting |
| 2025 | EID_AL_ADHA | 6/7 | subject to sighting |
| 2025 | MUHARRAM | 7/6 | subject to sighting |
| 2025 | MAWLID | 9/5 | subject to sighting |
| 2026 | EID_AL_FITR | 3/21 | subject to sighting |
| 2026 | EID_AL_ADHA | 5/28 | declared |
| 2026 | MUHARRAM | 6/26 | subject to sighting |
| 2026 | MAWLID | 8/26 | subject to sighting |
| 2027 | EID_AL_FITR | 3/10 | subject to sighting |
| 2027 | EID_AL_ADHA | 5/17 | subject to sighting |
| 2027 | MUHARRAM | 6/16 | subject to sighting |
| 2027 | MAWLID | 8/15 | subject to sighting |

