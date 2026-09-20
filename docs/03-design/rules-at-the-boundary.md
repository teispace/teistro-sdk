# Rules at the boundary: a chart reading that answers rules in every binding

Status: `built`, designed and built 2026-09-17. It settles the question
[`surface-areas.md`](surface-areas.md) §9 left open — how a rule and a rule
result cross the C ABI — so that Node, Dart and Python reach the kernel
[`rules-engine.md`](rules-engine.md) describes, which Rust reaches today
through `teistro::rules`, `teistro::RuleInputs` and the longevity readings.

## 1. Purpose and scope

The kernel ships 997 rules, a house reading, the dasha delivery of a result,
the three pairs, the three spans of life and the marakas. None of it is
reachable outside Rust. This page decides how a consumer in any binding asks
a chart for them and reads the answer.

In scope: the request, the result's shape, which rules can be named, errors,
cost, and the gates. Out of scope: rendering a result to prose (Phase 6's
`interpret`), and a consumer registering a rule pack by key on a context,
which §7 leaves for when a consumer asks.

## 2. What the research found

- **The boundary keeps no documents between calls.** A rule reads a founded
  chart; an entry point of its own would found the chart again. The SVG
  renderer met the same fact and answered it by riding on `ts_chart_found`: a
  nullable `theme_json` on the request, and a section of canonical JSON in
  the charts blob (`render-svg.md`, section 22 `svgs`). Rules take the same
  road for the same reason.
- **The request is versioned.** `TsChartRequest` carries `struct_size`, and
  `check-lints`' `handshake-is-checked` holds that the entry point refuses a
  shorter struct than it reads, so a field appended at the end is safe for a
  caller compiled against the older header.
- **Every result the kernel returns is already `Serialize`.** `RuleResult`,
  `HouseReading`, `ThreePairs`, `Ayurdaya` and `Vulnerability` serialise
  deterministically, and the boundary's `canonical_json` fixes key order and
  number spelling, so four bindings reading the same bytes agree by
  construction and `check-parity` can hold them.
- **What a rule set reads is derivable.** `ChartRequest::with_rule_inputs`
  asks for exactly the sections a set of rules reads (states always; the
  divisions, strengths and points only when a rule names them), so a consumer
  asking for rules need not also know to ask for Shadbala.
- **A polar birth has no special lagnas, and a reading asking for points is
  refused there.** A request for rules must not make a birth that could be
  read unreadable; §5 answers it.

## 3. The request

One nullable field is appended to `TsChartRequest`:

```c
/* Rules to evaluate over each chart, as JSON; null for none. */
const char *rules_json;
```

It is an object:

```json
{
  "shipped": ["nabhasas", "readings", "arishtas", "gandantas", "doshas", "yogas"],
  "rules": [ { "key": "…", "category": "…", "source": { … }, "conditions": [ … ] } ],
  "readings": "texts",
  "houses": true,
  "longevity": true
}
```

- `shipped` names the kernel's own sets by their `shipped::` function; `rules`
  is a consumer's own rules in the kernel's rule format, read strictly. Both
  may be given, and together they are one set: a consumer's rule may name a
  shipped rule by key. A key given twice is refused.
- `readings` is `"texts"` or `"recording-engine"`, the two named `Readings`;
  it defaults to `"texts"`.
- `houses` adds the twelve house readings; `longevity` adds the three pairs,
  the three spans and the marakas. Both default to false, so a consumer pays
  for what it asks.

The façade reads the same record as `RuleRequest`, validated into a `RuleSet`
(each key once, every reference resolved), and `sdk.chart().readings_with_rules`
answers it — so the C boundary and Rust read one type. The result is kept out
of the `Document`, which would otherwise make the serial crate depend on the
rules kernel and change the document schema three gates hold.

## 4. The result

Section `rules` of the charts blob: canonical JSON, an array with one entry
per chart in the order asked for.

```json
{
  "present": [ { "key": "…", "result": { "participants": [ … ], "houses": [ … ], "outcomes": [ … ], "status": "active" } } ],
  "houses": [ … ],
  "longevity": { "threePairs": { … }, "ayurdaya": { … }, "marakas": { … } }
}
```

- **Only present results are carried.** Measured through the SDK over the
  corpus's 53 readable charts, the text-written and generated sets together,
  895 rules, answer 3 145 times: about 59 a chart, one rule in fifteen. An
  absent result says nothing a missing key does not. A consumer wanting "is X
  present" looks for its key.
- **Every result carries its rule's key**, and nothing else of the rule: the
  rule is the consumer's own, or a shipped one it can read by key.
- **The longevity readings carry their presentation.** A maraka result is
  `Presentation::Vulnerability` in the bytes as in the type, so a binding
  rendering one cannot lose it.

## 5. Errors and degenerate states

- A rule that does not read strictly is `INVALID_ARG`, the field named from
  the root as `theme_json` names its own: `rules_json.rules[3].conditions[0].type`.
- An unknown shipped set is `INVALID_ARG` naming the ones there are.
- **A chart that cannot be read for a rule's inputs is not refused for the
  rules alone.** Where a rule set names a point and the birth has no sunrise,
  the chart is read without points, every rule naming one answers false as
  the kernel already defines, and the entry carries `"unreadable": ["points"]`
  so the consumer knows why. A reading the caller asked for itself is refused
  as it is today.

## 6. Cost

Evaluating 823 rules over a chart took 0.16 ms in release when it was last
measured (2026-09-16); the 997 shipped now are measured again when step 1 is
built. A house reading shares the evaluations. The cost
a consumer controls is the sections the rules make the reading compute —
Shadbala above all — which is why they are derived rather than always asked.
The JSON is written once per chart.

## 7. What this design does not settle

- **A consumer's rule pack registered on a context by key**, as dashas are
  registered, so a request names it rather than carrying it. Worth it when a
  consumer sends one pack with every request; not before.
- **A typed result in each binding.** The first step returns the JSON parsed
  into each language's native value, as `drawings` does; a generated type
  over the document schema is the second, when the result's shape has stood
  still for a release.

## 8. Order of work

1. `RuleRequest` in the façade with `ChartRequest::with_rules`, and the
   `rules` field of the document, tested in Rust against `RuleInputs`
   directly — the same answers by two roads.
2. `rules_json` on `TsChartRequest` and section `rules` in the blob, with the
   IDL, the header and `check-ffi`. **Built**: section 33; a held rule and a
   present rule are written by key through one helper, `teistro_rules::key_of`,
   so a house reading does not repeat whole rules; an ABI test founds two charts
   with `{"shipped": ["nabhasas"], "longevity": true}`, reads each chart's
   present rules and Pindayu from the bytes, finds the section empty without
   rules, and a rule that does not read refused as `rules_json.rules[0]`.
3. Node, Dart and Python: the option and the parsed section, each binding's
   own test, and the parity runners printing each chart's present keys, so
   `check-parity` holds the four to one answer. **Built**: Node's `rules`
   option and `chart.rules` (typed `RuleRequest` and `RulesReading`, frozen to
   the leaves), Python's `rules=` argument and `chart.rules` (strict
   `TypedDict`s), Dart's `RuleRequest` with `ShippedRules` and `RuleReadings`
   and `chart.rules`; each binding's test answers the Nabhasa set, finds a
   consumer's rule naming a shipped one by key present, and sees a malformed
   rule refused as `rules_json.rules[0]`. The four parity runners ask for the
   Nabhasa set with longevity and print each chart's present keys and its
   Pindayu; **all four agree on every one of 6 758 values** (the Rust runner
   reading with `readings_with_rules`, as the boundary does).
4. `check-areas` and the surface pages, and `surface-areas.md` §9 closed.
   **Done**: the rules ride on the `chart` area's `found`, so no area is added
   and the measured pages hold as they were.
