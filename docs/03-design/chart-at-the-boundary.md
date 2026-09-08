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
| 6 | `day` | fixed | the arc, its date, its convention and its state |
| 7 | `timing` | fixed | `ghati_reckoning`, `hora_reckoning`, the hora and the ishtakaal |
| 8 | `model` | bytes | the solar model's description, which is free text |
| 9 | `steps` | bytes | as `positions` writes them |
| 10 | `provenance` | bytes | the envelope, as canonical JSON |

`steps` and `provenance` are the same two sections `positions` ends with,
and a binding decodes them with the code it already has.

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

A request carries the instant or the date, the place, the chart kind, and
nothing else: everything else is the context's settings, which is what
makes two calls under one context comparable and what the settings hash
is for.

## 6. Tests

- **Round trip through the boundary**: a foundation computed in Rust,
  written to a blob, decoded, and compared field by field with the
  value — the same shape `check-ffi`'s blob fixtures already use.
- **The three bindings agree**, in `check-parity`, on every field of a
  founded chart and a panchanga day. The gate compares 103 values today;
  a foundation adds about 70 and a day about 60.
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

- **Whether the day's date needs its own section.** A `CalendarDate` has
  a calendar, a year, a month, a day, an era and its year, and a
  resolution — seven fields inside `day`, which is already the widest
  fixed section. It may want to be its own, and the panchanga blob wants
  the same shape.
- **What the panchanga blob shares with this one.** The day arc, the
  date and the place are in both. Two blobs that describe the same day
  twice would be exactly what §3 argues against; one shared section
  described once is the answer, and where it lives is the question.
- **Whether `ts_chart_found` should take a batch.** `Founder` has
  `found_one` and a batch form, and the boundary's whole shape elsewhere
  is one call per grid. A rectification pass wants a hundred charts and
  would otherwise cross a hundred times.
