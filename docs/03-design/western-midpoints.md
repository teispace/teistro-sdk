# Equal distances (the `western` module)

Status: `built`, 2026-10-03, written from the source before any code.
Every step is built: `sdk.chart().midpoints`, held to Leo's p. 47
configuration and to a recast of George V, measured over the corpus's
births in [`midpoints-measured.md`](midpoints-measured.md), and crossing
to every binding as `chart.midpoints`, with parity.

[`western-declinations.md`](western-declinations.md) reads two bodies
the same distance from the equator, and
[`western-antiscia.md`](western-antiscia.md) two the same distance from
a solstice. Leo reads a third equality: a planet the same distance from
two others along the zodiac, which later writers call a midpoint. This
page decides how a chart's equal distances are read, and which stand.

## What the source decides

**Alan Leo**, *How to Judge a Nativity* (1903 and later editions), is
the rank 1 text, read on the archive.org page images
(`howtojudgenativi00leoa`), pp. 47–48.

- **The configuration** (pp. 47–48): "If three planets are so arranged
  that two of them are equally distant from the third, the effect seems
  to be much the same as a parallel and is therefore good or bad
  according to the nature of the planets."
- **The limit** (p. 48): "Whether there is any limit to this kind of
  influence by position is uncertain."
- **An aspect governs** (p. 48): "if the three are in some recognised
  aspect to each other, good or bad, the effect will be according to
  the nature of that aspect."
- **The example** (p. 47, the paragraph before): "if the Sun is at 0°
  ♈ and the Moon at 10° ♎ the aspect is a wide opposition", "but if
  Mars were at 5° ♋ it would not only be in square to both but would
  render the opposition worse". Read off the page image, since the
  OCR drops the signs.
- **The parallel's orb** (p. 47): "For the parallel of declination 1°."

**The measure.** The Sun at 0° and the Moon at 190° are 170° apart the
short way, through Capricorn, and their midpoint that way is 5°
Capricorn (275°). Mars at 5° Cancer (95°) stands 95° from each, exactly
equally distant, on the point **opposite** that midpoint. So the
example holds only if both points of the axis count.

## Decisions

1. **The two points of the axis count alike (C246).** A third planet
   equally distant from two stands on the axis through their midpoint:
   at the midpoint of the shorter arc, or opposite it, on the longer
   arc's midpoint. Leo's only example stands on the second, so a row
   carries `far`, true when the third stands opposite the shorter arc's
   midpoint, and both are read.
2. **The orb is half the parallel's, measured from the axis (C245).**
   Leo names no orb for this, and calls the effect "much the same as a
   parallel", whose orb is 1° on the difference of two distances. A
   planet 0.5° from the axis has distances to the two that differ by
   1°, so the default orb is 0.5° from the nearer point of the axis. A
   caller sets any orb above 0° and up to 10°, the parallel's own
   limit.

   The orb is measured from the axis, not on the difference of the two
   arcs, because the difference fails on a close pair: two planets 0.4°
   apart are within 1° of equally distant from **every** third planet,
   since two arcs from one point can differ by no more than the two
   planets are apart. Measured from the axis, a close pair's equal
   distances are the planets conjunct or opposite it, as they should
   be. Near the axis the two measures agree: x from the axis is 2x on
   the difference.
3. **Any two planets, however far apart.** Leo leaves the limit
   "uncertain", so none is imposed, and every pair of planets is read.
   A row reports how far the third stands from each, so a reader who
   wants a limit applies one.
4. **The aspects are reported, not judged.** Where the three stand in a
   recognised aspect, Leo reads the aspect instead. The aspect table
   already answers that, so a row says only which planet is equally
   distant from which two, and leaves the weighing to the reader, as the
   antiscia's rows do.
5. **The chart's own longitudes.** An equal distance is a fact about
   arcs, which a constant ayanamsha does not move, so the chart's own
   longitudes are read, and a sidereal chart gives the same rows as a
   tropical one of the same birth, its axis points shifted by the
   ayanamsha.
6. **Planets, not angles.** Leo says "three planets". The seven are
   read, and the outer three when placed; the lagna and the midheaven
   are not, as no example reads them.
7. **A row.** `first` and `second`, the pair in catalogue order;
   `middle`, the planet equally distant from them; `far`;
   `distance_deg`, the mean of its two distances; `from_axis_deg`, how
   far it stands from the nearer point of the axis; and `orb_deg`.
   Rows come closest first.
8. **Across the boundary, as the parallels cross.** A `midpoints`
   record on the chart request (`{}` for the default, or `orbDeg`) asks
   every chart of the batch. A row a chart counts its equal distances,
   and they go ragged under it (sections 96 and 97).

9. **Across two charts, a pair of one and a planet of the other.** A
   synastry's `midpoints` reads each chart's planets on the axes of the
   partner's pairs, and the partner's planets on the axes of the
   chart's, under the same orb and in the synastry's zodiac. A row says
   whose pair it is (`partners_pair`). A pair of one planet from each
   chart is not read: its midpoint is the composite's place for two
   different planets, which no source here reads, and the composite
   reads the same planet's two places.

## Order of work

1. This page, and the cruxes C245 and C246.
2. `western::midpoints` and `sdk.chart().midpoints`. Leo's p. 47
   configuration is the test, Mars standing on the far point.
3. A measured page over the corpus's births: how many equal distances
   stand under the default orb, how many on the far point, and how many
   of their three planets also stand in an aspect, which Leo reads
   instead.
4. The boundary record and every binding, with parity.

## What building it found

- **George V holds one under the default.** Pluto stands 0.05° from the
  far point of the Moon and Jupiter, 137.7° from each. At 1.5° there
  are eight, and all eight agree with the Moshier recast to 0.01°; a
  sidereal chart of the birth gives the same rows.
- **The far point is a third of them.** Over the corpus, 40 of the 120
  equal distances under the default stand on the far point, so reading
  only the shorter arc's midpoint would lose a third (C246).
- **An aspect is beside about half.** 66 of the 120 have the planet
  between in an aspect to one of the two under Leo's table, where he
  reads the aspect instead (p. 48). The rest stand in none, which is
  what the equal distance adds to the aspect table.
- **One pattern for a table a chart.** The aspect table, the antiscia
  and now the equal distances each read one table a chart and name a
  refusal under their record and the chart; the boundary had written
  that loop twice, so the third made it one function.
- **A planet at a longitude is one type.** The antiscia's input was a
  planet and its tropical longitude, which the equal distances need
  too, so it became `PlanetAt`, beside the aspect table's `Placed`.

## What this does not decide

- **The angles.** Modern midpoint work reads the lagna and the
  midheaven as points among the planets. No rank 1 text read here
  does, so they wait on one.
- **Midpoint trees and the 90° dial.** The modern school reads the
  midpoint axis modulo 45° or 90°. That is a later method with no
  public-domain text read, and gets its own page if one is found.
