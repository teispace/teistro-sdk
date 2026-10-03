# Progressions and directions (the `western` module, step 1)

Status: `building`, 2026-10-03 — written from Leo's text before any code,
its worked figures recast with pyswisseph's Moshier series. Steps 2 to
4 are built: the measures in `crates/western`, the charts and the
contacts in the SDK's chart area.

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
   October**, about 4.40 UT. That is a day and a half before Leo's 22nd
   (C236). The Julian year gives the 21st too, at about 13h.
5. **The progressed Moon's year** (p. 41). Leo's lunar list for his
   forty-seventh year gives each contact of the progressed Moon to his
   radical points by month:
   - sesquiquadrate Mercury, October 1906;
   - quincunx Jupiter, January 1907;
   - quincunx Uranus, April;
   - sesquiquadrate Saturn, June;
   - quincunx the Sun, August;
   - trine the midheaven, September;
   - opposition Venus, November.

   Five of these are to the seven planets, so the contacts must find
   them in their months. Uranus and the midheaven are not natal points
   of the hit list yet.
6. **The Naibod measure** (p. 261). 20° 15′ is 20 years, 198 days and
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

- **`ChartArea::progressed(&birth, at, &ProgressionRequest, &ChartRequest)`** answers
  the progressed instant, the founded chart at that instant, and the
  angles by the chosen method. An instant's chart has angles of its own,
  and those are the quotidian angles, so the default angles are computed
  rather than read off the chart.
- **`ChartArea::directed(&birth, at, &DirectionArc, &ChartRequest)`**
  answers the arc and every point of the birth chart moved by it. The
  arc is the Sun's (`DirectionArc::Solar`, the solar arc) or a measure's
  (`DirectionArc::Measure`, Naibod's or a degree a year). The chart
  request says where the later chart is founded.
- **`ChartArea::progressed_contacts(&birth, from, to, &ContactRequest)`**
  answers each aspect a progressed planet makes to a radical point, and
  the instant of life it falls due. This is Appendix V. It is the transit
  hit list run on the window of sky that the progression matches to
  `[from, to]`, with each exact instant mapped back through the measure.
  The search, the aspect lattices and the order of the answer are the
  hit list's. A `ContactRequest` names:
  - the progression;
  - the progressed planets, the seven by default;
  - the radical points, the seven and the ascendant by default;
  - the aspects. The default is Leo's table of aspects (p. 48), whose
    columns are 30°, 45°, 60°, 90°, 120°, 135°, 150° and 180°, with the
    conjunction. He warns that "semi-squares and sesquiquadrates" are
    the ones most easily missed (p. 41).

  The hit list took aspects only at multiples of 30°, which C145 decided
  for transits, where no text sets a degree aspect. Leo's contact is a
  sesquiquadrate, so the hit list now takes any whole degree from 0 to
  180. Its lattice already steps by the angles' greatest common divisor.
  The orb stays under 15° and must also be under half that step, which
  is the condition that no two windows meet. Every request valid before
  is valid now.

  Two of Leo's contacts are not covered: those to the radical midheaven,
  which is not yet a natal point of the hit list, and the parallels of
  declination. Contacts between two progressed planets are not covered
  either.

The arithmetic goes in its own functions, so a consumer can use one
without the others:
- `progressed_instant(birth, at, rate, measure)`;
- `life_at(birth, instant, rate, measure)`, the inverse;
- `arc_to_years` and `years_to_arc` for each arc measure.

### Across the boundary (step 5)

A batch's chart request gains one record, `progressions_json`, the way
the perfection crossed. The batch applies it to every chart, because a
consumer asks one question of many births: where each one stands at an
instant of life, or what falls due in a window. Everything in the record
is optional, and a section the record does not ask for costs nothing.

- `at`, a UTC Julian day: the instant of life the progressed chart and
  the direction are read for;
- `rate`, `year` and `angles`: the measure (C236, C238) and the angle
  method (C237), spelled as their Rust members, with Leo's defaults;
- `direction`: `"SOLAR"` or an arc measure (`"NAIBOD"`, `"PTOLEMY"`, or
  `{"perYear": degrees}`);
- `contacts`: `{from, to, grahas, points, aspects}`, spelled as the hit
  list spells them.

Three sections answer it:
- `progressed`, a row a chart: the instants of life and sky, the
  progressed meridian, ascendant and midheaven. Its planets come in
  `progressed_grahas`, a row a graha with the longitude in the chart's
  zodiac, the tropical longitude and the speed. The full founded chart
  does not cross, since a consumer who needs it founds the instant of
  sky as an ordinary chart.
- `directed`, a row a chart: the arc and the directed angles. Its
  planets come in `directed_grahas`.
- `progressed_contacts`, ragged by the chart's contact count: the
  instants of life and sky, the planet, the point, the angle and the
  motion, as the hit list's columns spell them.

Node, wasm, Python and Dart read these as `chart.progressed`,
`chart.directed` and `chart.progressedContacts`. The parity runners
print Leo's birth under both year measures.

## What building it found

- **Leo's rule is a closed form.** Sidereal time at Greenwich noon comes
  round once a tropical year, so his arithmetic (pp. 304–305) has a
  closed form. `N` is the noon of the day of birth, `δ` the birth's
  offset from it, and the instant of sky is `k` whole days and `f` of a
  day past `N`. Then life = `N + k·Y + (f − δ)·(Y + 1)`. Each ephemeris
  noon measures to one date, and the solar hours after it count as
  sidereal hours, so a day of sky measures a year and a day.
  - The consequence is that the last 3m 56s before each ephemeris noon
    measure to dates the next noon reaches too. The inverse answers the
    later instant of sky.
  - It reproduces his "November 10th" for noon within 0.01 of a day of
    his printed sidereal times.
  - It reproduces his 22 October within the day. His tables put the
    contact 0.23 of a day past noon, and the closed form puts it at 0.15.
- **Birth by the rule.** The rule puts birth on the next day's noon for a
  birth before noon, and on its own day's noon for one after. That is
  the year and a day again. The day of birth is the Greenwich civil day,
  and longitude does not enter, which agrees with Leo's London figure.
  Keeping the birth's local sidereal time instead would move a New York
  birth's anniversary by about 75 days.
- **His p. 35 date is his approximate reckoning.** "5.49 a.m. 22/9/'60
  measures to the year commencing August 8th, 1906" agrees with the
  tropical year. His sidereal-time rule gives the 9th, the same
  one-day correction he makes for noon on p. 305.
- **The Julian year** gives his contact at 21 October about 13h, a day
  and a half after the tropical year's 4.40 and still not his 22nd.
- **Naibod's table** (p. 262) is the rate to within a day in the degree
  rows, rounding some rows down and others up. It is within an hour in
  most minute rows; the 40′ row is 1.03 hours from the rate.
- **Leo's figures reproduce through the SDK's own ephemeris.** On the
  built-in ephemeris his radical Mercury and both noon Moons agree to
  the minute, the Moon is 135° from Mercury at his 10.40 a.m. to within
  two minutes, and the progressed map's meridian is his 5h 54m 16s to
  within two seconds. The remainder is the equation of the equinoxes:
  the SDK's sidereal time is the apparent one, his the mean.
- **The quotidian is Leo's map on a whole day.** At the birth's own clock
  time on a progressed day, the chart's own meridian is the birth's
  turned by whole days of the mean Sun in right ascension, so the two
  methods agree there and part between. A test that compares the
  methods has to ask off the birth's clock time.
- **The midheaven is the ecliptic point at the meridian's right
  ascension**, not the ecliptic projection of the equator's point. The
  first build projected it, and the solar-arc test caught the
  difference. The house kernel's `circle_point` already answered it, so
  `Obliquity::new` in `teistro-astro` became public to make that
  function callable outside its crate.
- **A longitude method converts with the progressed obliquity.** The
  moved midheaven goes back to the equator with the obliquity the
  progressed angles are built on. With the birth's obliquity, the
  midheaven read back differs from the birth's plus the arc by about
  3e-6°.

- **The contacts fall where Leo dates them.** His Appendix V contact
  falls on 21 October 1906 under the tropical year and on the 22nd under
  his rule, its instant of sky within two minutes of his 10.40 a.m. In
  his lunar year (p. 41) the contacts to Mercury, Jupiter and Venus fall
  in his printed months. Those to Saturn (23 May) and the Sun (28 July)
  fall about a week before his June and August. He dates the list by
  counting a month for each degree the Moon passes. Measured from his
  first contact, the Moon has moved 3.2° to Jupiter, 7.4° to Saturn,
  9.7° to the Sun and 13.2° to Venus. Rounded, these give his months
  for Jupiter, the Sun and Venus, but Saturn's 7.4° gives May against
  his June. His counting cannot be rebuilt without his own radical
  places, so the test holds each contact to his month or the month
  before.
- **His solar list holds.** The progressed Sun and Mercury are each
  semisquare the radical Sun in his forty-seventh year: Mercury on 16
  October 1906, the Sun on 23 January 1907.
- **The search finds what his list leaves out.** The progressed Moon is
  sesquiquadrate the ascendant on 10 May 1907 and square the radical
  Moon on 10 August. His list names neither, and he does not say why.
- **The hit list rounded every line to 30°.** `aspect_hit` named a
  crossing by its nearest multiple of 30°, so a sesquiquadrate would
  have been reported as a trine or a quincunx. It now rounds to the
  whole degree, which every line of its lattice is.

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
   to tests 4 and 6 above. **Built.**
3. `ChartArea::progressed` and `directed`, held to tests 1 to 3.
   **Built.**
4. `progressed_contacts`, held to test 4 under both measures and to
   test 5's year. **Built.**
5. The boundary and every binding, with the parity gate.
6. A measured page over the corpus's births: each angle method's
   midheaven against the others by age, so the size of C237 is stated in
   degrees rather than argued.
