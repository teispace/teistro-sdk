# Interpretation, serialisation and rendering

Status: `research`, 2026-09-04. Checked against the baseline engine's interpret package
(38 state tables, yoga and dosha texts merged from per-file records,
namespaced facade, deterministic composers), its chart serialiser (v1.4.0
dossier with presets, date-free persistence, gochar sidecar), Parashara's
Light (interpretive reports, 300 worksheets), Solar Fire (report libraries,
point-and-click interpretations, wheel designer) and Kala (interpretive
reports).

## Interpretation as data

| feature | baseline | field | tier |
|---|---|---|---|
| state interpretations: planet in sign, planet in house, lagna, dignity states, avasthas, dasha lord effects, Shadbala tiers, transit states, and so on, keyed by stable state keys, four languages, with citations | yes (38 tables) | PL, Solar Fire, Kala | P0 |
| rule interpretations: one text per yoga and dosha, keyed by rule key | yes | all | P0 |
| composers: deterministic prose from structured results (milan verdict, graha-in-bhava with conjunction synthesis, dasha narrative, day narrative, muhurta reasons, namakarana notes, tara chakra, yoga timing) | yes | The baseline engine is ahead of the field here | P0 |
| templating with placeholders, plural forms, gendered forms, honorifics per language | ad hoc string replace | | P0 (MessageFormat 2 class templating; see `platform/04`) |
| interpretation packs as versioned content with licence metadata, separate from locale packs | no | | P0 |
| report assembly: sections, ordering, grounding references | yes (report catalogue: 18 narrative and 21 chart-grid sections) | PL worksheets, Solar Fire pages | P0 (data model), P1 (assembly helpers) |
| LLM prompt dossiers: date-free serialisation of the full chart with presets (lite, deep, family) and date-aware derived views | yes (serialiser 1.4.0) | none | P0 |

## Serialisation

| feature | baseline | tier |
|---|---|---|
| canonical JSON of every result with stable key names and deterministic ordering | partial | P0 |
| compact text dossier for prompts, presets, byte-stable | yes | P0 |
| binary chart blob with magic, format version and CRC (Teimeris `serial.h` model) for caches | no | P1 |
| content hashes of inputs and settings (the baseline engine's `contentHash` recipe) | yes | P0 |

## Rendering geometry (data for the consumer's drawing layer)

| feature | baseline | tier |
|---|---|---|
| North Indian, South Indian, East Indian and Bengali chart layouts as cell assignments (house or sign fixed) | partial (projection in milan) | P0 |
| Western wheel geometry (angles, aspect lines, multi-ring) | no | P1 |
| chakra diagrams (Sudarshana rings, Sarvatobhadra grid, Kota) as cell occupancy | Sudarshana yes | P1 |
| dasha timeline and tree data (bars, nested nodes) | yes | P0 |
| transit calendar and time map series | partial | P1 |
| glyph and label catalogue per script | locale packs | P0 |

The SDK ships geometry and data; it never draws. A separate optional
rendering package per platform (SVG for web, canvas for Flutter) can be
built on top later.
