# Research

Status: `research`, 2026-09-04. This directory is the evidence base for the
architecture. Nothing here is a decision; decisions live in `08-decisions/`.

## Four bodies of research

| directory | question it answers | method |
|---|---|---|
| [`feature-universe/`](feature-universe/00-taxonomy.md) | what does astrology software compute, across every tradition, and how does each feature decompose into inputs, variants and outputs | domain knowledge cross-checked against product feature lists and open-source implementations; each page cites what it was checked against |
| [`competitive-analysis/`](competitive-analysis/00-matrix.md) | what do the products the baseline engine set out to replace, and the wider field, actually offer | web research on 2026-09-04: Jagannatha Hora, Parashara's Light, Kala, Shri Jyoti Star, Sky Vision, Astro-Vision, Maitreya, Solar Fire, Delphic Oracle, PyJHora, VedAstro, jyotishganit |
| [`baseline-engine/`](baseline-engine/00-inventory.md) | what the baseline engine computes today, how, and where it hurts | full read of the baseline engine repository (private) on 2026-09-04 |
| [`platform/`](platform/01-core-language.md) | how to build it: language, bindings, ephemeris abstraction, localisation, calendars and timezones, modularity, performance, security, testing, docs, CI/CD, licensing | Teimeris as the model; ICU4X as the model for data-driven localisation; web research on binding generators, MessageFormat 2, calendar libraries, docs frameworks, WebAssembly |

## How to read the feature pages

Every feature page uses the same table columns so the module catalogue in
`02-architecture/01-module-catalog.md` can be derived from them:

| column | meaning |
|---|---|
| feature | the technique or computation |
| inputs | what it needs: ephemeris data, chart state, settings |
| variants | the named schools or methods that must be selectable |
| baseline | `yes`, `partial`, `no`: whether the baseline engine has it today |
| field | which products in the competitive matrix have it |
| tier | `P0` baseline parity (v1.0), `P1` designed in v1.0 and shipped in v1.x, `P2` later |
| module | the SDK module candidate |

Where a fact is stated from memory rather than from a cited source it is
marked **verify** and listed in the page's closing checklist, so the design
phase knows what to confirm against a classical text or a reference
implementation before it becomes a golden vector.
