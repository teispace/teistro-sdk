# The engine passthrough, measured

Status: `generated` by `cargo xtask engine`, gated by `check-engine`. Do not edit. Read from `adapters/ephemeris-teimeris/rust/data/teimeris.idl`, the engine's own generated description, vendored beside the adapter at version `0.1.0`. The same reading writes `adapters/ephemeris-teimeris/rust/src/dispatch.rs`, so a figure here and the code that answers it cannot disagree.

ADR-0030 puts an engine's own functions at `sdk.engine.*`, so that a consumer can reach what the SDK has not ported. The port says how — "the marshalling belongs to the adapter, generated from the engine's manifest" — and this is the measurement that generation is sized from.

The engine describes **161 functions**, beside 57 structs, 40 enums and 2 callbacks.

| standing | functions | what it means |
|---|---:|---|
| **callable** | 138 | offered at `sdk.engine.*` today |
| **the adapter's own** | 12 | never offered, whatever their shape |
| **not yet marshalled** | 11 | a queue for the generator, not a refusal |

## What the adapter will not hand over

**twelve functions, excluded by what they touch rather than by what they cost.** Each opens, closes or rebinds the context the adapter is holding, or the data bound to it for that context's life. A consumer who called one through the passthrough would close the context every other call depends on, or change the files underneath it.

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
- `tm_context_add_source`
- `tm_context_set_fetch`

## What is callable

**138 functions**. 45 of them take and answer scalars and enums alone, which is the shape a JSON object carries without a marshaller having to know anything else.

26 carry a string: nine read one the caller passes, five fill a buffer of the marshaller's, and twelve answer with one the engine lends and the marshaller copies before anything else can move it.

65 carry a struct, which crosses as a JSON object keyed by the engine's own field names, nested as the struct nests; eight of them carry a string as well. Every field is required going in, and the struct's own `struct_size` crosses in neither direction — the arm fills it, because the engine reads the struct only as far as it says (`03-design/engine-passthrough.md`).

32 carry an array, which crosses as a JSON array of whatever its element crosses as. The engine's description says how long each output is, and the marshaller sizes it from that and nothing else: 17 are as long as the inputs they answer or a field of the request says, two are as long as another function answers — the house cusps, `tm_house_cusp_count()` of the requested system, asked before the call — eleven are as long as the caller asks — how many eclipses to find, which is an argument — and cut to the count the engine gives, and four are as long as the engine says there are, which the marshaller learns by asking and asks again when there are more than fitted. No caller passes a capacity for an answer whose length is already decided.

| function | carries | changes engine state |
|---|---|---|
| `tm_status_name` | a string it lends |  |
| `tm_context_release_caches` |  |  |
| `tm_version_string` | a string it lends |  |
| `tm_version` |  |  |
| `tm_body_name` | a string it fills |  |
| `tm_julian_day` | a struct it reads |  |
| `tm_calendar_date` | a struct it fills |  |
| `tm_utc_to_jd` | a struct it reads |  |
| `tm_jd_to_utc` | a struct it fills |  |
| `tm_delta_t` |  |  |
| `tm_sidereal_time` |  |  |
| `tm_day_of_week` |  |  |
| `tm_weekday_name` | a string it lends |  |
| `tm_julian_day_many` | an array it reads, an array it fills |  |
| `tm_calendar_date_many` | an array it reads, an array it fills |  |
| `tm_delta_t_many` | an array it reads, an array it fills |  |
| `tm_sidereal_time_many` | an array it reads, an array it fills |  |
| `tm_equation_of_time` |  |  |
| `tm_local_mean_to_apparent` |  |  |
| `tm_local_apparent_to_mean` |  |  |
| `tm_local_to_utc` | a struct it reads, a struct it fills |  |
| `tm_utc_to_local` | a struct it reads, a struct it fills |  |
| `tm_position_calc` | a struct it reads, a struct it fills |  |
| `tm_position_value` |  |  |
| `tm_position_calc_many` | an array it reads, an array it fills |  |
| `tm_position_calc_grid` | a struct it reads, an array it reads, an array it fills |  |
| `tm_house_cusp_count` |  |  |
| `tm_house_system_name` | a string it lends |  |
| `tm_house_system_count` |  |  |
| `tm_houses_calc` | a struct it reads, a struct it fills, an array it fills |  |
| `tm_house_position` |  |  |
| `tm_chart_default_bodies` | an array it fills |  |
| `tm_chart_calc` | a struct it reads, a struct it fills, an array it reads, an array it fills |  |
| `tm_set_jpl_file` | a string it reads | **yes** |
| `tm_jpl_info` |  |  |
| `tm_set_ayanamsha` |  | **yes** |
| `tm_ayanamsha_value` |  |  |
| `tm_ayanamsha_name` | a string it lends |  |
| `tm_ayanamsha_count` |  |  |
| `tm_set_model` |  | **yes** |
| `tm_get_model` |  |  |
| `tm_model_name` | a string it lends |  |
| `tm_model_kind_name` | a string it lends |  |
| `tm_model_count` |  |  |
| `tm_model_info` | a struct it fills |  |
| `tm_set_tidal_acceleration` |  | **yes** |
| `tm_clear_tidal_acceleration` |  | **yes** |
| `tm_get_tidal_acceleration` |  |  |
| `tm_set_delta_t_override` |  | **yes** |
| `tm_clear_delta_t_override` |  | **yes** |
| `tm_get_delta_t_override` |  |  |
| `tm_set_nutation_interpolation` |  | **yes** |
| `tm_get_nutation_interpolation` |  |  |
| `tm_atmosphere_init_sized` | a struct it fills |  |
| `tm_refract` | a struct it reads, a struct it fills |  |
| `tm_refract_many` | a struct it reads, an array it reads, an array it fills |  |
| `tm_obliquity_calc` | a struct it fills |  |
| `tm_coord_rotate` | an array it reads, an array it fills |  |
| `tm_to_horizontal` | a struct it reads, a struct it fills |  |
| `tm_to_horizontal_many` | a struct it reads, an array it reads, an array it fills |  |
| `tm_from_horizontal` | a struct it reads |  |
| `tm_embedded_coverage` |  |  |
| `tm_body_coverage` | a struct it fills |  |
| `tm_loaded_files` | an array it fills |  |
| `tm_last_loaded_file` | a struct it fills |  |
| `tm_fallback_stats_get` | a struct it fills |  |
| `tm_fallback_stats_reset` |  |  |
| `tm_cache_dir_default` | a string it fills |  |
| `tm_nodes_apsides_calc` | a struct it reads, a struct it fills |  |
| `tm_nodes_apsides_calc_many` | a struct it reads, an array it reads, an array it fills |  |
| `tm_orbital_elements_calc` | a struct it fills |  |
| `tm_orbital_elements_calc_many` | an array it reads, an array it fills |  |
| `tm_orbit_distances_calc` | a struct it fills |  |
| `tm_orbit_distances_calc_many` | an array it reads, an array it fills |  |
| `tm_phenomena_calc` | a struct it reads, a struct it fills |  |
| `tm_phenomena_calc_many` | a struct it reads, an array it reads, an array it fills |  |
| `tm_star_name` | a string it fills, a struct it reads |  |
| `tm_star_designation` | a string it fills, a struct it reads |  |
| `tm_star_load_catalogue` | a string it reads | **yes** |
| `tm_star_count` |  |  |
| `tm_star_find` | a string it reads, a struct it fills |  |
| `tm_star_find_all` | a string it reads, an array it fills |  |
| `tm_star_query_init_sized` | a struct it fills |  |
| `tm_star_search` | a struct it reads, an array it fills |  |
| `tm_star_calc` | a string it reads, a struct it reads, a struct it fills |  |
| `tm_star_calc_many` | a struct it reads, an array it reads, an array it fills |  |
| `tm_eclipse_type_name` | a string it lends |  |
| `tm_eclipse_request_init_sized` | a struct it fills |  |
| `tm_solar_eclipse_search` | a struct it reads, an array it fills |  |
| `tm_lunar_eclipse_search` | a struct it reads, an array it fills |  |
| `tm_occultation_search` | a struct it reads, an array it fills |  |
| `tm_solar_eclipse_search_local` | a struct it reads, an array it fills |  |
| `tm_lunar_eclipse_search_local` | a struct it reads, an array it fills |  |
| `tm_occultation_search_local` | a struct it reads, an array it fills |  |
| `tm_solar_eclipse_where` | a struct it fills |  |
| `tm_occultation_where` | a string it reads, a struct it fills |  |
| `tm_solar_eclipse_how` | a struct it reads, a struct it fills |  |
| `tm_occultation_how` | a string it reads, a struct it reads, a struct it fills |  |
| `tm_lunar_eclipse_how` | a struct it reads, a struct it fills |  |
| `tm_event_kind_name` | a string it lends |  |
| `tm_event_request_init_sized` | a struct it fills |  |
| `tm_event_search` | a struct it reads, an array it fills |  |
| `tm_gauquelin_sector` | a string it reads, a struct it reads |  |
| `tm_visibility_defaults` | a struct it reads, a struct it fills |  |
| `tm_visibility_limit` | a string it reads, a struct it reads, a struct it fills |  |
| `tm_visibility_arcus` | a struct it reads |  |
| `tm_visibility_best_altitude` | a struct it reads, a struct it fills |  |
| `tm_heliacal_request_init_sized` | a struct it fills |  |
| `tm_heliacal_search` | a struct it reads, an array it fills |  |
| `tm_heliacal_detail` | a struct it reads, a struct it fills |  |
| `tm_crossing_request_init_sized` | a struct it fills |  |
| `tm_crossing_search` | a struct it reads, an array it fills |  |
| `tm_node_crossing_search` | a struct it reads, an array it fills |  |
| `tm_calendar_request_init_sized` | a struct it fills |  |
| `tm_calendar_grid` | a struct it reads, an array it reads, an array it fills |  |
| `tm_scan_request_init_sized` | a struct it fills |  |
| `tm_scan_grid` | a struct it reads, an array it fills |  |
| `tm_angle_normalize_deg` |  |  |
| `tm_angle_normalize_rad` |  |  |
| `tm_angle_diff_deg` |  |  |
| `tm_angle_diff_deg_positive` |  |  |
| `tm_angle_diff_rad` |  |  |
| `tm_angle_midpoint_deg` |  |  |
| `tm_angle_midpoint_rad` |  |  |
| `tm_degrees_to_centiseconds` |  |  |
| `tm_centiseconds_to_degrees` |  |  |
| `tm_centiseconds_normalize` |  |  |
| `tm_centiseconds_diff` |  |  |
| `tm_centiseconds_diff_signed` |  |  |
| `tm_centiseconds_round_seconds` |  |  |
| `tm_round_half_away` |  |  |
| `tm_angle_split` | a struct it fills |  |
| `tm_angle_format` | a string it fills |  |
| `tm_zodiac_sign_name` | a string it lends |  |
| `tm_nakshatra_name` | a string it lends |  |
| `tm_nakshatra_lord` | a string it lends |  |
| `tm_chart_blob_size` |  |  |
| `tm_ayanamsha_info` | a struct it fills |  |

### The nine that change engine state

These are **offered rather than refused**, and the column above is why the distinction is drawn at all. Reaching what the SDK has not ported is the point of the namespace; but after one of these the engine is answering under settings the SDK's own provenance does not record, so a chart cast afterwards says it was computed one way and was computed another.

A consumer who wants the change *recorded* has the settings for it (ADR-0013's override policy), and one who wants it anyway can have it and knows what it costs. What the SDK will not do is make the choice quietly on their behalf.

## What the marshaller has not learned

**eleven functions**, grouped by the hardest thing in the way and listed easiest first. This is a queue rather than a refusal: each group is one shape the generator has to learn, and learning one brings its whole group in at once.

| what it takes or returns | functions | examples |
|---|---:|---|
| an output whose length it cannot compute | 1 | `tm_houses_calc_many` |
| opaque bytes | 3 | `tm_chart_encode`, `tm_chart_blob_info`, `tm_chart_decode` |
| a struct no JSON object describes | 4 | `tm_config_init_sized`, `tm_position_calc_grid_columns`, `tm_fetch_config_init_sized` |
| a pointer it returns | 3 | `tm_last_error`, `tm_embedded_files`, `tm_embedded_find` |

The order of work, with what each step releases, is in `03-design/engine-passthrough.md` §6; the figures there were measured by the same classification as this table.

### The outputs it cannot size

**five output arrays**, each with the engine's own account of what decides its length. None is guessed: an output sized wrongly is a truncated answer at best, and at worst a refusal the caller has no way to fix.

| function | output | its length is |
|---|---|---|
| `tm_houses_calc_many` | `cusps` | count times the widest requested system's cusp count |
| `tm_chart_encode` | `out_blob` | the encoded size, which the call returns only once it fits |
| `tm_chart_decode` | `out_bodies` | what the blob holds; tm_chart_blob_info() |
| `tm_chart_decode` | `out_positions` | what the blob holds; tm_chart_blob_info() |
| `tm_chart_decode` | `out_cusps` | what the blob holds; tm_chart_blob_info() |

## What the typed façade hands back

ADR-0030 puts a typed façade in the **adapter's** package, generated from this same reading so that it cannot type an argument the dispatch would refuse by name. What a method answers with is decided here, by counting:

| values answered | functions | the façade's answer |
|---:|---:|---|
| 1 | 110 | **the value itself** — a number, a string, a struct |
| 0 | 11 | nothing |
| more | 17 | a record: an object, a Dart record, a `TypedDict` |

**110 of the 138 answer with exactly one value**, so a façade that always returned an object would have made every one of them an indexing exercise for the sake of 17. Those 17 get a record apiece — `tm_utc_to_jd`, `tm_get_tidal_acceleration`, `tm_get_delta_t_override`, `tm_from_horizontal`, `tm_embedded_coverage`, `tm_solar_eclipse_how`, `tm_occultation_how`, `tm_lunar_eclipse_how`, `tm_visibility_defaults`, `tm_calendar_grid`, `tm_version`, `tm_houses_calc`, `tm_solar_eclipse_where`, `tm_occultation_where`, `tm_scan_grid`, `tm_chart_calc`, `tm_jpl_info` — which is 17 types per target rather than 138.

The same count settles a question every target would otherwise have raised. `return` — the key a function's own return value comes back under — **never appears beside another key**: every one of those 17 is a status-returning function with out-parameters. So no record field is ever named `return`, and no target has to rename a keyword it could not spell.

## What this does not measure

**Whether a callable function answers correctly.** This reads a description and classifies shapes; it does not call anything. What the marshalling produces is tested against the engine where the adapter's own tests run, and a function's presence here is a claim about its *shape* alone.

**Any engine but this one.** The classification is of one vendored description. Another engine that answers `native_manifest` reaches `sdk.engine.call` by the dynamic route with no generation at all, and gets no typed façade until someone generates one from its description.

**Whether a consumer should use any of it.** A call through this namespace is to a named engine and does not survive changing it — which is why the namespace is called `engine` and not `ephemeris` (ADR-0030). What proves universal is promoted into the port, and then it is portable and this page is no longer where it lives.

