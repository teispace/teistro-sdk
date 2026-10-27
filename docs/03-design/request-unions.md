# A request field that is a name or one of several records

Status: `design`, 2026-10-27. Extends `generated-requests.md`.

Two request fields still cross every binding as a raw JSON value
(`RAW_REQUEST_PARTS` in `xtask/src/ffi.rs`), though the SDK reads each into
types it already describes:

| field | what the reader takes (`crates/sdk/src`) |
|---|---|
| `FestivalRequest.rules` | a pack's name, or a list whose items are each a pack's name or one of six rules, told apart by a key: `windows` a `SaitRule`, `vedha` an `EkadashiRule`, `origin` an `Anniversary`, `after` a `FollowingRule`, any of `civilDate`, `civilWeekday`, `sunEnters`, `easter` a `DatedRule`, and anything else a `FestivalRule` (`festival_request.rs`, `from_json`) |
| `MuhurtaRequest.rules` | an activity's name (`Activity`), or an `ActivityRules` spelt out (`muhurta_request.rs`) |

Each is raw because the reader takes it in a second step, so that a refusal
names the item that was wrong (`festivals.rules[2]`) rather than saying no
shape of an untagged union matched. That is a reason to *read* it in two
steps, not a reason to leave it untyped: Java's `FestivalRequest` takes
`Object rules`, Python types it `Any`, and Node and Dart each keep a
hand-written façade (`PYTHON_FACADES`, `DART_FACADES`) whose only reason
is this field. A spelt-out festival rule is written as a map in Java and
Python, and checked by nobody until the SDK refuses it.

## The shape

The IDL's or-named shape (`RecordShape::OrNamed`) is a set of names or one
record, which `asta` already uses and every generator renders. It becomes
a set of names or **one of several records**, each with the keys that pick
it:

```text
OrNamed { names, member, values: [NamedValue { ty, when: [key, ...] }] }
```

`when` is empty for the last value, the fallback, and for an or-named
field of one record (`asta`), whose rendering does not change. A record's
`when` is the keys of which any one present picks it, in the order the
reader tries them.

### One list of keys, read twice

The keys are declared once, in Rust, beside the reader, as a table of
`(keys, schema)` in reader order; `from_json` matches an item against the
same table it hands the schema. The schema carries them on each option of
the `anyOf` as `x-teistro-when`, and the IDL reader reads them into
`when`. A key the reader matches that the schema does not carry, or the
reverse, cannot happen: there is one table.

### What each binding writes

Requests are written by a binding and read only by the SDK, so a binding
needs to *build* a union and, where it decodes JSON into its own request
types (Java's `of`, Dart's deep test), to pick the variant as the reader
does.

| binding | a union of names and records |
|---|---|
| Node | the names' literals `\|` each record's interface |
| Python | `Union[Literal[names], RecordA, RecordB, ...]` |
| Dart | a `sealed class` the names' enum and one wrapper class per record extend (`FestivalRuleItemSaitRule(rule)`), as `asta`'s single value is wrapped today; a record class is shared by every union holding it, so it cannot implement each |
| Java | a `sealed interface` its `Key` enum and one wrapper record per variant implement, with a `value(...)` overload per record; `of(raw)` picks the first variant whose `when` key is present, the fallback otherwise |

Festival rules are a name **or a list** of items, each an or-named union.
So `FestivalRequest.rules` is `FestivalRules` (the pack names, or a list
of `FestivalRuleItem`), and `FestivalRuleItem` is the or-named union of
the pack names and the six rules.

## Muhurta's activities

`Activity` is named `MuhurtaActivity` in the schema, as every binding
already names it by hand. `MuhurtaRequest.rules` becomes the or-named
union of its keys and `ActivityRules`. `ActivityRules` brings its records
into every binding's generated requests; where one is also an answer
record (`MuhurtaBar`, `MuhurtaUnjudged`), the existing rules for a shared
name apply (`SHARED_NAMES`, `PY_TAKEN_BACK`, `DART_SHADOWED`). A generic
record (`Graded<Rashi>`) is named by `schemars(rename)` before it ships,
since schemars's own `Graded_for_Rashi` is not a name a binding should
carry.

## What goes

- `FestivalRequest.rules` and `MuhurtaRequest.rules` from
  `RAW_REQUEST_PARTS`.
- The façades whose only reason was a raw `rules`: Python's
  `FestivalRequest` and `MuhurtaRequest`, Dart's, and Node's hand
  `MuhurtaActivity` and `ActivityRules`, each deleted only once the
  generated type is shown to accept what it accepted (the equivalence step
  of `generated-requests.md`).

## Gates

- The IDL reader's tests: an or-named union of several records with its
  `when` keys, and a refusal of an option of several records without them
  (two records the reader cannot tell apart).
- The deep examples write one item of every variant (`examples.rs`), so
  each binding's deep test builds and reads back every rule kind.
- A Rust test holds `from_json` to the table: an item carrying a variant's
  key is read as that variant.

## Order

1. Festival rules: the table, the schema, the IDL shape, every emitter,
   the deep examples; the bindings' façades last.
2. Muhurta's rules: the activity rename, the generic names, the shared
   answer names, then the façades.
