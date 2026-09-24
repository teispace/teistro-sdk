# Dasha kernels and tables

Status: `draft`, 2026-09-04; the Vimshottari row measured 2026-09-15, a consumer's
own system registering the same day
([`dasha-measured.md`](dasha-measured.md), §"What the measurement
settled"). The falsification pass for ADR-0017 on the
dasha family: every catalogued system written as a row over a kernel, the
schema corrected where a system refused to fit, and the rows marked V, T
or S (ADR-0018). Implemented in Phase 5; the cursor and the exact period
arithmetic (`exact-arithmetic.md`) are Phase 1 types.

## Verdict

| | |
|---|---|
| systems examined | 56 (27 nakshatra-seeded, 27 sign-progression, Kalachakra, Sudarshana Chakra) |
| expressible as a row over the udu or rashi kernel | 50 |
| assigned to another kernel | Kalachakra (own kernel); Mudda, Varsha Yogini and Patyayini (the year kernel, `YearDasha`: corrected when built, see [`annual-dashas.md`](annual-dashas.md)); Tribhagi and Varsha Narayana (the scale decorator); Sudarshana Chakra and probably Yogardha (composition) |
| genuinely resisted | Panchaswara (shape not established) |
| corrections forced on the first schema | four (seed-to-lord map, balance window from the span, derived totals, three direction rules) |

**This table examines the systems; it does not say which are built.** The
56 it counts are what the survey read, which is more than the catalogue
names and far more than this build computes — three numbers, and prose
is where they go out of step. Which catalogued system this build computes
and what keeps each of the rest out is counted from the types in
[`dasha-coverage-measured.md`](dasha-coverage-measured.md), which also
asks for every unbuilt one on a real chart to check that the gap is a
declared refusal rather than a dead end.

The strongest evidence is in the baseline engine's own code: its
Padanadhamsa engine differs from its Chara engine in one expression (the
start sign is the arudha lagna), Niryana Shoola from Shoola in one
expression (the start sign is the navamsa lagna), and its proportional
nakshatra builder is one function parameterised by sequence, total,
start nakshatra and count direction that already serves six systems.

## What the measurement settled

The corpus records Vimshottari with its inputs and its whole tree under
both balance methods, so `cargo xtask dashas` decided what this page had
only designed. Four of its findings change the design:

- **The birth period's sub-periods are compressed**, each its share of the
  period it is in, the birth period being only as long as its balance. The
  other reading (sized against the whole period, the elapsed ones dropped)
  is refused by all 148 recorded answers, but both are taught and neither
  by a rank-1 text, so K-udu gains `birth_period: Compressed | Elapsed`,
  the knob `dasha.birth_period`, and crux C48.
- **The cycle ends.** Past the ninth mahadasha the recording engine answers
  no period. The cursor answers `None` there by default, and
  `dasha.after_cycle: End | Repeat` makes the other reading reachable.
- **Boundaries agree to a quarter of a millisecond and not to the bit**,
  under any order of float arithmetic. That confirms the `Ratio` shares
  below: a boundary is its parent's start plus an exact share of its
  length, compared with the corpus to its tolerance (a thousandth of a
  day).
- **The balance is written with its minutes rounded**: whole years of the
  year length, whole months of a twelfth of it, whole days, and the rest
  rounded to the minute. Flooring disagrees with half the records.

What it could not settle is crux C6: every record uses 365.25 days.

## What building Vimshottari corrected

`crates/dasha` builds the K-udu kernel with Vimshottari as its one row, and
reproduces every recorded answer (`crates/dasha/tests/baseline.rs`: 148
methods, 53 415 tree rows, 14 841 sampled children and 296 chains, worst
boundary 1.4e-9 days). Building it corrected the schema once more:

- **A fifth correction: whether the lords repeat.** The first seat rule
  flagged every Vimshottari seed past the ninth nakshatra as overflowing,
  because nine lords of one nakshatra cover nine. They run round three
  times. Ashtottari's eight windows of three cover 24 once and leave three
  outside; Yogini's eight lords do not divide 27 and repeat. No arithmetic
  on the table tells these apart, so the row states `repeats`.
- **A temporal balance over a window of nakshatras was refused**, because the
  corpus then recorded the temporal method for Vimshottari only and no
  source read defined it. The corpus's 0.2.0 records the recording engine's
  reading for Ashtottari, and building the other rows took it (below).
- **The elapsed reading keeps a period's place.** Under
  `dasha.birth_period = ELAPSED` the first sub-period running at birth may
  be the fourth in its sequence, and its path says so. A period carries
  the whole span its children are shares of beside the span it runs for.
- **The cursor allocates nothing.** Making a dasha allocates two small
  tables; reading the chain at an instant, at any depth and in any cycle,
  allocates nothing (`tests/allocations.rs`).

## What building the other rows corrected

`cargo xtask dasha-systems` (`dasha-systems-measured.md`) measured the eight
other nakshatra-seeded systems the recording engine implements, 1184 answers
computed from the corpus's own recorded Moon, and `crates/dasha` ships them
as rows reproducing every one (`tests/systems.rs`):

- **Every one is a row.** Ashtottari, Dwadashottari, Panchottari,
  Shatabdika, Chaturashiti-sama, Dwisaptati-sama, Yogini and Tribhagi take
  Vimshottari's seat, balance, compressed birth period, children and cycle
  end; the corpus decides five of the seats outright and cannot choose
  between the equivalent references of the other three.
- **A sixth correction: a scale on a row.** Tribhagi is Vimshottari's lords
  at two thirds of their years, twice round, the sub-periods still shares of
  120; a third of the years three times round is refused by every answer.
  The scale decorator the kernel table below names became two fields of the
  row, `scale` and its `rounds`, rather than a wrapper, because nothing else
  about the tree changes.
- **A temporal balance over a window reads the Moon's own nakshatra.** The
  window's whole nakshatras behind the seed are gone and the Moon's own is
  gone by the time it has spent in it. Every Ashtottari answer agrees; the
  window's remainder taken from the time fraction alone is refused; the Moon's
  time across the whole window is not recorded and is not built.
- **A child follows its parent's place in the row, not its graha.** Looking a
  lord up by its graha is ambiguous for a row that names a graha twice, so a
  period carries its lord's place and its children start from it.

## What building the sign-based rows corrected

`cargo xtask rashi-dashas` (`rashi-dashas-measured.md`) measured the
recording engine's eight sign-based systems over 616 answers computed from
the corpus's own charts, and `crates/dasha`'s `rashi` module ships them as
rows reproducing every one (`tests/rashi.rs`, worst boundary 4.7e-10 days):

- **The schema came out smaller than drafted.** Every system is a start
  (lagna, arudha lagna, navamsa lagna), an order (consecutive, trine
  groups, a drishti chain, a leap), a length (the count to the lord, the
  same by dignity, fixed, by modality) and which lord a mahadasha names. The
  draft's per-step direction rules and sub-progression tables are not
  fields yet: every recorded system runs one direction rule and one
  sub-progression, and each rival a school teaches is a crux (C49–C53, C2)
  rather than a field nobody can fill.
- **Footedness and parity are distinct types**, as the direction error
  above predicted; counting by parity is refused on all 385 counted
  answers.
- **Drig's chain does not always reach twelve signs.** For 33 of 77 charts
  the eleventh house aspects the ninth, and the engine appends the signs
  left out in the zodiac's order (C52).
- **A sign-based dasha allocates nothing**, its tables fixed arrays, and it
  shares `Period`, `Chain` and the chain walk with the nakshatra-seeded
  kernel through the `Timeline` trait.
- **The arudha lagna is a point.** Padanadhamsa starts from it, and
  `teistro-points` computes it (`arudhas-measured.md`).

## What building the Kalachakra corrected

`cargo xtask kalachakra` (`kalachakra-measured.md`) measured the recording
engine's Kalachakra over 148 answers, and `crates/dasha`'s `kalachakra`
module reproduces every one (`tests/kalachakra.rs`, worst boundary 9.3e-10
days), allocating nothing:

- **Its own kernel, as the table above says**, but a small one: four pada
  tables of nine signs, the signs' years, a balance, and antardashas shared
  by years from the mahadasha's place. It shares `Period`, `Chain`, the
  chain walk and the span check with the other kernels through `Timeline`
  and `Birth`.
- **Every fork is a knob.** The published tables agree with the engine's
  sign for sign, but which table five nakshatras take, the balance and what
  follows the ninth mahadasha are read differently by the sources, so
  `dasha.kalachakra_membership`, `dasha.kalachakra_balance` and
  `dasha.kalachakra_after_ninth` default to the engine's reading and offer
  the sources' (C54–C56). The next pada's nine is not offered: the sources
  read do not settle how it crosses into the next nakshatra.
- **It stops at the antardashas** (C58), and a reading says so: its depth is
  the shallower of the settings' and two.
- **The document records the choices applied** in a reading's `kalachakra`
  field, so a stored document rebuilds the same periods whatever the
  settings are now.

## What BPHS ch. 46 settled for the sign-based rows (2026-09-15)

The corpus's rows reproduce the recording engine, and two of its readings
waited on a source: the stronger lord of Scorpio and Aquarius (C51) and where
the systems that start from a stronger sign begin (C53). BPHS ch. 46 gives
both, so the text's reading is the default and the engine's is
`conformance-baseline`'s, the rule every strength module already follows.

- **The sign's strength** is `rashi::stronger_sign` (`strength-schemes.md`,
  "Rashi bala"), a comparison and not a score, because the verses give no
  score.
- **`dasha.dual_lord = BPHS`**: a lord standing in the sign counts to the
  other; else the lord in the stronger sign; on equal signs the lord the
  greater count reaches. `KENDRA` is the engine's kendra rule. Every row that
  counts to or names a stronger lord reads it.
- **`dasha.rashi_start = STRONGER`**: Mandooka from the stronger of the lagna
  and the seventh, Shoola of the second and the eighth, Trikona the strongest
  trine, the earlier sign on equal strength; the row says which houses in
  `stronger_of`. `LAGNA` is the engine's.
- **A reading records them** in `DashaReading.rashi`, so a stored document
  rebuilds under the readings it was computed with; a document from before
  the field existed was the engine's, which is what an absent field means.

Over the corpus's 616 answers the dual-lord rule moves 360 and the start 98
of the three systems' 231 (`tests/rashi.rs`, both pinned). The year the verses
add for an exalted graha and take for a debilitated one is not built: they do
not say whose sign it is (C51).

## At the boundary and in the bindings

A chart request names its systems (`TsChartRequest.dashas`, catalogue ids,
each refused by its place when no row implements it) and the charts blob
answers in two sections. `dashas` (23) holds one fixed row per chart per
system, charts outermost: the seed, first lord, overflow, the balance with
its method (the boundary's own `TsBalance`, since the settings' `Balance`
is a knob and not a catalogue member) and written form, the Moon's span
(NaN when the balance was spatial), the depth and `period_count`.
`dasha_periods` (24) is **ragged** by `period_count`: each period as its
`level`, its `index` under its parent, its lord and its span, depth first.
A path is not carried as text: it is the index appended to the path of the
nearest earlier period one level up, which each binding rebuilds while it
decodes, and the chain at an instant is one walk over the same rows. The
batch's `dasha_count` is one number because the request settles it, and
the encoder refuses a batch whose charts disagree rather than writing a
stride that lies.

Node (`chart.dashas`), Python (`Chart.dashas`) and Dart (`Chart.dashas`)
decode the sections once per batch, and `check-parity` holds all four
surfaces, Rust included, to the same seeds, balances, 819 periods per
chart and chains.

Every row reaches the boundary the same way: a request names any of the
seventeen, and a system the catalogue names with no row, Kalachakra for one,
is refused by its place with the built systems as the hint. A sign-based
dasha has no seed and no balance, so the `dashas` row says which of its
columns hold: `seeded` for the seed, overflow and balance, and `signed` for
the periods' `sign` column — two flags and not one family, because Tribhagi
is seeded and scaled and Kalachakra will be seeded and signed. The façade
reads the chart a sign-based dasha needs from the foundation: the grahas'
signs and dignities under the settings, the navamsa lagna, and the arudha
lagna `teistro-points` computes.

## A consumer's own system (2026-09-15; both kernels 2026-09-22)

Phase 5's exit asks for "a consumer-registered dasha system (a row in a
consumer pack)". A K-udu system is already data, so the SDK takes one the
way it takes a consumer's chart layout (ADR-0026 §1): a definition checked
by the rules a shipped row passes, registered on the context before it is
built, sealed after, and asked for by key.

**A K-rashi system is data too, and for eight months it could not be
registered.** That asymmetry was invisible until
[`dasha-coverage-measured.md`](dasha-coverage-measured.md) counted what
this build computes against what the catalogue names: of the twenty-two
systems left, three could be supplied by a consumer holding the text and
nineteen by nobody — and `Sthira` and `Varnada` were in the second group
only because there was no definition for them to arrive as. **The text
being unsettled blocks this build; a missing definition blocks everyone**,
and only the second is the SDK's to fix. `RashiDefinition` closes it, and
the count on that page moved from three and nineteen to five and
seventeen.

- **The kernel is stated, not guessed.** A definition crosses as a
  `DashaDefinition`, internally tagged by `kernel` — `udu` or `rashi`. The
  two rows are structurally disjoint, so a reader *could* tell them apart
  by which fields are present; it would then read a row with a typo in
  `lords` as sign-based and refuse it by a field the caller never wrote,
  which is the error message a consumer cannot act on. One tag buys the
  refusal that names what they did write.
- **One registry, not two.** A consumer registers *a dasha system*; which
  kernel the SDK files it under is the SDK's business. Two registries
  would be two things that can disagree about whether a key is taken, and
  a catalogued key must be refused whichever kernel asks.
- **The nakshatra-seeded definition** is `teistro_dasha::UduDefinition`,
  serde and a JSON Schema: `key`, `sources`, `lords` (`{graha, years}` in order),
  `reference` (a nakshatra, not an index), and `count`, `span`, `offset`,
  `repeats` and `scale`, each defaulting to Vimshottari's shape, plus its
  own `year_length` and `depth`, since the settings' per-system tables are
  keyed by the catalogue. `UduRow::validate` is its validation, so a
  registered row is refused by the same field a shipped one would be.
- **The sign-based definition** is `teistro_dasha::RashiDefinition`, the
  same shape for the other kernel: `key`, `sources`, `start`, `order`,
  `length`, `named_lord` and `stronger_of`, plus its own `year_length` and
  `depth`. Everything unsaid is Chara's, which is the family's ordinary
  row, so the smallest useful definition is a key. `RashiRow::validate` is
  its validation, and a sign-based row has no lords and no seed to get
  wrong, so what it checks is the two places a number can be: a length of
  no years, and a house outside one to twelve — or one house alone, whose
  strongest is itself, or one named twice. `RashiRow` gained a
  `Cow<'static, [u8]>` and a `DashaName` to be both shipped and owned, and
  a shipped row still clones without allocating, which the allocation
  tests hold.
- **Identity.** A row is named by a `DashaName`: a catalogued
  `DashaSystem`, or a registered key. It serialises as the bare key either
  way, so a document spells `VIMSHOTTARI` exactly as before and a
  consumer's system as `ACME_SAPTA`. The registry refuses a key the
  catalogue already has, so the two can never be confused. The row keeps
  its lords as a `Cow<'static, [Lord]>`: borrowed for a shipped row and
  owned for a registered one, with one kernel for both.
- **The document stays self-contained.** A registered system's reading
  carries its `definition` beside its rules, so a stored document rebuilds
  its cursor with the definition it was computed from, whatever the
  context has registered since. This follows the Kalachakra, whose reading
  carries its own choices.
- **The request.** `ChartRequest::with_dashas` takes anything that is a
  `KeyId`: a catalogue member as before, or the id a context gave a
  registered system (`sdk.keys().id("dasha_system.ACME_SAPTA")`). At the
  boundary a request's `dashas` array already carries ids, and a
  registered id is `0x8000` or more. `TsContextOptions.dashas_json` is a
  JSON array of definitions, refused by index and field as
  `layouts_json` is. The `dashas` section's `system` column carries the
  registered id, and each binding names it from the definitions it passed.
- **Scope.** Only nakshatra-seeded (K-udu) systems register. A K-rashi row
  is code as much as data (its start, order and length rules are enums
  that read a chart), and the Kalachakra is its own kernel. Registering
  either waits for a consumer who needs it.

## Kernels

| kernel | family | shape |
|---|---|---|
| K-udu | nakshatra-seeded proportional cycles (seed may be a tithi, yoga or karana instead) | seed → lord by a map; balance from the elapsed fraction of the seed span; lords walk in sequence with year shares; children divide the parent proportionally |
| K-rashi | sign progressions (Jaimini) | start sign by rule; order by rule; period length by rule; sub-periods by rule; direction rules that differ per step |
| K-kalachakra | Kalachakra | navamsa-path driven, deha and jeeva, paramayush variants; its own kernel, parameterised for the known variants |
| scale decorator | Tribhagi, Varsha Narayana | a `CycleScale { factor, rounds }` over another definition: Tribhagi is a Vimshottari definition scaled by 2/3 and run twice. Varsha Narayana's place here is unconfirmed: no book read gives it |
| year | Mudda, Varsha Yogini, Patyayini | a ring of lords with weights that opens part-way through its first lord's share and closes on the rest, every boundary a share turned into an instant through the year's clock (`YearDasha`, [`annual-dashas.md`](annual-dashas.md)). The first draft put the two nakshatra years under the scale decorator; a scaled row cannot show its first lord twice |
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
    pub birth_period: BirthPeriod,   // Compressed (measured default) | Elapsed (crux C48)
    pub after_cycle: AfterCycle,     // End (measured default) | Repeat
    pub year_length: YearLengthId,   // per system, from the profile's table (see the cruxes page)
    pub scale: Option<CycleScale>,   // Tribhagi
    pub applicability: Option<RuleRef>,   // a rules-engine rule (Ashtottari's conditions)
    pub sources: Vec<Citation>,
    pub confidence: Mark,            // V | T | S
}

pub struct SeedToLord {
    pub reference: u16,              // seed index that maps to lord 0
    pub direction: CountDir,         // FromReference | ToReference (Dwadashottari counts to Revati)
    pub span: u8,                    // seed units per lord: 1 for most, 3 for Ashtottari
    pub offset: u8,                  // added after the modulo: 3 for Yogini
    pub repeats: bool,               // the lords run round the nakshatras again (Vimshottari, Yogini) or cover them once (Ashtottari)
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
| Mudda, Varsha Yogini | birth nakshatra | the natal row's lords from its seat, one further each completed year | natal years as weights, through the year's clock | the ring from each lord | V (`YearDasha`, [`annual-dashas.md`](annual-dashas.md)) |
| Varsha Narayana | | scale decorator over the base definition, unconfirmed | | | T |
| Patyayini | annual lagna and the seven | by longitude within the sign, strength breaking a tie | the gaps between them | the ring from each lord | V (`YearDasha`) |

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

**Measured 2026-09-15** (`cargo bench -p teistro-dasha --bench dasha`, an
Apple M2 Pro, release; the medians of criterion's intervals):

| operation | measured | budget |
|---|---|---|
| Vimshottari `at(t, 5)` | 354 ns | 20 µs |
| Ashtottari `at(t, 5)` | 330 ns | 20 µs |
| a registered row's `at(t, 5)` | 350 ns | 20 µs |
| Chara `at(t, 5)` | 229 ns | 20 µs |
| Kalachakra `at(t, 5)` (to its antardashas) | 119 ns | 20 µs |
| Vimshottari `periods(120 years, 3)`, 819 periods materialised | 15.8 µs | 500 µs |
| making a Vimshottari dasha | 75 ns | none set |

Every read is some sixty times inside its budget and allocates nothing
(`tests/allocations.rs`, at depth six for every row of every kernel). A
wall-clock number on a shared runner is noise, so what CI gates is the
instruction count: the scenario's `dashas` section walks the chain at depth
five at 300 instants for every shipped row of every kernel, which the
benchmarks workflow counts under callgrind against the pull request's base
and the hash matrix digests across architectures.

## PyJHora, beside the kernel (2026-09-15)

Phase 5's exit asks for PyJHora cross-checks. The corpus's
`pyjhora/vimshottari` (0.9.0) records PyJHora 4.8.7, run as a black box
through its published functions, for 53 of the 55 charts, at evidence rank 3
(`CLEAN_ROOM.md`: its values may verify the SDK, its code may not be read).
`crates/dasha/tests/pyjhora.rs` gives the kernel the tool's own Moon and year
and compares every period, so an ephemeris difference cannot hide in the
answer.

**The arithmetic agrees.** Over 16 232 antardashas, matched by mahadasha
place and both lords:

| the tool's year | its days | the kernel's | worst start difference |
|---|---|---|---|
| mean tropical | 365.24219 | `TROPICAL`, 365.24219 | 1.4e-9 days, a tenth of a millisecond |
| mean sidereal | 365.256364 | `SIDEREAL`, 365.256363 | 1.2e-4 days, the constant's last digit over 120 years |
| mean lunar | 354.36707 | `LUNAR`, 12 × 29.530589 | 2.4e-4 days, the same |
| savana | about 360.004, computed | `SAVANA_360`, 360 | 0.69 days, the same |

Each bound in the test is the two years' difference times 140 years, so a
start that drifts for any reason but the constant fails.

**What differs, and why it is not a correction** (a rank-3 source never
corrects a rank-2 one):

- **The birth mahadasha's antardashas.** The tool lists them from the
  mahadasha's start, before birth, each its whole share, which is the SDK's
  `dasha.birth_period = ELAPSED`; the corpus's rank-2 engine compresses them,
  the SDK's default. The test reads the tool under `ELAPSED`.
- **The written balance.** 75 of 212 agree. Under the sidereal and tropical
  years the tool's day is one above the kernel's whole days, reading as the
  day begun; under the lunar and savana years it does not write against the
  dasha's year at all. A presentation, counted in the test so a change to
  either shows.
- **The tool's default year** is the true sidereal year, which measures 0.0
  days in 4.8.7 and whose periods cannot be computed, and the Gregorian year
  (365.2425 days) has no `YearLength`; both are recorded where they could be
  and neither is compared.
- **Its Moon is geocentric**, 0.40° from the corpus's topocentric Moon for
  the first chart, which is why the comparison feeds the kernel the tool's.

What this adds to crux C6 is in the register.

## Tests and golden vectors

Unit tests per kernel over its parameter space; the whole-table
invariants above; golden vectors from spike 1 for Vimshottari (every
level under both balance methods, with the year length, the balance and
the nakshatra span recorded: `fixtures/baseline/`, section `dashas`) and
from a second baseline export for the seventeen other systems before
Phase 5 (`05-testing/01-golden-vectors.md`);
property tests: any instant inside a period is found by `at`; `at(t, d)`
agrees with walking `materialise(d, window)`; boundary generators at
plus and minus one microarcsecond of every nakshatra boundary and one
millisecond of every period boundary.

## Localisation

Keys under `sdk.dasha.<id>` for system names and `sdk.entity` for lords;
period levels as an ordinal message with the profile's level names.

## What resisted and what is watched

- **Patyayini** needs its own kernel (periods from strengths); planned.
  **Built 2026-09-24, and two sentences here were wrong**
  ([`annual-dashas.md`](annual-dashas.md)). Its periods come from the
  gaps between the grahas' and the lagna's longitudes *within their
  signs*, not from strengths; strength only breaks a tie. And the Mudda
  and the Varsha Yogini are not the scale decorator: a year opens part-way
  through its first lord's share and closes on the rest, so the first
  lord appears twice. All three run on one year kernel, `YearDasha`, and
  none of them is a scaled row.
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
- **What a consumer still cannot register**, now that both kernels take a
  definition, is a row neither kernel expresses rather than one nobody has
  written down: a start that is the karakamsha (`Sudasa`), a seed that is
  not a nakshatra (the tithi, yoga and karana variants), periods the chart
  supplies (`Tara`, `Karaka`, `Ashtakavarga`), and compositions of systems
  (`Yogardha`, `Sudarshana Chakra`). Seventeen of the twenty-two, each
  named with its reason on
  [`dasha-coverage-measured.md`](dasha-coverage-measured.md).

## Open questions

Tracked on `01-research/feature-universe/19-verification-cruxes.md`:
Shashtihayani, Narayana antardasha tables, Ashtottari applicability,
Shatabdika edition, year length per system, the seed references for the
tithi, yoga and karana variants.
