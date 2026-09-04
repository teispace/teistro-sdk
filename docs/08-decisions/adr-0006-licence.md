# ADR-0006: Apache-2.0 open core

Status: accepted (maintainer, 2026-09-04)
Date: 2026-09-04
Question: Q5

## Context

The SDK is meant to be the foundation anyone can build astrology
applications on. It links no ephemeris (ADR-0002), so the AGPL and
Professional-licence questions that attach to Swiss Ephemeris and Teimeris
do not attach to it. The options were Apache-2.0, MIT, source-available,
proprietary, and private-first (`01-research/platform/12-licensing.md`).

## Decision

The SDK core, the bindings, the tooling and the documentation are licensed
under Apache-2.0. Interpretation and content packs, the docs site's
commercial content, hosting and the ephemeris adapters carry their own
terms (an adapter follows its ephemeris's licence; packs may be
commercial). The `LICENSE` file and headers go in the repository's first
commit. Contributions are accepted under the same licence with a
lightweight contributor agreement to be chosen at repository creation.

## Consequences

- Maximum adoption; value concentrates in packs, Teimeris and services.
- Q6 (ownership of baseline engine content) must be settled before any the baseline engine-
  derived pack ships under any licence.
- Trademark: "Teistro" and "Teispace" remain marks; the licence covers
  code, not the names.
- Third-party names (Swiss Ephemeris, Jagannatha Hora) are used only
  descriptively.

## Alternatives considered

Private-first (the Teimeris model) was the safest but delays the
ecosystem; BSL protects against hosted competitors at the cost of adoption;
proprietary contradicts the stated goal.

## Evidence

The licensing research page; the maintainer's brief ("anyone who wants to
build any scale astrology applications can use this SDK").
