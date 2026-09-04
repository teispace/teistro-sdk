# ADR-0014: Everything we author is Rust, including the tooling

Status: accepted (maintainer, 2026-09-04)
Date: 2026-09-04
Question: Q25

## Context

Teimeris keeps its gates and generators in Python beside a C core, which
was pragmatic for a port of a C library. The maintainer asked whether the
SDK could use only Rust. The first documentation gate had been written in
Python.

## Decision

Everything this repository authors is Rust:

- Repository tasks (gates, generators, ingestion tools, benchmarks,
  conformance runners, size reports, release steps) live in the `xtask`
  crate and run as `cargo xtask <task>`; they are tested like any crate.
- Consumer-facing tools (`teistro-intl`, and the `teistro` CLI it belongs
  to) are Rust binaries in the workspace.
- Binding generators are Rust programs emitting the target languages
  through templates; the API description is a Rust data model shared by
  the extractor, the generators and the reference generator.
- There is no Python, no `just`, and no shell beyond a one-line workflow
  step that invokes `cargo`.

What is not Rust, because it cannot be: the docs site (Fumadocs on
Next.js), the thin ergonomic layer and the tests of each binding (in the
binding's own language), the target-language toolchains CI needs to build
and test those bindings, and GitHub Actions workflow files, which are
declarative.

## Consequences

- One toolchain for contributors and CI; the tooling shares types with the
  core and is held to the same lints and tests.
- CI jobs install the Rust toolchain and cache the build; the fast check
  gains format, lint and unit-test steps from day one.
- The first gate was rewritten before the founding commit was finalised, so
  no Python ever appears in the history.

## Alternatives considered

Python for glue (the Teimeris model): a second toolchain and an untyped
harness beside a typed core. Shell: not portable to Windows contributors
and not testable.

## Evidence

The maintainer's request; the xtask pattern used across the Rust ecosystem
(cargo's own repository, ICU4X, wasm-bindgen).
