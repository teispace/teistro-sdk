# The Makaranda and the Mithila panchang

Status: `built`, 2026-11-08, for SR-16 and A2-8 (a `siddhanta` profile
that reproduces the almanacs of Mithila). The research was written
before any code (design page first). Section 3 is read from the sources.
Section 4 is measured against 162 printed days over eight years, 1980 to
2022. Section 5 is the profile as built, `mithila-makaranda`, and
section 6 is what stays open. It reproduces every printed day's sunrise
nakshatra and all but one sunrise tithi, and the printed ends within
minutes (sd 4 to 6, the worst 23). The residual's two sources are
shown, the Moon's anomaly and the Sun's, but not closed (C1225, C1226).

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
  (364 212.0116) says (C1221). The book itself (p. 21) confirms it:
  the Guru page prints the text's motion, and its footer subtracts the
  years since the aeon began over 1500 as degrees, which is −8
  revolutions an age. The node's printed motion fits no whole count, so
  it waits on the book's own Rahu page.

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

One fortnight's page a year was transcribed for eight years, 162 printed
days in all (§7): 1980, 1992, 1995, 1998, 2005, 2012, 2015 and 2022. Every
end was checked against the clock time the page prints beside it. An end
whose figure and clock disagree is written `?|?`, never guessed. That
leaves 156 tithi ends and 150 nakshatra ends. The figures below come from
a research script (exact trigonometry; the SDK's integer mean places,
text epoch and precession). `crates/siddhanta/tests/mithila.rs` holds the
SDK (the text's sine table) to bounds set from them.

- **The place is the almanac's own.** Its front matter (2022-23) says
  every Mithila panchang is computed for 85°30′ E, 26°35′ N, palabha 6
  angula, "as from of old". Palabha 6 is 26°34′, and its charakhandas
  (60, 48 and 20 palas) are the exact ascensional differences there to a
  pala. At that latitude the text's day length (obliquity 24°) is the
  printed one within 12 palas on all 162 days, mean +0.2 (C1224). The
  Makaranda's own deshantara for Mithila agrees: 0;20,48 of the Moon's
  motion (p. 17) and 0;0,8 of Jupiter's (p. 21) are both 1 ghati 35 palas,
  9½° east of Ujjain.
- **The Sun is the text's.** The printed Sun at *miśramāna* (the printed
  46|54 on 14 July 2022 is 30 ghatis + half the day, the apparent
  midnight) is the text's Sun, without bija, at local mean midnight on
  85½° E: 0.7 to 4.5 minutes of its motion over six days read, July to
  September 2022 (C1229).
- **Tithi and nakshatra ends are the Makaranda's Moon, give or take
  minutes.** Each end is solved in the sky named, from the sunrise
  named, at the stated place. Model minus print, minutes:

  | Sky and sunrise | Tithi mean, sd, max | ≤ 2 / ≤ 5 / ≤ 10 of 156 | Nakshatra mean, sd, max | ≤ 2 / ≤ 5 / ≤ 10 of 150 |
  | --- | --- | --- | --- | --- |
  | modern (Lahiri), modern sunrise | −3.9, 156, 369 | 2 / 2 / 8 | −49, 144, 363 | 1 / 4 / 9 |
  | the text, no bija, sunrise with bhujāntara | −0.7, 13.6, 27.9 | 17 / 34 / 69 | −1.4, 13.6, 35.8 | 11 / 26 / 61 |
  | the text + Makaranda bija, the text's sunrise | −2.7, 8.2, 24.5 | 38 / 66 / 112 | −3.5, 7.3, 20.0 | 23 / 67 / 111 |
  | **the text + Makaranda bija, sunrise with bhujāntara** | **+0.1, 6.4, 23.1** | **43 / 81 / 139** | **−0.7, 4.1, 20.8** | **71 / 118 / 149** |

  The last line by year (tithi; nakshatra; their difference):

  | Year | Tithi | Nakshatra | Tithi − nakshatra |
  | --- | --- | --- | --- |
  | 1980 | +9.7 ± 4.9 | +3.4 ± 3.8 | +6.3 |
  | 1992 | −1.2 ± 1.8 | +1.6 ± 2.4 | −2.8 |
  | 1995 | −4.6 ± 4.1 | +1.5 ± 2.2 | −6.1 |
  | 1998 | −8.7 ± 1.2 | +0.1 ± 5.9 | −8.8 |
  | 2005 | +4.2 ± 3.0 | −2.5 ± 2.5 | +6.8 |
  | 2012 | −1.0 ± 6.9 | −3.8 ± 1.7 | +2.8 |
  | 2015 | −4.1 ± 2.4 | +0.6 ± 1.7 | −4.7 |
  | 2022 | +1.4 ± 4.8 | −1.9 ± 3.9 | +3.3 |

  The modern sky misses by up to six hours, so the print is siddhantic
  beyond doubt. The bija halves the scatter. The bhujāntara (the Sun's
  equation of centre taken into the sunrise, II.46) takes it down by a
  third again. The nakshatra's worst, 20.8, is 1998-12-28, a printed
  end 54|55 after the one before it where its neighbours are 56|50 apart:
  a printing slip, kept as printed. Without it the nakshatra's worst is
  10.0.
- **What is left is not noise, and its sources are shown, not closed.**
  The nakshatra residual (the Moon alone) follows the Moon's anomaly.
  Fitting it asks for the apsis another 0.38° back and an equation 0.5%
  larger. Neither is a bija (−4.9 revolutions is not a multiple of four)
  and neither is in the book, whose Moon equation is the text's (C1225).
  The tithi's extra residual is the Sun's: tithi less nakshatra on the
  same day is 0.1 + 6.1 cos(the Sun's mean anomaly) minutes, correlation
  0.77 over 147 days. It is not the Sun's apogee: the Makaranda prints
  2s 17°17′21″ (p. 28), the text's (C1226). Neither is fitted into the
  profile.
- **The book's procedure, end to end.** Rupa's worked example (Śaka
  1891) gives the year's first tithi end, krishna 10, as Friday 55|34
  after mean sunrise. The text with the bija gives Friday 53|59 after
  6h local mean time at Ujjain. Add the Makaranda's Mithila deshantara
  (1|35, above) and it is 55|34: Rupa's unexplained +1|35 over the sum
  of the kanda cells is that deshantara. So the tables are the text plus
  bija to the pala on this example, and their place is Mithila. The
  printed days' residual is therefore not in the tables' reading of the
  text. It lies in what the printer does beyond them (C1225, C1226).
- **The star planets are not modern.** The printed Mars, Mercury,
  Jupiter, Venus and Saturn of 14 July 2022 are 1.5° to 11.6° from
  Lahiri's. This almanac, unlike Nepal's committee (C38), prints
  siddhantic planets. Whether it uses the text's four steps or the
  Makaranda's three (Rupa §9) is not yet measured (C1227).
- **Day labels.** The tithi and nakshatra at the model's sunrise agree
  with the printed ones on 161 and 162 of the 162 days. The one tithi
  that falls the other side ends minutes after sunrise.

## 5. The profile

`mithila-makaranda`, `Mark::Traditional`, base `surya-siddhanta` (the
text's zodiac and its sunrise on the geometric horizon):

```rust
patch.frame.siddhanta = Some(Siddhanta::Surya {
    bija: SuryaBija::Makaranda,      // Bija::MAKARANDA
    sunrise: SuryaSunrise::Bhujantara,
    planets: SuryaPlanets::Text,     // siddhantic, C1227
});

pub const MAKARANDA: Bija = Bija {
    moon: 0, moon_apsis: -4, moon_node: 4, mars: 0,
    mercury: 16, jupiter: -8, venus: -12, saturn: 12,
};
```

- The node's +4 is read from the Ketu vatika (p. 24, "Rahu's name 30"):
  its one-day cell is 59;59,28,12,30,50,2,56 of a 60-part circle, and 60
  less that is the text's node motion with four revolutions more in an
  age, to all seven sexagesimal places (C1221).
- `SuryaSunrise::Bhujantara` is the text's sunrise (six hours less the
  ascensional difference, local mean time) moved by the Sun's equation of
  centre as time: four minutes a degree, later when the true Sun is ahead
  of the mean (`SuryaSiddhanta::bhujantara_days`,
  `SiddhantaProvider::with_bhujantara`). It changes no existing profile;
  `TEXT` stays as built (C37, C1223).
- The location is the consumer's, as for every profile. The almanac's
  own is 85°30′ E, 26°35′ N.
- The check is `crates/siddhanta/tests/mithila.rs` over the data file:
  the day length within 15 palas; tithi ends mean within 2 minutes, at
  least 135 of 156 within 10 and all within 25; nakshatra ends mean
  within 2, at least 147 of 150 within 10 and all within 22; at least 161
  sunrise tithis and all 162 sunrise nakshatras; and the text without
  the bija under 90 of either within 10.

## 6. What is open

- C1225 and C1226: the printer's Moon and Sun beyond the text-plus-bija
  tables. The next step is the printer's own method. The almanac's front
  matter names the Makaranda and Narayana Daivajna but not the steps.
- C1227: the star planets' steps.
- C1229: the midnight Sun's drift against the bhujāntara.
- More years: the scans cover 35. Every new fortnight goes into the
  data file, and the test's counts move with it.

## 7. The printed days used

`crates/siddhanta/tests/data/vishwavidyalaya-panchang.txt`, one block a
year, its source page named. Columns: civil date, the row's tithi (1 to
30, krishna 1 = 16), its end in ghati|pala from sunrise (60|00 = no end
that day), the row's nakshatra (1 = Ashvini), its end, the day length.
