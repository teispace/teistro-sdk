# Binding architecture

Status: `draft`, 2026-09-04. Depends on Q2 (generation strategy) and Q3
(languages). Written for approach A (C ABI plus IDL plus generators); the
Diplomat variant changes the mechanical layer's origin, not the shape.

## Three layers per binding

| layer | origin | job |
|---|---|---|
| mechanical | generated from the IDL | loads the native library (or wasm), declares every entry point, marshals structs, arrays and blobs, checks the version handshake, maps statuses to exceptions |
| ergonomic | hand-written, thin | idiomatic objects (`Context`, request builders, lazy result decoders, iterators), option objects with validation, port adapters (wrapping a host object into a vtable), worker pools |
| packaging | generated plus templates | manifests, prebuilds, source fallback, `buildinfo.json`, docs snippets, install check |

The parity gate compares the ergonomic surface of every binding (methods,
options, result fields) against the IDL and against each other.

## Result marshalling

| result kind | encoding | decode |
|---|---|---|
| scalar and small struct | C struct | direct |
| grids (positions, cusps, dasha rows, panchanga limb rows) | columnar typed arrays | zero-copy views (Float64Array, numpy, Dart typed data) |
| trees and nested results (full chart, rule results, dasha tree, muhurta rows) | result blob: length-prefixed sections with a table of contents; strings as key ids | lazy decoders produce native objects on access; JSON export is a blob-to-JSON routine shared by all bindings (so canonical JSON is byte-identical everywhere) |

The blob layout is part of the IDL and versioned; every binding's decoder
is generated from the same description.

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
