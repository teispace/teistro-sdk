# The Tajika sahams

Status: built — 2026-09-23. `crates/tajika/src/saham.rs`, reached at
`sdk.chart().sahams`, `sahams_with_rules` and `saham_point`, and across
the boundary since the same day as `varsha_json.sahams` (see "Crossing the
boundary"). What each reading moves is measured over every recorded
birth's forty years in [`muntha-measured.md`](muntha-measured.md) §14,
where the counts are generated and cannot go stale; none are written here.

The source is K. S. Charak, *A Textbook of Varshaphala*, ch. XI, "based
mainly on the Tajika Neelakanthi" (rank 2; see the research page,
[`07-tajika-varshaphala.md`](../01-research/feature-universe/07-tajika-varshaphala.md)).

## One shape, forty-one tables of it

A saham is a sensitive point, and every one the source gives is the same
arithmetic. Call the three factors *a*, *b* and *c*, as the source does:
the saham is **a − b + c**, and "if *c* does not fall between *b* and
*a*", counted in the order of the signs from *b*, it is carried **one sign
further**. What distinguishes Punya from Vivaha is only which three points
they read and whether a year opening at night reads them another way.

So a saham is **data**:

| type | what it is |
|---|---|
| `SahamTerm` | one factor: the lagna, one of the seven, a house's point, a house's lord, the lord of a planet's sign, another saham, or a fixed degree |
| `SahamTriple` | a, b and c |
| `SahamFormula` | a triple by day and one by night; `same` and `reversed_by_night` build the two shapes the source uses, and Karya-siddhi, whose night reads the Moon's lord where its day reads the Sun's, spells both |
| `Saham` | the source's forty-one, in its order, the id of each its number less one |

`Saham::formula` is the table. A caller with a saham the source does not
give — another authority's Apamrityu, or a different reading of one of
these — writes a `SahamFormula` and hands it to `saham_point`, the same
evaluator the forty-one go through. The source says itself that
Neelakantha gives fifty, Venkatesha forty-eight and Keshava twenty-five;
a closed list of forty-one would be a dead end, and this is not one.

Three pairs share a formula and are kept as two sahams each — Vidya and
Guru, Raja and Pitri, Kshama and Kali — because the source names each for
a different matter and a reader asks for the matter. A test holds that
each pair's formulas are one, and the measured page that their counts are.

**A saham that reads another reads only an earlier one** in the source's
order (Yasha, Mitra, Mahatmya, Preeti and Bandhana read Punya, Guru or
Vidya). A test holds that for every formula under every reading, which is
what guarantees evaluation ends; each chart's sahams are memoised, so a
caller asking for all forty-one computes Punya once.

## Three readings, each a rule

| field | default | rivals | why the default |
|---|---|---|---|
| `add_sign` | `Degrees` | `Signs`, `Never` | every worked saham the source prints |
| `houses` | `Sripati` | `Chalit`, `Equal` | the source's own mid-points, built from the angles |
| `roga` | `Lagna` | `Saturn` | the formula the source gives as the saham |

### When the sign is added

The source states the rule once and works it seven times: Punya, Raja and
Matri in its Example Chart (the forty-first year, by day), Punya and Raja
in the forty-sixth (by night, Punya taking the sign), Punya and Mrityu in
the forty-seventh (by night), and Punya in the birth chart. **All seven
come back to the printed arcminute** from the printed longitudes, with
the sign added exactly where the source adds it, and four of them come
back end to end from a birth and a year the SDK founded itself.

**A widely used program reads "between" in whole signs.** Measured as a
black box over 600 random charts (values only, CLEAN_ROOM rule 3), it
adds the sign unless *c*'s sign is after *b*'s and up to *a*'s, *a* and
*b* in one sign counting as the whole circle — a rule that fits all 600
and that no worked example states. **The source's birth chart refutes it**:
the Sun, the lagna and the Moon all stand in Leo, at 3°49′, 14°33′ and
17°08′, the lagna is between by degrees and not by signs, and the source
prints Leo 27°52′ with no sign added. It ships as `AddSign::Signs` so a
consumer can reproduce what that program prints, and a test reproduces
sixteen of its sahams by day and by night to 1e-9 under it. `Never` is
a − b + c alone, as other traditions read their lots.

A point standing **on** an end of the arc is between. That matters only
where a formula reads one factor twice — Roga's first reading, whose *c*
is its *a* — and it keeps that saham from taking a sign it has no arc to
take it from.

### Where a house stands

Five sahams read a house past the first: Mrityu, Deshantara, Artha,
Santaapa and Labha. (Samarthya and Manmatha read "the lord of the
ascendant", the first house's, which is the lagna's under every reading.)
The source calls the point a house's **mid-point** and builds it itself:
the lagna and the midheaven and their opposites are the kendras'
mid-points, and each quadrant between is cut in three — Sripati's
division. `sripati_mid_points` is that recipe, and it reproduces all
twelve mid-points the source prints for its Example Chart from its lagna
and its tenth house, to the arcminute.

**The default is built from the chart's angles and not read off its
chalit**, and the measurement is why. A chart already carries a chalit,
and under the default profile it is Sripati's, so the first build read
the bhava middles off it. The measured page then said equal houses moved
**no** house saham in any chart: the conformance profile's chalit is
**Vehlow's**, equal houses centred on the lagna, and so is
`nepali-default`'s, so under those profiles "the chart's middles" *were*
equal houses and the source's reading silently was not the default. The
façade now recomputes the chart's midheaven from its instant and place —
`teistro_chart::foundation::angles_of`, which needs no ephemeris, the one
computation behind `Founder::angles_at` — and builds Sripati's from it,
so the default is the source's under every profile. A test changes only
the chalit and holds the sahams equal; read off the chalit, equal houses
now move the house sahams in about a fifth of placements (§14).

| `HousePoints` | the point |
|---|---|
| `Sripati` | the default: the source's trisection, from the lagna and the midheaven |
| `Chalit` | the chart's own chalit middles, under whatever division its profile names |
| `Equal` | (n − 1) × 30° past the lagna's degree, for a caller with no midheaven and for the program above, which reads only planets and a lagna |

The source's Chart X-9 prints an eighth mid-point, Capricorn 4°12′ under
a lagna of Gemini 7°15′, that no equal division gives: equal houses put
its Mrityu 3°03′ further on.

A house's **lord** is the lord of the sign its point falls in, so one
reading governs both the point and whose planet is read with it.

### Roga

The source gives lagna − Moon + lagna, then "according to another
authority" Saturn − Moon + lagna, and adds "we have found this latter
giving better results". The first is the saham as given, so it is the
default; the second is a field away.

## What a saham answers with

`SahamPlace`: its longitude, its sign, that sign's lord (the **saham
lord**, by whose strength the source judges it), its house counted in
whole signs from the lagna as the source says a saham "falls in the tenth
house", and whether the sign was added. `SahamReading` carries the
places asked for in the order asked, whether the chart was read by day,
and the rules it was read under.

**Any chart**: the source reads an annual chart's sahams beside the birth
chart's, since "only those Sahams which are strong in the birth chart can
produce results during a given year", so the façade takes a `Document`
and not an annual one.

## Crossing the boundary

A caller outside Rust asks for a year's sahams on the chart request, as
it asks for its matters:

| `varsha_json` | what it is |
|---|---|
| `sahams` | `"all"`, the forty-one in the source's order, or saham keys in the caller's order; absent, none is read. Needs `place`, since a saham is read from the year's own chart. A saham named twice is refused. |
| `sahamRules` | `{addSign, houses, roga}`, each a named reading, every one defaulted to the source's |

Each year's `annual_charts` row counts its sahams in `saham_count`, and
`year_sahams` carries them ragged beneath it: the saham, its longitude,
sign, lord and house, and whether a sign was added. Whether the year
opened by day is `annual_charts.daylight`, which is not repeated.

**A saham is named by its catalogue key**, the kebab-case `karya-siddhi`
every binding reads one back as, so a caller can hand back what an answer
gave it. The first cut spelt the wire in serde's snake case, as the rule
words are, and a Node caller writing the `Saham` type's own
`'karya-siddhi'` would have been refused — the `noneAspects` trap of the
yogas' crossing in another shape. The ABI test holds the two spellings
equal member for member.

`matters` and `sahams` are one reader, `Asked<T>`: `"all"` or these, none
twice, each member read as its field reads it — a house by number, a
saham by key.

**Not across**: a caller's own `SahamFormula`, which Rust has at
`saham_point`, and the birth chart's sahams, which the source reads beside
the year's. Both wait on a consumer asking; the second will cross with
saham strength, which needs it.

## What is decided and what is not

| | |
|---|---|
| **decided by the source** | the forty-one formulas; that the added sign is judged by degrees (seven worked examples); that a house's point is its Sripati mid-point, built from the angles (all twelve printed mid-points, and Chart X-9's) |
| **decided by measurement** | that the rival "between" is a whole-sign rule (600 of 600 probes) and what it moves (§14); that the default must not read the chart's chalit, which is Vehlow's under two shipped profiles |
| **a reading, not a decision** | Roga's two formulas, both the source's |
| **built** | `Saham` with `ALL` and `formula`; `SahamFormula`, `SahamTriple`, `SahamTerm`; `sahams` and `saham_point`, memoised per chart; `SahamRules` with `AddSign`, `HousePoints` and `RogaReading`; `sripati_mid_points`; `teistro_chart::foundation::angles_of`; the façade's `sahams`, `sahams_with_rules` and `saham_point`, which need no ephemeris |
| **not built** | a saham's **strength**, which the source judges by its lord's dignity, its associations and aspects, the Panchavargiya floor of five units, and the **Harsha bala** — which is not built, so strength waits on it; a caller's own formula and the birth chart's sahams across the boundary |

## The order of work

1. **Done**: the arithmetic, the table, the three readings, the façade.
2. **Done**: the crossing, `varsha_json.sahams` and `sahamRules`, into
   every binding with its parity keys.
3. The Harsha bala (Charak ch. VI), then a saham's strength over it and
   the Panchavargiya bala already built, crossing with the birth chart's
   sahams.
