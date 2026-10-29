# Composite and Davison charts (the `western` module)

Status: `built`, 2026-10-03, written from the sources before any code.
Every step is built: `sdk.chart().composite` and `Partner::davison`,
held to a recast of George V and Queen Mary, measured over the corpus's
pairs of births in [`composites-measured.md`](composites-measured.md),
and crossing to every binding as `chart.synastryComposite` and
`chart.synastryDavison`, with parity.

[`western-synastry.md`](western-synastry.md) reads two charts against
each other. A relationship chart makes **one** chart of two: the
composite from the midpoints of their positions, the Davison from the
midpoint of their births. This page decides how each is built.

## What the sources decide

Both methods are modern, and the books that introduced or spread them
are in copyright and not readable here: John Townley's *The Composite
Chart* (1973), Robert Hand's *Planets in Composite* (1975; archive.org
holds a 1981 printing, `planetsincomposi00hand`, for lending only) and
Ronald Davison's *Synastry* (1977; archive.org holds the 1983 edition,
`synastry00rona`, for lending only). So the definitions are read at
rank 2, from the composite's originator in his own words, and each
choice no source read here settles is a crux, decided as the SDK's own
with its reasoning. No program's code is read for any of it
(`CLEAN_ROOM.md`).

- **John Townley**, the composite's originator, defines both charts on
  his own site ("Composites vs. Davison time charts", March 2014,
  astrococktail.com/compositedavison.html):
  - the composite: "Take the midpoints of each of A's and B's mutual
    planets (Sun with Sun, Moon with Moon, etc.), put them into a house
    wheel made up of mutual house cusp midpoints, and that's your
    composite chart";
  - the midpoint is the **near** one: with A's Sun at 15 Gemini and B's
    at 15 Leo, "the near midpoint (C) at 15 Cancer" is the composite's,
    and 15 Capricorn the far one;
  - the Davison chart "is simply a natal chart cast for the midpoint in
    time and place between them". His worked example, Shawnee, Oklahoma
    (18 December 1963) with Los Angeles (4 June 1975), stands "about 85
    miles SW of Albuquerque".
- **Townley in interview** (*The Astrology Podcast*, episode 128, 2017,
  the published transcript): of a composite whose ascendant is looked up
  in a table of houses from the composite midheaven at a chosen
  latitude, "that means the midheaven is a real midpoint and the
  Ascendant has nothing to do with anything". The originator keeps the
  ascendant a midpoint.
- **Robert Hand**, *Planets in Composite*, is not read: lending only.
  Secondary pages credit it with a composite ascendant derived from the
  composite midheaven at the latitude where the couple live; none quotes
  the book, so no rule here rests on it, and the derived ascendant stays
  the rival named in C247.
- **The Davison place.** Astrodienst's documentation of its own
  relationship charts (astro.com, "Uncorrected Davison Relationship
  Chart", rank 3) gives the uncorrected chart at "the exact mean in time
  and geographic place of both charts", latitude and longitude each the
  half sum of the two, and names a *corrected* variant, the time moved
  so the chart's midheaven equals the composite's, as "the classical
  calculation method". Davison's own text is not read, so which of the
  two he printed is open (C248).

**The measure.** Townley's worked Davison stands "about 85 miles SW of
Albuquerque". The arithmetic mean of the two places stands 60 miles
south-west of Albuquerque (bearing 243°); the great-circle midpoint
stands 58 miles due west of it (275°). The direction is the arithmetic
mean's. His printed time does not follow from the births he gives under
either reading, so the example is read for its place alone.

## Decisions

1. **A composite planet is the near midpoint (Townley).** Each planet
   of one chart is paired with the same planet of the other, and placed
   at the midpoint of the shorter arc between them, Townley's "near
   midpoint", the same whichever chart is first. Two planets exactly
   opposite have no shorter arc; they then meet at the mean of their two
   longitudes, so the answer is fixed either way round (the SDK's own
   rule: no source names the case). Its speed is the mean of the two,
   which is the rate at which the midpoint of two moving points moves,
   so a composite's aspect table reads applying and separating (the
   SDK's own: Townley gives a composite no motion).
2. **The angles are midpoints, the lagna kept east of the midheaven
   (C247).** The composite midheaven is the near midpoint of the two,
   and the lagna the near midpoint of the two lagnas: Townley's "mutual
   house cusp midpoints", and in his own words the ascendant is not
   derived from the midheaven. The lagna is then turned by 180° when it
   falls more than 90° from the midheaven's eastern quadrature (the
   midheaven plus 90°). That turn is **the SDK's own decision**: no
   source read here states it. The reasoning is geometric: a natal
   lagna always stands in the half of the zodiac after its midheaven,
   and the near midpoint of two far-apart lagnas need not, so an
   unturned midpoint can put the composite's rising degree west of its
   own midheaven, which no chart can have. A row says when it was
   turned, so a reader who wants the bare midpoint has it. Deriving the
   lagna from the composite midheaven at some latitude, the method
   secondary pages credit to Hand, is not read in its source and is
   not offered.
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
   the mean of their longitudes the shorter way round; and the mean of
   their clocks, to the second, which only names the civil day. The
   mean of the instants and of the latitudes is Townley's "midpoint in
   time and place"; the arithmetic mean rather than the great-circle
   midpoint is his example's direction, south-west of Albuquerque, and
   Astrodienst's documented formula. The shorter way round, the
   altitudes and the clocks are **the SDK's own**: no source read here
   names them. The half sum of two signed longitudes either side of the
   antimeridian lands on the far side of the Earth, so it depends on
   where the ±180° seam is drawn rather than on the two places; the
   shorter way round does not, and agrees with the half sum whenever
   the two are within 180°, as in every case a source prints. The
   corrected Davison (the time moved to the composite midheaven) is not
   offered while Davison's text is unread. It is returned
   as a `Partner`, so any chart request, under any profile, founds it,
   and every reading a natal chart takes, a Davison takes.

6. **Across the boundary, in the synastry record.** A composite and a
   Davison birth are each made of a chart and its partner, which the
   `synastry` record already pairs. `"composite": true` answers each
   chart's composite with the partner: a row a chart (lagna, midheaven,
   whether turned, and how many planets) and its planets ragged under
   it. `"davison": true` answers each chart's Davison birth with the
   partner, a row a chart (instant, place and clock), which the caller
   founds with any chart request as it founds a birth. The arithmetic
   stays in one place, the SDK, and no new export is needed.

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

## What building it found

- **The recast agrees.** George V's and Queen Mary's composite holds
  every planet within 0.01° of the midpoints of a Moshier recast, its
  lagna not turned; and a chart cast at their Davison birth agrees with
  the recast cast there. Read in each chart's own zodiac, every point
  moves by the mean of the two ayanamshas, as decision 4 says.
- **A midpoint must not care which comes first.** The first measure
  found the Davison birth of a pair differing from the pair's reverse
  in 725 of 1485 pairs, in the last bit: `a + (b − a)/2` rounds
  differently from `b + (a − b)/2`. The near midpoint is now the mean of
  the two and, across the wrap, the point opposite it, which is the
  same either way round to the bit, the exactly opposite pair included.
- **The turn is rare and needed.** 72 of the 1485 pairs (4.8%) have the
  lagnas' near midpoint before the composite midheaven (C247).
- **The Sun parts the methods by the years between.** The Davison Sun
  stands near the composite's when the births are an even number of
  years apart and opposite it when odd, since it made that many whole
  turns between them; every other planet parts further, the Moon by a
  median of 166°.

- **A document keeps no clock.** The Davison birth's clock is the mean
  of the two, and a founded chart does not carry the clock it was asked
  on, so `synastry_with` cannot give it; `PartnerSynastry::davisons`
  takes the clock from its caller, and the boundary passes the batch's.

## What this does not decide

- **The composite's cusps** were decided with the Western houses
  ([`western-houses.md`](western-houses.md)): each the near midpoint of
  the two charts' same cusp, turned by the midheaven as the lagna is
  (the SDK's own rule, C247), in the module's division; none at a place
  where either chart's division is refused.
- **Weighted and multi-person composites.** Some programs weight a
  composite or a Davison chart toward one of the two, or compose more
  than two charts. No source read here uses either.
- **The derived lagna.** A composite lagna computed from the composite
  midheaven at a latitude is credited to Hand by secondary pages; his
  text is not read here, and Townley rejects the method.
