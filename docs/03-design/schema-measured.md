# The chart document's shape, measured

Status: `generated` by `cargo xtask schema`. Do not edit: `check-schema`
regenerates this page and fails on any difference. The design it is
written for is
[`serial-and-the-envelope.md`](serial-and-the-envelope.md) §8.

## 1. What can be measured about a schema

A JSON Schema is a set of claims about a document: which keys are
required, what type each value has, which strings come from a fixed
list, what may be null. Nothing is recorded that a schema could be
compared against, so this is the `aspect` kind of pass rather than the
`vargas` kind — it measures the document's own shape over real values,
and reads the source for what the values cannot say about themselves.

The sample is built rather than recorded, by `cargo run -p
teistro-serial --example documents`: 3 documents over the analytic test
provider, 237 distinct paths between them. A recorded sample would go
stale the first time a section gained a field and the pass would not
notice.

The question the pass exists to answer is **where the schema should come
from** — the documents, the Rust types, or the API description.
Sections 3 and 4 decide it.

## 2. The documents

| sample | what it holds | sections | paths |
|---|---|---|---|
| `whole` | every section the layer can produce | 7 | 237 |
| `day` | a foundation and the almanac of its day | 2 | 155 |
| `bare` | a foundation alone, the smallest document there is | 1 | 72 |

Across all three, by the type a schema would give the value:
7 boolean, 38 integer, 6 null, 93 number, 98 string.

## 3. A whole double is written as an integer

The canonical form removes a trailing zero on purpose —
`to_hash_form(&2.0)` is `"2"` — because the hash has to be stable and
`2` and `2.0` are the same number. The consequence is that a field the
SDK holds as an `f64` appears in the document as a JSON **integer**
whenever its value is whole, and is then indistinguishable from a field
that really is a count.

| numeric paths | integer in every sample | decimal somewhere | both, across samples |
|---|---|---|---|
| 128 | 35 | 90 | 3 |

The ambiguity is not theoretical. 3 path iss written both ways within
the same sample set:

- `.foundation.grahas[].latitude_deg`
- `.foundation.houses.madhya[]`
- `.foundation.houses.sandhi[]`

So a schema derived from the documents alone would type 35 paths on the
evidence of a sample that cannot tell a count from a round number. Some
of them really are counts — a day of the month, a bhava — and some
are doubles that happened to land on a whole value. Nothing in the JSON
separates them.

**Only the Rust types know.** This is the pass's first answer to where
the schema comes from.

## 4. Almost every string is a catalogue member

| string paths | drawn from the catalogue | free text |
|---|---|---|
| 98 | 93 | 5 |

A schema would constrain each of those 93 with an `enum`, and it cannot
get the members from the documents: the widest of them shows 12 values,
where the catalogue's own list is longer for every one. A sample proves
a member exists; it never proves a member does not.

**Only the description knows the full list.** This is the pass's second
answer, and it points at the same place as the first: the schema is
generated from what the SDK already describes, beside the C header, the
TypeScript surface, the Dart classes and the Python declarations, and
not from a sample.

## 5. What is required, measured

Every section but the foundation carries a `skip_serializing_if`, so a
Rust `Option` and an absent JSON key do not line up one to one and a
schema's `required` cannot be read off the struct. Over the three
samples:

| paths in every sample | paths in some | top-level sections |
|---|---|---|
| 72 | 165 | 7 |

The top-level sections of the widest document are `aspects`,
`foundation`, `houses`, `panchanga`, `points`, `state`, `vargas`. Only
`foundation` is in all three, which is what the module says it intends;
the measurement agrees with the intention here rather than contradicting
it.

## 6. What may be null

6 paths carry a null in the sample, so a schema has to admit one
there and a consumer has to expect it:

- `.panchanga.sun.sankranti`
- `.state[].combustion.from_sun_deg`
- `.state[].combustion.orbs`
- `.state[].combustion.orbs.deep_deg`
- `.state[].deeptadi`
- `.state[].war`

A null here is a real answer and not a missing one — no war, no
sankranti that day, no combustion orbs for a body that cannot be combust
— which is why the field is written rather than skipped.

## 7. Nothing reads back

A schema's most valuable consumer is the SDK itself: a stored document
is worth validating precisely because something will try to read it
later. Counting the derives over the layer's own source:

| crate | types that serialise | types that read back |
|---|---|---|
| `serial` | 2 | 0 |
| `chart` | 12 | 0 |
| `panchanga` | 14 | 0 |
| `vargas` | 11 | 0 |
| `state` | 10 | 0 |
| `aspect` | 8 | 0 |
| `points` | 3 | 0 |
| `houses` | 5 | 0 |
| **total** | **65** | **0** |

This is the same shape the previous pass found and the opposite end of
it. `serial-measured.md` found that `ChartFoundation` derived no
`Serialize`, so the SDK could not publish a chart; that was fixed, and
the SDK can now publish one it cannot read.

It matters for the schema rather than merely being untidy. The gate a
schema wants is *every sample validates, and every sample reads back
equal* — the second half of which cannot be written at all today, so a
schema shipped now would be a claim nothing checks from the inside.

## 8. Two spellings, in one document

A schema's `enum` has to spell a member the way the document really
writes it. Counting `rename_all` over the layer and the crates it holds
values from, 35 types declare one:

| convention | types |
|---|---|
| `SCREAMING_SNAKE_CASE` | 32 |
| `lowercase` | 3 |

The minority is not unreached: `CalendarResolution` is one of them, and
it appears in every document there is — a chart's foundation carries
the resolution of its own date. So a consumer reading one document meets
both conventions, and a generated schema must take the spelling from
each type rather than assume the majority's.

## 9. The grammar is a fixed point, and one parser cannot see it

The content hash rests on one invariant: a consumer that reads a stored
document and hashes it again gets the producer's hash. The grammar used
to write every number to 12 decimals, which is inside an `f64`'s
resolution for a longitude and outside it for a Julian day — four
orders of magnitude larger, where one unit in the last place is already
about 5e-10. The three digits past the resolution were the decimal
expansion of a binary value rather than information, and they did not
survive a parse: `2460483.108666389249` was written, read, and written
again as `2460483.108666389715`.

It now writes the **shortest** decimal that reads back as the same
double, which is a fixed point by construction and still never an
exponent.

| sample | largest number | decimals resolved there | numbers | a correct parser moves | `serde_json` moves |
|---|---|---|---|---|---|
| `whole` | 2460506 | 10 | 938 | 0 | 64 |
| `day` | 2460506 | 10 | 409 | 0 | 13 |
| `bare` | 2460483 | 10 | 171 | 0 | 7 |

The fifth column is the grammar's whole claim, and it is nought
everywhere: measured against a **correctly rounded** parser —
`str::parse`, JavaScript's `JSON.parse`, Python's `json` — every
number the form writes reads back as the very same double.

The sixth is a separate finding, and it lands on the reader this page
says has to be written. **`serde_json`'s own number path is not
correctly rounded**: it reads `218.91170673806658` as the double one
unit in the last place below, and does that to about one number in
fifteen. A Rust consumer reading a Teistro document through
`serde_json::Value` therefore cannot reproduce its hash, however correct
the grammar is. The SDK's own reader has to parse a number with
`str::parse`, or with `serde_json`'s `arbitrary_precision` which defers
to it, rather than with the default number path.

`serial-measured.md` asserted the fixed point and found it held, over
the corpus's recorded documents — whose numbers are longitudes and
speeds, all under 360. A chart document carries the instant it was cast
for, and that is where a fixed count of decimals ran out.

## 10. What this decides

| proposed rule | verdict | measured |
|---|---|---|
| the schema can be derived from the documents | falsified | 35 of 128 numeric paths are ambiguous |
| a sample gives a string field its full member list | falsified | a sample proves a member exists, never that one does not |
| the layer's types read back, so a round trip can gate the schema | falsified | 0 types derive `Deserialize` |
| one casing convention covers every enum in a document | falsified | 2 conventions declared |
| every number the form writes reads back as the same double | **holds** | 0 of 1518 move under a correct parser |
| any JSON parser can reproduce a stored document's hash | falsified | `serde_json` moves 84 of 1518 |

The measurement falsifies 5 of the 6 proposed rules. The first four say
the same thing about **where** a schema comes from: the description,
beside the other four surfaces, and not a sample nor a derive macro over
the Rust types. The description is the only place that has the member
lists, and the only place that cannot disagree with what the bindings
already say.

The last two are about the bytes a schema would be describing, and they
are why an emitter is not the next thing to write. The grammar now
holds: every number the form writes reads back as the same double. But
nothing in the layer derives `Deserialize`, so the schema's natural gate
— every sample validates and reads back equal — still cannot be
written; and when that reader is written it must not take its numbers
from `serde_json`'s default path, which cannot reproduce the hash it is
meant to check.

The reader comes first, then the emitter.

