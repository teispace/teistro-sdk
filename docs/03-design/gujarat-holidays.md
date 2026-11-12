# Gujarat's public holidays

Status: `built`, 2026-11-12. SR-03, the eighth state set, after Maharashtra's
(`maharashtra-holidays.md`), West Bengal's (`west-bengal-holidays.md`), Kerala's
(`kerala-holidays.md`), Tamil Nadu's (`tamil-nadu-holidays.md`), Assam's
(`assam-holidays.md`), Karnataka's (`karnataka-holidays.md`) and Telangana's
(`telangana-holidays.md`). Cruxes C1440 to C1450
(`01-research/feature-universe/19-verification-cruxes.md`). Measured in
`festival-measured.md` §17.

## 1. The source

The General Administration Department issues one notification each October or
November, `No. GS/<n>/<year>/...<file>/GH`, from Sachivalaya, Gandhinagar, for
the year after, in English and in Gujarati. It holds three parts under the one
number:

- **Part I**, "Public Holidays, <year>", the public holidays for the State
  Government's offices. Its **Note (1)** names the holidays not notified because
  they fall on a Sunday (in 2023 and 2025, a second or fourth Saturday too), with their
  dates, so the list is Part I and Note (1) together. **Note (2)** is the moon
  clause: when a Muslim holiday does not fall on the day notified, Muslim
  employees may take an optional holiday on the day it is observed. Note (3)
  extends the list to the State's undertakings, boards, corporations and
  panchayat offices.
- **Part II**, the optional holidays, of which an employee may take two: some
  forty a year, among them Parsi (Shahenshahi and Kadmi), Jewish, Jain paksha,
  Sikh and Islamic days and Hindu lunar days.
- **Part III**, the public holidays for banks under s. 25 of the Negotiable
  Instruments Act, 1881, and the banks' closing of accounts on 1 April.

Each row prints its date, its Saka date and its weekday; the Gujarati text adds
the amanta tithi, as "Kartak sud-1" for the New Year. All eight years were read
from the notification's own pages (C1450):

| year | notification | copy read |
|---|---|---|
| 2019 | No. GS-32/2018-PHD/102018/1411/GH, 3 November 2018 | the Government Central Press print (GCPG-(H-527)), as a blog shared it: `drive.google.com/open?id=1D8yx6nJ0993LqFIaiiTys9x5HyE0ECNM`; the department's own `gad.gujarat.gov.in/personnel/Images/pdf/public-holidays-eng-2019.pdf` was not reachable from here |
| 2020 | No. GS/30/2019/PHD/102019/1111/GH, 19 October 2019 | a staff site's copy of its text, with Parts I and II (the banks') and Note (1): `gconnect.in/news/state-government-holidays-2020-gujarat.html` |
| 2021 | No. GS/33/2020/PHD/102020/500/GH, 28 October 2020 | the signed scan (H-260-200-10-2020), as a blog shared it: `drive.google.com/file/d/1HrDXtCNoTcg3pB-MY05p9aeLfbDsFPQf` |
| 2022 | No. GS/20/2021/PHD/102021/431/GH, 24 November 2021 | the signed scan, as a blog shared it: `drive.google.com/file/d/1-5hnICrBWGUoptlpRVALrqsnMEue_2cd` |
| 2023 | No. GS/16/2022/PHD/102022/603/GH, 5 November 2022 | the signed scan, English and Gujarati: `datocms-assets.com/40521/1668168247-gujarat-state-list-of-holidays-2023.pdf` |
| 2024 | No. GS/27/2023/JSR/102023/930/GH, 20 November 2023 | the signed scan, English and Gujarati: `simpliance.in/uploads/notifdocs/1939/gujarat-state-list-of-holidays-2024.pdf` |
| 2025 | No. GS/26/2024/JSR/2024/605/GH, 14 November 2024 | the signed scan, English and Gujarati: `simpliance.in/uploads/notifdocs/2428/gujarat-state-list-of-holidays-2025.pdf` |
| 2026 | No. GS/22/2025/JSR/2025/509/GH, 27 November 2025 | the *Gujarat Government Gazette*, Extraordinary No. 160, Part IV-A, 27 November 2025: `hrinformative.com/wp-content/uploads/2025/12/Gujarat-State-Holidays-List-2026-Notification.pdf` |

The 2020 copy is the text alone; its table and note agree with the press lists of
the day.

## 2. The pack

`INDIA_GUJARAT` holds the list at any place. It is measured at Gandhinagar
(23.2156 N, 72.6369 E) on India's clock. Every day is kept in `["IN", "IN-GJ"]`
(ISO 3166-2; `IN-GJ` maps to the pack, SR-89), is `GAZETTED`, and none is
`optional`: the pack holds Part I and Note (1), every day of which is a public
holiday. The Parsi New Year names `PARSI`.

| holiday | key | form | the day's rule is cited to |
|---|---|---|---|
| Republic Day, Independence Day, Mahatma Gandhi's Birthday, Christmas, Good Friday | as `INDIA_CENTRAL` | the central list's rule | the notification |
| Makar Sankranti (Uttarayan), Dr Baba Saheb Ambedkar's Birthday, Sardar Vallabhbhai Patel's Birthday | `UTTARAYANA_PUNYAKALA`, `AMBEDKAR_JAYANTI`, `SARDAR_PATEL_JAYANTI` | 14 January, 14 April, 31 October | the notification (C1440) |
| Maha Shivratri, Holi 2nd Day (Dhuleti), Shree Ram Navmi, Mahavir Janma Kalyanak, Janmashtami (Shravan Vad-8), Dusshera (Aaso sud-10), Diwali, Guru Nanak's Birthday | `SHIVARATRI`, `HOLI`, `RAMA_NAVAMI`, `MAHAVIR_JAYANTI`, `JANMASHTAMI`, `VIJAYA_DASHAMI`, `LAKSHMI_PUJA`, `GURU_NANAK_JAYANTI` | the central list's rule; Dhuleti the day after `HOLIKA`, which the pack carries as the central list does | *Dharmasindhu*, the *Hari Bhakti Vilasa* and a printed panchanga, as the central list cites them |
| Raksha Bandhan, Bhai Bij | `RAKSHABANDHAN`, `YAMA_DWITIYA` | `DHARMASINDHU`'s rule | *Dharmasindhu* pp. 49 and 79 |
| Chetichand, Bhagvan Shree Parshuram Jayanti | `CHETI_CHAND`, `PARASHURAMA_JAYANTI` | `NEPAL_PANCHANGA`'s rule: Chetichand is its Samvatsara arambha, Chaitra's bright 1st at sunrise (C1446); Parshuram Jayanti Vaishakha's bright 3rd by its share of the pradosha | a printed panchanga |
| Samvatsari (Chaturthi Paksha) | `SAMVATSARI` | the `JAIN` pack's rule, Bhadrapada's bright 4th at sunrise | the Jain pack's sources |
| Vikram Samvat New Year Day (Nutan Varsh, Besatu Varsh) | `VIKRAM_SAMVAT_NEW_YEAR` | Kartika's bright 1st at sunrise | the notifications (C1441, C1442) |

The months are amanta, and the Vikram Samvat year the notification names (2080 on
14 November 2023, 2081 on 2 November 2024) is the Kartikadi year the calendar
crate already reckons (`lunisolar::kartikadi_month`), the year the Vira Nirvana
Samvat is read from. Janmashtami's "Shravan Vad-8" is the amanta Shravana's dark
8th, the central list's `JANMASHTAMI` (the *Hari Bhakti Vilasa*'s Vaishnava day,
C851), which meets all eight years, 24 August 2019 and 7 September 2023 among
them, the Vaishnava days. One new rule was needed, the New Year, and no new
form. `StateList::taken` keeps a rule of any shipped pack under a state's key, as
`print_kept` does for the printed panchanga's; `print_kept` now reads through it.

**The New Year at sunrise (C1442).** Kartika's bright 1st at sunrise meets all
eight years. *Dharmasindhu*'s Bali pratipada, the earlier day unless the 1st
lasts nine muhurtas past the later sunrise, meets seven: in 2020 the 1st ran from
10:37 on 15 November to 07:06 on the 16th, holding the 16th's sunrise (06:53) by
thirteen minutes, and the text keeps the 15th; the notification lists the 16th,
with Bhai Bij. Maharashtra's list parts from the text the same year (C1241), and
Karnataka keeps its Balipadyami at sunrise (C1375).

## 3. The sky

The lunar days are reckoned under the modern sky, the built-in ephemeris with the
default profile, at Gandhinagar. Makar Sankranti is a Gregorian date: the
notification lists 14 January in all eight years, where the Sun entered Makara at
19:57 on 14 January 2019, 02:14 on 15 January 2020, 20:49 on 14 January 2023 and
02:45 on 15 January 2024 (C1440). Uttarayan in Gujarat is the kite-flying day,
kept on the date.

## 4. The days printed, not reckoned

`PublishedSet::india_gujarat` (`in-gj-gad-printed`) carries:

- **The days set by the moon**, each marked `subjectTo: MOON_SIGHTING` under
  Note (2): Ramjan-Eid (Eid-Ul-Fitra), Bakri-Eid (Eid-Ul-Adha), Muharram
  (Ashoora) and Eid-e-Meeladunnabi, as `EID_AL_FITR`, `EID_AL_ADHA`, `MUHARRAM`
  and `MAWLID`, in all eight years.
- **The Parsi New Year Day, Pateti (Parsi Shahenshahi)** (`PARSI_NEW_YEAR`),
  unmarked: the catalogue has no Shahenshahi calendar (C1243). It is 17 August
  2019, 16 August 2020 to 2023 and 15 August 2024 to 2026, a day earlier every
  four years.

## 5. Against the print

At Gandhinagar, 169 of the 176 rows meet the print (`festival-measured.md` §17,
and per year in `crates/sdk/tests/gujarat.rs`). The rows are the fifteen
reckoned days of each year and the seven fixed days, which stand in every year.
Seven rows part, each named with its cause. No rule was moved to meet them:

| year | met / printed |
|---|---|
| 2019 | 20 / 22 |
| 2020 | 22 / 22 |
| 2021 | 22 / 22 |
| 2022 | 21 / 22 |
| 2023 | 20 / 22 |
| 2024 | 21 / 22 |
| 2025 | 22 / 22 |
| 2026 | 21 / 22 |

- **Shree Ram Navmi, 2019** (C1443). The bright 9th ran from 11:41 on 13 April to
  09:35 on the 14th; the central list's midday rule takes the 13th, and the
  notification lists the 14th, the sunrise's day, in Note (1) as a Sunday. Kept
  at sunrise, the rule would meet 2019 and part in 2026, when the 9th ran from
  11:49 on 26 March to 10:07 on the 27th and the notification lists the 26th:
  seven of eight either way.
- **Dusshera, 2019 and 2022** (C1444). In 2019 the bright 10th ran from 12:38 on
  7 October to 14:50 on the 8th, holding both afternoons, and *Dharmasindhu*
  keeps the earlier; in 2022 it ran from 14:21 on 4 October to 12:00 on the 5th,
  only the 4th's afternoon holding it. Both times the notification lists the day
  whose sunrise holds the dashami. The sunrise reading meets five of eight,
  parting in 2020, 2024 and 2026, so the list keeps no single window.
- **Dhuleti, 2023 and 2026** (C1445). In 2023 the full moon ran from 16:17 on
  6 March to 18:10 on the 7th, ending before the 7th's sunset (18:45); in 2026
  from 17:56 on 2 March to 17:07 on the 3rd. Each time the earlier evening alone
  held the full moon, and the Holika fire is that evening; the notification lists
  Part II's Holi on the later day and Dhuleti on the day after, a day later than
  the rule, as the central list's notices do in both years (C854).
- **Mahavir Janma Kalyanak, 2023** (C1447). The bright 13th ran from 06:24 on
  3 April to 08:05 on the 4th, holding both sunrises (06:30 and 06:29), and the
  print's rule keeps the earlier; the notification lists the 4th, as
  Maharashtra's does (C1242).
- **Chetichand, 2024** (C1446). Chaitra's bright 1st ran from 23:50 on 8 April to
  20:31 on the 9th, and the rule takes the 9th; the notification lists the 10th,
  whose sunrise the bright 2nd holds. In the other seven years Chetichand falls on
  the bright 1st's sunrise day, the day of Part II's Gudi Padvo where both are
  printed, and the bright 2nd at sunrise meets one year of eight.

## 6. What is not here

- **A Gujarati almanac's page** (C1441). The New Year is held to the
  notifications, not to a panchanga's statement of the day.
- **Part II's optional holidays and Part III's banks' list** (C1448). Part II's
  forty-odd days, two of which an employee may take, are mostly kept by
  calendars the SDK does not reckon (the Parsi Shahenshahi and Kadmi years, the
  Hebrew year, the Islamic months by sighting); Part III is the banks' list.
  The pack holds the list for the State Government's offices.
- **Days on no year's list** (C1449). Gujarat Day, 1 May, is on none of the
  eight Part I lists. Ganesh Chaturthi is printed once, in 2020's Samvatsari row
  ("Samvatsari (Chaturthi Paksha) Ganesh Chaturthi", 22 August), the two falling
  on one day; in every other year it is a Part II day. The one-off orders that
  move or add a day, such as 2025's additional Diwali holidays, are not read.
- **The weekend**: the Sundays and second and fourth Saturdays themselves.
  Note (1)'s holidays that fall on one ship as days.
