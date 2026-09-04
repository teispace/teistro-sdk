# Binding architecture

Status: `draft`, revised 2026-09-05 (ADR-0007 decides approach A by
measurement; ADR-0023 adds the type-safety section). Depends on Q3
(languages). Written for approach A (C ABI plus an extracted API
description plus generators of our own); the spike that decided it,
`spikes/02-binding-toolchain/`, is the model for the Phase 1 extractor,
emitters, result blob and finaliser-backed handles, and this page absorbs
its consequences in Phase 1.

## Three layers per binding

| layer | origin | job |
|---|---|---|
| mechanical | generated from the IDL | loads the native library (or wasm), declares every entry point, marshals structs, arrays and blobs, checks the version handshake, maps statuses to exceptions |
| ergonomic | hand-written, thin | idiomatic objects (`Context`, request builders, lazy result decoders, iterators), option objects with validation, port adapters (wrapping a host object into a vtable), worker pools |
| packaging | generated plus templates | manifests, prebuilds, source fallback, `buildinfo.json`, docs snippets, install check |

The parity gate compares the ergonomic surface of every binding (methods,
options, result fields) against the IDL and against each other.

## Type safety per binding (ADR-0023)

The Rust types are the source of truth; every binding's types are
generated from the API description and never hand-written. What the
description carries per member: type, unit, range, nullability,
documentation, an example. What each binding emits:

| binding | newtypes | enums and states | runtime validation | suggestions |
|---|---|---|---|---|
| TypeScript (Node, wasm) | branded types (`Latitude`, `Longitude`, `Nas`, ...) with validating constructors; `place(lat, lon)` does not compile with the arguments swapped | discriminated unions with `readonly` fields and `as const` literals; exhaustive `switch` helpers | generated Valibot schemas (Zod adapter) on the `/schemas` subpath | doc comments on every member; `.d.ts` verified under `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`, `verbatimModuleSyntax` by a consumer project in CI |
| Dart and Flutter | extension types (zero cost) with validating factories | sealed classes, exhaustive `switch` | `assert` in debug plus factory validation | doc comments; typed intl accessors |
| Python | `NewType` in generated `.pyi` stubs, `py.typed` | `Literal` unions, `TypedDict` parameters, `Enum` | optional Pydantic models generated from the same description | stubs give completion and type checking in every editor |
| Rust | the core newtypes themselves; typestate builders | `#[non_exhaustive]` enums | constructors | rustdoc with compiled examples |
| Java | records with validating factories, `@JvmInline`-style value semantics where the target supports it | sealed interfaces, exhaustive `switch` | factories | Javadoc from the description |
| C and C++ | typed enums, opaque handles, unit-suffixed field names; the C++ wrapper adds strong types | enums with an explicit unknown value | assertion-heavy debug build | the header is generated and documented per field |

Errors carry a stable numeric code in every binding (an exception with a
`code` field, `Result` in Rust); undefined astronomical outcomes are
typed states on the result, never exceptions. One shared corpus of valid
and invalid inputs runs through every binding's validators and the Rust
constructors and must agree, and the parity gate compares the generated
type surfaces against the description.

## Result marshalling

| result kind | encoding | decode |
|---|---|---|
| scalar and small struct | C struct | direct |
| grids (positions, cusps, dasha rows, panchanga limb rows) | columnar typed arrays | zero-copy views (Float64Array, numpy, Dart typed data) |
| trees and nested results (full chart, rule results, dasha tree, muhurta rows) | result blob: length-prefixed sections with a table of contents; strings as key ids | lazy decoders produce native objects on access; JSON export is a blob-to-JSON routine shared by all bindings (so canonical JSON is byte-identical everywhere) |

The blob layout is part of the IDL and versioned; every binding's decoder
is generated from the same description, reads the schema version first,
accepts the current and the previous version, and refuses others with a
typed error. Decoders are fuzz targets.

## Ports across the boundary

- Native providers pass a vtable pointer (see the ephemeris port page).
- Host-language providers are wrapped: the ergonomic layer registers
  trampolines (napi `ThreadsafeFunction` or synchronous callbacks on the
  calling thread, PyO3 `Py<PyAny>` calls with the GIL, Dart
  `NativeCallable.isolateLocal`) that receive grids as typed arrays and
  return columnar buffers; errors thrown in the host become provider
  errors in the core.
- Locale packs are bytes; no callback is needed.

## Threading and async

- The core is synchronous and single-threaded per context; contexts are
  `Send`.
- Node: a `ContextPool` over worker threads (Teimeris's shared pool
  machinery is the model), with the same-thread synchronous path as the
  default; host-language providers must be constructible in each worker.
- wasm: single-threaded; a pool over Web Workers where SharedArrayBuffer
  is unavailable uses message passing of blobs.
- Python: releases the GIL during native computation when the provider is
  native; holds it when the provider is Python.
- Dart: isolates with one context each.

## Loading and identity

- Native library located by an explicit path, then the package's
  prebuilds, then a development build directory, with a deny-list for
  sanitizer and unoptimised builds (Teimeris's loader lesson).
- The language half and the native half must be from the same build
  (`buildinfo.json` handshake) or the package refuses to load.
- The IDL ships inside every package so tooling on top can read real
  signatures.

## Per-language notes

| binding | mechanics | packaging |
|---|---|---|
| Node | N-API (NAPI_VERSION 8), one addon per profile, ESM subpath exports per module family with `sideEffects: false`, TypeScript types generated | npm tarballs with prebuilds and a source fallback that respects npm 12 install-script gating |
| wasm | wasm-bindgen or a hand C ABI over wasm exports with a JS glue generated from the IDL; per-profile binaries; browser-bundle gate | npm package; CommonJS and ESM; no Node built-ins on the browser path |
| Python | ctypes or PyO3; `py.typed`; numpy interop optional | wheels per platform plus sdist |
| Dart and Flutter | `dart:ffi` extension types; Flutter plugin builds the native library per platform with the profile from pubspec configuration; web through the wasm package | pub package and Flutter plugin |
| Rust | the core crates directly (no FFI); the C ABI crate for other consumers | crates |
| Java | FFM over the C header (Teimeris plan) | JAR with natives |
| C and C++ | the header; a C++ RAII wrapper generated | archives and headers |
