# Strength schemes (bala)

Status: `draft`, 2026-09-04. The falsification pass for ADR-0017 on the
strength family. Implemented in Phase 5 after the varga kernel and the
aspect model, which it depends on.

## Verdict

A scheme is a table, but not a flat one. The schools do not merely pick
different variants of the same components: they disagree about which
components exist and which group each belongs to. Ayana bala sits inside
Kaala bala for some authorities and outside the six for others; Yuddha
bala is a Kaala component for some and a post-hoc adjustment for others.
Move a component between groups and the arithmetic is unchanged; move it
out of the six and the total that is divided by sixty and compared to the
required rupas changes. This is why the baseline engine reports thirteen
components and PyJHora seventeen. A flat list of components cannot say
it; group membership is data.

Unlike the dasha and varga families, no reference implementation
parameterises this: the baseline engine hard-codes one convention in its
Shadbala service, and PyJHora carries the variants as suffixed duplicate
functions (four Saptavargaja, two Dig, two Drik, two Cheshta). The scheme
table below is therefore the least externally validated design in the
corpus and is built last among the kernels, against the baseline engine's
thirteen-component output as the first fixture set.

## Components

| group | component | variants known | mark |
|---|---|---|---|
| Sthana | Uchcha | 2: the BPHS formula and the Saravali formula (a third-party implementation hides the second behind a flag) | V |
| Sthana | Saptavargaja | 4 in a third-party implementation, unattributed to schools | V (variant 1) |
| Sthana | Ojayugma | 1 | V |
| Sthana | Kendradi | 1 | V |
| Sthana | Drekkana | 1 | V |
| Kaala | Nathonnatha, Paksha, Tribhaga | 1 each | V |
| Kaala | Abda, Masa | 1 each; year and month lord derivation needs a verse | T |
| Kaala | Vara, Hora | 1 each | V |
| Kaala or seventh | Ayana | 1; group membership disputed | V |
| Kaala or adjustment | Yuddha | 1; group membership disputed | V |
| Dig | Dig | 2 (cusp midpoints versus cusp starts) | V |
| Cheshta | Cheshta | 2 or more: from anomaly with an epoch table of mean longitudes, or from the speed fraction | V |
| Naisargika | Naisargika | 1, the fixed 60/7 ladder | V |
| Drik | Drik | 2 aspect-value tables (Parashari and one attributed to Narasimha Rao); the table is selected in the aspect model | V |

Eighteen components; every "1 variant" is a lower bound, because
variants hidden behind flags are invisible to a function-name inventory.
Unimplemented variants fail loudly rather than substituting, so the risk
of an undercount is a missing feature, not a wrong number.

## Schema

```rust
pub struct BalaScheme {
    pub id: SchemeId,                    // parashari (ships first); raman, sripathi, pvr registered, S
    pub groups: Vec<BalaGroup>,          // exactly the six group ids; a group may be empty, never a seventh
    pub aggregation: Aggregation,
    pub required_rupas: [Ratio; 7],      // Sun..Saturn; mandatory: without it there is no strength ratio
    pub sources: Vec<Citation>,
    pub confidence: Mark,
}

pub struct BalaGroup {
    pub id: GroupId,                     // Sthana | Kaala | Dig | Cheshta | Naisargika | Drik
    pub components: Vec<BalaComponentRef>,
    pub combine: Combine,                // Sum, usually; some schools cap Kaala
}

pub struct BalaComponentRef { pub component: ComponentId, pub variant: VariantId, pub weight: Ratio }

pub struct Aggregation {
    pub combine: Combine,                // Sum | WeightedSum | Max
    pub virupa_per_rupa: Ratio,          // 60, and a convention, not a constant
    pub report: ReportForm,              // Virupa | Rupa | RatioToRequired | All
}
```

Required rupas: the baseline engine ships Sun 5.0, Moon 6.0, Mars 5.0,
Mercury 7.0, Jupiter 6.5, Venus 5.5, Saturn 5.0 attributed to B.V. Raman;
the strengths research page lists the Sun at 390 shashtiamsas (6.5
rupas). The discrepancy is an open crux and the row ships with the
baseline engine's values as the `parashari-baseline` default until a text
settles it.

## Mark and continue

| item | status | how it ships |
|---|---|---|
| Saptavargaja variants 2 to 4 | S | registered variant ids with no implementation; `UNSUPPORTED (unsourced)` on request |
| Dig variant 2 | S | same |
| Cheshta: epoch table versus computed mean longitude | T | both implementable; the epoch-table form is the default and the result says which ran |
| Drik: aspect tables | T | both tables ship; selected in the aspect model |
| Abda and Masa lords | T | standard; needs the verse |
| Ayana and Yuddha membership | S | default: both inside Kaala, matching both references; the other grouping is a scheme row nobody has to use |
| Raman, Sripathi and Narasimha Rao full schemes | S | named scheme ids, no rows; `UNSUPPORTED (unsourced)`, never a fallback to Parashari |

## Invariants

1. Every component reference resolves to a registered component and an
   implemented variant; unimplemented variants fail at load, not at call.
2. No component appears in two groups within one scheme.
3. `groups` covers exactly the six group ids.
4. `required_rupas` has seven entries, Sun to Saturn.
5. The six-fold total equals the sum of its groups in exact arithmetic on
   every fixture.
6. `virupa_per_rupa > 0`.
7. The `parashari-baseline` scheme reproduces the baseline engine's
   output within the published tolerance on the reference corpus.

## The Ashtakavarga, built (2026-09-15)

The first strength measure built, before the bala schemes, because it is a
function of eight signs and nothing else, and the dasha and transit work
reads it. `cargo xtask ashtakavarga` (`ashtakavarga-measured.md`) measured
the conformance corpus's engine over 77 charts beside BPHS chs. 66 to 69 in
translation, a rank-1 text:

- **The bindu tables are the text's**, every chart holds 337, and the
  engine's reproduce on every chart.
- **Everything after them is read two ways.** The text reduces each graha's
  own Ashtakavarga and forms each graha's pindas from its own reduced bindus
  and the grahas standing in each sign; the engine reduces the sum, forms a
  rashi pinda of the reduced sum and a graha pinda of raw bindus, zeroes an
  empty co-ruled sign where the text keeps a difference, and gives Virgo the
  measure 8 where the text gives 6.
- **A rank-1 text corrects a rank-2 value**, so `strength.shodhana =
  EACH_GRAHA` and `strength.ekadhipatya = BPHS` are the defaults, and the
  conformance profile takes `SARVA` and `EMPTY_TO_ZERO`, under which
  `crates/strength` reproduces all 77 recorded answers (cruxes C59–C62).
- **The lagna keeps no Ashtakavarga of its own and the nodes are not
  occupants** (C62): neither the engine nor the text read settles either.

The document's `ashtakavarga` section, the boundary's three sections and
every binding carry it, and `check-parity` holds the four surfaces to it.

## The Vimshopaka, built (2026-09-15)

The second strength measure built, because like the Ashtakavarga it is a
function of signs alone: the seven grahas' in the sixteen vargas the four
schemes read. `cargo xtask vimshopaka` (`vimshopaka-measured.md`) measured
the conformance corpus's engine over 93 charts beside BPHS ch. 7 in
translation, a rank-1 text:

- **The four schemes' weights are the text's** (vv. 17 to 25), each summing
  to 20, and the engine's agree. `teistro_strength::vimshopaka::WEIGHTS`
  holds them in halves of a point, so the table is exact integers.
- **How one varga is scored is read two ways.** The text gives 20 in
  exaltation or the own sign and 18, 15, 10, 7 or 5 by the compound
  relationship with the sign's lord, times the varga's weight over 20. The
  engine reuses the Saptavargaja virupas (45, 30, 15, 7.5, 3.75, and nothing
  in debilitation) over 45, by natural friendship alone, so a friend's varga
  earns 6.67 of 20 where the text gives 15 and no varga is ever a great
  friend's; it rounds half up to hundredths through its own float
  arithmetic, which the module mirrors (two of 93 charts differ otherwise).
- **A rank-1 text corrects a rank-2 value**, so `strength.vimshopaka =
  BPHS` is the default and the conformance profile (version 5) takes
  `SAPTAVARGAJA_VIRUPAS`, under which `crates/strength` reproduces all 93
  recorded answers (crux C63). The text's reading differs from every one of
  the engine's 2604 scores, by up to 12.88 of 20.
- **Two things the text does not settle** (C63): which chart the compound
  relationship's temporary half is taken in (the module takes the rasi
  chart, where the grahas stand) and whether debilitation takes anything
  from a varga (the text's ladder has no step for it, so the module's does
  not either).

The document's `vimshopaka` section, the boundary's `vimshopaka` section
with `TsVimshopakaScoring` and section bit 64, and every binding's
`chart.vimshopaka` carry it; `check-parity` holds the four surfaces to it.

## The Shadbala, built (2026-09-15)

The six strengths of BPHS ch. 27, built as arithmetic on a chart and not as
the scheme table this page first proposed. `cargo xtask shadbala`
(`shadbala-measured.md`) measured the conformance corpus's engine over 71
charts and 497 graha cells beside the chapter in translation, and the
measurement changed the design:

- **Every one of the engine's seventeen components reproduces exactly**
  from the recorded inputs, so its reading is fully described.
- **It departs from the chapter at eleven forks**, each measured: the
  Saptavargaja's figures and relationship, Nathonnatha's shape and the night
  before dawn, the Sun's Ayana, the Moon's Cheshta, the kranti, the Abda and
  Masa lords, Dig's kendras, Drik's weighing, the natural strengths' rounding
  and the requirements (C64–C71). The schools' disagreement is **per fork and
  not per scheme**, so each fork is a `strength.*` setting, BPHS's reading the
  default and the engine's the other value, and `conformance-baseline`
  (version 6) takes the engine's at all eleven. `ShadbalaRules::BPHS` and
  `ShadbalaRules::RECORDING_ENGINE` name the two readings whole.
- **The ahargana settles the Abda and Masa lords.** The chapter counts them
  from Burgess's day count to 1 January 1860, whose weekday arithmetic the
  corpus confirms on 67 of 71 recorded weekdays — the other four are charts
  the engine put in the wrong Hindu day — and the chapter's own worked month
  comes out a Friday as the text says.
- **What neither settles ships the same under both and says so**: Drik reads
  the graded whole-sign drishti, because ch. 26's sphuta drishti is garbled
  in the translation read (C45, C69), and the Cheshta of Mars to Saturn reads
  the engine's J2000 mean elements, which the chapter does not give (C70).
- **`strength.bala_scheme` now has a reader**: `PARASHARA` is the six, and
  `PARASHARA_EXTENDED`, which the catalogue names without defining, is
  refused as unsupported rather than guessed at.
- **Sripati's reading, from B.V. Raman's worked example, came next.** *Graha
  and Bhava Balas* works every component on one Standard Horoscope, so it is
  the one reading checkable number by number. It gave the sphuta drishti
  (closing C45, now `teistro_aspect::sphuta`), a Drik of a quarter of the
  pinda, benefics by the Moon's phase and Mercury's company, the moolatrikona's
  45 in the rasi alone, a male, neuter, female Drekkana, the Hindu kranti
  table, Kedarnath Dutt's mean elements for the Cheshta (the recording
  engine's inferior planets turn out to take their own seeghrochcha as their
  mean too) and a Yuddha bala. Four more forks became settings
  (`strength.drekkana`, `benefics`, `cheshta`, `yuddha`), three gained a value
  (`kranti = HINDU_TABLE`, `drik = QUARTER`, `luminary_cheshta = NONE`), and
  `ShadbalaRules::SRIPATI` reproduces Raman's worked Shadbala within his
  rounding, every total within half a virupa once his three arithmetic slips
  are corrected (`crates/strength/tests/sripati.rs`). The chapter's reading
  takes Sripati's Cheshta elements and Yuddha, which it lacks, and its sphuta
  drishti. The module is now a directory, a file a strength.

`crates/strength`'s `shadbala` reproduces all 71 recorded Shadbalas under the
engine's reading; the façade reads the angles and the true obliquity from the
founder (`Founder::angles_at`) and searches the Mesha sankranti only when
the engine's year lord is asked for. The document's `shadbala` section, the
boundary's section 29 (bit 128, its value columns one table shared by the
schema and the writer) and every binding's `chart.shadbala` carry it, and
`check-parity` holds the four surfaces to it. The components are
`SthanaBala` and `KaalaBala` in every binding, because the catalogue already
has a `Kaala`.

## The Bhava bala, built (2026-09-15)

Each bhava's strength from its lord's Shadbala, its direction and the drishtis
it receives, measured the other way round from the Shadbala: `cargo xtask
bhava-bala` (`bhava-bala-measured.md`) runs the **built** module over the
corpus's 0.8.0 `baseline/bhava-bala` (852 houses), and it reproduces every
recorded aspect and component within the engine's hundredths under the
engine's reading. The three readings part at three forks, each a setting:
the Dig's sign classes and whether it counts house steps or an arc (C73), the
drishti as a quarter of the Dig or as the sphuta drishti on the madhya (C74),
and vv. 30 and 31's special rules, whose twilight no source defines (C75).
The verses as translated are the default; `BhavaBalaRules::SRIPATI`
reproduces B.V. Raman's Example 59 house by house, once his twelfth total's
slip of a hundred is corrected. The façade computes the Shadbala it reads when
the request does not ask for it too; the document's `bhava_bala` section, the
boundary's section 30 (bit 256) and `chart.bhavaBala` in every binding carry
it.

## Rashi bala

Rashi bala is unbuilt: no source read gives it component by component, and
the corpus records none, so it waits for one. Vaiseshikamsa is a count over
the same vargas the Vimshopaka reads and a row in the same crate.

## Build order

Varga kernel, then the aspect model (Drik bala reads it), then the
scheme kernel. Building bala first would stub two dependencies.

## Open questions

Whether the four Saptavargaja variants are one function with parameters
(then a weight table) or four algorithms (then a function selector, the
same tension as the chart-query field in the dasha kernel); answerable in
an hour once the code is being written. Tracked on the cruxes page with
the required-rupas discrepancy.
