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
7 boolean, 38 integer, 6 null, 91 number, 98 string.

## 3. A whole double is written as an integer

The canonical form removes a trailing zero on purpose —
`to_hash_form(&2.0)` is `"2"` — because the hash has to be stable and
`2` and `2.0` are the same number. The consequence is that a field the
SDK holds as an `f64` appears in the document as a JSON **integer**
whenever its value is whole, and is then indistinguishable from a field
that really is a count.

| numeric paths | integer in every sample | decimal somewhere | both, across samples |
|---|---|---|---|
| 128 | 37 | 90 | 1 |

The ambiguity is not theoretical. 1 path is written both ways within the
same sample set:

- `.foundation.grahas[].latitude_deg`

So a schema derived from the documents alone would type 37 paths on the
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

## 9. The form is not a fixed point where the numbers are large

The content hash rests on one invariant: a consumer that reads a stored
document and hashes it again gets the producer's hash. The grammar
writes every number to 12 decimals, which is inside an `f64`'s
resolution for a longitude and outside it for a Julian day — four
orders of magnitude larger, where one unit in the last place is already
about 5e-10.

| sample | largest number | decimals resolved there | numbers written past it | writing it twice |
|---|---|---|---|---|
| `whole` | 2460506 | 10 | 213 | **falsified** |
| `day` | 2460506 | 10 | 213 | **falsified** |
| `bare` | 2460483 | 10 | 6 | holds |

The three digits past the resolution are the decimal expansion of a
binary value, not information, and they do not survive a parse:
`2460483.108666389249` is written, read, and written again as
`2460483.108666389715`.

Whether a given value survives is a coin toss, which is why the smallest
sample holding does not make it safe: a foundation alone carries a
handful of instants and happens to win every toss, and a document with
an almanac in it carries two hundred and loses. The finding is not that
a large document fails but that any document may, and one that does is
one whose stored hash a reader cannot reproduce.

`serial-measured.md` asserted this invariant and found it held, over the
corpus's recorded documents — whose numbers are longitudes and speeds,
all under 360. A chart document carries the instant it was cast for, and
that is where the grammar runs out.

The fix is a decision rather than a patch, because it moves the hash of
every document: a shortest-round-trip decimal, which Rust and JavaScript
already agree on, rendered without an exponent as this grammar already
renders one. It belongs to the design page.

## 10. What this decides

| proposed rule | verdict | measured |
|---|---|---|
| the schema can be derived from the documents | falsified | 37 of 128 numeric paths are ambiguous |
| a sample gives a string field its full member list | falsified | a sample proves a member exists, never that one does not |
| the layer's types read back, so a round trip can gate the schema | falsified | 0 types derive `Deserialize` |
| one casing convention covers every enum in a document | falsified | 2 conventions declared |
| the canonical form of a document is a fixed point | falsified | 2 of 3 samples move when written twice |

The measurement falsifies 5 of the 5 proposed rules. The first four say
the same thing about **where** a schema comes from: the description,
beside the other four surfaces, and not a sample nor a derive macro over
the Rust types. The description is the only place that has the member
lists, and the only place that cannot disagree with what the bindings
already say.

The fifth says something about **when**. A schema describes a document a
consumer will store and read back, and two of the three samples do not
survive being read back and written again, so the bytes a schema would
describe are not yet stable. The round trip is the other half of that:
until the layer's values derive `Deserialize`, the schema's natural gate
— every sample validates and reads back equal — cannot be written at
all.

Both are the design page's questions rather than this pass's, and both
come before an emitter.

