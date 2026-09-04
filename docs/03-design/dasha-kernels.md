# Dasha kernels and tables

Status: `draft`, 2026-09-04. The falsification pass for ADR-0017 on the
dasha family: every catalogued system written as a row over a kernel, the
schema corrected where a system refused to fit, and the rows marked V, T
or S (ADR-0018). Implemented in Phase 5; the cursor and the exact period
arithmetic (`exact-arithmetic.md`) are Phase 1 types.

## Verdict

| | |
|---|---|
| systems examined | 56 (27 nakshatra-seeded, 27 sign-progression, Kalachakra, Sudarshana Chakra) |
| expressible as a row over the udu or rashi kernel | 50 |
| assigned to another kernel | Kalachakra (own kernel); Mudda, Varsha Narayana, Varsha Yogini and Tribhagi (the scale decorator); Sudarshana Chakra and probably Yogardha (composition) |
| genuinely resisted | Patyayini (periods from planetary strengths: its own small kernel); Panchaswara (shape not established) |
| corrections forced on the first schema | four (seed-to-lord map, balance window from the span, derived totals, three direction rules) |

The strongest evidence is in the baseline engine's own code: its
Padanadhamsa engine differs from its Chara engine in one expression (the
start sign is the arudha lagna), Niryana Shoola from Shoola in one
expression (the start sign is the navamsa lagna), and its proportional
nakshatra builder is one function parameterised by sequence, total,
start nakshatra and count direction that already serves six systems.

## Kernels

| kernel | family | shape |
|---|---|---|
| K-udu | nakshatra-seeded proportional cycles (seed may be a tithi, yoga or karana instead) | seed → lord by a map; balance from the elapsed fraction of the seed span; lords walk in sequence with year shares; children divide the parent proportionally |
| K-rashi | sign progressions (Jaimini) | start sign by rule; order by rule; period length by rule; sub-periods by rule; direction rules that differ per step |
| K-kalachakra | Kalachakra | navamsa-path driven, deha and jeeva, paramayush variants; its own kernel, parameterised for the known variants |
| scale decorator | Tribhagi, Mudda, Varsha Narayana, Varsha Yogini | a `CycleScale { factor, rounds }` over another definition: Tribhagi is a Vimshottari definition scaled by 2/3 and run twice; Mudda is scaled by one solar year over 120 |
| composition | Sudarshana Chakra (three simultaneous rashi progressions), Yogardha if it is the mean of two systems | a combinator over kernels, not an algorithm |
| tree layer | every system | lazy cursor, `dasha_at`, range iteration, search; written once against `roots` and `children` |

## K-udu: schema

```rust
pub struct UduDashaDef {
    pub id: DashaId,
    pub seed: SeedSource,            // Nakshatra | Tithi | Yoga | Karana | Fixed (no seed: runs from birth in order)
    pub map: SeedToLord,             // below
    pub lords: Vec<Lord>,            // in sequence
    pub periods: PeriodSource,       // Table(Vec<Ratio>) | FromChart(ChartQuery)   (Ashtakavarga dasha, Tara dasha)
    pub sub_start: SubStartRule,     // FromSelf | FromNext | FromNth(u8)
    pub balance: BalanceMethod,      // Spatial | Temporal; the window is `map.span` seed units wide, never per system
    pub year_length: YearLengthId,   // per system, from the profile's table (see the cruxes page)
    pub scale: Option<CycleScale>,   // Tribhagi, Mudda
    pub applicability: Option<RuleRef>,   // a rules-engine rule (Ashtottari's conditions)
    pub sources: Vec<Citation>,
    pub confidence: Mark,            // V | T | S
}

pub struct SeedToLord {
    pub reference: u16,              // seed index that maps to lord 0
    pub direction: CountDir,         // FromReference | ToReference (Dwadashottari counts to Revati)
    pub span: u8,                    // seed units per lord: 1 for most, 3 for Ashtottari
    pub offset: u8,                  // added after the modulo: 3 for Yogini
    pub overflow: Overflow,          // WrapToStart | Reject, explicit when span × lords < cycle
}
// lord_index = ((signed_count(seed, reference, direction) / span) + offset) mod lords.len()
```

The four corrections, each forced by one system:

1. **The seed-to-lord map is not `seed % lords`.** Systems count from
   different reference nakshatras, in different directions, covering
   different numbers of nakshatras per lord (Dwadashottari counts to the
   reference; Ashtottari gives each lord three nakshatras; Yogini adds
   three after the modulo).
2. **The balance window is `span` seed units wide.** When a lord covers
   three nakshatras the balance is the elapsed fraction of the
   three-nakshatra group, not of the current nakshatra. The baseline
   engine's Ashtottari engine says so in its own comment and computes
   `(position within group + fraction of the nakshatra) / 3`. A schema
   without `span` would have shipped an Ashtottari balance wrong by up to
   two thirds of a first mahadasha.
3. **`overflow` is explicit.** Ashtottari's eight lords times three cover
   24 of 27 nakshatras; a Moon in the other three (about 11% of births)
   must still terminate. The baseline engine wraps them to the start of
   the cycle; that is a choice, so the row names it and the result carries
   a flag saying the seed fell outside the classical cycle.
4. **Totals are derived.** `total` is `sum(periods)` and is asserted, not
   stored; two systems (Ashtakavarga dasha, Tara dasha) take their period
   lengths from the chart, so `periods` is a source, not always a table.

## K-udu: rows

Abbreviations: Su Mo Ma Me Ju Ve Sa Ra Ke. All rows below use
`sub_start: FromSelf`.

| system | seed | reference | dir | span | off | lords → years | total | mark |
|---|---|---|---|---|---|---|---|---|
| Vimshottari | nakshatra | Ashwini 0 | from | 1 | 0 | Ke 7, Ve 20, Su 6, Mo 10, Ma 7, Ra 18, Ju 16, Sa 19, Me 17 | 120 | V (baseline constants) |
| Ashtottari | nakshatra | Ardra 5 | from | 3 | 0 | Su 6, Mo 15, Ma 8, Me 17, Sa 10, Ju 19, Ra 12, Ve 21 | 108 | V (baseline; `overflow: WrapToStart`) |
| Dwadashottari | nakshatra | Revati 26 | to | 1 | 0 | Su 7, Ju 9, Ke 11, Me 13, Ra 15, Ma 17, Sa 19, Mo 21 | 112 | V (baseline) |
| Panchottari | nakshatra | Anuradha 16 | from | 1 | 0 | Su 12, Me 13, Sa 14, Ma 15, Ve 16, Mo 17, Ju 18 | 105 | V (baseline) |
| Shatabdika | nakshatra | Revati 26 | from | 1 | 0 | Su 5, Mo 5, Ve 10, Me 10, Ju 20, Ma 20, Sa 30 | 100 | V (baseline, Chaukhamba edition); `shatabdika-alt` swaps Mars 30 and Saturn 20 as a second row, T |
| Chaturashiti-sama | nakshatra | Swati 14 | from | 1 | 0 | Su Mo Ma Me Ju Ve Sa, 12 each | 84 | V (baseline) |
| Dwisaptati-sama | nakshatra | Mula 18 | from | 1 | 0 | Su Mo Ma Me Ju Ve Sa Ra, 9 each | 72 | V (baseline) |
| Yogini | nakshatra | Ashwini 0 | from | 1 | 3 | Mo 1, Su 2, Ju 3, Ma 4, Me 5, Sa 6, Ve 7, Ra 8 | 36 | V (baseline) |
| Tribhagi | nakshatra | Ashwini 0 | from | 1 | 0 | Vimshottari with `scale { factor: 2/3, rounds: 2 }` | 80 × 2 | V (baseline) |
| Shodashottari | nakshatra | Pushya 7 | from | 1 | 0 | Su 11, Ma 12, Ju 13, Sa 14, Ke 15, Mo 16, Me 17, Ve 18 | 116 | T (BPHS ch. 46; verse numbers to confirm before shipping) |
| Shattrimsha-sama | nakshatra | Shravana 21 | from | 1 | 0 | Mo 1, Su 2, Ju 3, Ma 4, Me 5, Sa 6, Ve 7, Ra 8 | 36 | T (BPHS ch. 46; verse numbers to confirm) |
| Shashtihayani | nakshatra | to confirm | | 1 | 0 | received text gives Ju 13, Su 13, Ma 13, then 6 each, which sums to 69, not 60 | 60? | S; see the cruxes page |
| Tithi-Ashtottari | tithi | to confirm | | 1 | | as Ashtottari | 108 | T |
| Tithi-Yogini | tithi | to confirm | | 1 | | as Yogini | 36 | T |
| Yoga-Vimshottari | yoga | to confirm | | 1 | | as Vimshottari | 120 | T |
| Karana-Chaturashiti | karana | to confirm | | 1 | | as Chaturashiti-sama | 84 | T |
| Naisargika | fixed | | | | | natural order, lifespan periods | | S (lord order and years) |
| Tara | nakshatra | | | | | `periods: FromChart` (tara counts) | | S |
| Kaala, Rashmi, Buddhi-Gathi, Moola, Saptarishi | nakshatra | | | | | udu-shaped | | S |
| Karaka | | | | | | lord order from chara-karaka strength: a third chart-query field, the kernel's watch item | | S |
| Aayu | | | | | | linked to the longevity module | | S |
| Ashtakavarga dasha | | | | | | `periods: FromChart` (bindu counts) | | S |
| Panchaswara | | | | | | shape not established | | S |

Two observations from normalising the rows: Yogini and Shattrimsha-sama
are the same table (lords, order and years identical) differing only in
`reference` and `offset`, which is what a per-system architecture would
implement twice; and the four seed-variant systems (tithi, yoga and
karana seeds) are rows that change one field. The reference index does
not carry over between seed kinds because the cycles differ (27, 30, 60),
so those rows wait for a text.

## K-rashi: schema and the direction error

A first draft had one `direction` field. There are three independent
direction rules in a Jaimini dasha, and they use two definitions of
"odd":

| rule | governs | definition | baseline engine |
|---|---|---|---|
| 1 | the duration count from a sign to its lord | footedness, by threes from Aries: Aries to Gemini odd-footed, Cancer to Virgo even-footed, Libra to Sagittarius odd, Capricorn to Pisces even | `jaimini-dasha-base.engine.ts`, with the comment that this is not the plain odd/even test |
| 2 | the mahadasha sequence direction | plain parity of the start sign | `chara.engine.ts` |
| 3 | the antardasha direction | plain parity of each period's own sign | `jaimini-dasha-base.engine.ts` |

Collapsing any pair produces charts that agree with reference software
for some inputs and not others. `Footedness` and `Parity` are therefore
distinct types, and a compile-fail test proves they cannot be swapped.

```rust
pub struct RashiDashaDef {
    pub id: DashaId,
    pub start: StartSignRule,        // Lagna | ArudhaLagna | NavamsaLagna | Karakamsha | SreeLagna | VarnadaLagna | Stronger(a, b) | ...
    pub order: OrderRule,            // Consecutive | TrineGroups | DrishtiChain { seeds } | Leap { step, passes } | KendraGroups | Paryaya
    pub seq_direction: ParityRule,   // rule 2
    pub length: PeriodLengthRule,    // FootedCountToLord { own_sign_years, exaltation_adjust } | Fixed(u8) | ByModality { movable, fixed, dual } | FromChart(ChartQuery)
    pub sub: SubProgressionRule,     // Equal12 { direction: ParityRule } | Table(Box<[[u8; 12]; 12]>)
    pub exceptions: Vec<ProgressionException>,   // { when: RuleRef, use_sub: SubProgressionRule }
    pub year_length: YearLengthId,
    pub applicability: Option<RuleRef>,
    pub sources: Vec<Citation>,
    pub confidence: Mark,
}
```

`SubProgressionRule::Table` and `exceptions` exist because a third-party
implementation carries permutation tables for Narayana antardashas that
no parity rule generates, with separate tables selected by Saturn's and
Ketu's placement. Whether those tables are a school, a misreading or a
gap in the baseline engine is an open crux; the kernel must be able to
express them either way, and the baseline engine's `Equal12` stays the
default until a primary text decides.

## K-rashi: rows

| system | start | order | length | sub | mark |
|---|---|---|---|---|---|
| Chara | lagna | Consecutive | FootedCountToLord, own 12, exaltation off | Equal12, per-sign parity | V (baseline) |
| Narayana | lagna | Consecutive | FootedCountToLord, own 12, exaltation on | Equal12; `narayana-table` registered as S | V (baseline) |
| Padanadhamsa | arudha lagna | Consecutive | FootedCountToLord | Equal12 | V (baseline) |
| Trikona | lagna | TrineGroups | FootedCountToLord | Equal12 | V (baseline) |
| Drig | lagna | DrishtiChain [9, 10, 11] | FootedCountToLord | Equal12 | V (baseline) |
| Shoola | lagna | Consecutive | Fixed 9 | Equal12 | V (baseline; 9 × 12 = 108 falls out) |
| Niryana Shoola | navamsa lagna | Consecutive | Fixed 9 | Equal12 | V (baseline) |
| Mandooka | lagna | Leap { step: -2, passes: 2 } | ByModality 7/8/9 | Equal12 | V (baseline; (7+8+9) × 4 = 96 falls out) |
| Sthira | lagna | Consecutive | ByModality 7/8/9 | Equal12 | T |
| Sudasa | Karakamsha or Sree lagna | Consecutive | FootedCountToLord | Equal12 | T |
| Varnada | Varnada lagna (five school variants of the lagna itself) | Consecutive | FootedCountToLord | Equal12 | T |
| Lagna Kendradi, Karaka Kendradi, Kendradi Rashi | lagna, atmakaraka's sign, lagna | KendraGroups | FootedCountToLord | Equal12 | T |
| Navamsa dasha, Lagnamsaka | navamsa lagna | Consecutive | to confirm | | S |
| Brahma, Chakra, Nirayana, Paryaya, Raashiyanka, Sandhya, Tara Lagna, Chathurvidha Utthara, Moola (rashi) | to confirm | | | | S |
| Yogardha | | | mean of two systems, if confirmed a composition | | S |
| Kalachakra | | own kernel; the baseline engine has it with its constants and tests | | | V (baseline) |
| Sudarshana Chakra | lagna, Sun, Moon | three simultaneous rashi progressions | | | T |
| Mudda, Varsha Narayana, Varsha Yogini | | scale decorator over the base definition | | | V (Mudda, baseline), T |
| Patyayini | | own kernel: periods from planetary strengths in the annual chart | | | T |

Chara and Narayana differ in one boolean; Padanadhamsa and Chara in the
start rule; Shoola and Niryana Shoola in the start rule; the three
Kendradi systems in the start rule. Eight of the baseline engine's
classes reduce to eight rows over four field values.

## Invariants asserted over the whole table

K-udu: `periods.len() == lords.len()`; `sum(periods) == total` exactly in
`Ratio`; `map.span × lords.len() <= seed.cycle()`; if less, `overflow` is
set explicitly; `map.reference < seed.cycle()`; the balance window is
`map.span` units and no row defines its own; every seed index yields a
valid lord (enumerated: 27, 30 or 60 cases); `sum(children) == parent`
exactly at every depth.

K-rashi: `order` visits all twelve signs exactly once from every start
sign (`Leap { step, passes }` can express a tiling that fails, and a
failing row is rejected at load); `length` returns 1 to 12 for every
(sign, lord position); footedness and parity are distinct types (a
`trybuild` compile-fail test); `sum(antardashas) == mahadasha` exactly;
every `exceptions[].when` resolves to a loadable rule.

## The cursor

```rust
pub trait DashaSystem {
    fn roots(&self, ctx: &ChartCtx) -> Result<Vec<Period>>;
    fn children(&self, ctx: &ChartCtx, parent: &Period) -> Result<Vec<Period>>;
}

impl DashaCursor<'_> {
    pub fn at(&self, t: Instant, depth: Depth) -> Result<Chain>;                 // O(children × depth), no allocation after warm-up
    pub fn expand(&self, p: &Period) -> Result<Vec<Period>>;                    // one level
    pub fn iter_range(&self, r: TimeRange, depth: Depth) -> impl Iterator<Item = Result<Period>>;   // prunes subtrees outside the range
    pub fn find(&self, pred: impl Fn(&Period) -> bool, depth: Depth) -> impl Iterator<Item = Result<Period>>;
    pub fn materialise(&self, depth: Depth, window: TimeRange) -> Result<Tree>; // explicit cost
}
```

Across the C ABI the cursor is an opaque handle with `next_batch(n)`
(`06-api-conventions.md`); the `dasha_tree` entry point takes an explicit
depth and window. Budgets: `at(t, 5)` under 20 microseconds with zero
allocations; a materialised depth-3 tree under 500 microseconds; the
per-request position cache keeps the ephemeris call count at one per
foundation regardless of depth.

## Tests and golden vectors

Unit tests per kernel over its parameter space; the whole-table
invariants above; golden vectors for the eighteen baseline systems from
spike 1 (every level, with the balance method and year length recorded);
property tests: any instant inside a period is found by `at`; `at(t, d)`
agrees with walking `materialise(d, window)`; boundary generators at
plus and minus one microarcsecond of every nakshatra boundary and one
millisecond of every period boundary.

## Localisation

Keys under `sdk.dasha.<id>` for system names and `sdk.entity` for lords;
period levels as an ordinal message with the profile's level names.

## What resisted and what is watched

- **Patyayini** needs its own kernel (periods from strengths); planned.
- **Panchaswara** has no attested shape here; stays S.
- **Karaka dasha** orders lords by chara-karaka strength; that would be a
  third chart-query field in K-udu. It is not added until Ashtakavarga,
  Tara and Karaka are implemented together, at which point either one
  chart-query mechanism serves all three or the kernel is redesigned as
  an interpreter (ADR-0017's kill criterion).
- **Year length per system** is not settled by any reference read so far;
  savana 360 against 365.25 compounds to about 21 months over a 120-year
  cycle. Resolved before any dasha conformance run (cruxes page).
- **Applicability rules** live in the rules engine, not here.

## Open questions

Tracked on `01-research/feature-universe/19-verification-cruxes.md`:
Shashtihayani, Narayana antardasha tables, Ashtottari applicability,
Shatabdika edition, year length per system, the seed references for the
tithi, yoga and karana variants.
