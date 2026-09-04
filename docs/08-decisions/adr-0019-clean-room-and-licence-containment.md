# ADR-0019: Clean room and licence containment

Status: accepted (maintainer, 2026-09-04)
Date: 2026-09-04
Question: Q29

## Context

ADR-0006 licenses the SDK under Apache-2.0 and keeps ephemeris adapters
separate. That promise is only as good as the dependency graph and the
provenance of the code. The field offers a live warning: a permissively
badged astrology engine whose astronomy is a port of Swiss Ephemeris
carries obligations its licence file does not describe, because the
encumbrance rides in through a dependency. The research also read AGPL
code (PyJHora) for feature discovery, which creates an obligation to say
publicly what was and was not taken.

## Decision

1. **`CLEAN_ROOM.md`** at the repository root states the sources by rank,
   what may be taken from each and what may never be, and is binding.
2. **The dependency policy is an allow list** in `deny.toml`, checked by
   `cargo deny` in the fast check: MIT, Apache-2.0 (with the LLVM
   exception), BSD-2 and BSD-3, ISC, Zlib, 0BSD, Unicode, CC0 and
   Unlicense. GPL, LGPL, AGPL, SSPL and MPL are denied in every crate of
   the workspace, dev-dependencies included. The MPL denial is deliberate:
   it is why the SDK ports ERFA from its BSD-3 C source instead of
   depending on the MPL-licensed Rust port, and why ANISE is an oracle
   and not a dependency.
3. **Oracles are never dependencies of publishable crates.** Swiss
   Ephemeris (through a licensed copy), CSPICE, ANISE and the C ERFA
   library live in crates marked `publish = false` outside the published
   graph, or as recorded fixtures.
4. **Adapters for licensed engines** (Teimeris, Swiss Ephemeris) are
   separate packages under their own terms; the dependency direction is
   adapter to SDK, never the reverse; a CI job builds and tests the
   workspace with the test provider and no adapter present, which is the
   check that containment holds.
5. **Rules for the Swiss host adapter** (spike 3 and after): the raw
   binding is never public; one owning type serialises every call, and
   the state-setting calls and the computation happen under one lock; a
   stress test interleaving distinct sidereal modes and observers across
   threads must return results identical to the serial baseline; a
   missing data file is an error, never a silent analytic fallback; the
   ephemeris flag is explicit at every call; data files are content-hashed
   and the hashes enter the provenance envelope.
6. **Ported permissive code** keeps its notice in `NOTICE` and a
   function-by-function provenance table.

## Consequences

- `deny.toml`, the `cargo deny` step and `CLEAN_ROOM.md` land in Phase 0;
  the test-provider-only CI job lands with the first adapter.
- `NOTICE` gains ERFA when the port lands (ADR-0021).
- The contributing guide points at the clean-room rules; a pull request
  that adds a dependency names its licence.

## Alternatives considered

A deny list (the default is then "allowed", and the first unusual licence
slips through); Swiss Ephemeris in-tree behind a feature (the badge would
describe the wrapper while the obligation attaches to what ships); no
public clean-room statement (provenance that is not contemporaneous is not
credible).

## Evidence

`01-research/platform/12-licensing.md`; the licence texts of the Swiss
Ephemeris wrappers on npm and crates.io, which declare the wrapper's terms
while compiling Astrodienst's code.
