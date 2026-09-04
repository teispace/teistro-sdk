# ADR-0010: Teistro Intl, one opinionated localisation standard

Status: accepted (maintainer, 2026-09-04)
Date: 2026-09-04
Question: Q10 (and Q20, Q21 for two conventions still open)

## Context

The baseline engine's localisation works but its architecture is poor (name blocks
inside results, several egress rules, string-replacement templates, no
tooling). The maintainer asked for one centralised, opinionated, extensible
standard in the manner of next-intl and slang.

## Decision

Teistro Intl: text in `i18n/<locale>/<namespace>.json` with a `_meta.json`
per locale; Unicode MessageFormat 2.0 syntax with a fixed function set
bound to SDK types (`:entity`, `:dms`, `:zodiac`, `:ghati`, calendar-aware
`:date`); the base locale as the source of truth for keys and parameters;
packs compiled by the `teistro-intl` CLI (validate, build, gen, extract,
analyze, apply, clean, edit, normalize, stats, outdated, diff, report,
XLIFF export and import, migrate); typed accessors generated for every
binding so key and parameter errors are compile errors; ICU4X for CLDR
data; runtime pack loading and overrides so consumers add languages
without touching the SDK; declared fallback chains; composers as narrative
plans rendered by the engine; cross-binding rendering parity gated. The
same engine and tooling are offered to consumer applications for their own
text. The tool was called `langgen` in earlier drafts; the name is now
`teistro-intl`.

## Consequences

- An MF2 subset engine to write, fuzz and keep to the standard.
- A code generator per binding target.
- Translation becomes a tooled workflow with coverage reports and
  translator-friendly exports.
- The SDK's four launch languages are packs; the baseline engine's content migrates
  through `migrate baseline`.

## Alternatives considered

Fluent syntax (close, but MF2 is the Unicode standard); ICU MF1 (what
next-intl uses; superseded); strings in code (the baseline engine).

## Evidence

`01-research/platform/04-localization.md` and the architecture page;
next-intl and slang documentation read on 2026-09-04; CLDR 47 MF2
stabilisation; ICU4X data management.
