# `teistro-geometry`

Status: `built` (the grid layouts and the Sudarshan Chakra), 2026-09-14. The design is
[`docs/03-design/chart-geometry.md`](../../docs/03-design/chart-geometry.md),
over [ADR-0026](../../docs/08-decisions/adr-0026-chart-geometry-and-the-first-party-renderer.md).

Chart layouts as data, and a chart placed in one. The part of drawing that
is chart data: which region of the page belongs to which sign or house,
and where its label and its bodies go. The core still never draws.

| module | what it settles |
|---|---|
| [`path`](src/path.rs) | points and closed outlines in the unit square, y downwards, with the quadratic curves the lotus needs |
| [`layout`](src/layout.rs) | a layout as a row, and the checks that refuse a wrong one by the cell it gets wrong |
| [`rows`](src/rows.rs) | North, South and East Indian, each held to the figure it cites, the Nepali lotus, and the Sudarshan Chakra |
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
  draws the rings in the reverse order (crux C47). The Western wheel, whose
  houses are as wide as the cusps, is next.

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
