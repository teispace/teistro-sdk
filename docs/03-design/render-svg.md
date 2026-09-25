# The SVG renderer: a placed chart and a theme to a byte-stable drawing

Status: `built`, designed and built 2026-09-14. This is step 6 of
[`chart-geometry.md`](chart-geometry.md) §8, and the third part of
[ADR-0026](../08-decisions/adr-0026-chart-geometry-and-the-first-party-renderer.md).

## 1. Purpose and scope

A developer evaluating the SDK renders a chart. `crates/geometry` already
gives every cell's outline, the sign and house it shows and the bodies in
it, in every binding. This page decides how that becomes an SVG string
that looks like a real kundali, reads in the reader's script, and is
the same bytes on every platform.

In scope:

- the theme record;
- the labels a drawing prints, and who composes them;
- the subset of SVG the output uses;
- how text is fitted into a cell without font metrics;
- determinism, and the gate that proves it.

Out of scope, as the ADR says: canvas, PDF and native drawing, and
interaction. The output carries the data attributes an interactive
consumer needs; the SDK does not handle events.

## 2. What the research found

Four kinds of source were read.

1. **The baseline application's kundali components**, which draw North,
   South, East, lotus and chakra views to readers today.
   - A cell prints the sign's number in the reader's numerals.
   - It stacks two-glyph graha abbreviations vertically about the
     cell's anchor, centred on the stack's middle.
   - Optional per-graha additions: a retrograde marker before the name,
     the degree after it, a combustion marker.
   - A reader chooses Devanagari, Latin or symbol glyphs.
   - The lagna's abbreviation is drawn in the accent colour.
2. **The SDK's own locales already carry every label a cell prints.**
   - `graha.*` and `rashi.*` have a `short` form (`Su`, `सू`) and a
     `glyph` form (`☉`, `♈`).
   - `point.LAGNA` has `short` (`As`, `ल`).
   - `intl::render::digits` gives each CLDR numbering system's digits.

   The renderer needs no glyph table of its own, so no label enters the
   SDK that a locale has not vetted.
3. **Unicode's emoji data** (`emoji-data.txt` and
   `emoji-variation-sequences.txt`, version 17.0).
   - The twelve zodiac signs U+2648–2653 have
     `Emoji_Presentation=Yes`: a browser or phone draws them as coloured
     emoji unless they are followed by U+FE0E.
   - ♀ and ♂ are emoji with text presentation by default, and a
     text-style sequence is defined for them too.
   - ☉ ☽ ☿ ♃ ♄ ☊ ☋ have no variation sequence at all.
   - So the renderer appends U+FE0E after exactly those fourteen
     characters, and no others.
4. **What SVG renderers support.**
   - Flutter's `flutter_svg` asks for presentation attributes rather than
     CSS, "because CSS is not fully supported", and its issue #561
     records `dominant-baseline` as not working.
   - `resvg`'s unsupported list names no text-anchoring property, but
     drops font elements and external `use`.

   A drawing that must render the same in a browser, in Flutter and in a
   server-side rasteriser therefore uses presentation attributes only,
   and places each text's baseline itself.

## 3. The model

```text
Placed  +  Labels  +  Style  ──render──▶  String (SVG)
Document + Context + Content ─compose──▶  Labels
```

- **`Style`** is how the drawing looks, and the renderer's only input
  besides the geometry:
  - the drawing's size in user units;
  - the background, ink, cell fill and lagna-cell fill colours;
  - the stroke width;
  - the font family;
  - the body, label and mark sizes as fractions of the size;
  - the baseline shift, as a fraction of an em.
- **`Content`** is what the drawing says. The façade reads it to compose
  the labels, and the renderer never sees it:
  - which form a body is written in: `SHORT` or `GLYPH`;
  - what a cell's label shows: its sign, its house or nothing;
  - whether a retrograde body is marked, and with which string;
  - whether the degree is written.
- **`Theme`** is one `Style` and one `Content`, serialised together, so
  a consumer stores and passes one record. `Theme::light()` and
  `Theme::dark()` ship.
- **`Labels`** are the strings a drawing prints, already in the reader's
  script:
  - one per body key;
  - one per cell's label;
  - an optional title for the drawing's accessible name.

  A body the labels do not name is refused by its key rather than drawn
  blank.

Composing the labels in the façade and not in the renderer keeps the
renderer free of astrology and of the intl engine. It is a crate that
turns geometry and strings into SVG, which a third party could have
written from the public geometry, as the ADR requires. Because the
composing is in Rust, every binding gets the same labels.

## 4. The SVG the renderer writes

- `<svg xmlns viewBox width height role="img">`, with a `<title>` when
  the labels carry one.
- One background `<rect>`.
- For each cell, a `<g>` carrying `data-sign`, `data-house` and, on the
  lagna's cell, `data-lagna`. It holds:
  - the cell's `<path>`;
  - its label's `<text>`;
  - one `<text data-body>` per body.
- The frame's paths, stroked and unfilled.
- For a layout with marks (the wheel), one `<line data-body>` tick and one
  `<text data-body>` per mark (§6), and no bodies stacked in cells, since
  the marks already draw every body once.

Outlines map one to one:

| geometry | SVG |
|---|---|
| a start | `M` |
| `Line` | `L` |
| `Quad` | `Q` |
| `Arc` | `A` with the radius from the start to the centre, and the flags from the direction and the swept angle |
| the implied close | `Z` |

What is not used:

- `<style>` and classes;
- `dominant-baseline`;
- `<defs>`, `<use>`, markers and filters.

Attributes are written in one fixed order, and text is XML-escaped.

## 5. Determinism

- **Numbers.** Every coordinate is the unit square's, scaled by the size
  and rounded to 0.01 of a user unit through an integer, then printed
  with its trailing zeros removed and `-0` as `0`. The unit square's
  own values are exact rationals for the grids and on a 1e-9 grain for
  the wheel, so no platform difference reaches the second decimal.
- **Arc radii** use `sqrt`, which IEEE 754 requires to be correctly
  rounded.
- **Text fitting** (§6) is arithmetic and fixed-count bisection, with no
  font metrics and no platform call.
- **The gate.** Golden drawings of real charts are checked in and compared
  byte for byte (§7). The hash matrix gains a `render` section, so the
  bytes are compared on three architectures and not only on the machine
  that blessed them.

## 6. Fitting text into a cell without metrics

A renderer that measures glyphs gets a different answer from every font.
This one estimates instead, and the estimate is part of the style.

1. **The box.** The largest axis-aligned box centred on the cell's
   anchor that lies inside the outline, found by bisecting its scale for
   a fixed number of steps. A candidate box is tested by sampling its
   edges against `Path::contains`.
2. **A label's width** is its count of Unicode scalar values times the
   style's advance, in ems. That overestimates a Devanagari conjunct,
   which errs towards a smaller font rather than towards overflow.
3. **The arrangement** is the column count, from one upwards, that
   allows the largest font for the bodies in the box, capped at the
   style's body size. Bodies fill columns top to bottom in the cell's
   order.
4. **The cell's own label is kept clear.** A candidate box that overlaps
   the label's estimated box does not fit, so a crowded cell spreads into
   more columns or shrinks rather than writing over its sign number.
5. **The baseline** of a line centred at `y` is `y + shift × size`.

A wheel's bodies are not stacked; each is drawn at its degree (§4).

- **The radius.** A mark is written at the radius of its ring's anchors,
  read off the placed cells. The house numbers sit nearer the centre, so
  the two do not collide.
- **Spreading.** A conjunction is the common case. Marks closer round the
  ring than their words need are one cluster, laid out evenly about the
  cluster's mean bearing, in order. Clusters that spread into each other
  merge and are laid out again, until nothing moves. Marks exactly one
  separation apart still touch, which is what lets a spread cluster merge
  whole rather than split into pairs.
- **The room** a mark needs is the wider of its widest label's estimated
  width and its line height, with a little air.
- **A tick** at each body's true degree, inside the ring's outer edge,
  shows where a spread body really stands. It is found by scaling along
  the body's own radius, so it takes no trigonometry.
- **Determinism.** The spreading needs `atan2`, `sin` and `cos`. As the
  wheel's own placement does (`chart-geometry.md` §7), every such value is
  rounded to the geometry's 1e-9 grain, and the hash matrix's `render`
  section checks it.

## 6a. What building found

- **Three test premises were wrong, and the renderer was right each
  time.**
  - The chakra writes each graha once per ring, not once, because its
    rings count houses from different places.
  - North Indian's first-house diamond holds six lines at full size.
  - Ten lines fit a quarter-width South Indian cell in four columns at
    full size.

  Each test now states the property that matters: every line is inside
  its cell and off its label, checked for every grid layout.
- **The first drawings showed what no structural test had.**
  - A stack could cover the cell's sign number, so the fitting now keeps
    the label clear.
  - Nudging a wheel's conjunction towards the centre put it on a spoke and
    a house number, so marks are spread round the ring instead, with a
    tick at each true degree.
  - The first spreading split an already spread cluster into pairs and
    never settled. Touching marks now stay one cluster.
- **`hypot` is not `sqrt`.** IEEE 754 requires `sqrt` to be correctly
  rounded and leaves `hypot` to the platform, so an arc's radius is taken
  with `sqrt`.
- **A wheel under whole-sign houses** draws house 1 as the whole lagna
  sign, which can lie entirely on one side of the ascendant's line. That
  is the house system, drawn faithfully, and not a fault in the wheel.

## 6b. The boundary and the bindings

- **A theme crosses with the request, and the SVGs come back in the same
  crossing.** `TsChartRequest.theme_json` is nullable JSON, the convention
  `settings_json` set. When it is given, the charts blob's section 22
  `svgs` carries one array of strings per chart. When it is null, nothing
  is rendered and nothing is paid. A second entry point would have to
  found the chart again, or keep documents alive across calls, which the
  ABI does not do.
- **A theme names what it changes over a shipped one.** An object's
  `extends` names `LIGHT` or `DARK` (`ShippedTheme`, a key like every other
  word a request writes), and the object is laid over it field by field, so
  a dark theme with another accent is
  `{"extends": "DARK", "style": {"accent": "#ffcc00"}}`. Refusals are named
  from the theme's root and the boundary calls that root `theme_json`, as
  in `theme.style.ink` and `theme.extends`.
- **Each binding types the record.**
  - Node: a `Theme` union of the names and a record of `ThemeStyle` and
    `ThemeContent`.
  - Python: `TypedDict`s under strict mypy.
  - Dart: `ChartTheme.light`, `ChartTheme.dark` and `copyWith`, over
    `ThemeStyle`, `ThemeContent`, `BodyForm` and `CellLabel`.

  Each reads the SVG back as `drawing.svg`. The Node test sets every field
  a record can name, so a key the SDK does not read fails a test rather
  than a consumer.
- **Parity covers the bytes.** The four runners ask for the dark theme and
  print each drawing's SVG, and they agree on every one.
- **Building found a defect in Node's error path, older than this work.**
  The layer read the context's last error for any exception its call
  threw, so an argument it refused itself, before the library was reached,
  was reported as whatever the library had refused last. The theme test
  caught it: a `theme: 7` after a refused `"sepia"` came back as the
  `sepia` refusal. The fix is in the addon's generator. A failed call's
  error now carries its own record as `lastError`, as a refused handle's
  already did, and the layer reads the context's last error only when a
  provider threw.
- **The hash matrix agrees.** Run 34877564969 compared the `render`
  section's 106 705 values across Linux x86-64, Linux aarch64 and macOS
  aarch64: none differ.

## 7. Order of work

1. **This page.**
2. **`crates/render-svg`** (built):
   - `Style`, `Content`, `Theme`, `Labels` and `render`;
   - unit tests for the writer's numbers, escaping, arcs, the fitting
     and the refusals;
   - an XML well-formedness test over every shipped layout.
3. **The façade** (built).
   - `Labels::compose` from a document, a context and a `Content`.
   - A drawing renders to SVG.
   - The golden drawings are generated by `cargo xtask render` and gated
     by `check-render`: one real chart in all six layouts and its navamsha,
     in Nepali on the light theme and English on the dark, fourteen files
     in `crates/render-svg/golden/`.
4. **The hash matrix's `render` section** (built): the wheel and the
   chakra drawn over the geometry section's sweep of ascendants, with a
   conjunction of five, every byte compared.
5. **The boundary and the bindings** (built, §6b): a theme crosses as
   JSON, the SVGs come back in the charts blob, and parity compares every
   byte.

## 8. What this design does not settle

- **A vetted retrograde marker per script.** `Content` carries it as a
  string written after the name, and the shipped themes use the Latin
  `(R)`, which a Devanagari chart shows as `बु(R)`. That is a visible
  fallback. No Devanagari marker ships until a source names one.
- **Outer planets on the wheel**, which arrive when the chart's bodies
  do.
- **Interaction**, beyond the data attributes.
