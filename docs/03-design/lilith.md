# The Black Moon Lilith: the lunar apogee in a chart

Status: `building`, 2026-10-17, written from the code and the references
before the chart layer changed. Step 2 is built: a chart asked
`ChartRequest::with_lilith(Apogee::Mean)` or `(Apogee::Osculating)`
places the apogee beside the nine in `ChartFoundation::lilith`, in the
same request to the ephemeris, and the mean apogee is held to Meeus's
worked example. The boundary, the bindings and the readers are steps 3
to 5.

The feature register lists the lunar apogee, mean and osculating, as P1
for the chart (`01-research/feature-universe/01-vedic-parashari-core.md`)
and the Black Moon Lilith among the Western points
(`15-western-modern.md`). The ephemeris side is already done: the port
names `Body::MeanApogee` and `Body::OsculatingApogee`, and the built-in
ephemeris computes both (`builtin-ephemeris-measured.md`,
`completion-measured.md`). No chart carried either, so nothing above the
port could read one. This page decides how a chart carries it.

**The name is later practice.** "Black Moon Lilith" is a modern Western
name for the lunar apogee, and no classical text read for this SDK uses
it. The astronomy is old and standard, and is what the code computes; the
name is the practice's, and the record is keyed by the astronomy (crux
C442).

## What the references define

**The mean apogee** is the far end of the Moon's mean line of apsides:
the mean perigee turned half a circle.

- *ELP2000-82B* (Chapront-Touzé and Chapront; the reader `elp82b.f`, CDS
  VI/79, which `crates/ephemeris-builtin/src/elp.rs` ports) carries the
  mean longitude of the perigee as its `W2` polynomial, 83°21′11.67475″
  at J2000 advancing 14 643 420.2632″ a century against the inertial
  equinox of J2000. The SDK's completion adds the precession, so of date
  it advances about 4069.01° a century.
- Jean Meeus, *Astronomical Algorithms* (2nd ed., 1998), ch. 47, gives
  the Moon's mean longitude L′ and mean anomaly M′ as polynomials from
  the later ELP 2000-85 fit. The mean anomaly counts from the perigee, so
  the mean perigee is L′ − M′. **Example 47.a** (1992 April 12, 0h TD)
  prints L′ = 134.290182° and M′ = 5.150833°: the mean apogee is
  309.139349°, and with the example's own nutation in longitude, Δψ =
  +0.004610°, 309.143959° of the true equinox of date. The chapter's
  rates, 481 267.881 234 21° and 477 198.867 505 5° a century, differ by
  4069.0137°: **40.690° a Julian year**, a turn in 8.85 years.

The two fits are not one polynomial. At Example 47.a's instant `W2` with
ELP's precession reads 309.139540°, 0.00019° (0.69″) from Meeus's.

**The osculating apogee** is the apogee of the two-body ellipse that
matches the Moon's geocentric position and velocity at the instant: the
eccentricity vector, reversed, at the distance `a(1 + e)`, the
semi-major axis from vis-viva. It needs the Earth-Moon mass parameter,
which neither theory carries; `GM_EARTH_MOON` in the built-in provider
is IERS Conventions 2010's `GM(Earth)` and DE440's `GM(Moon)`, checked
against the mass ratio they imply. The Sun swings it by tens of degrees
about the mean apogee over a month, so it is sometimes retrograde, which
the mean one never is.

## Decisions

1. **Beside the nine, not among them, and not yet a catalogue point
   (C438).** The port answers no graha for an apogee, and a port test
   holds that the apogees are the only bodies outside the catalogue's
   names; every reader of `ChartFoundation::grahas` counts nine. So
   Lilith is `ChartFoundation::lilith`, one `LilithPosition` with the
   fields a `GrahaPosition` has, left out of the document when not
   asked. Q38's rule decides the catalogue: a value earns a catalogue
   member when a consumer must *name* it, and nothing reads Lilith yet.
   The first reader that must name it (an aspect table, the wheel, a
   locale's name) adds the `point` row, with `UNNAMED` in
   `xtask/src/intl.rs` until a vetted Nepali name is read.
2. **Asked for, not by default, and asked by its apogee.**
   `ChartRequest::with_lilith(Apogee)` names which; a chart that does
   not ask is the chart it was, to the byte and to the hash. The founder
   takes `Carried { outer, lilith }` (`Founder::found_carrying`), and
   the input hash carries the apogee's key only when one is asked for.
   It is not a profile knob as `frame.node` is, because a settings field
   would change every chart's resolved settings and so its stamp.
3. **The mean apogee is `W2` + 180° (C439),** held to Meeus's Example
   47.a to a thousandth of a degree and to his 40.690° a year. The
   reference engine's mean apogee stands 265″ from it on average and
   417″ at worst over 1800 to 2400 (`completion-measured.md`), where the
   mean node beside it agrees within one arcsecond. That gap is not
   explained, and neither published polynomial reproduces the engine's
   value, so it is not mirrored. The SDK answers the mean apogee as a
   direction on the ecliptic, latitude zero; the engine places it on the
   orbit's plane. That is a convention, already recorded on the measured
   page.
4. **The osculating apogee is built, under its astronomical name
   (C440).** It is computed honestly from the ephemeris the SDK has, and
   ships as `Apogee::Osculating`. Later practice calls it the "true"
   Lilith and also reads an "interpolated" apogee smoothed out of it;
   that one is not built, and `Apogee` is non-exhaustive so it can join.
5. **The chart's own frame and centre (C441),** as the nine and as the
   outer planets (C239): the tropical zodiac of the true equinox of
   date, shifted by the chart's ayanamsha, from the chart's centre, in
   the same request. The mean apogee is a direction, so no observer
   displaces it. The osculating one is a place to the port
   (`Body::is_placed`), so a topocentric chart sees it moved by up to
   about a degree. The reference engine evidently does not: from its
   centre the two agree within 565″, from the place within 3575″ (mean
   107″ against 1953″), at the `standard` tier. The difference is the
   price of this decision, stated rather than hidden. The rival, always
   from the Earth's centre, costs a second request under a topocentric
   frame.
6. **Lilith is the apogee; the perigee is derived (C442).**
   `LilithPosition::perigee_deg` is the apogee turned half a circle,
   which both apogees are by definition. The asteroid (1181) Lilith,
   which needs an asteroid ephemeris the SDK does not carry, and the
   hypothetical second moon some practice also calls Lilith, are not
   computed.

## What building it found

- **Meeus sides with `W2`.** Evaluated side by side at Example 47.a's
  instant, Meeus's L′ − M′ and `W2` with ELP's own precession differ by
  0.00019°, and Meeus's polynomials reproduce every argument the example
  prints to the sixth decimal. So the reference engine's 265″ to 417″ is
  a third reading, not a second fit of the same one. The test asserts
  the chart's own mean apogee, through the SDK's precession and
  nutation, to a thousandth of a degree of the printed answer.
- **A provider that cannot place the apogee refuses the whole grid.**
  Validation refuses a request naming a body its capabilities lack, and
  that refusal does not say Lilith asked. The founder checks the
  capability first and refuses `UNSUPPORTED` on `lilith`, naming the
  body, with a hint; a cell the provider fails is refused the same way.
- **The nine do not move.** The apogee is the last cell of the grid,
  after every graha's, so the nine's cells and their speeds are the
  ones a chart without Lilith reads; a test holds them to the bit under
  the conformance profile and the Western one.

## What this does not decide

- **The readers.** No aspect table, transit search, progression or
  wheel reads Lilith yet, and a progressed chart or a return does not
  carry it even when the birth does. Each is a later step, and the step
  that first must name it adds the catalogue row (decision 1).
- **Meaning.** What Lilith signifies is practice this SDK has no source
  for; nothing interprets it.
- **The engine's mean apogee.** Why the reference engine's differs by
  up to 417″ is open on `completion-measured.md`.

## Order of work

1. This page, and C438 to C442.
2. The chart layer and the SDK: `Apogee`, `LilithPosition`, `Carried`,
   `Founder::found_carrying`, `ChartFoundation::lilith` and
   `ChartRequest::with_lilith`, held to Meeus's Example 47.a, to the
   apogee's motion and to the perigee in `crates/sdk/tests/lilith.rs`.
   **Built.**
3. The readers: the Western aspects, the transit hit list and the later
   charts accept Lilith where the birth carries it, with the catalogue's
   `point` row their names need.
4. The boundary and every binding, with the parity gate.
5. The wheel draws it, once a strict locale names it.
