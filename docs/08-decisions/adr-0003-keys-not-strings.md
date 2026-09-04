# ADR-0003: Engine emits keys; localisation is data packs

Status: accepted (maintainer, 2026-09-04); the standard built on it is
ADR-0010 (Teistro Intl)
Date: 2026-09-04
Question: Q10

## Context

The baseline engine threads four-language name blocks through computation results and
applies locale rules in several places; adding a language touches engine
code; templates use string replacement with no plural or gender support.

## Decision

Computation modules emit only stable keys and numbers. A separate `l10n`
module renders keys to text from versioned locale packs and interpretation
packs, modelled on ICU4X's data providers (baked, blob, filesystem;
sliced by namespace and locale; explicit fallback chains) and using a
Unicode MessageFormat 2.0 subset for templates. Packs are compiled from
YAML sources by `teistro-langgen`, which validates completeness,
placeholder parity and citations. Consumers add languages by building and
loading their own packs without modifying the SDK. Composers produce
language-neutral narrative plans that `l10n` renders.

## Consequences

- Four languages at launch (ne, en, sa, hi) become data; more are data.
- Every key needs an entry per shipped locale, enforced by a gate.
- Text quality becomes a translation workflow with tooling, not a code
  change.
- The MF2 subset engine is SDK code to write and fuzz.

## Alternatives considered

Fluent syntax: close, but MF2 is the Unicode standard heading into
ECMA-402. Strings in code: the baseline engine failure mode.

## Evidence

`01-research/platform/04-localization.md`; ICU4X data management
tutorial; CLDR 47 MessageFormat 2.0 stabilisation.
