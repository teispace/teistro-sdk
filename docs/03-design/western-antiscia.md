# Antiscia (the `western` module)

Status: `built`, 2026-10-03, written from the sources before any code.
Every step is built: `sdk.chart().antiscia`, held to Lilly's p. 181
table and to a recast of George V, measured over the corpus's births in
[`antiscia-measured.md`](antiscia-measured.md), and crossing to every
binding as `chart.antiscia`, with parity; and the antiscia across two
charts, George V against Queen Mary, as `sdk.chart().synastry_antiscia`
and every binding's `chart.synastryAntiscia`.

[`western-declinations.md`](western-declinations.md) reads two bodies
the same distance from the equator. Lilly reads the same equality along
the ecliptic: two degrees equally far from a solstice, which the Sun
passes on days of equal length. This page decides how a chart's
antiscia are read, and which pairs stand in one.

## What the sources decide

**William Lilly**, *Christian Astrology* (1647), is the rank 1 text. It
was read on the Wellcome scan (`b30338724`), page images where the OCR
fails.

- **The antiscion** (pp. 90–91): "The Antiscion Signes are those, which
  are of the same vertue and are equally distant from the first degree
  of the two Tropick Signes" (Cancer and Capricorn), "in which degrees
  whilest the Sun is, the dayes and nights are of equall length". His
  example: the Sun in the tenth degree of Taurus "hath his Antiscion"
  in the twentieth of Leo. To find the degree, subtract the planet's
  from 30 in the paired sign: Saturn in 20°35′ Leo has his antiscion
  in 9°25′ Taurus.
- **The contrantiscion** (p. 92): "the Signe and degree opposite to
  that place". The antiscions "of the good Planets we think are equall
  to a sextile or trine"; the contrantiscions are "of the nature of a
  square or opposition".
- **The use** (pp. 165, 181): Lilly prints a table of the seven
  planets' antiscions and contrantiscions beside a figure (p. 181, "If
  the Querent should be Rich"), and reads one when it falls "upon the
  very degree" of a cusp or a planet. In that figure "none of them fell
  exactly", "onely I observe the Contrantiscion of Saturn fals neer to
  the degree of" Jupiter, and he reads the two as "disturbed".

So an antiscion is the reflection of a degree about the solstitial
axis, at 180° less its longitude, and a contrantiscion the reflection
about the equinoctial axis, at 360° less it. Two planets stand in
antiscion when one's antiscion falls on the other, which is the same as
the other's falling on the first: their longitudes sum to 180°, or to
0° for a contrantiscion.

**The measure.** Lilly's p. 181 table gives each planet's antiscion to
the minute, so his planets' places follow from it: Saturn in 15°19′
Sagittarius, Jupiter in 17°31′ Cancer (in his exaltation, as the text
says), and the Sun, Mercury, the Moon and Venus in Leo. Of all
twenty-one pairs, only one stands within 6° of either relation:
Saturn's contrantiscion on Jupiter, 2°50′ away, the one Lilly reads.

## Decisions

1. **An antiscion is read in the tropical zodiac.** It is a reflection
   about the solstices, so a sidereal chart's planets are reflected from
   their tropical longitude, which every chart carries, and the
   reflections are tropical degrees. A sidereal chart gives the same
   antiscia as a tropical one of the same birth, as it gives the same
   declinations.
2. **A pair, not a direction.** One planet's antiscion on a second is
   the second's on the first, so a row names the pair once, the earlier
   planet first, as the parallels do. It carries `contrary`: true for
   the contrantiscion, the reflection across the equinoxes. Like the
   parallels, a row reports the gap and the orb, and no applying.
3. **The orb is Lilly's moieties unless the caller says (C244).** Lilly
   names no orb for an antiscion; he reads it "upon the very degree" and
   reads one "neer" at 2°50′. His aspects take the moieties (C240), and
   under them the p. 181 figure holds exactly the one pair he reads. The
   orb is the aspect table's `OrbModel`, read at the conjunction, since
   an antiscion falling "upon" a planet is a conjunction with its
   reflection. Leo's orbs, a caller's moieties or a caller's own orb for
   the conjunction replace the default as they do for the aspects.
4. **The table of points.** A chart's antiscia list each planet's
   antiscion and contrantiscion, as Lilly's p. 181 table does, and the
   outer three when placed. A reader who wants a planet's antiscion on a
   cusp reads it off this table.
5. **A planet the orbs leave out is listed, not refused.** Lilly's
   moieties give the outer three no orb. Refusing them would fail every
   chart that places them under the default, and dropping them would
   say "no pair" where the truth is "no orb". So they stand in the
   table of points, and the answer lists them as unpaired; a caller's
   moieties that give them an orb pair them.
6. **Across the boundary, as the parallels cross.** An `antiscia`
   record on the chart request (`{}` for the moieties, or `orbs` as the
   aspect table spells them) asks every chart of the batch. A row a
   chart counts its points and its pairs, and the two go ragged under
   it (sections 91 to 93). An unpaired planet crosses as a `paired` flag
   on its point, so a binding rebuilds the list without a second
   section.
7. **Across two charts, on the same engine.** A synastry's `antiscia`
   (the same record) pairs every planet of the chart with every planet
   of the partner's, the chart's first, so a planet may pair with its
   own namesake. The partner's planets are reflected once for the
   batch, and the pairs cross in sections of their own (94, a count a
   chart, and 95, ragged by it), empty when not asked, as the
   synastry's parallels do. The lagna does not join: the moieties give
   it no orb, and an antiscion on an angle waits with the cusps (below).

## Order of work

1. This page, and the crux C244.
2. `western::antiscia` and `sdk.chart().antiscia`. Lilly's p. 181 table
   is the test, every antiscion to the minute from his places, and the
   figure's one pair under the moieties.
3. A measured page over the corpus's births: how many pairs there are
   under the moieties and under Leo's orbs, and how many share their
   pair with a parallel of declination.
4. The boundary record and every binding, with parity.
5. The antiscia across two charts in a synastry, held to a recast of
   George V against Queen Mary.

## What building it found

- **Lilly's table reads back to the minute.** His seven printed
  antiscions give the figure's places, and the SDK's reflections of
  them return his table exactly; under the moieties the figure holds
  the one pair he reads and no other.
- **George V holds one pair.** Against the Moshier recast, every
  antiscion agrees to 0.01°, and under the moieties Mars's antiscion
  stands 5.94° from Mercury, inside their 7¼°. A sidereal chart of the
  same birth gives the same reflections to 1e-6° and the same pairs.
- **An antiscion is not a parallel, even at 1°.** Read at the
  parallel's own orb over the corpus, 8 of the 33 pairs stand in
  parallel too. With no latitude every one would, since a
  degree of longitude moves a declination by at most 0.4°; the planets'
  latitudes part the rest. So the two tables answer different
  questions, and neither is derived from the other.
- **George V and Queen Mary hold seven pairs across.** Under the
  moieties his Saturn's antiscion falls 0.09° from her Jupiter, and six
  more pairs stand within the orb out to his Mars on her Sun at 10.80°,
  one of them a contrantiscion (his Mars on her Saturn). All seven agree
  with the Moshier recast to 0.01°, and a batch's reading equals the
  direct one.
- **One pair's columns, two tables.** A chart's own pairs and the pairs
  across carry the same five columns, so the boundary writes both from
  one column set and every binding decodes both with one function; only
  the reading of `first` differs, the earlier in catalogue order in one
  and the chart's in the other.
- **An unpaired planet is a column, not a section.** The answer's
  `unpaired` list is a fact about each point, so it crosses as a flag
  beside the point's reflections; every binding reads the list back in
  the order the points stand.
- **One helper, four copies.** The measured pages' percentage helper
  had been written four times. The new page, the declinations page and
  the stations page print it alike and now share one copy, their pages
  unchanged; the Muntha page spells an empty share `--` and keeps its
  own.

## What this does not decide

- **The cusps** were decided with the Western houses
  ([`western-houses.md`](western-houses.md), C251): a reflection upon a
  cusp's very degree, its sign and whole degree, in Lilly's
  Regiomontanus unless the request names another division. A synastry
  refuses `cusps`, which are one chart's.
- **Benefic and malefic.** Lilly weighs an antiscion of a good planet as
  a sextile or trine and a contrantiscion as a square or opposition.
  The row says which relation holds and leaves the weighing to the
  reader, as the aspect table does.
