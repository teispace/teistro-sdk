# The rules engine

Status: `draft`, 2026-09-04. The design of the rules kernel (ADR-0017)
before any rule data is authored: the baseline engine's predicate algebra
plus the four capabilities its own dosha detectors had to bypass it for.
Phase 6, with the `Ref` and `RuleResult` changes landing before the
baseline rule corpus is exported, because every addition made afterwards
means re-authoring rules.

## What the baseline engine proves

Its condition evaluator implements 25 predicate variants over one
recursive algebra (`and`, `or`, `not`; body in house, sign, house from a
reference, kendra, kendra from, trikona; dignity, retrograde, combust;
conjunct, N grahas conjunct with; aspects planet and house; lord of house
in house or kendra, lord conjunct lord, exchange; chara karaka in house;
occupied-sign count, no planet in houses from, all planets between the
nodes, all classical grahas in houses) and evaluates 562 yoga rules and
most of 62 dosha rules through it. That is a working rules kernel at the
scale the SDK needs, and every one of the 25 variants is kept.

Seven detectors in its dosha service bypass the evaluator: Kala Sarpa,
the Kala Sarpa variant classifier, Kala Amrita, Mrityu Bhaga, Dagdha
Rashi, Badhaka and a natal-scope check. None is a new logic shape; they
are three missing capabilities, and left unfixed they would become seven
bespoke modules here too.

| detector | why the algebra could not express it | capability |
|---|---|---|
| Mrityu Bhaga | a per-planet-per-sign degree table | table lookup |
| Dagdha Rashi | a tithi × weekday table | table lookup |
| Badhaka | the badhaka lord is a derived point (from the lagna's modality) | derived-point subjects |
| Kala Sarpa and kin | the rule must say which of twelve named forms fired, not whether | classifying outcomes |

The same capabilities cover the affliction features the catalogue lists
and the baseline engine lacks (22nd drekkana, 64th navamsa, Marana
Karaka Sthana, Pushkara, Latta stars, Pachakadi relations): seven
features, one predicate.

## The four additions

### Subjects are references, not planet ids

```rust
pub enum Ref {
    Planet(GrahaKey), Lagna,
    LordOf(HouseExpr), CoLordOf(HouseExpr),
    Arudha(HouseExpr),                      // A1..A12, upapada
    Karakamsha, Swamsha,
    CharaKaraka(KarakaRank),
    Badhaka,
    SpecialLagna(LagnaId),                  // hora, ghati, sree, indu, bhrigu bindu, pranapada, kunda, varnada (per school)
    Upagraha(UpagrahaId),                   // incl. gulika and maandi
    Sphuta(SphutaId),                       // the fourteen
    Saham(SahamId),                         // the thirty-six
    Yogi, AvaYogi, DuplicateYogi,
    NthFrom(Box<Ref>, i8),
    Stronger(Box<Ref>, Box<Ref>),           // the Jaimini tie-break ladder
}
```

Every predicate that took a planet takes a `Ref`. This multiplies the
expressible rule space by the size of the special-point catalogue at the
cost of one type substitution now.

### Table lookups over cited tables

```rust
TableLookup { table: TableId, keys: Vec<KeyExpr>, test: Comparison, value: ValueExpr }
```

Tables are rule-pack data with their own citations, so a consumer adds a
Mrityu Bhaga variant by shipping a table, and the citation rule (P3 in
the pack schema) holds at the table level.

### Rules classify and grade, not only match

```rust
pub enum Outcome {
    Fires,                                   // the v1 behaviour
    Classify(Vec<(Condition, VariantKey)>),  // first match wins, ordered, exhaustive or with a default
    Grade(GradeExpr),                        // strength or severity as a value
}
pub struct RuleResult { pub fired: bool, pub variant: Option<VariantKey>, pub grade: Option<Score>, pub participants: Participants, pub trace: Option<Trace> }
```

`trace` records, per predicate node, whether it held and with what
values. It is opt-in per call (it allocates) and it is what makes a rule
result checkable by an astrologer and usable as ground truth under any
generative layer.

### Cancellation is first-class, for yogas and doshas

```rust
pub struct AfflictionRule {
    pub base: Rule,
    pub cancellations: Vec<Cancellation>,    // each a rule; partial or full; thresholds
    pub severity: SeverityRule,              // weights, per occurrence, cap, min, max
    pub full_cancellation_threshold: Ratio,
    pub remedy_keys: Vec<RemedyKey>,         // required for doshas
    pub scope: Scope,                        // natal | transit | window
}
pub enum NetStatus { Absent, Present, PartiallyCancelled { fraction: Ratio }, FullyCancelled }
```

The evaluator reports the net status and the cancellations that fired.
Bhanga applies to raja yogas as much as to doshas; the baseline engine
modelled it only on the dosha side; the SDK models it once over both.

## Composition and context

- `in_varga { varga, predicate }` evaluates any predicate in any
  divisional chart; varga positions are memoised per chart context and a
  static cost model orders these last within an `all`, so a cheaper
  predicate short-circuits first.
- `count { of, cmp, n }`, `any_of`, `all_of`, `if_then_else`, and
  `ref { rule }` to compose named sub-conditions (a kendra-trikona lord
  condition written once and referenced by forty yogas; the reference
  graph must be acyclic).
- **The chart-context axis** `(varga_for_planets, varga_for_lagna)` is one
  parameter threaded through the context, so every derived quantity and
  every rule gets its mixed-chart form without a second entry point. A
  third-party implementation carries a `_mixed_chart` twin of every
  sphuta and special lagna because it lacked this parameter.

## Pack format (v2 shape)

```yaml
pack: { id: "baseline-doshas", version: "1.0.0", sdk_catalogue: ">=1.0", licence: "Apache-2.0" }
tables:
  - id: MRITYU_BHAGA
    sources: [{ text: "Jataka Parijata", ref: "..." }]
    keys: [graha, rashi]
    values: degree
rules:
  - key: KALA_SARPA
    kind: dosha
    sources: [{ text: "...", ref: "..." }]
    when: { all_between_nodes: { bodies: CLASSICAL_SEVEN } }
    outcome:
      classify:
        - { when: { in_house: { ref: RAHU, house: 1 } }, variant: ANANTA }
        - { when: { in_house: { ref: RAHU, house: 2 } }, variant: KULIKA }
        # ...
    cancel:
      - { key: KALA_SARPA_CANCEL_KENDRA_BENEFIC, when: { ... }, weight: 0.5 }
    severity: { base: 60, per_occurrence: 10, cap: 100 }
    remedy_keys: [RAHU_KETU_SHANTI]
    tests:
      - { chart: "fixtures/chart-014", expect: { net: PRESENT, variant: ANANTA } }
      - { chart: "fixtures/chart-002", expect: { net: ABSENT } }
```

## Validation gates at pack build

Schema; every `sources.text` resolves in the source registry, no rule or
table without one; every `ref` resolves and the graph is acyclic; every
entity and table id resolves; ids unique; doshas carry `remedy_keys` and
`severity`; `classify` arms exhaustive or with a default; the threshold
in [0, 1]; every rule has a positive and a negative fixture before it is
marked stable; the pack serialises to a canonical byte form with a
pinned hash; rules that always co-fire across the corpus are flagged as
probable duplicates; every rule renders to prose (`teistro rule-doc`) so
an astrologer can review a change without reading the schema.

## Sequencing

1. `Ref` (changes every signature).
2. `RuleResult` and the trace (changes every consumer).
3. `TableLookup` (additive).
4. Cancellation and severity (additive).
5. Only then the export of the baseline engine's 562 yoga and 62 dosha
   rules, re-validated against the stricter schema; rules whose citation
   data does not survive are held back, not weakened (ADR-0018).

## Performance

The indexed chart state in `02-architecture/09-performance-architecture.md`
serves the 25 base predicates from bitsets; `Ref` resolution for arudhas,
sphutas and special lagnas is memoised on the context; the cost model
orders predicates cheapest first. Budget: 900 rules under 2 milliseconds
on the native path, measured in Phase 6.

## Open questions

Authoring ergonomics for non-programmers with `Ref` trees (the prose
renderer is load-bearing); whether 28 predicate variants cover Tajika,
KP, Lal Kitab and Western rules, which were not enumerated here (expect
a v2.1); tracked in `QUESTIONS.md` when Phase 6 opens.
