# West Bengal's public holidays

Status: `built`, 2026-11-09. SR-03, the second state set, after Maharashtra's
(`maharashtra-holidays.md`), from the state-holidays research of 2026-11-09 (its §2).
Cruxes C1270 to C1279 (`01-research/feature-universe/19-verification-cruxes.md`).
Measured in `festival-measured.md` §11.

## 1. The source

The Finance (Audit) Department issues a notification each year. Up to 2020 it was four
notifications of one day; from 2021 it is one with three lists:

- **List I**: public holidays under s. 25 of the Negotiable Instruments Act, 1881, and
  the yearly closing of bank accounts, for banks alone;
- **List II**: holidays under the State Government's order, among them the days added
  around a festival;
- **List III**: sectional holidays, each for the community it names;
- **a note** naming the occasions that fall on a Sunday, which are left out of the lists,
  and footnotes naming the occasions that fall on another holiday;
- **a moon clause**: Id-ul-Fitr, Id-ud-Zoha, Muharram, Fateha-Dwaz-Daham and
  Shab-e-Barat may move with the moon, and a separate order then fixes the day.

The state's server refused scripted fetches, so the copies read are a staff site's,
which give each notification's number, its lists and its notes (rank 1, as
Maharashtra's copies of the same kind are; C1279):

| year | notification | moved after |
|---|---|---|
| 2019 | Nos. 6812-F(P2) to 6815-F(P2), 5 November 2018 | |
| 2020 | Nos. 5962-F(P2) to 5965-F(P2), 1 November 2019 | |
| 2021 | No. 3100-F(P2), 5 November 2020 | Muharram to 20 August (No. 2365-F) |
| 2022 | No. 3640-F(P2), 26 November 2021 | |
| 2023 | No. 4331-F(P2), 21 October 2022 | Mahavir Jayanti to 3 April (No. 1430-F(P2), 25 March 2023) |
| 2024 | No. 6112-F(P2), 9 November 2023 | |
| 2025 | No. 4712-F(P2), 22 November 2024 | |
| 2026 | No. 4188-F(P2), 27 November 2025 | |

## 2. The pack

`INDIA_WEST_BENGAL` holds the list at any place, and is measured at Kolkata on India's
clock. Every day is kept in `["IN", "IN-WB"]` (ISO 3166-2; `WEST_BENGAL`, its first key, is read as an alias, SR-89), `GAZETTED`. The pack holds the
latest list's rows; a row new to the list (Rathayatra from 2021, Rakhi Bandhan from
2023, Ram Navami from 2025, Viswakarma Puja in 2026) is measured only in the years its
notice lists it.

| holiday | key | form | the day's rule is cited to |
|---|---|---|---|
| Republic Day, Independence Day, Gandhiji's birthday, Christmas, Good Friday | as `INDIA_CENTRAL` | civil date, or Easter less two days | the notification |
| Easter Saturday (List III, Christians) | `EASTER_SATURDAY` | Easter less one day | the notification |
| New Year's Day, Vivekananda's, Netaji's, Thakur Panchanan Barma's, Ambedkar's and Birsa Munda's birthdays, May Day, Hul Diwas (List III, Santhal), Bhanu Bhakta's birthday (Darjeeling and Kalimpong) | `NEW_YEAR_S_DAY_AD`, `VIVEKANANDA_JAYANTI`, `NETAJI_JAYANTI`, `THAKUR_PANCHANAN_BARMA_JAYANTI`, `AMBEDKAR_JAYANTI`, `BIRSA_MUNDA_JAYANTI`, `INTERNATIONAL_LABOUR_DAY`, `HUL_DIWAS`, `BHANU_JAYANTI` | a Gregorian date | the notification |
| the yearly closing of bank accounts | `BANK_ACCOUNTS_CLOSING` | 1 April; `communities: ["BANKS"]` | the notification |
| Bengali New Year, Tagore's birthday, Viswakarma Puja | `POHELA_BOISHAKH`, `RABINDRA_JAYANTI`, `VISHWAKARMA_PUJA` | Baishakh 1, Baishakh 25, Bhadra's last day of `BENGAL_SOLAR` | the notification; the calendar's rule (`regional-solar.md`) |
| Shivaratri, Ram Navami, Buddha Purnima, Mahavir Jayanti, Guru Nanak's birthday | as `INDIA_CENTRAL` | `INDIA_CENTRAL`'s lunar rule | *Dharmasindhu*; a printed panchanga |
| Rakhi Bandhan, Janmastami, Bhratridwitiya | `RAKSHABANDHAN`, `JANMASHTAMI`, `YAMA_DWITIYA` | `DHARMASINDHU`'s rule | *Dharmasindhu* |
| Saraswati Puja, Rathayatra, Mahalaya, the Puja's Saptami and Ashtami, Lakshmi Puja, Chhat Puja | `BASANTA_PANCHAMI`, `JAGANNATH_RATH_JATRA`, `AMAVASYA_SHRADDHA`, `PHULPATI`, `MAHA_ASHTAMI`, `KOJAGRAT_PURNIMA`, `CHHATH` | `NEPAL_PANCHANGA`'s rule | the printed panchanga, as that pack cites it |
| Doljatra | `DOLA_YATRA` | Phalguna's full moon, at sunrise | the Bengal panjikas, page not read (C1278) |
| Kali Puja | `KALI_PUJA` | Ashwina's (amanta) new moon at nishitha: the later day only when it alone holds it | the Bengal panjikas, page not read (C1278) |
| the Puja's Maha Navami | `MAHA_NAVAMI` | Ashwina's bright 9th, at sunrise | the Bengal convention (C1270) |
| Harichand Thakur's birthday | `HARICHAND_THAKUR_JAYANTI` | Phalguna's (amanta) dark 13th, at sunrise | the notice's own "Madhu Krishna Troyodashi" (C1274) |
| Holi, the day after Doljatra | `HOLI` | `DOLA_YATRA` + 1 | the notification |
| the Puja's Maha Chaturthi, Panchami and Shashthi | `DURGA_CHATURTHI`, `DURGA_PANCHAMI`, `DURGA_SHASHTHI` | `PHULPATI` − 3, − 2, − 1 | the notification |
| the Puja's Dasami | `VIJAYA_DASHAMI` | `MAHA_NAVAMI` + 1 | the notification |

**The Bengal convention for Durga Puja.** The notices count the Puja's days as a run.
Chaturthi, Panchami and Shashthi are the three days before the Saptami in every year
listed (20 of 20). The tithi rules would part in 2026, where the 6th holds both
sunrises (16 October 03:25 to 17 October 05:54). The Dasami is the day after the
Navami. The Saptami (Nabapatrika) and the Ashtami are reckoned, as Nepal's panchanga
reckons them; the Navami is the 9th at sunrise. This needs a `FollowingRule` that
counts back, which the Mithila pack made signed (`days: -3`).

**The central list's keys kept by another rule.** Diwali is Kali Puja, a new key; the
central list's `LAKSHMI_PUJA` (Diwali) is not on this list. Dasara is the day after the
Navami, Janmastami is *Dharmasindhu*'s Smarta day, and Holi is the day after Dol, not
the day after the Holika fire, so `HOLIKA` is not in the pack.

**Bengali New Year by the Sun.** The SDK ships the Bengal solar calendar
(`Calendar::BengalSolar`, `regional_solar::BENGAL`): a month begins on the civil day
after the sidereal sankranti's, or the day after next when the sankranti falls after
midnight. `POHELA_BOISHAKH` is its Baishakh 1, so no Mesha rule is reckoned here. It
meets all eight years, the 14 April years 2020 and 2024 among them.

`Fit::LaterAlone`, the later day only when it alone holds the tithi at a window, is new
for Kali Puja; the Nepal and Mithila packs read their rules through the same fits.

## 3. The days printed, not reckoned

`PublishedSet::india_west_bengal` carries, for each year a notification is in hand:

- **The days set by the moon**: Id-ul-Fitr, Id-ud-Zoha, Muharram and Fateha-Dwaz-Daham,
  as `EID_AL_FITR`, `EID_AL_ADHA`, `MUHARRAM` and `MAWLID`, named as the central list
  names them, and `SHAB_E_BARAT`, each marked `subjectTo: MOON_SIGHTING`. Muharram 2021
  stands unmarked on 20 August, the day No. 2365-F declared. The 2021 copy prints
  Id-ul-Fitr as "4 May, Friday", with its additional day on Saturday 15 May; the Friday
  was 14 May, which the pack carries.
- **The days added around a festival**: Saraswati Puja, Durga Puja (and its Tritiya in
  2020), Lakshmi Puja, Kali Puja, Bhratridwitiya, Chhat Puja and the two Ids. The
  Department chooses them each year around the weekend, so they are printed, not
  counted, and unmarked. A festival's added days in one year are numbered, first to
  fourth (`DURGA_PUJA_ADDITIONAL_DAY_1` to `_4`), since a published day of one key
  replaces another within a week of it.
- **The sectional days of a year or a few**: Baisakhi (2019, 2020), Guru Gobind Singh's
  Prakash Purab (2025) and Guru Rabidas's birthday (2022 to 2025), and the birthdays a
  footnote names on another holiday's day: Pandit Raghunath Murmu's (2022 to 2026),
  Parshwanath's Rathajatra (2024 to 2026) and Bir Chilarai's (2026).

Karam Puja is "to be notified later" every year, so it has no day. Guru Rabidas's 2026
day is misprinted ("Is February") and is left out. A year with no notification holds
none of these.

## 4. Against the print

At Kolkata over the built-in ephemeris, 206 of the 215 rows beside the twelve
every-year dates meet the print (`festival-measured.md` §11). The 215 are the lunar
and solar days, Good Friday and Easter Saturday, and the two birthdays listed from
2022, with the Sunday note's rows among them; the twelve fixed dates stand in all eight
years. Nine part, and each is named with its cause; no rule was moved to meet them:

- **The Navami and the Dasami, 2024** (C1270): the notice prints Ashtami and Navami
  together on 11 October and the Dasami on the 12th.
- **Lakshmi Puja, 2021** (C1271): *Dharmasindhu* p. 74's nishitha rule takes the 19th,
  the notice the 20th.
- **Saraswati Puja, 2025** (C1272): the 5th began after sunrise on 2 February; the note
  keeps the 2nd.
- **Chhat Puja, 2025** (C1273): the notice keeps the evening arghya's day.
- **Harichand Thakur's birthday, 2024** (C1274).
- **Janmastami, 2022 and 2025** (C1275).
- **Mahavir Jayanti, 2023** (C1277): declared on 3 April, where the rule at Kolkata
  gives the 4th, the day first listed.

The notices name Holi on Doljatra's day in 2024 and 2025, and list the day after as
"Day after Doljatra"; the holiday is the same day every year, so `HOLI` meets the print
in all eight (C1278).

## 5. What is not here

- **The panjika's own sky** (C1276). A Bengal panjika (Gupta Press, Surya Siddhanta; or
  Bisuddha Siddhanta, drik) may give other times for the rows that part. A context over
  the built-in ephemeris refuses `frame.siddhanta = SURYA`, as it should, so the
  question cannot be tested here.
- **Regions below the state.** Bhanu Bhakta's birthday is for Darjeeling and Kalimpong,
  Bir Chilarai's for North Bengal; both are kept in the state.
- **The double-asterisk mark**, a day kept only by offices whose Saturdays are not whole
  holidays.
- **Tamil Nadu**, the third state set (the research's §5).
