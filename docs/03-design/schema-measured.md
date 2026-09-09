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
provider, 238 distinct paths between them. A recorded sample would go
stale the first time a section gained a field and the pass would not
notice.

The question the pass exists to answer is **where the schema should come
from** — the documents, the Rust types, or the API description.
Sections 3 and 4 decide it.

## 2. The documents

| sample | what it holds | sections | paths |
|---|---|---|---|
| `whole` | every section the layer can produce | 7 | 238 |
| `day` | a foundation and the almanac of its day | 2 | 156 |
| `bare` | a foundation alone, the smallest document there is | 1 | 72 |

Across all three, by the type a schema would give the value:
7 boolean, 38 integer, 6 null, 93 number, 99 string.

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
| 99 | 94 | 5 |

A schema would constrain each of those 94 with an `enum`, and it cannot
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
| 72 | 166 | 7 |

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

## 7. The layer reads back, and five types cannot

A schema's most valuable consumer is the SDK itself: a stored document
is worth validating precisely because something will try to read it
later. Counting the derives over the layer's own source:

| crate | types that serialise | types that read back |
|---|---|---|
| `serial` | 2 | 2 |
| `chart` | 12 | 10 |
| `panchanga` | 14 | 14 |
| `vargas` | 11 | 8 |
| `state` | 10 | 10 |
| `aspect` | 8 | 8 |
| `points` | 3 | 3 |
| `houses` | 5 | 5 |
| **total** | **65** | **60** |

The five that do not derive it are the five that **cannot**, and they
are all one shape: a value whose identity is a shipped constant, holding
a `&'static` reference no document can produce — a divisional scheme's
group table, its `Map::Listed` of signs, an aspect angle's key, the
drishti table a chart was read under. Each has a reader written by hand
instead, and each reads the value back **by its identity**: the key or
the catalogued name is looked up in this build's own table, and what the
document says about that table is checked against it rather than
trusted. A document that names D9 and describes something else is
refused by name.

That is stricter than a derive would have been, and it is the property a
stored chart wants: reading one under a build whose tables have moved is
an error rather than a quiet reinterpretation.

`serial-measured.md` found the opposite end of this shape —
`ChartFoundation` derived no `Serialize`, so the SDK could not publish a
chart at all. It can now publish one and read it back.

## 8. Two spellings, in one document

A schema's `enum` has to spell a member the way the document really
writes it. Counting `rename_all` over the layer and the crates it holds
values from, 36 types declare one:

| convention | types |
|---|---|
| `SCREAMING_SNAKE_CASE` | 33 |
| `lowercase` | 3 |

The minority is not unreached: `CalendarResolution` is one of them, and
it appears in every document there is — a chart's foundation carries
the resolution of its own date. So a consumer reading one document meets
both conventions, and a generated schema must take the spelling from
each type rather than assume the majority's.

## 9. The grammar is a fixed point, and the parser had to be told

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

| sample | largest number | decimals resolved there | numbers | a correct parser moves | this build's parser moves |
|---|---|---|---|---|---|
| `whole` | 2460506 | 10 | 938 | 0 | 0 |
| `day` | 2460506 | 10 | 409 | 0 | 0 |
| `bare` | 2460483 | 10 | 171 | 0 | 0 |

Both of the last two columns are nought, and the second only because the
build asks for it. `serde_json`'s **default** float parser is a fast
path that is not correctly rounded: it reads `218.91170673806658` as the
double one unit in the last place below, and did that to 84 of these
1518 numbers — about one in fifteen — until `teistro-core` turned on
its `float_roundtrip` feature. A reader on the fast path cannot
reproduce the hash it exists to check, however correct the grammar is.

`arbitrary_precision` fixes the same numbers and is the wrong tool:
serde buffers an internally tagged enum before writing it, and that
buffer writes a number as `{"$serde_json::private::Number": ...}`, which
breaks `DeltaTModel`, `CalendarResolution`, `Outcome` and every other
`#[serde(tag = ...)]` the SDK has. It also leaves `from_str` wrong, so a
reader would have had to go through a `Value`. `float_roundtrip` has
neither cost.

Turning it on made three of the corpus's own comparisons exact that had
not been: `points`'s clock-driven lagnas went from 138 to 141 of 213,
because the fixtures are JSON too and had been read a unit in the last
place low. Nothing the SDK computes moved.

`serial-measured.md` asserted the fixed point and found it held, over
the corpus's recorded documents — whose numbers are longitudes and
speeds, all under 360. A chart document carries the instant it was cast
for, and that is where a fixed count of decimals ran out.

## 10. What this decides

| proposed rule | verdict | measured |
|---|---|---|
| the schema can be derived from the documents | falsified | 35 of 128 numeric paths are ambiguous |
| a sample gives a string field its full member list | falsified | a sample proves a member exists, never that one does not |
| the layer's types read back, so a round trip can gate the schema | **holds** | 60 types derive `Deserialize` |
| one casing convention covers every enum in a document | falsified | 2 conventions declared |
| every number the form writes reads back as the same double | **holds** | 0 of 1518 move under a correct parser |
| this build's parser reproduces a stored document's hash | **holds** | it moves 0 of 1518 |

The measurement falsifies 3 of the 6 proposed rules. Those three say the
same thing about **where** a schema comes from: the description, beside
the other four surfaces, and not a sample nor a derive macro over the
Rust types. A sample cannot tell a count from a whole double, cannot
give a string field its member list, and does not know that one enum in
a document is spelled in a different case from the rest. The description
knows all three, and it is the only place that cannot disagree with what
the bindings already say.

The other three held once the work this page asked for was done. The
layer reads back, every number the form writes reads back as the same
double, and this build's parser reproduces a stored document's hash —
so the schema's natural gate, *every sample validates and reads back
equal*, is written and passing (`crates/serial/tests/document.rs`).

What is left is the emitter.

