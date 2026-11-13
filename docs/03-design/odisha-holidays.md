# Odisha's public holidays

Status: `built`, 2026-11-12. SR-03, the eleventh state set, after Maharashtra's
(`maharashtra-holidays.md`), West Bengal's (`west-bengal-holidays.md`), Kerala's
(`kerala-holidays.md`), Tamil Nadu's (`tamil-nadu-holidays.md`), Assam's
(`assam-holidays.md`), Karnataka's (`karnataka-holidays.md`), Telangana's
(`telangana-holidays.md`), Gujarat's (`gujarat-holidays.md`), Andhra Pradesh's
(`andhra-pradesh-holidays.md`) and Uttar Pradesh's (`uttar-pradesh-holidays.md`). Cruxes C1480 to C1499
(`01-research/feature-universe/19-verification-cruxes.md`). Measured in
`festival-measured.md` §21.

## 1. The source

The Registration branch of the Revenue and Disaster Management Department issues
one notification each October or November, "List of Holidays for the year
<year>", for the State Government's offices and its Revenue and Magisterial
Courts, for the year after. It holds:

- **Annexure-I**, the general holidays, on weekdays.
- **Annexure-II**, the optional holidays, of which an employee may take one, with
  those that fall on a Sunday or a second or fourth Saturday named apart.
- **Annexure-III**, the general holidays that fall on a Sunday or a second or
  fourth Saturday. Offices are closed on all Sundays and second and fourth
  Saturdays besides.

No year's notification leaves a day to the sighting of the moon: the Islamic days
are printed as dates, with no clause (C1494). Copies go to the Mukti Mandap
Pandita Sabha of Puri, the Kohinoor Press and the *Asali Khadiratna Panjika*, the
almanac makers, and to the Positional Astronomy Centre; the almanacs were not
read (C1496).

Eight years were read from the notifications' own pages (C1480):

| year | notification | copy read |
|---|---|---|
| 2019 | No. RDM-Reg-MISC-0014-2018-40221/R&D.M., 29 October 2018 | the department's PDF, `revenue.odisha.gov.in/sites/default/files/2020-05/Holidays_State_Govt_2019.pdf`, through the Internet Archive (20210616210737) |
| 2020 | No. RDM-Reg-REGN-0050-2019-32550/R&D.M., 23 October 2019 | the department's PDF, `…/State_Govt_Holidays_2020.pdf`, through the Internet Archive (20210523010655) |
| 2021 | No. RDM-Reg-REGN-0072-2020-30754/R&D.M., 20 October 2020 | the signed scan, `…/2021-08/30754_20_10_20.pdf`, through the Internet Archive (20211011180901) |
| 2022 | No. RDM-Reg-REGN-0080-2021-34561/R&D.M., 17 November 2021 | the signed scan, `…/2021-11/34561_17_11_21.pdf`, through the Internet Archive (20211126012102) |
| 2023 | No. RDM-Reg-MISC-0014-2022-37021/R&D.M., 1 November 2022 | the signed scan, `…/2022-11/37021_01_11_22_0.pdf`, through the Internet Archive (20230602083815) |
| 2024 | No. RDM-Reg-MISC-0014-2023-39304/R&D.M., 7 November 2023 | the signed scan, `…/2023-11/39304_07_11_23.pdf`, through the Internet Archive (20240314232435) |
| 2025 | No. RDM-Reg-MISC-0013-2024-39077/R&D.M., 12 November 2024 | **as copied**: the signed scan enclosed in Biju Patnaik University of Technology's notice of 20 December 2024, `krupajal.ac.in/upload/file_1101251316270.pdf` |
| 2026 | No. RDM-Reg-MISC-0021-2025-42428/R&D.M., 11 November 2025 | **as copied**: the signed text a compliance site carries, `uncomplycate.com/wp-content/uploads/2025/11/Government-of-Odisha-List-of-Holidays-for-Government-Offices-and-Courts-in-Odisha-for-the-Year-2026-NOV132025.pdf` |

The department's own server (`revenue.odisha.gov.in`) was not reachable from
here; the six archived files are its own. Later orders were not read (C1495):
2024's holiday for the day after the Ratha Yatra (8 July), its Shree Mandira
Parikrama day (17 January) and its half day of 22 January; 2025's added Nuakhai
day (29 August); and 2026's amendment No. 21787, which news reports say moved
Id-ul-Zuha from 27 to 28 May. The pack keeps the days each notification prints.

## 2. The pack

`INDIA_ODISHA` holds the list at any place. It is measured at Bhubaneswar (the
Lok Seva Bhavan, 20.2961 N, 85.8245 E) on India's clock. Every day is kept in
`["IN", "IN-OD"]` (ISO 3166-2; `IN-OD` maps to the pack) and is `GAZETTED`. An
optional holiday is `optional`: Guru Gobind Singh's birthday names `SIKH`, Easter
Saturday and Christmas Eve `CHRISTIAN`. A day's `optional` is the latest
notification's (C1493): Buddha Purnima is optional to 2021 and general after.

| holiday | key | form | the day's rule is cited to |
|---|---|---|---|
| Republic Day, Independence Day, Gandhi Jayanti, Christmas, Good Friday | as `INDIA_CENTRAL` | the central list's rule | the notification |
| Maha Shivaratri, Shree Rama Navami, Buddha Purnima | `SHIVARATRI`, `RAMA_NAVAMI`, `BUDDHA_JAYANTI` | the central list's rule | the central list (C1485) |
| Janmashtami | `JANMASHTAMI` | *Dharmasindhu*'s rule, with its fast | *Dharmasindhu* |
| Jhulana Purnima | `JHULANA_PURNIMA` | *Dharmasindhu*'s Rakshabandhan rule | *Dharmasindhu* (C1489) |
| Basanta Panchami, Ratha Yatra, Ganesh Puja | `BASANTA_PANCHAMI`, `JAGANNATH_RATH_JATRA`, `GANESHA_CHATURTHI` | Nepal's printed panchanga's rules | a printed panchanga (C1486) |
| Nuakhai | `NUAKHAI` | Bhadrapada's bright 5th at midday, the panchanga's Rishi Panchami rule | a printed panchanga (C1499) |
| Mahalaya | `MAHALAYA_AMAVASYA` | Bhadrapada's new moon at the afternoon, the panchanga's shraddha rule | a printed panchanga |
| Maha Saptami, Maha Astami, Maha Navami | `PHULPATI`, `MAHA_ASHTAMI`, `MAHA_NAVAMI` | Ashvina's bright 7th, 8th and 9th at sunrise, the panchanga's rules | a printed panchanga (C1484) |
| Rahas Purnima | `RAHAS_PURNIMA` | Kartika's full moon at sunrise, the panchanga's Guru Nanak Jayanti rule | a printed panchanga |
| Anla Navami (optional) | `ANLA_NAVAMI` | Kartika's bright 9th at sunrise, the panchanga's Kushmanda Navami rule | a printed panchanga (C1490) |
| Dola Purnima, Kali Puja / Diwali | `DOLA_YATRA`, `KALI_PUJA` | West Bengal's rules: Phalguna's full moon at sunrise, Ashvina's new moon at midnight | West Bengal's list (C1488) |
| Holi | `HOLI` | the day after Dola Purnima | the notification |
| Bahuda Yatra (optional) | `BAHUDA_JATRA` | the eighth day after the Ratha Yatra | the notification (C1497) |
| Vijaya Dashami | `VIJAYA_DASHAMI` | the day after Maha Navami | the notification (C1484) |
| Sabitri Amabasya | `SAVITRI_AMAVASYA` | Vaishakha's new moon at sunrise | the notification |
| Kumar Purnima | `KUMAR_PURNIMA` | Ashvina's full moon at moonrise | the notification (C1487) |
| Prathamastami (optional) | `PRATHAMASHTAMI` | Kartika's dark 8th at sunrise | the notification |
| Guru Gobind Singh's birthday (optional) | `GURU_GOBIND_SINGH_JAYANTI` | Pausha's bright 7th at sunrise | the notification (C1491) |
| Maha Vishuva Sankranti | `MESHA_SANKRANTI` | the civil day the Sun enters Ashwini | the notification (C1481, C1483) |
| Makar Sankranti, Raja Sankranti, Pahili Raja | `UTTARAYANA_PUNYAKALA`, `MITHUNA_SANKRANTI`, `PAHILI_RAJA` | the last day of `BENGAL_SOLAR`'s Poush and Jyaistha, the civil day of the entry; Pahili Raja the day before | the notification (C1481, C1482) |
| Netaji's and Veer Surendra Sai's birthday, Utkal Divas, Ambedkar's birthday | `NETAJI_JAYANTI`, `UTKAL_DIVAS`, `AMBEDKAR_JAYANTI` | a Gregorian date | the notification |
| Raja Rammohan Roy's birthday, the Brahmo Samaj's foundation day, Christmas Eve (optional) | `RAM_MOHAN_ROY_JAYANTI`, `BRAHMO_SAMAJ_FOUNDATION_DAY`, `CHRISTMAS_EVE` | a Gregorian date | the notification (C1492) |
| Easter Saturday (optional) | `EASTER_SATURDAY` | the day before Gregorian Easter | the notification |

The months are amanta: Sabitri Amabasya is Vaishakha's new moon and
Prathamastami Kartika's dark 8th, which the Odia almanacs name in Jyeshtha and
Margashirsha by the purnimanta reckoning. No new rule form was needed. Thirteen
keys are new: `UTKAL_DIVAS`, `PAHILI_RAJA`, `NUAKHAI`, `SAVITRI_AMAVASYA`,
`JHULANA_PURNIMA`, `BAHUDA_JATRA`, `KUMAR_PURNIMA`, `RAHAS_PURNIMA`,
`ANLA_NAVAMI`, `PRATHAMASHTAMI`, `GURU_GOBIND_SINGH_JAYANTI`,
`RAM_MOHAN_ROY_JAYANTI` and `BRAHMO_SAMAJ_FOUNDATION_DAY`.

## 3. The sky

The lunar days are reckoned under the modern sky, the built-in ephemeris with
the default profile. The sankranti days are kept on the civil day of the Sun's
entry, as the notifications list them in all but two rows (C1481): Maha Vishuva
Sankranti by the entry into Ashwini on India's clock, and Makar and Raja
Sankranti by the generated `BENGAL_SOLAR` table, whose month ends on the civil
day of the entry.

## 4. The days printed, not reckoned

`PublishedSet::india_odisha` (`in-od-rdm-printed`) carries, all unmarked:

- **The Islamic days**, Id-ul-Fitre, Id-ul-Zuha, Muharram and the Prophet's
  birthday, as `EID_AL_FITR`, `EID_AL_ADHA`, `MUHARRAM` and `MAWLID`, in all eight
  years, and Shab-e-Qader (`SHAB_E_QADR`, optional) in the seven years listing it
  (not 2023). No notification leaves them to the moon (C1494).
- **Panchayati Raj Divas** (`PANCHAYATI_RAJ_DIVAS`), 5 March, listed 2019 to 2025
  and not in 2026 (C1493).
- **Mahaveer Jayanti** (`MAHAVIR_JAYANTI`), optional, listed 2019 to 2021: 2021's
  14 April is not Chaitra's bright 13th (25 April), so the print is kept as
  printed (C1493).
- **2026's day following Nuakhai** (`NUAKHAI_NEXT_DAY`), 16 September.

## 5. Against the print

At Bhubaneswar, 296 of the 318 rows meet the print (`festival-measured.md` §21,
and per year in `crates/sdk/tests/odisha.rs`). The rows are the reckoned and
dated days, general and optional, and the nine fixed days, which stand in every
year; a row meets when the first day the pack gives it is the print's. Twenty-two
rows part, each named with its cause. No rule was moved to meet them:

| year | met / printed |
|---|---|
| 2019 | 38 / 39 |
| 2020 | 36 / 39 |
| 2021 | 39 / 40 |
| 2022 | 39 / 40 |
| 2023 | 38 / 40 |
| 2024 | 38 / 40 |
| 2025 | 36 / 40 |
| 2026 | 32 / 40 |

- **The Brahmo Samaj's foundation day, 2019 to 2025** (C1492). The notifications
  to 2025 list 25 January; 2026's lists 20 August, the day of the Samaj's first
  meeting in 1828, and the pack keeps the latest. Seven rows.
- **Pahili Raja and Raja Sankranti, 2020** (C1482). The Sun entered Gemini at
  00:01 on 15 June by the built-in sky, and the notification lists Raja Sankranti
  on the 15th; the generated Bengal table, whose entry falls before midnight,
  closes Jyaistha on the 14th, so the rule gives the 14th and Pahili Raja the
  13th.
- **Maha Vishuva Sankranti, 2024** (C1483). The Sun entered Aries at 21:06 on
  13 April; the rule takes the civil day of the entry and the notification the
  14th, the first sunrise after it. In 2020 (20:30 on 13 April) the notification
  lists the 13th, so the two years read the evening entry two ways.
- **Guru Gobind Singh's birthday, 2023 and 2026** (C1491). Pausha's bright 7th
  fell on 29 December 2022 and 27 December 2025, so the rule finds none in
  2023 or 2026; the notifications list 5 and 6 January. In 2022 and 2025 the
  rule finds two, and the first meets the print.
- **Basanta Panchami, 2025** (C1486). The bright 5th ran from 09:14 on
  2 February to 06:53 on the 3rd; the sunrise rule takes the 3rd and the
  notification lists the 2nd, whose forenoon the 5th held, as C1272.
- **Kumar Purnima, 2025** (C1487). The full moon ran from 12:24 on 6 October to
  09:17 on the 7th; the moonrise rule takes the 6th, the notification the 7th.
  In 2026 (11:56 on 25 October to 09:41 on the 26th) it lists the 25th, the
  moonrise's day. The moonrise rule meets 7 of 8; Kojagrat's midnight rule 5.
- **Kali Puja, 2025** (C1488). The new moon ran from 15:45 on 20 October to
  17:55 on the 21st; the midnight rule takes the 20th and the notification the
  21st. In 2024 (15:53 on 31 October to 18:17 on 1 November) it lists the 31st,
  the midnight's day, so the two years disagree.
- **Shree Rama Navami, 2026** (C1485). The bright 9th ran from 11:49 on 26 March
  to 10:07 on the 27th; the central midday rule takes the 26th, the notification
  the 27th. A sunrise reading meets 2026 and parts 2019, as C1443.
- **Jhulana Purnima, 2026** (C1489). The full moon ran from 09:09 on 27 August
  to 09:48 on the 28th; *Dharmasindhu*'s rule takes the 28th, the notification
  the 27th. A midday reading meets all eight years and is not the rule shipped.
- **The Durga days, 2026** (C1484). The bright 7th began at 05:54 on 17 October,
  the 8th at 08:28 on the 18th, the 9th at 10:52 on the 19th and the 10th at
  12:50 on the 20th; the sunrise rules take the 18th, 19th and 20th, and the
  notification lists the 17th, 18th and 19th, with Vijaya Dashami on the 20th.
  A morning reading meets 2026 and parts 2020. Four rows.
- **Anla Navami, 2026** (C1490). The bright 9th ran from 06:05 on 18 November to
  07:06 on the 19th; the sunrise rule takes the 19th and the notification the
  18th.

## 6. What is not here

- **The Odia almanacs** (C1496): the Pandita Sabha's and the *Khadiratna*
  panjikas, which the notifications' days follow, were not read.
- **2027's notification** (C1495), issued after this pack was measured, and the
  later orders §1 names.
- **The weekend**: the Sundays and second and fourth Saturdays themselves.
  Annexure-III's holidays and Annexure-II's that fall on one ship as days.
