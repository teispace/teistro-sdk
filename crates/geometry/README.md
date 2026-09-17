# `teistro-geometry`

Status: `built` (every layout ADR-0026 lists), 2026-09-14. The design is
[`docs/03-design/chart-geometry.md`](../../docs/03-design/chart-geometry.md),
over [ADR-0026](../../docs/08-decisions/adr-0026-chart-geometry-and-the-first-party-renderer.md).

Chart layouts as data, and a chart placed in one. The part of drawing that
is chart data: which region of the page belongs to which sign or house,
and where its label and its bodies go. The core still never draws.

| module | what it settles |
|---|---|
| [`path`](src/path.rs) | points and closed outlines in the unit square, y downwards, with the quadratic curves the lotus needs |
| [`layout`](src/layout.rs) | a layout as a row, and the checks that refuse a wrong one by the cell it gets wrong |
| [`rows`](src/rows.rs) | North, South and East Indian, each held to the figure it cites, the Nepali lotus, the Sudarshan Chakra and the Western wheel |
| [`registry`](src/registry.rs) | the shipped layouts and a consumer's own: the same checks, and a shipped key cannot be taken over |
| [`drawing`](src/drawing.rs) | one chart, founded or divisional, drawn in one layout, which is what a chart document carries; a pair that cannot be drawn is refused by name |
| [`clock`](src/clock.rs) | the directions a radial layout needs, built from square roots so they are bit for bit the same on every platform |
| [`place`](src/place.rs) | a chart placed in a layout, every cell carrying both its sign and its house |

## What the research found

- **Bengali is the East Indian chart**, not a row of its own.
- **The baseline application's East Indian view is not the published chart**
  (crux C47). The SDK ships the figure's.
- **The lotus is the North Indian chart drawn as petals.** It is parsed from
  the application's own path strings and snaps the source's rounded thirds
  through a stated table.
- **Layouts are two kinds.** The square charts are cells fixed in the row.
  The Sudarshan Chakra is rings computed per chart: three rings counted from
  the lagna, the Moon and the Sun, in the tutorials' order. The application
  draws the rings in the reverse order (crux C47). The Western wheel is a
  ring of houses between the chart's cusps inside a ring of the zodiac, the
  ascendant at nine o'clock, with each body marked at its own degree.

## How it is held

- **Validation.** Every shipped row passes the checks a consumer's would:
  - twelve cells, each sign or house once;
  - outlines inside the square;
  - anchors inside their cells;
  - no overlap, by sampling;
  - cells that run in the declared direction.
- **Refusals.** Each check is made to fire on a broken row, and the refusal
  names the field. Two adjacent signs swapped break the direction, and that
  check refuses them.
- **Figures.** A ring turned by one cell passes every structural check, so
  each square chart is also held to its reference figure: every number the
  figure prints must land in the cell showing that sign.
- **Placement.** It is checked for all twelve lagnas in all four grid
  layouts. For the chakra, each ring counts from its own place, house 1
  begins just clockwise of twelve o'clock, no two sectors overlap, every
  anchor is inside its sector, and a chart with no Moon is refused by name
  rather than counted from the lagna.
- **The wheel.**
  - The ascendant's cusp is exactly at nine o'clock, and houses run
    anticlockwise.
  - A body is in the house the chart's own `Bhavas` put it in: the wheel
    asks them rather than keeping a second rule.
  - A sector is as wide as its cusps, the zodiac's signs are equal, and each
    mark sits at its body's degree.
  - A chart without cusps, a lagna degree or a body's degree is refused by
    the field.
- **Determinism.** The wheel is the one layout that uses a platform's `sin`,
  so its coordinates are rounded to a grain, and a test checks every one
  sits on it. The `geometry` section of `teistro-scenario` places both
  radial layouts over a sweep of real Placidus cusps, so the hash matrix
  checks the rounding holds on three architectures. It did: the same
  digest on Linux x86-64, Linux aarch64 and macOS aarch64.
- **The list.** A test holds `rows::shipped()` and the `chart_layout`
  catalogue kind to the same six keys, both ways. The first run caught the
  chakra and the wheel missing from the shipped list.
- **Across the boundary.** A request's drawings cross as packed
  `layout << 16 | varga` ids, and the placed charts come back as a canonical
  JSON section of the charts blob. Node, Python and Dart each give them a
  typed shape, and the four parity runners agree on every coordinate
  ([§7e](../../docs/03-design/chart-geometry.md)).
- **A consumer's own layout, from any binding.** A context's `layouts` take
  rows as `sdk.chart.layout(key)` answers them, read strictly (every key must
  be one the reader took) and checked by the rules a shipped row passes. A
  registered layout is drawn by its full key, and the four parity runners
  register and draw the same one ([§7f](../../docs/03-design/chart-geometry.md)).
