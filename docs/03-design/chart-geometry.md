# Chart geometry: layouts as data, and what places a chart in one

Status: `built` — steps 1 to 6 built 2026-09-14 (the renderer in [`render-svg.md`](render-svg.md)); §7f, a consumer's layout from a binding, built 2026-09-15; §7g, the Nepali lotus and round chart redrawn from printed janma patrikas, built 2026-11-10; §7h, North Indian drawn with petals, built 2026-11-10; §7i, the Moon's chart and the bhava chalit as drawings, built 2026-11-11. It builds the first two parts of
[ADR-0026](../08-decisions/adr-0026-chart-geometry-and-the-first-party-renderer.md)
(layouts are data, and geometry lands in Phase 4). The third part, the
SVG renderer, is built ([`render-svg.md`](render-svg.md)). The research this page rests on was
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

A fourth was read for the Nepali charts on 2026-11-10 (§7g): three
printed janma patrikas, photographed, two of one printer's form and one
of another's. A fifth, for North Indian drawn with petals (§7h): a
consumer's app's kundali view, one screenshot read the same day.

| layout | what is fixed | where it starts | direction | agreement |
|---|---|---|---|---|
| North Indian | the **houses** | house 1 is the top diamond | anticlockwise | figure, tutorials and the application agree |
| South Indian | the **signs** | Pisces top-left, Aries beside it | clockwise | figure, tutorials and the application agree |
| East Indian | the **signs** | Aries in the top-centre cell | anticlockwise | figure and tutorials agree; **the application does not** (C47) |
| Nepali lotus | the **houses** | house 1 the top petal | anticlockwise | printed janma patrikas, three scans, 2026-11-10: two draw the lotus, and agree |
| Nepali round chart | the **houses** | house 1 the cell above the hub | anticlockwise | printed janma patrikas, three scans, 2026-11-10: the third draws four round charts, and they agree |
| North Indian, petals | the **houses** | as North Indian | anticlockwise | a Nepali app's kundali view, one screenshot, 2026-11-10; North Indian's cells, so the North Indian sources agree on the order |
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
lines and quadratic curves, and the round chart's cells end on arcs, so
a cell's outline is a path of line, quadratic **and** arc segments.

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

Grid                            (North, South, East Indian, the Nepali charts, the petals)
  cells [12]     each: outline (a closed path: MoveTo, LineTo, QuadTo, ArcTo)
                       holds    Sign(Rashi) | House(1..=12)
                       label    the anchor of the sign or house number
                       bodies   the anchor the cell's bodies stack about
  frame          paths that are drawn but hold nothing (the border)
  hub            optional: a circle no cell holds, where the chart's name
                 is written (the round chart's ल or न)

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

- **The two Nepali charts** are held to the printed patrikas they are
  drawn from, the same way: each number a photographed patrika prints is
  transcribed as a point and the sign it names (§7g).
- **The Sudarshan Chakra and the wheel** have no numbered reference
  figure. They are held by §5 and by placement tests that compute a
  sector's angle from a known lagna.

## 7. Determinism

The renderer's output is gated byte for byte (ADR-0026). Geometry is
where that is won or lost.

- **The three square grid rows** are rational coordinates, and placement
  only selects cells. There is no arithmetic that could differ between
  architectures.
- **The petalled North Indian** is dyadic fractions (37/128, 9/64,
  7/256) and their halves and complements, all exact, drawn once and
  turned a quarter at a time by `(x, y) → (y, 1 − x)`, which is exact
  too. A test holds the right half of every curve to the left half's
  mirror, bit for bit.
- **The two Nepali rows** are clock directions (§7a) times radii in 32nds,
  and one correctly rounded square root (where the hub's side lines meet
  the circle). IEEE 754 fixes both.
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

## 7d. The document section, in Rust

- **A request names drawings as pairs**: `with_drawings([(layout, varga)])`,
  where `D1` is the founded chart. A list of layouts times the vargas asked
  for would multiply what nobody wanted, and `with_everything` draws
  nothing, since every layout times every chart is 126 placements.
- **A pair that cannot be drawn is refused by its place in the request.**
  A Western wheel of the D9 is refused as `drawings[1].varga`, naming both
  and hinting at a grid or the chakra. It is not refused by the missing
  degree it would otherwise fail on.
- **The layout is a `KeyId`**, so a shipped `ChartLayout` and a consumer's
  own are one type. The context owns a sealed `Layouts`, and
  `ContextBuilder::layout` registers into it under the same checks,
  refusing a shipped key at build.
- **`Document.drawings`** carries each drawing as the varga and the placed
  chart. `teistro::geometry` and `Drawing`, `Layout`, `Layouts` and `Placed`
  are re-exported by the façade.
- **What building found:**
  - The schema's numbered-name guard refused the first build: geometry's
    `Point` met the catalogue's `Point` (the derived points) in one
    namespace. Its schema name is now `UnitPoint`.
  - The document samples gained a North Indian and a Western wheel drawing,
    so the round trip and the schema see straight edges, arcs, marks and
    rounded coordinates, and both read back byte for byte.
- **Tested against real founded charts** in the façade:
  - D1 in North Indian matches the foundation's signs;
  - D9 in South Indian matches the navamsha's own lagna and signs;
  - each graha's wheel house equals the foundation's own bhava.

## 7e. The boundary and the bindings

- **A drawing crosses as one `u32`**: `layout << 16 | varga`, in
  `TsChartRequest.drawings` with its `drawing_count`. Two parallel arrays
  could disagree in length and a struct array would be a second record
  every binding generates; one packed id can do neither, and both halves
  are catalogue ids that fit 16 bits. No binding writes the bits by hand:
  each takes pairs (`{ layout, varga }` in Node, a `(ChartLayout, Varga)`
  tuple in Python and a record in Dart) and packs them in one helper.
- **The section is canonical JSON, not columns.** A drawing is nested
  paths of three segment kinds, so the charts blob carries it as bytes
  section 21, one array per chart, from the same canonical writer the
  document envelope uses. Each binding parses a batch's section once, on
  first read, and every chart in the batch reads from that.
- **Each binding gives the layer's shape**: catalogue keys in full
  (`rashi.LEO`, `varga.D9`), camel case where the binding uses it, and the
  segments as a typed union (a TypeScript union, Python dataclasses, a
  Dart sealed class), so a renderer's `switch` is checked.
- **A malformed pair is refused before the boundary**, naming its place
  (`drawings[1].layout`). Python's first version unpacked the pair before
  checking it, so a one-element tuple raised `ValueError` rather than the
  SDK's refusal; it now checks the shape first, and a test in Node and in
  Python holds every refusal to its field. Dart's record type makes the
  wrong pair a compile error.
- **Parity**: every runner requests the same three drawings (North
  Indian D1, South Indian D9, the wheel's D1) over two instants and print
  every cell's sign, house, lagna flag, ring, bodies, label, anchor and
  outline, and every mark. The runners agree on every value.
- **Across the boundary**: a consumer's own layout, registered from any
  binding (§7f), as the Rust façade registers any layout.

## 7f. A consumer's own layout, from any binding

ADR-0026 promises that a consumer registers a regional layout "without
forking anything". Rust could since §7c. This section extends that to the
bindings.

- **Registered when the context is made.** `TsContextOptions.layouts_json`
  is a nullable JSON array of rows, and the context seals its layouts once
  built, as `ContextBuilder::layout` does. A row is refused by its place
  in the array and its own field (`layouts_json[0].shape.cells[3].outline`).
  A key the SDK ships is refused, so a row adds a layout and never
  replaces one.
- **A row is read strictly.** Every key a row names must be one the reader
  took. A misspelt `start_at` that silently kept the default would draw a
  different chart from the one written. The geometry types cannot deny
  unknown fields themselves, because an internally tagged enum hands its
  tag to the variant. So the reader compares the keys it was given against
  the keys it read back and refuses the first extra key by its path.
- **Start from a shipped row.** `ts_chart_layout_row` answers any layout the
  context knows as its JSON row. The flow a consumer takes is: read the
  North Indian row, rename it, change what differs, and register it.
- **Referenced by key.** A drawing still crosses as packed ids. `sdk.keys`
  now resolves a registered member as well as a catalogued one, and a
  binding asks it for each registered layout's id **once, when the context
  is made**. It never recomputes a registry's numbering, a request never
  crosses the boundary again to look a layout up, and a key the context did
  not register is refused before anything crosses. `sdk.chart.layout(key)`
  answers a row in Rust as in the bindings.
- **Typed in each binding**, with the row's field names as the SDK spells
  them, as the theme record's are, because both cross as JSON verbatim:
  - Node: interfaces, and a drawing's `layout` that takes a `ChartLayout`
    or a `chart_layout.*` key.
  - Python: `TypedDict`s, and `Union[ChartLayout, str]`.
  - Dart: classes, and a generated `KeyOf<K>` interface on every catalogue
    enum with `Registered<K>` beside it. `(ChartLayout.northIndian,
    Varga.d1)` still compiles, and a registered layout is a
    `KeyOf<ChartLayout>` that a `Graha` can never be passed as.
- **Parity** registers the same row in every runner and draws it.
- **What building found:**
  - **Undefined behaviour at the boundary, older than this work.**
    `ts_chart_found` documented "null with a count of zero for none" for its
    arrays and called `slice::from_raw_parts(null, 0)`, which Rust forbids
    even for an empty slice. The bindings pass non-null empty pointers, so
    nothing had tripped it. The first test to pass exactly what the header
    allows aborted a debug build. Every array at the boundary is now read
    through one `support::slice`, which answers the empty slice without
    touching the pointer.
  - **A registered layout could not be read back.** Python's
    `Drawing.layout` refused a key it did not catalogue, and Dart's mapped
    it to `unknown`, losing the key. Python's is now a `ChartLayout` or the
    full key, with `layout_key` for a caller that wants either as text;
    Dart's is a `KeyOf<ChartLayout>`, `ChartLayout.registered('ACME_KERALA')`
    for one of the consumer's own.
  - **A refusal did not say which row.** The builder registered its layouts
    without naming their places, so two rows under one key were refused as
    `key`. The builder now names `layouts[1].key`, and the boundary calls
    that root `options.layouts_json`.
  - **The boundary had a second copy of the key lookup.** `ts_key_parse`
    and `ts_key_name` resolved against the catalogue directly, beside the
    façade's `sdk.keys`. Both now call the façade, which is what makes a
    registered member reach every binding from one place.
  - **The surface-areas page caught a module reached from two areas.** The
    first binding code resolved a registered key through the key module on
    every `foundMany`, so `key` was reached from `chart` as well as `keys`,
    and the page's rule that a module has one area turned `falsified`.
    Resolving the ids once at context creation keeps the rule and saves a
    crossing per request. The same page caught `chart.layout` missing from
    every parity runner's surface list.
  - **A misspelt required field** is refused as the field that is missing,
    before the unread-key check runs. The message names it, and the field
    is the row's root.

## 7g. The Nepali charts, from printed janma patrikas (SR-36)

A consumer asked for the Nepali lotus to match printed janma patrikas,
and sent three, photographed. The scans are private and are not in the
repository: only the charts' geometry and their numbers were transcribed,
and the placement test uses the charts' pattern without any birth data.

- **The lotus the print draws is not the one the SDK drew.** Two patrikas
  (one printer's form, headed जन्मकुण्डली, चलितकुण्डली and राशिकुण्डली)
  draw twelve equal petals radiating from the centre inside a circle in a
  decorated square. House 1's petal is at twelve o'clock and the numbers
  rise anticlockwise, a clock hour apart: North Indian's houses in North
  Indian's order, on a different figure. The row it replaces drew the
  North Indian diamonds with curved edges, and had only the baseline
  application behind it, so the print replaces it under the same key.
- **The petal.** A petal shares with its neighbour the line half an hour
  off its axis for its inner half, as the printed petals touch, then
  curves to a tip on the circle along a quadratic pulled towards that
  line; the background shows between the tips. It is widest (0.064 of
  the square) about 0.27 from the centre, which is where its bodies go;
  its number sits towards the tip. The first scan's numbers stand within
  5° of their petal's axis at about 0.55 of the radius.
- **The round chart is a square grid, not twelve wedges.** The third
  patrika (another printer) draws four round charts, ल, रा, न and भा,
  each a circle cut by the lines of a central square's sides and by the
  square's diagonals, run out to the circle: the East Indian topology
  clipped to a circle, but houses fixed, house 1 above the square and
  anticlockwise. Measured from the photograph, the square's half-side is
  0.205 of the radius down and 0.209 across, so the row takes a fifth;
  an edge cell spans about 23° of the circle and a corner cell about 34°.
  It ships as a new row, `NEPALI_CIRCLE`, catalogue id 6.
- **The hub names the chart.** The print writes the chart's initial in a
  small circle in the square. A grid gained an optional `hub` (a centre
  and a radius) that validation holds clear of every cell, `Placed`
  carries it, and the renderer writes `Labels.hub` there in the accent,
  shrunk to fit. The façade says the lagna's short form for D1 (ल, `As`)
  and a divisional chart's short form, which only the navamsha has (न,
  `D9`); other vargas are written by name. The रा chart (the Moon's) and
  the भा chart (the bhava chalit) are not drawings the SDK makes yet.
- **A photograph is foreshortened.** Its circle is an oval, so each
  figure is measured across and down separately, from the chart's own
  centre (where the petals meet, or the hub's centre), and scaled to the
  row's circle. Every printed number lands in its cell; the first scan's
  grahas (the lagna in sign 1, three in 2, one each in 3, 4 and 9, three
  in 5) land in the cells the patrikas write them in, in both charts.
  Turned by one petal, the lotus passes every structural check and fails
  its patrika.
- **The default is not changed.** Which layout a Nepali reader sees is
  the consumer's setting; the SDK's default stays North Indian.
- **Reckoning.** The second patrika notes its chart was reckoned by the
  Surya Siddhanta. That is the chart's computation, not its drawing, and
  changes nothing here.

## 7h. North Indian drawn with petals (SR-36)

The consumer sent a screenshot of a Nepali app's kundali view: North
Indian, with the same fixed houses (house 1 the top kendra,
anticlockwise) and a number in each cell, drawn with petals. It ships as
a new row, `NORTH_INDIAN_PETALS`, catalogue id 7. Only the geometry and
the numbers were transcribed.

- **The cells are North Indian's.** The same houses and the same label
  and body anchors; only the outlines change. In the screenshot's 550 px
  square every number stands within 0.005 of the side of North Indian's
  label anchor, and all twelve land in their cells.
- **A kendra's edges are ogee curves.** Each runs from a side's midpoint
  to a vertex on a corner diagonal, measured at 0.291 of the side from
  the corner, where plain North Indian meets it at 0.25; the row takes
  37/128. A curve leaves the midpoint at right angles to the side,
  flattens about 0.142 in (the row takes 9/64) and turns down onto the
  vertex. It is two quadratic steps: midpoint to the flat's middle,
  pulled to `(0.5, 9/64)`, then on to the vertex, pulled to the point
  above it on the flat. The two steps share a tangent where they meet, so
  the curve has no corner. A corner cell is bounded by one such curve,
  two sides of the square and its diagonal.
- **The diagonals stop at a swastika.** The screenshot's diagonals do not
  meet: each ends at the tip of an arm of a small swastika, about 0.055
  of the side across (the row's half-side is 7/256), whose arms run from
  the centre to the middle of each side of its square and turn onto a
  corner. The arms are the kendras' inner edges, so the hub needs no
  ornament of its own and no text: each kendra takes the quarter of the
  hub its arm opens towards, and the twelve cells still fill the square.
  The swastika turns one way, so the cells are not mirror images; the
  curves are.
- **Drawn once.** The top kendra and the two cells left of it are written
  out, and every other cell is one of them turned a quarter, a half or
  three quarters (§7).

What the screenshot also writes, the renderer gained as theme options
whose defaults keep every drawing's bytes ([`render-svg.md`](render-svg.md)
§3): a colour per body (`style.body_colours`), the retrograde mark before
the name (`content.retrograde_place`, `*श`), the nodes marked too
(`content.mark_nodes`), and the minute after the degree in the locale's
digits (`content.minutes`, `र २६°३२'`). The view writes no space between
the name and the degree; the SDK keeps its one.

## 7i. What a drawing counts from (SR-36)

The printed janma patrikas draw the founded chart more than once: the
lagna's chart (ल), the Moon's (रा) and the bhava chalit (भा), each in the
round layout with its name in the hub. A drawing is therefore a layout,
a varga and a frame, the new catalogue kind `chart_frame`:

| Frame | id | Houses count from | Grahas stand by | Source |
|---|---|---|---|---|
| `LAGNA` | 0 | the lagna's sign | their signs | every scan |
| `MOON` | 1 | the Moon's sign | their signs | the scans' रा charts |
| `SUN` | 2 | the Sun's sign | their signs | BPHS's Sudarshana chakra, its middle ring |
| `BHAVA` | 3 | the lagna's sign | the bhava the cusps put them in | the scans' भा charts |

- **A frame moves the houses, not the signs' contents.** In a Moon or Sun
  chart every sign holds what it holds in the lagna's chart; house 1 is
  the Moon's or the Sun's sign, and the lagna flag stays on the lagna's
  own sign, as the scans mark it.
- **The bhava chalit keeps the houses.** Each cell keeps the sign the
  lagna's chart gives it, and each graha is placed by
  `houses.place(longitude).bhava` under the profile's house system: a
  graha before its sign's cusp moves to the house behind.
- **A grid only.** A radial layout's rings already count from the Moon
  and the Sun, so a non-lagna frame on one is refused under `from`.
- **The lagna's frame is not written.** `Drawing.counts_from` is skipped
  when it is `LAGNA`, so every existing drawing keeps its bytes; requests
  spell it `countsFrom`. Across the C ABI each drawing is one `u32`,
  `layout << 16 | frame << 8 | varga`, so an old packing reads as before.
- **The hub names the frame.** A founded chart's hub writes its frame's
  short form (ल, रा, सू, भा in Nepali; As, Mo, Su, Bh in English), a
  divisional one its varga's.

Every binding takes the frame beside the layout and varga (Rust
`ChartDrawing`, Node `{ layout, varga, countsFrom }`, Python a triple,
Dart `ChartDrawing(..., countsFrom:)`, Java `drawing(layout, varga,
frame)`) and reads it back on each drawing.

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
5. **The document section and the boundary**: `ChartRequest::with_drawings`,
   the section in every binding, and parity (built: §7d, §7e).
6. **`render-svg`**: the theme record and golden SVGs, gated byte for
   byte (built, [`render-svg.md`](render-svg.md)).

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
