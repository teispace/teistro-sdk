# KP: Krishnamurti Paddhati

Status: `draft`, 2026-09-29; §6 steps 1 to 3 **built** by 2026-09-30. Written
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
| C150 | how many ruling planets | the Readers' **five** | `kp.ruling_planets = WITH_SUBS`, the baseline's seven |
| C151 | the day lord's day | **sunrise to sunrise**, the panchanga's vara | `CIVIL`, the weekday of the clock |
| C152 | a node among the rulers | **in a ruler's sign, or in the same sign as one** (the worked example's conjunction) | the sign only |
| C153 | which retrograde rulers are rejected | **in a retrograde planet's star**, the rule as stated; the sub reported beside it | `STAR_OR_SUB`, the later procedure |
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
- **The boundary** (step 6): one chart section, the bindings, parity.

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
4. Ruling planets, over the panchanga's vara for C151.
5. Horary from a number.
6. The boundary and the bindings.
7. The measured pass: over the recorded births, each sub lord's margin
   in minutes of birth time (how many charts a minute's error changes),
   and how many sub lords the Krishnamurti and VP291 ayanamshas part on.
