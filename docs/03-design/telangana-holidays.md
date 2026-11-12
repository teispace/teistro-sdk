# Telangana's public holidays

Status: `built`, 2026-11-12. SR-03, the seventh state set, after Maharashtra's
(`maharashtra-holidays.md`), West Bengal's (`west-bengal-holidays.md`), Kerala's
(`kerala-holidays.md`), Tamil Nadu's (`tamil-nadu-holidays.md`), Assam's
(`assam-holidays.md`) and Karnataka's (`karnataka-holidays.md`). Cruxes C1388 to
C1400 (`01-research/feature-universe/19-verification-cruxes.md`). Measured in
`festival-measured.md` §16.

## 1. The source

The General Administration (Spl.E) Department issues one Government Order each
November or December, `G.O.Rt.No.<n>`, "Public Services – Holidays – General
Holidays and Optional Holidays for the year <year> – Notified", for the year
after. It holds:

- **Annexure-I**, the general holidays. Its Part II names the holidays that fall
  on a Sunday or a second Saturday, with their dates, so the list is both parts
  together. Offices are closed on all Sundays and second Saturdays besides.
- **Annexure-II**, the optional holidays, of which an employee may take five. A
  row that falls on a general holiday says so ("General Holiday in view of
  ...").
- **Paragraph 5**, the moon clause: should the day of Idu'l Fitr, Idu'l Azha,
  Muharram or Id-e-Milad change from the notified day, the change is announced
  by the press or the media, and the optional days set by the moon move with
  the sighting.

All eight years were read from the G.O.'s own pages (C1399):

| year | G.O. | copy read |
|---|---|---|
| 2019 | G.O.Rt.No.2493, 16 November 2018 | the G.O.'s text, republished by Jawaharlal Nehru Technological University Hyderabad: `jntuh.ac.in/uploads/bulletins/2018GAD_RT2493.PDF` |
| 2020 | G.O.Rt.No.3022, 21 November 2019 | the signed scan, in JNTU Hyderabad's circular of 23 November 2019: `jntuh.ac.in/uploads/bulletins/GeneralHolidaysOptionalHolidaysfortheyear2020.pdf` |
| 2021 | G.O.Rt.No.1756, 9 November 2020 | the signed scan, in JNTU Hyderabad's circular of 10 November 2020: `jntuh.ac.in/uploads/bulletins/generalholidaysandoptionalholidays.pdf` |
| 2022 | G.O.Rt.No.2618, 26 November 2021 | a staff site's copy of the G.O.: `labourlawadvisor.in/blog/wp-content/uploads/2022/01/telangana-state-list-of-holidays-2022.pdf` |
| 2023 | G.O.Rt.No.2140, 15 November 2022 | the signed scan, with G.O.Rt.No.2141 (the Negotiable Instruments Act list) and the Labour Department's Proc. E/2152/2022 of 12 December 2022: `simpliance.in/uploads/notifdocs/1629/telangana-state-list-of-holidays-2023.pdf` |
| 2024 | G.O.Rt.No.1633, 12 December 2023 | a staff site's copy of the G.O.: `promptpersonnel.com/wp-content/uploads/2023/12/telangana-state-list-of-holidays-2024.pdf` |
| 2025 | G.O.Rt.No.1479, 8 November 2024 | the signed scan, an HR firm's copy: `datocms-assets.com/40521/1731987486-telangana-holidays-list-of-2025.pdf` |
| 2026 | G.O.Rt.No.1715, 6 December 2025 | the signed scan, a compliance site's copy: `comply4hr.com/noti/2016/TLGSEA2025Jun12.pdf` |

The state's own servers (`telangana.gov.in`, `goir.telangana.gov.in`) were not
reachable from here, and no archived copy of the G.O.s was found on them.
G.O.Rt.No.381 of 19 March 2025, which moved 2025's optional Shahadat Hzt Ali
from 21 to 22 March, was not read: the pack keeps the day G.O.Rt.No.1479 prints
(C1399).

## 2. The pack

`INDIA_TELANGANA` holds the list at any place. It is measured at Hyderabad
(17.385 N, 78.4867 E) on India's clock. Every day is kept in `["IN", "IN-TG"]`
(ISO 3166-2; `IN-TG` maps to the pack, SR-89) and is `GAZETTED`. An optional
holiday is `optional` with no community; the Parsi New Year names `PARSI`. A
day's `optional` is the latest G.O.'s: New Year's Day is a general holiday to
2025 and optional in 2026 (C1400), and Durgashtami, a general holiday in 2019
and 2020, is optional from 2021.

| holiday | key | form | the day's rule is cited to |
|---|---|---|---|
| Republic Day, Independence Day, Gandhi Jayanti, Christmas, Good Friday | as `INDIA_CENTRAL` | the central list's rule | the G.O. |
| Maha Shivaratri, Holi, Kartika Purnima / Guru Nanak's birthday; Mahaveer Jayanthi and Buddha Purnima (optional) | `SHIVARATRI`, `HOLI`, `GURU_NANAK_JAYANTI`, `MAHAVIR_JAYANTI`, `BUDDHA_JAYANTI` | the central list's rule; Holi the day after `HOLIKA`, which the pack carries as the central list does | *Dharmasindhu*; the central list |
| New Year's Day (optional in 2026), Babu Jagjivan Ram's birthday, Ambedkar Jayanti, Christmas Eve (optional), Boxing Day | `NEW_YEAR_S_DAY_AD`, `JAGJIVAN_RAM_JAYANTI`, `AMBEDKAR_JAYANTI`, `CHRISTMAS_EVE`, `BOXING_DAY` | a Gregorian date | the G.O. |
| Ugadi; Basava Jayanthi, Sri Panchami, Ratha Yathra (optional) | `SAMVATSARA_ARAMBHA`, `BASAVA_JAYANTI`, `BASANTA_PANCHAMI`, `JAGANNATH_RATH_JATRA` | Nepal's printed panchanga's rule; Basava Jayanthi is its Akshaya Tritiya, as Karnataka's list keeps it | a printed panchanga |
| Sri Rama Navami; Sravana Purnima / Rakhi Purnima (optional) | `RAMA_NAVAMI`, `RAKSHABANDHAN` | Chaitra's bright 9th and Shravana's full moon at sunrise | the G.O.s (C1389) |
| Vinayaka Chavithi | `GANESHA_CHATURTHI` | Bhadrapada's bright 4th at midday, the later day only when it alone holds the tithi there | the G.O.s (C1389) |
| Bathukamma starting day | `BATHUKAMMA` | Bhadrapada's new moon (Mahalaya Amavasya) at sunrise | the G.O.s (C1395) |
| Saddula Bathukamma (2026); Durgashtami, Maharnavami (optional), Vijaya Dasami | `SADDULA_BATHUKAMMA`, `MAHA_ASHTAMI`, `MAHA_NAVAMI`, `VIJAYA_DASHAMI` | Ashvina's bright 8th, 9th and 10th at midday, the later day only when it alone holds the tithi | the G.O.s (C1393) |
| Following day of Vijaya Dasami (from 2021) | `VIJAYA_DASHAMI_NEXT_DAY` | a day after `VIJAYA_DASHAMI` | the G.O.s |
| Naraka Chaturdhi (optional) | `NARAK_CHATURDASHI` | Ashvina's dark 14th at the afternoon, the later day only when it alone holds the tithi | the G.O.s (C1397) |
| Deepavali | `LAKSHMI_PUJA` | Ashvina's new moon at pradosha, the later day only when it alone holds the tithi | the G.O.s (C1396) |
| Bhogi, Sankranti / Pongal, Kanumu (optional) | `BHOGI`, `UTTARAYANA_PUNYAKALA`, `KANUMU` | Dhanu's last day and Makaram's first and second days of `MALAYALAM_SOLAR` | the G.O.s (C1388) |
| Tamil New Year's Day (optional, from 2021) | `PUTHANDU` | Chithirai 1 of `TAMIL_VAKYA`, Tamil Nadu's rule | Tamil Nadu's orders (C1351) |
| Sri Krishna Astami, as per the Srivaishnava Agamam | `JANMASHTAMI_VAISHNAVA` | Rohini in Avani (`TAMIL_SOLAR` month 5), the earlier of two (`nakshatraInMonth`) | the G.O.s (C1394) |

The months are amanta. Deepavali keeps the central list's key, `LAKSHMI_PUJA`,
as Kerala's list does, with Telangana's own rule. No new rule form was needed.

**The Dasara days at midday (C1393).** Vinayaka Chavithi, read at midday with
the later day kept only when it alone holds the bright 4th, meets all eight
years, where Nepal's printed panchanga's midday rule parts in 2023, as it does
on Karnataka's list (C1386). Read the same way, the Dasara days meet every year from 2021 to
2026. 2019 and 2020 keep the day whose sunrise holds each tithi:

| rule | at midday | at sunrise |
|---|---|---|
| Durgashtami (2019 to 2025) | 5 of 7 | 6 of 7 |
| Saddula Bathukamma (2026) | 1 of 1 | 0 of 1 |
| Maharnavami | 7 of 8 | 5 of 8 |
| Vijaya Dasami | 7 of 8 | 5 of 8 |

The bright 8th meets 6 of 8 either way, and the four days are one festival, so
they are read alike. Saddula Bathukamma, the festival's last day, is
Durgashtami's day: 2026's G.O. prints it on 18 October and no Durgashtami row.

## 3. The sky

The lunar days are reckoned under the modern sky, the built-in ephemeris with
the default profile. The solar days use the regional calendars' fixed places on
India's clock:

- **Bhogi, Sankranti and Kanumu** are the last day of Dhanu and the first two of
  Makaram of `MALAYALAM_SOLAR`: the sankranti's day when it falls before three
  fifths of the daylight at Thiruvananthapuram, otherwise the day after, as
  Karnataka's Makara Sankranti is (C1373). All three meet all eight years,
  13 to 15 January 2021 and 2025 among them (C1388).
- **Tamil New Year** is Tamil Nadu's `PUTHANDU`, Chithirai 1 of `TAMIL_VAKYA`,
  14 April in all six years listed.
- **Sri Krishna Astami** is Rohini in Avani, solar Simha of `TAMIL_SOLAR`
  (C1394).

## 4. The days printed, not reckoned

`PublishedSet::india_telangana` (`in-tg-gad-printed`) carries:

- **The days set by the moon**, each marked `subjectTo: MOON_SIGHTING` under
  paragraph 5:
  - the general Eid ul Fitr (Ramzan), its following day, Eid ul Azha (Bakrid),
    Shahadat Imam Hussain (10th Moharam) and Eid Miladun Nabi, as
    `EID_AL_FITR`, `EID_AL_FITR_NEXT_DAY`, `EID_AL_ADHA`, `MUHARRAM` and
    `MAWLID`, in all eight years;
  - Annexure-II's eleven: Hazrath Ali's birthday (`HAZRAT_ALI_BIRTHDAY`, twice
    in 2026, on 3 January and 26 December), Shab-e-Meraj, Shab-e-Barat,
    Shahadat Hzt Ali, Jumatul Wida, Shab-e-Qader, Eid-e-Ghadeer, 9th Moharram,
    Arbayeen, Yaz Dahum Shareef and the birthday of Syed Mohammed Juvanpuri
    Mahdi Ma'ud.
- **Three days printed with no rule**, unmarked:
  - **Bonalu** (`BONALU`), a general holiday: a Monday of Ashadha, which the G.O.
    picks among Ashadha's Mondays. The language has no form for the last or a
    chosen weekday of a lunar month (C1390).
  - **Varalakshmi Vratham** (`VARAMAHALAKSHMI_VRATA`), optional: the Friday
    before Shravana's full moon, as Karnataka's restricted day (C1391, C1376).
  - **The Parsi New Year** (`PARSI_NEW_YEAR`), optional: the catalogue has no
    Shahenshahi calendar (C1243).

## 5. Against the print

At Hyderabad, 254 of the 265 rows meet the print (`festival-measured.md` §16,
and per year in `crates/sdk/tests/telangana.rs`). The rows are the reckoned and
dated days, general and optional, and the nine fixed days, which stand in every
year. Eleven rows part, each named with its cause. No rule was moved to meet
them:

| year | met / printed |
|---|---|
| 2019 | 27 / 30 |
| 2020 | 28 / 32 |
| 2021 | 34 / 34 |
| 2022 | 33 / 34 |
| 2023 | 34 / 34 |
| 2024 | 33 / 34 |
| 2025 | 33 / 34 |
| 2026 | 32 / 33 |

- **Durgashtami, Maharnavami and Vijaya Dasami, 2019; Durgashtami, 2020**
  (C1393). In 2019 the bright 8th ran from 09:50 on 5 October to 10:54 on the
  6th, the 9th to 12:38 on the 7th and the 10th to 14:50 on the 8th, and the
  G.O. lists each on the day whose sunrise the tithi held, a day after the
  midday reading. In 2020 the 8th ran from 06:57 on 23 October to 06:59 on the
  24th, and the G.O. lists the 24th, by two minutes of the tithi at sunrise.
- **Holi, 2020** (C1398). The full moon ran from 03:04 on 9 March to 23:17, so
  the Holika fire is the 9th and Holi the 10th; the G.O. lists Holi on the 9th.
  The other seven years keep the day after the fire.
- **Bathukamma starting day, 2020** (C1395). Bhadrapada's new moon fell on
  17 September. Ashvina was doubled, and the G.O. lists 17 October, the first
  day of the nija Ashvina, a month later. The new moon before the nija month,
  the adhika month's (04:53 on 16 October to 01:01 on the 17th), gives the
  16th, and no reading tried meets 2020 and the other six years.
- **Sri Krishna Astami, 2020, 2024 and 2025** (C1394). Rohini in Avani meets 5
  of 8. In 2020, before the G.O. named the Srivaishnava Agamam, it lists
  11 August, the dark 8th's day; Rohini of that month fell in Adi, and Avani's
  is 10 September. In 2024 Rohini ran from 15:55 on 26 August to 15:38 on the
  27th and the G.O. lists the 26th; in 2025 Avani held Rohini twice, the
  earlier from 04:38 on 17 August, and the G.O. lists the 16th. Both are the
  dark 8th's days. The central list's *Dharmasindhu* reading and the 8th at
  sunrise each meet 5 of 8 as well, parting in 2020 to 2022, the years the
  print kept Rohini's day; the later Rohini of two parts in 2025 by a month.
- **Deepavali, 2022** (C1396). The new moon ran from 17:27 on 24 October to
  16:18 on the 25th, ending before the 25th's sunset, so the evening reading
  takes the 24th; the G.O. lists the 25th, whose sunrise it held. The central
  list's rule parts in 2022 and 2024.
- **Naraka Chaturdhi, 2026** (C1397). The dark 14th ran from 10:48 on
  7 November to 11:28 on the 8th; the afternoon reading takes the 7th, and the
  G.O. lists the 8th, with Deepavali. The sunrise reading meets 4 of 7.

## 6. What is not here

- **A Telugu almanac's page** (C1389). The lunar days are held to the G.O.s, not
  to a panchanga's statement of each day's window.
- **Telangana Formation Day**, 2 June (C1392). No G.O. of 2019 to 2026 lists it
  among the general or optional holidays. The Labour Department's list for
  shops and establishments names it (Proc. E/2152/2022), and that list is not
  this pack's.
- **The Negotiable Instruments Act list** (G.O.Rt.No.2141 of 2022 and its kin),
  the banks' holidays (C1399). It was read for 2023 only; the pack holds the
  list for the state's offices.
- **The weekend**: the Sundays and second Saturdays themselves. Part II's
  holidays that fall on one ship as days, as Karnataka's Note 1 days do.
- **One-off days**, as an election day or a holiday a later order moves
  (G.O.Rt.No.381 of 2025, C1399).
