# Assam's public holidays

Status: `built`, 2026-11-11. SR-03, the fifth state set, after Maharashtra's
(`maharashtra-holidays.md`), West Bengal's (`west-bengal-holidays.md`), Kerala's
(`kerala-holidays.md`) and Tamil Nadu's (`tamil-nadu-holidays.md`). Cruxes C1360 to
C1367, with C1303 closed for Assam and C1302 brought forward here
(`01-research/feature-universe/19-verification-cruxes.md`). Measured in
`festival-measured.md` §14.

## 1. The source

The General Administration Department (GAD) issues a notification each year. Its
first paragraph lists the days the offices of the Government of Assam and its
revenue and magisterial courts close. Its second lists the half holidays: Muharram
until 2024, and Lakshmi Puja. Paragraph 3(a) closes the banks for their annual
closing of accounts under the Negotiable Instruments Act, 1881. A separate list of
restricted holidays follows, and it is not shipped. Each row prints a Gregorian date,
its weekday and the Saka date. From 2022 the row also prints the Bhaskarabda (Assamese)
date, which C1300 measured. The notifications say the days come from the state's
"concerned astrologers". They carry no clause on the moon.

All eight years were read from the notifications' own pages, all rank 1 (C1360):

| year | notification | copy read |
|---|---|---|
| 2019 | No. GAG(A).81/2016/Pt/I/220, 19 December 2018, Assam Gazette Extraordinary No. 629 | the gazette's scan, gad.assam.gov.in `public_utility/holiday_list_2019.pdf` (Wayback 20220720032005) |
| 2020 | No. GAG(A)81/2016/494, 29 November 2019, Assam Gazette Extraordinary No. 486 | the gazette's scan, `holiday_list_2020.pdf` (Wayback 20220720031950) |
| 2021 | No. GAG(A).81/2016/637, 18 December 2020, Assam Gazette Extraordinary No. 566 | the gazette's scan, `holiday_list_2021.pdf` (Wayback 20220720031939) |
| 2022 | No. GAG(A).81/2016/749, 28 October 2021, with corrigenda of 30 December 2021 and 12 January 2022 | the signed scans, gad.assam.gov.in (Wayback 20220101123137, 20220103102823, 20220720031812) |
| 2023 | e-file No. 221561/1, 28 November 2022 | the signed scan, `public_utility/Holiday 2023 Assam.pdf` (Wayback 20240809135125) |
| 2024 | e-file No. 221561/11, November 2023 | a copy of the e-signed notification, teamleaseregtech.com's proxy of its article 27980 (rank 1 as copied) |
| 2025 | No. E-221561/347, 6 December 2024, with its corrigendum of 27 December 2024 | the e-signed PDFs, `public_utility/Holiday List 2025 - final.pdf` and the corrigendum (Wayback 20250104215535, 20250116023646) |
| 2026 | e-file No. 221561/440, 17 November 2025 | the e-signed PDF, `public_utility/Holiday List , 2026.pdf` (Wayback 20260412154132) |

The 2022 corrigendum of 12 January moved Magh Bihu's first day to 14 January, and the
pack is held to the day as corrected. The 2025 corrigendum's corrections are likewise
read into the list. Election days and other one-off holidays declared by separate
notifications are not shipped.

## 2. The pack

`INDIA_ASSAM` holds the list at any place. It is measured at Guwahati (26.1445 N,
91.7362 E) on India's clock. Every day is kept in `["IN", "IN-AS"]` (ISO 3166-2; `ASSAM`, its first key, is read as an alias, SR-89) and is
`GAZETTED`. The banks' closing of accounts names the community `BANKS`.

| holiday | key | form | the day's rule is cited to |
|---|---|---|---|
| Republic Day, Independence Day, Gandhi Jayanti, Christmas, Good Friday, Buddha Purnima, Guru Nanak's birthday | as `INDIA_CENTRAL` | the central list's rule | the notification |
| Netaji's birthday, Me-Dam-Me-Phi, May Day, Lachit Divas, Asom Divas, annual closing of accounts (banks) | `NETAJI_JAYANTI`, `ME_DAM_ME_PHI`, `INTERNATIONAL_LABOUR_DAY`, `LACHIT_DIVAS`, `ASOM_DIVAS`, `BANK_ACCOUNTS_CLOSING` | a Gregorian date | the notification |
| Magh Bihu (two days), Bohag Bihu (three days), Kati Bihu | `MAGH_BIHU`, `MAGH_BIHU_SECOND`, `BOHAG_BIHU`, `BOHAG_BIHU_SECOND`, `BOHAG_BIHU_THIRD`, `KATI_BIHU` | the last day of Puh, Magh 1; the last day of Chot, Bohag 1 and 2; the last day of Ahin, all on `ASSAM_SOLAR` | the notification (C1300, C1306) |
| Sati Sadhani Divas | `SATI_SADHANI_DIVAS` | Bohag 7 of `ASSAM_SOLAR` | the notification |
| Gwthar Bathou San | `GWTHAR_BATHOU_SAN` | Magh's second Tuesday on `ASSAM_SOLAR` | the notification |
| Bir Chilarai Divas, Karam Puja | `BIR_CHILARAI_DIVAS`, `KARAM_PUJA` | Magha's full moon and Bhadrapada's bright 11th, at sunrise | the notifications (C1361, C1365) |
| Damodardeva's Tithi, the Janmotsav and Tirobhav of Madhabdeva and Sankardeva, Janmastomi | `DAMODARDEVA_TITHI`, `MADHABDEVA_JANMOTSAV`, `MADHABDEVA_TIROBHAV`, `SANKARDEVA_JANMOTSAV`, `SANKARDEVA_TIROBHAV`, `JANMASHTAMI` | a tithi in an Assamese month (`tithiInMonth`) | the notifications (C1361, C1362) |
| Dol Jatra, Durga Puja's Navami, Kali Puja | `DOLA_YATRA`, `MAHA_NAVAMI`, `KALI_PUJA` | `INDIA_WEST_BENGAL`'s rule | West Bengal's citations |
| Vijaya Dashami | `VIJAYA_DASHAMI` | the day after the Navami (a `FollowingRule`) | as West Bengal's list counts it |
| Durga Puja's Saptami and Ashtami, Lakshmi Puja (half day), Chhath Puja | `PHULPATI`, `MAHA_ASHTAMI`, `KOJAGRAT_PURNIMA`, `CHHATH` | `NEPAL_PANCHANGA`'s rule | a printed panchanga |
| Bhatri Dwitiya | `YAMA_DWITIYA` | *Dharmasindhu*'s rule | *Dharmasindhu* |

**One new form, `tithiInMonth`.** The Vaishnava saints' days and Janmastomi keep a
tithi of an Assamese month, not of a lunar one: Madhabdeva's birthday falls on the dark
1st in Jeth, Sankardeva's on the bright 10th in Ahin, and Janmastomi on the dark 8th in
Bhada (`festival-rule-language.md`, "A tithi in a solar month"). An amanta month and a
solar month part by up to a month, so a lunar rule cannot give these days in every
year. For example, Sankardeva's Tirobhav is Bhada's bright 2nd, which in some years
falls in amanta Shravana and in others in Bhadrapada.

The form takes a calendar, a month, the tithi, which of two to keep when the month holds
the tithi twice, and the window it is judged at. At `SUNRISE`, the default, the day is
the first whose sunrise the tithi holds, or the day it runs in when it holds no sunrise.
This is `nakshatraInMonth`'s reading. With another window the day is the one holding
more of that window, the earlier on a tie. When no day holds any of it, the day is the
one the tithi runs in. The moonrise is refused, as `nakshatraInMonth` has no moonrise.

The days of a saint's passing (*tirobhav*) and Damodardeva's Tithi are judged at the
whole daylight, the fifteen day muhurtas (C1362). Over the eight years, at sunrise:

| rule | at sunrise | at madhyahna | over the daylight |
|---|---|---|---|
| Damodardeva's Tithi | 5 of 8 | 8 of 8 | 8 of 8 |
| Madhabdeva's Tirobhav | 5 of 8 | 7 of 8 | 8 of 8 |
| Sankardeva's Tirobhav | 5 of 8 | 6 of 8 | 6 of 8 |

The birthdays and Janmastomi keep the sunrise.

**The rest are forms the language has.** The Bihus are the ends and beginnings of
`ASSAM_SOLAR`'s months as `civilDate`s, with a negative day counting from the month's
last. Gwthar Bathou San is `civilWeekday`. Vijaya Dashami is the day after West Bengal's
Navami, counted as West Bengal counts it.

## 3. The sky

The Assamese months are `ASSAM_SOLAR`'s: the Surya Siddhanta's Sun, with a sankranti
before IST midnight beginning the month the next day (C1300). The lunar days are
reckoned under the modern sky, the built-in ephemeris with the default profile. Over the
eight years this sky meets more rows than the Surya Siddhanta's tithis do. Under the
modern sky the Durga Puja's days meet 8 of 8; under the text's, 7.

## 4. The days printed, not reckoned

`PublishedSet::india_assam` carries Id-ul-Fitr and Id-ul-Zuha for each year, and
Muharram for the years it is listed, 2019 to 2024. Each day is shipped as
`EID_AL_FITR`, `EID_AL_ADHA` or `MUHARRAM` on the day the notification prints it.
Each is the moon's day, so each is marked `subjectTo: MOON_SIGHTING`, as the central
list's are. Muharram and Lakshmi Puja are half holidays in the notifications. The
pack lists them as days, because the rule language has no half-day form (C1367).

## 5. Against the print

At Guwahati 211 of the 218 reckoned rows meet the print (`festival-measured.md` §14,
291 of 298 with the ten fixed dates, and per year in `crates/sdk/tests/assam.rs`):

| year | met / printed |
|---|---|
| 2019 | 24 / 25 |
| 2020 | 24 / 25 |
| 2021 | 26 / 28 |
| 2022 | 28 / 28 |
| 2023 | 25 / 28 |
| 2024 | 28 / 28 |
| 2025 | 28 / 28 |
| 2026 | 28 / 28 |

2019 and 2020 list 25, before Sati Sadhani Divas, Karam Puja and Bhatri Dwitiya.
Seven rows part. Each is named with its cause, and no rule was moved to meet them:

- **Sankardeva's Tirobhav, 2019** (C1364): the bright 2nd ran from 12:13 on 31
  August to 08:26 on 1 September. It held 5 h 31 min of the 31st's daylight and
  3 h 24 min of the 1st's, so the daylight reading takes the 31st. The notification
  lists the 1st, whose sunrise the tithi held.
- **Sankardeva's Tirobhav, 2023** (C1364): the notification lists 18 August, Saon 32.
  The bright 2nd held that day's whole daylight, but before Bhada began. The rule is
  held to Bhada, so it takes Bhada's own bright 2nd, 16 September.
- **Janmastomi, 2020** (C1363): the dark 8th began at 02:06 on 10 September. The
  notification lists the 9th, which the ashtami held at neither sunrise nor midnight.
- **Janmastomi, 2023** (C1363): the dark 8th began at 15:38 on 6 September. The
  notification lists the 6th, whose midnight it held, and the sunrise reading gives
  the 7th.
- **Karam Puja, 2021 and 2023** (C1365): the notifications list 29 August 2021, a
  dark 7th, and 27 August 2023. The latter is nija Shravana's bright 11th, and also
  Bhada's bright 11th. Bhadrapada's bright 11th falls on 17 and 25 September. Read as
  the bright 11th in Bhada, the rule would meet 2023 but part in 2021 and 2026, so it
  meets 4 of 6 either way. The lunar rule ships.
- **Lakshmi Puja, 2021** (C1366): the full moon ran from 19:03 on 19 October to 20:26
  on the 20th. Nepal's printed panchanga's rule keeps the night of the 19th. The
  notification lists the 20th, whose sunrise the full moon held.

## 6. The month near midnight (C1302)

The 2024 and 2026 lists, read now (C1303, closed for Assam), give 24 more Bhaskarabda
month beginnings. `crates/calendar/src/regional_solar/tests.rs` holds one pair a month.
The 2024 list agrees with `ASSAM_SOLAR` on every printed day. Saon 1431 began on 17
July 2024, after a text sankranti at 23:23 IST on the 16th, the day after, as the
midnight rule gives.

The 2026 list begins Bhada 1433 on 19 August and Ahin on 19 September. The text
sankrantis fall at 23:12 IST on 17 August and 23:40 IST on 17 September, so
`ASSAM_SOLAR` begins those months on the 18th, a day earlier. Every other 2026 month
meets. The gazette's Sun therefore runs up to about an hour behind the text's near the
Simha and Kanya sankrantis, but not in July 2024, at 23:23. No midnight shift meets both
years. C1302 stays open, and `ASSAM_SOLAR` stays on IST midnight. The 2026
restricted list's "1 November = 15 Kati" and "7 November = 21 Kati" contradict the
main list's "8 November = 21 Kati" and are taken as clerical.

## 7. What is not here

- **An Assamese almanac's page** for the tithi days (C1361). The rules are held to
  the notifications, not to a panjika's statement of each day's window.
- **The half day** (C1367). Muharram and Lakshmi Puja ship as whole days.
- **The restricted holidays**, and the one-off days declared by separate
  notifications (elections, accords, a holiday in lieu).
- **Bodoland's and the hill districts' own holidays**, which their councils declare.
