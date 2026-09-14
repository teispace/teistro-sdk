# `teistro-render-svg`

Status: `built`, 2026-09-14. The design is
[`docs/03-design/render-svg.md`](../../docs/03-design/render-svg.md), over
[ADR-0026](../../docs/08-decisions/adr-0026-chart-geometry-and-the-first-party-renderer.md).

A placed chart, its labels and a theme to a byte-stable SVG. It sits above
the core and reads only what a third party could: a `Placed` chart from
`teistro-geometry`, the strings to write, and a style. It knows no
astrology and no locale.

| module | what it settles |
|---|---|
| [`theme`](src/theme.rs) | `Style` (how a drawing looks), `Content` (what it says) and the `Theme` that carries both; every field defaults, and an unknown one is refused |
| [`labels`](src/labels.rs) | the strings a drawing prints, checked against the chart: one per cell, every body named, nothing XML cannot carry |
| [`svg`](src/svg.rs) | numbers rounded through an integer, escaping, text presentation for the zodiac signs, and outlines as path data |
| [`fit`](src/fit.rs) | a cell's labels arranged inside its outline and off its label, without font metrics |
| [`marks`](src/marks.rs) | a wheel's bodies at their degrees, a conjunction spread round the ring, and a tick at each true degree |

## What the research found

- **Presentation attributes only.** Flutter's SVG renderer does not fully
  support CSS and does not honour `dominant-baseline`. The output has no
  `<style>`, and places each baseline itself.
- **Signs are text, not emoji.** The zodiac signs have
  `Emoji_Presentation=Yes`, so each is followed by U+FE0E. So are ♀ and ♂,
  which also define a text-style sequence, and no other character.
- **No glyph table.** Every label comes from a locale's vetted `short` or
  `glyph` form, composed by the façade.

## What proves it

- Every shipped layout is drawn in Latin and Devanagari, in both styles.
  Each drawing is parsed back as XML: one group per cell with its sign and
  house, every body written once in its own name, the lagna first in its
  own cell, and no stylesheet.
- The same chart draws the same bytes. A crowded cell draws smaller than a
  roomy one, and a box never leaves a triangle.
- The golden drawings in [`golden/`](golden) are one real chart in every
  layout, in Nepali and English, gated byte for byte by `cargo xtask
  check-render`. The hash matrix's `render` section compares the bytes
  on three architectures.
- Numbers print as hundredths with no trailing zeros, arcs carry the right
  flags, and a style or labels are refused by the field they get wrong.
- A request's `theme_json` writes every drawing as SVG at the boundary.
  Node, Python and Dart read it back as `drawing.svg`, and the four parity
  runners agree on every byte.
