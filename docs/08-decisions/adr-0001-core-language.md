# ADR-0001: Rust core with a stable C ABI

Status: accepted (maintainer, 2026-09-04)
Date: 2026-09-04
Question: Q1

## Context

The SDK is mostly data and rules crossing an FFI boundary in both
directions; it needs tree-shaking, WebAssembly, safe callbacks from
consumer code, and a binding ecosystem. The candidates were Rust, C, C++
and Zig (`01-research/platform/01-core-language.md`).

## Decision

Write the core in Rust as a Cargo workspace of one crate per module, with
`#![forbid(unsafe_code)]` everywhere except a single `ffi` crate that
exposes a stable C ABI following Teimeris's conventions (`struct_size`,
capacities, structured errors, batch shapes). Bindings are built over that
ABI (ADR-0004).

## Consequences

- Type-safe modelling of the catalogue, rules and results; compiler-checked
  exhaustiveness.
- ICU4X primitives and calendar crates reusable directly.
- A second toolchain beside Teimeris's C; mitigated by shared ABI
  conventions and, if ADR-0004 chooses A, shared binding tooling.
- Compile-time discipline needed: crate splitting, feature hygiene.

## Alternatives considered

C (the Teimeris choice): fastest build, smallest binaries, but every safety
property manual and data modelling at this scale painful. C++: no advantage
over Rust here and more ways to go wrong. Zig: attractive but pre-1.0 with a
small ecosystem.

## Evidence

The research page; Teimeris's own Rust binding as a working example in the
same house; ICU4X as the model for Rust libraries with generated multi-
language bindings.
