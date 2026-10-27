# Generated request types for Node, Python and Dart

Status: Node, Python and Dart built 2026-10-27. Java already generates every request record
(`java-binding.md` §4.1, `crates/idl/src/emit/java/requests.rs`). Node,
Python and Dart still type their requests by hand: of the request records
the façade's schemas describe, `index.d.ts` hand-types about 100,
`teistro/__init__.py` about 72 and `teistro.dart` about 186. Each is a
copy of a Rust type kept in step by review, and the copies have drifted in
strictness: Python types a nested part as `Mapping[str, Any]` where Node
and Java type it field by field.

The maintainer's rule (2026-10-27): generate, do not hand-write, what the
description already holds.

## What is generated

One file per binding, rendered by `cargo xtask gen ffi` from
`api.requests` (built in `xtask/src/ffi.rs` from the façade's own schema
of each request, so a request is typed as serde reads it) and checked by
`check-ffi` like every generated file:

| Binding | File | Shapes |
|---|---|---|
| Node | `bindings/node/lib/requests.d.ts` | an object is an `interface` of `readonly` fields, optional where serde defaults it and `\| null` where it is an `Option`; a closed key set a literal union, or the boundary enum's type where `REUSED_ENUMS` says it is that enum (its bare keys, `Bare<catalogue.Sex>`, where the enum is a catalogue kind); a tagged union a union of interfaces with the tag's literal; external tagging the bare keys' literals or a one-key object per data variant; a catalogue member `Member<catalogue.Graha>`, its full or bare key; a registered key that or any string; a Rust array of at most seven scalars a tuple of its length, a longer one or one of records an array whose doc gives the length |
| Python | `bindings/python/teistro/_requests.py` | an object a `TypedDict(total=False)` with `Required[...]` for a required field and `Optional[...]` where it takes null, in the call form where a key is a keyword (`from`); a key set a `Literal[...]`, or `Union[TheEnum, Literal[...]]` for a boundary enum (the keys alone for a catalogue kind's); a tagged union a `Union` of `TypedDict`s; external tagging the bare keys' `Literal` and a private one-key `TypedDict` per variant with data; a catalogue member `Union[TheEnum, str]`; a record the layer also answers with (`PY_TAKEN_BACK`: `NatalPoint`, `AlmutenRules`, …) that record or the layer's answer class |
| Dart | `bindings/dart/lib/src/requests.dart`, a `part` of `teistro.dart` | an object a `final class` with a `const` constructor: `required` where serde requires the field, the schema's default where Dart can write it as a constant (an enum member, a number, a list, a record built of those), and nullable and left off the wire otherwise, so serde gives its own; a key set an `enum` with its `key`, implementing the layer's `_Keyed` so its decoders read it; a tagged, external or or-named union a `sealed class` of one `final class` per variant; a catalogue member the catalogue's enum, written as its key, and a registered key `KeyOf<T>`, written in full. Every record writes its JSON as `_wire` and compares field by field (`_Value`) |

Docs come from the schema's descriptions, which are the Rust doc comments
(`a-doc-comment-has-generated-readers`), so a request documents itself
once, in Rust.

What stays by hand: the layers' methods, the loader, presets that name a
combination (`KootaRules::PRINTED_IN_NEPAL` stays a Rust constant and a
documented spelling), and `RAW_REQUEST_PARTS`, which stay JSON values in
every binding as in Java.

## How it lands without breaking a consumer

Each binding moves on its own branch, Node first (declarations only, the
runtime passes the object through), then Python (declarations, the runtime
passes the dict through), then Dart (runtime encoders).

1. Render the generated file beside the hand-written layer.
2. **Prove equivalence before deleting.** A generated check file asserts,
   for every name both define, that each is assignable to the other
   (TypeScript `Equals<A, B>`, mypy `assert_type` round trips, a Dart
   test writing both records). Every mismatch is a finding: either the
   hand copy drifted (fix it to the schema) or the schema is wrong (fix
   the Rust type). The findings are listed in the branch's PR.
3. Re-export the generated names from the ergonomic entry point
   (`index.d.ts`, `teistro/__init__.py`, `teistro.dart`) and delete each
   hand-written twin. A consumer's import does not change.
4. A type the hand layer had and the schema does not is either a façade
   type (kept by hand) or a dead name (removed, with a CHANGELOG line).

Python's types get stricter where the hand copy was `Mapping[str, Any]`;
that is a typing change only (the runtime is unchanged) and is listed in
the CHANGELOG.

## Gates

- `check-ffi` fails on any difference in a generated file.
- `check-node`'s `typecheck/layer.ts`, `check-python`'s strict mypy and
  `typecheck/wrong.py`, and `check-dart`'s analyser read the generated
  files through the existing positive and negative usages.
- A lint (`xtask/src/lints.rs`) refuses a hand-written declaration whose
  name is a generated request record, so a twin cannot come back.

## What generating Node found

The hundred request records `index.d.ts` typed by hand were held against
the generated ones, each assignable to the other, before a twin was
deleted. Every difference was a finding, and they ran both ways.

- **The hand copies had drifted.** Several catalogue fields took a
  member's full key only, where serde reads the bare one too; rule sets
  required fields serde defaults (`AccidentalRules.combustionDeg`,
  `KujaLifts.benefic`); `SubDegree` lacked its `NONE`; Python's copies are
  looser still.
- **A schema stated less than its reader.** `PointAsked`, the natal point a
  hit list or a progression asks for, took `"LAGNA"`, `"MIDHEAVEN"` and a
  graha's bare key in its hand-written `Deserialize`, while its schema was
  `NatalPoint`'s, objects only. The hand copy had it right; the schema now
  says so (an `OrNamed` gains `member`, a catalogue kind beside the names
  and the value), and Java's `PointAsked` gains `LAGNA`, `MIDHEAVEN` and
  `member(Graha)`.
- **The reader lost two facts.** A Rust array's length (`[f64; 7]`) and
  whether a field takes null or only absence. Both are now read (`List.len`,
  `RecordField.null`).

Choices made in the light of them:

- A member takes the type an answer gives it, `'unknown'` included, so a
  member read from an answer is asked back without a cast
  (`a-request-takes-what-an-answer-gives`); the SDK refuses `unknown` by
  field.
- An optional field is `?: T`, not `?: T | undefined`: the layer's answers
  echo their rules as `Required<Rules>`, which keeps an explicit
  `undefined`.
- A tuple only where a person writes the items out. The twelve-sign terms
  table built with `Array.from` is an array, its length in its doc; the
  five Sarvatobhadra motions are a tuple, so four are refused at compile
  time.

Eleven records stay hand-written, because the layer takes them in a shape
of its own and writes the wire's (`REQUEST_FACADES` in `xtask/src/lints.rs`
names each with its reason): a `place` of `{ latitude, longitude }` where
the wire has `latitudeDeg`, calendar dates where it has day numbers, typed
arrays of instants, and the two requests whose `rules` the schema leaves
raw. Six generated names are also an older export's (`Partile`, `Siege`
and `Terms` are catalogue enums, `Confidence` an answer record, `CivilTime`
and `ZoneSpec` boundary types); `index.d.ts` exports the older under the
name, and the request's own is reached through the field that takes it.

## What generating Python found

The same comparison ran through strict mypy over the layer, its tests,
examples, `typecheck/wrong.py` and the parity runner, with every twin
replaced. It found the Python layer's own shape:

- **A member crossed as its id where the layer did not write it.** Python's
  enums are `IntEnum`s, and the shared request writer, used at 54 call
  sites, handed its mapping to `json.dumps` as it was, so a member anywhere but where a
  section's own writer looked crossed as a number. Every request now
  crosses through one writer (`_written`), which writes a member as its key
  at any depth and a `NatalPoint` as an answer spells it; the three
  section-specific writers it replaces are gone.
- **A caller hands back what it was given.** The layer takes its answer
  classes (`AlmutenRules`, `PerfectionRules`, a `NatalPoint`, …) where a
  request takes the record, so the generated type names both
  (`PY_TAKEN_BACK`), and strict mypy reads each against the layer's class.
- **The annual chart's rules are written in snake case.** The layer takes
  `saham_rules: {"add_sign": ...}` and camel-cases it all the way down, so
  the seven rule records under it stay the layer's own; the lint lists them
  with that reason. `AyurdayaRules` is snake case on the wire too and is
  generated.
- `ProgressionContacts` was a copy of the generated `ContactsAsked` under
  its own name; it is now that type by the name the layer gave it.

## What generating Dart found

Dart's layer is a runtime encoder, so its records are classes a caller
builds and the layer writes, and the generated ones are a `part` of
`teistro.dart` to share its value equality. Of the 186 request names it
declared by hand, 137 are generated now: the 102 key sets and 35 records
whose every field had the schema's own type. The analyser held each swap,
and `check-dart` ran the layer's tests over the result.

- **The hand records spoke three encodings.** A section wrote itself as a
  JSON string (`_json`), a nested part as a map (`_record` or a map-typed
  `_json`), and some as both. Every request record, generated or kept,
  now has one JSON value, `_wire`; a section crosses as `_encoded(section,
  name)`.
- **Only one record refused a number JSON cannot hold.** `RemedyRequest`
  threw `ArgumentError` naming `remedies.at` for NaN; every other double
  went to `jsonEncode`, which fails without naming the field. `_encoded`
  now walks every section and refuses a non-finite number by its path, so
  all of them do.
- **`SubDegree` gains `none`**, which Node's generation found missing too.

Forty-nine records stay the layer's own, each listed with its reason in
`DART_FACADES` (`crates/idl/src/emit/requests/dart.rs`), which the
generator leaves out and `check-lints` holds to the layer: named presets
(`AlmutenRules.lilly`, `NumerologyRules.baseline`), positional records the
layer also decodes as answers (`HarmonicRequest(5)`, `BirthDate`,
`Interval`), unions the layer builds through its own constructors
(`NatalPoint`, `OrbModel`), and records whose parts the layer names more
clearly than the Rust types do (`PakshiClock` where the schema has
`Clock`). A copy cannot come back: a hand class named like a generated one
is a duplicate declaration the analyser refuses.

