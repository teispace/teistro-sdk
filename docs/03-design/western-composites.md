# Composite and Davison charts (the `western` module)

Status: `designed`, 2026-10-03, written from the sources before any code.

[`western-synastry.md`](western-synastry.md) reads two charts against
each other. A relationship chart makes **one** chart of two: the
composite from the midpoints of their positions, the Davison from the
midpoint of their births. This page decides how each is built.

## What the sources decide

Both methods are modern, and the books that introduced them are in
copyright and not to hand: John Townley's introduction to composites
(1973) and Ronald Davison's *Synastry* (1977; archive.org holds the
1983 edition, `synastry00rona`, for lending only). So the definitions
are read at rank 2, from the originator's own account and a documented
open implementation, and each choice they leave open is a crux.

- **John Townley**, the composite's originator, defines it on his own
  site ("Composites vs. Davison time charts", astrococktail.com): "Take
  the midpoints of each of A's and B's mutual planets (Sun with Sun,
  Moon with Moon, etc.), put them into a house wheel made up of mutual
  house cusp midpoints". The Davison chart "is simply a natal chart cast
  for the midpoint in time and place between them".
- **Astrolog** (Walter Pullen, GPL, read in the 5.30 source,
  `CastRelation` and `Midpoint`) builds both:
  - a composite planet is the midpoint of the two "closest to the
    positions in question", the near midpoint; its latitude and its
    speed are the means of the two;
  - a composite cusp is the near midpoint of the two cusps, and then
    any cusp more than 90° from where the midheaven puts it is turned
    by 180°, "to make sure we don't have any 180 degree errors in house
    cusp complement pairs, which may happen if the cusps are far
    apart". The midheaven itself is never turned;
  - the time-space midpoint is cast at the mean of the two instants, the
    mean of the two latitudes and the mean of the two longitudes, the
    longitude the shorter way round when they are more than 180° apart.

**The measure.** Townley's worked Davison, for two births in Shawnee,
Oklahoma and Los Angeles, stands "about 85 miles SW of Albuquerque".
The arithmetic mean of the two places stands 60 miles south-west of
Albuquerque (bearing 243°); the great-circle midpoint stands 58 miles
due west of it (275°). The direction is Astrolog's mean. His printed
time does not follow from the births he gives under either reading, so
the example is read for its place alone.

## Decisions

1. **A composite planet is the near midpoint (Townley, Astrolog).** Each
   planet of one chart is paired with the same planet of the other, and
   placed at the midpoint of the shorter arc between them. Two planets
   exactly opposite have no shorter arc; the midpoint is then taken 90°
   ahead of the first chart's, so the answer is fixed. Its speed is the
   mean of the two, so a composite's aspect table reads applying and
   separating, as Astrolog's does.
2. **The angles are midpoints, the lagna kept east of the midheaven
   (C247).** The composite midheaven is the near midpoint of the two.
   The composite lagna is the near midpoint of the two lagnas, turned by
   180° when it falls more than 90° from the midheaven's eastern
   quadrature (the midheaven plus 90°), as Astrolog turns every cusp:
   a natal lagna always stands in the half of the zodiac after its
   midheaven, and a midpoint of two far-apart lagnas need not. A row
   says when it was turned. Deriving the lagna from the composite
   midheaven at some latitude is a later writer's method, not read
   here, so it is not offered.
3. **The two charts place the same planets.** A composite pairs a planet
   with itself, so two charts that place different sets (one with the
   outer planets, one without) are refused rather than read for what
   they share.
4. **In the tropical zodiac by default, or the charts' own (C241).** The
   synastry's `SynastryZodiac` is the composite's: tropical by default,
   or each chart's own, which refuses two charts founded in different
   zodiacs. A constant ayanamsha moves every midpoint alike, so the
   choice matters only between charts whose ayanamshas differ, that is,
   births of different years.
5. **A Davison chart is a birth, and the caller founds it (C248).** The
   midpoint of two births is a third birth: the mean of their instants
   on the UTC scale; the mean of their latitudes and their altitudes;
   the mean of their longitudes the shorter way round (Astrolog, and
   Townley's example south-west of Albuquerque); and the mean of their
   clocks, to the second, which only names the civil day. It is returned
   as a `Partner`, so any chart request, under any profile, founds it,
   and every reading a natal chart takes, a Davison takes.

## Order of work

1. This page, and the cruxes C247 and C248.
2. `western::composite` and `sdk.chart().composite`, and
   `Partner::davison`. A recast of George V and Queen Mary (Leo, p.
   130) is the test for both: every composite planet against the
   midpoints of the recast's, and the Davison chart against a recast
   cast at the midpoint birth.
3. A measured page over the corpus's pairs of births: how often the
   composite lagna is turned (C247), and how far the composite and the
   Davison charts' planets stand apart.
4. The boundary records and every binding, with parity.

## What this does not decide

- **The composite's cusps.** Townley sets the composite in a wheel of
  cusp midpoints, and Astrolog turns each by the midheaven. The
  foundation carries its bhavas in the profile's chalit, not a Western
  house system, so the cusps wait on one; the lagna and midheaven are
  the two every system shares.
- **Weighted and multi-person composites.** Astrolog weights a composite
  or a Davison chart toward one of the two, and composes more than two
  charts. No source read here uses either.
- **The derived lagna.** A composite lagna computed from the composite
  midheaven at a latitude is reported in later writers, whose texts are
  not read here.
