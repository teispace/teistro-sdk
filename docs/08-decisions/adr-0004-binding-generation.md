# ADR-0004: One API description, generated bindings, parity gate

Status: accepted (maintainer, 2026-09-04): one description, generated
bindings, parity gate; the choice between A and B is made by the Phase 0
spike and recorded as ADR-0007 when measured
Date: 2026-09-04
Question: Q2 (decided), Q3 (open: binding order)

## Context

Same API, same signatures, same behaviour in every language, with
consumer-implemented ports. Candidates: (A) C ABI plus an extracted IDL
plus per-language generators (the Teimeris toolchain); (B) Diplomat; (C)
per-ecosystem tools (UniFFI, napi-rs, wasm-bindgen, flutter_rust_bridge,
PyO3).

## Decision

The mechanical layer of every binding is generated from one description;
a thin hand-written ergonomic layer per language is held to a common
surface by a parity gate; the API reference is generated from the same
description. Which generator produces the mechanical layer is decided by a
Phase 0 spike of A against B on one real slice with a port callback,
measured for code volume, callback and marshalling cost, and idiomatic
quality. Approach C is rejected as a primary strategy because it has no
single source of truth.

v1 bindings: Node native, wasm, Dart/Flutter, Python, Rust, with C and C++
headers as the base; Java after; Swift and Kotlin on demand.

## Consequences

- A spike before the repository's first non-docs code.
- If A: the IDL extractor and generators are shared with Teimeris and
  maintained by Teispace.
- If B: Diplomat's backend maturity per language (no Swift, Python in
  progress) shapes the binding order.
- Either way, tree-shaped results need a designed wire encoding (columns,
  blobs, handles), measured in the spike.

## Alternatives considered

See the context. The WebAssembly component model is not a binding strategy
in 2026 for the non-web ecosystems.

## Evidence

`01-research/platform/02-binding-technology.md`; Diplomat's June 2026
design post; UniFFI's foreign traits documentation; Teimeris's `tools/idl`.
