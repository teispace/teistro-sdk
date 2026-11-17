# Ketaki: Ketkar's reckoning

Status: `research`, 2026-11-17 (lane `lane/ketaki-research`, cruxes C2180 to C2199). The
consumer's addendum 8 (§6) lists Ketaki among the methods to add after Nepal, with one
line: "Its ayanamsha is reported as Revati-based; not verified". This page reads the
primary texts and answers seven questions: the texts, the ayanamsha, the planetary model,
the conventions, who uses it, existing software and the disputes. The implementation plan
is `03-design/ketaki.md`. The general framework a third and later reckonings plug into
is `03-design/reckonings.md`.

Evidence ranks follow ADR-0018. Rank 1 is a primary text or a printed almanac, rank 2 is
scholarship or an official report, and rank 3 is secondary writing or software. Every
number below cites its source by the key in the sources table (§10). A number computed
here says so, and names the script that produced it. The scripts were scratch files and
are not committed.

## Findings at a glance

| question | answer | confidence |
|---|---|---|
| the ayanamsha | **Chitra-paksha, not Revati.** *Ketakī Grahagaṇitam* (1899) puts the origin opposite Chitra (Spica) and gives a closed rule. The value is 22°08′33″ at the Mesha sankranti of Shaka 1800 (12 April 1878), growing by (n/70)° − (n/50)′ a year, where n is the Shaka years since 1800. That is **50.2286″ a sidereal year**, linear. | high (rank 1, two texts agree, checked numerically) |
| the "Revati" report | It comes from a different ayanamsha, 18°10′25″ in Shaka 1800. That value is ζ Piscium's own tropical longitude in 1878, computed here to 0.3″. Ketkar printed it in the first edition of *Jyotirgaṇitam* (1898) beside the 22° value, and by 1928 he had dropped it in writing. The Tilak panchang (Pune, Nagpur) took it up. | high |
| against Lahiri | Ketaki stands **53″ to 66″ below** mean Lahiri over 1878 to 2100 (56″ at the Calendar Reform Committee's 1956 epoch, 61″ in 2026). Against Spica held at 180° of mean geometric longitude, the gap is 1″ to 5″ over the same span. | high (computed, §2.4) |
| the SR-112 form | `CUSTOM { epoch_jd_tt: 2407086.5, value_deg: 22.1425, rate_deg_per_year: 0.013952138 }` restates the text **exactly**. The definition is a value plus a linear rate. `PRECESSED` and `ANCHORED` only approximate it. | high |
| the planetary model | Two models. *Ketakī Grahagaṇitam* (1899) is a **karana**: verse rules in Grahalaghava's format, an epoch at Ujjain mean sunrise on 3 April 1878, 6940-day cycles, rational mean motions and ten-degree tables. Its constants are modern: its lunar motion matches the modern sidereal month to 2 × 10⁻⁹ °/day, and it carries evection, variation, the annual equation and the Moon's secular acceleration. *Jyotirgaṇitam* (1898) is a **book of tables** reduced from Le Verrier, Hansen, Chhatre and Newcomb, which claims places correct to 1′. | high |
| reproducible? | The karana is fully specified and can be reproduced exactly once its verses and six tables are transcribed from the page images. The tables of *Jyotirgaṇitam* are reproducible in principle but run to hundreds of pages. | medium-high |
| what Ketaki panchangs compute today | A modern ephemeris with a Chitra-paksha ayanamsha (Ketaki's own value, or Lahiri's from 1957), amanta months, Shaka years and sunrise-based days. In 1955 the Calendar Reform Committee surveyed 16 almanacs that named Ketkar's works as their method. Every one reported an ayanamsha of 23°12′06″ to 23°12′10″, which is Ketaki's rule within 4″. | high for 1955, medium for today |
| who uses it | In 1955: Date's (Solapur), Nirnaysagar (Bombay), Chitrashala (Pune), Latkar and the Kolhapuri (Kolhapur), Ruikar's (Karad), Vaijayanti (Puttur, Kannada), the Podagatlapalli almanacs (Telugu), Khandesh (Jalgaon), the Brihan Maharashtriya (Pune) and four in Delhi and the north. Today the Date Panchang is the chief heir in Maharashtra. | medium |

## 1. The texts

Ketkar (1853 or 1854 to 1930; the son's biography gives Shaka 1775) was a schoolmaster at
Bagalkot and later lived at Bijapur. In a supplement dated 20 November 1928 he listed his
own works ([JG1937] preface, supplement §2). The list is the authority for the table
below.

| # | work | year | language | contents | scan |
|---|---|---|---|---|---|
| 1 | *Jyotirgaṇitam* (*sopapattika grahagaṇita*) | 1898 (1st, Anandashrama Press, Poona); 1937 (2nd, Poona, ed. D. V. Ketkar); a Chaukhamba reprint (Vitthaldas Sanskrit Series 14, ed. Surakant Jha) | Sanskrit with an English preface | the full modern reckoning in tables: the Sun and the planets from Le Verrier, the Moon de novo with Hansen's day-arguments recast in arc, Jupiter and Saturn from K. L. Chhatre, Uranus and Neptune from Newcomb, stars from the *Nautical Almanac* for 1881; eclipses, phases, the paths of solar eclipses, refraction at rising and setting, heliacal risings, occultations, the mahapata | [JG1937], [JG-Jha], [JG-plain] (archive.org) |
| 2 | *Ketakī Grahagaṇitam* | 1899 (Shaka 1821, Indira press, Poona) | Sanskrit verse with his own *vivṛti* | a karana: ten adhikaras (madhyama, spashta, five planets, triprashna, lunar and solar eclipses, conjunctions, phases, the Moon's horns, pata), six tables (pp. 81–88), worked examples at Bagalkot | [KGG] (Commons, 95 pp.) |
| 3 | *Vaijayantī* (*pañcāṅgagaṇitam*) | 1900 | Sanskrit | the panchang reckoning; a modern edition is in print (Bharatiya Vidya Bhavan) | not found |
| 4 | *Ketakī pariśiṣṭa* | 1916 | Sanskrit | a supplement | not found |
| 5 | *Bhārata-bhū-maṇḍala sūryagrahaṇam* | 1921 | Sanskrit | solar eclipses over India | not found |
| 6 | *Saurārya-brāhma tithigaṇitam* | 1927 | Sanskrit | tithis in the three old pakshas | not found |
| 7 | *Tables of the Sun, Moon and Planets* (*Marāṭhī jyotirgaṇita pustakāvalī* no. 2, *Grahagaṇita*) | 1914 | Marathi | the tables in Marathi | [GG1914] (Commons, 190 pp.; downloaded, not yet read) |
| 8 | *Nakṣatra-vijñāna* | 1916 | Marathi | the stars | not found |
| 9–11 | *Problem of two bodies* (1918), *Calendar Reform* (1923), *Problem of three bodies* (1924, incomplete) | | Marathi | | not found |
| 12 | *Indian and Foreign Chronology* | 1923 (the DLI record says 1921) | English | chronology with Ketaki's tables; §200 argues the Chitra origin | [IFC] (archive.org) |

His son Dattatreya (Dattaraj) Venkatesh Ketkar added the *Ketakī-parimala-vāsanā-bhāṣya*
(1930) and *Śāstra-śuddha pañcāṅga ayanāṃśa nirṇaya* (1929) ([JG1937] supplement §3).
Neither was found.

**The epoch.** *Ketakī Grahagaṇitam* counts its days (ahargana) from Chaitra shukla 1 of
Shaka 1800, a Wednesday, at mean sunrise on the meridian of Ujjain ([KGG] madhyama vv. 1–2
and the example on p. 3). [IFC] (p. 145) gives the same epoch as Julian day 2,407,078,
and reckons the weekday as Wednesday. That day is 3 April 1878. Its mean places
(*kṣepaka*) are for that instant (madhyama vv. 3–9). Its cycle (*cakra*) is 6940 days,
the 19-year cycle, with a *dhruvaka* for each body's motion over one cycle (vv. 10–14).
The ayanamsha's own epoch is the Mesha sankranti of the same Shaka year, 12 April 1878
([JG1937] p. 346).

## 2. The ayanamsha

### 2.1 The definition (C2180)

*Ketakī Grahagaṇitam* states the origin in its third verse ([KGG] p. 1, v. 3). The Surya
Siddhanta gives Chitra a longitude of half the circle. So the ayanamsha is the distance
from the equinox to the point opposite that star, measured along the ecliptic, and in
Shaka 1800 it was 22°9′. The spashta adhikara (v. 6, p. 12) gives the working rule. Take
n = the Shaka year less 1800. Divide n by 70 for degrees and by 50 for minutes, and take
the difference. Add 22°8′33″. The result is the ayanamsha at that year's Mesha sankranti.
The text's own example takes Shaka 1815 and obtains 22°21′6″.

*Jyotirgaṇitam* agrees. Its panchang chapter ([JG1937] ch. 1, p. 7) says the observed
ayanamsha at the start of Mesha in Shaka 1800 was 22°8′33.04″, with the origin opposite
Chitra. A worked problem in the triprashna chapter ([JG1937] pp. 346–347) computes Spica's
tropical longitude from its observed right ascension and declination on 1 January 1892,
obtaining 22°20′2.9″ after subtracting 180°. It carries the result back by the table's
yearly motion to 22°8′33″ on 12 April 1878. The 1928 note on the same page sets
Varahamihira's 3°20′ (Shaka 452) plus 1348 years of motion beside it at 22°8′39″, as
"slightly coarse".

The rate, (1/70)° − (1/50)′ per year, is 51.4286″ − 1.2″ = **50.228571″ per year**. The
year is the sidereal one: the text adopts the observed sidereal year in place of the Surya
Siddhanta's, which v. 4 says is 8½ palas too long. *Jyotirgaṇitam* gives that year as 365
days 15 ghatis 22.9 palas, or 365.256361 days ([JG1937] p. 7). Per Julian year, the rate is
50.228571 × 365.25 / 365.256361 = **50.22770″**.

The epoch value against later dates (all from the text's rule):

| date | Ketaki | source of the date's printed value |
|---|---|---|
| Mesha 1878 (Shaka 1800) | 22°08′33″ | [KGG] v. 6; [JG1937] p. 7 |
| Mesha 1884 (Shaka 1806, Chaitra shukla 15) | 22°13′34″ | [JG1937] p. 296 prints 22°13′36″ |
| 1 January 1892 | 22°20′02.2″ | [JG1937] p. 346 computes 22°20′02.9″ from Spica |
| Mesha 1893 (Shaka 1815) | 22°21′06.4″ | [KGG] p. 12 prints 22°21′6″ |
| Mesha 1954 (Shaka 1876) | 23°12′10″ | [RCRC] Annexure VI: 16 Ketkar-method almanacs print 23°12′06″ to 23°12′10″ |

### 2.2 The basis (C2181)

Ketkar's stars are the *Nautical Almanac*'s mean places for 1881 ([JG1937] preface). His
1892 figure, 22°20′2.9″, falls 0.5″ from Spica's **geometric longitude on the mean
equinox** of that date (22°20′03.4″). Spica's apparent longitude that day, with
aberration and nutation, is 22°19′46.1″, 17″ away (computed here, `chk1892.py`). The value
is therefore a **mean** value: no nutation and no aberration. In the SDK, a mean value
subtracted on the root's `TRUE` basis (SR-100, C1750) gives sidereal longitudes referred
to the stars, which is what Ketkar's mean sidereal tables gave. Whether the
*Jyotirgaṇitam* tables applied nutation to a planet's tropical longitude is a second
question (C2182). It moves a sidereal longitude by at most 17.2″.

### 2.3 What the "Revati" report is (C2183)

Several web pages give "18°10′25″ in Shaka 1800" as Ketkar's ayanamsha and tie it to ζ
Piscium (rank 3). The number is real, but it is not Ketaki's:

- ζ Piscium's tropical longitude at the Mesha sankranti of 1878 is **18°10′25.3″**
  (computed here, `ayan.py`). The 18° ayanamsha is therefore the star itself taken as
  the end of Revati, at 0° of the sidereal zodiac.
- [JG1937] (publisher's note, pp. (21)–(22)) records three panchang types current in
  Shaka 1807 (1885): one on 22°, accepted by all; a proposed one on 18°; and a sayana one
  begun by S. B. Dikshit. Ketkar's first edition (1898) printed two types, the 22° and
  the 18°. His note of Shaka 1850 (1928) says he has now dropped the 18° type, as new,
  against the sages' division of the nakshatras, and easily seen to be wrong. He keeps
  22° alone, "pure by tradition, calculation and scripture".
- At the Sangli jyotisha conference, with Tilak's consent, a resolution took the Revati
  yogatara as the zodiac's origin. The Tilak panchang followed it ([PAM] pp. 1–3). The
  Nagpur Tilak Panchang reported 19°13′51″ for Shaka 1875 to 1876, with Daftari's
  *Karaṇakalpalatā* as its method ([RCRC] Annexure VI). ζ Piscium stood at 19°14′15″
  then, so the gap is 24″.
- The Maharashtra panchang-unity board of 1925 named four parties: the Zeta (Tilak)
  panchang, the Chitra (Ketaki) panchang, the Grahalaghava and the siddhantic ([PAM]
  p. 8).

So the addendum's line conflates two schools. **Ketaki is Chitra-paksha.** The
Revati-paksha value belongs to Ketkar's first edition and lives on in the Tilak panchang.
ζ Piscium and the point opposite Spica stand 3°58′ apart ([RCRC] Appendix 5-B, cited by
[Mercier] p. 6), so the two schools' sankrantis fall about four days apart.

### 2.4 Against the other ayanamshas (computed)

`ayan.py` (scratch) uses the IAU 1976 precession of Spica and ζ Piscium (Hipparcos
J2000.0 places, proper motion, no aberration) and Lahiri as the Swiss Ephemeris defines
it (23.250182778° − 0.004658035° at JD 2,435,553.5, carried by IAU 1976). Its star
longitudes are good to a few arcseconds. The SDK's own catalogue values would replace
them in the measurement phase.

| instant | Ketaki | mean Lahiri | Ketaki − Lahiri | Spica − 180° (mean, geometric) | Ketaki − that | ζ Psc (Tilak's origin) |
|---|---|---|---:|---|---:|---|
| Mesha 1878 | 22°08′33.0″ | 22°09′25.7″ | −52.7″ | 22°08′34.3″ | −1.3″ | 18°10′25.3″ |
| Mesha 1893 | 22°21′06.5″ | 22°21′59.7″ | −53.3″ | 22°21′07.7″ | −1.2″ | 18°23′01.1″ |
| Mesha 1954 | 23°12′10.3″ | 23°13′06.4″ | −56.1″ | 23°12′11.7″ | −1.4″ | 19°14′14.8″ |
| 1956-03-21 | 23°13′47.7″ | 23°14′43.9″ | −56.2″ | 23°13′49.1″ | −1.4″ | 19°15′52.5″ |
| J2000.0 | 23°50′26.8″ | 23°51′25.5″ | −58.8″ | 23°50′28.9″ | −2.1″ | 19°52′39.2″ |
| 2026-04-14 | 24°12′26.8″ | 24°13′27.4″ | −60.5″ | 24°12′29.6″ | −2.8″ | 20°14′44.0″ |
| 2100-01-01 | 25°14′09.5″ | 25°15′15.7″ | −66.2″ | 25°14′14.8″ | −5.3″ | 21°16′40.9″ |

Read the table this way. Ketkar fixed Spica's mean longitude in 1878 and carried it at a
constant 50.23″. Spica's own longitude moves at about 50.27″ a year, so the two part by
about 4″ by 2100. Lahiri, fixed at 23°15′ (true) in 1956 to reconcile the almanacs "based
on modern constants" ([Mercier] p. 5, citing [RCRC] p. 16), sits about one minute of arc
above Ketaki. Ketaki's zero year, 22°08′33″ divided by 50.2286″, is 1587 years before 1878,
which is **AD 291**. That is the year [IFC] (§162, §200) and [Mercier] (p. 4) name.

**The effect of one arcminute.** The Sun moves about 59′ a day, so a sankranti under
Ketaki falls about 24 minutes after Lahiri's. A nakshatra or yoga end, where the Moon
moves 13° a day, falls 1.5 to 2 minutes later. Tithis do not move. An adhika masa turns
on a sankranti and a new moon falling within 24 minutes of each other. On a rough
estimate (twelve sankrantis, a 48-minute window in a 29.5-day month), that happens in
about one year in seventy. To measure (C2187).

### 2.5 Which SR-112 form expresses it (closes C2165 for Ketaki)

| form | restates Ketaki | why |
|---|---|---|
| `CUSTOM { epoch_jd_tt: 2407086.5, value_deg: 22.1425, rate_deg_per_year: 0.013952138 }` | **exactly** | The text's definition is a value at an epoch plus a constant yearly rate. 0.013952138° is 50.22770″ per Julian year (§2.1). The epoch is 12 April 1878, 0h. The text names the day, not the hour. A day's difference moves the value by 0.14″. TT against UT at that epoch is about −5 s, which is negligible. |
| `PRECESSED { … }` | no | It carries the value by the general precession, which has a t² term and a slightly different rate. Over 1878 to 2100 it parts from the text's rule by 5″ to 10″. |
| `ANCHORED { star: SPICA, longitude_deg: 180 }` | no, ±20″ | The SDK reads an anchored star's apparent longitude with aberration (`astro-custom-ayanamsha.md` §4). Ketkar's is the mean place. Annual aberration puts a yearly wave of about ±20″ on the anchored value. Without aberration, the anchored value would sit within 5″ of the text over 1878 to 2100. |

The Revati value of Ketkar's first edition and the Tilak panchang is `ANCHORED { star:
REVATI, longitude_deg: 0 }`, within the same aberration wave, or a linear form if a text
gives its rate (C2183, C2184).

## 3. The planetary model

### 3.1 *Ketakī Grahagaṇitam*: a karana with modern constants (rank 1)

The opening verse (v. 2) explains the book. Ganesha's *Grahalāghava* still rules India,
but its places have grown loose against sight, so Ketkar writes a reckoning proved by
good observation. It keeps the *Grahalāghava*'s form: verse rules for a computer who
knows arithmetic and no trigonometry. Its contents, by adhikara ([KGG]):

| part | what the text gives |
|---|---|
| meridian | the line through Jalandhar, Jaipur, Tonk, Ujjain, Bhusawal, Bid, Solapur, Bagalkot, Harihar, Chikmagalur, Madikeri and Calicut (vv. 6–7); the Earth's circumference 2500 yojanas (v. 8) |
| mean places | the ahargana from Chaitra shukla 1, Shaka 1800, in 19-year cycles (vv. 1–2); mean places at the epoch (vv. 3–9); motion per 6940-day cycle (vv. 10–14); for the remaining days, rational rules: the Sun moves the days less a 70th in degrees, less a 160th in minutes (0.98561012° a day against the modern sidereal 0.98560911°); the Moon 14 days less a 17th of that in degrees, less a 150th of the days in minutes (13.1763595° a day against 13.1763580°); and like rules for the apogee, the node and the five planets (vv. 16–20) |
| secular terms | the Moon's acceleration as (n/100)²/7 arcminutes from Shaka 1800, four times that for the apogee and three quarters for the node, always additive (vv. 21–25, with an example for Shaka 2200: 2′17″); the Jupiter–Saturn great inequality on a 918-year cycle from Shaka 1481 (vv. 26–27) |
| the Sun | the equation of centre from a table at every 10° of anomaly, greatest 115′ (spashta vv. 3–5); daily motion from a table (v. 10) |
| the Moon | from the mean Moon, in order: *cyuti* (evection, about 74′ at most), *tithi-saṃskāra* (variation, about 36′), the manda equation (centre, about 6°18′), the annual equation through the bhujantara, and a reduction to the ecliptic of up to 7′; each from a table at 10° steps with linear interpolation (spashta vv. 13–24, pp. 16–21) |
| the planets | heliocentric mean places, a manda equation, the radius vectors (*mandakarṇa*) of the planet and the Earth, then the geocentric place by a rule that avoids trigonometry; latitudes (pp. 23–39) |
| the day | places at mean sunrise at Ujjain, carried to the true sunrise of a place by the longitude difference (*rekhāntara*), the ascensional difference (*cara*) from the noon shadow (*palabhā*) and the equation of time (*udayāntara*) (spashta vv. 7–9; the Moon's v. 9) |
| the panchanga | the tithi from (Moon − Sun)/12°, its end by the ratio of the difference to the daily motions, the karana, the nakshatra and the yoga (p. 22) |

Every constant is a modern one recast as a fraction: the sidereal year, the mean
motions, the evection and variation, the secular acceleration and the great inequality.
The arithmetic is a karana's. It is a closed procedure, so it can be reproduced to the
digit, provided the verses and the six tables are transcribed correctly (C2185).

### 3.2 *Jyotirgaṇitam*: tables from European theory (rank 1)

The English preface ([JG1937] pp. 1–5, dated Bagalkot, 20 November 1898) sets out the
sources:

- The Sun, Mercury, Venus and Mars come from Le Verrier, with the mean anomaly in days
  after Hansen's manner. The tables give the true anomaly at equal intervals of days in
  place of an equation of centre, and include the reduction to the ecliptic. The secular
  change is given per thousand years.
- The Moon's tables were built anew, with arguments in arc so that the tithi structure
  helps. A double-entry table groups evection, variation, the equation of centre and two
  lesser inequalities.
- Jupiter and Saturn come from K. L. Chhatre's tables, with the perturbations applied to
  the mean anomaly. Uranus and Neptune come from Newcomb's tables, which the Smithsonian
  Institution sent him.
- The geocentric places come from mean elongations and multipliers, again avoiding
  trigonometry.

He claims places correct to a minute of arc and prints comparisons with the *Nautical
Almanac* for 31 March and 1 April 1893 ([JG1937] pp. 4–6; the second table is for mean
sunrise at Ujjain). The OCR of the comparison tables is too damaged to transcribe. The
page images are needed (C2185).

### 3.3 Reproducible, and what is computed today

- **The karana** is reproducible exactly from its text. Its errors against the real sky
  are the ones its fractions and ten-degree tables carry. Ketkar compares it with the
  *Nautical Almanac* elsewhere in the book.
- **The tables** are reproducible in principle. Their accuracy (about 1′) is inside what
  a modern ephemeris gives, so for a panchang's minutes a modern ephemeris under Ketaki's
  ayanamsha and conventions reproduces them. A table-faithful reproduction would only
  matter for reading printed almanacs of 1898 to the 1950s exactly.
- **Almanacs since about 1950.** The 1955 survey ([RCRC] Annexure VI) lists every
  Ketkar-method almanac as "Modern method". They name *Jyotirgaṇitam*, *Grahagaṇita*,
  Gorakh Prasad's solar and lunar tables, Daftari's *Karaṇakalpalatā*, Bhatt's Mercury
  tables, Rajan's *Rāja-jyotiṣa-gaṇitam* and the *Nautical Almanac*. After the Rashtriya
  Panchang (1957), the almanacs moved to modern ephemerides. Lahiri's ayanamsha sits one
  minute from Ketaki's. **"Ketaki" in a modern almanac is therefore a modern sky, the
  Chitra-paksha zodiac and Ketkar's conventions.** The karana is the historical layer.

## 4. Conventions (C2186)

| convention | Ketaki | source |
|---|---|---|
| zodiac | sidereal, Chitra-paksha (§2) | [KGG] v. 3, v. 6 |
| year | sidereal, 365.256361 days (365 days 15 ghatis 22.9 palas) | [KGG] v. 4; [JG1937] p. 7; [RCRC]: the Ketkar almanacs print 365d 15g 22p 57vp |
| months | amanta; adhika is a lunar month with no sankranti, kshaya one with two (Bhaskara's rule, quoted) | [JG1937] ch. 1, p. 7; [RCRC] |
| year count | Shalivahana Shaka, elapsed, from Chaitra shukla 1 | [KGG] madhyama v. 1; [RCRC] |
| the day | the tithi, nakshatra and yoga current at sunrise are written; one that spans two sunrises is written twice (vriddhi); one wholly between two is dropped (kshaya); a tithi lasts 50 to 67 ghatis, a nakshatra 52 to 68 and a yoga 49 to 63 | [JG1937] ch. 1, pp. 7–8 |
| time | the tables give ends in Ujjain mean time, which is mean rather than apparent (savana); a place adds its longitude difference | [JG1937] ch. 1, p. 8; [KGG] spashta v. 9 |
| sunrise | the karana carries mean sunrise to true sunrise by the ascensional difference and the equation of time, which makes it the Sun's centre on the geometric horizon. *Jyotirgaṇitam* applies refraction to every rising and setting (an example adds 5 palas for Spica at Bagalkot), but neither text was found to name the limb | [KGG] spashta vv. 7–9; [JG1937] pp. 312–314, the refraction table p. 503 |
| the node | the mean node with its secular term; no true node | [KGG] madhyama vv. 17, 23 |
| centre | geocentric; parallax only in eclipses | [KGG] chs. 6–7 |
| the ayanamsha in use | only for sayana quantities (the ascensional difference, declinations, the lagna); the places themselves are computed sidereally | [KGG] spashta vv. 6–8 |
| festivals | not in Ketkar's astronomical texts. The Maharashtra almanacs in his tradition decide festivals by the dharmashastra digests. The Date Panchang describes its decisions as dharmashastra-based (rank 3). No Ketaki-specific rule was found. | [Pudhari], [Navprabha] |

## 5. Who uses it

**Ketkar's own almanac.** He published the *Ketakī Pañcāṅgam* from Shaka 1821 (1899–1900),
printed at the Venkateshwar Steam Press, Bombay. He says seventeen others "of its own
type" followed ([JG1937] supplement §5). In 1923 he named places where almanacs on
Ketaki, Vaijayanti and Grahaganita were published: South Canara, Belgaum, Ellichpur in
Berar and Mathura ([IFC] §200(i), p. 153).

**The 1955 survey.** The Calendar Reform Committee's questionnaire ([RCRC] Part A,
Annexure VI) gives each almanac's founding year, place, author, ayanamsha and rate,
year length and method. Those naming Ketkar:

| almanac | language, place | since | ayanamsha (Shaka 1875–76) | rate | method named |
|---|---|---|---|---|---|
| Chitrashala Panchang | Marathi, Pune (D. L. Date, Solapur; G. B. Joshi, Pune) | 1924 | 23°12′07″ | 50.2″ | *Jyotirgaṇitam*, *Grahagaṇita* |
| Datey's Panchang | Marathi, Solapur (L. G. Date) | Shaka 1833 | 23°12′07″ | 50.25″ | Gorakh Prasad's tables, *Jyotirgaṇita*, *Karaṇakalpalatā*, Bhatt, Rajan, the *Nautical Almanac* |
| Nirnaysagar Panchang | Marathi, Bombay (L. G. Date) | Shaka 1786 | 23°12′07″ | 50.25″ | the same |
| Latkar Panchang | Marathi and Sanskrit, Kolhapur | 1910 | 23°12′06″ | 50.22″ | *Jyotirgaṇita*, *Grahagaṇita*, *Karaṇakalpalatā* |
| Kolhapuri Panchang | Marathi and Sanskrit, Kolhapur | 1910 | 23°12′06″ | 50.22″ | *Jyotirgaṇita*, *Karaṇakalpalatā* |
| Gharcha Jyotishi; Ruikar Varsha Bhavishya | Marathi, Karad (U. V. Ruikar) | 1920; 1933 | 23°12′08″ | 50″ | "Ketaki Jyotirganita" |
| Brihan Maharashtriya Panchang | Marathi, Pune | Shaka 1871 | 23°12′07″ | 50.26″ | *Jyotirgaṇitam* and *Grahagaṇita* |
| Khandesh Panchang | Marathi, Jalgaon | 1944 | 23°12′08″ | 50.2″ | "Mathematical system of Ketkar" |
| Vaijayanthi Panchanga | Kannada, Nerlakatte, Puttur, South Canara | 1917 | 23°12′06″ | 50.2″ | *Jyotirgaṇitam* |
| Purna Sastriya Andhra Patrika; Krishnamurthi Sastry; Purnasashtriya Panchangam | Sanskrit and Telugu, Podagatlapalli, East Godavari (the Pidaparthi family) | 1945; old | 23°12′07″ | 50.268″ | Chhatre's tables, *Jyotirgaṇita* |
| Gouri Sankara Panchang | Sanskrit and Telugu, East Godavari | 1930 | 23°12′07″ | 50.268″ | Chhatre, *Jyotirgaṇita*, Ketkar's Marathi *Grahagaṇitam* |
| Sri Viswa Martanda | Hindi, Delhi | 1934 | 23°12′08″ | 50″ | "Ketaki and Grahalaghavi" |
| Sri Saptarshi | Hindi, Delhi | 1933 | 23°12′07″ | 50.2″ | tithis by siddhanta, planets by Chhatre and *Jyotirgaṇitam* |
| Shri Visva Vijaya | Hindi and Sanskrit, Solan | 1946 | 23°12′08″ | 50.2″ (as printed) | *Jyotirgaṇita*, *Grahalāghava* |
| Kumbakonam Madathu | Tamil (solar), Trichinopoly | 1876 | 23°12′09.88″ | 50.2677″ | *Ketakī Grahagaṇitam*, *Jyotirgaṇitam*, *Grahakoṣṭhaka*, the *Nautical Almanac* |

For contrast, from the same survey: the Nagpur Tilak Panchang had 19°13′51″ (Revati, §2.3).
The Grahalaghava almanacs had 23°08′ to 23°10′ with a rate of 58.2″ to 58.5″. A Surya
Siddhanta panjika (Gupta Press, Calcutta) had 21°49′26″ with a rate of 54″.

**Today.** No almanac calling itself "Ketaki" was found online. The lines of descent are
these:

- The **Date Panchang** (Solapur), founded by L. G. Date, is the most used Marathi almanac.
  It is described as Chitra-paksha, trusted in Maharashtra, Goa, Gujarat and Karnataka,
  and as having Lahiri's ayanamsha ([Navprabha], [Misalpav]; rank 3, not read at the
  source: misalpav.com refused the connection).
- The **Tilak Panchang** (Pune) keeps the Revati origin. Families in Ratnagiri still keep
  Ganeshotsav by it ([Lokmat] 2015).
- A Hindi blog names three methods in use: Surya Siddhanta (Nepal), Grahalaghava
  (Varanasi) and "drik (Ketaki chitrapakshiya)" (Delhi) ([Sharadvedanta]; rank 3).

No evidence was found of a Nepali almanac on Ketaki. A Nepali article says Lahiri
refined Ketkar's Chitra-paksha ayanamsha mathematically ([Nagarik]; rank 3).

## 6. Existing software

| software | Ketaki? | how checked |
|---|---|---|
| Swiss Ephemeris and Teimeris | no: its sidereal modes have `SS_REVATI` and `TRUE_REVATI` (ζ Piscium at 359°50′, the Surya Siddhanta's), `TRUE_CITRA` and Lahiri, and nothing named for Ketkar | `teimeris/core/src/tm_ayanamsa.c`, the mode table and names |
| PyJHora 5.0.0 (PyPI, wheel SHA-256 72bb3212…5ab5) | no: `const.available_ayanamsa_modes` has 21 modes, none Ketaki, and `SIDM_USER` takes an epoch and value only | the wheel's `jhora/const.py` and `panchanga/drik.py` |
| Jagannatha Hora | not verified: the Windows program and its manual were not read | — |
| Drik Panchang (web) | not found: its pages name Lahiri | search |
| GitHub `Raagavan1092/wall-clock` (updated 2026-11-12, **no licence**) | yes: `js/engine/ketaki.js` implements the karana's mean places, secular terms, the Sun, the Moon (cyuti, tithi, manda, reduction, annual equation) and the planets, with notes in `docs/graha-sphuta-sources.md`. It reads the Sun's rule as a 158th where the text's worked example (5477 days, 34′14″) gives a 160th, and some table entries differ from the Wikisource text layer. | read for comparison only. Without a licence it cannot be copied (`CLEAN_ROOM.md`), and the SDK transcribes from the scan |
| GitHub `vishvAsa/jyotiSham` | a Markdown transcription of [RCRC] Annexure VI (the survey) | search |

## 7. Disputes in practice

1. **Ketaki against Lahiri.** The gap is one minute of arc (§2.4): sankrantis about 24
   minutes apart, nakshatra and yoga ends 1.5 to 2 minutes, and an adhika masa that
   differs perhaps one year in seventy (C2187). Both are Chitra-paksha, so in most years
   no festival differs.
2. **Chitra (Ketaki) against Revati (Tilak).** The gap is 3°58′: sankrantis about four
   days apart, and adhika months in different months or years. In Shaka 1848 (1926–27)
   the Tilak panchang had an adhika Jyeshtha where the Grahalaghava almanacs had an
   adhika Chaitra. Gudhi Padva, Ramnavami, Hanuman Jayanti, Akshaya Tritiya and the
   Vasanta Gauri were each kept twice that year in Maharashtra ([PAM] p. 1). In 2015 the
   Tilak panchang kept Ganesh Chaturthi on 18 August against 17 September elsewhere
   ([Lokmat]). This is the dispute the "Revati" line in the addendum points at.
3. **Ketaki against Grahalaghava.** Ketkar's preface gives the Grahalaghava's Moon errors
   up to 2½°, occultations five or six hours off, and Mercury and Mars up to 7° or 8°
   ([JG1937] preface). By 1925 the Ketaki and Grahalaghava parties had agreed within a
   small margin on an ayanamsha ([PAM] p. 4, the board's recommendation of 12 September
   1925).
4. **The scholarly dispute.** Mercier holds that Ketkar's reading of
   *Pañcasiddhāntikā* XIV.37 (*ardhāśrama* for *ardhāṣṭama*, putting Chitra at 180°) is
   at variance with three independent editors. He calls the general acceptance of the
   Chitra origin a mistake ([Mercier] pp. 4–6). This bears on history, not on what the
   almanacs compute.
5. **The siddhantic objection.** Advocates of the Surya Siddhanta object to subtracting a
   Chitra-paksha ayanamsha from modern positions at all, and give Ketkar's rate as 50″27‴
   ([Deshpande]; rank 3).

## 8. Acceptance values found

| # | value | rank | use |
|---|---|---|---|
| A1 | the ayanamsha rule's own example: Shaka 1815 → 22°21′6″ | 1 [KGG] p. 12 | the `KETAKI` member to the arcsecond |
| A2 | 22°8′33.04″ at Mesha, Shaka 1800; 22°13′36″ at Chaitra shukla 15, Shaka 1806; 22°20′2.9″ on 1 January 1892 | 1 [JG1937] pp. 7, 296, 346 | the member, within 2″ |
| A3 | sixteen almanacs' 23°12′06″ to 23°12′10″ for Shaka 1875–76; year 365.25636 days | 2 [RCRC] | the member, within 4″ |
| A4 | the karana's worked example: Shaka 1815 Chaitra shukla 15, Saturday 1 April 1893, Ujjain, ahargana 5477; the Sun's motion and its true place 11s 19°8′11″ at Ujjain mean sunrise; sayana 0s 11°29′17″; the ascensional difference −12 vikalas at Bagalkot; the Moon's corrections in turn | 1 [KGG] pp. 3–21 | the karana provider, step by step, once transcribed |
| A5 | the karana's vyatipata of Shaka 1805 Shravana krishna 4 at Bagalkot: entry 45.7 ghatis, middle 53.0, exit 0.3 ghatis on Thursday | 1 [KGG] pp. 84–86 | the karana's panchanga |
| A6 | *Jyotirgaṇitam*'s lunar eclipse of Shaka 1806 Chaitra shukla 15 at Bagalkot (parvanta 26 ghatis 57 palas) and its 1893 place tables against the *Nautical Almanac* | 1 [JG1937] pp. 4–6, 296 | the modern profile within 1′; needs the page images |
| A7 | Shaka 1848: Tilak adhika Jyeshtha against Grahalaghava adhika Chaitra | 2 [PAM] | the Revati profile's adhika masa |
| A8 | 2015: Tilak Ganesh Chaturthi on 18 August, others on 17 September | 3 [Lokmat] | the Revati profile against the Chitra one |

**Not found:** a printed Ketaki almanac page with tithi and nakshatra ends and
sankrantis (C2188). Where to get them:

- the Venkateshwar Press's *Ketakī Pañcāṅgam* (from 1899) in Pune and Mumbai libraries:
  BORI, the Bharat Itihas Sanshodhak Mandal and the Marathi Granth Sangrahalaya;
- the Date Panchang's back years from the publisher at Solapur;
- the Vaijayanthi Panchanga office at Puttur;
- the Wikimedia Commons Marathi scans (the Rajya Marathi Vikas Sanstha grant), which
  hold [KGG], [GG1914] and [PAM] and may hold an almanac.

## 9. Not found, after searching

- No scan or text of *Vaijayantī*, the *Ketakī pariśiṣṭa*, *Tithigaṇitam* or the son's
  *Ayanāṃśa nirṇaya* was found. Searched: archive.org (advanced search for "ketkar",
  "ketaki", "grahaganitam", "jyotirganitam" and केतकी), Wikimedia Commons and Marathi
  Wikisource, Google results and Exotic India's catalogue (a modern *Vaijayantī* edition
  is listed).
- Pingree's *Census of the Exact Sciences in Sanskrit* and the *Indian Journal of History
  of Science* were not reached. Ketkar falls after the *Census*'s usual scope (it lists
  authors to about 1900), and no IJHS paper on Ketaki surfaced in search.
- No online printed Ketaki almanac page (C2188), and no Ketkar statement of the 18°
  ayanamsha's rate.
- The Date Panchang's current ayanamsha was not read at the publisher. The report that
  it is Lahiri's is rank 3.

## 10. Sources

| key | title | author | year | URL | what it gives | rank |
|---|---|---|---|---|---|---|
| KGG | *Ketakī Grahagaṇitam* (Sanskrit, with the author's *vivṛti*) | V. B. Ketkar | 1899 (Shaka 1821) | commons.wikimedia.org/wiki/File:केतकी_ग्रहगणितम्_।.pdf; text layer on mr.wikisource.org | the definition, rule and example of the ayanamsha; the epoch; the karana | 1 |
| JG1937 | *Jyotirgaṇitam*, 2nd ed., with the 1898 preface and the 1928 supplement | V. B. Ketkar, ed. D. V. Ketkar | 1898; 1937 | archive.org/details/ekmk-jyotirganitam-by-venkatesh-ramkrishna-ketkar-sansk | the sources of the tables; the 22°8′33″ value; the 18° history; the Spica computation; the conventions; the list of works | 1 |
| JG-Jha, JG-plain | *Jyotirgaṇitam*, Chaukhamba reprint (Vitthaldas Sanskrit Series 14) | V. B. Ketkar, ed. Surakant Jha | n.d. | archive.org/details/jyotir-ganita-venkatesh-ramakrishna-kethkar-dattatreya-ketkar-surakant-jha; archive.org/details/Jyotirganitam | the same text, a second OCR | 1 |
| IFC | *Indian and Foreign Chronology* | V. B. Ketkar | 1923 | archive.org/details/in.ernet.dli.2015.142146 | Ketaki's epoch JD; the Chitra argument (§200); the sidereal year (§60); the almanacs on Ketaki | 1 |
| GG1914 | *Marāṭhī jyotirgaṇita pustakāvalī* no. 2, *Grahagaṇita* | V. B. Ketkar | 1914 | commons.wikimedia.org/wiki/File:मराठी_ज्योतिर्गणित_पुस्तकावली_नंबर_२_ग्रहगणित.pdf | the Marathi tables (downloaded, not read) | 1 |
| RCRC | *Report of the Calendar Reform Committee* | Government of India (M. N. Saha, chair) | 1955 | archive.org/details/calendar_reform_comittee_report | the 1955 survey of almanacs (Annexure VI), the 23°15′ decision | 2 |
| PAM | *Mahārāṣṭrīya pañcāṅgaikya maṇḍaḷa, pratham adhiveśanācā vṛttānta* | Dnyaneshwar Krishna Laddu (secretary), Pune | 1926 (session of 14–16 August 1925) | commons.wikimedia.org/wiki/File:महाराष्ट्रीय_पंचांगैक्य_मंडळ_प्रथम_अधिवेशनाचा_वृत्तांत.pdf | the four parties; the Sangli resolution; the Shaka 1848 split; the board's recommendation | 2 |
| Mercier | "Astronomical computation for the history of Indian astronomy" | Raymond Mercier | n.d. | raymondm.co.uk/IndianAstronomyComputation.pdf | Ketkar's reform, its zero year AD 291, its spread, the critique; Mercier had not seen [KGG] or [JG1937] (his n. 13) | 2 |
| Teimeris | `core/src/tm_ayanamsa.c` | Teispace | 2026 | local | no Ketkar mode | 3 |
| PyJHora | PyJHora 5.0.0 | P. V. R. Narasimha Rao's methods, Python port | 2026 | pypi.org/project/PyJHora | no Ketaki | 3 |
| WallClock | `Raagavan1092/wall-clock` | — | 2026 | github.com/Raagavan1092/wall-clock | a JS karana implementation, unlicensed | 3 |
| Navprabha | "गणेशचतुर्थी: समज-गैरसमज" | Navprabha | n.d. | navprabha.com | the Date method's reach | 3 |
| Misalpav | "टिळक कि दाते" | misalpav.com | n.d. | misalpav.com/node/15157 (refused the connection; search summary only) | Date against Tilak, 4° | 3 |
| Lokmat | "टिळक पंचांगाप्रमाणे आजपासून रत्नागिरीत गणेशोत्सव" | Lokmat | 18 August 2015 | lokmat.com/sindhudurga/tilak-panchang-ganesh-festival-ratnagiri/ | Ganesh Chaturthi 2015, Tilak against others | 3 |
| Deshpande | "सूर्यसिद्धांतीय गणितावर आधारित पंचांगच का वापरावे?" | Deshpande Panchang | n.d. | deshpandepanchang.com/vaishak-month-information/ | the siddhantic critique; the rate 50″27‴ | 3 |
| Sharadvedanta | "पंचांग निर्माण की तीन पद्धतियां" | blog | 2018 | sharadvedanta.blogspot.com/2018/07/blog-post_20.html | "Ketaki chitrapakshiya" drik in Delhi | 3 |
| Nagarik | "अयनांश: वेददेखि विज्ञानसम्म" | Nagarik News | n.d. | nagariknews.nagariknetwork.com | Lahiri refining Ketkar (Nepali) | 3 |
| Pudhari | Ganesh Chaturthi 2023, Solapur | Pudhari | 2023 | pudhari.news | the Date Panchang's self-description | 3 |

### Downloads

Kept under `/private/tmp/claude-501/ketaki-dl/` (outside every git tree) and read with
`python3 -I`, `pdftotext` and `grep`.

| file | bytes | SHA-256 | from |
|---|---:|---|---|
| `ketaki-gg/ketaki_grahaganitam.pdf` | 32,525,091 | 5b908ab97128521dc2698b38f3aaab89c1c5a60cd80b3aabe1b057162fcb4b6e (Commons SHA-1 f89b0c34…1d34b matches) | upload.wikimedia.org, [KGG] |
| `jg1937/jg1937_djvu.txt` | 1,892,502 | 24eb5fa5ab3f13acf80c8d9b2e7d5da23eed9bfa3ca8fdc0e7f0e3c8bdbd07da | archive.org, [JG1937] |
| `jgjha/jgjha_djvu.txt` | 1,925,671 | ba43e15b22b0144e12c9275fd89075a9869d7423420c6ca56b76aa2d9a36e2a1 | archive.org, [JG-Jha] |
| `jgplain/jg.pdf` | 23,382,646 | 9a63c65c91b863f3fc85f3125191ce030be52574fc2efaf2b8142d22f17e78df | archive.org, [JG-plain] |
| `jgplain/jg_djvu.txt` | 2,063,329 | d3004a813494488c7ca9182174717218464a0bebdb27138d46388b5c341526d3 | archive.org |
| `chron/chron.pdf` | 3,105,444 | 6d6b54f0619451ec27c313b05aa879de8240957854d59b9e04fbf1c220233fca | archive.org, [IFC] |
| `chron/chron_djvu.txt` | 267,776 | 04de29b485ffea4e6924da3999304f9bd21f3443c91767c5009f3b7d72acb18b | archive.org |
| `rcrc/rcrc.pdf` | 23,777,047 | 45b4710b7cf48a2a27141bed8cda30c50708d6e7fe9f68e09c28077b168016f3 | archive.org, [RCRC] |
| `rcrc/rcrc_djvu.txt` | 1,278,650 | 4def9c7c6e0df740b190b39f0919d791cbb07a0c2343474098371fb6a652678e | archive.org |
| `aikya/panchangaikya.pdf` | 101,842,960 | f8e475bda941496094bcd7cc69f79dbe3d171bb1ad8191cac65ed29580664119 (Commons SHA-1 63553a01…8159 matches) | upload.wikimedia.org, [PAM] |
| `gg1914/grahaganit_1914.pdf` | 44,632,948 | a7e7ac41dc5a4ca44ba59f4718435821d3c2686fb8e532094058ec285c29f284 (Commons SHA-1 7a90219d…665a matches) | upload.wikimedia.org, [GG1914] |
| `mercier/mercier.pdf` | 182,653 | 633caedfb13673254eaf84809b48eede1179199a058faf5c3a9915aeb56cb573 | raymondm.co.uk, [Mercier] |
| `pyjhora/pyjhora.whl` | 29,228,394 | 72bb3212e743e2ee19e4a58d610f9d23b12f29cb29430273d874401624865ab5 (PyPI digest matches) | files.pythonhosted.org |

Page numbers for [JG1937] are the book's printed pages, read from the OCR text's running
heads. Before any of them becomes a fixture, it is to be confirmed on the page image
(C2185).
