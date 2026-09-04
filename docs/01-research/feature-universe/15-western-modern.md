# Western modern astrology

Status: `research`, 2026-09-04. Checked against Solar Fire 9's feature page
(the most complete published list), Maitreya and the baseline engine's research module.
Tier P1 throughout: designed in v1.0, shipped in v1.x (Q4).

## Frames and points

| feature | inputs | variants | field | module |
|---|---|---|---|---|
| tropical zodiac end to end, with sidereal as an option and precession-corrected comparisons | ephemeris | | all | `core` |
| points catalogue: Sun through Pluto, nodes (mean, true), Chiron, main asteroids, Black Moon Lilith (mean, true, interpolated), Eris, Sedna, Vertex, East Point, Part of Fortune (day/night), Aries Point, Galactic Centre, Uranian points (8 plus Transpluto), hypothetical Vulcan variants | ephemeris and arithmetic | Solar Fire lists 50 standard points plus user-defined | Solar Fire | `western.points` |
| Arabic parts: 100+ pre-defined with A+B−C formulas, day/night variants, custom editor | cusps, positions | | Solar Fire, Delphic Oracle | `western.parts` |
| fixed stars: catalogue (290), Ptolemaic 31, parans, rise/set/culminate, star aspects, heliacal events | star catalogue via port | | Solar Fire | `western.stars` |
| declinations and latitudes, parallels and contra-parallels, out-of-bounds | equatorial output | | Solar Fire | `western.aspects` |
| antiscia and contra-antiscia | | | Solar Fire | `western.points` |

## Aspects

| feature | variants | module |
|---|---|---|
| aspect set: conjunction, opposition, trine, square, sextile, quincunx, semi-sextile, semi-square, sesquiquadrate, quintile family, septile family, novile family, undecile, and user-defined; 26 predefined in Solar Fire | orbs per aspect, per body class (luminaries versus others), per chart type (natal, transit, progressed); moieties; applying versus separating with speed; 3-D aspects; sign-to-sign aspects | `western.aspects` |
| aspect grids, aspectarian, sorted aspect lists | | `western.aspects` |
| configurations: grand trine, T-square, grand cross, yod, kite, mystic rectangle, stellium, and user-defined | rule pack | `rules` |
| midpoints: trees, sorted lists, 45° and 90° dials, planetary pictures, Munkasey weighting | modulus and orb | `western.midpoints` |
| harmonics and harmonic charts, age harmonics | | `western.harmonics` |

## Predictive

| feature | variants | module |
|---|---|---|
| secondary progressions (day for a year), tertiary (mean and true), minor, user-defined rates; MC by solar arc or Naibod; Q2 daily houses; converse | | `western.progressions` |
| directions: solar arc, ascendant arc, vertex arc, user arc; whole, half, double, reverse; primary directions (Ptolemy, Naibod, Van Dam; Placidus semi-arc and Regiomontanus; zodiacal and mundane) | | `western.directions` |
| returns: solar, lunar, planetary, asteroid; precession-corrected; converse; demi and quarti; progressed solar returns; Wynn key cycles | crossing search | `western.returns` |
| transits: hit lists (entering, exact, leaving), ingresses, stations, eclipses, void-of-course Moon (standard and Lilly), lunar phases and phase returns | | `gochar` (shared) |
| time map and graphic ephemeris data (longitude, latitude, declination tracks; modulus 360, 30, 45, 90) | positions grid | `western.ephemeris-data` |
| eclipse paths and Saros | port | `port` |

## Relationships

| feature | module |
|---|---|
| synastry grids with orbs, inter-aspects, house overlays | `matching.western` |
| composite (midpoint) and Davison; coalescent; multi-person composites | `matching.western` |
| four-chart dials | presentation |

## Relocation and mapping

| feature | module |
|---|---|
| relocated charts | `chart` |
| astro-lines: MC, IC, ascendant, descendant lines, parans, aspect and midpoint lines; local space (azimuth lines); geodetic charts | `western.mapping` (geometry only; the map is the consumer's) |

## Other

| feature | module |
|---|---|
| dispositor trees, final dispositors, mutual receptions | `relationship` |
| element, mode, quadrant, hemisphere balances; Astrodynes; temperament | `western.analysis` |
| financial astrology helpers (first-trade charts, planetary returns, declination cycles) | out of scope beyond the primitives |
| esoteric: rulership sets (modern, traditional, esoteric, hierarchical), rays | data packs |
| Sabian symbols and degree meanings | interpretation packs |

## Closing checklist

- The aspect engine, orb tables and points catalogue must be designed in
  Phase 0 so the core chart model carries the tropical frame, declinations
  and speeds for every point from day one.
- Progressions and directions need only positions and cusps, so they are
  pure SDK modules over the port; returns need the crossing search.
