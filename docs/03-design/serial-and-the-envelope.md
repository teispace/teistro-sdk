# The canonical form, the seal, and the chart document

Status: `designed`, written 2026-09-07 from the falsification pass in
[`serial-measured.md`](serial-measured.md). Derives from
[`core-types-and-catalogue.md`](core-types-and-catalogue.md) (the
envelope and `canonical_json`), ADR-0020 (what a stamp carries),
ADR-0022 (why the form is canonical) and every Phase 4 module whose
value it holds. `02-architecture/01-module-catalog.md` gives the module
its row. Built as `crates/serial`.

## 1. Purpose and scope

One JSON document for a chart, and one way of writing it that two
bindings agree on byte for byte. Three parts:

- the **seal**, which pairs a value with its provenance and computes the
  one field a caller cannot fill by hand;
- the **canonical form**, whose bytes are what the content hash is taken
  over and what a consumer stores;
- the **document**, which holds everything the chart layer computes —
  the foundation, the panchanga day, the divisional charts, the
  planetary state, the aspects, the derived points and the houses —
  under one envelope.

It is not: the binary blob at the C boundary (`ffi`, which has its own
schemas), the dossier text and its presets, the gochar sidecar, or the
chart layout geometry. Those are the module's later rows.

## 2. What the pass found

Three things, and each shapes the module.

**The content hash is the hash of nothing.** `Provenance` carries every
field ADR-0020 asks for, and the field the whole envelope exists for —
the hash of the value — is set to `Hash::of(&[])` by `Provenance::new`
as a placeholder and replaced by exactly one producer of three. A
founded chart and a daily panchanga both go out claiming a hash of the
empty string.

That is a shape problem rather than a bug in a producer: a value and its
stamp are built separately and joined at the end, so the one field that
*cannot* be filled until the value exists is the one everybody forgets.

**The canonical form is canonical.** Keys in code-point order at every
depth over the corpus's 55 documents, the same bytes twice, and the same
bytes however the value's own maps were ordered.

**A number needs a stated format.** Every number in the corpus
round-trips, but 116 of the 193 366 are already written with an
exponent, and the form switches to one **below 10⁻⁶** where
JavaScript's switches below 10⁻⁷:

| value | this form | a JavaScript binding |
|---|---|---|
| 10⁻⁶ | `1e-6` | `0.000001` |

Two bindings that agree about the number disagree about the bytes, and
therefore about the hash. Leaving the format to the JSON layer is not an
option.

## 3. The seal

```rust
pub struct Sealed<T> { /* value, provenance */ }
impl<T: Serialize> Sealed<T> {
    pub fn new(value: T, provenance: Provenance) -> Sealed<T>;   // hashes
    pub fn value(&self) -> &T;
    pub fn provenance(&self) -> &Provenance;
    pub fn content_hash(&self) -> Hash;
    pub fn cache_key(&self) -> (Hash, Hash, u32);
    pub fn to_canonical(&self) -> String;
}
```

`Sealed::new` is the **only** way to make one, and it computes the
content hash from the value it is given. There is no setter and no way
to construct the pair with a stale hash, which is the whole design: the
field that cannot be filled by a caller is filled by the constructor.

`Envelope<T>` stays where it is and keeps its meaning — a value with a
stamp, which a module builds as it goes. `Sealed<T>` is what a value
becomes when it leaves: `Sealed::from_envelope` takes one and seals it.
So a producer does not change, and nothing can be *published* unsealed.

## 4. Two forms, and why they are two

A canonical form has two jobs that pull apart, so the module has two
functions rather than one with a flag.

**The hash form is fixed and settings-independent.** Keys in code-point
order; no whitespace; and every number written in a stated decimal
grammar with no exponent, so that a binding needs only to implement the
grammar rather than to match `ryu`. It cannot honour
`output.precision`, because a hash that moves with a display setting is
a worse cache key — the reader would find two identical answers under
two hashes and recompute.

**The rendered form honours `output.precision`.** An angle to its
`angle_decimals`, an instant to its `instant_decimals`, a score to its
`score_decimals` — which is what that knob is for, and which nothing
had read. It is a rendering and is never hashed.

The settings hash already distinguishes two results computed under
different precision, because `output.precision` is part of the resolved
settings; so nothing is lost by keeping it out of the content hash.

```rust
pub fn to_hash_form<T: Serialize>(value: &T) -> String;
pub fn to_rendered<T: Serialize>(value: &T, precision: &Precision) -> String;
pub fn hash_of<T: Serialize>(value: &T) -> Hash;   // over the hash form
```

### The number grammar

A number is written as a decimal with an optional leading `-`, at least
one digit before the point, and up to **twelve** digits after it with
trailing zeros removed — never an exponent, never a bare `.5`, never
`-0`. Twelve is chosen because a double carries about seventeen
significant decimal digits and the SDK's own quantities are degrees,
days and scores whose magnitudes are under 10⁶: twelve decimals is
finer than a nanoarcsecond and coarser than the noise.

This is a **lossy** form, and deliberately so. Two doubles that differ
below the grammar's resolution hash alike, which is what a caller
wanting "the same answer" means; a caller wanting bit equality has the
double itself.

## 5. The document

```rust
pub struct Document {
    pub foundation: ChartFoundation,
    pub panchanga: Option<Panchanga>,
    pub vargas: Option<VargaChart>,
    pub state: Option<Vec<GrahaState>>,
    pub aspects: Option<Aspects>,
    pub points: Option<Points>,
    pub houses: Option<Houses>,
}
```

Every section but the foundation is optional, because a caller that
wants a panchanga should not pay for a divisional chart. The foundation
is not, because every other section is computed from it and a document
without one cannot be checked against anything.

The whole is sealed once: one envelope for the document rather than one
per section, because the provenance of all of them is the same
provenance and repeating it eleven times would make the JSON larger
than the values.

## 6. Errors

| when | what |
|---|---|
| a value that cannot serialise | `INTERNAL`, naming the section |
| a precision beyond the grammar's twelve decimals | `OUT_OF_RANGE` with the bound |
| a document with no foundation | not representable: the field is not an `Option` |

## 7. Tests

- **The grammar, exhaustively over the shapes that break it**: the
  exponent thresholds either side, negative zero, integers, values at
  the twelve-decimal boundary, and every number of the corpus.
- **The seal**: that a sealed value's hash is its own, that it moves
  when the value moves, and that the pair cannot be built with a stale
  one.
- **Round trip**: every Phase 4 value serialises and reads back equal.
- **Over the corpus**: the hash form's own invariants over the 55
  documents, as the pass measured them, recomputed through this crate.

## 8. Open questions

- **A JSON Schema for the document**, which a consumer would validate
  against and which `idl` already has machinery for. Deferred with the
  dossier and the blob.
- **Whether the producers should seal.** `chart` and `panchanga` return
  an `Envelope`; sealing at the boundary means the hash is right in the
  document but still empty on the envelope a Rust caller holds. Sealing
  in the producers would fix both and is a change to two crates'
  behaviour, which wants its own measurement.
- **A gate for knobs with no reader.** Three have now been found in as
  many modules — `state.combustion_orbs`, `houses.module_overrides`,
  `output.precision` — which is a pattern rather than an accident, and
  `check-lints` is where such a rule would live.
