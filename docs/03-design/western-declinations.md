# Declinations and parallels (the `western` module)

Status: `building`, 2026-10-03, written from the sources before any code.
Steps 2 and 3 are built: `sdk.chart().declinations` and
`sdk.chart().parallels`, held to George V against a recast, and
measured over the corpus's births in
[`declinations-measured.md`](declinations-measured.md).

[`western-aspects.md`](western-aspects.md) reads the aspects along the
ecliptic. Leo adds one aspect that is not an angle there: two bodies the
same distance from the equator. This page decides how a chart's
declinations are read, and which pairs stand in parallel.

## What the sources decide

**Alan Leo**, *How to Judge a Nativity* (1928 edition), is the rank 1
text, as it is for the aspect table. It was read on the scan
`howtojudgenativi00leoa`.

- **The parallel** (p. 42): "To the aspects just enumerated may be added
  the Parallel of Declination, when the two bodies are the same
  distance, whether north or south, from the equator; its nature is
  variable, like the conjunction." It "is generally regarded as the
  same in nature as the conjunction"; when two planets are in parallel
  and in an aspect at once, the aspect decides (p. 43).
- **The orb** (p. 47): "For the parallel of declination 1°."
- **The ascendant's declination** (p. 141, on the hyleg): "The
  declination of the cusp of the Ascendant is that which the Sun would
  have if it were in the same degree of the zodiac."

So a parallel is an equality of distance from the equator, on the same
side or on opposite sides, held to within 1°. An ecliptic point such as
an angle has the declination of that degree with no latitude.

**The measure.** A recast of George V's birth (Leo, p. 130) with the
Moshier ephemeris (pyswisseph, rank 3) gives each planet's declination
two ways: as the ephemeris rotates it to the equator, and from its
ecliptic longitude, its latitude and the true obliquity by
`sin δ = sin β cos ε + cos β sin ε sin λ`. The two agree to 1e-11″ for
all ten planets. A chart already carries both inputs, so the SDK does
not need to ask its ephemeris for anything new.

## Decisions

1. **A declination is read from what the chart placed.** It comes from
   each planet's tropical longitude, its ecliptic latitude and the true
   obliquity at the chart's instant. It does not depend on the chart's
   zodiac: a sidereal chart's planets carry their tropical longitude
   too. The formula is one function, `core::angle::declination_deg`,
   which both `western` and the strength crate reach. The Shadbala's
   ayana bala reads the same declination under its `True` kranti and
   now calls it rather than repeating the formula.
2. **The angles have declinations too.** The lagna and the midheaven
   are degrees of the ecliptic, so each takes the declination the Sun
   would have there, at no latitude (Leo, p. 141). A chart's
   declinations list the planets, then the two angles.
3. **A parallel is either side of the equator (C243).** Leo's "whether
   north or south" makes the contra-parallel a parallel. Later authors
   split the two and read the contra-parallel as an opposition. So a
   row carries `contrary`: true when the two stand on opposite sides.
   A reader who splits them filters by it, and no second aspect is
   needed.
4. **The orb is 1° unless the caller says.** Leo's 1° is the default.
   `orb_deg` replaces it, for every pair alike, since Leo widens the
   parallel for no planet.
5. **A table of its own, beside the aspects.** A parallel is not an
   angle along the ecliptic, so it is not a `WesternAspect` member. Its
   row measures the difference between two distances from the
   equator, not an arc between two longitudes. The pairs are the
   aspect table's: the seven planets and, when placed, the outer three,
   each pair once and closest first. Where Leo says the aspect decides
   (p. 43), the reader holds both tables; the measured page counts how
   often the two coincide.
6. **No applying.** Leo gives a parallel no motion, and a chart does not
   carry a declination's speed. A row reports the gap and the orb only.
7. **Across the boundary, as the aspect table crosses.** A `parallels`
   record on the chart request (`{"orbDeg": 1}`) asks every chart of
   the batch. One section carries each chart's declinations, and the
   parallels go ragged under a count section, as the Western aspects
   do. Synastry's parallels are a later step, read on the same engine
   across two charts.

## Order of work

1. This page, and the crux C243.
2. The declination in `western` on the astronomy's rotation, the
   ayana bala on the same call, `western::parallels`, and
   `sdk.chart().declinations` and `sdk.chart().parallels`. George V is
   the test: every declination against the recast, Mercury parallel to
   Venus and Neptune contra-parallel to the Moon.
3. A measured page over the corpus's births: how many parallels and
   contra-parallels there are, and how many coincide with an aspect
   under Leo's orbs.
4. The boundary record and every binding, with parity.
5. Parallels across two charts in a synastry.

## What building it found

- **The formula is the ephemeris's, exactly, and the SDK's agrees.**
  George V's ten declinations and both angles match the Moshier recast
  to 0.01°. A sidereal chart of the same birth gives the same
  declinations to 1e-6°, since every planet carries its tropical
  longitude.
- **The ayana bala's copy kept every bit.** Moving it onto the shared
  function left `shadbala-measured.md` unchanged.
- **Over the corpus.** The 55 births hold 217 parallels within 1°,
  138 on the same side and 79 contrary. About half of each (50.7% and
  54.4%) share their pair with one of Leo's nine aspects, where he
  reads the aspect instead (p. 43).
- **A Sun past the obliquity is the frame, not a defect.** One birth's
  Sun stands 7.65″ past the obliquity. Under the profile's topocentric
  frame the Sun has a latitude of up to its 8.8″ parallax, and the
  measured page now gates that bound. Venus, the Moon, Mars and
  Mercury pass the obliquity by degrees in 8 to 15 births each.

## What this does not decide

- **The declination's speed and the out-of-bounds planets.** A planet
  whose declination passes the obliquity is a modern reading with no
  rank 1 text read yet. The declinations are reported, so a reader can
  compare them with the obliquity.
- **Leo's equal distances** (p. 48). Two planets equally distant from a
  third act "much the same as a parallel". That is a midpoint, and it
  waits on a page of its own.
- **Antiscia.** Lilly's reflections about the solstices are a different
  equality (of declination by construction, along the ecliptic), read
  on their own page when his text is.
