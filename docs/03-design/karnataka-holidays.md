# Karnataka's public holidays

Status: `built`, 2026-11-12. SR-03, the sixth state set, after Maharashtra's
(`maharashtra-holidays.md`), West Bengal's (`west-bengal-holidays.md`), Kerala's
(`kerala-holidays.md`), Tamil Nadu's (`tamil-nadu-holidays.md`) and Assam's
(`assam-holidays.md`). Cruxes C1370 to C1387
(`01-research/feature-universe/19-verification-cruxes.md`). Measured in
`festival-measured.md` §15.

## 1. The source

The Department of Personnel and Administrative Reforms (State Protocol) issues one file
each November, `DPAR <n> HHL <year>`, for the year after. It holds three documents:

- **Notification-I**, the general holidays. It opens with all Sundays and second
  Saturdays (and fourth Saturdays from 2020) and the days of its table. The table leaves
  out a holiday that falls on such a weekend day, and **Note 1 names those days with
  their dates**, so the list is the table and Note 1 together. Note 3 is the moon clause:
  a Muslim festival moves to the day it is observed. Note 4 declares Kodagu district's
  local holidays.
- **The Annexure**, the restricted holidays, of which an employee may take two. Its notes
  name the days it leaves out because they fall on a weekend day or on a general holiday.
- **Notification-II**, the same list under section 25 of the Negotiable Instruments Act,
  1881, which adds the banks' closing of accounts on 1 April.

Seven of the eight years were read from the notification's own pages (C1370):

| year | notification | copy read |
|---|---|---|
| 2019 | DPAR 13 HHL 2018, November 2018 | a news site's list of the general holidays (rank 3); no restricted list |
| 2020 | DPAR 14 HHL 2019, 22 November 2019 | the signed English scan of all three documents, a compliance site's copy |
| 2021 | DPAR 12 HHL 2020, 21 November 2020 | the *Karnataka Rajyapatra*, Part IV-A No. 582, the gazette's pages (Kannada) |
| 2022 | DPAR 37 HHL 2021, 30 November 2021, with its corrigendum of 3 December 2021 | the signed Kannada and English scans, enclosed in a state examinations authority's circular |
| 2023 | DPAR 16 HHL 2022, 21 November 2022 | the signed Kannada scan, a compliance site's copy |
| 2024 | DPAR 16 HHL 2023, 28 November 2023 | the signed English scan of all three documents, an HR firm's copy |
| 2025 | DPAR 16 HHL 2024, 21 November 2024 | the signed Kannada set and the English Notification-I, which agree |
| 2026 | DPAR 16 HHL 2025, 17 November 2025 | the signed Kannada set |

From 2023 the notifications are in Kannada only. The department's own site was not
reachable, and the web archive holds no copy of its lists.

## 2. The pack

`INDIA_KARNATAKA` holds the list at any place. It is measured at Bengaluru (12.9716 N,
77.5946 E) on India's clock. Every day is kept in `["IN", "IN-KA"]` (ISO 3166-2;
`IN-KA` maps to the pack, SR-89) and is `GAZETTED`. A restricted holiday is `optional`
with no community. The banks' closing of accounts names the community `BANKS`.

| holiday | key | form | the day's rule is cited to |
|---|---|---|---|
| Republic Day, Independence Day, Gandhi Jayanti, Christmas, Good Friday, Buddha Purnima (restricted), Guru Nanak Jayanti (restricted) | as `INDIA_CENTRAL` | the central list's rule | the notification |
| Maha Shivaratri, Mahavira Jayanti | `SHIVARATRI`, `MAHAVIR_JAYANTI` | the central list's lunar rule | *Dharmasindhu*; the central list |
| Ambedkar Jayanti, May Day, Kannada Rajyotsava, the banks' closing of accounts; New Year, Kanya Mariyamma Jayanti, Vishwakarma Jayanti, Christmas Eve (restricted) | `AMBEDKAR_JAYANTI`, `INTERNATIONAL_LABOUR_DAY`, `KANNADA_RAJYOTSAVA`, `BANK_ACCOUNTS_CLOSING`, `NEW_YEAR_S_DAY_AD`, `KANYA_MARIYAMMA_JAYANTI`, `VISHWAKARMA_PUJA`, `CHRISTMAS_EVE` | a Gregorian date | the notification |
| Holy Saturday (restricted) | `EASTER_SATURDAY` | Easter less one day | the notification |
| Ugadi, Basava Jayanti, Varasiddhi Vinayaka Vrata, Mahalaya Amavasye, Naraka Chaturdashi, Ananta Padmanabha Vrata (restricted) | `SAMVATSARA_ARAMBHA`, `BASAVA_JAYANTI`, `GANESHA_CHATURTHI`, `AMAVASYA_SHRADDHA`, `NARAK_CHATURDASHI`, `ANANTA_CHATURDASHI` | Nepal's printed panchanga's rule; Basava Jayanti is its Akshaya Tritiya | a printed panchanga |
| Holi, Yajur Upakarma, Raksha Bandhana, Krishna Janmashtami, Swarnagowri Vrata (all restricted) | `HOLIKA`, `UPAKARMA_TAITTIRIYA`, `RAKSHABANDHAN`, `JANMASHTAMI`, `SWARNA_GOWRI_VRATA` | *Dharmasindhu*'s rule; Swarnagowri is its Haritalika, Bhadrapada's bright 3rd | *Dharmasindhu* |
| Mahanavami, Vijayadashami, Maharshi Valmiki Jayanti, Balipadyami, Kanakadasa Jayanti; Devara Dasimayya Jayanti, Sri Ramanavami, Shankaracharya Jayanti, Madhva Navami (restricted) | `MAHA_NAVAMI`, `VIJAYA_DASHAMI`, `VALMIKI_JAYANTI`, `BALI_PRATIPADA`, `KANAKADASA_JAYANTI`, `DEVARA_DASIMAYYA_JAYANTI`, `RAMA_NAVAMI`, `SHANKARACHARYA_JAYANTI`, `MADHVA_NAVAMI` | a tithi of an amanta month at sunrise | the notifications (C1371, C1375) |
| Makara Sankranti (Uttarayana Punyakala), Tula Sankramana (restricted) | `UTTARAYANA_PUNYAKALA`, `TULA_SANKRANTI` | Makaram 1 and Thulam 1 of `MALAYALAM_SOLAR` | the notifications (C1373) |
| Souramana Ugadi (restricted) | `SOURAMANA_UGADI` | the civil day of the Sun's entry into Mesha | the notifications (C1372) |
| Onam, Brahmashri Narayanaguru Jayanti, Sri Ramanujacharya Jayanti (restricted) | `THIRUVONAM`, `SREE_NARAYANA_GURU_JAYANTHI`, `RAMANUJACHARYA_JAYANTI` | Shravana and Shatabhisha in Chingam, Ardra in Chittirai, the later of two (`nakshatraInMonth`) | Kerala's list (C1323); the notifications (C1374) |

The months are amanta. Naraka Chaturdashi always falls before Balipadyami, and Mahalaya
Amavasye moves into October in the adhika years 2020, 2023 and 2026, as Bhadrapada's
amanta new moon does. Karnataka's own tithi days are kept on these readings:

- Devara Dasimayya Jayanti is Chaitra's bright 5th, the fourth day after Ugadi in every
  year listed. No text giving the day was found.
- Basava Jayanti is Vaishakha's bright 3rd. Every notification prints it on one row with
  Akshaya Tritiya.
- Valmiki Jayanti is Ashvina's full moon, and Kanakadasa Jayanti is Kartika's dark 3rd.
  The sources differ on Kanakadasa's tithi. The print supports the dark 3rd, in all eight
  years at sunrise.
- Madhva Navami is Magha's bright 9th.

**Kept at sunrise (C1375).** Karnataka's own tithi days are read at sunrise. Devara
Dasimayya Jayanti, Madhva Navami and Kanakadasa Jayanti meet every year there. The
Navami, Vijayadashami and Valmiki Jayanti part in the years §5 names, and no other
window tried meets more years for them. Three days that other lists keep by another
window part from the notifications under those windows:

| rule | its source's window | under it | at sunrise |
|---|---|---|---|
| Sri Ramanavami | the central list's midday | 6 of 7 | 7 of 7 |
| Shankaracharya Jayanti | Nepal's print's midday | 3 of 7 | 7 of 7 |
| Balipadyami | *Dharmasindhu*'s nine muhurtas past sunrise | 6 of 8 | 8 of 8 |

The three are held to sunrise with the state's own days. This reading meets every year,
and it is the reading Karnataka's own days are held to, so it is not a fit to a single
year. The share of the morning (the first fifth of the daylight) also meets all three,
and sunrise is the simpler reading. Mahavira Jayanti,
Varasiddhi Vinayaka Vrata and the others keep their sources' rules: at sunrise they
meet no more years.

## 3. The sky

The lunar days are reckoned under the modern sky, the built-in ephemeris with the
default profile. The solar days use the regional calendars' fixed places on India's
clock:

- **Makara Sankranti** is Makaram 1 of `MALAYALAM_SOLAR` at Thiruvananthapuram: the
  sankranti's day when it falls before three fifths of the daylight, otherwise the day
  after. This meets all eight years, 2022 and 2026 among them, when the ingress fell after
  the three-fifths mark on the 14th and the notification printed the 15th. Tamil Nadu's `TAMIL_SOLAR` (by
  sunset) meets 6 of 8, `ASSAM_SOLAR`'s month end 6 and `BENGAL_SOLAR`'s 4. **Tula
  Sankramana** on the same calendar meets 6 of 7 (C1373).
- **Souramana Ugadi** is the civil day of the Sun's entry into Ashwini (Mesha). This
  meets all eight years, 13 April 2020 and 2024 among them. Medam 1 of `MALAYALAM_SOLAR`
  meets 4 of 8 and Chithirai 1 of `TAMIL_SOLAR` 6 (C1372).

## 4. The days printed, not reckoned

`PublishedSet::india_karnataka` (`in-ka-dpar-printed`) carries:

- **The days set by the moon**, each marked `subjectTo: MOON_SIGHTING` under Note 3:
  - Ramzan (Khutba-e-Ramzan), Bakrid, the last day of Moharam and Eid-Milad, as
    `EID_AL_FITR`, `EID_AL_ADHA`, `MUHARRAM` and `MAWLID`, in all eight years;
  - from 2020, the restricted Shab-e-Barat, Shab-e-Qadar and Jumat-ul-Vida, as
    `SHAB_E_BARAT`, `SHAB_E_QADR` and `JUMAT_UL_VIDA`.
- **Three restricted days printed with no rule**, from 2020 and unmarked:
  - **Varamahalakshmi Vrata** (`VARAMAHALAKSHMI_VRATA`), the Friday before Shravana's
    full moon. The language has no form for a weekday within a tithi span (C1376).
  - **Rig Upakarma** (`RIG_UPAKARMA`). Shravana in Chingam meets 5 of 7 years, but 2022
    and 2025 part (C1377).
  - **Huttari** (`HUTTARI`), Kodagu's harvest, which the Padi Igguthappa temple's
    astrologers set each year. Rohini in Vrishchika meets 4 of 7 (C1378).

## 5. Against the print

At Bengaluru, 246 of the 263 rows meet the print (`festival-measured.md` §15, 302 of
319 with the seven general civil dates, and per year in `crates/sdk/tests/karnataka.rs`). The rows are the reckoned and dated days and
the restricted ones; the seven general civil days stand in every year. Seventeen rows
part, each named with its cause. No rule was moved to meet them:

| year | met / printed |
|---|---|
| 2019 | 15 / 15 |
| 2020 | 34 / 36 |
| 2021 | 33 / 35 |
| 2022 | 33 / 35 |
| 2023 | 32 / 35 |
| 2024 | 32 / 36 |
| 2025 | 33 / 35 |
| 2026 | 34 / 36 |

- **Mahanavami and Vijayadashami, 2024; Vijayadashami, 2025** (C1380). In 2024 the
  bright 9th began at 12:07 on 11 October, and the notification keeps the 11th, whose
  afternoon the navami held. The bright 10th began at 10:59 on the 12th, and the
  notification keeps the 12th. The sunrise reading takes the 12th and the 13th. In
  2025 the bright 10th ran from 19:02 on 1 October to 19:11 on the 2nd, which is Gandhi
  Jayanti. The notification prints Vijayadashami on the 1st, on the Navami's row. No
  window met all eight years for both days: at midday each meets 5 or 6 of 8.
- **Krishna Janmashtami, 2021, 2022 and 2025** (C1381). *Dharmasindhu*'s midnight rule
  meets 4 of 7. In 2022 and 2025 the notification keeps the day whose sunrise the
  ashtami held, a day after the midnight. In 2021 the ashtami held both midnights,
  from 23:26 on 29 August, and the notification keeps the 29th. The sunrise reading
  also meets 4 of 7, and midday 5.
- **Yajur Upakarma, 2020 and 2026** (C1382). In 2020 the notification prints 1
  September, the next lunar month's full moon, in solar Simha. In 2026 it prints 27
  August, the day before its own Raksha Bandhana on the same full moon (09:09 on the
  27th to 09:49 on the 28th).
- **Ananta Padmanabha Vrata, 2020 and 2024** (C1383). The notification keeps the day
  whose afternoon the bright 14th held: 31 August 2020 (08:49 on the 31st to 09:39 on
  1 September), and 16 September 2024 (15:11 on the 16th to 11:45 on the 17th). Read at
  the afternoon (aparahna) the rule meets all seven years. No source giving that window
  was found, so the printed panchanga's rule ships.
- **Holi, 2023 and 2024** (C1384). The notification keeps Holika Dahan's full moon in
  2020, 2021, 2022, 2025 and 2026, and the day after in 2023 (8 March, the full moon
  having ended at 18:10 on the 7th) and 2024 (25 March, ending at 12:30). No rule meets
  more than 5 of 7.
- **Mahavira Jayanti, 2023** (C1385, as C1242). The bright 13th began at 06:25 on 3
  April, after Bengaluru's sunrise. The notification keeps the 3rd.
- **Varasiddhi Vinayaka Vrata, 2023** (C1386). The bright 4th ran from 12:40 on 18
  September to 13:44 on the 19th. The midday rule takes the 19th, and the notification
  keeps the 18th, with Swarnagowri named on the same day.
- **Maharshi Valmiki Jayanti, 2026** (C1387). The full moon ran from 11:57 on 25 October
  to 09:42 on the 26th. The notification keeps the 25th, a Sunday named in Note 1. At
  midday the rule meets 2026 and parts in 2025.
- **Tula Sankramana, 2021** (C1373). `MALAYALAM_SOLAR` begins Thulam on 17 October,
  and the notification keeps the 18th.
- **Sri Ramanujacharya Jayanti, 2022** (C1374). Ardra in Chittirai gives 6 May, and the
  notification keeps the 5th. The later of two meets 6 of 7: in 2024 Chittirai held
  Ardra on 14 April and 12 May, and the notification keeps 12 May. The earlier would
  meet 5.

## 6. What is not here

- **A Karnataka almanac's page** (C1371). The lunar days are held to the notifications,
  not to a panchanga's statement of each day's window.
- **Kodagu's district holidays** (C1379): Kail Muhurtha on 3 September, and Tula
  Sankramana and Huttari as local holidays. The pack holds the state's list, and the
  rule language has no district scope below `IN-KA`. Tula Sankramana and Huttari ship as
  the Annexure's restricted days.
- **The weekend**: the Sundays and second and fourth Saturdays themselves. Note 1's
  holidays that fall on one ship as days, as Kerala's Sunday rows do.
- **One-off days**, as an election day or a holiday a later order moves.
- **2019's restricted list**, and a rank-1 copy of 2019's notification (C1370).
