# Design principles

Status: `draft`, 2026-09-04. Each principle names the failure it prevents.
Most of those failures were observed in the baseline engine or in upstream astrology
software; the source is cited where it is not obvious.

## 1. The engine never owns the ephemeris

The SDK defines an ephemeris port and consumes whatever fulfils it. It links no
ephemeris, embeds no ephemeris data, and carries no ephemeris licence. A
provider declares its capabilities and the SDK validates settings against them
at setup, failing loud rather than computing a plausible wrong answer.

Prevents: licence coupling (Swiss Ephemeris AGPL versus Professional), a
single point of accuracy failure, and the baseline engine pattern where twelve call
sites each mutate global ephemeris state.

## 2. Keys, not strings

Every engine output is a stable, language-neutral identifier: entity keys,
state keys, rule keys, reason codes. Display text is produced by a separate
localisation layer from data packs. No computation module imports a string
table.

Prevents: the baseline engine condition where four-language name blocks are threaded
through computation results and cached in locale-specific shapes, and where
adding a language means touching engine code.

## 3. Settings are explicit, complete and snapshotted

Every computation takes a settings object with no hidden defaults beyond a
named profile, and every result records the settings, the provider identity
and the SDK version it was produced under. A default never changes silently;
changing one is a major version with the size of the difference in the
changelog (Teimeris `COMPATIBILITY.md` rule).

Prevents: a stored chart disagreeing with the engine that produced it, and
the "which house system did this use" class of support tickets.

## 4. Named variants, never a silent school

Where traditions disagree, the SDK implements the variants under distinct keys
(Bhava-Chalit Sripati and Vehlow; chara karakas seven and eight; Tara counts;
Vashya matrices; Kalachakra sequences) and a profile selects. The baseline engine's
Bhava-Chalit was measured to be Vehlow while documented as Sripati; the SDK
makes such a thing impossible by construction, because the key says which.

## 5. Batch is the primary shape

Grids of instants and bodies, lists of charts, ranges of days. Scalar calls are
wrappers. This is what makes FFI cost amortisable, what a panchanga month or
a transit calendar actually needs, and what lets the ephemeris port be
fulfilled efficiently from another language.

## 6. Rules are data

Yogas, doshas, muhurta rules, matching tables, remedy tables and interpretation
mappings are declarative data with citations, evaluated by generic engines.
Adding a rule is adding a record; a consumer can ship their own rule pack.
The baseline engine already does this for 562 yogas and 62 doshas; the SDK extends it to
every rule-shaped feature.

## 7. Modules are independent and tree-shakable

Each module has an explicit dependency list forming a DAG with no cycles, its
own tests, its own data, and its own package in every binding. A consumer
ships only the modules and the locale data they use. A gate enumerates the
dependency graph and fails on a cycle or an undeclared edge.

## 8. One description, every binding

The API is described once, machine-readably. Every binding's mechanical layer
is generated from it, the API reference is generated from it, and a parity
gate refuses a capability reachable in one binding and not another. Hand-
written ergonomic layers are thin and held to the same surface.

## 9. Measured, not asserted

A number in a document is read from the file that gates it. Performance
claims come from interleaved benchmarks with a reported noise floor; accuracy
claims come from conformance runs against golden vectors. A check that cannot
fail is proven red once before it is trusted. This is Teimeris's discipline
and it is adopted whole.

## 10. Deterministic and reproducible

Same inputs, same settings, same provider, same version: byte-identical
output, in every binding. Floating-point policy is fixed (no fast-math;
contraction policy stated), iteration caps are fixed, and results carry the
version triple. Cross-binding parity tests assert it.

## 11. Fail loud, never NaN

Invalid input is an error with a message naming the field and the accepted
range. Iterative searches that do not converge return a status, not a stale
date. No output field is ever NaN; degenerate cases (polar sunrise, house
systems undefined at a latitude) are reported states.

## 12. Secure by construction

No panics cross the FFI boundary. Every array carries its capacity; every
struct its size. Data packs are validated and bounded before use. Fuzzing and
sanitizers run in CI. Dependencies are audited and pinned; artefacts carry
provenance. Nothing in the SDK reads the environment or the network.

## 13. Documentation is a deliverable

Every entry point has reference documentation generated from its
description, every module has a design page, every guideline has a worked
example, and every example in the docs is executed by the build.

## 14. Extensible from outside

Languages, calendars, timezone data, ephemerides, dasha systems, house
systems, rule packs, interpretation packs and custom vargas are all
registered through public extension points. A consumer never forks the SDK
to add one, and the guidelines in `09-guidelines/` show how.
