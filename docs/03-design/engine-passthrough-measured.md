# The engine passthrough, measured

Status: `generated` by `cargo xtask engine`, gated by `check-engine`. Do not edit. Read from `adapters/ephemeris-teimeris/rust/data/teimeris.idl`, the engine's own generated description, vendored beside the adapter at version `0.1.0`.

ADR-0030 puts an engine's own functions at `sdk.engine.*`, so that a consumer can reach what the SDK has not ported. The port says how — "the marshalling belongs to the adapter, generated from the engine's manifest" — and this is the measurement that generation is sized from.

The engine describes **161 functions**, beside 57 structs, 40 enums and 2 callbacks.

| standing | functions | what it means |
|---|---:|---|
| **callable** | 42 | offered at `sdk.engine.*` today |
| **the adapter's own** | 11 | never offered, whatever their shape |
| **not yet marshalled** | 108 | a queue for the generator, not a refusal |

## What the adapter will not hand over

**eleven functions, excluded by what they touch rather than by what they cost.** Each opens, closes or rebinds the context the adapter is holding, or the data bound to it for that context's life. A consumer who called one through the passthrough would close the context every other call depends on, or change the files underneath it.

They are named here rather than left to the marshaller to fail on, because the reason is a **boundary** and not a difficulty — teaching the generator more shapes must never bring them in.

- `tm_context_open`
- `tm_context_close`
- `tm_data_source_from_memory`
- `tm_data_source_from_path`
- `tm_data_source_custom`
- `tm_data_source_close`
- `tm_data_source_name`
- `tm_data_source_size`
- `tm_data_source_map`
- `tm_data_source_read`
- `tm_context_set_fetch`

## What is callable

**42 functions**, every one of them taking and returning scalars and enums alone, which is the shape a JSON object carries without a marshaller having to know anything else.

| function | changes engine state |
|---|---|
| `tm_context_release_caches` |  |
| `tm_version` |  |
| `tm_delta_t` |  |
| `tm_sidereal_time` |  |
| `tm_day_of_week` |  |
| `tm_equation_of_time` |  |
| `tm_local_mean_to_apparent` |  |
| `tm_local_apparent_to_mean` |  |
| `tm_house_cusp_count` |  |
| `tm_house_system_count` |  |
| `tm_house_position` |  |
| `tm_jpl_info` |  |
| `tm_ayanamsha_count` |  |
| `tm_set_model` | **yes** |
| `tm_get_model` |  |
| `tm_model_count` |  |
| `tm_set_tidal_acceleration` | **yes** |
| `tm_clear_tidal_acceleration` | **yes** |
| `tm_get_tidal_acceleration` |  |
| `tm_set_delta_t_override` | **yes** |
| `tm_clear_delta_t_override` | **yes** |
| `tm_get_delta_t_override` |  |
| `tm_set_nutation_interpolation` | **yes** |
| `tm_get_nutation_interpolation` |  |
| `tm_embedded_coverage` |  |
| `tm_fallback_stats_reset` |  |
| `tm_star_count` |  |
| `tm_angle_normalize_deg` |  |
| `tm_angle_normalize_rad` |  |
| `tm_angle_diff_deg` |  |
| `tm_angle_diff_deg_positive` |  |
| `tm_angle_diff_rad` |  |
| `tm_angle_midpoint_deg` |  |
| `tm_angle_midpoint_rad` |  |
| `tm_degrees_to_centiseconds` |  |
| `tm_centiseconds_to_degrees` |  |
| `tm_centiseconds_normalize` |  |
| `tm_centiseconds_diff` |  |
| `tm_centiseconds_diff_signed` |  |
| `tm_centiseconds_round_seconds` |  |
| `tm_round_half_away` |  |
| `tm_chart_blob_size` |  |

### The six that change engine state

These are **offered rather than refused**, and the column above is why the distinction is drawn at all. Reaching what the SDK has not ported is the point of the namespace; but after one of these the engine is answering under settings the SDK's own provenance does not record, so a chart cast afterwards says it was computed one way and was computed another.

A consumer who wants the change *recorded* has the settings for it (ADR-0013's override policy), and one who wants it anyway can have it and knows what it costs. What the SDK will not do is make the choice quietly on their behalf.

## What the marshaller has not learned

**108 functions**, grouped by what stands in the way. This is a queue rather than a refusal: each group is one shape the generator has to learn, and learning one brings its whole group in at once.

| what it takes or returns | functions | examples |
|---|---:|---|
| a pointer it returns | 2 | `tm_last_error`, `tm_embedded_files` |
| a string it fills | 3 | `tm_body_name`, `tm_cache_dir_default`, `tm_angle_format` |
| a string it reads | 11 | `tm_set_jpl_file`, `tm_embedded_find`, `tm_context_add_source` |
| a string it returns | 12 | `tm_status_name`, `tm_version_string`, `tm_weekday_name` |
| a struct | 57 | `tm_config_init_sized`, `tm_julian_day`, `tm_calendar_date` |
| a type that is not a plain scalar | 3 | `tm_position_value`, `tm_set_ayanamsha`, `tm_ayanamsha_value` |
| an array | 18 | `tm_julian_day_many`, `tm_calendar_date_many`, `tm_delta_t_many` |
| opaque bytes | 2 | `tm_chart_blob_info`, `tm_chart_decode` |

The order to learn them in is **not** the order of that table by size. A string the engine *returns* is one `CStr` read and brings its whole group at once; a string it *fills* and an array it fills share one protocol — ask the size, allocate, ask again — so whichever is learned first brings the other nearly free; a struct is fifty-seven field lists and is the largest group because it is the largest job. Opaque bytes are last and may stay there: what crosses is a buffer, and a buffer has no meaning in a JSON object.

## What this does not measure

**Whether a callable function answers correctly.** This reads a description and classifies shapes; it does not call anything. What the marshalling produces is tested against the engine where the adapter's own tests run, and a function's presence here is a claim about its *shape* alone.

**Any engine but this one.** The classification is of one vendored description. Another engine that answers `native_manifest` reaches `sdk.engine.call` by the dynamic route with no generation at all, and gets no typed façade until someone generates one from its description.

**Whether a consumer should use any of it.** A call through this namespace is to a named engine and does not survive changing it — which is why the namespace is called `engine` and not `ephemeris` (ADR-0030). What proves universal is promoted into the port, and then it is portable and this page is no longer where it lives.

