# Generated request types for Node, Python and Dart

Status: Node built 2026-10-27; Python and Dart to follow. Java already generates every request record
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
| Python | `bindings/python/teistro/_requests.py` | an object a `TypedDict(total=False)` with `Required[...]` for a required field; a key set a `Literal[...]`, or `Union[TheEnum, Literal[...]]` for a boundary enum; a tagged union a `Union` of `TypedDict`s; a catalogue member the catalogue's enum or its key |
| Dart | `bindings/dart/lib/src/requests.dart` | an object a `final class` with a `const` constructor, defaults where the schema states them, and the `_record` it writes; a key set an `enum` with its `key`; a tagged union a `sealed class` of records; a catalogue member the catalogue's enum |

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

