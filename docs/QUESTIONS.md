# Open questions and decisions

Every question the research raised, each with options, trade-offs and a
recommendation. A decided question keeps its number, records the decision
and links its ADR. Status key: `open`, `decided`, `deferred`.

## Decided

| # | question | decision (2026-09-04) | record |
|---|---|---|---|
| Q1 | core language | Rust core, one audited `ffi` crate exposing a Teimeris-style C ABI | ADR-0001 |
| Q2 | binding generation | one description, generated bindings, parity gate; the generator chosen by the Phase 0 spike with numbers: option A (a designed C ABI, an extracted API description, our own generators in Rust), Diplomat rejected because its JavaScript and Dart backends cannot pass a host provider, marshal trees only through accessors, and serve Node only through wasm | ADR-0004, ADR-0007 |
| Q3 | binding order | Node native, wasm, Dart/Flutter, Python, Rust, then Java; Swift and Kotlin on demand; C and C++ headers from Phase 1 | roadmap |
| Q4 | v1 scope | **revised 2026-09-10**: v1.0 is the whole feature universe — Western and Hellenistic move from v1.x into Phase 7, the maintainer accepting the longer road explicitly. The original decision (v1.0 is baseline parity) held from 2026-09-04 until the developer-market survey falsified the evidence it rested on | ADR-0025, superseding ADR-0005's scope clause |
| Q5 | licence | Apache-2.0 open core; packs and adapters under their own terms | ADR-0006 |
| Q6 | ownership of the baseline engine's content | Teispace owns the SDK and the baseline engine's interpretations, rules, names and corpora; the baseline engine will replace its packages with the SDK | this table |
| Q7 | fallback ephemeris | a built-in analytic ephemeris ships in v1 as its own module and phase | ADR-0008 |
| Q8 | who computes houses and the rest | everything is in the SDK from v1: the `astro` layer owns every computation above raw positions; providers may declare native overrides | ADR-0009 |
| Q9 | calendars in v1 | at least the baseline engine's set plus Julian, mixed, ISO week and the Indian lunisolar calendar; the rest as plug-ins later | calendar architecture page |
| Q10 | localisation | Teistro Intl, one opinionated standard in the manner of next-intl and slang | ADR-0010 |
| Q11 | docs site | Fumadocs | ADR-0012 |
| Q12 | prose conventions | British spelling; Teimeris comment discipline | guideline 01 |
| Q13 | numeric policy | precision-first `f64` with error-bounded algorithms | ADR-0011 |
| Q14 | team and infrastructure | the maintainer and this assistant; public repository | roadmap |
| Q15 | Teimeris relationship | Teimeris updated as needed | ADR-0002, ADR-0009 |
| Q16 | names | GitHub `teispace/teistro-sdk`; npm `@teistro/*`; crates `teistro-*`; PyPI `teistro`; pub.dev `teistro` and `teistro_flutter`; adapters published separately. **Held from 2026-09-10**: the `teistro` npm organisation exists and `@teistro/sdk@0.0.0` is published, and `teistro` plus the `teistro-*` crates are being reserved on crates.io as placeholders that say so. Every one of them was still free that morning, checked against each registry's API. PyPI and pub.dev are not held: pub.dev removes placeholders, so it is watched instead | implementation page, `06-cicd/03-release-process.md` |
| Q17 | built-in ephemeris tiers and data terms | all three tiers, `standard` default; published-series tables with citations in `NOTICE`; Pluto fitted from a public-domain JPL kernel | ADR-0013 |
| Q18 | contributor agreement | DCO sign-off, no CLA | guideline 07, `DCO` |
| Q19 | commit convention | Conventional Commits with bodies that say what was wrong | guideline 05 |
| Q20 | Teistro Intl source conventions | base locale `en-Latn`; JSON canonical with YAML accepted; `i18n/<locale>/<namespace>.json` | ADR-0010 |
| Q21 | Teistro Intl for consumer applications | yes, the same engine, CLI and typed accessors | ADR-0010 |
| Q22 | eclipses and the full star catalogue | v1.x; anchor stars and yogataras in v1.0 | ADR-0013 |
| Q23 | provider override policy default | `prefer-native`, with `sdk-only` selectable, both gated | ADR-0013 |
| Q24 | contact for conduct and security reports | `support@teispace.com` for both, with GitHub's private channel kept beside it | `SECURITY.md`, `CODE_OF_CONDUCT.md` |
| Q25 | tooling language | everything we author is Rust: repository tasks as `cargo xtask`, consumer tools as Rust binaries, generators in Rust; no Python, `just` or shell scripts; only the docs site, the bindings' own-language layers and workflow YAML are not Rust | ADR-0014 |
| Q26 | exact classification and periods | `f64` astronomy stays; angles as canonical nanoarcsecond integers; classification by exact integer arithmetic with half-open boundaries; dasha spans as exact rationals; classify from what is serialised | ADR-0016 |
| Q27 | kernel and table | one kernel per family, systems as cited rows, falsified before code, whole-table invariants, a lazy dasha cursor; a system needing its own code is a kernel defect | ADR-0017, `03-design/` |
| Q28 | evidence and unsourced variants | texts outrank the baseline engine, which outranks third parties; rows carry V, T, S; unsourced variants are registered and refused, never silent defaults; discrepancies go to the cruxes register | ADR-0018 |
| Q29 | clean room and containment | `CLEAN_ROOM.md`; a licence allow list in `deny.toml` with copyleft and MPL denied; oracles unpublished; adapters outside; a test-provider-only build; Swiss adapter rules | ADR-0019 |
| Q30 | calculation version and provenance | a calculation version bumped on any numeric change; the cache key; the envelope extended with time, calendar, deviation, conventions and confidence fields | ADR-0020 |
| Q31 | reference-accuracy ephemeris | the IAU routines as an ERFA port; a JPL DE file reader provider (v1.x); a DE-refit `reference` tier; stage-isolated validation | ADR-0021 |
| Q32 | determinism and conformance | byte identity across architectures compared by hash; the corpus in a separate CC0 repository consumed as a pinned submodule; the quality bar extended (mutation, instruction counts, feature matrix, semver, compile-fail) | ADR-0022 |
| Q33 | type safety in every binding | validated newtypes, generated typed surfaces with documentation and examples in every binding, runtime schemas, coherence validation, typed states and errors, compile-fail and strictness gates | ADR-0023 |
| Q34 | the default profile | `parashari-classical`, the texts as read, patching the root and nothing else: geocentric, Gregorian, the three pan-Indic eras, an undefined polar day, with Sripati bhava, proportional ghatis, eight chara karakas and the Surya Siddhanta's orbs. It was declared over `nepali-default` and so had inherited that engine's topocentric centre and Nepal's calendar; the centre alone moves the Moon by up to 39′ and changes a mahadasha lord in one of the six charts the corpus records both ways | ADR-0024 |
| Q36 | chart geometry and drawing | layouts are cited rows a consumer can extend; geometry lands in Phase 4 as chart data; an optional first-party `render-svg` above the core turns geometry plus a theme record into a deterministic SVG in every binding. The core still never draws | ADR-0026 |
| Q37 | the lunar theory, after ELP/MPP02 proved unobtainable | ELP2000-82B with a quadratic bija correction to its mean longitude: 24 bytes take the Moon from 19.4″ to 3.0″ over 1800 to 2400 and 0.26″ over the span it is fitted to. Tier bounds become per body and per span; ADR-0021's `reference` budget rises to 4 MB, because a fitted Moon alone is 3.76 MB | ADR-0027 |

Principle confirmed by the maintainer: the baseline engine's packages are the minimum
bar, not the model. The SDK is structured, designed and governed as a
large-scale, professional, open-source, market-ready product from the
first commit. Confirmed 2026-09-04: every API and signature in every
binding is type safe, offers suggestions, and is robust (Q33).

## Q24. Contact addresses for conduct and security reports: `decided`

**support@teispace.com**, for both, given by the maintainer on
2026-09-10. One address rather than a `security@` and a `conduct@`: two
mailboxes only help when different people read them, and here they would
not, while a reporter who has to choose between them may choose wrongly
and a report may sit unread.

`SECURITY.md` and `CODE_OF_CONDUCT.md` name it, and both keep GitHub's
private channel beside it — a reporter who would rather stay on GitHub,
or who does not want to write from an identifiable address, should not
have to.

## Q35. An MCP server, so an agent can compute rather than guess: `deferred`

Raised by the maintainer 2026-09-09 and **deferred by them to the end of
the plan**, to be discussed before anything is built. Recorded now so the
analysis is not repeated.

*Why it fits here rather than being a fashion.* Three things the SDK
already has:

1. **It would be a sixth emitter, not a new product.** The binding
   architecture is a C ABI, an extracted API description and generators
   (ADR-0004); five emitters exist. `idl/api.json` already carries per
   field a `unit`, a `brand`, a `range` and an `example`, and per
   function a documented purpose — which is a tool's JSON Schema with
   its description, minus the transformation. The eighteen boundary
   fields that gained units for the Python binding are the same fields
   that would make a tool self-describing.
2. **The provenance envelope is the differentiator.** A model asked for
   a nakshatra answers with no ayanamsha named: the ambiguity that makes
   this domain hard is the part a model silently drops. Every value the
   SDK returns is stamped with the settings hash, the input hash, the
   profile, the provider, the calculation and catalogue versions, the
   time and calendar resolutions and any classical deviation (ADR-0020).
   A tool that returns *that* is auditable and reproducible.
3. **It composes with the engine passthrough.** B1 and B2 of
   `02-plan-performance-and-passthrough.md` build a dispatcher over an
   engine's self-describing manifest; tool discovery over a manifest is
   the same mechanism, so an agent could reach the engine's own
   operations without a second design.

*The two hard parts, so they are not discovered late.* Several of the 41
functions are lifetime plumbing — `ts_context_new`, `ts_string_free`,
`ts_blob_free`, `ts_context_last_error` — and must never become tools, so
which functions are agent-callable belongs in the **description** as a
`meta` flag rather than as a list inside an emitter, which would be a
second list to forget. And the settings tree cannot become tool
parameters: the shape to weigh is a `profile` plus a settings patch
object, with a separate tool that lists and explains the knobs, which the
SDK can generate because `Settings::knob_paths` is already gated against
the settings document.

*What to settle when it is discussed:* where it runs (a local stdio
binary, a hosted service, or an embeddable surface both wrap); what the
tools cover first (compute alone is available today from Phase 4,
interpretation needs Phase 6, passthrough needs B1–B3); and whether the
generator lands early — because a generated surface grows itself as the
SDK grows — with only the packaged, signed, install-checked server in
Phase 9.

## Q40. Whether a modern engine's overrides reach a chart's day and zodiac: `decided`

Raised 2026-09-26 by building the classical chart
(`03-design/classical-chart.md` §8). ADR-0013 says `prefer-native` uses a
declared native computation, "so results agree exactly with
Swiss-compatible tools through Teimeris". The completion does: a
sidereal position, the obliquity and a crossing are the engine's own.
**A chart does not**: its day comes from the SDK's rise and set solver
and its zodiac from the SDK's catalogue, whatever the engine declares.
For a classical astronomy that was a defect and is fixed, because the
text's sunrise and zodiac are definitions with no SDK answer to agree
with. For a modern engine it is a choice nobody made.

*What is measured* (the conformance kit, `ephemeris-port-and-adapters.md`
§9, Teimeris 0.1.0): the engine's sunrise stands 0.13 s from the SDK's
under the geometric convention and **7.3 s** under the refracted one, the
refraction model's difference (C34: 34.46′ Sinclair against 34.48′
Bennett at the reference air, parting with the temperature); its
ayanamsha members sit within 0.011° of the published values. Every figure
is inside the published bounds, so the choice moves no chart outside
them — but it moves every Teimeris-backed chart's bits, and a sunrise by
up to 7 s where the horizon is refracted.

*The options.*
- **A. The engine's own**, under `prefer-native`, as the completion does.
  A chart over Teimeris is then Swiss-exact; charts over two modern
  engines differ by their models, and `sdk-only` is the way back to
  one definition.
- **B. The SDK's, as today**, and ADR-0013 amended to say `prefer-native`
  governs the completion's steps and not a chart's conventions. One
  definition of a chart's day and zodiac across every modern engine;
  never Swiss-exact on sunrise.
- **C. A knob**, `provider.chart_conventions: SDK | PROVIDER`, default
  `SDK`. B's cross-engine identity by default, A's tool parity for a
  consumer who asks, and the choice declared rather than implicit, which
  is the no-dead-ends rule.

*Recommendation as raised: C.* The default keeps what a chart means
independent of the engine it was cast on, which is the property the
`sdk-only` byte-identity check guards; the knob gives the consumer who
must match another tool to the second a way to, stamped in every result.

**Decided 2026-09-26: B**, by the maintainer handing it over to be
researched and settled. Measuring the premise of A and C — that the
engine's own answer is the Swiss-exact one — refused it:

- **The day.** Against the Swiss-based recording, under its own
  convention (sea level, the upper limb refracted), over the 55 charts'
  106 events: the SDK's solver 9.77 s at worst, Teimeris's own search
  **32.39 s** (`adapters/ephemeris-teimeris/rust/tests/fixtures.rs`,
  which now asserts the order and says to reopen this question if it
  flips). The Swiss family disagrees with itself as much as with the SDK
  (C34), and the port's `STANDARD_REFRACTION` already means each side's
  own standard air.
- **The zodiac.** The port's `ayanamsha_deg` is the **mean** value; the
  recording applies the nutated one, which the catalogue reproduces
  within 0.0086″ under the true basis while the mean stands 18.46″ off.
  Taking the engine's value would have moved every chart over Teimeris
  18″ *away* from the tools ADR-0013 wanted it to match.

So A would have spent the chart's engine-independence to agree *less*
with Swiss-compatible tools, and C's knob would exist to choose the
worse answer; the consumer who wants the engine's own sunrise regardless
reaches it through `sdk.engine`, and every chart already stamps
`day:SDK` and `zodiac:SDK`. ADR-0013 is amended to say `prefer-native`
governs the completion's steps, not a modern chart's conventions; a
classical astronomy still defines both.

Deciding it found a latent defect: the completion took a modern
provider's native ayanamsha on **either** basis, so a caller asking for
the true one would have been the nutation short. A modern provider now
answers the mean basis alone; the catalogue answers the true one under
`prefer-native`, and `native-only` refuses it by name. No caller asked
for the true basis through the completion yet, so no number moved.

## Q38. Whether a computed value that is not a catalogue member should become one: `decided`

Raised 2026-09-22 by migrating the baseline engine's state readings.
**74 of the 126 records still unmigrated describe something the SDK
already computes and gives no key to.**

`rules::longevity` computes all three ayurdaya methods, the four haranas,
twenty maraka reasons and the vulnerability over a dasha's running
levels, and serialises every one of them in **kebab-case**. *(As raised,
this sentence went on: "which is the very spelling the corpus keys by".
It is kept here as it was written because the decision below is what
measuring it produced — it is true of `pindayu` and of five other keys,
and false of the other forty.)* `rules::LifeClass` is what `sdk.reading.lifeClass`
has said in two languages since the readings landed.
`houses::Quadrant` is which third of the wheel a bhava stands in, and
trikona, dusthana and upachaya are predicates on a `Bhava`.
`panchanga::Muhurtas` carries Abhijit and Brahma as **fields**. None of
them is a catalogue member, so no reading can hang on any of them and no
composer can say one (`03-design/state-readings.md` §8,
`03-design/interpret-measured.md`).

The question is therefore not *does the SDK model this?* but **did the
model reach the catalogue?** — and it is worth deciding once, because the
same answer settles a dozen instances.

**Option A — give them kinds.** Each becomes a `catalogue/<kind>.yaml`
with its members, and the readings migrate as every other category did.
It is what the corpus is waiting for and what a composer needs.
Against it: a kind's **number is permanent at the C boundary and is never
reused**, so a kind added and regretted is carried forever; and
`entity-names.md` §4 refuses a translated stub, so every member needs a
name from a vetted source before any locale can print it — though a kind
with no names is legal and useful (`state` and `rule` are both), and the
readings are the text a consumer actually wants.

**Option B — leave them Rust enums and let the readings wait.** Costs
nothing now and keeps the catalogue small. Against it: the records exist,
are translated into five languages, and are unreachable; and the same
question returns with every corpus that keys onto a computed value.

**Option C — a kind for each *family* as its module lands**, rather than
all at once: `life_class` with Phase 5's longevity work, the muhurta
kinds with Phase 7's muhurta search. Spreads the permanent decisions over
the phases that have the sources to name them.

**Option D — an *open* kind per family, added now.** This option did not
exist when the question was written, because the evidence for it was
produced afterwards. An open kind is **five lines of YAML** and a
permanent number, and it enumerates **no members at all**:

```yaml
kind: rule
number: 32
version: 1
open: true
doc: "Rule keys, which rule packs register at runtime; the catalogue holds the kind only."
```

`rule` (32) and `graha_bhava` (65) are both this shape, and both carry
readings today. The composer builds the key from what the SDK already
computes — `graha_bhava_key(graha, bhava)` is the precedent — and the
reading renders from the pack.

**What closed `planet-condition` on 2026-09-22 is the proof.** Its seven
readings key onto `dignity.*` and `state.*`, and `state` is **named by no
locale at all**: it is not among the 34 kinds `sdk.entity` holds. The
readings are said anyway, because `{$state :entity kind=state form=phala}`
asks for the `phala` **form** and never for the name. So the sentence in
Option A — "every member needs a name from a vetted source before any
locale can print it" — is **false of a reading**. It is true only of a
composer that would print a member's *name*.

The gate cost is smaller too. `check-intl` scopes itself to the kinds the
**document schema** carries, so a kind that only a readings pack keys onto
never enters it: no `UNNAMED` listing, no vetted table, nothing to
maintain. `rule` and `graha_bhava` are outside that scope for exactly this
reason.

What D still costs, and it is the whole cost: **one permanent number and
one permanent name per family**, eight of each. A number is never reused
and a kind's name is what every locale, pack and binding will spell
forever.

D also wants a gate of its own, in this repository's usual shape: for each
family, every variant of the Rust enum has a record in the pack and every
record names a variant, failing both ways. Without it, the composer
builds a key by `format!` and a typo is silent — which is the one safety
an open kind gives up against a closed one.

**Recommended: D, and it is one commit of YAML away.** A is right about
the destination; its objection about names was measured and is false for
readings. C is right that the phases with the texts open should choose the
*names*, but wrong that the readings must wait for them — a reading needs
a key space, not a vocabulary, and D gives it one for five lines. B is
only tenable if nobody minds the readings staying unread, which the
measured pages now count out loud: **74 records, translated into five
languages, reachable by nothing.**

This stays `open` rather than decided because a kind's number is permanent
at the C ABI and its name is permanent everywhere, and that is the one
class of change worth a maintainer's yes even under a broad delegation.
The eight names D would spend are the decision: `ayurdaya_method`,
`ayurdaya_harana`, `ayurdaya_maraka`, `ayurdaya_maraka_trigger`,
`ayurdaya_tier`, `ayurdaya_vulnerability`, `ayurdaya_classical_rule`,
`ayurdaya_balarishta` — or better names, which is exactly what a
maintainer is for.

What is **not** waiting on this: 26 of the 38 categories are migrated,
gated and said. The rest are named on
`03-design/state-readings-measured.md`, so the list cannot rot while the
question is open.

**Decided: C**, on 2026-09-22, the maintainer having delegated the
choice — and **D is refuted**, by the measurement its own recommendation
asked for and did not make.

**The rule, which is the part that generalises.** A computed value earns
a catalogue kind when a consumer must be able to **name** it. Where a
consumer only needs to *select* on it, the key it already crosses as is
enough, and `.match` on that key is the pattern: `sdk.reading.lifeClass`
has selected on all seven classes of life in two languages since the
readings landed, `vimshopaka` says its four schemes the same way — "four
words" — and `bhava_bala` needed no vocabulary at all. None of these
values is unreachable today: every one of them crosses `rules_json` as
its own kebab key, so a consumer can read it and switch on it now.

**And a corpus reading is blocked by a key space only when the corpus
keys by the vocabulary the SDK computes.** That was the claim under
Option D, and under this question's opening sentence, and under
`state-readings.md` §8, and it was never checked. It is now, on
[`03-design/state-readings-measured.md`](03-design/state-readings-measured.md):
over the 74 records of the ten categories whose subject the SDK
computes, **6 of the keys are the SDK's own spelling**. Not 74.

The claim was read off `ayurdaya-method`, where it is perfectly true —
`pindayu`, `nisargayu` and `amsayu` are three for three — and
generalised. What the rest of the family has is not a spelling
difference:

- `ayurdaya-maraka` keys by **nine classes** (`lord-2`, `occupant-7`,
  `associate`) where `maraka::Reason` has the verses' **twenty**
  (`lord-of-second`, `malefic-in-seventh`, `malefic-with-second-lord`).
  Mapping twenty onto nine is a judgement about which reasons are one
  class, which is a reading of the tradition and not an alias list.
- `ayurdaya-classical-rule` keys by five verse citations of an edition
  the SDK does not ship; the SDK ships eleven Brihat Jataka balarishta
  rules with their own keys.
- `ayurdaya-vulnerability` wants three severity bands and six
  conditions; `Vulnerability` is a struct that grades nothing.
- `shadbala-strength` wants four bands where `GrahaShadbala::strong` is
  a verdict against the required rupas — and §8 had this one right all
  along: the bands are a threshold the corpus does not record, and
  inventing four would be making up a rule.
- Only `ayurdaya-harana` (4) and `ayurdaya-tier` (3 of 4) are genuine
  alias lists, the shape `kaala` closed with.

**A kind supplies a key space. It cannot supply a vocabulary.** Option D
would have spent eight permanent numbers and some thirty-four permanent
member keys to land three readings outright and eleven with alias lists,
and left thirty-five blocked on exactly what they were blocked on
before. That is the whole of the refutation.

**Why C and not B.** B is "leave them and let the readings wait", which
is true of what happens and wrong about why. The readings were never
waiting on the catalogue. They are waiting on a **classification**, and
the phase that reads the sources is the one that can choose it — which
is what C said. Phase 5's longevity work has the texts open; it decides
whether a maraka is named by the verse or by the class, and whether the
tier is `short` or `alpayu`. Spending the number now means choosing that
spelling now, from this desk, with no source in front of it.

**What is bought instead, now.** The measurement is on the page and
gated as far as this repository can gate it: five of the six vocabulary
sizes are **counted from the type** — `Method::ALL`, `LifeClass::ALL`,
`Reason::ALL`, a default `Reductions` serialised, and the shipped
balarishta pack — and the pass fails when a stated size disagrees.
`Method` gained an `ALL` for it, the one sibling that lacked one. The
row that forced it is `ayurdaya-maraka`: §8 said **fifteen** maraka
reasons where the type has twenty, which is the count-in-prose this
repository keeps finding. The keys-it-spells column is **recorded and
not gated**, and the page says so in as many words: this repository
carries the migrated packs and not the exporter's document, so there is
nothing here to count it from.

**What would reopen it.** A vetted source that names one of these
families — which is a new RFC with new evidence, as every reopening is.
The one family whose shape is already settled is `shadbala-strength`:
its keys are a graha and a band, so when a banding rule exists it is a
**composite open kind**, `graha_bhava`'s shape exactly, and that part
needs no deciding again.


## Q39. Whether a plan should say the almanac's own limbs: `decided`

Raised 2026-09-22 by the section table on
[`03-design/interpret-measured.md`](03-design/interpret-measured.md),
which enumerates every section a chart document can carry and what says
it. `PANCHANGA` is **said only where a corpus carries a reading**:
`phala` renders a loaded record for the tithi, vara, nakshatra and yoga
and says nothing of its own, so a consumer with no pack gets no item
from that section at all. The limbs themselves — and the **karana**,
which no composer mentions in any form — are computed, named in all five
locales, and said by nothing.

Everything a composer would need is already bought: `tithi`, `vara`,
`nakshatra`, `yoga` and `karana` are catalogue kinds whose members every
strict locale names, and the tithi carries its paksha. The cost is a
frame a message, as `states` was, and nothing for the native review.

The question is whether a **plan** should recite them, and it is a
design call rather than a task, because the page has twice declined a
message for being the wrong *kind* of thing to say:
`sdk.reason.exactLongitude` renders a longitude alone (`222°34′35″`) and
is a fragment a consumer formats with, not a sentence a plan says; and
`sdk.reason.rashiNature` is a fact about the zodiac rather than about a
chart, where every chart would say the same twelve sentences.

**Option A — a composer over the panchanga section.** A thirteenth
member of `PlanRequest`, six items a chart, and a reading that opens the
way every panchangam opens. Neither declined precedent applies: "the
tithi is Shukla Pratipada" is a sentence rather than a fragment, and it
differs chart by chart. Against it: a `PlanRequest` member is permanent
API, and two were added on 2026-09-22 already.

**Option B — leave it to the consumer.** The section crosses to every
binding and a consumer reads it directly; `sdk.entity` names every limb,
so a page can say it in one line without a composer. Against it: the
karana is then said by nothing anywhere, and "a plan is what a chart has
to say" reads oddly when the chart's own day is missing from it.

**Option C — fold the limbs into `phala`**, which already reads this
section, rather than adding a member. Costs no API. Against it: `phala`
is the composer that says what a **corpus** carries and is silent
without a pack, by design and by its own documentation; putting computed
facts in it would make "off until a pack is loaded" untrue of it.

**Decided: A**, on 2026-09-22, the maintainer having delegated the choice
("you can research, analyse and find what is best to do"). The two
precedents that declined a message declined it for reasons that do not
hold here — a limb is a sentence rather than a fragment, and it differs
chart by chart — and C would break a promise `phala` makes in its first
paragraph. B was the cheaper answer and it leaves the **karana** said by
nothing at all, in a library whose whole subject is the almanac.

Built as `panchanga`, the thirteenth composer, over
`sdk.reason.panchanga`: the five limbs, the Moon's pada in its nakshatra,
and whether the birth fell between sunrise and sunset. Every slot is a
catalogue member every locale already names, so the frame was the whole
cost and the native review sees nothing it has not seen
(`03-design/interpret-composers.md` §4).

The scope was drawn where the *shape* changes rather than where interest
runs out: `spans`, `on_sankranti` and the eclipse are on the same record
and are **not** said, because a span is a pair of ghatikas a consumer
formats with — the `exactLongitude` line — and a sankranti and an eclipse
are conditions of the day rather than limbs of it. The section table on
`03-design/interpret-measured.md` carries that as what is left, so the
line is measured rather than remembered.

## Decisions log

Decisions are recorded in the table above with the date; the reasoning is
in the linked ADR or page. A decision is reopened only by a new RFC with
new evidence.
