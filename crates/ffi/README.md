# teistro-ffi

Status: `draft`, 2026-09-06.

The C ABI of the Teistro SDK: the one audited boundary every binding is
generated against (ADR-0001, ADR-0007). Built as a C library
(`cdylib`, `staticlib`) and as a Rust crate for the tests. The header is
`bindings/c/include/teistro.h`, the description `idl/api.json`, both
generated from this crate's source by `cargo xtask gen ffi`.

| module | entry points |
|---|---|
| `lib` | `ts_abi_version`, `ts_sdk_version`, `ts_catalogue_version`, `ts_default_profile`, `ts_status_message` |
| `context` | `ts_context_new`, `ts_context_free`, `ts_context_last_error`, `ts_context_profile`, `ts_context_settings_json`, `ts_context_settings_hash` |
| `strings`, `blob` | `ts_string_free`, `ts_blob_free` |
| `keys` | `ts_key_parse`, `ts_key_name` |
| `calendar` | `ts_calendar_from_fixed`, `ts_calendar_to_fixed`, `ts_calendar_convert`, `ts_calendar_month_length`, `ts_calendar_is_leap`, `ts_calendar_weekday`, `ts_calendar_jd_of_fixed`, `ts_calendar_fixed_of_jd` |
| `time` | `ts_time_resolve`, `ts_time_civil`, `ts_time_convert`, `ts_time_delta_t` |
| `intl` | `ts_intl_load_pack`, `ts_intl_set_locale`, `ts_intl_locale`, `ts_intl_has`, `ts_intl_render` |
| `positions` | `ts_positions` |

The design page is `docs/03-design/ffi-abi-and-api-description.md`.
