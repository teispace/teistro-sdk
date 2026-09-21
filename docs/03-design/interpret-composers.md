# Composers: a chart's answers as a narrative plan

Status: `built`, designed and built 2026-09-21; measured in
[`interpret-measured.md`](interpret-measured.md). It builds the layer
[`../02-architecture/03-localization-architecture.md`](../02-architecture/03-localization-architecture.md)
fixes under "Composers" — *a composer returns a narrative plan: an ordered
list of (message key, slots); plans are language-neutral, testable and
serialisable, and `intl` renders them* — and it is Phase 6's `interpret`
([`../07-roadmap/00-roadmap.md`](../07-roadmap/00-roadmap.md)), the step after
a rule rendered to prose ([`rule-doc.md`](rule-doc.md)).

## 1. Purpose and scope

The SDK computes a chart and answers it by rule. Nothing turns those answers
into sentences a reader sees, in the reader's language. This page decides the
plan — what a composer returns — the first composers, what decides a message
key, and the gates that keep a plan renderable in every shipped locale.

In scope: the `Plan` and `Item` types, which composers come first and what
they take, how a key is chosen, serialisation, determinism, and the
measurement.

Out of scope: the **interpretation records** — the baseline engine's 108
graha-in-bhava cells and its yoga and dosha readings in four languages —
which reach `i18n/` through `teistro-intl migrate baseline` and are a step of
their own; the **report section catalogue**, which groups plans into a
document; and **crossing the C boundary**, which rides on a chart reading the
way rules do ([`rules-at-the-boundary.md`](rules-at-the-boundary.md)) and is
worth deciding once there is more than one composer to carry.

## 2. What the research found

- **The shape is already decided, and so is the runtime.** `Params` is
  `BTreeMap<String, Value>` — ordered, so a plan is deterministic by
  construction — and `Value` already carries text, integers, numbers, a
  catalogue key, a list, a date, a time and a ghati count, which is every
  slot the messages below need. `sdk.intl.render(key, &params)` answers with
  a `Rendered` that says which locale resolved it, whether it fell back and
  what warned. Nothing new is needed to render a plan.
- **`Value` is not `Serialize`, and a plan must be.** The derive belongs on
  `Value` in `crates/intl`, where the type lives, rather than on a mirror
  type in `interpret`: two value enums would be two lists to keep in step,
  which is the defect this project has hit before.
- **The first composer's whole vocabulary already exists, in both strict
  locales.** `i18n/*/sdk.reason.json` carries `grahaInBhava`, `grahaInRashi`,
  `grahaAt`, `conjunction`, `lordship`, `occupants`, `rashiNature` and
  `strength` in `en-Latn` and in `ne-Deva-NP`, translated by hand. That
  decides which composer is first: `ne-Deva-NP` is a **strict** locale, so a
  new key is a key that must be translated before it can ship, and the
  project forbids machine-translated stubs ("a plausible wrong astrological
  term is worse than a visible fallback"). A first composer that adds no key
  proves the whole path — chart to plan to text in two languages — with no
  translation debt at all.
- **The corpus records no interpretation text.** Checked over every section
  on 2026-09-20: it holds numbers, keys and flags, and not one composed
  sentence. So a composer has **no oracle** here — the corpus can neither
  derive a plan nor falsify one — and the roadmap's byte-for-byte comparison
  against the baseline engine's text waits on `migrate baseline`. What can be
  measured over the corpus is **coverage**: run the composers over every
  recorded chart and count what they say, what they cannot say, and what
  would fall back.
- **The façade's reading types are the façade's.** `RulesReading` and
  `Present` live in `crates/sdk`, so a composer taking them would put
  `interpret` above the façade and invert the crate graph. Composers take the
  kernel's own types — a placement, a `&Rule` with its `RuleResult`, a
  `HouseReading` — and the façade adapts what it holds into them.

## 3. The plan

```rust
/// One thing to say: a message and its slots.
pub struct Item {
    pub key: String,
    pub params: Params,
}

/// What a composer says, in the order it says it.
pub struct Plan {
    pub items: Vec<Item>,
}
```

- **Flat and ordered.** A composer is about one subject and says its piece in
  order; a report that wants several concatenates their plans. Grouping is
  the report catalogue's job, not the plan's.
- **No text.** An item carries keys and values, never a rendered word, so one
  plan renders in every locale and a snapshot of it is a test of the composer
  rather than of the translator.
- **Serialisable**, both ways, so a plan crosses a boundary and a golden file
  holds it.
- **Deterministic**: the slots are a `BTreeMap` and the items are a `Vec` the
  composer fills in a fixed order, so the same chart gives the same bytes.

## 4. The first composers

**`placements`** — what the chart is, from the kernel's `RuleChart`: for each
of the nine grahas and the lagna, where it stands (`grahaInRashi`,
`grahaInBhava`), and for each sign holding two or more, who shares it
(`occupants`). Every message is one of the eight that already exist.

**`readings`** — what the rules answered: for each rule a chart held, what
its verse says, who took part, whether a cancellation moved it and how grave
it is. **Built** over a namespace of its own, `sdk.reading`, six messages in
both strict locales.

Its messages are deliberately **mechanical** — a span, a class of life, a
list of grahas with a verb that agrees with it, a status, a severity —
because those are the parts a locale can say for itself. **The verse's own
statement is not translated**: it crosses as a `text` slot in the words the
rule cites, and the message prints them as they are, so a Nepali reading says
everything but the verse's sentence in Nepali. A machine translation there
would be worse than the visible seam, and the seam closes when a locale
carries a reading of that rule written by someone who reads the text.

**`strength`** — what the chart weighs: each graha's Shadbala in rupas,
the strongest first. **Built**, and it adds no message either:
`sdk.reason.strength.score` was already carried by both strict locales and
used by nothing, so the third composer ships with no translation debt for
the same reason the first did.

It is the first composer over a **section** rather than over the chart's
placements or a rule's answer — it reads `Document.shadbala`, which a
request asks for by name and which the boundary now computes for it, as it
computes what a rule set reads. That is the shape the remaining composers
take, so it is worth having one of them built.

Two things it deliberately does not do. It does not say whether a graha
**reaches** the rupas its text requires: the Shadbala carries
`required_rupas` and `strong`, and no locale carries a message for either,
so the plan is silent and the measured page counts the silence. And it does
not emit `sdk.reason.strength.rank`, which renders an ordinal alone — `1st`,
`१लो`. That is a **fragment a consumer formats with, not a sentence a plan
says**, and the distinction is worth stating: a message in the pack is not
automatically a plan item, and `KEYS` lists what the composers emit rather
than everything the locale has.

Every item names its rule in a `rule` slot the base messages declare and do
not print, so a consumer can group a plan by rule and a locale that wants the
key in its prose has it. The measured page's snapshot prints it as a prefix,
because a reviewer of a rendering needs to know which rule said what.

## 5. What decides a key

A composer that emits a key the locale does not carry produces a visible
fallback, which is a defect and not a feature. Two rules keep that from
happening:

1. **Every key a composer can emit is listed** in the module, `KEYS`, and a
   gate holds the list against the base locale and against every strict
   locale, failing both ways — the pattern the rules kernel's `KINDS` list
   follows.
2. **Where a composer chooses between a general key and a specific one** — a
   reading of "any rule" against a reading written for `MAHAPURUSHA_HAMSA` —
   it asks the **base locale**, not the reader's. The base locale is a fact
   of the build; the reader's locale is not, and a plan that changed shape
   with the reader would not be language-neutral. The trait that asks arrives
   with the composer that chooses (step 4): `placements` has one key for each
   thing it says, and an interface with no implementor is an interface
   designed from a guess.

## 6. The measurement, and the gates

`cargo xtask interpret` writes `interpret-measured.md` and `check-interpret`
holds it. Over every readable chart of the corpus:

- how many items a chart's plan carries, and of which keys;
- what the composer **cannot** say — a body the chart does not place, a rule
  whose verse states no effect — counted rather than hidden;
- that every item renders in **both strict locales** with no fallback and no
  warning, which is the honest end-to-end check available without an oracle:
  `Rendered` reports both.

Beside the pass, in the crate: a golden plan for one chart, and its rendered
text in English and Nepali (the checklist's "composer plans have snapshots
per language"); a test that the key list matches what the composers emit; and
a test that a plan round-trips through JSON.

## 7. Order of work

1. `crates/interpret` with `Plan`, `Item` and the `placements` composer;
   `Value` given serde derives in `crates/intl`; the key list and its gate;
   snapshots in English and Nepali. **Built.**
2. The pass and `interpret-measured.md`, `check-interpret` in `fast-check`.
   **Built**: 93 recorded charts, 1878 items, 3756 renderings with no
   fallback and no warning, and one chart said whole in both languages.
3. The façade: `sdk.chart().plan(…)` returning a plan, and `sdk.intl` already
   renders it; the Rust example.
4. `readings`, with the `sdk.reading` namespace written in English and
   Nepali, and the rule's own cited effect as a slot where no locale carries
   a reading of its own. **Built**: 93 charts against five shipped packs
   compose to 8347 items, 16 694 renderings, no fallback and no warning.
   The Nepali of `sdk.reading` awaits the native review the roadmap's exit
   criterion already requires for `ne` and `hi`; the terms are the texts'
   own (अल्पायु, मध्यायु, पूर्णायु), not invented prose.
5. The plan at the boundary, so a composer added reaches four languages
   rather than one. **Built**: `03-design/plans-at-the-boundary.md`.
6. `strength`, the first composer over a section. **Built**: it adds no
   message, reads `Document.shadbala`, and is measured over the corpus's
   **recorded** Shadbalas rather than the SDK's computed ones — the numbers
   themselves are `check-shadbala`'s business, and what this pass decides is
   whether a plan made of them can be said.

## 8. What this design does not settle

- **The interpretation records.** 108 graha-in-bhava cells with condition
  modifiers and conjunction synthesis, and a reading for each yoga and dosha,
  in four languages, are data and not code; they arrive through `migrate
  baseline` and turn `placements` from a description into an interpretation
  without changing its shape.
- **A consumer's own composer.** The extensibility table already promises one
  (a plan function in Rust, a declarative plan in v1.x). A registry costs
  nothing to add once a second composer exists to prove the interface, and
  guessing it from one is how a registry gets the wrong shape.
- **The report.** Sections, their order, and which composers fill them.
