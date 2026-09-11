# Provider plugins and the v1 target matrix

Status: research, 2026-09-11. Grounds ADR-0029 (the provider plugin
model) and ADR-0030 (the consumption surface).

The maintainer's brief, 2026-09-11: a consumer installs the SDK for their
platform and **plugs an ephemeris into it** the way a Node program plugs a
transport into nodemailer — Teimeris, Swiss Ephemeris, something of their
own — with the SDK validating the choice and staying type safe. They then
consume it as `sdk.<area>.<operation>`, with the engine's own functions
reachable at `sdk.engine.*` where the SDK has not ported one. **In
something like 98% of cases the consumer should be on a real engine; the
built-in ephemeris is the fallback**, not the intended path.

v1 must cover **Rust, Node, wasm, Flutter/Dart and Python** at least.

This page is what a survey of the code found, not what it proposes.

## The finding that reorders the work

**`sdk.engine.*` is built at the boundary and reaches nothing.**

`ts_ephemeris_manifest` and `ts_ephemeris_call` exist, the bindings wrap
them (`Engine.names`, `.signature()`, `.call()`), and the port declares
`native_manifest` and `native_call` as optional provider routes. The
Teimeris adapter does not claim the route, and says so in its own source:

> Teimeris describes 161 functions of its own in `tools/idl/teimeris.idl`,
> and answering the port's `native_manifest` and `native_call` from it is
> this adapter's remaining work; until it does, it must not claim the
> route.

So the escape hatch the brief calls for is a working mechanism attached to
no engine. That is the single largest gap between the brief and the code,
and it is larger than it looks only until the second finding.

## The second finding: it is a generation problem

`tools/idl/teimeris.idl` is a generated, fully typed description of the
engine: **161 functions**, each parameter carrying a `role` — `handle`,
in, out — and a type with base, pointer depth and constness, beside
complete `enums`, `structs` and `callbacks`. The engine already generates
its own Node, Dart and Java bindings from it.

The SDK's own boundary is described the same way in `idl/api.json` and
generated into four languages by machinery that is
[role-driven](../../../spikes/02-binding-toolchain/README.md): "nothing in
them names a function of the slice; roles come from types and naming
conventions".

Two typed descriptions and a role-driven generator on each side. A typed
`sdk.engine.*` is therefore **generated**, not 161 hand-written wrappers,
and the untyped JSON `call(name, args)` route stays as the fallback for an
engine that describes itself but is not one the SDK generates for.

## Two seams, and every target has at least one

The port already offers two ways in, and the distinction decides the
whole target matrix:

| seam | how | cost | needs |
|---|---|---|---|
| **native** | a `ProviderVtable` of C function pointers | a C call per batch | the host can load a shared library |
| **host** | an object with `positions(request)` in the host language | a callback per batch, measured at **0.08 µs into Dart** | nothing |

The host seam is not a poor relation. The binding spike measured a
provider implemented in Dart at 11.4 µs against 10.8 µs for the native
one on a depth-3 chart — because the port's one required operation takes
a **grid**, so there is one callback per batch rather than one per cell.
A host-language adapter over a wasm engine pays that once.

## The target matrix

| target | can load a shared library? | seam | what an engine costs there |
|---|---|---|---|
| **Rust** | n/a — links directly | native | works today: `TeimerisProvider::open(dir)` |
| **C** | yes | native | the vtable, once an adapter ships one |
| **Node** | yes — the SDK's addon is already a `cdylib` and can `dlopen` | native | a platform binary per target triple |
| **Python** | yes — `ctypes` already loads the SDK's library | native | the same binary |
| **Flutter/Dart, mobile and desktop** | yes — `dart:ffi` | native | the same binary, bundled as an asset |
| **Flutter web** | **no** — no `dart:ffi` | host only | a wasm or JS engine behind `positions()` |
| **wasm / browser** | **no** — no dynamic linking | host only, or linked at build | see below |

Two consequences worth stating plainly.

**The loader belongs in the SDK, not in each binding.** Node, Python and
Dart can each `dlopen`, so each *could* load an adapter and hand its
vtable across. Doing it once at the boundary — `ts_provider_load(path,
config)` — means one implementation, one place where the `unsafe` lives,
one error vocabulary, and no raw pointer crossing a host language. It is
the same argument that moved `validate` to the SDK's side of the boundary
when three bindings had grown three copies of the policy.

**wasm cannot load anything, and that is the whole of its difficulty.**
There is no `dlopen`; a wasm module's imports are fixed at build. So a
browser consumer has exactly three routes:

1. **The built-in ephemeris**, compiled into the SDK's own wasm. The
   `compact` tier exists for this — its manifest says "one arcminute in
   about 80 KB, for wasm and mobile" — and it needs no files and no
   network. This is the only route that works with nothing installed.
2. **A host adapter over an engine compiled to wasm**: the C engine built
   with emscripten or wasi, its data files fetched and cached, behind a
   JavaScript `positions()`. A real project, and the data delivery is the
   larger half of it, not the compile.
3. **A host adapter over a remote service**: `positions()` that calls an
   HTTP endpoint. Trivially portable, and a latency and privacy decision
   rather than an engineering one.

The binding spike's wasm measurements carry a warning for route 2: 300
depth-5 charts in a row **exhausted the wasm heap** (`rust_oom`) under an
opaque-handle design, where this SDK's blob-and-views design decodes in
constant time. The design already in place is the one that survives
there; a wasm engine adds its own ephemeris data on top of it.

## Licensing is why the plugin model is the right shape anyway

| component | licence |
|---|---|
| the SDK | **Apache-2.0** |
| Teimeris | **AGPL-3.0** |
| Swiss Ephemeris | AGPL-3.0, or commercial |

`deny.toml` refuses AGPL across the workspace under ADR-0019, and the
workspace **already excludes both adapters** (`adapters/ephemeris-teimeris`,
`adapters/ephemeris-sweph`) so that it never enters the SDK's dependency
graph.

This is not an obstacle the plugin model has to work around — it is the
argument *for* the plugin model. An adapter the consumer installs
deliberately, as its own package, under its own licence, is a cleaner
boundary than anything bundled could be, and it is exactly how a transport
relates to nodemailer. What the SDK owes in return is to say plainly what
loading an AGPL engine into a process means for distributing the result.

## Where the adapters actually are

Both are Rust `rlib`s: no `crate-type`, no exported C entry point, no
package for any binding. A Node, Python or Dart consumer **cannot reach
either engine today by any route** except writing a provider in their own
language. For a brief whose 98% case is a real engine, that is the
headline.

The Node addon is already a `cdylib`, which shows the packaging shape is
routine; what is missing is that the adapters were never given one.

## What this does not settle

**Whether the engine compiles to wasm cleanly**, and what its data files
cost over a network. Nothing here built it; route 2 above is costed as "a
real project" on the strength of what the engine is, not on a trial.

**What a namespaced surface should be named.** `idl/api.json` records each
function's `source` module — `calendar.rs`, `time.rs`, `intl.rs`,
`frame.rs`, `panchanga.rs`, `ephemeris.rs` — and the documentation site
already groups its reference pages by exactly that. So a namespacing has a
*derivation* rather than an invention, but the bindings' hand-written
layer is richer than the 43 boundary functions and does not map one to
one. ADR-0030 decides it; this page only records that the grouping exists
and is already in use.

**Java, Swift and Kotlin**, which the brief does not put in v1 and which
the C ABI reaches when they arrive.
