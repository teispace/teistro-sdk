# ADR-0015: The quality bar is gated, not aspired to

Status: accepted (maintainer, 2026-09-04)
Date: 2026-09-04
Question: the maintainer's instruction that the code must be proper, well
tested, validated, benchmarked and measured for memory, optimisation and
leaks

## Context

An SDK that is sold and built upon is judged on correctness, speed and
resource use over years. Claims that are not measured drift; measurements
that are not gated regress.

## Decision

`docs/05-testing/01-quality-bar.md` is binding. A module is done only
when every applicable gate there is green: unit tests with classical
examples, golden-vector conformance, property and snapshot tests,
cross-binding parity, the provider conformance kit, executed doc examples,
coverage floors that only rise, fuzzing, sanitizers and Miri, input
validation and iteration caps, a panic-free boundary, criterion benchmarks
with checked-in baselines and a results schema that refuses unsupported
claims, allocation counts asserted in tests, peak-memory and leak checks
in Rust and in every binding, cache bounds, size budgets, thread scaling,
format and lint with pedantic clippy as errors, unsafe confinement with
reviewed `SAFETY` comments, documented public items, vetted dependencies,
regenerated artefacts, and gates proven red before they are trusted.

Every optimisation is preceded by a profile and followed by a measurement
recorded in the commit body. The generated accuracy, conformance,
performance, memory and size documents are the public record.

## Consequences

- The fast check grows with each module; the nightly carries the slow
  gates; releases run everything.
- Test-only infrastructure (counting allocator, leak harnesses per binding,
  benchmark schema) is built in Phase 1 before the first domain module.
- Pull requests lead with numbers, and reviewers ask for the profile.

## Alternatives considered

Best-effort testing with benchmarks "when needed": the way regressions
arrive unnoticed.

## Evidence

Teimeris's experience: its most repeated defect was a documented claim that
nothing measured, and its gates found defects a green suite could not see.
