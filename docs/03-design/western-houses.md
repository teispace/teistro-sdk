# A Western chart's houses (the `western` module)

Status: `building`, 2026-10-03, written from the sources before any code.
Steps 2 and 3 are built: `sdk.chart().western_houses` holds Leo's p. 150
figure, and [`western-houses-measured.md`](western-houses-measured.md)
reads the corpus's births.

The Western pages so far read the zodiac alone: aspects, parallels,
antiscia, equal distances, the composite's planets. A founded chart's
houses are its profile's bhava chalit, which is a Vedic division under
the shipped profiles, so a Western reading had no houses of its own.
The composite's cusps ([`western-composites.md`](western-composites.md))
and an antiscion on a cusp ([`western-antiscia.md`](western-antiscia.md))
both wait on them. This page decides which division the `western`
module reads, how a planet is counted in a house, and what the composite
does with its cusps.

The geometry is not new. `astro::houses` computes all twenty-two
catalogued systems with their polar outcomes and is measured against
Teimeris to 1e-6° ([`astro-house-systems.md`](astro-house-systems.md));
the `hellenistic` module already reads Lilly's Regiomontanus through
`houses.module_overrides.hellenistic`. This page adds a reader, not a
kernel.

## What the sources decide

- **Leo's division is Placidus, read off his own figure.** *How to
  Judge a Nativity* never names a system; it says "see in the Table of
  Houses" (p. 90), and the tables of houses of his day were Raphael's.
  His p. 150 illustration prints six cusps for "a female born at 2.42
  A.M. 13th December, 1835, London": X ♋29½, XI ♍4, XII ♎1, I ♎22,
  II ♏18, III ♐21. Recast by pyswisseph at 51.5° N, every Placidus
  cusp is within 0.5° of the printed one, which is the printing's own
  grain. Regiomontanus misses the third house by 3.8°, Koch the eleventh
  by 7.0°, Porphyry, Alcabitius and Campanus by 7° to 10°, and whole
  signs by up to 34° (C249).
- **Leo widens the ascendant by one sidereal hour.** "About 15° above
  the first house may be considered as included in the ascendant"
  (p. 90), and the footnote makes it exact: "strictly speaking 15° of
  Oblique Ascension", found by subtracting "one hour from the Sidereal
  Time at Birth" and reading in the table of houses "what degree is
  then rising". That degree, in the twelfth house, "may then be taken
  as forming the limit of influence of the first house" (C250). Leo
  states no such reach for any other house.
- **Lilly's rule is a different rule.** A planet within five degrees of
  a cusp is counted in that cusp's house, the nearer cusp winning
  (`AccidentalRules::cusp_orb_deg`, crux C214). It belongs to the
  `hellenistic` reading and its Regiomontanus, and it stays there.
- **A composite's cusps are Astrolog's.** Townley puts the composite
  planets "into a house wheel made up of mutual house cusp midpoints";
  Astrolog makes each cusp the near midpoint of the two charts' same
  cusp and turns any cusp more than 90° from the midheaven's quadrant
  place (MC + 30° × (n − 10)) by 180°. C247 already ships that turn
  for the lagna, the first cusp; the other ten follow the same rule.

## Decisions

1. **The division is the module's, Placidus by default (C249).** The
   `western` module reads `houses.module_overrides.western` and takes
   Placidus without one, as `hellenistic` takes Regiomontanus. A
   `WesternHouseRequest` may name any catalogued system instead, so a
   consumer reads Koch or whole signs without a profile of its own. The
   answer names the division the cusps are actually of, because a polar
   policy may fall back to another.
2. **A planet's house is by the cusps alone.** A planet stands in the
   house whose cusp it has passed and whose next cusp it has not, in
   the chart's zodiac; Leo widens no cusp but the ascendant's, and that
   widening is reported, not applied (decision 3).
3. **The ascendant's reach is a flag (C250).** Each planet carries
   `with_ascendant`: true in the first house, and true in the twelfth
   when it stands between the degree rising one sidereal hour before
   the birth and the ascendant. The degree is computed, not
   approximated by 15° of longitude, which Leo calls near enough "in a
   great many cases" and which the footnote corrects. The house number
   is never moved, so a reader counting houses and a reader asking
   what rises both read the same row.
4. **A planet's latitude is not read.** Leo reads the zodiac's degree
   against the cusps; a semi-arc placement by latitude is a later
   refinement no source here makes.
5. **The composite's cusps are the near midpoints, turned (Astrolog).**
   `Composite` gains twelve cusps; the first and tenth are the lagna
   and midheaven it already carries, so the two never disagree. Each
   composite planet's house follows decision 2 on those cusps.
6. **An antiscion may fall on a cusp.** `AntisciaRequest` gains the
   cusps as points among the planets, each reflected. A cusp takes the
   orb C242 gives the ascendant: a planet's under Leo's model, and under
   Lilly's moieties none, so a cusp is read there only against a
   planet's own moiety.
7. **Across the boundary, a section of its own.** A chart request's
   `western_houses` record names the system; the answer is a row a
   chart (the division, the twelve cusps, the sidereal-hour degree) and
   a row a planet (graha, house, `with_ascendant`).

## Order of work

1. This page, and the cruxes C249 and C250.
2. `western::houses` and `sdk.chart().western_houses`. Leo's p. 150
   figure is the test: the six printed cusps within 0.5° under the
   default, and Regiomontanus refused as the fit.
3. A measured page over the corpus's births: how many planets change
   house between Placidus and Regiomontanus, how many stand in the
   ascendant's reach, and how wide that reach is in longitude by
   latitude.
4. The composite's cusps and the antiscia on cusps.
5. The boundary record and every binding, with parity.

## What building it found

- **Leo's figure reads as he cast it.** Under the default every recast
  cusp of the p. 150 figure is within 0.01° of pyswisseph's and every
  printed one within his half degree, each planet stands in the house
  the recast puts it in, and Saturn, in the first, is the only planet
  with the ascendant. Its glyph on the page reads like Virgo; the recast
  puts it at ♏2, so it is Scorpio's.
- **A reader's division moves planets.** Over the corpus, Regiomontanus
  counts 6.2% of the planets in another house than Placidus, and 41.5%
  of births have one moved; whole signs move half of them. C249 is not
  a rounding question.
- **The hour is not always in the twelfth.** Leo takes the degree that
  rose an hour before to be "found in the twelfth house, of course".
  Where the signs about the ascendant rise quickly, Placidus's twelfth
  is narrower than the hour, and at 2 of 53 births the reach passes its
  cusp. The flag follows the degree, as the footnote does.
- **The zodiac moves no house; a profile can.** The births founded
  again under `western-tropical-default` count every planet in the same
  house but one: the Moon at Kabul, which `conformance-baseline` places
  topocentrically, 0.56° from its geocentric place and on the eleventh
  cusp. The measured page holds the claim that the zodiac alone moves
  nothing.
- **Inside the polar circles a profile decides.** Placidus does not
  exist at Tromsø, and the conformance profile refuses rather than
  falls back, so the two births there are named and left out; a profile
  that falls back reads them, and the answer names the division used.
- **One rule to count a house.** Lilly's `house_of` already counted a
  longitude by the cusps with an orb; it moved to `core::house`, and the
  `western` reading calls it with an orb of zero.

## What this does not decide

- **The other houses' reach.** Leo states one for the ascendant only.
- **Gauquelin sectors and house positions by semi-arc.** Both read a
  planet's latitude; neither is in a source read here.
