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

## Bhava bala, rashi bala, vimshopaka

Bhava bala and rashi bala reuse the scheme kernel with their own
component sets (a smaller repeat of this page, written when they are
built). Vimshopaka and vaiseshikamsa are weight tables per varga group
(the baseline engine's tables, with the invariant that group weights sum
to 20); they are rows in the same crate.

## Build order

Varga kernel, then the aspect model (Drik bala reads it), then the
scheme kernel. Building bala first would stub two dependencies.

## Open questions

Whether the four Saptavargaja variants are one function with parameters
(then a weight table) or four algorithms (then a function selector, the
same tension as the chart-query field in the dasha kernel); answerable in
an hour once the code is being written. Tracked on the cruxes page with
the required-rupas discrepancy.
