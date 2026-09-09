# Implementation plan: the batch, the threads and the engine passthrough

Status: `plan`, written 2026-09-09. Two bodies of work the maintainer
asked for in one sitting, kept in one document because they answer the
same brief — **an SDK a consumer never has to work around**. The first is
measured (`03-design/batch-and-parallelism-measured.md`); the second is
designed here and measured in the same pass. Read
[`../STATUS.md`](../STATUS.md) first: it says which step of this is done.

## Why these two, together

The maintainer's standing brief has two halves that turn out to be the
same requirement seen from either end.

**No dead ends.** A consumer must be able to change anything and reach
anything. Defaults yes; a wall, no. The largest wall in the project is
that the ephemeris port names **eight** operations and Teimeris's public
header names **161 functions, 57 structs and 40 enums** — so a consumer
who wants an eclipse, a star, an orbit or the engine's own calendar grid
must open a second handle to the same engine and manage it themselves.

**Efficiency designed in, not bolted on.** Batching, parallelism and
threads from the start rather than as a later optimisation. The
measurement says the SDK is at the opposite end today: `positions` is a
true batch (one call whatever its size) and **everything built on it is a
loop** — 69 ephemeris calls per chart, 1 213 per almanac day, at a mean
width of 1.1 cells through a port designed for grids.

Both are the same defect in different clothes: the SDK decides for the
consumer how the engine is used, and decides it narrowly.

## Part A — the batch, the cache and the threads

Designed from `03-design/batch-and-parallelism-measured.md`, whose
numbers are counts and therefore gated (`check-batching`). **The order is
the point**: threads would multiply redundant work rather than reduce it,
so the arithmetic is fixed first and the hardware spent last.

### A1. Widen the searches into grids

*The finding:* one sunrise costs 18 provider calls, every one a single
cell, none a repeat. The horizon solver brackets and bisects — thrifty in
count — but each bracket step is its own round trip when the whole
bracket is known before the first one.

*The change:* the bracket scan asks for its instants as **one grid**;
only the bisection stays serial, because each of its steps depends on the
last. Same instants, same answers, bit-identical; seventeen round trips
in eighteen become one.

*Where:* `astro::rise_set` first (it is the pattern), then
`astro::solve`'s crossing search, which every panchanga limb is built on.

*Gate:* the counts on the measured page, which fall when this lands.

### A2. A memo across a batch, gated on the provider's own declaration

*The finding:* the repeat share **rises with the batch** — 22.5% within
one chart, 64.7% across fifty. That growth can only come from sharing
between the items, which is what a batch exists to exploit.

*The change:* one memo, scoped to a single call, keyed by (instant, body,
frame). Not a global cache: a batch's memo dies with the batch, so
nothing outlives a request and no consumer is surprised by a stale sky.

*The correctness argument, which is already in the port:*
`Capabilities::deterministic` — "whether identical requests give
identical bits" — is declared by both shipped adapters and **read by
nothing**. The memo is sound exactly when that flag is true, and must
refuse to exist when it is false. The flag finally has its reader, and it
is the right one.

*The knob:* `provider.cache` — `Auto` (memo when the provider declares
determinism), `Off`, `On` (refused with the reason when the provider does
not declare it). Reported in provenance, as everything applied is.

### A3. Parallelism, as a knob, without a pool

*The finding:* the items of a batch are independent — `Founder::value`
takes `&self` and every field of `&self` is `Sync`, because the port
already requires `EphemerisProvider: Send + Sync`.

*The design, and why it is not rayon:* a library must not install a
global thread pool in its consumer's process. `std::thread::scope` costs
no dependency, starts and joins inside the call, and leaves nothing
behind. Chunk boundaries are fixed by index rather than by scheduling, so
**the answers are bit-identical whatever the thread count** — there is no
reduction to reorder, only independent writes into pre-sized columns.
That is what keeps the determinism matrix and the cross-architecture
hashes valid.

*The knob:* `compute.parallelism` — `Serial` (the default), `Auto`
(threads above a measured threshold, serial below it, because a batch of
two loses to the spawn), `Threads(n)`. Reported in provenance. The C ABI
carries it; the three bindings expose it.

*The threshold is measured, not guessed*: the pass gains a row for the
batch size at which threading begins to pay on this machine, and the
default is stated with that number beside it.

### A4. The order, and the number to beat

A chart batch of fifty costs 3 434 calls today. A2 alone removes about
two thirds of them; A1 removes most of what remains; A3 divides what is
left by the cores. The page is regenerated at each step and the number
moves in public.

## Part B — the engine passthrough

**The requirement, in the maintainer's words:** if the ephemeris is
updated later and gains new functions, they must be callable from the SDK
**without the SDK being updated**. Not a port method per operation — a
direct link.

### B1. Why this is achievable rather than aspirational

Teimeris already describes itself. `tools/idl/teimeris.idl` is 13 472
lines of machine-readable JSON, extracted from its own headers by its own
tooling, and it is what every Teimeris binding is generated from. It
carries every function's signature **and a role for every parameter** —
`handle`, `value`, `struct_in`, `scalar_out`, `struct_out`, `array_in`
with its `array_len`, `array_out` with its `array_cap`, `string_in` —
which is the same role vocabulary this SDK's own IDL uses, arrived at
independently.

A role vocabulary is exactly what a generic dispatcher needs. The engine
is not opaque; it is documented in a form a machine can read.

### B2. The shape

Three entry points, and no per-function code anywhere in the SDK.

| entry point | answers |
|---|---|
| `ts_ephemeris_describe` | the engine's manifest, as JSON: every operation, its parameters and their roles |
| `ts_ephemeris_call` | one operation by **name**, arguments as JSON, result as JSON |
| `ts_ephemeris_manifest_version` | what the manifest was extracted from, for the stamp |

The SDK holds no list of engine operations. It holds a *dispatcher* that
reads the manifest the engine ships and marshals by role. A function
added to the engine appears in the engine's manifest and is callable the
same day, with no SDK release — which is the requirement, stated as a
mechanism.

### B3. The one hard part, stated plainly

Calling a C function chosen at runtime needs its ABI, and **doubles do
not travel in the same registers as integers** on any platform the SDK
ships. 82 of Teimeris's functions take doubles by value, so a uniform
word-sized dispatcher is wrong and would be wrong silently.

Two ways, and the plan takes both in order:

1. **Generated dispatch (first).** The adapter's build reads the
   manifest and emits a correctly typed call per function. No runtime
   dependency, full type safety, and it proves the marshalling by role is
   right. New engine functions need an adapter rebuild — no SDK source
   change, but a rebuild.
2. **`libffi` dispatch (then).** A call frame built from the manifest at
   runtime. A consumer drops in a new engine and its manifest and the new
   functions work **with no rebuild at all**, which is the maintainer's
   requirement in full. The dependency belongs to the adapter, never to
   the SDK core, and is behind a cargo feature so a consumer who does not
   want it does not link it.

Shipping (1) first is not a detour: it is how (2) is tested, because both
must produce the same JSON for the same call.

### B4. What a consumer sees

- **Rust:** `context.ephemeris().describe()` and
  `context.ephemeris().call(name, args)`.
- **Node / Python / Dart:** `ctx.ephemeris.native.<name>(…)` through a
  dynamic proxy — a JS `Proxy`, Python `__getattr__`, Dart
  `noSuchMethod` — so a function added to the engine after the binding
  shipped is reachable **without regenerating the binding**. Optional
  typed wrappers can be generated from the manifest for consumers who
  want completions, and are a convenience over the dynamic path rather
  than the only way in.

### B5. What it must not do

- A native result **never enters the completion**. It is the engine's
  answer in the engine's own frame, returned as such. The SDK does not
  pretend it is a port result, does not complete it, and does not stamp
  it as one.
- A native call **is** recorded in provenance, because a result a
  consumer got with one is not reproducible from the settings alone.
- The manifest is the contract: a name not in it is refused, with the
  near matches named, as every other refusal in this SDK is.

## Part C — what the same brief also asks for

### C1. One spelling per name

A name should be spelled in one place. The known live instance:
`Body::key()` hand-writes `"SUN"` while the generated catalogue's
`Graha::Sun` also spells it, and nothing checks that they agree. They are
genuinely different vocabularies — the port's `MEAN_NODE` and
`TRUE_NODE` are both the catalogue's `RAHU` — so the fix is not to merge
them but to **gate the overlap**: every `Body` that maps to a `Graha`
must spell the same key. A `check-names` rule, in the same family as
`knob-has-a-reader`.

### C2. Generated code in other languages

Asked and answered, recorded here so it is not asked twice.
`bindings/python/teistro/__init__.py` is hand-written and is a real
`.py` file. What is generated from Rust is `_ffi.py`, `catalogue.py` and
`_blob.py` — ctypes declarations, 81 enums, blob decoders — and those
must stay generated: hand-writing them guarantees drift, which is what
`check-python` exists to disprove. Moving them to `.py` template files
was considered and rejected on a measurement: the emitters are about
eight lines of logic per line of emitted text, so they are computation
and not templates, and a template engine would cost a dependency and buy
nothing. The generated files are type-checked by mypy in strict mode, so
the emitted code is verified in its own language.

## Sequence

1. **A0** the measurement and this plan — *done*, `check-batching`.
2. **B0** the passthrough measured: what the port reaches of an engine,
   counted rather than asserted — part of the same pass.
3. **A1** grid the searches.
4. **A2** the batch memo and `provider.cache`.
5. **B1** the manifest, `ts_ephemeris_describe`, generated dispatch,
   Rust surface.
6. **B2** the dynamic proxy in the three bindings.
7. **A3** `compute.parallelism` with the threshold measured.
8. **B3** `libffi` dispatch behind a feature.
9. **C1** the `check-names` rule.

Each step regenerates the measured page, so the numbers move in public
and a regression is a failed gate rather than a memory.
