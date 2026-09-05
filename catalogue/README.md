# The catalogue

The hand-maintained truth of every key, id and attribute the SDK
speaks: one YAML file per kind, generated into `crates/core` by
`cargo xtask gen catalogue` and held to its sources by
`cargo xtask check-catalogue`. The design is
`docs/03-design/core-types-and-catalogue.md`; the rules below are the
ones the generator enforces.

## A file

```yaml
kind: graha            # lowercase snake; the Rust type is its PascalCase (`Graha`)
number: 1              # the kind's number at the C boundary; unique, never reused
version: 1             # bumps when the file's members or attributes change
doc: "..."             # the Rust enum's documentation
types:                 # composite value types used by this file's attributes
  SignDegree: { sign: key:rashi, degree: u8 }
attributes:            # the attribute schema, name to type, in the order the struct has
  exaltation: "option<SignDegree>"
  own: list<key:rashi>
members:
  - key: SUN           # [A-Z][A-Z0-9_]{0,47}, unique in the kind
    id: 0              # dense from 0 in file order, stable forever
    doc: "The Sun"
    glyph: "☉"         # optional
    aliases: [SOL]     # optional: former keys that still resolve, flagged deprecated
    deprecated: false  # optional
    attributes: { ... }   # one value per schema entry; `~` for an absent option
    sources:           # at least one
      - { text: BPHS, ref: "3" }
    mark: V            # V verified, T traditional (awaiting a citation), S shape only
    unverified: [own]  # optional: attributes whose values are T inside a V row
```

## Types

`u8`, `u16`, `i32`, `f64`, `bool`, `str`, `key:<kind>` (a member of
another kind, or of this one), `option<T>`, `list<T>` (any length),
`array<T,N>` (exactly N), and a composite name declared under `types`
in the same file. A file without `attributes` generates a plain enum.

## Rules

1. A fact the texts agree on is an attribute; a value the schools
   dispute is a row in the kernel that uses it; a presentation or
   remedy fact is pack or module data. The design page has the three
   rules with examples.
2. Ids are dense from 0 in file order and never change; a member is
   never removed, only marked `deprecated`; a renamed member keeps its
   id and lists its former key under `aliases`.
3. Every member cites at least one source and carries a mark. The
   baseline engine's data is rank 2 (`text: baseline-engine`); a text
   citation upgrades a row to rank 1.
4. Every `key:` value must name an existing member of the named kind;
   the generator refuses a dangling reference.
5. Generated files are never edited: `crates/core/src/catalogue/generated/`
   and `catalogue/catalogue.json` are outputs, and the gate fails when
   they do not match the sources.

## Kinds

Numbers 1 to 40 are the kinds the design page lists in that order;
41 and above are the small value sets their attributes use (body
class, parity, rising type, the avastha families and the like). Kind
32, `rule`, has no file: its members come from rule packs at runtime.
