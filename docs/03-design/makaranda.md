# The Makaranda and the Mithila panchang

Status: `research`, 2026-11-08, for SR-16 and A2-8 (a `siddhanta`
profile that reproduces the almanacs of Mithila). Written before any
code (design page first). Section 3 is read from the sources. Section 4
is measured against 90 printed days from three years. Section 5 is the
proposed profile, and section 6 is why it is **not built yet**: the
measured agreement is ±5 to ±10 minutes, the printer's sunrise needs a
knob the SDK lacks, and two residuals have no source yet (C1223 to
C1229).

## 1. Purpose

North Bihar and Nepal's Madhesh keep their festival days by almanacs
computed in the Surya Siddhanta's tradition, through the *Makaranda*
(*Makarandasāriṇī*, Makaranda son of Ānanda, Kāśī, Śaka 1400 = 1478).
That is a book of tables for tithi, nakshatra, yoga, the mean planets,
their equations and eclipses. The most widely used printed almanac is the
*Vishwavidyalaya Panchang* of Kameshwar Singh Darbhanga Sanskrit
University, edited since 1978–79 under Pt. Ramchandra Jha. It says it
is computed from the Surya Siddhanta and the Makaranda. The consumer
wants a profile whose tithi, nakshatra and festival days are that
almanac's.

## 2. Sources

| Rank | Source | Where | Used for |
| --- | --- | --- | --- |
| 1 | *Makaranda Sāriṇī* with Viśvanātha's *Udāharaṇa*, Śrī Veṅkaṭeśvara Press, Bombay (1923 print of the 1913 edition), 91 pp. | Internet Archive `in.ernet.dli.2015.405753` (Digital Library of India). Tables are read off the page images; the OCR is unusable | the tithi tables (pp. 1–4), the Moon's equation table (p. 29) |
| 2 | K. Rupa, Padmaja Venugopal, S. Balachandra Rao, "Makaranda Sāriṇī and allied Saurapakṣa tables — a study", *IJHS* 49.2 (2014) 186–208 | insa.nic.in | the mean daily motions (Table 5.1), the tithi procedure (§§2, 6), the planets' three-step equation (§9) |
| 2 | D. Pingree, *Sanskrit Astronomical Tables in the United States* (1968), p. 43; *… in England* (1973) | cited by Rupa et al.; **not read here** | the mean motions (Rupa note 6 says they differ slightly from Table 5.1) |
| print | *Vishwavidyalaya Panchang*, KSDSU Darbhanga, years 1955–56 and 1980–81 to 2022–23 (35 scans) | Internet Archive `vishwavidyalaya-panchang` | the acceptance values: daily tithi and nakshatra ends in ghatis and palas from sunrise, day length, the Sun at *miśramāna*, daily star planets |

The almanac is copyrighted; this page keeps only the numbers it prints
(the facts measured), never its text. The days used are listed in
§7.

## 3. The Makaranda as a computation

**It is the Surya Siddhanta with a bija.** The book's first verse says
it follows the Sūryasiddhānta. Its mean daily motions (Rupa et al.,
Table 5.1, from the 1913 print) turned into revolutions per age over the
text's 1 577 917 828 civil days give the text's counts, but for:

| Body | Text (I.29 to 34) | Makaranda | Bija |
| --- | --- | --- | --- |
| Sun | 4 320 000 | 4 320 000 | 0 |
| Moon | 57 753 336 | 57 753 336 | 0 |
| Moon's apsis | 488 203 | 488 199 | −4 |
| Mars | 2 296 832 | 2 296 832 | 0 |
| Mercury's conjunction | 17 937 060 | 17 937 076 | +16 |
| Jupiter | 364 220 | 364 212 | −8 (C1221) |
| Venus's conjunction | 7 022 376 | 7 022 364 | −12 |
| Saturn | 146 568 | 146 580 | +12 |
| Moon's node | 232 238 | not settled | (C1221) |

Three things in that table are checks rather than readings:

- **The Sun is the text's.** The printed motion 0;59,8,10,10,24,12,30
  is the text's 4 320 000 × 360 / 1 577 917 828 to 10⁻¹⁴°. Rupa et
  al. turn it into 0.98560267° and so a bija of −26 civil days. That is
  an arithmetic slip: their own sexagesimals give the text's day (C1220).
- **A bija is a multiple of four.** The text's planets all stand at the
  first of Mesha when the Kali age begins (I.57). That moment is 452.75
  ages after creation, so a bija of *b* revolutions an age moves every
  epoch place by 0.75 *b* revolutions. Only a multiple of four keeps
  I.57. Every bija the table gives (−4, +16, −12, +12) is one.
- **Jupiter's printed motion has a slipped cell.** Rupa's
  0;4,59,8,48,56,31,30 gives 364 220.117 revolutions, which is not a
  whole number. The text's −8 gives 0;4,59,8,24,56,31,33: the same digits
  but one place, 48 for 24. Its bija is −8, as Rupa's revised count
  (364 212.0116) says (C1221). The node's printed motion fits no whole
  count, so it waits on the book's own Rahu page.

The Moon's apsis bija is the one the Nepal committee's almanac was
measured to carry (`Bija::NEPAL_COMMITTEE`, C28). Both traditions'
Moons are the Makaranda's.

**The tables are rules.** As far as they were checked, each table is a
rule over the text's own sky:

- **The Moon's equation (p. 29, "अथादिचन्द्रफलं")** is the text's
  equation with the epicycle 32° to 31°40′ (II.34 to 38), degree by
  degree. It gives 0°5′20″ at 1°, 2°32′2″ at 30° and 4°22′29″ at 60°,
  against exact trigonometry's 5′19.9″, 2°32′3″ and 4°22′33″; the
  difference is the text's sine table.
- **The *tithisaurabha* (pp. 3–4)** is the Moon's part of a tithi's end
  in ghatis. It has 60 columns by the *valli* (the Moon's mean anomaly in
  sixtieths of a revolution) and rows for the valli's sixtieths in steps
  of 6. It is 24|57 plus the time Δ that solves
  Δ·(v_Moon − v_Sun) + eq_Moon(anomaly + v_anom·Δ) = 0, iterated, over
  the text's mean motions. Row 0's sixteen cells (cols 0 to 15) agree with
  that rule within 0.12 ghati. The first-order rule, without the
  iteration, misses by up to 1.3 ghati (C1222).
- **The tithi procedure (p. 1, Rupa §§2, 6).** The *kaṇḍa* gives the
  mean tithi at the year's start for every 16th Śaka year from 1544, plus
  a 16-year remainder table. The *guccha* (p. 2) gives each later
  tithi's mean increment and a solar correction (*tithicālana*). The
  saurabha adds the Moon's part. Rupa's worked example (Śaka 1891) is
  the procedure end to end.

## 4. What the Mithila print computes, measured

Ninety printed days were transcribed from the page images (§7):
Jyeshtha (adhika) shukla 1980, Bhadra krishna 2012, and Shravana krishna
to Bhadra shukla 2022. Each 1980 value was checked against its printed
clock time. The 2022 ones were checked against their clock times where
read. Every model below was run in a research script (exact
trigonometry; the SDK's integer mean places, text epoch and
precession), not yet in the SDK.

- **The Sun is the text's.** The printed Sun at *miśramāna* (the
  printed 46|54 on 14 July 2022 is 30 ghatis + half the day, the
  apparent midnight) is the text's Sun, without bija, at local mean
  midnight on 85.48° E: 0.7 to 4.5 minutes of its motion over six days
  read, July to September 2022 (C1229).
- **The day length is the text's, at about 25¾° N.** The printed day
  length matches the text's (obliquity 24°, II.28; ascensional
  difference, II.61 to 63) at 25.7° to 25.8°: mean +0.2 to +0.4 palas,
  sd 1.5. That is neither Darbhanga (26.15° N) nor Sitamarhi (26.6° N).
  Modern obliquities fit worse. The almanac may well use a traditional
  palabha (C1224).
- **Tithi and nakshatra ends are the Makaranda's Moon, give or take
  minutes.** Each end is solved in the sky named, from the sunrise named
  at 25.75° N, 85.9° E. The table gives the model minus the print in
  minutes, over the 88 tithi ends and 87 nakshatra ends printed:

  | Sky and sunrise | Tithi mean, sd, max | ≤ 2 / ≤ 5 / ≤ 10 min | Nakshatra mean, sd, max | ≤ 2 / ≤ 5 / ≤ 10 min |
  | --- | --- | --- | --- | --- |
  | modern (Lahiri), modern sunrise | +18, 146, 299 | 1 / 1 / 4 | −42, 133, 343 | 2 / 2 / 5 |
  | the text, no bija, the text's sunrise | −3.8, 16.4, 34.6 | 5 / 13 / 30 | −7.2, 15.3, 31.9 | 6 / 14 / 37 |
  | the text + Makaranda bija, the text's sunrise | −0.9, 9.7, 24.7 | 15 / 26 / 57 | −4.9, 7.5, 16.1 | 4 / 22 / 56 |
  | the text + Makaranda bija, sunrise with bhujāntara | +2.9, 6.0, 24.1 | 23 / 58 / 75 | −0.9, 4.1, 9.9 | 32 / 66 / 87 |
  | the text + Makaranda bija, modern sunrise (centre, refraction) | −0.8, 11.7, 31.8 | 5 / 28 / 56 | −4.6, 10.0, 23.2 | 6 / 16 / 47 |

  The best line by year: 1980 tithi +9.8 ± 5.0, nakshatra +3.4 ± 3.9;
  2012 tithi +0.1 ± 6.9, nakshatra −2.7 ± 1.7; 2022 tithi +1.8 ± 4.6,
  nakshatra −1.5 ± 3.9.

  The modern sky misses by up to five hours, so the print is siddhantic
  beyond doubt. The bija halves the scatter. The bhujāntara (the Sun's
  equation of centre taken into the sunrise, II.46) halves it again. A
  modern sunrise is worse than either text sunrise.
- **What is left is not noise.** The nakshatra residual (the Moon
  alone) follows the Moon's anomaly. Fitting it asks for the apsis
  another 0.38° back and an equation 0.5% larger (an epicycle
  +0.15°), which brings both limbs to rms 2.65 minutes. Neither is a
  bija: the apsis would need −4.9 revolutions, which is not a multiple
  of four. And neither is in the book: its Moon equation is the text's.
  Tithis also run 3 to 6 minutes later than nakshatras, by a term that
  follows the Sun (C1225, C1226). Separating the Moon's and the Sun's
  parts as the tables do was measured too. It is *worse* (2022, the text's sunrise: tithi sd 10.9
  minutes against 6.1 joint), so the table procedure as read above is
  not yet the printer's.
- **The star planets are not modern.** The printed Mars, Mercury,
  Jupiter, Venus and Saturn of 14 July 2022 are 1.5° to 11.6° from
  Lahiri's. This almanac, unlike Nepal's committee (C38), prints
  siddhantic planets. Whether it uses the text's four steps or the
  Makaranda's three (Rupa §9) is not yet measured (C1227).
- **Day labels.** Over the 90 days the tithi and nakshatra at the model's
  sunrise agree with the printed ones on every day, in both text skies
  with the bija. A 10-minute residual decides a label only when an end
  falls within 10 minutes of sunrise. None of the 90 did.

## 5. The profile, as proposed

`mithila-makaranda`, `Mark::Traditional`, base `surya-siddhanta`:

```rust
patch.frame.siddhanta = Some(Siddhanta::Surya {
    bija: SuryaBija::Makaranda,          // new member: Bija::MAKARANDA
    sunrise: SuryaSunrise::TextBhujantara, // new member (C1223)
    planets: SuryaPlanets::Text,         // siddhantic, C1227
});
patch.frame.ayanamsha = Some(Ayanamsha::Suryasiddhanta.into());
patch.day.sunrise = Some(Sunrise::CentreNoRefraction.into());
```

```rust
pub const MAKARANDA: Bija = Bija {
    moon_apsis: -4, mercury: 16, jupiter: -8, venus: -12, saturn: 12,
    ..Bija::NONE   // moon 0, mars 0, moon_node: 0 until C1221 is read
};
```

- `SuryaSunrise::TextBhujantara` is the text's sunrise (six hours less
  the ascensional difference, local mean time) plus the Sun's equation of
  centre as time: four minutes a degree, later when the true Sun is ahead
  of the mean. That is the *bhujāntara* the text applies to a place
  computed for mean sunrise (II.46). It changes no existing profile.
  `TEXT` stays as built (C37).
- The location is the consumer's. The print's own place fits 25.75° N,
  85.9° E (C1224). The profile carries no place, as no profile does.
- Provenance: `Deviation { model: "SURYA_SIDDHANTA", .. }` as now, with
  the bija named in the stamp.
- Gate: an `xtask makaranda` pass, like `nepal-day`, that reads the
  transcribed days (§7, to be grown from the 35 scans) and generates a
  measured page. `check-makaranda` holds its counts.

The fixed lists this touches (catalogue kinds, bindings' enum counts,
`SHIPPED_PROFILES` 7→8, the profile loops in every binding's tests)
are listed when it is built.

## 6. Why it is not built yet

1. **Agreement.** The best sky reproduces the printed ends with sd 4 to
   6 minutes and maximum 24. Nepal's committee profile was shipped at
   1 039 of 1 138 within 1.5 minutes. A Mithila profile shipped now would
   carry a known ±10-minute gap whose cause is unknown (C1225, C1226). It
   would decide no day label in the 90 days measured, but would print
   ends minutes from the almanac's.
2. **The sunrise needs a new member** (C1223), and the evidence for it
   points two ways. The ends ask for the bhujāntara, but the printed
   midnight Sun drifts the other way across the same months (C1229).
3. **Several years.** Three years and 90 days are measured. The scans
   cover 35 years. The gate the consumer asked for (several years)
   needs more of them transcribed: one fortnight's page a year from
   1980 to 2022 at least, and the leap-month years.

What would settle each is in its crux. In order:

- read the Makaranda's Rahu and planet pages (C1221);
- run the table procedure end to end on Rupa's Śaka 1891 example and on
  the printed days (C1225, C1226);
- transcribe the almanac's front matter for its stated palabha and
  method (C1224);
- grow the transcribed days.

## 7. The printed days used

Format: civil date, the row's tithi (1 to 30, krishna 1 = 16), its end in
ghati|pala from sunrise (60|00 = no end that day), the row's nakshatra
(1 = Ashvini), its end, the day length. Each row is read from the page
image of the scan named.

```text
# 1980-81 scan p-005: first Jyeshtha (adhika) shukla, Saka 1902
1980-05-15 1 25|47 3 12|46 33|16
1980-05-16 2 22|15 4 10|53 33|19
1980-05-17 3 19|35 5 09|52 33|22
1980-05-18 4 18|07 6 10|01 33|25
1980-05-19 5 17|54 7 11|23 33|28
1980-05-20 6 19|00 8 14|03 33|30
1980-05-21 7 21|20 9 17|54 33|32
1980-05-22 8 24|47 10 22|53 33|34
1980-05-23 9 29|07 11 28|42 33|36
1980-05-24 10 34|07 12 35|09 33|38
1980-05-25 11 39|14 13 41|44 33|40
1980-05-26 12 44|10 14 48|05 33|42
1980-05-27 13 48|22 15 53|47 33|44
1980-05-28 14 51|38 16 58|35 33|45
1980-05-29 15 53|48 17 60|00 33|46
# 2012-13 scan p-005: Bhadrapada krishna
2012-09-01 16 33|00 24 01|28 31|28
2012-09-02 17 33|54 25 03|24 31|24
2012-09-03 18 36|15 26 06|36 31|20
2012-09-04 19 38|25 27 10|55 31|17
2012-09-05 20 43|38 1 16|16 31|14
2012-09-06 21 48|31 2 22|22 31|10
2012-09-07 22 53|42 3 28|51 31|06
2012-09-08 23 58|36 4 35|18 31|02
2012-09-09 24 60|00 5 41|17 30|58
2012-09-10 24 03|01 6 46|35 30|54
2012-09-11 25 06|30 7 50|45 30|50
2012-09-12 26 08|52 8 53|48 30|46
2012-09-13 27 09|56 9 55|34 30|42
2012-09-14 28 09|48 10 56|07 30|38
2012-09-15 29 08|22 11 55|28 30|34
2012-09-16 30 05|48 12 53|47 30|30
# 2022-23 scan pp. 10-13: Shravana krishna to Bhadrapada shukla
2022-07-14 16 41|21 21 43|28 33|48
2022-07-15 17 35|34 22 39|36 33|46
2022-07-16 18 30|27 23 36|22 33|45
2022-07-17 19 25|39 24 33|51 33|44
2022-07-18 20 21|53 25 32|21 33|43
2022-07-19 21 19|07 26 31|31 33|42
2022-07-20 22 17|31 27 32|04 33|41
2022-07-21 23 17|21 1 33|50 33|40
2022-07-22 24 18|09 2 36|53 33|38
2022-07-23 25 20|21 3 41|02 33|36
2022-07-24 26 23|41 4 46|29 33|34
2022-07-25 27 27|53 5 52|28 33|32
2022-07-26 28 32|47 6 58|47 33|30
2022-07-27 29 37|50 7 60|00 33|27
2022-07-28 30 42|39 7 05|26 33|24
2022-07-29 1 46|51 8 11|22 33|20
2022-07-30 2 50|09 9 16|48 33|17
2022-07-31 3 52|21 10 21|10 33|14
2022-08-01 4 53|16 11 24|26 33|11
2022-08-02 5 52|53 12 26|23 33|08
2022-08-03 6 51|41 13 27|05 33|05
2022-08-04 7 48|30 14 26|36 33|02
2022-08-05 8 44|45 15 25|03 32|59
2022-08-06 9 40|08 16 22|42 32|56
2022-08-07 10 34|51 17 19|32 32|53
2022-08-08 11 29|02 18 15|52 32|50
2022-08-09 12 22|56 19 11|49 32|47
2022-08-10 13 16|44 20 07|37 32|44
2022-08-11 14 10|37 21 03|12 32|41
2022-08-12 15 04|47 23 56|12 32|38
2022-08-13 17 54|50 24 53|29 32|35
2022-08-14 18 51|03 25 51|39 32|32
2022-08-15 19 48|13 26 50|41 32|29
2022-08-16 20 46|35 27 50|54 32|26
2022-08-17 21 46|10 1 52|23 32|23
2022-08-18 22 47|03 2 55|05 32|20
2022-08-19 23 49|14 3 58|58 32|16
2022-08-20 24 52|32 4 60|00 32|13
2022-08-21 25 56|43 4 03|49 32|10
2022-08-22 26 60|00 5 09|48 32|07
2022-08-23 26 01|38 6 16|13 32|04
2022-08-24 27 06|41 7 22|45 32|01
2022-08-25 28 11|37 8 28|57 31|58
2022-08-26 29 15|54 9 34|31 31|54
2022-08-27 30 19|23 10 39|08 31|50
2022-08-28 1 21|39 11 42|39 31|46
2022-08-29 2 22|43 12 44|55 31|42
2022-08-30 3 22|29 13 45|53 31|38
2022-08-31 4 21|00 14 45|39 31|34
2022-09-01 5 18|18 15 44|20 31|30
2022-09-02 6 14|39 16 42|12 31|26
2022-09-03 7 10|08 17 39|10 31|22
2022-09-04 8 04|54 18 35|35 31|18
2022-09-05 10 53|10 19 31|33 31|14
2022-09-06 11 46|58 20 27|21 31|10
2022-09-07 12 40|54 21 23|11 31|07
2022-09-08 13 35|06 22 19|13 31|04
2022-09-09 14 29|50 23 15|40 31|00
2022-09-10 15 25|12 24 12|43 30|56
```

Reading notes. Three scan fonts are involved:

- In 2022, a glyph like a Latin "R" is १: the printed Sun of 21 July,
  `03।04।R८।16`, is 3s 4°18′16″ by its neighbours.
- In 1980, the glyph like ६ is ९, confirmed by the printed clock times.
- One 2012 value (4 September, the tithi's 38|25) misses every model by
  15 minutes and is probably misread.

The 12 August 2022 row names Dhanishtha: Shravana ended before that
sunrise.
