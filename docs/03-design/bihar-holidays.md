# Bihar's public holidays

Status: `built`, 2026-11-13. SR-03, the twelfth state set, after Maharashtra's
(`maharashtra-holidays.md`), West Bengal's (`west-bengal-holidays.md`), Kerala's
(`kerala-holidays.md`), Tamil Nadu's (`tamil-nadu-holidays.md`), Assam's
(`assam-holidays.md`), Karnataka's (`karnataka-holidays.md`), Telangana's
(`telangana-holidays.md`), Gujarat's (`gujarat-holidays.md`), Andhra Pradesh's
(`andhra-pradesh-holidays.md`), Uttar Pradesh's (`uttar-pradesh-holidays.md`) and
Odisha's (`odisha-holidays.md`). Cruxes C1560 to C1573
(`01-research/feature-universe/19-verification-cruxes.md`). Measured in
`festival-measured.md` §22.

## 1. The source

The General Administration Department (सामान्य प्रशासन विभाग) of the Government
of Bihar issues one notification each August to December,
`No. 3/एच0-<nn>/<year>-<n>`, from Patna, for the year after. It holds four
schedules:

- **Schedule-1, the general holidays** declared by executive order.
- **Schedule-2, the restricted holidays** (निर्बन्धित अवकाश), of which an
  employee takes up to three.
- **Schedule-3, the public holidays** under the Negotiable Instruments Act,
  1881.
- **Schedule-4**, the banks' closing of accounts on 1 April (C1572).
- **Note 1**: the Muslim festivals' days may change with the sighting of the
  moon (C1563).

Each row prints the Gregorian date, the weekday and the festival; the Hindu
days are those of the Government Sanskrit College, Kajipur, Patna, as the
amendment of 6 November 2026 says of 2026's (its letter No. 67/25 of 12 July
2025). All eight years were read from the notification's own pages (C1560):

| year | notification | copy read |
|---|---|---|
| 2019 | No. 3/एच0-02/2018-सा0प्र0 15543, 29 November 2018 | the signed scan: `simpliance.in/uploads/notifdocs/432/The%20Bihar%20List%20Of%20Holidays%20For%202019.pdf` |
| 2020 | No. 3/एच0-03/2019-13501, 30 September 2019 | the signed scan: `simpliance.in/uploads/notifdocs/590/Bihar%20State%20List%20Of%20Holidays%202020.pdf` |
| 2021 | No. 3/एच0-02/2020-12008, 16 December 2020 | the signed scan: `simpliance.in/uploads/notifdocs/1036/Bihar%20State%20List%20Of%20Holidays%202021.pdf` |
| 2022 | No. 3/एच0-01/2021-11265, 24 September 2021 | the signed scan: `simpliance.in/uploads/notifdocs/1338/bihar-state-list-of-holidays-2022.pdf` |
| 2023 | No. 3/एच0-01/2022-15471, 31 August 2022 | the *Bihar Gazette*, Extraordinary, No. 653, as `greythr.com` republished it: `datocms-assets.com/40521/1666084110-bihar-state-list-of-holidays-2023_compressed.pdf` |
| 2024 | No. 3/एच0-02/2023-16953, 5 September 2023 | the *Bihar Gazette*, Extraordinary, No. 736 of 11 September 2023, as `teamlease.com` republished it (article 26500) |
| 2025 | No. 3/एच0-01/2024-15764, 1 October 2024 | the *Bihar Gazette*, Extraordinary, No. 956 of 3 October 2024, as `teamlease.com` republished it (article 35979) |
| 2026 | No. 3/एच0-01/2025-15401, 19 August 2025 | the signed scan, as `teamlease.com` republished it (article 49401) |

The amendment No. 3/एच0-01/2025-20490 of 6 November 2026 (*Bihar Gazette*,
Extraordinary, No. 1383; `teamlease.com` article 61174) moves 2026's Chitragupta
Puja and Bhai Dooj from 10 to 11 November (C1569). The state's servers
(`state.bihar.gov.in`, `egazette.bih.nic.in`) were not reachable from here.
Several scans set the text in a legacy Devanagari font, which does not extract;
those were read from the rendered pages.

## 2. The pack

`INDIA_BIHAR` holds the three schedules at any place. It is measured at Patna
(25.5941 N, 85.1376 E) on India's clock. Every day is kept in `["IN", "IN-BR"]`
(ISO 3166-2; `IN-BR` maps to the pack) and is `GAZETTED`. A restricted holiday
is `optional`; Vishwakarma Puja and the last Monday of Shravana name `HINDU`,
Christmas Eve `CHRISTIAN`, Guru Gobind Singh's and Guru Nanak's birthdays
`SIKH`.

| holiday | key | form | the day's rule is cited to |
|---|---|---|---|
| Republic Day, Independence Day, Gandhi Jayanti, Christmas, Good Friday | as `INDIA_CENTRAL` | the central list's rule | the notification |
| Maha Shivaratri, Holi, Ram Navami, Mahavir Jayanti, Buddha Purnima | `SHIVARATRI`, `HOLI`, `RAMA_NAVAMI`, `MAHAVIR_JAYANTI`, `BUDDHA_JAYANTI` | the central list's rule; Holi the day after `HOLIKA` | *Dharmasindhu* and a printed panchanga, as the central list cites them |
| Holika Dahan, Raksha Bandhan, Hartalika Teej, Kalash Sthapana (restricted); Janmashtami, Chitragupta Puja and Bhai Dooj | `HOLIKA`, `RAKSHABANDHAN`, `HARITALIKA`, `NAVARATRA_ARAMBHA`, `JANMASHTAMI`, `YAMA_DWITIYA` | `DHARMASINDHU`'s rule | *Dharmasindhu* (C1562) |
| Durga Puja (Vijaya Dashami) | `VIJAYA_DASHAMI` | `DHARMASINDHU`'s Devi visarjana | *Dharmasindhu* p. 70 (C1562) |
| Basant Panchami and Saraswati Puja, Samrat Ashok Ashtami, Kabir Jayanti, Durga Puja (Saptami, Ashtami), Chhath; Jiutiya (restricted) | `BASANTA_PANCHAMI`, `SAMRAT_ASHOK_ASHTAMI`, `KABIR_JAYANTI`, `PHULPATI`, `MAHA_ASHTAMI`, `CHHATH`, `JITIYA` | `NEPAL_PANCHANGA`'s rule; Samrat Ashok Ashtami its Chaite Dashain | a printed panchanga (C1562) |
| Sant Ravidas Jayanti | `RAVIDAS_JAYANTI` | Uttar Pradesh's rule, Magha's full moon at sunrise | Uttar Pradesh's list (C1521) |
| Durga Puja (Navami), Deepawali | `MAHA_NAVAMI`, `LAKSHMI_PUJA` | Telangana's rule; the Navami at midday | Telangana's list (C1562) |
| Anant Chaturdashi (restricted) | `ANANTA_CHATURDASHI` | the Jain pack's rule | Sangave (C1562) |
| Guru Gobind Singh Jayanti | `GURU_GOBIND_SINGH_JAYANTI` | Odisha's rule, Pausha's bright 7th at sunrise | Odisha's list (C1565) |
| Janaki Navami | `SITA_NAVAMI` | Vaishakha's bright 9th at sunrise | the notifications (C1561) |
| Holi, second day; Kharna (restricted); Chhath, the morning arghya; Durga Puja (Ekadashi, restricted) | `HOLI_SECOND_DAY`, `CHHATH_KHARNA`, `CHHATH_SECOND_DAY`, `VIJAYA_DASHAMI_NEXT_DAY` | two days after `HOLIKA`; the day before and after `CHHATH`; the day after `VIJAYA_DASHAMI` | the notification |
| Makar Sankranti (restricted) | `UTTARAYANA_PUNYAKALA` | the first of Makara of `TAMIL_SOLAR`, as Uttar Pradesh's list keeps it | the notifications (C1522, C1566) |
| Bihar Diwas, Dr Ambedkar Jayanti, Veer Kunwar Singh Jayanti, May Day; New Year's Day, Jannayak Karpoori Thakur Jayanti, Anugrah Narayan Sinha Jayanti, Vishwakarma Puja, Jayaprakash Narayan Jayanti, Shri Krishna Sinha Jayanti, Dr Rajendra Prasad Jayanti, Christmas Eve (restricted) | `BIHAR_DIWAS`, `AMBEDKAR_JAYANTI`, `VEER_KUNWAR_SINGH_JAYANTI`, `INTERNATIONAL_LABOUR_DAY`, `NEW_YEAR_S_DAY_AD`, `KARPOORI_THAKUR_JAYANTI`, `ANUGRAH_NARAYAN_SINHA_JAYANTI`, `VISHWAKARMA_PUJA`, `JAYAPRAKASH_NARAYAN_JAYANTI`, `SHRI_KRISHNA_SINHA_JAYANTI`, `RAJENDRA_PRASAD_JAYANTI`, `CHRISTMAS_EVE` | a Gregorian date | the notification |

Hartalika Teej and Jiutiya are on the lists of 2025 and 2026 only. The months
are amanta, as every shipped rule names them: Jiutiya is Bhadrapada's dark 8th,
the notifications' Ashvina krishna ashtami. One new lunar rule was needed,
Janaki Navami, and no new form. The new keys are `SAMRAT_ASHOK_ASHTAMI`,
`CHHATH_KHARNA`, `CHHATH_SECOND_DAY` and six civil days.

## 3. The sky

The lunar days are reckoned under the modern sky, the built-in ephemeris with
the default profile. Makar Sankranti uses `TAMIL_SOLAR`'s fixed place, Chennai,
on India's clock: its first of Makara is the day of the ingress when the Sun
enters before sunset, and the next day otherwise (C1522).

## 4. The days printed, not reckoned

`PublishedSet::india_bihar` (`in-br-gad-printed`) carries:

- **Marked `subjectTo: MOON_SIGHTING`**, the ten Islamic days Note 1 leaves to
  the moon (C1563):
  - public: Shab-e-Barat, Id-ul-Fitr, Id-uz-Zoha (Bakrid), Muharram, Chehlum and
    Hazrat Mohammad Saheb's birthday, as `SHAB_E_BARAT`, `EID_AL_FITR`,
    `EID_AL_ADHA`, `MUHARRAM`, `ARBAEEN` and `MAWLID`;
  - restricted, `optional`: Jamat-ul-Vida and the days after Id-ul-Fitr,
    Id-uz-Zoha and Muharram, as `JUMAT_UL_VIDA`, `EID_AL_FITR_NEXT_DAY`,
    `EID_AL_ADHA_NEXT_DAY` and `MUHARRAM_SECOND_DAY`.

  2020's list prints the Prophet's birthday on 30 November; its weekday and its
  Hijri date give 30 October, which the set carries.
- **Unmarked**: the restricted last Monday of Shravana, `LAST_SHRAVANI_SOMVAR`,
  which no shipped form reckons (C1564); and Guru Nanak's birthday,
  `GURU_NANAK_JAYANTI`, on the lists of 2019 and 2020 only (C1573).

That is 12 days in 2019 and 2020 and 11 in each later year: 90 days.

## 5. Against the print

At Patna, 341 of the 363 rows meet the print (`festival-measured.md` §22, and
per year in `crates/sdk/tests/bihar.rs`). The rows are each year's reckoned days
of the three schedules and the sixteen fixed days, which stand in every year.
A row is a key's first day of the year; 2025's second Guru Gobind Singh
Jayanti, 27 December, meets the print too, and 2022's, 29 December, which no list
holds, parts with 2023's (C1565):

| year | met / printed |
|---|---|
| 2019 | 43 / 45 |
| 2020 | 44 / 45 |
| 2021 | 44 / 45 |
| 2022 | 42 / 45 |
| 2023 | 43 / 45 |
| 2024 | 41 / 45 |
| 2025 | 42 / 47 |
| 2026 | 42 / 46 |

Each lunar day's rule is the shipped rule that meets the most notifications.
Every shipped pack's rule for each key was measured at Patna against the eight
years, and a tie keeps the text's (C1562).

Twenty-two rows part, each named with its cause. No rule was fitted to them:

- **The Durga Puja's days** (C1567):
  - Mahanavami, 2019 and 2020. Telangana's midday rule takes the earlier day,
    and the notification the day whose sunrise holds the 9th.
  - Vijaya Dashami, 2024, and the day after it. The notification lists the
    12th, whose afternoon holds the 10th, where the visarjana keeps the 13th.
  - Saptami and Ashtami, 2026. Each tithi began after sunrise, and the
    notification lists the day it fills.
- **Chhath, 2025** (C1568). The 6th began ten minutes after the 27th's sunrise;
  the notification lists the 27th, the evening's offering, and Kharna and the
  morning arghya part with it.
- **Holi** (C1570). Holika Dahan in 2023, listed a day before the text's rule;
  Holi in 2024 and its second day, listed on the day whose sunrise holds
  Chaitra's dark 1st.
- **A later order** (C1569). Deepawali 2025 was moved to 20 October and Bhai
  Dooj 2026 to 11 November after the notification; in both the rule takes the
  day moved to.
- **A tithi held over two days** (C1571): Samrat Ashok Ashtami 2019, Raksha
  Bandhan 2022, Janmashtami 2022 and 2025, and Rama Navami 2026.
- **Janaki Navami, 2021** (C1561). The notification lists the day whose midday
  holds the 9th.
- **Makar Sankranti, 2022** (C1566). The Sun entered Makara at 14:35 on 14
  January and the notification lists the 15th.
- **Guru Gobind Singh Jayanti, 2023** (C1565). No Pausha bright 7th falls in
  the year; the notification lists 5 January.

## 6. What is not here

- **Schedule-4**, the banks' closing of accounts, the polling days of an
  election, and a district's local holidays (C1572).
- **Later orders that move a day** (C1569). The pack holds the notifications.
- **The Kajipur college's list** (C1561). The department's Hindu days are taken
  from it, and it is not read.
- **A form for a weekday of a lunisolar month** (C1564).
