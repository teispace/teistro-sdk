# Kerala's public holidays

Status: `built`, 2026-11-10. SR-03, the third state set, after Maharashtra's
(`maharashtra-holidays.md`) and West Bengal's (`west-bengal-holidays.md`). Cruxes
C1301 and C1304 (closed here) and C1320 onward
(`01-research/feature-universe/19-verification-cruxes.md`). Measured in
`festival-measured.md` §12.

## 1. The source

The General Administration (Co-ordination) Department issues a Government Order each
year, published in the Kerala Gazette Extraordinary. It lists:

- **the public holidays** to the public offices, under the Negotiable Instruments
  Act, 1881, each with its Gregorian date, its weekday and its Malayalam Era date;
- **the occasions falling on a Sunday or a second Saturday**, listed apart and not
  declared;
- **the restricted holidays**, each for the community it names: Avani Avittom for the
  Brahmins, Ayya Vaikunda Swami's birthday for the Nadars, Vishwakarma Day for the
  Vishwakarmas;
- **a moon clause**: Id-ul-Fitr, Id-ul-Ad'ha, Muharram and Milad-i-Sherif hold the day
  the moon is seen, and the order prints the expected day.

| year | order | gazette | copy read |
|---|---|---|---|
| 2021 | G.O.(P) No. 16/2020/GAD, 22 October 2020 | | a staff site's text of the signed order |
| 2022 | G.O.(P) No. 29/2021/GAD, 21 October 2021 | No. 3060 | gad.kerala.gov.in |
| 2023 | G.O.(P) No. 22/2022/GAD, 26 October 2022 | No. 3657 | a copy of the signed gazette |
| 2024 | G.O.(P) No. 24/2023/GAD, 4 October 2023 | No. 3254 | a copy of the signed gazette |
| 2025 | G.O.(P) No. 34/2024/GAD, 14 October 2024 | | a copy of the signed gazette |
| 2026 | G.O.(P) No. 14/2025/GAD, 31 October 2025 | No. 3845 | gad.kerala.gov.in |

## 2. The pack

`INDIA_KERALA` holds the list at any place, and is measured at Thiruvananthapuram on
India's clock. Every day is kept in `["IN", "KERALA"]`, `GAZETTED`; a restricted day
is `optional` and names its community.

| holiday | key | form | the day's rule is cited to |
|---|---|---|---|
| Republic Day, Independence Day, Gandhi Jayanthi, Christmas, Good Friday | as `INDIA_CENTRAL` | civil date, or Easter less two days | the notification |
| Maundy Thursday, Easter | `MAUNDY_THURSDAY`, `EASTER` | Easter less three days, Easter | the notification |
| Mannam Jayanthi, Dr. B. R. Ambedkar Jayanthi, May Day, Ayyankali Jayanthi, Vishwakarma Day (restricted) | `MANNAM_JAYANTHI`, `AMBEDKAR_JAYANTI`, `INTERNATIONAL_LABOUR_DAY`, `AYYANKALI_JAYANTHI`, `VISHWAKARMA_PUJA` | a Gregorian date | the notification |
| Sivarathri, Deepavali | `SHIVARATRI`, `LAKSHMI_PUJA` | `INDIA_CENTRAL`'s lunar rule | *Dharmasindhu*; a printed panchanga |
| Vishu | `VISHU` | the Sun's entry into Ashwini, by sunrise, one day on | the notifications (C1301) |
| First Onam (Uthradom), Thiruvonam, Third Onam, Fourth Onam | `FIRST_ONAM`, `THIRUVONAM`, `THIRD_ONAM`, `FOURTH_ONAM` | Shravana in Chingam, the later of two; −1, 0, +1, +2 days | the notifications (C1323) |
| Sree Narayana Guru Jayanthi | `SREE_NARAYANA_GURU_JAYANTHI` | Shatabhisha in Chingam, the later of two | the notifications (C1323) |
| Sreekrishna Jayanthi (Ashtami Rohini) | `JANMASHTAMI` | `DHARMASINDHU`'s rule | *Dharmasindhu* (C1320) |
| Sree Narayana Guru Samadhi | `SREE_NARAYANA_GURU_SAMADHI` | Kanni 5 of `MALAYALAM_SOLAR` | the notification |
| Karkidaka Vavu, Maha Navami, Vijaya Dasami | `KARKIDAKA_VAVU`, `MAHA_NAVAMI`, `VIJAYA_DASHAMI` | Ashadha's (amanta) new moon, Ashwina's bright 9th and 10th, at sunrise | Kerala's almanacs, page not read (C1321) |
| Avani Avittom (restricted, Brahmins) | `AVANI_AVITTOM` | *Dharmasindhu*'s Taittiriya upakarma | *Dharmasindhu* |

**Two new forms.** Vishu and the starred days needed forms the rule language did not
have (`festival-rule-language.md`, the dated rules):

- `sunEnters` gains `reckoning: SUNRISE`, which keeps the Sun's entry on the day from
  one sunrise to the next. Vishu is the first civil day whose sunrise follows the Mesha
  sankranti: the entry into Ashwini by sunrise, one day on. This is Medam 1 except when
  the sankranti falls between sunrise and the three-fifths mark, as in 2022 and 2026,
  when the month begins a day later and Vishu stays on the day after the sankranti.
- `nakshatraInMonth` keeps the day a nakshatra holds in a calendar's month, with
  `twice` naming which of two occurrences in one month keeps it and `days` counting
  civil days from it. The Onam days are Thiruvonam's day less one and plus one and two:
  a `FollowingRule` counts only from a karmakala rule, and a dated day is not one.

**Kanni 5.** Sree Narayana Guru's Samadhi is a day of a solar month alone, which a
`civilDate` on `MALAYALAM_SOLAR` already keeps: 21 September, or 22 September in 2023,
when Kanni began on the 18th.

**Sreekrishna Jayanthi is the ashtami's day.** The notices name it Ashtami Rohini, and a
rule on Rohini in Chingam meets 2 of the 6 years: the printed day is the one the dark
8th holds at midnight, Rohini or not (2022's Rohini began on the 20th, the print keeps
the 18th). *Dharmasindhu*'s Janmashtami (`JANMASHTAMI`, as West Bengal's list keeps
it) is shipped under its own key and parts only in 2025, when the month's 8th fell in
Karkidakam and the notice kept the next one, in Chingam (C1320).

**Deepavali** is the central list's `LAKSHMI_PUJA`, the key `INDIA_CENTRAL` keeps
Diwali under; Kerala's list prints it on the central list's day in all six years.

## 3. The days printed, not reckoned

`PublishedSet::india_kerala` carries, for each year a notification is in hand:

- **The days set by the moon**: Id-ul-Fitr, Id-ul-Ad'ha (Bakrid), Muharram and
  Milad-i-Sherif, as `EID_AL_FITR`, `EID_AL_ADHA`, `MUHARRAM` and `MAWLID`, each marked
  `subjectTo: MOON_SIGHTING`, on the day the order prints. A day a later order moved is
  not shipped as moved: the orders read are the yearly lists.
- **Ayya Vaikunda Swami's birthday** (restricted, Nadars), printed on 12 March to 2024
  and on 4 March from 2025, unmarked: the change has no rule the lists give.

## 4. Against the print

At Thiruvananthapuram over the built-in ephemeris, 100 of the 102 rows beside the nine
every-year dates meet the print (`festival-measured.md` §12). The 102 are the lunar,
solar and starred days and the Christian days counted from Easter, 17 a year, with the
Sunday and second-Saturday rows among them; the nine fixed dates stand in all six years.
Two part, and each is named with its cause; no rule was moved to meet them:

- **Karkidaka Vavu, 2024** (C1322): the new moon began at 15:51 on 3 August; the sunrise
  reading takes the 4th, the notice lists the 3rd.
- **Sreekrishna Jayanthi, 2025** (C1320): the notice keeps the dark 8th in Chingam, 14
  September; *Dharmasindhu*'s rule keeps August's, which fell in Karkidakam.

Vishu meets all six years, 2022 and 2026 among them, when it is Medam 2. The four Onam
days and Sree Narayana Guru Jayanthi meet all six, Onam 2024 on Chingam 30, the later of
Chingam 1200's two Thiruvonams (C1323), and Fourth Onam 2024 on Kanni 1.

## 5. What is not here

- **A Kerala almanac's page** (C1321). The lunar days and the convention for the
  star's day are held to the notifications; no panchangam's statement of either was
  read.
- **The orders that moved a day after its list**, as a holiday moved for an Id when
  the moon was seen a day later. Each year ships its list as printed.
- **Local holidays**, which District Collectors declare.
- **Tamil Nadu**, the fourth state set (`tamil-nadu-holidays.md`).
