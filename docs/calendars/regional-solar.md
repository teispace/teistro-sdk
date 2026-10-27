# The regional solar calendars: Tamil, Bengal, Odia and Malayalam

Status: `built`, 2026-10-27. `Calendar::TamilSolar`, `BengalSolar`,
`OdiaSolar`, `MalayalamSolar` (`calendar.TAMIL_SOLAR`, `BENGAL_SOLAR`,
`ODIA_SOLAR`, `MALAYALAM_SOLAR`), `crates/calendar/src/regional_solar/`.
P2-1 step 2. Assam's calendar waits on a crux (C765); Punjab's is not
built (below).

The solar calendars of Tamil Nadu, West Bengal, Odisha and Kerala begin
each month at a sankranti, the Sun's entry into a sidereal sign. Bikram
Sambat does the same. What differs between them is the civil day the
sankranti is placed on, the place that day is reckoned at, the sign the
year opens with and the era it counts. Sewell and Dikshit (*The Indian
Calendar*, 1896, art. 28, p. 12) give the rules, built as
`solar::MonthStartRule`:

| rule | Sewell and Dikshit, art. 28 |
|---|---|
| `FollowingDay` | Bengal: a sankranti between sunrise and midnight begins the month the next day, after midnight the day after that, which is the civil day after the one it falls in |
| `SunriseToSunrise` | Orissa: "the same day as the sankranti, whether this takes place before midnight or not", with "the week-day … counted from sunrise to sunrise" (n. 1) |
| `BeforeSunset` | Tamil: after sunrise and before sunset the same day, after sunset the next |
| `BeforeAparahna` | Malabar: within the first three of the daylight's five parts the same day, otherwise the next |

`SankrantiDay`, the civil day from midnight, is not the Orissa rule,
though an earlier version of this page and of the type's documentation
called it so: Sewell's "same day" is the sunrise-to-sunrise day, and the
measurement below separates the two (117 against 96).

## The definitions

Each calendar is the solar engine (`bikram_sambat::Engine`) with a
`YearStart` (the year's first sign and the offset from the Gregorian year
it opens in), a place, a rule, and Indian Standard Time (+05:30)
throughout, as the *Rashtriya Panchang* prints. The sankrantis are the
modern Sun's (drik) under Lahiri's ayanamsha, which is the *Rashtriya
Panchang*'s "increasing ayanamsa … 23° 15' 00" for 0h on 21st March, 1956"
(1945 SE edition, pp. vi–vii), on the **true basis**: the ayanamsha
carries the nutation in longitude, so the sidereal Sun is the apparent
one less the nutated ayanamsha (`DrikSun::with_ayanamsha_basis`). That
is how the book's transits are reckoned: on the true basis the SDK's 109
sankrantis of the 2018–19 to 2026–27 appendices fall within 32 seconds
of the printed minute, every one; on the mean basis, the SDK's default
elsewhere, they run from 4.6 minutes early to 7.7 late, swinging with the
nutation, and Bengal's Ashadha 2020 (23:54 IST, eight minutes before
midnight) lands on the wrong day. Sunrise and sunset are the Sun's upper
limb with standard refraction.

| calendar | rule | place | year opens | era | era year |
|---|---|---|---|---|---|
| `TAMIL_SOLAR` | `BEFORE_SUNSET` | Chennai, 13.0827°N 80.2707°E | Mesha (Chittirai) | `SHAKA` | CE − 78 |
| `BENGAL_SOLAR` | `FOLLOWING_DAY` | Kolkata, 22.5726°N 88.3639°E | Mesha (Vaisakha) | `BENGALI` | CE − 593 |
| `ODIA_SOLAR` | `SUNRISE_TO_SUNRISE` | Bhubaneswar, 20.2961°N 85.8245°E | Mesha | `SHAKA` | CE − 78 |
| `MALAYALAM_SOLAR` | `BEFORE_APARAHNA` | Thiruvananthapuram, 8.5241°N 76.9366°E | Simha (Chingam) | `KOLLAM` | CE − 824 |

The place is part of the definition: a rule that reads a sunrise or a
sunset reads it there. The Bengal rule reads none, so Kolkata names the
calendar and moves no day.

Month 1 is the month of the year's first sign, and every month is named
by the sign entered (`RegionalSolar::sign`, `Region::sign_of_month`): the
regions' own names (Chittirai, Vaisakha, Chingam and the rest) are labels
of those signs. No locale names them yet: no vetted Nepali source for the
four calendars' month names has been read, and a strict locale binds the
English to one, so a date renders its month by number and the calendars
join the other calendar systems in `xtask/src/intl.rs`'s unnamed list.

**The eras.** Bengal counts the Bengali San: Sewell and Dikshit (art. 71,
p. 43) make Bengali San 1300 the year of A.D. 1893–94 and say it "runs
with the solar Saka year, beginning at the Mesha sankranti"; the *Indian
Astronomical Ephemeris* for 2018 has Bengali San 1425 begin on 15 April
2018. Both give CE − 593, as Bangladesh's revised calendar counts
(`calendars/bangladeshi.md`). Kerala counts the Kollam era: "A.D. 1894–95
being called Kollam 1070", the year beginning in South Malabar "with the
solar month Chingam (Simha)" (art. 71, p. 45), so Kollam = CE − 824 from
Chingam (not − 825, which is the epoch year). Tamil Nadu and Odisha count
the solar Saka year, as elapsed years like the Indian national calendar:
Sewell and Dikshit read a Tamil solar panchang for "Saka 1815 current"
(art. 28, n. 3). Odisha's Amli and Vilayati eras (art. 71) open the year
elsewhere (lunar Bhadrapada sukla 12th and the Kanya sankranti) and are
not counted; the Tamil 60-year cycle is not a date's era.

**The tables.** Each calendar ships a table of month lengths for every
year opening from 1800 to 2398, computed by the engine over the SDK's
built-in ephemeris (standard tier) and written by `cargo xtask gen
calendars`; `check-calendars` regenerates it and fails on any difference.
So a date needs no ephemeris and no place at run time
(`needs_ephemeris: false`), and every date's `resolution` is
`Computed { model }`, the model naming the ephemeris, the ayanamsha, the
sunrise, the clock, the place and the rule. The years have 365 or 366
days and the months 29 to 32.

## What was measured

**The list.** The *Rashtriya Panchang* (Positional Astronomy Centre,
India Meteorological Department, Kolkata), English edition, prints a
"Regional Calendar — Beginning of months of different Regional Calendars"
table in four solar columns: *Bengal & Assam*, *Punjab, Haryana etc. and
Odisha*, *Tamil Nadu* and *Kerala*, and an appendix of the Sun's
nirayana transits to the minute in IST. Ten editions were read: Saka 1914
(1992–93) and 1940 to 1948 (2018–19 to 2026–27), 120 month beginnings per
column. Every printed national (Saka) date was checked against its printed
Gregorian date, 480 of 480. The transcription is kept outside the
repository and is cited, not copied; the tests assert individual printed
days as facts.

**The fits.** Each column against each rule, the sankrantis computed as
above:

| column | rule shipped | months matched |
|---|---|---:|
| Tamil Nadu | before sunset, Chennai | 120/120 |
| Bengal | the civil day after | 119/120 |
| Odisha | sunrise to sunrise, Bhubaneswar | 117/120 |
| Kerala | before three fifths of the daylight, Thiruvananthapuram | 109/110 outside Mesha, 4/10 at Mesha |

The Odisha column under `SANKRANTI_DAY`, the civil day, matches 96/120.

**The misses, with their causes.**

- *Bengal, Vrisha 2021.* The sankranti fell at 23:25 IST on 14 May 2021;
  the rule gives 15 May and the column prints 16 May. The column is
  Bengal's and Assam's together, and the day after the day after is what
  Assam's rule gives a sankranti that late (C765), so the print most
  likely carries Assam's date here without its footnote.
- *Odisha, three months of the Saka 1941 edition* (2019–20): Kartika
  (18 October 2019), Agrahayana (17 November 2019) and Magha (15 January
  2020). Their sankrantis fell between midnight and dawn, and the edition
  prints them on the civil day, where every other edition, before and
  after, prints a sankranti before dawn on the day before. The edition
  contradicts itself; the SDK follows the other nine.
- *Kerala, Simha 2023.* The sankranti fell at 13:32 IST on 17 August
  2023 (the book prints 13:33), a minute and a half before the
  three-fifths mark at Thiruvananthapuram; the rule gives 17 August and
  the column prints the 18th. A boundary case inside the spread of the
  sunrise conventions and the places a column could be reckoned for.
- *Kerala, Mesha (Vishu).* The column's 2018 and later editions fit the
  sunrise-to-sunrise day for Mesha alone: sankrantis at 20:23 (2020) and
  21:04 (2024) printed the same day, at 02:33 (2021) and 03:21 (2025) the
  day before, and at 14:09 (2019) and 14:59 (2023), after the
  three-fifths mark, the same day; six of the ten Mesha rows miss the
  Malabar rule. The 1992–93 edition and Kerala's own observance of Vishu as
  reported by third parties fit the Malabar rule. Sewell and Dikshit state
  one rule for every sign. The SDK ships the Malabar rule for Mesha too
  and records the *Rashtriya Panchang*'s Mesha rows as a measured
  divergence until a printed Kerala almanac settles it (C764).

**An earlier list, rejected.** Before the *Rashtriya Panchang* was read,
Prokerala's Tamil calendar for Chennai (108 months, 2020–2028) was set
against the Lahiri and the Surya Siddhanta sankrantis under every rule.
No rule and no uniform shift fitted past 92 (Lahiri) or 110 (Siddhanta),
and its misses could not be ordered by any cutoff: it is a third
reckoning, most likely vakya, and not a drik corpus.

## Not built

- **Assam.** Assam shares Bengal's column, and the column footnotes
  where Assam differs. The cases point to a local-midnight rule, the
  Bengal rule with midnight reckoned at Guwahati rather than on IST:
  Ashadha 2020 (sankranti 23:53 IST on 14 June; Assam printed 16 June,
  Bengal 15 June) and Vrisha 2021 (23:25 IST on 14 May; the column
  prints 16 May, C765's reading). But Shravana 2026 (23:39 IST on
  16 July) carries no Assam footnote. Two cases of three is not a rule;
  the calendar waits on C765, and needs a local-midnight variant of
  `FollowingDay` when built.
- **Punjab.** The *Rashtriya Panchang* prints Punjab and Haryana in one
  column with Odisha, and that column is reckoned at Bhubaneswar (its
  sunrise decides the pre-dawn sankrantis), not at a place in Punjab. A
  Punjab calendar would be Odisha's under another name, or a guess at a
  Punjab place no source names, so none is shipped.

## Sources

- Robert Sewell and Sankara Balkrishna Dikshit, *The Indian Calendar*
  (London, 1896), art. 28 (pp. 12–13) and art. 71 (pp. 43–45); rank 2.
- *Rashtriya Panchang*, English edition, Positional Astronomy Centre,
  India Meteorological Department, Kolkata: Saka 1914 (archive.org
  `ijpk-rashtriya-panchang-saka-era-1914-1992-1993-ad-engl`) and Saka 1940
  to 1948 (packolkata.imd.gov.in, `EnglishRP1819.zip` to
  `EnglishRP2627.zip`; 1943 SE through archive.org `rashtriya-panchang`);
  rank 1 for the month beginnings it prints.
- *Indian Astronomical Ephemeris* 2018 (Positional Astronomy Centre), the
  beginning of Bengali San 1425.
