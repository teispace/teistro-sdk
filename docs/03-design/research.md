# `research`: statistics over chart batches

Status: `built`, 2026-10-10 (drafted 2026-10-08; every step of §4 built,
step 4 for dasha delivery alone: transits wait for a consumer).
Track B row 7 of the completion plan
(`07-roadmap/00-roadmap.md`), the module catalogue's `research` row
("batch computation, statistics, rule search over sets") and
`01-research/feature-universe/14-remedies-numerology-misc.md`, "Research and
statistics". The kernel, `crates/research`, the façade and the boundary are
built, and every binding carries the four studies (steps 2 to 6 of the order
of work, 2026-10-09). Every figure this page would otherwise
state belongs on a generated `research-measured.md` (order of work, step 1).

## 1. What credible astrological statistics looks like

The field has a long record of results that went away under a correct
null, so the module's job is to make the correct null the easy one.

### 1.1 The unit and the question

The unit is a chart (one birth). A study asks one of three questions:

| design | question | the null that answers it |
|---|---|---|
| **two groups** | is a predicate commoner among cases (champions, a diagnosis, the married-twice) than among controls? | the case/control labels are exchangeable: **permute the labels** |
| **one group against its own population** | is a predicate commoner in this sample than chance, with no control group? | chance is *not* uniform: build the expectation from the sample's own date, clock-time and place distributions, **shuffling clock times among subjects** (the Gauquelin method, §1.3) |
| **events in lives** | do events (a marriage, an accident) fall where a timing rule says (in a dasha of the rule's formers, under a transit) more often than chance? | the pairing of event and person is arbitrary under the null: **shuffle events among people** (the shuffled-event null, §1.4) |

A permutation test needs no distributional assumption. It needs only that
the shuffled quantity is exchangeable under the null, which is a claim
about the design and must be stated per study. Shuffling inside strata
(birth decade, region, sex, age band) is how a study keeps exchangeability
true when a margin drifts.

### 1.2 The permutation p-value

With `m` random permutations and `b` of them at least as extreme as the
observed statistic, the p-value is `(b + 1) / (m + 1)`, never `b / m`
(Phipson and Smyth, *Stat. Appl. Genet. Mol. Biol.* 9, 2010: the second
understates by about `1/m` and can be zero). The observed labelling counts
as one member of the reference set. A p-value from `m` draws also carries
Monte Carlo error, so the answer reports a Clopper–Pearson interval on it,
and the smallest p-value `m` can say is `1/(m + 1)`. That bounds what any
multiple-comparison correction can reach (§2.6).

For two groups and a yes/no predicate, the exact null distribution of the
case count is hypergeometric (Fisher's exact test). The module computes it
alongside, as a cross-check (§3, test 3).

### 1.3 Why the expected frequency needs the birth distributions

Gauquelin divided the diurnal circle into sectors and reported planets in
"key" sectors (rising and culminating) more often than expected for
eminent professionals. Mars and sports champions is the best known case.
The expected frequency is the crux. Births are not uniform in clock time:
before induced births became routine they peaked in the early morning, and
the schedules of modern obstetrics move them again. They are not uniform
over the year either, and the length of the day sectors changes with the
season and the latitude. Because Mars is often near the Sun, a planet's
sector at birth is tied to the hour of birth and the date. A uniform 1/12
expectation therefore finds "effects" in any sample of ordinary births.
Gauquelin's answer was to build the expectation from the sample itself:
combine one subject's date with another's hour, which keeps each margin
and breaks only their pairing. Abell, Kurtz and Zelen (1983) accepted that
this allowed for the demographic and astronomical factors. The dispute
moved to sample selection: the Comité Para (1967), the Zelen test
(1976–77), the US test (1979) and the CFEPP's French replication (1994),
with Ertel's eminence argument against the last two and Dean's account of
misreported birth times.

So the module offers the recombined control (§2.4) as the one-group
expectation and refuses a uniform expectation for a predicate that
depends on the clock time (§2.6).

### 1.4 The shuffled-event null

An event study pairs each person with dated events. A timing rule
(`teistro_rules::Timing`, delivered through `Evaluator::delivery` from the
periods running at an instant) or a transit condition is evaluated at
each event. The null keeps every chart and every event and breaks only
who had which event. Two shuffles preserve different things, and the
study must choose:

- **event dates** permuted among people. This keeps the calendar
  distribution of events (seasons, the slow planets every chart shares at
  a date). It changes each person's age at the event, and a dasha is
  driven by age. An event may also land before a birth, and the shuffle
  must refuse or restrict that pairing (§2.6).
- **ages at event** permuted among people (person *i*'s event moved to
  `birth_i + age_j`). This keeps the age distribution, which suits a
  dasha. It moves events in the calendar, which matters for transits.

Neither is right in general. The answer names the shuffle it used, and the
measured page shows both on a planted effect and on a null (§3, test 5).

### 1.5 Many predicates

A study that tries many predicates will find some "significant" by
chance: the look-elsewhere effect. The module treats every predicate in
one request as one family and reports, for each predicate:

- the raw permutation p-value;
- **Westfall–Young max-T** (step-down min-p) adjusted p-values, from the
  same permutations. This controls the familywise error rate and uses the
  real dependence between predicates. Yogas sharing conditions are highly
  dependent, and Bonferroni would be needlessly strict for them;
- **Holm** (step-down Bonferroni), which holds under any dependence and is
  easy for a reader to check by hand;
- **Benjamini–Hochberg** q-values for the false discovery rate. These
  assume positive dependence, so **Benjamini–Yekutieli** is offered for
  arbitrary dependence.

Bonferroni itself is reported only as the bound Holm improves on.

### 1.6 Effect sizes, not verdicts

A p-value says nothing about size. Each predicate's answer carries:

- the counts and proportions per group;
- the risk difference and the risk ratio, each with an interval
  (Newcombe's hybrid score interval for the difference, Katz's log
  interval for the ratio), and the odds ratio;
- Cohen's *h* for comparing studies;
- for the one-group design, observed over expected. This is the ratio
  Gauquelin's literature quotes.

Following the project's own rule ("report clauses, not a verdict"), the
answer never says "significant". It reports the numbers and the
correction, and a caller who passes an `alpha` gets which predicates fall
under it by each method. Nothing in the answer says the predicate is
astrologically true.

### 1.7 Pitfalls the module can see, and the ones it cannot

| pitfall | what the module does |
|---|---|
| look-elsewhere: many predicates, one reported | the family is the request; adjusted p-values are always present, and the provenance hash names the whole family, so a reader can ask for it |
| optional stopping on permutations: draw more until p < 0.05 | `permutations` is fixed in the request; there is no adaptive mode in v1. A sequential scheme (Besag–Clifford) is valid and can come later as its own named knob |
| optional stopping on subjects: add charts until significant | invisible to a single call. The request hash in the envelope is a **pre-registration**: publish the hash of the request (predicates, seed, permutations, shuffle, strata) before collecting the data |
| seed shopping: rerun with seeds until one is significant | the seed is in the request hash; the measured page shows that the p-value's Monte Carlo interval is the honest spread |
| uniform expectation for a clock-dependent predicate | refused (§2.6); the recombined control is the answer |
| birth times rounded to the hour or misreported | a per-birth uncertainty (`uncertaintyMinutes`, `Birth::uncertain_by`) re-evaluates each chart at the edges of its recorded uncertainty. A predicate that flips inside the uncertainty is counted as **unstable** and reported per predicate. It is not silently decided |
| a predicate that cannot be evaluated on some charts (a special lagna on a polar day) | counted as unreadable per predicate and excluded from that predicate's denominator, never counted as absent (`a-fall-through-is-an-answer`) |
| mixed settings across the batch | refused: every chart must carry one settings hash |

## 2. What the module answers

### 2.1 Where the code lives

- **`crates/research`**, a new crate with no sky: the counter-based
  generator, the shuffles, the permutation engine over bitsets, p-values,
  the corrections, the effect sizes and their intervals. It is pure data in,
  data out, so it joins the `COMPUTATION` list in `xtask/src/lints.rs` and
  is held by `deterministic-iteration` and `ambient-input`.
- **The façade area `sdk.research()`**, `crates/sdk/src/area/research.rs`,
  behind a `research` family feature that requires `chart` (as `kp` does
  in `03-design/wasm-profiles.md`). It founds and reads the batch, turns a
  `RuleSet` into a predicate matrix and hands the matrix to the crate.
- The boundary: one `research_json` request record and a JSON answer,
  as `rules_json` crosses (`03-design/rules-at-the-boundary.md`), so every
  binding reads one type. It is added to `03-design/surface-areas.md` so
  `check-areas` holds the bindings to it.

### 2.2 Determinism

No ambient randomness, no `HashMap` iteration, and **the same request gives
the same bits on every platform, in every binding and at any thread
count**:

- **Generator**: SplitMix64, keyed per permutation. Permutation `k`'s
  stream is seeded from `mix(seed, k)`, so a permutation does not depend
  on how many came before it or on which thread drew it. The algorithm is
  written in the crate, not taken from `rand`, because it is part of the
  answer's contract. A dependency's minor release must not move a
  published p-value.
- **Shuffle**: Fisher–Yates with Lemire's unbiased bounded integer
  (multiply-shift with rejection). Within strata, each stratum is shuffled
  in its key order (`BTreeMap`).
- **Versioned**: the answer's provenance names `research/shuffle/1`. A
  change to the generator or the shuffle is a new version, and the old one
  stays selectable so a published study can be rerun.
- **Integer statistics**: counts are summed as integers, so a sum across
  threads is order-free. A floating statistic (a weighted score) is
  computed per permutation in a fixed order, and only integers cross
  threads.
- **Lints**: `ambient-input` gains `thread_rng`, `OsRng`, `getrandom`,
  `rand::random` and `RandomState`, and `crates/research` joins its scope.
  The lint is proved red first by planting one of each.

### 2.3 Batch and parallel by signature

The expensive work happens once: founding the charts and evaluating each
predicate on each chart. The permutations then run over an
`N × M` bitset (charts × predicates). A permutation builds the case mask
and takes one `popcount(case & predicate)` per predicate. Max-T needs
every predicate's statistic for every permutation, and this layout gives
them together.

`Parallelism::{One, Threads(n)}` splits the permutation range `0..m` into
contiguous chunks. By §2.2 the answer is identical for every `n`. wasm is
`One`. The core takes no locks. The founding phase uses the context's
existing batched calls (`found_many` and `readings_with_rules` per place),
so a batch of births at many places is grouped by place and offset before
it is founded.

Event studies precompute an `N × N` matrix (chart *i* at event *j*'s
instant, or at `birth_i + age_j`), and a permutation sums along it in
O(N). Under the event-date shuffle, a transit predicate asks the sky only
at the N distinct event instants, in one batched request. Under the
age shuffle there are N² instants. The request was to carry `max_pairs` and
be refused above it, naming the field, so that cost is chosen rather than
met. Designed, not built: transits are not built, and dasha delivery asks
no sky per pair, so no request carries the cap yet.

### 2.4 API sketch

In the façade's style (`ChartArea`, `NumerologyArea`): a borrowing view, a
request type that validates into a checked value, an `Envelope` sealed
over what is published, and errors that name their field.

```rust
/// `sdk.research`: counts and tests over a batch of charts.
#[derive(Clone, Copy, Debug)]
pub struct ResearchArea<'a> { context: &'a Context }

impl<'a> ResearchArea<'a> {
    /// How often each predicate holds over a batch: per group, with the
    /// unreadable and unstable charts counted apart. No null, no shuffle.
    pub fn counts(
        self,
        batch: &Batch,
        predicates: &RuleSet,
    ) -> Result<Envelope<Counts>, Error>;

    /// Two or more groups, the labels permuted (within strata when given).
    pub fn compare(
        self,
        batch: &Batch,
        predicates: &RuleSet,
        test: &GroupTest,
    ) -> Result<Envelope<Tested>, Error>;

    /// One group against its own recombined population (§1.3): each
    /// replicate refounds the batch with clock times shuffled among
    /// subjects, inside strata.
    pub fn expected(
        self,
        births: &[Birth],
        predicates: &RuleSet,
        control: &Recombine,
    ) -> Result<Envelope<Tested>, Error>;

    /// Events in lives against the shuffled-event null (§1.4).
    pub fn timed(
        self,
        batch: &EventBatch,
        study: EventStudy<'_>,                // built name; `EventPredicates` was the sketch's
        strata: Option<&[u32]>,
        test: &EventTest,
    ) -> Result<Envelope<Tested>, Error>;
}

/// The charts of a study: births or documents already read, each with
/// its group and its strata. One settings hash for all.
pub struct Batch { /* … */ }
impl Batch {
    pub fn of_births(births: &[Birth], request: &ChartRequest) -> BatchBuilder;
    pub fn of_documents(documents: &[Document]) -> BatchBuilder;
}
impl BatchBuilder {
    pub fn groups(self, groups: &[GroupId]) -> Self;           // one per chart
    pub fn strata(self, strata: &[StratumKey]) -> Self;        // optional
    pub fn build(self) -> Result<Batch, Error>;
}

/// One birth: where and when, under its local clock.
pub struct Birth { pub instant: JulianDay<Utc>, pub place: Place, pub offset: UtcOffset }
// built: `Birth::uncertain_by(minutes)` sets each birth's own uncertainty

pub struct GroupTest {
    pub seed: u64,                         // required: no default seed
    pub permutations: u32,
    pub contrast: Contrast,                // CaseVsRest { case } | AnyDifference
    pub alternative: Alternative,          // Greater | Less | TwoSided
    pub level: f64,                        // every interval's confidence, 0.95
    pub alpha: Option<f64>,                // only to list which fall under it
    pub parallelism: Parallelism,
    pub shuffle: ShuffleVersion,           // research/shuffle/1
}

pub struct Recombine {
    pub seed: u64,
    pub replicates: u32,                   // each a refounded sample of size N
    pub shuffle: Recombined,               // ClockTime (date and place kept together)
    pub strata: Strata,                    // by decade, region, … from the batch
    pub parallelism: Parallelism,
}

pub struct EventTest {
    pub seed: u64,
    pub permutations: u32,
    pub shuffle: EventShuffle,             // EventDates | AgesAtEvent
    pub after_birth: AfterBirth,           // Refuse | RestrictPairings
    // `max_pairs: u64`: designed, not built (§2.3)
    pub parallelism: Parallelism,
}

/// What a test answers: per predicate, never a single verdict.
pub struct Tested {
    pub rows: Vec<PredicateRow>,           // in the RuleSet's order
    pub permutations: u32,
    pub resolution: f64,                   // 1 / (m + 1)
    pub shuffle: ShuffleVersion,
}
pub struct PredicateRow {
    pub rule: KeyId,
    pub counts: GroupCounts,               // present, absent, unreadable, unstable per group
    pub observed: f64,
    pub p: PValue,                         // (b+1)/(m+1) with its Clopper–Pearson interval
    pub exact: Option<f64>,                // the hypergeometric p, two groups and yes/no only
    pub adjusted: Adjusted,                // max_t, holm, bh, by
    pub effect: Effect,                    // risk difference, risk ratio, odds ratio, h, observed/expected
    pub under_alpha: Option<UnderAlpha>,   // per method, only when alpha was given
}
```

A predicate is a `Rule` (`teistro_rules::Condition` and its
cancellations), so a study uses the shipped sets (`ShippedRules::Yogas`, …)
or writes its own in the same language, and `RuleRequest::rule_set`
validates it as it does for a reading. An event study (`EventStudy`,
`crates/sdk/src/area/research.rs`) reads its rule set through its delivery
in the running periods of the dasha the study names; a gochar condition at
the event's instant is designed, not built.

### 2.5 The knobs

| knob | default | why it is a knob |
|---|---|---|
| `seed` | none: required | a default seed hides that one was chosen |
| `permutations` | none: required | it sets the resolution, and the resolution bounds the corrections |
| `contrast` | none: required | one group against the rest, or any difference among several |
| `alternative` | `TwoSided` | a directional hypothesis must be declared, not chosen after |
| `level` | 0.95 | the confidence of every interval; the p-value's and the effects' alike |
| `strata` | none | exchangeability by decade, region or sex is the study's claim |
| `EventShuffle` | none: required | §1.4: the two keep different margins |
| `after_birth` | `Refuse` | `RestrictPairings` permutes only among people born before the event, which changes the reference set; the answer says so |
| `uncertaintyMinutes` (per birth) | 0 | rounded hours are the field's commonest data error |
| `Readings` | the rule set's own | the predicate's meaning is part of the request |
| `parallelism` | `One` | the answer is the same either way |
| `max_pairs` | a fixed cap | event studies' cost is chosen, not met; designed, not built (§2.3) |

**Decided when the kernel was built (2026-10-09).** Three knobs of the
sketch were dropped, and the reasons are kept here:

- **No `statistic` knob.** A case-against-rest test ranks by the case count
  standardised under the hypergeometric that holds the margins,
  `(a − n₁P/R)/sd`. Given the cases read, a count and a risk difference are
  monotone in each other, so they give the same test, and the standardised
  form puts predicates of different prevalence on one scale. Max-T needs
  that scale: on raw counts a common predicate would dominate the family's
  maximum. `AnyDifference` is Pearson's chi-square, which is already
  standardised and has no direction, so a direction asked of it is refused.
  The counts and the risk difference are in every row's effect.
- **No `corrections` knob.** Every row carries all five: max-T, Holm,
  Bonferroni, BH and BY. They cost nothing beside the permutations, and a
  reader of a published table should not have to ask for one.
- **No `Pairwise` contrast yet.** Each pair of groups is its own reference
  set, so it is a family of tests over sub-batches rather than one
  permutation of the whole. It comes later as its own contrast.

The exact p (§1.2) is the hypergeometric of the *same* statistic, so it is
offered only where the case count is the whole story: two groups, no
strata, and a predicate read on every chart.

### 2.6 What it refuses

Each refusal is an `INVALID_ARG` (or `UNSUPPORTED`) naming its field, with
a hint where one helps:

- a batch with fewer than two charts, a group with none, or one label for
  every chart (`groups`);
- groups or strata not one per chart (`groups`, `strata`);
- charts with different settings hashes (`batch`, naming the first that
  differs);
- a predicate that is not evaluable (`Rule::is_evaluable` false), naming
  the rule. It is never counted as absent;
- a rule key given twice, or an unresolved reference (as `rule_set` already
  refuses);
- `permutations` of zero or above the cap; `replicates` the same;
- a resolution the corrections cannot use: when `1/(m + 1)` exceeds
  `alpha / M` (Bonferroni's threshold, the strictest the answer reports),
  refused on `permutations` with the hint "at least ⌈M/alpha⌉ − 1";
- an `alpha` outside (0, 1);
- every stratum a singleton, so nothing can move (`strata`);
- under `EventDates` with `Refuse`, a shuffled pairing that would put an
  event before a birth, and no permutation possible (`after_birth`);
- `max_pairs` exceeded (`max_pairs`, with the pair count it would need):
  designed, not built, with the cap;
- a uniform expectation, which is not offered at all, so `expected`
  takes only the recombined control (C365). The sketch allowed one for a
  predicate on signs alone, but births are seasonal and the Sun's stay in
  a sign is unequal, so no predicate on a chart has one;
- a timed study that names no `shuffle` (`research.shuffle`): neither is a
  default (C364);
- a shuffle version the build does not carry (`shuffle`).

## 3. Tests

The tests are written against the promise ("a null is found at the
expected rate, a planted effect is found"), not against the code. Each
uses fixed seeds, so the counts are exact and also golden.

1. **The null is calibrated.** Labels independent of the charts over a
   batch of real founded births (a sky that can happen, from a realistic
   spread of dates, hours and places). Run `R` independent datasets at
   `alpha`. Assert that the rejection count lies inside the binomial
   interval for `R × alpha` and that the p-values pass a uniformity check.
   For max-T and Holm, assert the familywise rate over a family of null
   predicates. For BH, the false discovery rate.
2. **A planted effect is found.** Labels drawn with a known risk ratio
   when a predicate holds. Assert it is found at the planned size, that the
   effect interval covers the planted ratio, and that the other predicates
   in the family are not raised above the null rate.
3. **Permutation meets exact.** Two groups and a yes/no predicate: the
   permutation p lies inside its own Monte Carlo interval around the
   hypergeometric p.
4. **The Gauquelin artefact.** A null sample with an early-morning birth
   peak and a seasonal one, and a predicate on a house or sector. The
   uniform expectation would reject; the request is refused. The
   recombined control does not reject, at the calibrated rate. This test
   is the reason `expected` exists.
5. **Shuffled events.** A planted delivery effect (events placed in the
   rule's formers' periods) is found under both shuffles. A null is not.
   An event before a birth is refused under `Refuse` and restricted under
   `RestrictPairings`.
6. **Determinism.** One request gives bit-identical answers at 1, 2 and 8
   threads, in every binding through the parity runners, and across the
   hash matrix's architectures. A different seed gives a different
   answer.
7. **The shuffle is fair.** Fisher–Yates on four items over many draws
   hits each of the 24 orders within its binomial interval.
   Lemire's bound on non-powers of two is checked the same way.
8. **Every refusal**, one test each, naming its field. A refusal after a
   refusal on the same context still names its own
   (`error-carries-its-record`).
9. **Unreadable and unstable are counted.** A predicate on a special lagna
   over a polar day is excluded from its denominator, and the count says
   so. A birth inside its time uncertainty of a sign boundary is
   unstable.

## 4. Order of work

1. **The measured page first.** `cargo xtask research` writes
   `docs/03-design/research-measured.md`, held by `check-research`: the
   calibration (test 1), the power (test 2), the artefact demonstration
   (test 4) and the two event shuffles (test 5). It runs over a seeded
   synthetic batch of founded births, priced on CI before it is merged
   (`price-a-pass-on-ci`). The figures this page leaves out live there.
   **Built** 2026-10-10, last rather than first, because it measures the
   module through its public calls. Every correction holds the familywise
   rate under the complete null, max-T nearest the level; the
   permutation p's interval covers the exact p; the recombined control
   finds nothing in an early-morning sample a uniform expectation reads
   as beyond chance; and neither shuffle rejects a null event study more
   often than its level. **Found** writing it: the rising Sun alone, the
   sector the claim first named, did not reach the threshold at the
   sample's size, and the culminating Sun, which the early hours empty,
   did; the claim is now the family's, which is the look-elsewhere effect
   §1.5 is about, met by the page that measures it.
2. **`crates/research`**: generator, shuffle, bitset engine, p-values and
   intervals, max-T, Holm, BH, BY, effect sizes. Tests 3, 6 (thread
   counts), 7. The lint changes (§2.2), proved red. **Built** 2026-10-09.
3. **`sdk.research().counts` and `compare`** over documents and births,
   with refusals and tests 1, 2, 8, 9. Add to `surface-areas.md` with the
   boundary (step 6), whose module the page's table names. **Built**
   2026-10-09 over births (`crates/sdk/tests/research.rs`); a batch a
   polar birth spoils is read again a chart at a time, and a batch that is
   refused names the birth that refused it.
4. **`timed`**: dasha delivery first (no sky per pair), then transits
   under the event-date shuffle, then the age shuffle under `max_pairs`
   (the cap not built).
   Test 5. **Built** 2026-10-09 for dasha delivery under both shuffles;
   transits wait for a consumer.
5. **`expected`** with `Recombine`. Test 4. **Built** 2026-10-09.
6. **The boundary** (`research_json`), the bindings, parity, and a page in
   the docs site's guides with an executed example of a two-group study.
   **Built** 2026-10-09 as one entry point, `ts_research`, whose record
   names the study (`COUNTS`, `COMPARE`, `EXPECTED` or `TIMED`) and is
   read by `ResearchRequest::from_json`, refusing a field the study does
   not read by name. The answer crosses as the envelope, so the input
   hash reaches every binding, and a seed may be a decimal string because
   a JavaScript number does not hold every 64-bit seed. Node, Python,
   Dart and Java carry `research.counts`, `compare`, `expected` and
   `timed`, every runner agrees value for value and on the input hash, and
   `research` is a shared example, a two-group study over
   labels that mean nothing; the site's guide is `research.mdx`.
   **Found** building it: a recombined sample beyond replicates that all
   agree published its ranking sentinel, `f64::MAX`, as the statistic,
   so `observed` is now absent there; and the replicates' mean of equal
   shares did not return the share to the bit, which read as a spread of
   1e-17 and a statistic of 1e15, so equal shares are now taken as the
   share with no spread.
7. Cruxes for the conventions a reader could argue with: the p-value
   formula, max-T as the default family correction, the two event
   shuffles, and the refusal of a uniform expectation. **Built**
   2026-10-09 as C362 to C365. **Found** writing them against the code:
   a timed study's shuffle defaulted to `EVENT_DATES`, against §2.5 and
   the worst choice for a dasha, which reads the age a date shuffle
   moves, so it is now required in the façade, the record and every
   binding; and the hint on a refusal of too few permutations was
   computed apart from the check it answered, so for some families it
   named a count the check refused (alpha 0.3 over three rules) or one
   more than the fewest it takes (0.01 over 73). The check and its hint
   now read Bonferroni's own arithmetic, and a test walks the boundary
   over a grid of alphas and family sizes.

## Sources

- B. Phipson and G. K. Smyth, "Permutation p-values should never be zero",
  *Stat. Appl. Genet. Mol. Biol.* 9 (2010), arXiv:1603.05766.
- P. H. Westfall and S. S. Young, *Resampling-Based Multiple Testing*
  (Wiley, 1993): max-T and min-p.
- S. Holm, "A simple sequentially rejective multiple test procedure",
  *Scand. J. Statist.* 6 (1979).
- Y. Benjamini and Y. Hochberg, *J. R. Statist. Soc. B* 57 (1995);
  Y. Benjamini and D. Yekutieli, *Ann. Statist.* 29 (2001).
- R. G. Newcombe, "Interval estimation for the difference between
  independent proportions", *Stat. Med.* 17 (1998).
- D. Lemire, "Fast random integer generation in an interval", *ACM TOMACS*
  29 (2019).
- J. Besag and P. Clifford, "Sequential Monte Carlo p-values", *Biometrika*
  78 (1991).
- The Mars effect record: the Comité Para (1967), the Zelen test
  (1976–77), Kurtz, Zelen and Abell (1979), Abell, Kurtz and Zelen (1983),
  the CFEPP (1994), Ertel's eminence critique, Dean on misreported birth
  times. Summaries: <https://en.wikipedia.org/wiki/Mars_effect>,
  <https://skepsis.nl/gauquelins-mars-effect/>.
