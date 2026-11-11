# Tamil Nadu's public holidays

Status: `built`, 2026-11-11. SR-03, the fourth state set, after Maharashtra's
(`maharashtra-holidays.md`), West Bengal's (`west-bengal-holidays.md`) and Kerala's
(`kerala-holidays.md`). Cruxes C1350 to C1358
(`01-research/feature-universe/19-verification-cruxes.md`). Measured in
`festival-measured.md` §13, under the vakya sky and the modern.

## 1. The source

The Public (Miscellaneous) Department issues a Government Order each year. It
declares the public holidays under s. 25 of the Negotiable Instruments Act, 1881, in
a notification appended to the order and published in the Tamil Nadu Government
Gazette. It closes the state's offices on those days (all but the banks' annual
closing of accounts) and on every Saturday and Sunday. Each row prints a Gregorian
date and its weekday; the order names no Tamil date and has no clause on the moon.

| year | order | copy read | rank |
|---|---|---|---|
| 2019 | G.O.Ms. No. 840, 29 October 2018 | a staff site's copy with its number and table (it prints "15.18.2019" for Independence Day) | 1 as copied |
| 2020 | G.O.Ms. No. 762, 22 October 2019 | a staff site's copy with its number and table | 1 as copied |
| 2021 | G.O.Ms. No. 554, 27 October 2020 | a staff site's copy with its number and table; Thai Poosam, 28 January, was added by a later order of January 2021, its number not found | 1 as copied |
| 2022 | G.O.(Ms.) No. 935, 2 November 2021, Gazette Extraordinary Part II(1) No. 525 | a list whose first seven rows match the gazette's snippet; the gazette refused a scripted fetch | 3 (C1354) |
| 2023 | G.O.(Ms.) No. 682, 11 October 2022, amended by G.O.(Ms.) No. 528, 31 August 2023 | a staff site's copy; the amendment from the press | 1 as copied |
| 2024 | G.O.(Ms.) No. 692, 9 November 2023 | a staff site's copy with its number and table | 1 as copied |
| 2025 | G.O.(Ms.) No. 792, 22 November 2024 | the signed order and its appendix (a bank's copy), read again for this page | 1 |
| 2026 | G.O.(Ms.) No. 708, 11 November 2025 | the signed order and its appendix (a scan) | 1 |

G.O.(Ms.) No. 528 of 2023 moved Vinayakar Chathurthi from 17 to 18 September at the
request of the Commissioner of Hindu Religious and Charitable Endowments. The pack
is held to the 18th, the day declared. A 2022 order made 17 January a holiday in
lieu of 29 January; that is a one-year substitution and is not shipped.

## 2. The pack

`INDIA_TAMIL_NADU` holds the list at any place. It is measured at Chennai on India's
clock. Every day is kept in `["IN", "IN-TN"]` (ISO 3166-2; `TAMIL_NADU`, its first key, is read as an alias, SR-89) and is `GAZETTED`. The banks'
closing of accounts names the community `BANKS`.

| holiday | key | form | the day's rule is cited to |
|---|---|---|---|
| Republic Day, Independence Day, Gandhi Jayanthi, Christmas, Good Friday | as `INDIA_CENTRAL` | civil date, or Easter less two days | the order |
| New Year's Day, May Day, Dr. B. R. Ambedkar's birthday, annual closing of accounts (banks) | `NEW_YEAR_S_DAY_AD`, `INTERNATIONAL_LABOUR_DAY`, `AMBEDKAR_JAYANTI`, `BANK_ACCOUNTS_CLOSING` | a Gregorian date | the order |
| Pongal, Thiruvalluvar Day, Uzhavar Thirunal | `THAI_PONGAL`, `THIRUVALLUVAR_DAY`, `UZHAVAR_THIRUNAL` | Thai 1, 2 and 3 of `TAMIL_VAKYA` | the order (C1351) |
| Tamil New Year's Day | `PUTHANDU` | Chithirai 1 of `TAMIL_VAKYA` | the order (C1351) |
| Thai Poosam | `THAI_POOSAM` | Pushya in Thai of `TAMIL_VAKYA`, the later of two | the order (C1352) |
| Mahaveer Jayanthi | `MAHAVIR_JAYANTI` | `INDIA_CENTRAL`'s lunar rule | a printed panchanga |
| Krishna Jayanthi | `JANMASHTAMI` | `DHARMASINDHU`'s rule | *Dharmasindhu* (C1355) |
| Telugu New Year's Day, Vinayakar Chathurthi, Deepavali | `SAMVATSARA_ARAMBHA`, `GANESHA_CHATURTHI`, `NARAK_CHATURDASHI` | `NEPAL_PANCHANGA`'s rule | a printed panchanga (C1356) |
| Vijaya Dasami | `VIJAYA_DASHAMI` | Ashwina's bright 10th at sunrise | Tamil Nadu's vakya almanacs, page not read (C1350, C1357) |
| Ayutha Pooja | `AYUDHA_PUJA` | the day before Vijaya Dasami | the orders (C1353) |

**No new form.** Every day is a form the rule language has:

- **The Tamil months are the vakya school's.** Thai 1 to 3 are `civilDate`s on
  `TAMIL_VAKYA`, the calendar the vakya almanacs begin their months by
  (`calendars/regional-solar.md`). The drik `TAMIL_SOLAR` parts once in the eight
  years: in 2026 the drik Makara sankranti fell on the afternoon of 14 January, before Chennai's sunset,
  so the drik Thai began on the 14th; the vakya Thai began on the 15th, the day the
  order prints (C1351). Chithirai 1 is 14 April in all eight years under both schools,
  so these years do not tell them apart there.
- **Thai Poosam is `nakshatraInMonth`**, the form Kerala's Onam brought (C1323):
  Pushya in Thai, its day the first whose sunrise the star holds, the later of two in
  one month. Pushya is the full moon's star while the Sun is in Makara. When Thai holds
  it twice, the first is in Thai's opening days, joined to Margazhi's full moon, and the
  second is joined to Thai's own; in 2025 the order kept the second, 11 February
  (Thai 29), where the first fell on 15 January (C1352).
- **Ayutha Pooja is a negative `FollowingRule`,** the count back West Bengal's Durga
  Puja brought: the day before `VIJAYA_DASHAMI`. The day before Vijaya Dasami meets
  7 of 8 years under the vakya sky. No navami rule does better: the navami at sunrise
  meets 6 (C1353).
- **Deepavali is the Naraka Chaturdashi,** as the central memorandum's paragraph 6
  allows a state to keep it, at sunrise (`NEPAL_PANCHANGA`'s `NARAK_CHATURDASHI`). The
  central list's `LAKSHMI_PUJA` is not in this pack (C1356).

## 3. The vakya sky (TN-a)

The state keeps its solar days by the vakya almanac, and its lunar days were
suspected to follow the same almanac: the drik chaturthi of 2023 could not give the
18 September that G.O.(Ms.) No. 528 declared on the Endowments Commissioner's word.
The SDK now reckons the vakya almanac's tithi and nakshatra (`calendars/vakya.md`),
so the question was measured rather than argued. Each lunar row of the eight years
was reckoned by the same rules under the `tamil-vakya` profile and under the modern
sky (the built-in ephemeris, the default profile):

| holiday | vakya | modern | where the modern sky parts |
|---|---|---|---|
| Telugu New Year | 8/8 | 8/8 | |
| Mahaveer Jayanthi | 8/8 | 8/8 | |
| Krishna Jayanthi | 8/8 | 6/8 | 2022, 2025 (C1355) |
| Vinayakar Chathurthi | 8/8 | 7/8 | 2023 (C1296) |
| Ayutha Pooja | 7/8 | 6/8 | 2024, 2026 |
| Vijaya Dasami | 7/8 | 6/8 | 2024, 2026 |
| Deepavali | 7/8 | 8/8 | |
| Thai Poosam | 5/6 | 5/6 | 2024, both skies |

**Decided as measured: the pack ships for the `tamil-vakya` profile** (C1350). Under
it 98 of the 102 rows beside the eight every-year dates meet the print; under the
modern sky 94 do. The vakya sky meets five rows the modern misses (Krishna Jayanthi
2022 and 2025, Vinayakar Chathurthi 2023, Vijaya Dasami and Ayutha Pooja 2024) and
misses one the modern meets (Deepavali 2021). The two Krishna Jayanthis turn on whether the ashtami holds the earlier
night's midnight: the drik ashtami began at 21:21 on 18 August 2022 and at 23:50 on
15 August 2025, the vakya ashtami after midnight (01:40 and 01:28), so *Dharmasindhu*'s
rule takes the later day under the vakya sky, the day printed. The hypothesis the
research note proposed for these two years (the earlier day only when the ashtami
stands at its sunset) is not needed and is not shipped.

The rules are the same under either sky: a pack does not name a profile. A consumer
reckoning Tamil Nadu's holidays opens its context with `profile("tamil-vakya")` over
`[SuryaSiddhanta, Builtin]`, as `crates/sdk/tests/tamil_nadu.rs` does. Under another
profile the pack still answers, by that sky, and §13 shows what it gives under the
modern one.

## 4. The days printed, not reckoned

`PublishedSet::india_tamil_nadu` carries, for each year an order is in hand, Ramzan
(Idu'l Fitr), Bakrid (Idul Azha), Muharram and Milad-un-Nabi, as `EID_AL_FITR`,
`EID_AL_ADHA`, `MUHARRAM` and `MAWLID`, on the day the order prints. The order has no
moon clause, but each day is the moon's, so each is marked
`subjectTo: MOON_SIGHTING`, as the central list's are (C1358). The 2026 order prints
Bakrid on 28 May, the day the central list later declared.

## 5. Against the print

At Chennai under the `tamil-vakya` profile, 98 of the 102 reckoned rows meet the
print (`festival-measured.md` §13, 162 of 166 with the fixed dates, and per year in
`crates/sdk/tests/tamil_nadu.rs`).
The 102 are 13 rows a year (12 in 2019 and 2020, before Thai Poosam was listed); the
eight fixed dates stand in all eight years. Four part, and each is named with its
cause; no rule was moved to meet them:

- **Deepavali, 2021** (C1356): the vakya chaturdashi ran from 07:26 on 3 November to
  05:41 on the 4th and held neither sunrise, so the sunrise reading takes the 3rd. The
  order lists the 4th. The chaturdashi held that morning's arunodaya, the four ghatis before
  sunrise in which the Naraka Chaturdashi bath is taken; the rule language
  has no arunodaya window, and the modern chaturdashi, which ended at the 4th's
  sunrise, meets the print.
- **Thai Poosam, 2024** (C1352): Pushya ran from 09:09 on 25 January to 11:06 on the
  26th, so its first sunrise is the 26th's. The order lists the 25th, the full moon's
  day, whose daylight the star held the most of. The modern sky parts the same way.
- **Vijaya Dasami, 2026** (C1357): the vakya dashami ran from 10:37 on 20 October to
  11:47 on the 21st, so the sunrise reading takes the 21st. The order lists the 20th,
  whose aparahna the dashami holds. *Dharmasindhu*'s aparahna rule gives the 20th but
  parts in 2020, 2022 and 2023, so it is not shipped.
- **Ayutha Pooja, 2026** (C1353): counted from that Vijaya Dasami, the 20th; the order
  lists the 19th.

## 6. What is not here

- **A vakya almanac's page for the lunar days** (C1350). The rules are held to the
  orders; no Tamil almanac's statement of a festival's window was read. The
  vakya tithi itself is held to the *Manonmani Vilasam* panchangam
  (`calendars/vakya.md`).
- **An arunodaya window** (C1356), which Deepavali 2021 asks for.
- **The orders that changed a day after the list,** but G.O.(Ms.) No. 528 of 2023,
  whose day the reckoned row is held to; the holiday in lieu of 2022 is not shipped.
- **Thai Poosam before 2021.** The rule gives a day in every year; the orders list it
  from 2021.
- **Local holidays**, which District Collectors declare.
