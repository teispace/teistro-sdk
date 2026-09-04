# ADR-0023: Type safety is a correctness feature, in every binding

Status: accepted (maintainer, 2026-09-04)
Date: 2026-09-04
Question: Q33

## Context

An astrology API is dense with parameters that share a primitive type
and mean different things: latitude and longitude, degrees and radians,
a 0-based sign index and a 1-based house number, a tropical and a
sidereal longitude, a Julian day in UT1 and one in TT. Every swap
produces a chart that looks plausible and is wrong, with no exception and
no stack trace. Most consumers live in the bindings, so a guarantee that
stops at the C ABI stops where the users are. The maintainer's
instruction: every API and signature in every binding must be type safe,
must offer suggestions (completion with documentation), and must be
robust.

## Decision

1. **Parse, do not validate.** Every domain quantity is a newtype
   validated once at construction (`Latitude`, `Longitude`, `Altitude`,
   `Degrees`, `Nas`, `JulianDay<Scale>`, `SignIndex` 0-based,
   `HouseNumber` 1-based, `NakshatraIndex`, `PadaIndex`,
   `VargaDivisions`, `Depth`, every key type). No bare primitive appears
   in a public signature; where one must cross the C ABI the field name
   carries the unit (`lon_deg`, `jd_tt`). Conversions between units are
   explicit and named; there is no implicit `From<f64>`.
2. **The Rust types are the single source of truth.** Every binding's
   types are generated from the API description, never hand-written:
   TypeScript branded types for newtypes, discriminated unions for enums
   and result states, `readonly` fields, `.d.ts` files verified under
   `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes` and
   `verbatimModuleSyntax`, plus generated runtime schemas (Valibot by
   default, a Zod adapter) as an optional subpath; Dart sealed classes
   with exhaustive `switch`, extension types for newtypes, validating
   factory constructors; Python `.pyi` stubs with `NewType`, `Literal`
   unions and `TypedDict` parameters, `py.typed`, optional Pydantic
   models; Java records and sealed interfaces; C typed enums, opaque
   handles and a documented header. A field added in Rust appears in
   every binding on the next build; a field renamed without the
   consumers fails the parity gate.
3. **Suggestions come from the description.** Every entry point,
   option, field and enum member carries its documentation, units, range
   and an example in the API description, and the generators emit them
   as doc comments, so completion in every editor shows what a parameter
   means and what it accepts. Request builders with named options,
   result objects with named typed fields, typed intl accessors
   (ADR-0010) and typed cursors are the shape everywhere; positional
   parameters stop at three.
4. **Illegal states are unrepresentable.** A period cannot end before it
   starts; a chart cannot exist without provenance; a dosha rule cannot
   exist without remedies and severity; a varga cannot have zero
   divisions; a fallback chain cannot be empty; footedness and parity are
   distinct types. Enums are `#[non_exhaustive]` in Rust and closed
   unions in the bindings with an explicit `unknown` arm for forward
   compatibility. Construction-time invariants use typestate builders
   (a chart request without a place does not compile in Rust and does
   not type-check in TypeScript).
5. **Robust at every boundary, trusting within.** The C ABI, the blob
   decoders, pack loaders and data-file readers validate; internals do
   not re-check. Settings coherence is validated with typed results (a
   tropical zodiac with a sidereal-only ayanamsha, KP sub-lords with
   whole-sign houses, a classical siddhanta with a topocentric flag).
   Errors are typed with stable numeric codes and map to each language's
   idiom (exceptions carrying the code, `Result` in Rust); degenerate
   astronomical outcomes are typed states, never exceptions. The result
   blob carries a schema version; readers accept the current and the
   previous version and refuse others with a typed error; decoders are
   fuzz targets.
6. **Proven, not hoped.** `trybuild` compile-fail tests in Rust assert
   that swapped arguments and incomplete builders do not compile; a
   consumer project per binding is type-checked at maximum strictness in
   CI; one shared corpus of valid and invalid inputs runs through every
   binding's validators and the Rust constructors and must agree; the
   parity gate compares generated type surfaces against the description.
   Unchecked constructors exist in Rust only, named `_unchecked`,
   debug-asserted, never across the ABI.

## Consequences

- The API description (IDL) gains units, ranges, nullability, examples
  and documentation per member; the generators gain type, schema and doc
  emitters; the parity gate covers types, not only names.
- The quality bar gains the compile-fail, strictness-consumer and
  shared-validator rows.
- The ergonomic layers stay thin but are fully typed; the verbosity
  (`Latitude.of(27.7)` instead of `27.7`) is accepted deliberately.

## Alternatives considered

Numbers with documentation (the swap bug in every language); hand-written
typings per binding (drift within two releases); runtime validation only
(a wrong chart that validates).

## Evidence

Teimeris's generated bindings and parity gate; the baseline engine's
hand-copied types across its clients, documented in its own code as a
maintenance hazard.
