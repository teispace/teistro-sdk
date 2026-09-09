# The chart foundation and the panchanga day at the boundary

Status: `designed`, written 2026-09-08 from the shape
[`schema-measured.md`](schema-measured.md) measured. Derives from
[`ffi-abi-and-api-description.md`](ffi-abi-and-api-description.md) (the
result blob and how a section is described),
[`chart-foundation.md`](chart-foundation.md) and
[`panchanga-day.md`](panchanga-day.md) (the values themselves), and
[`serial-and-the-envelope.md`](serial-and-the-envelope.md) (the
provenance a result carries). `02-architecture/01-module-catalog.md`
gives it its row.

## 1. Purpose and scope

Two entry points, `ts_chart_found` and `ts_panchanga_day`, and the two
result blobs they answer with, so that a binding can show a chart.

It is Phase 4's own exit condition: *the baseline engine's foundation and
daily panchanga golden vectors reproduced within tolerance **in three
bindings** on both providers*. Nothing above `positions` crosses today,
which is why each binding's six examples stop where they do and why the
parity gate compares 103 values rather than a chart.

It is not: the divisional charts, the planetary state, the aspects, the
derived points or the houses service. Those are the same pattern applied
five more times, and doing two first is what proves the pattern before
it is repeated (§7).

## 2. What the measurement says

The serialised document is an exact specification of what has to cross,
and it was already measured. A foundation is **72 distinct paths in
twelve groups**, and 228 leaves once the repeating groups are counted
out:

| group | leaves | shape |
|---|---|---|
| `grahas` | 14 × 9 | the one genuinely columnar group |
| `day` | 20 | the day arc, its date and its model |
| `zodiac` | 11 | the ayanamsha and the frame asked for |
| `timing` | 9 | the ishtakaal and the planetary hour |
| `houses`, `chalit` | 5 each | a reading, and twelve madhya and sandhi |
| `place` | 3 | latitude, longitude, altitude |
| `instant`, `kind`, `lagna_deg`, `day_lagna_deg`, `steps` | 1 each | scalars and the step list |

Only one group repeats per row. That is what makes a columnar section the
right shape for `grahas` and the wrong shape for everything else.

## 3. What is reused rather than described again

Four shapes appear more than once, and the boundary describes each of
them once. This is the part of the design worth arguing, because a blob
that writes every leaf it is given would be correct and would also
enshrine the repetition in four generated decoders and three ergonomic
layers.

| what repeats | naive | here |
|---|---|---|
| `zodiac.request` is a `Frame` | 8 fields | **1 packed `u32`**, exactly as `positions` writes `frame_bits` |
| `day.day.place` is the chart's own place | 3 fields | **0** — a chart has one place, and the day belongs to the chart |
| a graha's `house` and `placement` are one shape | described twice | one described shape, two column prefixes |
| `houses` and `chalit` are one shape (`Bhavas`) | described twice | one described shape, two sections |

Ten leaves go, and the two shapes that remain twice are named once in the
description rather than twice. The `frame_bits` reuse is the load-bearing
one: a binding already unpacks a frame, in a helper the three of them
share, so `zodiac.request` costs a binding nothing new at all.

## 3a. What a consumer must be able to change

A dead end is a place where the SDK decided something a consumer might
reasonably need to decide, and left no way to say otherwise. Three rules
follow from that, and each has already caught something on this page.

**Batch by default.** Every entry point that computes a value takes a
grid, and a caller wanting one passes a grid of one. The first draft of
`ts_chart_found` took a single instant while `Founder` had a batch form
sitting unused — §8 has the correction, and it is built: the request
carries `instants` and a count, the blob carries `chart_count`, and each
binding's layer offers `found(one)` and `foundMany(list)` over the one
crossing. A convenience belongs in a binding's ergonomic layer, not in a
second entry point.

**A choice the SDK makes is a knob, or it is named.** The precession
model is chosen by the boundary because the settings have no knob for
it, and eleven models ship. That is defensible only while the choice is
*declared* — `PrecessionModel::default()` rather than a literal — and
the open question in §8 is whether it should be a knob at all. The test
for a dead end is: could a consumer want the other answer, and can they
have it?

**What was applied is reported.** A result carries the frame it was
computed in, the steps applied, the method that placed each graha and
the reckoning each ghati was counted under, because a consumer who
cannot see what was chosen cannot tell whether to change it. That is why
`readings` and `timing` are sections rather than settings echoes.

## 4. The foundation blob

The blob is a **batch**: charts founded at one place, sharing settings, a
solar model and a day reckoning. A batch of one is the ordinary case and
the blob's counts say so. Sections in the order a reader wants them,
using the three kinds the description already has (`fixed`, `columns`,
`bytes`):

| id | section | kind | rows | fields |
|---|---|---|---|---|
| 1 | `summary` | fixed | 1 | `kind`, `chart_count`, `graha_count`, `latitude_deg`, `longitude_deg`, `altitude_m` |
| 2 | `cast` | columns | `chart_count` | `instant`, `lagna_deg`, `day_lagna_deg`, `ayanamsha_offset_deg` |
| 3 | `grahas` | columns | `chart_count × graha_count` | `graha`, `longitude_deg`, `latitude_deg`, `distance_au`, `speed_deg_per_day`, `tropical_deg`, and `house_*` and `placement_*` for each of `bhava`, `method`, `from_madhya_deg`, `through` |
| 4 | `readings` | fixed | 1 | which house system produced each set of bhavas, and which bound each is read against |
| 5 | `houses` | columns | `chart_count × 12` | `madhya_deg`, `sandhi_deg` |
| 6 | `chalit` | columns | `chart_count × 12` | the same shape, each chart's chalit |
| 7 | `zodiac` | fixed | 1 | `frame_bits`, `ayanamsha_kind`, `ayanamsha` |
| 8 | `day` | columns | `chart_count` | the arc, its date, its convention and its state — **the section the panchanga blob shares**, §8 |
| 9 | `timing` | columns | `chart_count` | `ghati_reckoning`, `hora_reckoning`, the hora and the ishtakaal |
| 10 | `model` | bytes | — | the solar model's description, which is free text |
| 11 | `steps` | bytes | — | the completion steps, as JSON |
| 12 | `provenance` | bytes | — | the envelope, as canonical JSON |

`steps` and `provenance` are the same two sections `positions` ends with,
and a binding decodes them with the code it already has.

**Charts outermost**, as the positions blob puts instants outermost: row
`i × graha_count + j` of `grahas` is chart `i`, graha `j`, and row
`i × 12 + j` of `houses` is chart `i`, bhava `j`. One rule for every
per-chart section, and the same rule the SDK already had.

The split between `summary` and `cast` is the batch's own line. What a
**request** decides — the place, the kind, how many of what — is written
once, from the request, so a batch of none still says under what it
founded none. What an **instant** decides is a row of `cast`. The
ayanamsha offset moved there from `zodiac` when the blob became a batch:
it precesses, so two instants a month apart do not share one.

This corrects what §4 first said. The first table had `summary` carrying
one chart's `instant` and two lagnas, `houses` and `chalit` as "fixed +
columns" (a kind that does not exist — they are a shared `readings`
section plus two column sections), and `day` and `timing` as fixed
sections of one row. All three were the shape a one-chart entry point
wants, and none survived the batch.

Section 8 is not this blob's alone. The panchanga's day is the same
eighteen fields with the same values, measured field for field on the
same chart, so it is declared once — a `fn day_section(id)` both blobs
call — and carries a **shape name** so that the three bindings decode it
into one type rather than two identical ones (§8). Shaping now works for
column sections as well as fixed ones, because that is what the day
became; `check_shapes` refuses two sections that name one shape and
disagree about it, since the emitters render a shape once and would
otherwise silently decode the second through the first one's type.

## 4a. The panchanga blob

A day's lists are **ragged**, which is the one thing a chart blob never
had to decide, and
[`panchanga-at-the-boundary-measured.md`](panchanga-at-the-boundary-measured.md)
measured it rather than supposing: ten of the fifteen vary, and a
rectangular layout wastes 78.1% of its rows once one polar day joins a
batch, because that day sets the stride for every other. So every list is
concatenated across the batch and a `counts` section says how many rows
are each day's — **one rule for all thirteen**, including the five whose
length never moved, because two layouts in one blob is two things for a
reader to learn and the fixed ones lose nothing by it.

| id | section | kind | rows | fields |
|---|---|---|---|---|
| 1 | `summary` | fixed | 1 | `day_count`, the place, the calendar, the lunar-month convention |
| 2 | `days` | columns | `day_count` | the window, the month under both conventions, the paksha, the ayana, the disha shool, and the three values a day may not have |
| 3 | `counts` | columns | `day_count` | thirteen counts: how many rows of each per-day section are this day's |
| 4 | `day` | columns | `day_count` | **the section a chart shares**, under the same shape name |
| 5–11 | `tithi`, `nakshatra`, `yoga`, `karana`, `panchaka`, `moon_signs`, `sun_signs` | columns | ragged | a span each: the member, its own bounds, and the clipped ones |
| 12–17 | `kaalas`, `choghadiya`, `horas`, `muhurtas`, `moon_events`, `muhurta_yogas` | columns | ragged | the periods, and what held |
| 18–19 | `model`, `provenance` | bytes | — | as every blob ends |

Three things it settles that the chart blob did not have to.

**A value a day may not have crosses as a presence flag beside it.** The
sankranti, Abhijit and Brahma muhurta are each an `Option`, and a blob
has no such thing: every column has a value in every row. A sentinel
cannot serve, because an absent Abhijit and an Abhijit at Julian day zero
are both nought and a reader cannot tell them apart. So `has_abhijit`
sits beside `abhijit_from` and `abhijit_to`, which is the rule §8's
tagged enums already follow.

**Two lists that answer one question in two halves become one section
with a discriminant.** The fifteen muhurtas of the daylight and the
fifteen of the night are one `muhurtas` section with a `daylight` column;
the moonrises and the moonsets are one `moon_events` section with a
`kind`. Two sections of one column each would be two counts, two offsets
and two decoded types for what a reader thinks of as one list.

**The seven span lists do *not* share a shape**, and this is the first
place the shape rule needed a boundary. A shape makes two sections decode
to **one** type, which is right when they are the same section in two
blobs — a chart's day and a panchanga's — and wrong here: a span of
tithis and a span of nakshatras have the same structure and different
meanings, and one type for both would let a caller pass either where the
other is wanted. The repetition worth removing is in the *declaration*,
which a `span_section(…)` helper removes; the distinction worth keeping
is in the *type*, which seven sections keep. **A shape is for sameness of
meaning, not similarity of structure.**

## 5. The entry points

```c
ts_status ts_chart_found(ts_context *ctx, const ts_chart_request *request,
                         ts_blob **out_blob);
ts_status ts_panchanga_day(ts_context *ctx, const ts_panchanga_request *request,
                           ts_blob **out_blob);
```

Both take a request struct rather than a long argument list, as
`ts_positions` does, so a field added later does not move an argument.
Both answer a blob the caller frees with `ts_blob_free`.

A request carries the instant or the date, the place, the chart kind —
and **the local clock**, which is the one thing a chart needs that no
setting knows.

That last is a correction to what this section first said. `Founder::new`
takes seven things: a provider, the resolved settings, a solar model, a
calendar, a clock, a precession model and a Delta T model. The context
has the provider, the settings and Delta T; the model and the precession
come from the settings. The clock does not come from anywhere. A chart's
day is reckoned from a local sunrise and its date is a civil date, and a
longitude gives local *mean* time rather than a civil offset, so the
caller has to say. Everything else stays in the settings, which is what
makes two calls under one context comparable and what the settings hash
is for.

The calendar has an answer already waiting for it. `calendars.civil_calendar`
is one of the thirteen knobs `check-lints` reports as having no reader,
and its own deferral says why: *"this gains a reader when `serial` or a
binding builds a chart from a settings document alone."* That is this
entry point. The knob is read here rather than the request naming a
calendar, which is both the honest reading of the knob and one fewer
field on the request.

## 6. Tests

- **Round trip through the boundary**: a foundation computed in Rust,
  written to a blob, decoded, and compared field by field with the
  value — the same shape `check-ffi`'s blob fixtures already use.
- **The three bindings agree**, in `check-parity`, on every field of a
  founded chart and a panchanga day. The gate compares 103 values today;
  a foundation adds 72 distinct paths and a panchanga 85, of which 18 are
  the day the two share — so 139 new values, and 224 in all.
- **The golden vectors reproduce in three bindings**, which is Phase 4's
  exit condition and the reason for the work.
- **The examples show a chart**, which is what a reader will actually
  copy: `birth_chart` stops at nine longitudes today because that is all
  that crosses.

## 7. Why two and not seven

The five sections left out are the same pattern again: a columnar group,
a few fixed ones, and the two `bytes` sections every blob ends with. The
argument for doing two first is that the pattern is unproven — no blob
but `positions` and `intl_render` exists, and neither has a nested value
tree in it. A mistake in §3's reuse, or in how a fixed section holds a
nested struct, is a mistake repeated seven times before anything catches
it.

The argument against is that a document with two of seven sections is not
a document, and the JSON Schema this unblocks would describe a fragment.
That is true and it is the price. The schema emitter is worth writing
when the description holds a whole document, so it waits for the other
five rather than describing part of one.

## 8. Open questions

- **~~What the panchanga blob shares with this one.~~ Measured, and it
  is the whole day.** `foundation.day.day` and `panchanga.day` are the
  same nine fields — `convention`, `date`, `model`, `next_sunrise`,
  `place`, `state`, `sunrise`, `sunset`, `vara` — with the same values,
  field for field, on the same chart. They are **18 of the panchanga's
  85 distinct paths** and the widest fixed section in either blob, and
  describing them twice is precisely what §3 argues against.

  Sharing it splits into three questions, and reading
  `crates/ffi/src/schemas.rs` settles two of them.

  **The declaration is already shareable.** A blob schema is Rust data —
  `positions()` returns a `BlobSchema` built from `SectionSchema::fixed`
  and its like — so one `fn day_section(id: u32) -> SectionSchema` called
  by both blobs is the whole of it. Nothing in the model is in the way.

  **The serialised description repeating it is not repetition.** Each
  blob's wire layout really does contain those fields, and `api.json`
  describes wire layouts. A reader of the description should see what is
  in the bytes.

  **The generated decoders repeating it is the real cost.** Two
  structurally identical sections become two decoded types in each of
  three bindings — a `FoundationDay` and a `PanchangaDay` with the same
  fields, and a caller who wants to render a day has to write it twice.
  That is the wart worth removing, and the smallest thing that removes it
  is a **shape name** on the section: `SectionSchema` gains an optional
  `shape`, two sections that declare the same one are emitted as one
  decoded type, and a section without one behaves exactly as today. It is
  additive to the model, and the five sections still to come — the
  vargas, the state, the aspects, the points, the houses — all carry a
  chart's identity and would each want the same day again.
- **~~Whether the day's date needs its own section.~~ It already has
  one.** The boundary describes `TsCalendarDate` — calendar, era, year,
  era year, month, day, resolution, and the two computed fields — and
  `TsResolution` with it, because a date already crosses for
  `ts_calendar_convert`. The day section's date is those nine fields
  flattened, and every binding already decodes one.
- **Two of the day's fields have no boundary form, and one of them is
  the interesting kind.** Of the day's eighteen leaves, the date is
  described, the place belongs to the summary (§3), the instants are
  doubles, and `vara`, `calendar` and `era` are catalogued. Two are
  neither:

  | field | in the document | what it is |
  |---|---|---|
  | `state` | `{"state": "NORMAL"}` | a tagged enum, not described |
  | `convention` | `{"kind": "NAMED", "which": "CENTRE_NO_REFRACTION"}` | a tagged enum **with a payload**, not described |

  The second is the one that matters. A catalogued member crosses as a
  `u16` id, which is what every enum field in a blob is today. A tagged
  enum whose variant carries data cannot: `NAMED` carries a `which`, and
  the family this belongs to has variants carrying a `f64` elsewhere in
  the calendar crate (`MonthStartRule::Shifted { days }`). Two fields
  side by side would encode today's shape and would be a lie the first
  time a variant carried something else.

  Reading the type settles it, and rules out both flattenings. The
  payload variant is not hypothetical, it is there today:

  ```rust
  pub enum SunriseConvention {
      Named { which: Sunrise },
      Custom { altitude_deg: f64 },
  }
  ```

  So `kind` and `which` side by side is wrong **now**, not later — a
  `Custom` convention has no `which` — and a `u16` over the pairs is
  wrong too, because the pairs are not enumerable: an altitude is a
  continuous double.

  What is left is the shape a tagged union has always had at a C
  boundary: **a `kind` id and one payload slot, read according to the
  kind.** A fixed section already gives every field an eight-byte slot,
  so the payload costs the wire format nothing new — `Named` puts a
  `Sunrise` id in it and `Custom` an altitude.

  And the boundary already has the precedent for what the layers do with
  it. `frame_bits` is a packed `u32` that each ergonomic layer unpacks
  into a `Frame`; a tagged enum is the same bargain, a scalar pair the
  layer turns into its language's own union. The description stays flat
  and truthful, and no emitter needs a field whose type depends on
  another field.

  **The rule, then**: a tagged enum crosses as a `<name>_kind` and as
  many payload fields as its widest variant needs, each named
  `<name>_<field>`, and each binding's layer presents the union. It
  needs no new machinery, and it is the same trade the frame already
  makes.

  An earlier draft of this rule said `<name>_kind` and one
  `<name>_value`, which writing the day section showed to be too narrow.
  `DayState::Polar { kind, policy }` carries two, so the day crosses as
  `state_kind`, `state_polar_kind` and `state_polar_policy` — a normal
  day leaving the last two at nought. `SunriseConvention` really does
  need only one, and `AyanamshaChoice::Custom { value_deg, rate }`
  needs none at all, because a chart carries the ayanamsha it *applied*
  and the settings hash pins what it was asked for: a result carries
  what it computed, and the envelope carries what it was told.
- **What `State` at the boundary is.** The description has a `State`, and
  it is the planetary one — retrograde, combust, gandanta — not the
  day's. The two names collide and the day's needs a different one.
- **Which precession, and who chooses.** `Founder::new` takes a
  `PrecessionModel` and the settings have no knob for one, so the
  boundary must pick.

  It picks `PrecessionModel::default()`, which is Vondrak2011 —
  `#[derive(Default)]` with `#[default]` on that variant and a doc
  comment saying so. (An earlier draft of this page claimed the SDK had
  a de facto default that nothing declared, on the strength of a grep
  for `impl Default for` that cannot see a derive. It is declared, in
  one place, and the five callers naming it explicitly are all tests,
  where saying what you use is right.)

  What remains open is whether it should be a **settings knob** at all,
  as `time.delta_t` is — precession is the same kind of choice, and the
  astro layer ships eleven models. That is a larger question than this
  page: a new knob moves the settings hash of every profile, so it is a
  decision with a Numbers line rather than a tidy-up.
- **A chart cannot be founded under `nepali-default`.** Founding one
  through the Node layer refuses with *"frame completion step `centre`
  is not implemented"*. The profile asks for `Centre::Topocentric`
  (`profiles.rs`), the analytic provider answers geocentric, and
  `completion.rs:333` refuses any request whose centre differs from the
  provider's native one.

  Nothing had tried it: every Rust test that founds a chart uses
  `DEFAULT_PROFILE`, which is geocentric, so the first caller to ask for
  the Nepali profile was a binding. Under the default profile a chart
  founds and reads back exactly as it should.

  **It is not an oversight, and that makes it worse.** Phase 2's exit
  note says the step was *"deferred by decision: the completion's centre,
  corrections and equinox steps to Phase 3, where the built-in ephemeris
  needs them"*. So the astronomy layer closed knowing it, and Phase 3
  owns it.

  What no phase says is that **Phase 4's exit depends on it**. The exit
  is the golden vectors reproduced in three bindings; all 55 recorded
  charts are `topocentric: true`; and the corpus carries fixtures built
  precisely to make the difference decide an outcome — c049 to c055 put
  the Moon "at a pada edge where the geocentric Moon is still on the
  other side, so the dasha lord or the pada depends on the frame". A
  geocentric SDK cannot reproduce them.

  The dependency is invisible today because nothing compares a *computed*
  longitude against those charts: `crates/chart/tests/baseline_bhavas.rs`
  reads the recorded `sidereal_longitude_deg` and checks the bhava it
  falls in, so it tests the placement rule rather than the position. The
  first thing to compare computed positions against the corpus will meet
  this, and it will meet it as 495 wrong longitudes rather than as a
  refusal.

  Two consequences. **The examples** cannot show a chart under the
  profile their five neighbours use, so a chart example either changes
  profile or waits. **Phase 4 cannot exit before Phase 3 delivers the
  centre step**, which is a cross-phase dependency neither phase records
  and the roadmap should.
- **The boundary seals and the producer does not.** `ts_chart_found`
  sets `content_hash` on the provenance before writing the blob, as
  `ts_positions` does, because `Founder` leaves the placeholder that
  `serial-measured.md` found. So a Rust caller holding an `Envelope` and
  a binding caller holding the blob get **different hashes for the same
  chart**. That is
  [`serial-and-the-envelope.md`](serial-and-the-envelope.md) §8's open
  question — "whether the producers should seal" — and this is the first
  place it stops being theoretical.
- **~~Whether `ts_chart_found` should take a batch.~~ It must, and this
  page built the dead end it warned about.** `Founder::found(instants,
  place, kind)` already exists — "founds many charts at one place,
  sharing the settings and the solar model" — with one stamp over the
  batch, one input hash over the whole request, and an
  `empty_provenance` for a batch that founds nothing. It is complete,
  and the first version of `ts_chart_found` exposed only `found_one`.

  That is the shape the SDK exists to avoid. `ts_positions` takes a
  **grid** for exactly this reason: a year of the sky is one crossing
  rather than 366, and the provider's setup is paid once. A chart is the
  same bargain and a stronger one, because the solar model and the
  settings resolution are shared across the batch by the founder itself.
  A rectification pass wants a hundred charts.

  So the entry point takes `instants` and a count, as `PositionRequestC`
  does, and the blob carries `chart_count` with every per-chart section
  gaining a chart dimension — charts outermost, as the positions blob
  puts instants outermost. What is per-*request* rather than per-chart —
  the place, the model's description, the provenance — stays one to a
  blob.

  Each binding's ergonomic layer then offers both shapes over the one
  entry point: `found(one)` for the common case and `foundMany(list)`
  for the batch, exactly as a caller passing a single instant to
  `positions` gets a one-row grid. **A convenience is a layer's job; a
  dead end is not the boundary's right.**

  **Built, and the building corrected the plan in four places.**

  1. *The blob became a batch, not a batch of blobs.* The per-chart
     sections are `cast`, `grahas`, `houses`, `chalit`, `day` and
     `timing`; the per-request ones are `summary`, `readings`, `zodiac`,
     `model`, `steps` and `provenance`. The ayanamsha offset was written
     down as per-request and is not — it precesses — so it moved into
     `cast`.
  2. *Per-request fields are written from the request, not scraped from
     the first chart.* Writing them from `charts[0]` reads the same for
     every batch of one or more and is a lie for a batch of none, which
     the founder explicitly supports (`empty_provenance`). `summary`
     therefore takes the place and the kind as arguments. Three
     sections — `readings`, `zodiac` and `model` — can only be learnt by
     founding, so an empty batch writes them as zero and its envelope
     carries the settings hash that would have produced them.
  3. *`day` and `timing` became column sections*, which is what made
     shaping a column section necessary: the day is shared with the
     panchanga blob and had to stay one type in each binding.
  4. *A columns section written from rows needed a writer of its own.*
     A fixed section gives every field an eight-byte slot; a column
     stores the scalar's own width. `Writer::rows` transposes rows of
     `FixedValue` into typed columns and **refuses** a value too wide
     for its column rather than truncating it, so `day_values` stays the
     one place the day's field order lives.

  The three ergonomic layers now offer `found(one)` and
  `foundMany(list)` over the one crossing, and a batch of none is an
  empty result in all three rather than an error.
- **The two blobs spell a completion step differently.** A positions
  result carries `steps` as objects — `{"name": "positions",
  "implementation": "PASS_THROUGH"}`, serde's SCREAMING_SNAKE_CASE — and
  a chart carries them as strings, `"positions:PassThrough"`, because
  `Completed::step_keys` formats the variant with `{:?}`. Same steps,
  two spellings and two shapes, in two blobs a caller may hold at once.

  The chart cannot simply carry the objects: `ChartFoundation` derives
  `Deserialize` and `Step` borrows a `&'static str` name, so the
  structured form does not round-trip through the type a stored chart
  uses. Making it a `String` name, or giving `Step` an owned form, is a
  change to the astronomy layer's public type — small, but not one to
  make at the tail of a boundary change. The schema now documents what
  each blob actually carries; which of the two every blob should use is
  the question left.
- **An empty grid is refused for positions and accepted for charts.**
  `foundMany([])` answers an empty batch, because a caller who filtered a
  list to nothing should not have to special-case it, and the boundary
  accepts an empty grid for both entry points. The Node layer's
  `positions` still refuses one. The refusal predates the batch entry
  point and may be right — an empty grid of instants to place bodies at
  is more likely a slip than an ask — but two entry points behaving
  differently on `[]` is a papercut, and choosing deliberately is better
  than the asymmetry standing because nobody looked.
- **The day's catalogued columns cross as ids, and only some are named.**
  A chart's `day.vara` is a `u16` in the blob; the Node layer names it
  and leaves the other seventeen columns as the numbers they are, which
  is what the layer did before the batch. Naming them all by hand in
  three bindings is the kind of table that drifts from the schema. The
  emitters know which columns name an enum — `field.enum_name` is what
  writes "The values are `Vara` ids" into the generated documentation —
  so a generated map from section and column to enum name would let each
  layer resolve them without a table of its own. Worth doing when a
  second blob carries catalogued columns.
- **~~What the panchanga blob shares with this one.~~ Built, and the
  sharing cost a correction.** The `day` section really is one section in
  two blobs, decoding to one type in each binding — the shape mechanism
  doing exactly what it was built for. But it carried **twenty** fields
  while this page had said all along that a chart's day and a
  panchanga's are the same **eighteen**, and the two extra were the
  reason: `part` — which arc of the day the instant falls in — and
  `elapsed` — how far through that arc it is. Both belong to an
  **instant**, not to a day. A chart has one and a panchanga day has
  none, so a panchanga could only have filled them with a lie.

  Nothing had noticed because a chart was the only blob carrying a day,
  and a field that is wrong for a reader who does not exist reads as
  right. They now sit in the chart's `cast` section with the other things
  an instant decides, and the shared section is the eighteen it always
  claimed to be. The page's own count was the evidence, unread for two
  sessions.
- **`has` covers messages and not entities.** A binding's `has(key)`
  answers whether the locale or its fallbacks hold a *message*; there is
  no non-throwing way to ask the same of an **entity**. The almanac
  example needs one, because a pack names most of the catalogue and not
  all of it, so all three bindings catch the refusal instead. Catching an
  exception to ask a question is the shape of a missing accessor.
- **Two catalogue kinds have no name in any locale.** `masa` and
  `direction` are absent from all five entity packs the SDK ships
  (`i18n/*/sdk.entity.json` carry 25 or 26 kinds and not these), so an
  almanac cannot print the lunar month or the disha shool in the
  reader's language — the two things a panchanga page leads with. The
  examples fall back to the key and say so. Filling them is content
  rather than code, and content in this project needs a source: twelve
  masa names and the directions, in Devanagari and in IAST, from a text
  rather than from a guess.
