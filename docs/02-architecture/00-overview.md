# Architecture overview

Status: `draft`, revised 2026-09-04. Q1 is decided (Rust core with a C
ABI, ADR-0001); Q2 in principle (one description, generated bindings; the
generator is chosen by the Phase 0 spike, ADR-0004); Q7 and Q8 add the
astronomy layer and the built-in ephemeris (ADR-0008, ADR-0009); Q10 sets
Teistro Intl as the localisation standard (ADR-0010).

## The shape in one diagram

```
 consumer application (Node, browser, Flutter, Python, Rust, Java, C)
   │  idiomatic API: Context, settings, requests, results, typed intl accessors
   ▼
 binding: ergonomic layer (hand-written, thin, parity-gated)
   │
 binding: mechanical layer (GENERATED from the API description)
   │  C ABI: extern "C", struct_size, capacities, structured errors,
   │         batch shapes, columnar arrays, result blobs
   ▼
 ffi crate ──────────────────────────────────────────────────────────┐
   │  catch_unwind, validation, marshalling, callback trampolines     │
   ▼                                                                  │
 core crates (pure Rust, #![forbid(unsafe_code)])                     │
   ┌──────────────┬───────────────┬──────────────┬─────────────────┐  │
   │ interpret    │ intl          │ serial       │ (presentation)  │  │  L3
   ├──────────────┴───────────────┴──────────────┴─────────────────┤  │
   │ domain: chart, houses, vargas, state, aspect, points,         │  │
   │ strength, dasha, rules, jaimini, kp, tajika, panchanga,       │  │  L2
   │ muhurta, gochar, prashna, matching, rectification, longevity, │  │
   │ remedies, numerology, lalkitab, pakshi, namakarana, rashifal, │  │
   │ research, calendar, time, western*, hellenistic*              │  │
   ├───────────────────────────────────────────────────────────────┤  │
   │ astro: timescales, Delta T, precession, nutation, obliquity,  │  │
   │ sidereal time, frame completion, topocentric, transforms,     │  │  L1.5
   │ ayanamsha catalogue, house systems, rise/set, crossings,      │  │
   │ stations, star table, eclipses*                               │  │
   ├───────────────────────────────────────────────────────────────┤  │
   │ ports: ephemeris (positions required, overrides optional),    │  │  L1
   │ calendar, timezone, geo, intl-data, log                       │  │
   ├───────────────────────────────────────────────────────────────┤  │
   │ core: types, keys, entity catalogue, angles, time, settings,  │  │  L0
   │ profiles, errors, capabilities, registries, result envelope   │  │
   └───────────────────────────────────────────────────────────────┘  │
   ▲                                                                  │
   │ providers: the SDK's own ephemeris-builtin (VSOP87, ELP/MPP02,    │
   │ tiers) and siddhanta modules; the Teimeris adapter; Swiss and     │
   │ other adapters; tzdb and geo providers; intl packs;               │
   │ interpretation packs; rule packs ◄────────────────────────────────┘
```

`*` designed in v1.0, shipped in v1.x.

## The rules that hold it together

1. **Layers only depend downward.** L3 may use L2 and below; L2 uses
   `astro`, the ports and `core`, never L3; `astro` uses the ephemeris port
   and `core` only; L1 defines interfaces; L0 depends on nothing but the
   standard library and a few vetted crates. A gate enumerates the graph.
2. **No computation module knows about text.** Results are keys and
   numbers. `interpret` maps results to narrative plans; `intl` renders;
   `serial` exports. Languages are data.
3. **The SDK owns the astronomy above raw positions.** A provider supplies
   positions (and may supply native overrides); `astro` completes frames,
   computes ayanamshas, houses, events and crossings; so a chart means the
   same thing on every provider, and the built-in provider makes the SDK
   work with nothing else installed.
4. **The context is the unit of everything.** Settings profile, providers,
   packs, caches, limits, locale; no global state; one context per thread;
   pools in bindings.
5. **Every computation is foundation plus slice**, memoised on the context.
6. **Ports and providers are capability-negotiated**; overrides are used
   when declared and allowed; results stamp which implementation ran.
7. **Results carry provenance**: SDK and module versions, profile hash,
   provider identity and tier, corrections applied, pack versions, content
   hash.
8. **Extension points are registries in L0** populated at context creation.
9. **One i18n standard** for the SDK's text and, if the consumer wishes,
   for the application's own text: Teistro Intl.

## A request, end to end

1. The consumer creates a `Context` with a profile, a provider (the
   built-in ephemeris by default, or the Teimeris adapter), and intl packs.
2. The consumer calls `chart.full(birth, options)`.
3. The ergonomic layer validates and calls the generated mechanical layer,
   which calls the C ABI entry point.
4. The ffi crate validates sizes, ranges and capacities and calls
   `chart::full`.
5. `chart::full` obtains the `Foundation` from the memo, computing it if
   absent: one `positions` grid from the provider, frame completion and the
   sidereal transform in `astro`, cusps from `astro` (or the provider's
   native houses when declared and allowed), then the slices.
6. The result is written to columnar and blob outputs; the ffi crate stamps
   provenance.
7. The ergonomic layer decodes lazily; `context.intl` renders any narrative
   plan or key in the chosen locale through the generated typed accessors.

## Pages

| page | depends on decision |
|---|---|
| [`01-module-catalog.md`](01-module-catalog.md) | none open |
| [`02-ephemeris-port.md`](02-ephemeris-port.md) | Q17 (built-in tiers) |
| [`03-localization-architecture.md`](03-localization-architecture.md) | Q20, Q21 |
| [`04-calendar-time-architecture.md`](04-calendar-time-architecture.md) | none open |
| [`05-data-model-identifiers.md`](05-data-model-identifiers.md) | none open |
| [`06-api-conventions.md`](06-api-conventions.md) | ADR-0007 (generator) |
| [`07-binding-architecture.md`](07-binding-architecture.md) | ADR-0007, Q3 |
| [`08-extensibility.md`](08-extensibility.md) | none |
| [`09-performance-architecture.md`](09-performance-architecture.md) | none |
| [`10-security-architecture.md`](10-security-architecture.md) | none |
