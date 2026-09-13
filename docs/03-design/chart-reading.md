# The chart reading: the whole document, and where it is assembled

Status: `designed`, written 2026-09-13. Derives from
[`chart-at-the-boundary.md`](chart-at-the-boundary.md) §7, which deferred
exactly this and said why; from
[`serial-and-the-envelope.md`](serial-and-the-envelope.md) (the document
and its seal); and from
[`rust-consumer-surface.md`](rust-consumer-surface.md), which moved the
SDK's composition into the façade and so decides *where* an assembly
lives now. `02-architecture/01-module-catalog.md` gives the crates their
rows.

## 1. Purpose and scope

`teistro_serial::Document` — the foundation, the day's almanac, the
divisional charts, the planetary states, the aspects, the derived points
and the houses — **exists and nothing assembles one**. The type is
built, every section serialises, the whole seals to its own hash, and
the only code that puts one together is an *example of the `serial`
crate* that the schema pass runs.

So a consumer cannot ask for a divisional chart in any language,
including Rust, and each binding's README says so in as many words:
*houses, divisional charts, planetary states, aspects and dashas are
computed by the SDK's Rust crates and do not yet cross the boundary.*

This page decides the assembly and the operation that offers it. It is
not the JSON Schema emitter, which `serial-and-the-envelope.md` §8 says
waits for a whole document to describe — this is what it waits for.

Dashas are out of scope and are Phase 5's: no crate computes one.

## 2. What the measurement says

Read from the source rather than argued, because the six producers'
signatures are the whole of it:

| section | producer | what it needs beyond the foundation |
|---|---|---|
| `foundation` | `chart::foundation::Founder::found` | — it *is* the foundation |
| `panchanga` | `panchanga::almanac::Almanac::day` | the chart's own date, place and clock |
| `vargas` | `vargas::chart::chart(foundation, axis)` | **which** charts, and nothing else |
| `state` | `state::chart::state(foundation, settings)` | the settings |
| `aspects` | `aspect::chart::Aspects::of(foundation, settings)` | the settings |
| `houses` | `houses::chart::Houses::of(foundation)` | — |
| `points` | `points::chart::Points::of(foundation, vara, arc, is_day, &dyn Ascendant)` | a lagna at an arbitrary instant |

**Six of the seven sections are a pure function of the foundation and
the settings**, and the settings are the context's. That is the finding
this page turns on, and it is stronger than it looks: it means the
sections are *derived* rather than *computed* — none of them asks the
provider anything, none of them searches, and none of them can cost a
crossing. A reading is a founding plus arithmetic.

Two of the seven need a word more.

**`panchanga` needs the day, not a range.** The document's field is "the
almanac of the day the chart belongs to", and the chart's day is
`foundation.day.day` — its date, and the place and clock the chart was
cast under. `AlmanacArea::day` takes exactly those. So this section is a
second crossing, and the only one: an almanac searches for sunrise, the
Moon's rises and the limbs' boundaries.

**`points` needs a lagna at an instant that is not the birth.** Saturn's
eighth divides the day's arc and asks for the ascendant at each
division, which is why `Points::of` takes an `Ascendant`. Everything
else it wants is on the foundation already: `day.day.vara`,
`day.part_bounds()` for the arc, and `day.part` for whether it is a day
birth. The lagna is the founder's own — `Founder` computes one at the
instant and one at the day's sunrise — and it is **private**. Making it
public is the one primitive this design adds.

`Points::from_longitudes` is the section without that: the solar chain
and the special lagnas, which are functions of the Sun and the birth
time. The `serial` crate's own sample uses it, which is why nothing has
needed the ascendant yet.

## 3. Where the assembly lives

**In the façade**, `crates/sdk`, beside the areas.

That is not a preference. `rust-consumer-surface.md` moved the SDK's
composition out of the C boundary and into the façade, and the boundary
depends on the façade: `TsContext::build` is a call into
`ContextBuilder`. An assembly written at the boundary would be the
second composition that page exists to have deleted, and a Rust consumer
would again be the one language that cannot reach what the SDK computes.

So `sdk.chart` gains the operation, `crates/ffi` calls it, and the four
bindings decode what it produced. One assembly, four surfaces.

## 4. The operation

```rust
let one = sdk.chart().reading(instant, &request)?;    // Envelope<Document>
let many = sdk.chart().readings(&instants, &request)?; // Envelope<Vec<Document>>
```

The instants are an argument and the rest is the request, which is
`found_many`'s shape exactly: what varies per chart is the instant, and
what a batch holds constant is everything else.

A **batch in the signature**, as `found_many` is and for the reason the
maintainer's brief gives: a rectification pass wants a hundred charts,
the founding is shared, and every section after it is arithmetic on one
foundation. `reading` is the batch of one unwrapped, as `found` is.

### What a caller asks for

A `Reading` request, and what a consumer writes is a **builder rather
than a bit set**, because ADR-0023's type-safety rule applies to a
request as much as to an answer and because one of the fields is a list:

```rust
let request = Reading::at(place, offset)
    .with_kind(ChartKind::Natal)
    .with_vargas([Varga::D9, Varga::D10])
    .with_state()
    .with_aspects()
    .with_houses()
    .with_points()
    .with_panchanga();
```

**Every builder method is `with_*` and every reader is the bare field
name.** Without that rule a setter and its getter want the same word,
and the crate ends up with `kind` beside `chart_kind` for no reason a
reader can see.

What the builder *stores* for the five yes-or-no sections is a bit set,
which is the same split the frame already has: `ts_frame_pack` packs,
and no consumer writes bits. It is also the `sections: u32` §5 says will
cross, so the crossing is a cast rather than a translation.

Three rules decide the defaults, and each is the "defaults yes, dead
ends no" rule of the maintainer's brief:

1. **The foundation is not optional.** Every other section is computed
   from it and a document without one cannot be checked against
   anything, which is what `Document::of` already says in the type.
2. **Nothing else is on by default.** A caller who wants a birth chart
   should not pay for twenty-one divisional charts, and §2's measurement
   is what makes that cheap to honour: an unasked section costs its
   producer not being called.
3. **`vargas` is a list and not a flag**, because "which" is a real
   question with twenty-one answers. `with_vargas([])` is none,
   `with_every_varga()` is all of them, and `with_everything()` is every
   section and every chart — which is what a consumer storing a chart
   for later wants, and what the parity runner will ask for.

   A setter **replaces** rather than accumulates, here as everywhere in
   this builder: one that accumulated would make `with_vargas([])` mean
   nothing at all, and "none" has to be sayable.

### What it answers

`Envelope<Document>` — the `serial` crate's own type, sealed by the area
as the other two are, so the hash is of the document and not of the
foundation inside it (`serial-and-the-envelope.md` §8: the publishers
seal).

## 5. At the boundary

**One blob, not five.** `ts_chart_found` grows the five sections rather
than gaining five entry points beside it, for three reasons:

- A document is one document. Five entry points would be five envelopes
  over one chart, each carrying the same provenance, which is precisely
  what `Document` was shaped to avoid.
- The sections are derived (§2), so there is nothing to save by asking
  for them separately — the cost of a section is the section.
- A binding would otherwise grow five chart types that have to be
  stitched back together by a consumer, and the stitching would be
  written four times.

**A per-chart section is ragged where its length varies with the
values.** The divisional charts are one count for the batch — a chart
asked for is a chart for every instant — and the drishti are not, which
the encoder's own check found by refusing a batch of 47 and 40. So the
rule is: a count in `summary` where the request fixes it, a column of
`cast` where the values do, and the panchanga blob's prefix sum either
way.

**An unasked section is an empty section**, and the format needs no
change to say so. `blob::Writer::finish` refuses a blob with a section
missing, so every section is always written; a section not asked for is
written with zero rows. That is unambiguous here in a way it would not
be everywhere: a chart that *has* houses has twelve of them and a chart
that has states has one per graha, so zero rows cannot mean "asked for
and empty". Where a future section could legitimately be empty, it takes
a count-and-flag as the panchanga's absent muhurtas do.

**The request grows a selector.** `TsChartRequest` carries `struct_size`
and a `reserved: u16`, so a `sections: u32` bit set and a varga list are
an append under the handshake the boundary already performs. A bit set
*here* and a typed struct in each ergonomic layer is the same split the
frame already has: `ts_frame_pack` packs, and no consumer writes bits.

## 6. What each binding gains

The generated decoders come from the description, so the work per
binding is the ergonomic layer: a `reading` option on the chart call and
the accessors over the new sections. The parity runner gains the
sections' values, which is what holds the four to one answer; the
examples gain a ninth scenario, which is what holds the four to one
*shape* — and `check-areas` holds the operation itself to being listed
by all four runners.

## 7. Order of work

1. ~~**`Founder::ascendant_at`**, the one primitive §2 found missing, and
   the façade's `chart().reading()` / `readings()` over it.~~ **Done.**
   `sdk.chart().reading(instant, &request)` and `readings(instants,
   &request)` assemble the document; `Reading` is the builder; the
   founder's lagna is public, taking the chart's own zodiac rather than
   recomputing one, because a division eight hours away would otherwise
   be read in a different ayanamsha than the chart it belongs to.

   **Gulika and Mandi are the acceptance test**, and naming them is
   sharper than counting. They are Saturn's eighth of the day's arc, so
   they are exactly the two points `Points::from_longitudes` — all that
   was reachable before, and what the `serial` crate's own sample
   uses — cannot produce. A count would pass on eleven; the names pass
   only if the founder was threaded all the way through. The whole
   document is 21 vargas, 9 states, 42 drishti, 12 bhavas, 13 points and
   the day's almanac, and it came out right on the first run.

   Two things the building corrected. The `Reading`'s five yes-or-no
   sections are stored as a **bit set** and not five `bool` fields —
   clippy's `excessive_bools` asked, and it is right twice over, because
   that set is the `sections: u32` §5 says will cross. And every builder
   method is `with_*` while every reader is the bare field name, because
   without that rule a setter and its getter want the same word and the
   crate ends up with `kind` beside `chart_kind` for no reason a reader
   can see.
2. **The boundary**: the five sections described, encoded and gated by
   `check-ffi`, with `cargo xtask gen ffi` writing the four decoders.

   **The divisional charts have crossed**, which is the first of the
   five and the one that proves the rest. `TsChartRequest` grew
   `sections` and a varga list; `ts_chart_found` builds a `ChartRequest`
   and calls the façade's `readings`; the blob grew `vargas` and
   `varga_grahas` beside a `varga_count` in the summary; and all four
   ergonomic layers offer `vargas` and answer with a list of charts,
   each graha placed as `{rashi, part, sign}` — where `sign == rashi` is
   the body keeping its sign, which in the navamsha is *vargottama*.

   **And the drishti**, the second of the five, which found the thing
   §5 got wrong. A chart's relations are a function of **where** the
   bodies stand rather than of how many there are: two charts of the
   same nine grahas at one place hold 47 and 40. So the aspects section
   is **ragged** — the rows concatenated, `cast.aspect_count` saying
   where each chart's begin, which is the panchanga blob's own rule —
   and §5's "one count for the batch" was true of the divisional charts
   and false here. The check that would have enforced it is what found
   out, which is why it was written as a check rather than assumed.

   `Strength` crosses as `TsStrength`, the boundary's own enum, because
   it is a property of a relation rather than a thing with a key.

   **And the derived points**, the third, which took the drishti's
   lesson as read: ragged from the start, because Saturn's eighth needs
   an arc to divide and a chart whose day has none carries two points
   fewer. That is the second reason a section's length can vary, and it
   is not the first — the drishti vary with *where* the bodies stand and
   the points with *what the day allows* — so the rule §5 now states
   covers both.

   **And the houses service**, the fourth, which is the case that says
   why §5's rule is about the *values* and not about the section: a chart
   that has bhavas has twelve, always, so the section is fixed and an
   empty one can only mean "not asked for".

   What crosses is **what is not elsewhere**. The madhya and the sandhi
   are already in `houses` and `chalit`; which bhava each body falls in
   is already in `grahas`; the systems are already in `readings`. What
   only this service computes is the sign a bhava's *middle* falls in —
   which under an unequal division is not the sign it begins in — its
   lord, and its quadrant. That is `chart-at-the-boundary.md` §3's rule
   applied: describe each shape once.

   The `outcome` a chart's houses were computed under does **not**
   cross, and the reason is the type's own: it is a fact about the
   computation that made the foundation, which the foundation does not
   carry, so `Houses::of` takes `Defined` and the boundary would have
   nothing truer to send.

   **Four bindings agree on 1 451 values**, 777 of them new and every one
   right on its first run. The runners ask for **two** charts over
   **two** instants deliberately: the layout is charts outermost, so one
   of each would pass a transposed stride; and they print **every**
   drishti rather than the count, because a count that agrees while the
   rows disagree is exactly what a ragged section risks.

   Three things the crossing found. `ts_chart_found` and
   `ts_panchanga_days` still **composed their own founder and almanac** —
   the second copy of each, left behind by the dependency inversion —
   and now call the façade. Both request structs carried a `struct_size`
   that **nothing read**, so a caller compiled against an older header
   would have read past the end of a grown struct rather than met a
   `SCHEMA_VERSION` refusal; eleven of the thirteen boundary structs
   were registered and these two were not. And `nullable` on an *array
   of enum members* is a shape no emitter has been shown — it mapped the
   option's contents where it meant to map the array's — so the varga
   list is not nullable, as `instants` is not: an empty array says
   "none" without needing one.
3. **The ergonomic layers**, one per binding, each with its own tests.
   Done for the divisional charts, and the pattern for the other four.
4. **Parity and the examples**: the sections' values compared across four
   runners, and a ninth example in each binding. The parity half is done
   for the divisional charts.
5. **The JSON Schema emitter**, which `serial-and-the-envelope.md` §8
   says waits for a whole document to describe.

Steps 1 and 2 are where the design can be wrong; 3 and 4 are the pattern
`chart-at-the-boundary.md` proved twice and this repeats five times.

## 8. What this design does not settle

- **Dashas.** No crate computes one, and Phase 5 owns it. The document
  has no field for it and gains one when there is something to put in
  it.
- **Whether `vargas` should default to none or to D9.** None is chosen
  here on the "pay for what you ask for" rule, and D9 is the one chart
  a Vedic consumer almost always wants beside the rashi. If the
  measurement of what consumers actually request ever exists, it decides
  this; until then a default that computes something nobody asked for is
  the worse error.
- **The `geometry` layout registry** (ADR-0026), which is a rendering
  concern over a document rather than a section of one.
