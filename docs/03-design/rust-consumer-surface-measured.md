# The Rust consumer surface, measured

Status: `generated` by `cargo xtask rust-surface`, gated by `check-rust-surface`. Do not edit. Read from `idl/api.json`, the boundary's own description; from `crates/ffi/src/`, for what every function of it names and calls; from `crates/*/Cargo.toml`, for which crates depend on which; and from `bindings/node/lib/index.js`, the one ergonomic layer whose areas and the entry points they reach can both be read from one file.

ADR-0030 §9 leaves Rust's own consumer surface to the Rust binding's own page, and the obvious proposal is that it mirrors the other three: **8 areas and a root**, 39 operations, over one context. What that proposal is worth depends on how far a Rust consumer is from it today, which is the thing this page measures rather than argues. The design it decided is [`rust-consumer-surface.md`](rust-consumer-surface.md), and this page's second property is that design's acceptance test.

**The method**, because the number is only as good as it. Every function of the boundary crate is read for the SDK crates it names — by a path, or by a name its module imported — and for the boundary's own functions it calls, resolved through `use crate::<module>::…` and `crate::<module>::<name>` rather than by bare name. Those calls are then closed over until nothing new is added, so a function whose own lines name no crate still reaches whatever it calls: `ts_calendar_convert` names none, calls `system_of`, which calls `teistro-calendar`'s `shipped`. Which of the 46 entry points each area's operations reach comes from the Node layer, as [`surface-areas-measured.md`](surface-areas-measured.md) reads it.

**A context and its areas need 9 of the SDK's crates**: `teistro-astro`, `teistro-calendar`, `teistro-chart`, `teistro-core`, `teistro-ephemeris-builtin`, `teistro-intl`, `teistro-panchanga`, `teistro-port-ephemeris`, `teistro-time`.

## The properties

| proposed rule | verdict | measured |
|---|---|---|
| an area's operations come from one SDK crate, so a Rust consumer already has the area | falsified | 7 of 8 disagree; more than one: `almanac (7)`, `calendar (2)`, `chart (6)`, `engine (2)`, `frame (2)`, `intl (3)`, `time (4)` |
| every area reaches the boundary, so every area names crates | **holds** | 0 of 9 disagree; so every row of the table below is a measurement and not a gap |
| an entry point's work reaches one SDK crate, so a façade over it is a rename | falsified | 23 of 46 disagree; 23 reach two or more; 8 reach none at all, and those are the C caller's memory: `ts_abi_version`, `ts_sdk_version`, `ts_default_profile`, `ts_build_info`, `ts_string_free`, `ts_blob_free`, `ts_context_free`, `ts_provider_free` |
| the façade owns the composition: every crate a context needs is one it depends on | **holds** | 0 of 9 disagree; so every area's composition has a home outside the C boundary |
| and the boundary is inverted onto it, so the composition is written once | falsified | 1 of 1 disagree; `teistro-ffi` does not depend on `teistro` yet |

**The façade owns the composition and the boundary has not been inverted onto it**, so it is written twice — knowingly, and only until step 3 of [the design page](rust-consumer-surface.md)'s order of work.

## What each area needs

| area | crates | which |
|---|---|---|
| `(root)` | 4 | `teistro-astro`, `teistro-core`, `teistro-port-ephemeris`, `teistro-time` |
| `almanac` | 7 | `teistro-astro`, `teistro-calendar`, `teistro-chart`, `teistro-core`, `teistro-panchanga`, `teistro-port-ephemeris`, `teistro-time` |
| `calendar` | 2 | `teistro-calendar`, `teistro-core` |
| `chart` | 6 | `teistro-astro`, `teistro-calendar`, `teistro-chart`, `teistro-core`, `teistro-port-ephemeris`, `teistro-time` |
| `engine` | 2 | `teistro-core`, `teistro-port-ephemeris` |
| `frame` | 2 | `teistro-core`, `teistro-port-ephemeris` |
| `intl` | 3 | `teistro-calendar`, `teistro-core`, `teistro-intl` |
| `keys` | 1 | `teistro-core` |
| `time` | 4 | `teistro-astro`, `teistro-calendar`, `teistro-core`, `teistro-time` |

## What each entry point reaches

Widest first. Read through the boundary's own helpers, because a body that names no crate still reaches whatever it calls: `ts_calendar_convert` names none, calls `system_of`, which calls `teistro-calendar`'s `shipped`.

| entry point | module | crates | which |
|---|---|---|---|
| `ts_panchanga_days` | `panchanga` | 7 | `teistro-astro`, `teistro-calendar`, `teistro-chart`, `teistro-core`, `teistro-panchanga`, `teistro-port-ephemeris`, `teistro-time` |
| `ts_chart_found` | `chart` | 6 | `teistro-astro`, `teistro-calendar`, `teistro-chart`, `teistro-core`, `teistro-port-ephemeris`, `teistro-time` |
| `ts_context_new` | `context` | 5 | `teistro-astro`, `teistro-core`, `teistro-ephemeris-builtin`, `teistro-intl`, `teistro-port-ephemeris` |
| `ts_positions` | `positions` | 4 | `teistro-astro`, `teistro-core`, `teistro-port-ephemeris`, `teistro-time` |
| `ts_time_civil` | `time` | 4 | `teistro-astro`, `teistro-calendar`, `teistro-core`, `teistro-time` |
| `ts_intl_render` | `intl` | 3 | `teistro-calendar`, `teistro-core`, `teistro-intl` |
| `ts_time_convert` | `time` | 3 | `teistro-astro`, `teistro-core`, `teistro-time` |
| `ts_time_delta_t` | `time` | 3 | `teistro-astro`, `teistro-core`, `teistro-time` |
| `ts_time_resolve` | `time` | 3 | `teistro-astro`, `teistro-core`, `teistro-time` |
| `ts_calendar_convert` | `calendar` | 2 | `teistro-calendar`, `teistro-core` |
| `ts_calendar_from_fixed` | `calendar` | 2 | `teistro-calendar`, `teistro-core` |
| `ts_calendar_is_leap` | `calendar` | 2 | `teistro-calendar`, `teistro-core` |
| `ts_calendar_month_length` | `calendar` | 2 | `teistro-calendar`, `teistro-core` |
| `ts_calendar_to_fixed` | `calendar` | 2 | `teistro-calendar`, `teistro-core` |
| `ts_calendar_weekday` | `calendar` | 2 | `teistro-calendar`, `teistro-core` |
| `ts_context_new_with_provider` | `provider` | 2 | `teistro-core`, `teistro-port-ephemeris` |
| `ts_ephemeris_call` | `ephemeris` | 2 | `teistro-core`, `teistro-port-ephemeris` |
| `ts_ephemeris_manifest` | `ephemeris` | 2 | `teistro-core`, `teistro-port-ephemeris` |
| `ts_frame_canonical` | `frame` | 2 | `teistro-core`, `teistro-port-ephemeris` |
| `ts_frame_pack` | `frame` | 2 | `teistro-core`, `teistro-port-ephemeris` |
| `ts_frame_unpack` | `frame` | 2 | `teistro-core`, `teistro-port-ephemeris` |
| `ts_intl_transliterate` | `intl` | 2 | `teistro-core`, `teistro-intl` |
| `ts_provider_load` | `provider` | 2 | `teistro-core`, `teistro-port-ephemeris` |
| `ts_calendar_fixed_of_jd` | `calendar` | 1 | `teistro-calendar` |
| `ts_calendar_jd_of_fixed` | `calendar` | 1 | `teistro-calendar` |
| `ts_catalogue_version` | `lib` | 1 | `teistro-core` |
| `ts_context_last_error` | `context` | 1 | `teistro-core` |
| `ts_context_profile` | `context` | 1 | `teistro-core` |
| `ts_context_settings_hash` | `context` | 1 | `teistro-core` |
| `ts_context_settings_json` | `context` | 1 | `teistro-core` |
| `ts_intl_entity` | `intl` | 1 | `teistro-core` |
| `ts_intl_has` | `intl` | 1 | `teistro-core` |
| `ts_intl_load_pack` | `intl` | 1 | `teistro-core` |
| `ts_intl_locale` | `intl` | 1 | `teistro-core` |
| `ts_intl_set_locale` | `intl` | 1 | `teistro-core` |
| `ts_key_name` | `key` | 1 | `teistro-core` |
| `ts_key_parse` | `key` | 1 | `teistro-core` |
| `ts_status_message` | `lib` | 1 | `teistro-core` |
| `ts_abi_version` | `lib` | 0 | — |
| `ts_blob_free` | `blob` | 0 | — |
| `ts_build_info` | `lib` | 0 | — |
| `ts_context_free` | `context` | 0 | — |
| `ts_default_profile` | `lib` | 0 | — |
| `ts_provider_free` | `provider` | 0 | — |
| `ts_sdk_version` | `lib` | 0 | — |
| `ts_string_free` | `string` | 0 | — |

## What this does not measure

- **What a Rust façade's signatures would look like.** This page counts crates, which decides *whether* the composition exists in Rust; it says nothing about whether a Rust consumer wants a context object, a builder, or free functions over a settings value. That is the design page's question.
- **What each crate's public API already offers.** A crate behind an area may already expose exactly the operation, or may expose the pieces it is built from. Counting crates cannot tell the two apart.
- **What a build script composes.** The locale bundles are built into `crates/ffi` by its own build script and reach the source as a `pub(crate) static` in `OUT_DIR`, so no reading of dependencies or of function bodies sees them. The façade needs them, and [the design page](rust-consumer-surface.md) says where they go.
- **A crate reached by inference alone.** A name resolves here through an import, a path, or the module it is declared in; a method called on a value whose type is never written, in a module that imports nothing of that crate, is invisible. Nothing in the boundary is written that way today — the readings agree with a second, coarser one that attributes a whole module's imports to each of its entry points — but a body that grew that shape would read low rather than loudly, so it is said here.
- **What the C caller's memory costs.** `blob`, `string` and `support` are listed below and add nothing to any area, because no area's operations reach them: they are the C caller's memory and its handshake, and a Rust consumer of the crates has neither — the crates hand back their own types. [`surface-areas-measured.md`](surface-areas-measured.md) calls them plumbing for the same reason. So a Rust façade is smaller than the boundary, not larger.

