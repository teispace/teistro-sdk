# KP: Krishnamurti Paddhati

Status: `draft`, 2026-09-29; §6 steps 1 to 7 **built** by 2026-09-30. Written
from the Readers before any code; the building is expected to correct it.

Derives from `01-research/feature-universe/06-kp.md` (P0: the 249
sub-lord table, the star, sub and sub-sub lords of every body and cusp,
the significators, the ruling planets and horary from a number; its
closing checklist asks that the table match the published one "to the
arcsecond" and that KP be bound to its own ayanamsha) and the roadmap's
Phase 7 `kp`, whose exit criterion is "the KP profile enforces the KP
ayanamsha".

## 1. What the Readers say

K. S. Krishnamurti's *KP Readers* (rank 1; the Internet Archive's
`kp-readers` scans). The books are under copyright: this page states
their facts and quotes a few words, never a table or a passage.

- **The sub.** Each nakshatra of 13°20′ is divided among the nine
  Vimshottari lords in proportion to their years of the 120, starting
  from the nakshatra's own lord: Ashwini's first sub is Ketu's (7/120 of
  it, 46′40″), then Venus's, and so round. A sub is its lord's years ×
  6′40″ wherever it falls. Each sub divides again the same way, starting
  from its own lord: the **sub-sub** (Reader VI, "one can sub-divide").
- **249, not 243** (Reader VI). The lords of the sign, the star and the
  sub come to 249 combinations rather than 27 × 9, because "certain subs
  are found in 2 adjacent signs": a sub a sign's end cuts is two rows.
- **Horary from a number** (Reader I). The querent names a number from 1
  to 249; the start of that row is the **lagna**, the other cusps are
  Placidus's (Raphael's tables at the place's latitude) under the
  Krishnamurti ayanamsha, and the planets are taken at the **moment of
  judgement**. The worked examples: 48 opens at Gemini 8°40′ (Mercury's
  sign, Rahu's star, Jupiter's sub), 74 at Cancer 14°53′20″ (the Moon's,
  Saturn's Pushya, Jupiter's).
- **The ruling planets** (Reader VI): the lord of the day, the lagna's
  star lord and sign lord, and the Moon's star lord and sign lord at the
  moment of judgement: "these five planets". The day runs "sunrise to
  next sunrise".
- **A node among them** (Reader VI): a node in a ruler's sign "should be
  taken", as its agent; the worked example also takes Ketu for being
  conjoined with Jupiter.
- **Retrograde rulers** (Reader VI): rulers "deposited in the
  constellation of a retrograde planet should be rejected"; a later
  procedure in the same book checks the "constellation or sub".
- **The significators of a house** (Reader VI), strongest first: (a)
  planets in the stars of its occupants, (b) the occupants, (c) planets
  in the stars of its lord, (d) the lord; then (e) planets conjoined with
  those and (f) planets aspecting them.
- **A node as a significator** (Reader VI, in one passage): a node gives
  "first" the results of the planets it is conjoined with, "then" of the
  planet in whose star it stands, "then" of the planets aspecting it,
  "and lastly" of its sign's lord. The same example reads a conjunction
  as the same sign and an aspect as a graha's drishti ("by its 7th
  aspect").
- **Interception** (Reader III): a sign lying between two cusps
  "without touching either".

## 2. What the baseline engine does

The baseline engine's KP module (rank 2) builds the same 249 rows, and
parts from the Readers in five places, each a fork below:

| | the Readers | the baseline engine |
|---|---|---|
| ruling planets | five | seven: adds the lagna's and the Moon's **sub** lords |
| the day lord's day | sunrise to sunrise | the civil weekday |
| significators | (a) to (d), then (e) and (f) | (a) to (d), and an intercepted sign's lord added to a house |
| high latitude | Placidus, silent past the polar circle | equal houses |
| arithmetic | — | degrees in floating point, `1e-10` added at the edges |

The last is not a fork but a defect of method: a sub-sub's edge is
generally not a whole number of any unit, and a floating-point sum of
spans with an epsilon decides a longitude near it by rounding. §4 removes
the question.

## 3. Forks

| # | fork | default | the other reading |
|---|---|---|---|
| C150 | how many ruling planets | the Readers' **five** | `kp.ruling_count = WITH_SUBS`, the baseline's seven |
| C151 | the day lord's day | **sunrise to sunrise**, the chart's own vara | `kp.day_lord_day = CIVIL`, the weekday on the clock the request names |
| C152 | a node among the rulers | **in a ruler's sign, or in the same sign as one** (the worked example's conjunction) | `kp.node_rulers = SIGN` |
| C153 | which retrograde rulers are rejected | **in a retrograde planet's star**, the rule as stated; the sub reported beside it | `kp.retrograde_rejection = STAR_OR_SUB`, the later procedure |
| C154 | a house's significators | **(a) to (d) as levels**, (e) and (f) reported apart | the baseline's intercepted lord as a fifth level |
| C155 | a node as a significator | **the Reader's order**: conjoined, star lord, aspecting, sign lord | — |
| C156 | the horary chart | **the number's start as the lagna**, Placidus cusps from it at the place, planets at the moment | — |
| C157 | the ayanamsha a KP reading takes | **Krishnamurti** (the profile's), refused under any other unless the request says so | `KRISHNAMURTI_VP291`, or any, named |

C150 to C155 are the Readers against a rank-2 engine and are settled by
the Readers, with the engine's reading a named value rather than a
silent alternative. C156 has one reading and is registered because the
horary chart is built from a number rather than a time, which nothing
else in the SDK does. C157 is the research page's checklist item: KP
under another ayanamsha moves every sub, and the common mistake is doing
so without knowing.

## 4. The arithmetic

A sub spans its lord's years × 6′40″, which is 4 × 10¹¹ nanoarcseconds
a year: every sub edge is a whole number of `Nas`. A sub-sub is 1/120 of
that times its own lord's years, and is generally not. So the lords are
found in integers scaled by 120 once per level below the star: the
position inside its nakshatra, times 120ᵏ⁻¹ for a chain of k levels, is
compared with running sums of `width / 120 × years` that are exact at
every level (`i128`; nine levels reach 2 × 10³⁰). A position on an edge
belongs to the level it opens, as a sign's 0° does.

Each level carries its **span**: the first `Nas` inside it and the first
past it, rounding a fractional edge up so that `contains` is exactly the
set the comparison selects. `Span::margin` is the arc to the nearer
edge, which is what a sub lord's reliability turns on: the lagna moves
about a degree in four minutes, so a sub whose margin is a few
arcminutes changes with a birth time known to the minute.

The 249 rows are generated by the same rule in a `const fn` and
asserted, when the crate compiles, to come to 249; they are held to
every row the scan prints legibly (1, 48, 74, 207 and 208 either side
of Aquarius, 228 and 229 either side of Pisces, and 238 to 249). The six
split subs are a pattern the rule predicts and the table confirms:
Rahu's sub in each of the Sun's three stars and the Moon's in each of
Jupiter's, the only subs a sign's end falls inside.

## 5. The design

- `teistro-kp` (core and dasha): `lords(Nas) -> Lords` (the sign, star,
  sub and sub-sub, each with its span), `chain::<N>(Nas)` for 1 to 9
  levels, and `KpNumber` (1 to 249, `of`, `start`, `span`, `lords`). The
  nine lords and their years are the dasha crate's `VIMSHOTTARI_LORDS`,
  the one list the Vimshottari row also borrows.
- **The KP chart** (step 2): the cusps under the house system the
  `kp` module override names (`PLACIDUS` under `kp-default`), each cusp
  and graha with its `Lords`, a graha's house the one whose cusp it
  follows, and the ayanamsha check of C157.
- **Significators** (step 3): per house, the four levels of C154 in
  order with (e) and (f) apart; per graha, the houses it signifies at
  each level; the nodes by C155.
- **Ruling planets** (step 4): at an instant and place, the five (or
  seven), each with the reason it is there, the nodes C152 adds, and the
  rejections of C153 with the retrograde planet that caused each.
- **Horary** (step 5): `KpNumber` and a moment and place to a chart
  whose lagna is the number's start (C156).
- **The boundary** (step 6). A chart request gains `kp_json`, null for
  no KP, else `{"number": 74, "clock": 19800, "anyAyanamsha": true}`
  with every member optional: `number` makes every chart a horary chart
  for it, `clock` (seconds east of UT) is the civil day lord's clock —
  the chart request's own `utc_offset_seconds` when absent, since a
  binding has already said it once — and `anyAyanamsha` lifts C157. Every chart then answers a **`KpReading`**
  — `{chart, significators, ruling}` — which is also one façade call in
  Rust, `sdk.chart().kp_reading(&chart, &request)`. The ruling planets
  are always the moment's, its own lagna and Moon, since the Reader
  takes them "at the moment of judgment"; only the cusps take the
  number.

  The reading crosses as **one canonical-JSON section**, `kp`, an array
  with one reading a chart, empty when none was asked for: the shape of
  `plans` and `rules`, not of columns. A reading is nested and variable
  — a house's significators are lists of any length, a ruler carries
  every reason it rules — and columns would flatten it into ragged
  counts each binding reassembles, three times over, for a section read
  once a chart. The JSON is serde's own, and the ABI test
  (`a_chart_request_answers_kp`) holds the section to the façade's
  reading value for value and to the member names its description
  lists, one by one: `keys.rs` holds closed enums and has nothing here
  to hold.

  Building it corrected the typing. Each binding gives the reading in
  **its own answers' shape**, not the JSON's: Node writes every key in
  full (`graha.SUN`, `house_system.PLACIDUS`), as every other Node
  accessor does and as its declarations' `Graha` already said — the
  first draft handed the bare keys through and every test stayed green,
  because none compared a spelling. Python builds frozen dataclasses
  with snake-cased fields and catalogue members, like `SadeSatiReport`,
  rather than `TypedDict`s over strings. Dart builds `final` classes
  over one private field-wise value base, so a reading compares by
  value. Parity prints every cusp, planet, house, node and ruler in
  all five runners.

## 6. Order of work

1. **Built**: the exact lords at any depth and the 249 numbers
   (`crates/kp`), held to the printed rows; the Vimshottari lords made
   one list.
2. **Built**: the KP chart. `sdk.chart().kp(&chart, &KpRequest)` reads
   a founded chart's cusps under the system `houses.module_overrides.kp`
   names (Placidus when none is), rebuilt from its instant, place and
   zodiac by `teistro_chart::foundation::cusps_of`, so a stored chart
   answers it without an ephemeris; each cusp and graha with its lords
   and each graha in the house whose cusp it follows (`KpChart`). C157 is
   the request's: a chart outside `KP_AYANAMSHAS` is refused on
   `frame.ayanamsha` unless `under_any_ayanamsha()` is asked. Acceptance
   is a read-back through the chart: the 1st cusp is its lagna and the
   10th its midheaven, to the nanoarcsecond.
3. **Built**: the significators. `Significators::of(&KpChart,
   NodeAspects)` and `sdk.chart().kp_significators(&kp)`: per house the
   four levels in order (`in_order` flattens them strongest first), (e)
   the planets sharing a sign with any of them and (f) the planets they
   aspect by drishti, each planet counted at its strongest place, and the
   signs intercepted in it; per node its `NodeAgency` in the Reader's
   order; per planet the houses it signifies at each level. The drishti
   is the aspect crate's own table under `aspect.node_aspects`, not a
   second copy. Worked by hand on a chart that fills every level, and a
   node knob shown to move (f).
4. **Built**: the ruling planets. `RulingPlanets::of(&KpChart, day_lord,
   RulingRules)` and `sdk.chart().kp_ruling(&chart, &KpRequest)`, under
   a new `kp` settings group whose four knobs are C150 to C153. Each
   ruler is listed once with every reason it rules (`Reason`), a node's
   as the agent of which ruler and how; a rejection names the retrograde
   planet, and the sub reading's rejection is reported under either
   setting. Two readings the Reader leaves implicit are written down: a
   node's own backward motion rejects nobody, and a ruler's own
   retrogression is delay, not rejection. The acceptance is the Reader's
   worked horary (Sunday, Moon in Venus's star in Sagittarius, Libra
   rising): Sun, Jupiter, Venus, Rahu and Ketu, reproduced. The
   `conformance-baseline` profile does not take the baseline engine's
   readings: the corpus records no KP to hold them to, and a profile
   version is spent on a measured change.
5. **Built**: horary from a number. `sdk.chart().kp_horary(&moment,
   KpNumber, &KpRequest)` keeps the moment's planets and gives the
   chart the number's start as its lagna exactly, with the other cusps
   those that ascendant has at the place: the meridian that raises it is
   found by bisection at the moment's obliquity and ayanamsha
   (`teistro_chart::foundation::cusps_raising`), which is what reading
   Raphael's tables at the latitude does. The acceptance reads it back
   through founded charts: for numbers 1, 48, 74, 229 and 249 the moment
   that day when the lagna itself reaches the number's start, found by
   founding charts, has the same twelve cusps to under 0.1″. A latitude
   where no meridian raises the start is refused on `place.latitude`.

   The measured pass corrected the search. The ascendant goes once round
   the circle as the meridian does only where every ecliptic point
   rises, under 90° less the obliquity. Beyond it the ascendant jumps
   across the arcs that never rise or never set, and one bisection over
   the day bracketed the jump. At Tromsø that refused 165 of the 249
   numbers where the sky refuses 81. There the turn is now scanned in
   quarter-degree steps and the first true crossing bisected, and the
   temperate search is unchanged to the bit. A test at 69.65° raises a
   rising number and reads it back through founded charts, and refuses
   two that never rise.
6. **Built**: the boundary and the bindings, as §5 says, with
   `sdk.chart().kp_reading` the one façade call, `KpRequest::from_json`
   the record every binding writes, and section 59 `kp`.
7. **Built**: the measured pass (`kp-measured.md`, `cargo xtask kp`,
   gated by `check-kp`). Over the recorded births:
   - The lagna's sub lord holds a median of under two minutes of birth
     time, and the sub-sub lord seconds. The margins are read back
     through the founder.
   - VP291 stands about 72″ from Krishnamurti's ayanamsha, and parts
     from it on a handful of sub lords and many sub-sub lords.
   - Each place refuses exactly the numbers whose start's declination
     keeps it off the horizon, number by number. That claim found the
     polar defect in step 5.
