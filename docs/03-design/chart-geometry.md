# Chart geometry: layouts as data, and what places a chart in one

Status: `building` — steps 1 to 4 built 2026-09-14; the document section and the renderer follow. It builds the first two parts of
[ADR-0026](../08-decisions/adr-0026-chart-geometry-and-the-first-party-renderer.md)
(layouts are data, and geometry lands in Phase 4). The third part, the
SVG renderer, is §8's later step. The research this page rests on was
done before it was written, and it corrects the ADR in two places (§2).

## 1. Purpose and scope

A developer who can compute a chart and cannot draw one has not finished
evaluating the SDK. Practitioners reject output that "does not look like
a real kundali" (ADR-0026's evidence). The SDK already has everything a
drawing needs: the lagna, every graha's sign and longitude, the houses.
What it lacks is the **layout**: which region of the page belongs to
which sign or house, and where its label and its bodies sit.

This page decides:

- the layout **data model**;
- the **rows** that ship, and what cites each;
- the **placement** that turns a chart into drawable geometry;
- the **validation** a row passes before it is accepted, whether shipped
  or registered by a consumer;
- the **order of work** to the renderer.

Out of scope: drawing. The core never draws. `render-svg` is a separate
crate above this one, consuming the same public geometry a third party
would (ADR-0026 §3).

## 2. What the research found

Three sources were read against each other:

1. The published figures of the three regional square charts
   (Wikipedia's *Kundali (astrology)*, `Kundali_01a/02a/03a.png`, which
   number every cell).
2. Tutorials that describe them in prose.
3. The baseline application's production kundali components, which
   render North, South, East, lotus and Sudarshan views to readers
   today.

| layout | what is fixed | where it starts | direction | agreement |
|---|---|---|---|---|
| North Indian | the **houses** | house 1 is the top diamond | anticlockwise | figure, tutorials and the application agree |
| South Indian | the **signs** | Pisces top-left, Aries beside it | clockwise | figure, tutorials and the application agree |
| East Indian | the **signs** | Aries in the top-centre cell | anticlockwise | figure and tutorials agree; **the application does not** (C47) |
| Nepali lotus | the **houses** | as North Indian | anticlockwise | the application's own drawing; no second source |
| Sudarshan Chakra | three rings of **houses** | house 1 at the top | the application draws clockwise | the ring order (lagna inner, Moon, Sun outer) agrees across tutorials; the direction is the application's alone (C47) |
| Western wheel | the **cusps** | the ascendant at nine o'clock | anticlockwise | every tutorial agrees |

Two corrections to ADR-0026 follow.

- **"Bengali" is not a row of its own.** Every source names the East
  Indian chart the Bengali, Odia or Assamese chart. The ADR listed both,
  so the shipped rows are six, not seven.
- **Layouts are two kinds, not one.** The ADR described "cells, their
  polygons in a unit square". That is true of the four square layouts,
  and false of the wheel and the chakra:
  - a Western wheel's houses are as wide as the chart's cusps make them,
    so its regions are computed per chart;
  - the chakra is three rings of sectors.

  The model below therefore has a **grid** kind (cells fixed in the
  row) and a **radial** kind (sectors computed from the chart).

And one refinement: the lotus's cells are not polygons. Its petals are
quadratic curves over the North Indian topology, so a cell's outline is
a path of line **and** quadratic segments.

## 3. The model

A layout is a row, as a varga scheme or a dasha system is. The unit
square is `[0, 1] × [0, 1]` with **y downwards**, the convention of
every drawing surface the renderer will target, so no consumer flips an
axis.

```text
Layout
  key            SCREAMING_SNAKE key (`NORTH_INDIAN`), catalogue kind `chart_layout`
  sources        citations, as every catalogue row carries
  shape          Grid | Radial

Grid                            (North, South, East Indian, the lotus)
  cells [12]     each: outline (a closed path: MoveTo, LineTo, QuadTo)
                       holds    Sign(Rashi) | House(1..=12)
                       label    the anchor of the sign or house number
                       bodies   the anchor the cell's bodies stack about
  frame          paths that are drawn but hold nothing (the border)

Radial                          (the chakra, the wheel)
  rings          each: inner and outer radius, the reference the ring
                       counts house 1 from (Lagna, Moon, Sun, Cusps)
  start          the angle house 1 begins at
  direction      Clockwise | Anticlockwise
```

`holds` is what makes the four square layouts one kind.

- A **sign-fixed** grid (South, East) puts a sign in each cell, and
  placement computes each cell's house from the lagna.
- A **house-fixed** grid (North, lotus) puts a house in each cell, and
  placement computes each cell's sign.

Either way the drawn cell carries **both** the sign and the house, so a
consumer never repeats the arithmetic. Getting it wrong is how the
application's `buildHousesFromVarga` came to warn about negative
remainders.

## 4. Placement

```text
place(layout, chart) -> Placed
  layout     the row
  chart      Placements: the lagna's sign, and each body's key, sign and
             longitude (a founded chart, or a divisional chart, which has
             its own lagna)

Placed
  layout     the key
  cells      each: outline, sign, house, label anchor, body anchor,
                   bodies (in catalogue order), whether it holds the lagna
  frame      the row's frame paths
```

- **A divisional chart places like a rashi chart.** It is `Placements`
  with its own lagna, which is the rule the application had to discover.
- **Bodies are listed, not positioned one by one.** How to stack four
  labels in a triangle is a drawing decision, which belongs to the
  renderer and its theme. Geometry gives the anchor and the order.
- **A radial layout places into sectors.** A chakra ring's twelve 30°
  sectors start at its reference sign. The wheel's houses run from cusp
  to cusp, and its zodiac ring is rotated so the ascendant's longitude
  sits at the start angle.
- **`Placed` serialises**, derives the document schema, and is what the
  chart document will carry (ADR-0026 §2, step 5 below).

## 5. Validation: a row is refused, not believed

Every row passes the same checks, shipped or registered, through the
registry's `Definition::validate`. A wrong cell is then a refusal naming
the cell, not a chart that looks almost right.

1. **Exactly twelve regions per ring**, each sign (sign-fixed) or house
   (house-fixed) exactly once.
2. **Every outline is closed and inside the unit square.**
3. **No two cells overlap**: no point of an offset 97×97 sample grid
   lies in two cells, and the cells' areas sum to no more than the
   square. Curves are flattened to a fixed number of segments, and the
   shoelace formula gives the areas. Building corrected the first
   version, which required the areas to *tile* the frame: the South and
   East Indian charts have an empty centre that holds nothing, so
   coverage is not a property a layout owes.
4. **Every anchor lies inside its own cell.**
5. **The declared direction is the drawn one**: taken in sign or house
   order, the cells' centroids turn around the centre in the row's
   direction.

## 6. What proves the shipped rows

Structure is not correctness, though building found it catches more
than this page first said. **Two adjacent signs swapped are refused** by
§5's direction rule, because the cells then step backwards once in sign
order. What passes every structural rule is **the whole ring turned by
one cell**: each sign held once, every anchor inside its cell, the
direction intact. That is exactly the mistake a transcribed layout makes.
So each square layout is also held to its **reference figure**. The number printed in each cell of `Kundali_01a/02a/03a.png`
is transcribed once, as a point in the unit square and the sign or house
that figure puts there. The test places a chart and asserts that each
point lies in the cell holding that number. A swapped sign is then a
failed point, found by name.

- **The lotus** is held to the North Indian figure's points, which is
  the claim that it shares North Indian's topology.
- **The Sudarshan Chakra and the wheel** have no numbered reference
  figure. They are held by §5 and by placement tests that compute a
  sector's angle from a known lagna.

## 7. Determinism

The renderer's output is gated byte for byte (ADR-0026). Geometry is
where that is won or lost.

- **The four grid rows** are rational coordinates, and placement only
  selects cells. There is no arithmetic that could differ between
  architectures.
- **The chakra's sectors** fall at multiples of 30°, whose sines and
  cosines are exact constants in a table, not calls to a platform's
  `sin`.
- **The wheel** is the one layout that needs a general `sin` and `cos`
  (a cusp at 17.3°). Its coordinates are rounded to a fixed grain before
  they leave the crate, and its placement joins the hash-matrix workflow,
  which already compares what three architectures compute.

## 7a. What building the chakra settled

- **A sector is bounded by arcs**, so an outline gained an arc segment
  (a centre, a direction, an end). Its endpoints are exact and only the
  validation flattens it.
- **A ring's start is a clock hour** (`starts_at: 12`). Every sector
  boundary is then a multiple of 30° and every label a multiple of 15°,
  whose sines and cosines are built from √2, √3 and √6 (`clock`). No
  platform `sin` reaches the chakra's output.
- **Placement can fail.** A ring counting from the Moon, for a chart that
  lists no Moon, is refused naming `graha.MOON`. Counting it from the lagna
  would draw a chakra that was not asked for.
- **The frame draws each shared circle once**: three rings have four
  circles, not six, because a stroke drawn twice darkens.
- **The ring order is the tutorials'** (lagna inner, Sun outer), which the
  application reverses. C47 records the disagreement.

## 7b. What building the wheel settled

- **The wheel needed no third shape.** It is a radial layout of two more
  ring kinds:
  - `Cusps`: houses from each cusp to the next;
  - `Zodiac`: twelve equal signs turned so the lagna's degree sits at the
    start hour.

  Its `starts_at` is 9 and it runs anticlockwise, the convention every
  source gives.
- **A body is in the house the chart's own `Bhavas` put it in.**
  `Placements` carries the chart's bhavas, not bare cusps, and the wheel
  asks them. The first version kept its own copy of the rule, which is the
  duplication that lets a wheel and a chart disagree about one graha's
  house.
- **A wheel draws bodies where they stand**, so `Placed` gained `marks`: a
  body, its ring, its point and the longitude that put it there. The
  grids leave it empty.
- **Placement input grew what a wheel reads**, each piece optional because
  a divisional chart has none of them: the lagna's degree, the bhavas, and
  each body's degree. A ring that needs a missing piece is refused by the
  field (`houses`, `lagna_deg`, `bodies[2].longitude_deg`).
- **Rounding, and a row to check it.** Coordinates reached through `sin`
  are rounded to 1e-9 of the square. A test holds every wheel coordinate to
  that grain, and `teistro-scenario`'s new `geometry` section places both
  radial layouts over a sweep of real Placidus cusps, so the hash matrix
  compares them across three architectures. The benchmarks walk the same
  section, so its cost is tracked too.

## 7c. The registry, and what the hash matrix said

- **`chart_layout` is catalogue kind 62**, with the six rows as members.
  Each is cited, and the chakra is marked `T` because its direction awaits
  a citation (C47). The bindings gain `ChartLayout` from the same
  generator as every other kind.
- **`Layouts` is the registry**, and the first consumer of core's
  `Registry` anywhere in the SDK. It holds the shipped rows and a
  consumer's own, which pass the same `Layout::validate`. A key the SDK
  ships is refused, so a registered row can add a layout and never
  replace one, and sealing stops registration once a context draws.
- **A test holds the shipped rows and the catalogue to one list, both
  ways**, and it failed the first time it ran. Two earlier edits to
  `rows::shipped()` had not matched after `rustfmt` reformatted it, so
  the chakra and the wheel were built, tested one by one, and never
  shipped. The list is now checked, not trusted.
- **The hash matrix agrees on three architectures.** The `geometry`
  section's 24 024 values hash to the same digest on Linux x86-64, Linux
  aarch64 and macOS aarch64, so the wheel's rounding holds on real
  hardware and not only by argument.

## 8. Order of work

1. **This page**, crux C47, and the corrections to ADR-0026 recorded
   there.
2. **`crates/geometry`**:
   - the model, the path type and §5's validation;
   - the four grid rows with their reference-figure tests;
   - placement from a founded chart and from a divisional one.
3. **The radial kind**: the chakra, then the wheel with its rounding and
   its hash-matrix row.
4. **The registry and the catalogue kind** `chart_layout`, so a consumer
   registers a regional layout without forking anything.
5. **The document section and the boundary**: `ChartRequest::with_layouts`,
   the section in all four bindings, and parity.
6. **`render-svg`**: the theme record and golden SVGs, gated byte for
   byte.

## 9. What this design does not settle

- **C47**, recorded in the cruxes register:
  - the East Indian drawing in the baseline application disagrees with
    the published figure;
  - the chakra's direction has only the application as its source.

  The SDK ships the figure's East Indian, and the chakra's direction as
  a declared, overridable property of its row.
- **Other regional rows** (Kerala, Sri Lanka): each is a registered row
  away, and none ships until something cites it.
- **Canvas, PDF and native drawing**: out of scope, as the ADR says.
  They consume the same geometry.
