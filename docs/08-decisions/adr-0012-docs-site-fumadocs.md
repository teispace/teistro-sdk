# ADR-0012: Documentation site on Fumadocs

Status: accepted (maintainer, 2026-09-04)
Date: 2026-09-04
Question: Q11

## Context

The docs must be extreme in detail: a generated API reference with
per-binding tabs, concept pages, executed examples, versioning, English and
Nepali, interactive wasm examples.

## Decision

The site lives in `site/` in the SDK repository on Fumadocs (Next.js, RSC).
Concept and guide pages are authored under `docs/` and mirrored into the
site by the build; the API reference is generated from the API description
into MDX, one page per entry point with per-binding snippets; every example
is executed by a gate; the site deploys on tag.

## Consequences

- A Next.js toolchain in the repository beside Rust.
- Generated reference pages are committed and diffed.

## Alternatives considered

Starlight (static-first, built-in i18n; the fallback if product coupling
is unwanted), Nextra, Docusaurus.

## Evidence

`01-research/platform/10-docs-site.md`.
