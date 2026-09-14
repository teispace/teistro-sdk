# The document schema: where it comes from and what holds it

Status: `built`, written and built 2026-09-14. This is step 5 of
[`chart-reading.md`](chart-reading.md) §7 and settles the question that
[`serial-and-the-envelope.md`](serial-and-the-envelope.md) §8 left open.
[`schema-measured.md`](schema-measured.md) is the pass it answers.

## 1. Purpose and scope

A chart document is published as canonical JSON and stored by the
people who publish it. A consumer who stores one wants to know, before
reading it back, whether it has the right shape. They also want that
answer in whatever language holds the file. A JSON Schema gives both,
and it is the one surface of the document that a non-Rust program can
check against.

This page decides three things:

- where the schema's claims come from;
- what the published file is and how it is versioned;
- the gate that proves the schema and the document agree.

Out of scope:

- a schema for the binding blobs, which have their own layout
  description (`ffi-abi-and-api-description.md` §3.4);
- a schema for the settings patch, which is refused by name at the
  boundary and is a separate question.

## 2. What the pass decided, and what it did not

`schema-measured.md` falsified the sample as a source for two reasons:

- It cannot tell a count from a whole double. 35 of 128 numeric paths
  are ambiguous.
- It cannot give a string field its full member list. A sample proves a
  member exists, never that one does not.

From this the page concluded **the description**. But the description it
means, `idl/api.json`, **does not describe the document at all**. Its
structs are the C boundary's, and none of `Document`, `ChartFoundation`
or `GrahaState` is in it. So the conclusion named the right *kind* of
source, which is what the SDK already knows about its own types, without
naming one that exists. This page names one, as the pass's own
reasoning requires.

## 3. The source, weighed

What a schema has to know is decided by **serde**, because serde writes
the document: every `rename_all`, `tag`, `transparent`,
`skip_serializing_if`, `default` and `deny_unknown_fields`. Counted over
the twelve crates the document may reach:

| attribute | uses |
|---|---|
| `rename_all` | 46 |
| `tag` (internally tagged enums) | 16 |
| `skip_serializing_if` | 6 |
| `transparent` | 6 |
| `deny_unknown_fields` / `default` | 9 |
| hand-written `Serialize` | 5 core types, and the 60 generated catalogue enums |

Three sources were weighed:

| source | what it would take | verdict |
|---|---|---|
| extend the API description's extractor to read the layer's Rust types | a second implementation of serde's attribute semantics, including internally tagged enums, kept equal to serde by hand | **no**: the failure the pass warned against, one step removed |
| trace the types at run time through serde (`serde-reflection`) | nothing per type | **no**: it does not support internally tagged enums, which 16 types are |
| **derive `schemars::JsonSchema`** | one derive per type, behind a feature | **chosen** |

The deciding fact is that `schemars_derive` parses attributes with
`serde_derive_internals`, the same parser `serde_derive` uses. **It
cannot disagree with serde about an attribute**, which is exactly the
property the description has with the C header: one reading of one
source. A scratch spike confirmed each form the layer uses before this
page was written:

- a tagged enum becomes a `oneOf` whose tag is a `const`;
- `transparent` takes its inner type;
- `deny_unknown_fields` becomes `additionalProperties: false`;
- a `u8` carries its bounds and an `f64` is a `number`, which accepts
  the integer the canonical form writes for a whole double.

A derive's documented blind spot is a type that serialises itself by
hand. The SDK has exactly two families of those, and neither is a guess:

- **The catalogue enums** are generated, and their key lists come from
  the catalogue, which is where the pass said member lists live. The
  generator (`cargo xtask gen catalogue`) emits each enum's
  `JsonSchema` beside its `Serialize`, from the same key list, so the
  schema's `enum` and the reader's `from_key` are one list. This is
  "only the description knows the full list" made literal.
- **Five core types** (`Nas`, `Hash`, `Error`, `KeyId`, `Ratio`) get a
  hand-written `JsonSchema` next to their hand-written `Serialize`.
  Section 5's gate is what holds each pair together.

## 4. What is published

- **One file**, `crates/serial/schema/document.schema.json`, draft
  2020-12, describing a
  `Sealed<Document>`: the value and its provenance, which is what is
  stored. Every `$defs` entry carries the Rust doc comment as its
  `description`, so an editor that reads the schema shows the SDK's own
  documentation.
- **Inside the crate that embeds it**, and not in `idl/` beside the
  other surfaces, which is where this page first put it: a published
  crate cannot `include_str!` a file from outside itself, and building
  the constant is what found that out.
- **Generated and checked in** by `cargo xtask document-schema`, and
  held by `check-document-schema`, which regenerates it and fails on a
  difference. This is the pattern of every other generated surface. A
  reviewer sees a document's shape change in the diff of the pull
  request that changed it.
- **`$id`** is `urn:teistro:schema:sealed-document:<sdk version>`. A URN
  is used because the project has no published schema host, and an
  `https` identifier that resolves to nothing is worse than an honest
  name. The SDK version is in it because the shape is a property of a
  release.
- **In Rust**, `teistro_serial::schema::DOCUMENT` is the file's text
  (and `DOCUMENT_ID` its identifier), re-exported by the façade. It is a
  constant, so reaching it costs no feature and no dependency; only
  generating it does.
- **Every `$defs` name says what it is.** The generator numbers two
  types that share a name (`Span2` … `Span6`, `Placement2`), a name that
  says nothing and changes whenever a type is added. Building found
  seven such collisions; each colliding type now carries its own name
  (`TithiSpan`, `VargaPlacement`, `PanchangaMuhurtaYoga`, …), and the
  generator **refuses** a numbered name rather than writing one.
- **A description is the doc comment's first paragraph.** The whole
  comment carried doctest fences and rustdoc links into the schema;
  the summary is what an editor shows, and the rest belongs to the
  Rust documentation.

**Optional is optional in the schema.** A field serde always writes but
reads back as `None` when absent (an `Option` without
`skip_serializing_if`) is not `required`. The schema describes what the
reader accepts, not only what this writer happens to produce. A stored
document is validated to learn whether it will read back, so a stricter
schema would refuse documents the SDK reads.

**Unknown top-level keys are allowed**, for the same reason: `Document`
does not deny unknown fields, and the schema does not pretend it does.

## 5. The gate

The pass's own condition was "every sample validates and reads back
equal". That test already exists for the round trip
(`crates/serial/tests/document.rs`); it gains the schema:

1. **Every sample validates**: the three built documents, as sealed and
   written, against the checked-in file, using `boon` (a draft 2020-12
   validator with no network or runtime dependencies; a dev-dependency
   only).
2. **The schema is never stricter than the reader**, one case per kind
   of claim, each made to fire. The direction matters: some refusals are
   semantic (a D9 that says it divides a sign eleven times, a classifier
   this build does not sort by), and no schema can express those. So a
   schema refusal must also be a reader refusal, but not the other way
   round:
   - a catalogue member that is not one (`"MARZ"`);
   - a count out of range;
   - a wrong type;
   - an unknown field inside a `deny_unknown_fields` struct;
   - a tagged enum with an unknown tag;
   - a missing required field.

   Each case is also fed to the reader, which must refuse it too. A
   schema claim the reader does not share would falsify §4's rule.
   The converse is also held, once: a document with a key from a newer
   build is valid for both.
3. **The generated file is current** (`check-document-schema`).
4. **The feature builds**: the `schema` feature is part of the gated
   configurations, because an unbuilt configuration is broken.

## 6. The feature

`schemars` is an optional dependency of each crate the document reaches,
behind a feature named `schema` that turns on the same feature in that
crate's dependencies. The derive is
`#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]`, so a
consumer who does not generate schemas compiles no proc-macro for it.
It is on in the fifteen crates the document reaches (§7, step 1).

## 7. Order of work

1. The feature and the derives across the crates. Built as **every type
   that derives `Serialize` in those crates also derives `JsonSchema`**
   (210 derives), rather than only the ones the compiler asked for: that
   rule is one a lint can hold for every future type, and a field added
   to the document can then never break the emitter.
2. The catalogue generator's `JsonSchema`, and the five hand-written
   ones.
3. The emitter and `check-document-schema`; `teistro::schema::DOCUMENT`.
4. The gate's tests (§5).
5. Close `serial-and-the-envelope.md` §8 and `chart-reading.md` step 5.

## 8. What this design does not settle

- **Publishing the schema at a URL**, which waits for a schema host.
  The `$id` changes then, and only then.
- **The bindings' own copy.** No binding hands a consumer the JSON
  document today; each decodes the blob. When one does, it ships this
  file rather than a translation of it.
