# ADR-0005: Module families as crates and packages; profiles; size gates

Status: accepted (maintainer, 2026-09-04, together with the v1 scope:
baseline parity in v1.0, Western and Hellenistic designed in Phase 0)
Date: 2026-09-04
Question: Q4

## Context

Consumers must ship only what they use, on platforms with different
tree-shaking mechanics (linkers, bundlers, wasm binaries, Dart tree-shaking,
wheels).

## Decision

The SDK is a DAG of module families, each a Rust crate with features for
optional pieces and each a package or subpath in every binding. Named
profiles (`panchanga`, `kundali`, `baseline-parity`, `full`) are the native
build targets; wasm ships per-profile binaries plus a build tool for exact
module sets; locale and interpretation data are sliced per namespace and
locale by `langgen`. A dependency gate forbids cycles and undeclared edges;
a size gate per profile per platform fails on regression; an install check
runs each profile in a throwaway project.

## Consequences

- More packages to publish and test; mitigated by generation and by the
  release matrix being derived from the catalogue.
- Consumers choose a profile or build their own; the docs explain both.
- Extension points are registries on the context, never globals.

## Alternatives considered

One monolithic library per binding (simplest, ships everything);
per-function packages (unmanageable).

## Evidence

`01-research/platform/06-modularity-tree-shaking.md`; ICU4X's
data-slicing results; the baseline engine's package DAG gate.
