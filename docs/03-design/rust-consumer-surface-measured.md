# The Rust consumer surface, measured

Status: `generated` by `cargo xtask rust-surface`, gated by `check-rust-surface`. Do not edit. Read from `idl/api.json`, the boundary's own description; from `crates/ffi/src/`, for the SDK crates each of its modules calls; from `crates/*/Cargo.toml`, for which crates depend on which; and from `bindings/node/lib/index.js`, the one ergonomic layer whose areas and the entry points they reach can both be read from one file.

ADR-0030 §9 leaves Rust's own consumer surface to the Rust binding's own page, and the obvious proposal is that it mirrors the other three: **8 areas and a root**, 39 operations, over one context. What that proposal is worth depends on how far a Rust consumer is from it today, which is the thing this page measures rather than argues.

The measurement is a composition of two readings the repository already has: the boundary's description says which module each of its 46 entry points came from, and the Node layer says which entry points each area's operations reach ([`surface-areas-measured.md`](surface-areas-measured.md)). Each boundary module names the SDK crates it calls, so an area's operations name the crates behind them.

**A context and its areas need 9 of the SDK's crates**: `teistro-astro`, `teistro-calendar`, `teistro-chart`, `teistro-core`, `teistro-ephemeris-builtin`, `teistro-intl`, `teistro-panchanga`, `teistro-port-ephemeris`, `teistro-time`.

## The properties

| proposed rule | verdict | measured |
|---|---|---|
| an area's operations come from one SDK crate, so a Rust consumer already has the area | falsified | 7 of 8 disagree; more than one: `almanac (5)`, `calendar (2)`, `chart (6)`, `engine (2)`, `frame (2)`, `intl (3)`, `time (3)` |
| every area reaches the boundary, so every area names crates | **holds** | 0 of 9 disagree; so every row of the table below is a measurement and not a gap |
| no crate a context needs is brought in by the boundary alone | falsified | 2 of 9 disagree; only `crates/ffi` depends on `teistro-ephemeris-builtin`, `teistro-intl` |

**2 of the crates a context needs are held by `crates/ffi` and by nothing else** — `teistro-ephemeris-builtin`, `teistro-intl` — so the composition that makes a context is written once, at the C boundary, and a Rust consumer cannot reach it without going through C. This is the finding that decides the question the page was written to ask: a Rust façade would not be a convenience over crates a consumer already composes, it would be the *first* place that composition exists in Rust.

## What each area needs

| area | crates | which |
|---|---|---|
| `(root)` | 4 | `teistro-astro`, `teistro-core`, `teistro-port-ephemeris`, `teistro-time` |
| `almanac` | 5 | `teistro-astro`, `teistro-calendar`, `teistro-core`, `teistro-panchanga`, `teistro-port-ephemeris` |
| `calendar` | 2 | `teistro-calendar`, `teistro-core` |
| `chart` | 6 | `teistro-astro`, `teistro-calendar`, `teistro-chart`, `teistro-core`, `teistro-port-ephemeris`, `teistro-time` |
| `engine` | 2 | `teistro-core`, `teistro-port-ephemeris` |
| `frame` | 2 | `teistro-core`, `teistro-port-ephemeris` |
| `intl` | 3 | `teistro-calendar`, `teistro-core`, `teistro-intl` |
| `keys` | 1 | `teistro-core` |
| `time` | 3 | `teistro-astro`, `teistro-core`, `teistro-time` |

## What each boundary module calls

Read off the source and not the manifest: the manifest says what the boundary *crate* depends on, and this page is about what each operation needs.

| module | calls |
|---|---|
| `blob` | — |
| `calendar` | `teistro-calendar`, `teistro-core` |
| `chart` | `teistro-astro`, `teistro-calendar`, `teistro-chart`, `teistro-core`, `teistro-port-ephemeris`, `teistro-time` |
| `context` | `teistro-astro`, `teistro-core`, `teistro-ephemeris-builtin`, `teistro-intl`, `teistro-port-ephemeris` |
| `ephemeris` | `teistro-core`, `teistro-port-ephemeris` |
| `frame` | `teistro-core`, `teistro-port-ephemeris` |
| `intl` | `teistro-calendar`, `teistro-core`, `teistro-intl` |
| `key` | `teistro-core` |
| `lib` | `teistro-core` |
| `panchanga` | `teistro-astro`, `teistro-calendar`, `teistro-core`, `teistro-panchanga`, `teistro-port-ephemeris` |
| `positions` | `teistro-astro`, `teistro-core`, `teistro-port-ephemeris`, `teistro-time` |
| `provider` | `teistro-core`, `teistro-port-ephemeris` |
| `schemas` | — |
| `string` | `teistro-core` |
| `support` | `teistro-core` |
| `time` | `teistro-astro`, `teistro-core`, `teistro-time` |

## What this does not measure

- **What a Rust façade's signatures would look like.** This page counts crates, which decides *whether* the composition exists in Rust; it says nothing about whether a Rust consumer wants a context object, a builder, or free functions over a settings value. That is the design page's question.
- **What each crate's public API already offers.** A crate behind an area may already expose exactly the operation, or may expose the pieces it is built from. Counting crates cannot tell the two apart.
- **What the C caller's memory costs.** `blob`, `string` and `support` are listed below and add nothing to any area, because no area's operations reach them: they are the C caller's memory and its handshake, and a Rust consumer of the crates has neither — the crates hand back their own types. [`surface-areas-measured.md`](surface-areas-measured.md) calls them plumbing for the same reason. So a Rust façade is smaller than the boundary, not larger.

