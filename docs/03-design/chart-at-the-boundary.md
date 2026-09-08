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

## 4. The foundation blob

Sections in the order a reader wants them, using the three kinds the
description already has (`fixed`, `columns`, `bytes`):

| id | section | kind | fields |
|---|---|---|---|
| 1 | `summary` | fixed | `instant`, `kind`, `lagna_deg`, `day_lagna_deg`, `latitude_deg`, `longitude_deg`, `altitude_m`, `graha_count` |
| 2 | `grahas` | columns | `graha`, `longitude_deg`, `latitude_deg`, `distance_au`, `speed_deg_per_day`, `tropical_deg`, and `house_*` and `placement_*` for each of `bhava`, `method`, `from_madhya_deg`, `through` |
| 3 | `houses` | fixed + columns | the reading (`method`, `reading`, `source`) and twelve `madhya` and `sandhi` |
| 4 | `chalit` | fixed + columns | the same shape, the chart's chalit |
| 5 | `zodiac` | fixed | `ayanamsha`, `ayanamsha_kind`, `offset_deg`, `frame_bits` |
| 6 | `day` | fixed | the arc, its date, its convention and its state — **the section the panchanga blob shares**, §8 |
| 7 | `timing` | fixed | `ghati_reckoning`, `hora_reckoning`, the hora and the ishtakaal |
| 8 | `model` | bytes | the solar model's description, which is free text |
| 9 | `steps` | bytes | as `positions` writes them |
| 10 | `provenance` | bytes | the envelope, as canonical JSON |

`steps` and `provenance` are the same two sections `positions` ends with,
and a binding decodes them with the code it already has.

Section 6 is not this blob's alone. The panchanga's day is the same nine
fields with the same values, measured field for field on the same chart,
so it is declared once — a `fn day_section(id)` both blobs call — and
carries a **shape name** so that the three bindings decode it into one
type rather than two identical ones (§8).

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
- **Whether `ts_chart_found` should take a batch.** `Founder` has
  `found_one` and a batch form, and the boundary's whole shape elsewhere
  is one call per grid. A rectification pass wants a hundred charts and
  would otherwise cross a hundred times.
