# ADR-0026: Chart geometry as data, and a first-party SVG renderer

Status: accepted (maintainer, 2026-09-10)
Date: 2026-09-10
Question: Q36

## Context

`01-research/feature-universe/18-interpretation-rendering.md` put chart
layouts at P0 and drew a line: "the SDK ships geometry and data; it never
draws. A separate optional rendering package per platform (SVG for web,
canvas for Flutter) can be built on top later." The roadmap then carried
chart geometry in Phase 9, the release-engineering phase.

Two pieces of evidence say that gap is the wrong way round.

1. **It is what a chart library is compared on.** The leading Western
   library's headline feature is SVG output in two styles and six themes.
   A developer evaluating dependencies renders a chart; if nothing comes
   out, the evaluation ends there, whatever the depth behind it.
2. **The baseline engine's own field research** names the traditional
   print format as its single biggest adoption driver — ahead of every
   AI feature — and its practitioners reject a system whose output "does
   not look like a real kundali". That is a rendering requirement, and it
   was learned from practitioners rather than guessed.

The baseline engine resolved this by stopping its backend at structured
placements (`signIndex` per body per varga, "structural chart-grid data,
no interpretive prose") and rendering through Handlebars templates and a
pooled headless browser. Calculation and presentation are fully decoupled
there, and new regional formats are template files rather than code
changes. That separation is right and is kept.

What it does not give is a chart in a language-neutral core, and it is a
server-side browser — unavailable to wasm, to Flutter and to anyone
embedding the SDK.

## Decision

Three parts, in order.

**1. Layouts are data, not code.** A `geometry` module holds a layout
registry in the manner this project already uses for dasha systems and
varga schemes: each layout is a cited row describing its cells, their
polygons in a unit square, which sign or house each cell carries and
whether that assignment is fixed or rotates with the lagna, and where
labels and bodies sit within a cell. North Indian, South Indian, East
Indian, Bengali, the Nepali lotus (Ashtadala Padma) and the Western
wheel are rows. **A consumer registers their own regional layout as a
row, without forking anything** — the same extension shape as a
consumer-registered dasha system.

**2. Geometry moves early.** Cell assignment is chart data, not release
engineering, so it lands in Phase 4 beside `chart`, `houses` and
`vargas`, and its output is part of the serialised chart. Anyone can
draw from it in any language, which is what keeps the core honest.

**3. A first-party SVG renderer ships.** `render-svg` is a Rust crate
that turns geometry plus a theme into an SVG string. Rust, so it reaches
every binding and wasm for free (Q25); optional, so a consumer who draws
their own ships nothing; and **themeable as data** — colours, strokes,
fonts, glyph sets, script and numeral system, cell ornament, and which
of the chart's values appear in a cell are all a theme record, never
hardcoded. Its output is deterministic, byte-stable and gated like every
other generated artefact.

The line from `feature-universe/18` survives intact: the *core* still
never draws. `render-svg` is a separate optional module above the core
that consumes the same public geometry a third party would.

## Consequences

- A developer can render a chart from the SDK alone, in any binding, on
  the first evaluation.
- The Nepali lotus becomes reachable, which no competitor ships and which
  the baseline engine's market requires.
- Layout rows need citations and reference images like any other row, and
  a wrong cell is now a gate failure rather than a styling opinion.
- SVG determinism needs a gate: float formatting, attribute order and
  path rounding are all sources of drift between architectures.
- Canvas, PDF and platform-native drawing stay out of scope. They consume
  the same geometry; the SDK does not grow a print pipeline.

## Alternatives considered

**Geometry only, no renderer** — holds the architectural line most
strictly and leaves every consumer to write a renderer. Rejected: it
leaves the comparison the SDK is judged on unanswered, and every consumer
rebuilding the same SVG is the duplication this SDK exists to remove.

**Leave both in Phase 9** — no change; rejected because it puts a P0
capability behind every other phase.

**Per-platform renderers (SVG for web, canvas for Flutter)** — as the
research page originally imagined. Rejected for v1: one Rust renderer
emitting SVG reaches every binding at once, and SVG renders in web,
Flutter and native alike. Platform-native drawing can follow later
without changing the geometry contract.

## Evidence

`01-research/feature-universe/18-interpretation-rendering.md`;
`01-research/competitive-analysis/02-developer-market.md`; the baseline
engine's blueprint sections 36.1 to 36.5 (traditional format, chart
styles, template-driven architecture) and its own render-section
contract, read 2026-09-10.
