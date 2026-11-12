# Andhra Pradesh's public holidays

Status: `built`, 2026-11-12. SR-03, the ninth state set, after Maharashtra's
(`maharashtra-holidays.md`), West Bengal's (`west-bengal-holidays.md`), Kerala's
(`kerala-holidays.md`), Tamil Nadu's (`tamil-nadu-holidays.md`), Assam's
(`assam-holidays.md`), Karnataka's (`karnataka-holidays.md`), Telangana's
(`telangana-holidays.md`) and Gujarat's (`gujarat-holidays.md`). Cruxes C1401 to C1408
(`01-research/feature-universe/19-verification-cruxes.md`). Measured in
`festival-measured.md` §18.

## 1. The source

The General Administration (Poll.B; Political.B in the headings from 2024) Department issues one
Government Order each November or December, `G.O.Rt.No.<n>`, "Holidays –
General Holidays and Optional Holidays for the year <year> – Declared" (from
2024 "– Notified"), for the year after. It holds:

- **Annexure-I**, the general holidays, and **Annexure-I(A)**, those of them that
  fall on a Sunday or a second Saturday, with their dates. Offices are closed on
  all Sundays and second Saturdays besides (para 2).
- **Annexure-II**, the optional holidays, of which an employee may take five
  (para 3), and **Annexure-II(A)**, those that fall on a Sunday.
- **Paragraph 5**, the moon clause: should the day of Ramzan, Bakrid, Moharam or
  Eid Miladun Nabi change "as per the sighting of the moon", or any Hindu
  holiday's, the change is announced through the media. 2021's para 5 names
  Shab-e-Barath as well, a general holiday that year. No year's para 5 names
  Annexure-II's other Islamic days (C1403).

Seven years were read from the G.O.'s own pages (C1402):

| year | G.O. | copy read |
|---|---|---|
| 2020 | G.O.Rt.No.2745, 5 December 2019 | its four annexures, reproduced in Rajiv Gandhi University of Knowledge Technologies' proceedings No. RGUKT-AP/GH & OH/GO/Adopt/2020 of 26 December 2019, which adopt it: `rgukt.in/pdfdoc/Holidays2020.pdf` |
| 2021 | G.O.Rt.No.1963, 15 December 2020 | the G.O.'s text, a news site's copy: `thecurrentindia.com/wp-content/uploads/2020/12/Government_Holidays_2021.pdf` |
| 2022 | G.O.Rt.No.1997, 26 November 2021 | the signed scan, with G.O.Rt.No.1998 (the Negotiable Instruments Act list) and the Labour Department's lists, a compliance site's copy: `simpliance.in/uploads/notifdocs/1345/andhra-pradesh-state-list-of-holidays-2022.pdf` |
| 2023 | G.O.Rt.No.2692, 15 December 2022 | the signed scan, RGUKT's copy: `rgukt.in/pdfdoc/2023APHolidays.pdf` |
| 2024 | G.O.Rt.No.2318, 30 November 2023 | the attested scan, a news site's copy: `thecurrentindia.com/wp-content/uploads/2024/03/GO-Rt-2318-General-Holidays-OHs-2024.pdf` |
| 2025 | G.O.Rt.No.2115, 6 December 2024 | the G.O.'s text as teachers' sites typed it, a compliance site's copy (read through the Internet Archive): `legalitysimplified.com/wp-content/uploads/2024/12/10050376.pdf` |
| 2026 | G.O.Rt.No.2276, 4 December 2025 | **rank 3**: its annexures as a teachers' site transcribed them, `apgovtschool.blogspot.com/2025/12/holidays-for-2026-calendar-year.html`; G.O.Rt.No.613 of 19 March 2026 (`static.collegedekho.com/media/uploads/2026/03/19/2026gad_rt613_e.pdf`), read in full, confirms its Ramzan of 20 March |

The state's servers (`goir.ap.gov.in`, `gad.ap.gov.in`, `apegazette.cgg.gov.in`)
were not reachable from here. 2019's G.O. (G.O.Rt.No.2413 of 14 November 2018,
as APTRANSCO's order cites it) was not found in a readable copy, and is not in
the pack. G.O.Rt.No.612 of 19 March 2026, which moved 2026's Ramzan to
21 March, and G.O.Rt.No.1176 and 1177 of 15 June 2026, which moved Moharrum,
were not read: the pack keeps the days G.O.Rt.No.2276 prints (C1402).

## 2. The pack

`INDIA_ANDHRA_PRADESH` holds the list at any place. It is measured at Amaravati
(the Secretariat at Velagapudi, 16.5193 N, 80.515 E) on India's clock. Every day
is kept in `["IN", "IN-AP"]` (ISO 3166-2; `IN-AP` maps to the pack, SR-89) and
is `GAZETTED`. An optional holiday is `optional` with no community; the Parsi New
Year names `PARSI`. A day's `optional` is the latest G.O.'s (C1400): Holi is
optional in 2020 and general after, Mahaveer Jayanthi general in 2022 (with
Ambedkar's birthday) and optional otherwise.

Andhra Pradesh and Telangana keep one Telugu almanac tradition, and the two
states' G.O.s print the same day for every Telugu row both list but the few §5
names. So a Telugu day reuses Telangana's rule (`StateList::taken`), cited to
this list's G.O. and to Telangana's:

| holiday | key | form | the day's rule is cited to |
|---|---|---|---|
| Republic Day, Independence Day, Gandhi Jayanthi, Christmas, Good Friday | as `INDIA_CENTRAL` | the central list's rule | the G.O. |
| Maha Sivaratri, Holi; Mahaveer Jayanthi, Buddha Purnima, Karthika Purnima / Guru Nanak Jayanthi (optional) | `SHIVARATRI`, `HOLI`, `MAHAVIR_JAYANTI`, `BUDDHA_JAYANTI`, `GURU_NANAK_JAYANTI` | the central list's rule; Holi the day after `HOLIKA`, which the pack carries as the central list does | *Dharmasindhu*; the central list (C1405) |
| New Year's Day, Christmas Eve, Boxing Day (optional); Babu Jagjivan Ram's birthday, Ambedkar's birthday | `NEW_YEAR_S_DAY_AD`, `CHRISTMAS_EVE`, `BOXING_DAY`, `JAGJIVAN_RAM_JAYANTI`, `AMBEDKAR_JAYANTI` | a Gregorian date | the G.O. |
| Ugadi; Basava Jayanthi, Ratha Yatra (optional) | `SAMVATSARA_ARAMBHA`, `BASAVA_JAYANTI`, `JAGANNATH_RATH_JATRA` | Nepal's printed panchanga's rule; Basava Jayanthi is its Akshaya Tritiya | a printed panchanga (C1407) |
| Sri Krishna Astami | `JANMASHTAMI` | *Dharmasindhu*'s rule, as Tamil Nadu's list keeps it | *Dharmasindhu* (C1406) |
| Sri Rama Navami, Vinayaka Chavithi | `RAMA_NAVAMI`, `GANESHA_CHATURTHI` | Telangana's rules: Chaitra's bright 9th at sunrise, Bhadrapada's bright 4th at midday | the G.O.s; Telangana's (C1389) |
| Mahalaya Amavasya (optional) | `MAHALAYA_AMAVASYA` | Bhadrapada's new moon at sunrise, Telangana's Bathukamma rule | the G.O.s; Telangana's (C1395) |
| Durgastami, Vijayadasami | `MAHA_ASHTAMI`, `VIJAYA_DASHAMI` | Ashvina's bright 8th and 10th at midday, the later day only when it alone holds the tithi | the G.O.s; Telangana's (C1401) |
| Naraka Chaturdasi (optional), Deepavali | `NARAK_CHATURDASHI`, `LAKSHMI_PUJA` | Ashvina's dark 14th at the afternoon and new moon at pradosha, Telangana's rules | the G.O.s; Telangana's (C1404) |
| Bhogi, Makara Sankranti, Kanuma | `BHOGI`, `UTTARAYANA_PUNYAKALA`, `KANUMU` | Dhanu's last day and Makaram's first and second days of `MALAYALAM_SOLAR`, Telangana's rules; Kanuma is a general holiday here | the G.O.s; Telangana's (C1388) |

The months are amanta. No new rule form was needed; the one new key is
`MAHALAYA_AMAVASYA`.

## 3. The sky

The lunar days are reckoned under the modern sky, the built-in ephemeris with
the default profile. Bhogi, Sankranti and Kanuma use `MALAYALAM_SOLAR`'s fixed
place on India's clock, as Telangana's do (C1373, C1388).

## 4. The days printed, not reckoned

`PublishedSet::india_andhra_pradesh` (`in-ap-gad-printed`) carries:

- **The days set by the moon**, marked `subjectTo: MOON_SIGHTING` under
  paragraph 5: Ramzan (Eid-ul-Fitr), Bakrid (Eid-ul-Azha), Moharrum and Eid
  Miladun Nabi, as `EID_AL_FITR`, `EID_AL_ADHA`, `MUHARRAM` and `MAWLID`, in all
  seven years; and 2021's Shab-e-Barath (`SHAB_E_BARAT`), a general holiday with
  Holi that year, which 2021's para 5 names.
- **Annexure-II's Islamic days**, unmarked, as printed, since no para 5 names
  them (C1403): Hazrath Ali's birthday, Shab-e-Meraj, Shab-e-Barath, Shahadat
  Hazrat Ali, Juma-atul-Vida, Shab-e-Qadar, Eid-e-Ghadeer, 9th Muharram, Arbaeen,
  Yaz-Dahum-Shareef and the birthday of Hazrath Syed Mohammed Juvanpuri Mehdi,
  under Telangana's keys. 2023's Shab-e-Meraj is a general holiday, printed with
  Maha Sivaratri.
- **Varalakshmi Vratham** (`VARAMAHALAKSHMI_VRATA`), the Friday before
  Shravana's full moon as on Telangana's list (C1391): optional to 2024 and a
  general holiday from 2025, each year's entry with its own standing.
- **The Parsi New Year** (`PARSI_NEW_YEAR`), optional: the catalogue has no
  Shahenshahi calendar (C1243).
- **2023's second Vijayadasami** (`VIJAYA_DASHAMI_NEXT_DAY`), Annexure-II's
  "Vijayadasami (Tidhi Dwayam)" of 24 October (C1401).

## 5. Against the print

At Amaravati, 190 of the 198 rows meet the print (`festival-measured.md` §18,
and per year in `crates/sdk/tests/andhra_pradesh.rs`). The rows are the reckoned
and dated days, general and optional, and the nine fixed days, which stand in
every year. Eight rows part, each named with its cause. No rule was moved to
meet them:

| year | met / printed |
|---|---|
| 2020 | 27 / 28 |
| 2021 | 28 / 29 |
| 2022 | 27 / 29 |
| 2023 | 26 / 28 |
| 2024 | 27 / 28 |
| 2025 | 27 / 28 |
| 2026 | 28 / 28 |

- **Durgastami, 2020 and 2024; Vijayadasami, 2023** (C1401). In 2020 the bright
  8th ran from 06:57 on 23 October to 06:59 on the 24th, and the G.O. lists the
  24th, as Telangana's does (C1393). In 2024 it ran from 12:32 on 10 October to
  12:07 on the 11th; the midday reading takes the 10th, which Telangana's G.O.
  lists, and this one lists the 11th, whose sunrise the 8th held. In 2023 the
  10th ran from 17:45 on 23 October to 15:14 on the 24th; the rule takes the
  24th, and the G.O. lists the 23rd, the day the 10th begins in, with the 24th
  as an optional "Vijayadasami (Tidhi Dwayam)".
- **Basava Jayanthi, 2021** (C1407). The bright 3rd began at 05:39 on 14 May, a
  minute and a half after Amaravati's sunrise and five minutes before
  Hyderabad's, so the print's rule takes the 15th here and the 14th at
  Hyderabad; the G.O. lists the 14th. It is the one row the place moves.
- **Sri Krishna Astami, 2022 and 2025** (C1406). The dark 8th ran from 21:21 on
  18 August 2022 to 22:59 on the 19th and from 23:50 on 15 August 2025 to 21:35
  on the 16th; *Dharmasindhu*'s midnight rule takes the 18th and the 15th, and
  the G.O. lists the 19th and the 16th, whose sunrise the 8th held. In 2020 and
  2023 the G.O. lists the midnight's day, so the sunrise reading parts there.
- **Naraka Chaturdasi, 2022** (C1404). The dark 14th ran from 18:03 on
  23 October to 17:27 on the 24th; the afternoon rule takes the 24th, as
  Telangana's G.O. lists it, and this one lists the 23rd.
- **Holi, 2023** (C1405). The full moon ran from 16:17 on 6 March to 18:10 on
  the 7th, so the Holika fire is the 6th and Holi the 7th, as Telangana's G.O.
  lists it; this one lists the 8th.

## 6. What is not here

- **2019** (C1402): G.O.Rt.No.2413 of 14 November 2018 was not found in a
  readable copy.
- **Andhra Pradesh Formation Day** (C1408). Neither 1 November nor 2 June is
  among the general or optional holidays of any G.O. of 2020 to 2026. The Labour
  Department's lists for shops and establishments declare such days, and those
  lists are not this pack's.
- **The Negotiable Instruments Act list** (G.O.Rt.No.1998 of 2021, 2277 of 2025
  and their kin), the banks' holidays.
- **The weekend**: the Sundays and second Saturdays themselves. Annexure-I(A)'s
  and II(A)'s holidays that fall on one ship as days.
- **One-off days**, as an election day or a holiday a later order moves
  (G.O.Rt.No.612 of 2026, C1402).
