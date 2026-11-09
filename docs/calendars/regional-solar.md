# The regional solar calendars: Tamil, Bengal, Odia and Malayalam

Status: `built`, 2026-10-27. `Calendar::TamilSolar`, `BengalSolar`,
`OdiaSolar`, `MalayalamSolar` (`calendar.TAMIL_SOLAR`, `BENGAL_SOLAR`,
`ODIA_SOLAR`, `MALAYALAM_SOLAR`), `crates/calendar/src/regional_solar/`.
P2-1 step 2. Assam's calendar waits on a crux (C765); Punjab's is not
built (below). The Tamil vakya school, `Calendar::TamilVakya`
(`calendar.TAMIL_VAKYA`), was added 2026-11-03 for consumer request A2-9
([The two Tamil schools](#the-two-tamil-schools)).

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
reckoning, most likely vakya, and not a drik corpus. It has not been set
against the vakya calendar below.

## The two Tamil schools

Tamil almanacs come in two schools. The **thirukanitha** (drik) almanacs
reckon the modern Sun, as `TAMIL_SOLAR` does. The **vakya** almanacs
reckon the Sun of the *Vakyakarana*, a Kanchi manual of about 1300 in the
Aryabhata school, whose tables the temples and most household almanacs
still compute from. Both put the month on the sankranti's day by the same
rule (after sunrise and before sunset that day, otherwise the next), but
the two Suns enter a sign up to several hours apart, so the schools
disagree on the day a month starts. Over the Tamil years opening 1900 to
2100 the vakya month begins a day after the drik month in 450 of the
2,412 months, and a day before it in 74, never further apart (a test
holds both counts). The vakya year is about 3⅓ minutes longer than the
sidereal year, so its months drift later against the modern Sun, about a
day in four centuries: in the 19th century the vakya month is more often
the earlier, and after 2280 it begins two days after the drik month in 99
months (C1206). The vakya calendar ships as `calendar.TAMIL_VAKYA`
(id 14), a separate catalogue member rather than a knob on `TAMIL_SOLAR`,
since a date names its calendar and the two schools' dates must not
compare equal (C1200).

The vakya almanacs' tithi and nakshatra come from the same text's Moon,
with the almanac makers' correction to it; that Moon, its measurement
against the print and the `tamil-vakya` profile are in
[`vakya.md`](vakya.md).

**The arithmetic** (`teistro_siddhanta::vakya`), from the edition of
Kuppanna Sastri and Sarma (1962):

- *The year* (I.2–3). Kali years elapsed are the Saka years elapsed plus
  3179. The Kali days to the Sun's entry into Mesha are 365 *y* + *y*/4 +
  (5*y* − 1237)/576, counted from the mean sunrise at the Ujjain meridian
  that began Kali day 1, a Friday. The year is Aryabhata's, 365 days
  15 nadis 31¼ vinadis; the 1237/576 days are the equation of centre at
  Mesha, a constant because the school holds the apsis fixed. The
  arithmetic is exact in quarter-vinadis (1/14,400 of a day). Example 1
  of the edition (Saka 1869: 18,43,823 days 40 nadis 19 vinadis, a night
  sankranti, so the year began on Monday, 14 April 1947) is a doctest.
- *The months* (I.4). Each later sankranti is the Mesha sankranti plus a
  vakya: 30-55-32, 62-19-44, 93-56-22 days, nadis and vinadis, on to
  334-55-10 for Mina, and the year 365-15-31. Aryabhata's rule (the
  true Sun from a fixed-apsis equation) regenerates the vakyas only to
  ±35 vinadis, so they are a table of their own and ship as printed
  (C1204).
- *The daytime* (III.1, III.7–8). The cara is 20, 16 and 6⅔ vinadis an
  angula of equinoctial shadow for the first, second and third sign of
  the sayana Sun's bhuja, negative in the northern half of the year; the
  daytime is thirty nadis less the cara. The sayana Sun is the vakya Sun
  plus the precession of III.1, (Kali years − 3600) less a 121st of
  itself, over sixty, in degrees.

**The rule** (`regional_solar::vakya::month_start`). A month begins on
the Kali day that holds its sankranti when the sankranti's time after
that day's sunrise is less than the day's daytime, and on the next day
otherwise. The time is the *Vakyakarana*'s fraction of its day read as
nadis after the local sunrise: the edition's introduction notes the
vakya almanacs apply none of the corrections for the place's longitude
(desantara), half the cara (carardha) or the equation of time
(bhujantara) to the Sun (C1207). The daytime is the almanac's own
(thirty nadis less the vakya cara) for a shadow of 2¼ angulas, with the
Sun at the sign's first point (C1201, C1205). A fixed thirty-nadi cutoff
is not the rule: Kumbha in Kali 5101 fell 29 nadis 41 vinadis after the
sunrise of 12 February 2001, under thirty nadis but past that day's
printed daytime of 29 nadis 7 vinadis, and Masi is printed from the 13th
(C1202). The day count maps to the SDK's fixed days by Kali day 1,843,824
= 13 April 1947.

The calendar keeps `TAMIL_SOLAR`'s year (Chittirai first, Saka years
elapsed, era `SHAKA`) and span (years opening 1800 to 2398, though the
arithmetic needs no ephemeris and has no limit of its own, C1206). Its
table is computed at first use rather than generated, since it is a few
thousand integer steps; its dates' frame names the *Vakyakarana*, the
unreflected precession, the rule and the shadow.

### The vakya almanacs measured

The *Manonmani Vilasam* suddha vakya almanac is on archive.org in two
bound runs of annual editions, one from Sarvajit (1947–48) and one from
Isvara (1997–98). Each month's page prints the Gregorian date of every
day, from the 1950s the day's length (அகஸ், in nadis and vinadis), and
the sankranti's time in nadis after sunrise on the day it falls. The
almanac is published at Chennai (its back cover). The month beginnings
were read by eye from cropped page images, and the Koviloor vakya
almanac of 1978–79 was read the same way.

| almanac | years | months read | the vakya calendar | `TAMIL_SOLAR` |
|---|---|---:|---:|---:|
| *Manonmani Vilasam* | 1947–48 to 1961–62, and 1962–63 to Adi | 184 | 184/184 | 155/184 |
| Koviloor vakya | 1978–79 | 12 | 12/12 | 9/12 |
| *Manonmani Vilasam* | 1997–98 to 2021–22 | 300 | 300/300 | 243/300 |

Every month read agrees: 496 of 496, of which `TAMIL_SOLAR` misses 89,
so the list tells the schools apart. No printed month's sankranti falls
nearer the daytime's end than 3.6 vinadis, so the shadow is not pinned by
a boundary case (C1201). The sankranti times the
almanac prints on a month's first day agree with the *Vakyakarana*'s to
the vinadi (twelve read, 1997 to 2009: for example Kataka 1997 at 32-43,
Simha 1998 at 16-27 against 16-26.4, Tula 2007 at 5-40 against 5-39¾),
and its printed day lengths fit the vakya cara for a shadow of 2¼
angulas within two vinadis (root mean square about one vinadi) with the
precession unreflected; reflected about 24°, as III.1 goes on to say,
the fit is worse (C1203). The calendar's tests hold every
month read (`VAKYA_PRINTED`, 1947 to 1962, 1978 and 1997 to 2021, by the
Gregorian year the Tamil year opens in).

**Thirukanitha almanacs.** Two were read for comparison. The Vasan
thirukanitha almanac of 1983–84 matches both calendars, 12 of 12 (the
schools do not part that year). The *Anandha Bodhini* thirukanitha
almanac of 1978–79 matches the vakya calendar 12 of 12 and the drik one
9 of 12: it begins Adi on 17 July, Aippasi on 18 October and Karthigai on
17 November, where `TAMIL_SOLAR` gives the 16th, 17th and 16th, and it
prints its own Tula sankranti at 8 nadis 38 vinadis on 17 October yet
begins Aippasi on the 18th. That almanac computed its sky by the drik
method and still printed the vakya months, which the civil Tamil calendar
may have required then (C1208). `TAMIL_SOLAR` stays drik: the *Rashtriya
Panchang*'s Tamil Nadu column fits it 120 of 120.

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
- *Vakyakarana*, with the *Laghuprakasika* of Sundararaja, ed. T. S.
  Kuppanna Sastri and K. V. Sarma (Madras: Kuppuswami Sastri Research
  Institute, 1962), I.2–4, III.1, III.6–8, the introduction and the
  worked examples 1, 15, 16 and 19 to 21; archive.org
  `vakyakaranawithlaghuparkasikasudararajaed.kuppannasastrit.s.sarmak.v._865_t`;
  rank 1. The numbers are the text's; nothing of the translation is
  copied.
- *Manonmani Vilasam* suddha vakya panchangam (Chennai), annual editions
  bound in two runs: from Sarvajit (1947–48), archive.org
  `bqfm-manonmani-vilasa-suddha-vakya-panchangam-sarvajitt`, read to
  Subhakrit (1962–63); and Isvara (1997–98) to Plava (2021–22),
  archive.org `calw-manonmani-vilasa-suddha-vakkiya-panchangam-isvara-`;
  rank 1 for the vakya months, sankranti times and day lengths it prints.
- Koviloor vakya saiva panchangam, 1978–79 (Tamil Digital Library
  `KMM1082`); *Anandha Bodhini* thirukanitha panchangam, 1978–79
  (`KMM1084`); Vasan thirukanitha panchangam, 1983–84 (`KMM1087`).
