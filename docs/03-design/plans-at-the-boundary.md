# Plans at the boundary: a chart's narrative, said in every binding

Status: `building`, designed 2026-09-21; §8's four steps built the same
day. It settles what
[`interpret-composers.md`](interpret-composers.md) §1 put out of scope —
*crossing the C boundary, which rides on a chart reading the way rules do
and is worth deciding once there is more than one composer to carry* — now
that there are two, `placements` and `readings`. The road it takes is the
one [`rules-at-the-boundary.md`](rules-at-the-boundary.md) laid.

## 1. Purpose and scope

The composers turn a chart's answers into a narrative plan, and `sdk.intl`
renders that plan in any locale the engine carries. Both halves already
cross the boundary — the chart does, the renderer does — and the half
between them does not, so Node, Dart and Python can compute a chart and can
render a message, and cannot get from one to the other.

In scope: how a plan is asked for, what comes back, what a binding does with
it, errors, cost, and the gates.

Out of scope: the composers themselves, which add keys and not
mechanism — `interpret-composers.md` is where they are decided and
`interpret-measured.md` counts them; the report section catalogue, which groups plans into a document;
and a **rendered** plan at the boundary, which §7 declines with a reason.

## 2. What the research found

- **A plan's slots are already what `render` takes.** This is the whole
  reason the crossing is cheap, and it was not true a day ago: `Value` wrote
  a shape of its own until the parameters at the boundary and the slots in a
  plan were unified on the boundary's `$`-tagged JSON
  (`teistro_intl::wire`). All three bindings' `render(key, params)` take a
  plain map and `JSON.stringify` it, so an item parsed out of the blob hands
  straight back with **no conversion** — `sdk.intl.render(item.key,
  item.params)` in JavaScript, in Python and in Dart alike.
- **A plan is small enough to cross whole**, and the reason is not the one
  expected. Measured over the corpus's 93 charts
  ([`interpret-measured.md`](interpret-measured.md)): 12 573 bytes a chart,
  17 425 for the widest, 140 an item. The verses' own cited words — the one
  slot whose length belongs to a text rather than to the SDK — are **9%** of
  it, not the bulk. What a plan costs is its items, each naming its message
  and its rule again. Nothing here asks to be packed, sectioned or paged.
- **The boundary keeps no documents between calls**, so a plan rides on
  `ts_chart_found` as the SVGs and the rules do: a nullable JSON field on the
  request, a section of canonical JSON in the charts blob. An entry point of
  its own would found the chart a second time.
- **`readings` cannot compose what was not evaluated.** It takes `(&Rule,
  &RuleResult)` pairs, and `RulesReading::present` is exactly a list of them,
  so the façade's adaptation is a `map`. But it means the composer has no
  answer at all unless `rules_json` named rules — §5 refuses rather than
  answering an empty plan, because an empty plan and an unasked question look
  the same to a consumer and only one of them is their mistake.
- **`placements` reads the graha states**, through `RuleInputs::of`, and
  nothing else. So a request asking for a plan must compute the states
  whether or not `sections` asked for them, exactly as `rules_json` computes
  what its rules read.
- **The plan carries no locale, and that is the point.** The `svgs` section
  comes back in the context's locale because an SVG has words in it. A plan
  has none: one crossing serves every locale a consumer then renders in, and
  two readers of one blob can read it in two languages.

## 3. The request

One nullable field is appended to `TsChartRequest`, after `rules_json`;
`check-lints`' `handshake-is-checked` already holds the entry point to
refusing a struct shorter than it reads, so a caller compiled against the
older header is safe by not passing it.

```c
/* Narrative plans to compose over each chart, as JSON; null for none. */
const char *interpret_json;
```

It is an object, one member per composer:

```json
{ "placements": true, "readings": true }
```

- Every member defaults to false, so a consumer pays for what it asks, and
  null costs nothing.
- **Every member reaches every binding**, held by a lint. A `PlanRequest`
  crosses as JSON rather than as a struct, so none of the three bindings
  generates its surface from the API description: each spells the record
  itself, which is three copies of one list. `composer-reaches-every-binding`
  reads `PlanRequest::MEMBERS` and requires each name in both declarations
  of each binding — the request a caller fills in and the record of plans it
  gets back — because one without the other is a composer that can be asked
  for and never read, or read and never asked for. It was added after two
  composers in a row had to be remembered into three files by hand, and no
  other gate sees that failure: `check-parity` compares the values a
  scenario answers, and a composer nobody can ask for answers nothing.
- **An unknown member is refused**, naming the composers there are. A typo
  that silently composes nothing is the dead end the no-dead-ends mandate
  forbids; a composer added later is a member added here, and the refusal
  lists it from one place — `PlanRequest::MEMBERS`, which a test holds
  against the record's own serialisation both ways, because the hint was
  hand-written once and went stale one composer later.
- An object rather than a bit set, because a composer will want options of
  its own — which rules to read, which house — and a bit set has nowhere to
  put them. `sections` is a bit set because its members never will.

The façade reads the same record as `PlanRequest`, so the C boundary and
Rust read one type, and `sdk.interpret()` gains `readings(&RulesReading)`
beside the `placements(&Document)` it has.

## 4. The result

Section `plans` of the charts blob: canonical JSON, an array with one entry
per chart in the order asked for, each an object carrying only the composers
that were asked for.

```json
[ { "placements": [ { "key": "sdk.reason.grahaInRashi",
                      "params": { "graha": { "$entity": "graha.SUN" },
                                  "rashi": { "$entity": "rashi.ARIES" } } } ],
    "readings":   [ … ] } ]
```

- **A plan crosses as `Plan`'s own JSON**, which is the array of its items:
  `Plan` is `#[serde(transparent)]` over them, and this page guessed
  `{"items": [...]}` before checking. One shape either way, and the simpler
  one — a plan written by a Rust consumer, a plan in a golden file and a plan
  out of the blob are the same bytes, so a fixture moves between them and a
  binding's typed layer is a rename rather than a conversion.
- **An item's `params` are the render's `params`.** That is the acceptance
  test of this whole page, and §6 makes it one in each binding.
- A composer asked for on a chart it can say nothing about answers `[]` —
  present and empty, which is an answer — rather than being left out.

## 5. Errors and degenerate states

- `interpret_json` that does not read is `INVALID_ARG`, the field named from
  the request's root as `theme_json` and `rules_json` name theirs:
  `interpret.readings`.
- An unknown composer is `INVALID_ARG` naming the ones there are.
- **`readings` without `rules_json` is refused**, `INVALID_ARG` on
  `interpret.readings`, saying that a reading composes what rules
  answered and naming `rules_json`. A set that names rules none of which hold
  is not this case: it answers `[]`.
- A chart that cannot be read for a plan's inputs is not refused for the plan
  alone, as §5 of the rules page already settles for rules: the entry carries
  the same `unreadable` list, and a composer says what it can.

## 6. The gates

- `check-ffi` holds the new field, the section and the header, as it holds
  section 33.
- An ABI test founds two charts asking for both composers, reads each plan
  out of the bytes, and finds the section absent without `interpret_json`,
  `readings` refused without `rules_json`, and an unknown composer refused.
- **Each binding renders what it parsed**, which is the crossing's real
  acceptance test rather than a side effect of it: the test takes an item
  from the blob and passes `item.params` to that binding's own
  `intl.render`, asserting the text — no conversion step is allowed to
  appear between them: a gate that only proves the section parses would go
  green without the property the crossing exists for.
- **Every composer asked for alone answers, or says why not.** A member
  of `PlanRequest` is a promise that asking for it gets you something and
  that the section it reads is computed for it; `sections_for` is the one
  place that mapping lives and nothing held it. The ABI test asks for
  each member **by itself**, because asking for several together hides a
  missing section behind a sibling's — `strength` and `houses` each bring
  the states four other composers read, so a member whose own section was
  forgotten still answers in company. A member that refuses or says
  nothing fails unless it is listed with a reason, and the list fails
  both ways. Only `phala` is listed, because it says what a **loaded**
  corpus carries and that context loads none, which is the composer
  working rather than failing.

  It is worth saying what this does *not* catch, since the temptation is
  to believe a new gate covers the bug that prompted it. The almanac dead
  end of 2026-09-22 had the section asked for and the **document's**
  panchanga never turned into the birth's limbs, so `phala` returned an
  empty plan this test excuses. The pass that catches that one founds a
  chart with both corpora loaded and requires **every key** to be
  emitted. *Can it be asked for* and *does it ever say anything* are two
  questions, and each needs its own check.
- **The four parity runners print each chart's rendered lines** in
  `en-Latn`, so `check-parity` holds Rust, Node, Dart and Python to the same
  sentences. This is the first parity over **text**, and it exercises the
  composers, the wire shape and the locale engine in one comparison.

## 7. What this design does not settle

- **A rendered plan at the boundary** — an entry point answering sentences
  rather than keys and slots. Declined, for now, with a reason: it would put
  a locale on a chart request, make one blob readable in one language, and
  duplicate `ts_intl_render`, which already answers every item at 140 bytes
  of parameters. Worth revisiting only if a measurement shows the per-item
  crossing costs a consumer more than the render itself.
- **A typed plan in each binding**, generated over a schema, as the rules
  page defers its own typed result: the first step parses the section into
  each language's native value.
- **The composers' own options** — which rule set a reading reads, which
  house a house reading is of. The members are booleans until a composer
  arrives that needs more, and the object's shape is what leaves room.

## 8. Order of work

1. The façade: `PlanRequest`, and `sdk.interpret().readings(&RulesReading)`,
   tested in Rust against the composers directly — the same plan by two
   roads. **Built**: the area's `readings` is the `map` this page said it
   would be, and the test fails if it ever becomes more; a reading without
   rules and a composer that is not one are both refused by name.
2. `interpret_json` on `TsChartRequest` and section `plans` in the blob,
   with the IDL, the header, `check-ffi` and the ABI test. **Built**:
   section 34; the composers run over the charts the same crossing founded
   and the same rules it answered, never evaluating either again. The ABI
   test founds two charts, reads every item of both plans out of the bytes
   and **says each one through `ts_intl_render`** with its params handed
   straight back — no fallback, nothing warned — which is the property §6
   asks a gate to hold rather than the section merely parsing.
3. Node, Dart and Python: the option, the parsed section, and each binding's
   render-what-you-parsed test. **Built**: Node's `interpret` option and
   `chart.plans`, Python's `interpret=` and `chart.plans`, Dart's
   `PlanRequest` and `chart.plans`, each binding's test saying every item
   through its own renderer, and a tenth worked example each. Writing the
   same program three times paid again, as it has before: the Dart message
   accessors **did not parse**, because `sdk.reading.lifeClass` selects on a
   slot called `class` and the emitter wrote `required String class` —
   right in Python (`class_`) and moot in JavaScript, wrong only in Dart.
   `check-intl` now reads the identifiers back out of the Dart it generates,
   because a generated file being up to date says nothing about its
   compiling and the gate that compiles Dart runs in the verify matrix.

   The examples earned their keep a second time: Dart's `found` takes the
   new option and did not forward it to `foundMany`, so `chart.plans` was
   null for every single-chart call. The parity runners could not see it —
   they call `foundMany` — and Node cannot have the bug at all, because its
   `found` spreads the request. **Python and Dart list `found`'s options by
   hand**, so each new one is two places in Python and two in Dart, and the
   test that catches a missed one is the one that calls `found`. Both new
   binding tests do.
4. The four parity runners over the rendered lines, and `check-parity`;
   then `check-areas` and the surface pages, which should hold as they are:
   the plans ride on the `chart` area's `found`, so no area is added.
   **Built**: each runner prints every chart's plans said item by item, and
   the four agree on all **6918** values, up from 6758. The runners are in
   `ne-Deva-NP`, so the first parity over text is a parity over Devanagari,
   and it compares the composers, the params shape and the locale engine at
   once. No area was added, and the surface pages hold as they were.
