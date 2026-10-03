# The Western aspects and their orbs (the `western` module)

Status: `design`, 2026-10-03, written from the sources before any code.

A chart's drishti are the Vedic relations, which are read by sign and
house and have no orb (`aspect-and-drishti.md`). The Western aspect is a
different question. Two bodies stand at an angle, the angle is one of a
listed set, and the gap from exact is inside an orb. Lilly's
perfection reads the Ptolemaic five with his moieties
(`hellenistic-perfection.md`), and a progression's contacts read exact
angles with no orb at all (`western-progressions.md`). Nothing yet
answers the plain question a Western reader asks first: *which aspects
does this chart hold?* Synastry and the composite charts ask the same
question of two charts, so they wait on this one.

## What the sources decide

**Alan Leo**, *How to Judge a Nativity* (1928 edition), chapter "The
Aspects", pp. 43–47, is the rank 1 text for the set and the orbs. It
was read in the Internet Archive scan `howtojudgenativi00leoa`.

- **The set.** The good aspects divide the circle by three: the trine
  (120°), the sextile (60°) and the semi-sextile (30°). The evil ones
  halve or quarter it: the opposition (180°), the square (90°), the
  semi-square (45°) and the sesquiquadrate (135°) (p. 43). The quincunx
  (150°) has an orb on p. 47. The conjunction and the parallel of
  declination are "variable", taking the nature of the bodies.
- **The orbs** (p. 47), offered as "hints … not hard and fast rules",
  each an "outside limit":
  - the conjunction and the opposition: 12° when the Sun aspects the
    Moon, "about 10°" when either luminary aspects a planet, "about 8°"
    between planets;
  - the square and the trine "about 8° all round";
  - the sextile "about 7°";
  - the semi-square and the sesquiquadrate 4°;
  - the semi-sextile and the quincunx 2°;
  - the parallel 1°.
- **Strength.** "The closer an aspect is, the stronger it is." An
  aspect barely inside its orb is "very weak" (p. 47).

**William Lilly**, *Christian Astrology* (1647), is the second model:
five Ptolemaic aspects, and an orb for each planet rather than for each
aspect (p. 107: Saturn 10°, Jupiter 12°, Mars 7½°, the Sun 17°, Venus
8°, Mercury 7°, the Moon 12½°). Two planets are in aspect while their
gap is inside the mean of their two orbs, their "moieties" (p. 110).
The SDK already ships his table as `LILLY_ORBS_DEG` and reads it in the
perfection.

**The worked figure.** Leo reads King Edward VII's nativity (pp.
295–296): 9 November 1841, 10.48 a.m. GMT, at Buckingham Palace. He
names the Sun trine Uranus, the Sun sextile Mars, the Sun square
Neptune, and the Moon square Saturn. These four are the acceptance test,
under his orbs.

## Decisions

1. **The set is a list, defaulting to Leo's nine.** `WesternAspect`
   names the nine longitudes: the conjunction, semi-sextile, semi-square,
   sextile, square, trine, sesquiquadrate, quincunx and opposition. A
   request lists the ones it wants. The Ptolemaic five are a named
   subset for Lilly's reading. The parallel waits for the declinations,
   which no chart carries yet.
2. **The orb is a model, not a number.** `OrbModel` has three members:
   - `Leo`, by aspect, with the conjunction and opposition widened for
     the luminaries as p. 47 says;
   - `Moieties`, a table of orbs by body, read as the mean of the pair's
     two (Lilly);
   - `ByAspect`, a caller's own orb for each aspect.

   Leo's is the default (C240): his set reaches the outer three, and
   Lilly's table has no orb for them. The caller can change every
   number. A body with no orb under the
   chosen model, such as Uranus under Lilly's table, is refused by name
   rather than given zero.
3. **The bodies are the chart's.** The default is the seven planets and
   the outer three when the chart placed them. The nodes and the two
   angles, the ascendant and the midheaven, are asked for by name. An
   angle has no motion of its own in a natal chart, so an aspect to one
   is neither applying nor separating.
4. **One row a pair and aspect.** Each row names the two bodies in the
   catalogue's order and the aspect. It gives the gap, the distance from
   exact, the orb allowed, and whether the faster body is closing. Rows
   are sorted closest first, since that is Leo's order of strength. Two
   aspects can share a pair only when their orbs overlap. That is
   possible under a caller's own orbs, and both rows are kept.
5. **The orb engine is shared.** `teistro_aspect::orb` already measures
   a gap against an angle and says which way it moves. The new code
   only chooses each angle's orb. The engine stays the single place
   where a separation is measured.
6. **A chart section like the rest.** `ChartRequest::with_western_aspects`
   takes the request, and the document carries the rows. The boundary
   and the bindings follow as one step, with parity.

## What this does not decide

- **Synastry and composites**, which ask the same question of two
  charts. They get their own page once this one is built.
- **Declinations**: the parallel and the contra-parallel.
- **Weighting.** Leo says a closer aspect is stronger and gives no
  formula. The row carries the distance from exact and the orb, and
  leaves any weighting to the reader.

## Order of work

1. This page, and a crux for the orb model (C240).
2. `western::aspects`: the set, the three models, and the table over a
   chart's bodies. Edward VII's four are the test, and a gate reports
   how the two models differ over the corpus.
3. The chart section, the boundary and every binding, with parity.
