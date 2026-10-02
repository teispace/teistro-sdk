# Progressions and directions (the `western` module, step 1)

Status: `design`, 2026-10-03 — written from Leo's text before any code,
its worked figures recast with pyswisseph's Moshier series.

A progression reads the sky some days after birth as the native's life
some years after it: the chart for the forty-sixth day is read as the
forty-sixth year. A direction moves every point of the birth chart by
one arc, the arc the Sun has moved in that time. Neither needs new
astronomy. Both are charts at other instants, or the birth chart turned
by an angle, so the work is in the doctrine: how a day is matched to a
year, and how the angles move.

## What the source decides

**Alan Leo**, *The Progressed Horoscope* (1906), is the rank 1 text. It
is the book that named the method, and it is public domain. It was read
in the Internet Archive scan `bub_gb_OY7ZLx4jQmcC` (PDF page = printed
page + 12), every figure below checked on the page image.

- **The measure.** "One day measures a year" (Appendix IV, p. 303).
  Leo lists the measure's varieties there: the day as "a mean day", which
  "is the method usually followed"; as "a true solar day", the map then
  calculated "for the apparent time of birth"; and as a "mundane day",
  ended by the Sun's return to its place in the semi-arc.
- **The progressed map.** A map "for exact birth-time on equivalent day
  after birth", one day for one year (Appendix I, p. 295, quoting a 1903
  paper). The planets' places between two such maps fall between the two
  birthdays they measure to (p. 35).
- **The progressed angles.** The worked map on p. 35 is cast for the
  birth's clock time on the forty-sixth day, at sidereal time 5h 54m 16s.
  That is the birth's 2h 52m 54s plus forty-six days of 3m 56.555s, so
  his midheaven moves by the mean Sun's motion in right ascension, about
  a degree a year (p. 296: one degree "gives the 'progressed M.C.'").
- **The arc measures.** Two ways of turning an arc into years (pp.
  260–261): Ptolemy's, a degree for a year, and Naibod's, "the mean daily
  motion of the Sun to represent one year of life". His example: 20° 15′
  gives 20 years, 198 days and 16 hours by Naibod.
- **The lunar measures.** His closing table (p. 311) adds the
  "Progressed Lunar" horoscope, one month for one year, and one day for
  one month. The month is the "synodic month, 29·53059 days" (p. 295).
  Today these are called the minor and the tertiary progression.
- **The dated contact.** Appendix V (pp. 304–305) works one contact to
  the day: the progressed Moon's sesquiquadrate to the radical Mercury.

## The acceptance tests

Leo's own nativity is the fixture: London, 7 August 1860, 5.49 a.m.
His sidereal time at birth, 2h 52m 54s, and at Greenwich noon that day,
9h 4m 55s (p. 305), both agree with the recast to the second.

1. **The progressed map's sidereal time** (p. 35). The chart for 5.49
   a.m. on 22 September 1860 is cast at 5h 54m 16s, exactly as printed.
2. **The radical Mercury and the Moon** (p. 304). Mercury at Leo 20°
   11′. The Moon at Greenwich noon on 21 September at Sagittarius 22°
   58′, and on 22 September at Capricorn 5° 54′. All three agree to the
   minute.
3. **The contact** (p. 304). The Moon reaches Capricorn 5° 11′, 135°
   from Mercury, at 10.40 a.m. on 22 September 1860. The recast puts it
   at 10.39.
4. **The day it measures to** (p. 305). Leo prints two answers:
   - **19 October 1906**, by counting months and days;
   - **22 October 1906**, by sidereal time, which he says is the more
     accurate. The recast's noon sidereal time on that day is 14h 0m 1s
     against his 14h 0m 56s.

   A day measured as a tropical year, the modern default, gives **21
   October**, about 00.20 UT. That is a day and a half before Leo's 22nd
   (C236).
5. **The Naibod measure** (p. 261). 20° 15′ is 20 years, 198 days and
   16 hours: 0.985647° a year, and 365.2422 days to the year.

## The design

A new crate, `crates/western`, holds the pure arithmetic. The SDK's
chart area founds the charts. Rust comes first; the boundary and the
bindings follow the same way the perfection crossed.

- **`Progression`** — the rate:
  - `Secondary`, a day for a year;
  - `Tertiary`, a day for a month;
  - `Minor`, a month for a year;
  - `Rate { ephemeris_days, life_days }`, for any other rate.
- **`YearMeasure`** — how much calendar time one unit of life is:
  - `Tropical`, 365.24219 days, the default;
  - `Julian`, 365.25 days;
  - `NoonSiderealTime`, Leo's Appendix V rule, which is what reproduces
    his printed 22 October.
- **`AngleMethod`** — how the angles move:
  - `NaibodRightAscension`, Leo's p. 35 map and the default;
  - `SolarArcLongitude`;
  - `SolarArcRightAscension`;
  - `NaibodLongitude`;
  - `Quotidian`, the chart at the progressed instant itself, which
    turns a full circle and a degree each year (Leo's figure A′).

The calls:

- **`ChartArea::progressed(&birth, at, &ProgressionRequest)`** answers
  the progressed instant, the founded chart at that instant, and the
  angles by the chosen method. An instant's chart has angles of its own,
  and those are the quotidian angles, so the default angles are computed
  rather than read off the chart.
- **`ChartArea::directed(&birth, at, &DirectionRequest)`** answers the
  arc and every point of the birth chart moved by it. The arc is the
  Sun's (solar arc) or the measure's (Naibod or a degree a year).
- **`ChartArea::progressed_contacts(&birth, from, to, &request)`** answers
  each aspect a progressed point makes to a radical one, and the day it
  falls due. This is Appendix V. It reuses the crossing search the
  returns use, run on ephemeris time and mapped back through the
  measure.

The arithmetic goes in its own functions, so a consumer can use one
without the others:
- `progressed_instant(birth, at, rate, measure)`;
- `life_at(birth, instant, rate, measure)`, the inverse;
- `arc_to_years` and `years_to_arc` for each arc measure.

## What is not decided

- **C236, the year measure.** Leo's printed day is the sidereal-time
  rule's. In effect it counts the hours after an ephemeris noon in
  sidereal time, so 22h 40m becomes 22h 44m, a year and a day. The
  tropical year is what every modern implementation uses, and it misses
  his day by a day and a half. Both ship as knobs, with the tropical year
  as the default. The test asserts each answer against its own rule.
- **C237, the progressed angles.** Leo's own map advances the midheaven
  by the mean Sun in right ascension. The solar arc, in longitude or in
  right ascension, is the twentieth-century habit. No text weighs one
  against the other here, so every method is a knob and Leo's is the
  default.
- **C238, the tertiary month.** Leo's month is the synodic one (p. 295),
  for the month-for-a-year measure. For the day-for-a-month measure his
  table names no month. Later authors use the sidereal month, 27.32166
  days. `Tertiary` takes the month as a parameter and defaults to Leo's
  synodic month until a source for the other is read.
- **Converse progressions** run backwards from birth. They are the same
  arithmetic with a negative interval, and Leo discusses them (p. 296)
  without working one. They are allowed and not tested until a figure
  is found.

## Order of work

1. This page, and the crux rows C236 to C238.
2. `crates/western`: the rate, the measure and the arc arithmetic, held
   to tests 1 and 5 above.
3. `ChartArea::progressed` and `directed`, held to tests 2 and 3.
4. `progressed_contacts`, held to test 4 under both measures.
5. The boundary and every binding, with the parity gate.
6. A measured page over the corpus's births: each angle method's
   midheaven against the others by age, so the size of C237 is stated in
   degrees rather than argued.
