# The edges of the coverage

Status: `built`, 2026-11-17. Design for the application's SR-136
(`tithipatra-v2/docs/04-architecture/sdk-briefs/teistro-teimeris-brief-addendum-10.md`,
items 1 to 6), written before the code and corrected by it. Cruxes C2240 to
C2250.

## 1. What was asked

The built-in ephemeris and Teimeris both state 1800 to 2400. The consumer
found that a date inside that span was still refused near either end: the
almanac of 1 January 1800 at Kathmandu, any lunar year from 1800 to 1803,
festivals from 1800 to early 1801, and from November 2399 to the end. Every
refusal was `OUT_OF_RANGE` or `NOT_CONVERGED`, naming an instant **outside**
the coverage that a search around the date had read: the tithi running at
the first sunrise began the evening before, the lunar month began in
December 1799, and the lunar year in the spring of 1799.

The brief asks for four things:

1. the range as data, one call, on every target and binding;
2. every date of the coverage answered, a member that began before the
   coverage (or outlasts it) reported as such rather than refused;
3. a property test over 1800-01-01 to 2399-12-31 at Kathmandu and New York;
4. any rule narrower than the range stated.

## 2. Where the span itself ended (C2240, C2241)

Before anything else, the built-in span was measured. Its declared
`JD_RANGE` opened at **noon** on 1 January 1800 (2378497.0) and closed at
noon on 31 December 2399, so the first and last half days of the range were
refused by the provider itself. The Chebyshev blocks reach midnight at both
ends; the declaration was the narrower of the two. It is now midnight to
midnight, 2378496.5 to 2597641.5 TT.

Pluto's table begins later than the other bodies'. A day past the start of
its fit, read from the neighbouring block and measured against the fit,
Pluto is off by 1.573″ (compact tier, bound 32.06″), 0.392″ (standard,
bound 0.867″) and 0.00205″ (full, bound 0.0061″): each inside its tier's
own bound, so the block nearest is read for a day past its edge
(`PLUTO_REACH_DAYS`) rather than refusing the first day.

The span is stated in TT and every SDK search runs on UT1, so the
completion carries each end to UT1 (Delta T about 14 s at 1800, about
17.6 min at 2400) and moves it one millisecond inward, which keeps the
carried end on the provider's side of its own bound after rounding
(`Completion::coverage`). A sky that takes parts of a modern ephemeris for
a classical text (the official profile's sunrise) ends an hour inside the
modern one's, more than every Delta T model's spread there (C2245).

## 3. The alternatives, measured

| | how | what it costs | verdict |
|---|---|---|---|
| **A. Clip at the search** | every search a date makes is cut to the coverage where it is made, and a member the cut touches carries a mark saying which end the coverage cut | one comparison per search; nothing inside the coverage moves | **chosen** |
| B. Read a margin from a fallback | widen the configured ephemeris with another (the analytic theory) for the reach a date's searches make | the reach, measured over the analytic sky at Kathmandu, 15 June 2026: days 45.9 days before and 32.7 after (official), 32.3 / 32.7 (drik); lunar years 1225.3 / 326.7; festivals 104.7 / 63.9 (official), 105.2 / 38.7 (drik); so 3.4 years before and nearly a year after on another engine | rejected: the answers near the edges would mix two engines, and a provider with a hard end (Teimeris, a consumer's own) has no fallback to read |
| C. Shrink the advertised span | advertise only the dates whose every search fits | the first date whose lunar years fit is 1804-03-13 | rejected by the brief: it hides four years the engine computes |

Under A, a date is answered from what the coverage holds, and a member that
reaches past an edge says so: its cut end is the coverage's edge, never an
instant nobody computed, and the type says which end.

## 4. The design (C2242 to C2249)

**One mark.** `teistro_core::coverage::Clipped` is `START`, `END` or `BOTH`:
the member began before the coverage, outlasts it, or both. It is carried by
`Span` (every limb, sign, panchaka), `LunarMonth`, `LunarYear` and the day
sky (`DaySky`), each `clipped`, left out of the wire when absent. A day cut
at its end (`LocalDay.clipped`) is the last day whose next sunrise is past
the coverage: its next sunrise is the coverage's last instant.

**One cut, at the port.** Every source of positions states its coverage
(`Longitudes::coverage`, `ApparentPositions::coverage`,
`SolarModel::coverage`), forwarded through every wrapper, unbounded unless a
source says otherwise. The crossing search (`Search::between_each`), the
rise and set solver (`Solver::event`), the day sky and the sankranti count
cut their windows to it where they are made, so no caller near an edge
does anything of its own:

- the crossing scan keeps its anchored grid, and only the cell an edge falls
  in is shortened to end at it, so every cell inside is the grid's own to
  the bit (C2242);
- a limb's spans are cut by the same function that closes them (`cut`),
  the first or last span marked;
- the rise and set solver searches the cut window, and an iteration whose
  estimate leaves the coverage hands the search to the scan;
- the sankranti count past the model's last instant reads the Sun's sign at
  the two ends, the Sun running forwards and less than a sign in a month.

**Months (C2246).** A lunar month whose new moon the coverage does not hold
is cut at the edge. How far past the edge that new moon lies is bounded by
the Moon's age there over her least and greatest elongation rates, and by a
lunation of 29.2 to 29.9 days less the part the coverage holds. Its name is
the Sun's sign at the edge less the sankrantis his own rates (0.95 to 1.03°
a day) say he passed in between; its kind (nija, adhika, kshaya) counts the
sankrantis inside and those the rates settle outside. When the rates leave
either open, the month is refused `OUT_OF_RANGE` on `date`, naming why. Over
the built-in ephemeris they never do: the first lunation (opened about 26
December 1799) and the last (closing about 26 January 2400) are both
settled. A cut month has no decision margin, which would be read at the
new moon the coverage does not hold.

**Years (C2249).** A lunar year that opened before the coverage is cut at
its first instant. Its numbers are the next year's less one; its name is
read from the Jovian count running over every instant its first sunrise can
have been (twelve to thirteen lunations before the next year's opening);
it has no office-bearers. A year the coverage ends in is cut at its last
instant, and an office whose ingress falls after it is left out. The
naming's look back over earlier years reads them the same way. When a
Jovian year changes inside the window a first sunrise can be, the year is
refused `OUT_OF_RANGE`, naming the window; over the built-in ephemeris it
never is.

**Festivals (C2248).** A festival range widens by the days its rules read
(up to 33 before and a lunation after). The widening is cut to the days the
almanac answers at the place (`Almanac::answered`), so a day near an edge is
reckoned from the days there are, exactly as a rule at the edge of any
range is. A panchaka that began before the coverage has no first instant
and so no vara to be named by, and is left unnamed; one that ends after it
is named and cut. The Sun's nakshatra at a civil midnight a few hours
outside the coverage is read at the edge when his greatest rate cannot
reach a boundary in between (`sun_nakshatra_near`), and refused otherwise.

## 5. The range as data (C2250)

`Context::coverage()` (`ts_context_coverage`, `coverage()` on every
binding's context, the MCP tool `coverage.describe`) answers:

| field | what it is |
|---|---|
| `ephemeris` | `{fromJd, toJd, first, last}`: the first and last instants the configured ephemeris answers under the profile, UT1, and the first and last Gregorian dates the span holds at least half of, on UT (an end on UT1 is not a midnight: the built-in span opens 14 s before 0 h UT on 1 January 1800 under the drik profile, an hour after it under the official one). Built-in and Teimeris: 1800-01-01 to 2399-12-31 |
| `bikramSambat` | `{firstYear, lastYear, officialFirstYear, officialLastYear}`: the shipped table's years and those the government's calendar committee published (`BikramSambat::official_years`) |
| `holidays` | `[{set, years}]`: each shipped holiday set and its years (`NEPAL_GAZETTE`: VS 2083) |

## 6. What is narrower than the range

- **A day whose sunrise precedes the coverage is refused.** On 1 January
  1800 the first instant is 0 h UT; a place east of about 97° E has its
  sunrise before it, and that one civil day is `OUT_OF_RANGE` on `date`
  there. Kathmandu (85.3° E) and everywhere west are answered from
  1 January 1800; the last date, 31 December 2399, is answered everywhere
  west of the date line, its next sunrise cut to the coverage's last
  instant (C2244).
- **No festival rule is narrower.** `NEPAL_PANCHANGA`'s rules answer every
  Bikram Sambat year from 1857 to 2455, the years the coverage holds whole;
  a rule near an edge is reckoned from the days the coverage holds.
- **The conditional refusals** of §4 (a month's name or kind, a year's
  name, a nakshatra entry, each when the rates leave it open) are stated
  for a consumer's own provider; over the built-in ephemeris and
  Teimeris none of them fires, as the property test of §7 shows.

## 7. The proof

`crates/sdk/tests/coverage_edges.rs` asks every call of the brief's table
(the almanac of a day under the Nepal official and drik profiles, with its
years, its festivals, its sky; the sky view) at Kathmandu and New York on
every day of the first and last 40 days of the coverage and on a sparse
sample between, and asserts each answers, that a member cut by the
coverage carries its mark, and that none carries one in the middle.
