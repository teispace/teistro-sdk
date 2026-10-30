# The surface areas, measured

Status: `generated` by `cargo xtask areas`, gated by `check-areas`. Do not edit. Read from `idl/api.json`, the boundary's own description, and from `bindings/node/lib/index.js`, the one ergonomic layer whose areas and the entry points they reach can both be read from one file.

The surface is `sdk.<area>.<operation>` (ADR-0030, designed in [`surface-areas.md`](surface-areas.md)). This measured the grouping **before** it was built, and falsified the rule the ADR words it by: five boundary modules were never reached by anything a consumer called, one member reached two, and the areas had to be chosen with the derivation as evidence rather than derived from it. That argument is made and [the design page](surface-areas.md) keeps it.

What this page holds now is **the built thing**: the areas the layer wires, what each reaches, and the properties a namespaced surface has to keep as the remaining phases add to it.

## The properties

| proposed rule | verdict | measured |
|---|---|---|
| a boundary module a consumer reaches is reached from one area | **holds** | 0 of 15 disagree; so no operation can be looked for under two names |
| every area holds an operation that reaches the boundary | **holds** | 0 of 12 disagree |
| no operation's name repeats its own area's | **holds** | 0 of 58 disagree; which is what the namespace is for |
| every boundary module is reached, or is the caller's memory or the context's life | **holds** | 0 of 22 disagree; unreached: `blob`, `lib`, `provider`, `string` |
| an entry point's name already carries its own module | falsified | 14 of 69 disagree; the exceptions are `ts_abi_version`, `ts_sdk_version`, `ts_catalogue_version`, `ts_default_profile`, `ts_build_info`, `ts_status_message`, `ts_error_free`, `ts_almanac_disagreements`, `ts_almanac_published_reasons`, `ts_almanac_sankalpa`, `ts_almanac_ashauch`, `ts_almanac_first_year`, `ts_almanac_festival_calendar`, `ts_context_new_with_provider` |
| every operation the layer declares is listed by every parity runner | falsified | 8 of 263 disagree; unlisted: `(root).recorded_settings by parity.mjs`, `intl.in_words by parity.mjs`, `(root).recorded_settings by parity.dart`, `intl.in_words by parity.dart`, `(root).recorded_settings by parity.py`, `intl.in_words by parity.py`, `(root).recorded_settings by parity.rs`, `intl.in_words by parity.rs` |
| every area the layer wires is named by the site's guide, and no other | falsified | 1 of 12 disagree; not on the page: `naming` |

**No operation spells its own area.** Several did on the flat surface — `convertTime`, because `convert` was taken by the calendar, is the one to remember — and each gave the word back when the namespace took it; [`surface-areas.md`](surface-areas.md) lists them. This row is the one that decays quietly as operations are added, which is why it is gated.

The unreached modules are `blob`, `lib`, `provider`, `string`. 4 of them are the plumbing the rule allows for (`blob`, `context`, `lib`, `provider`, `string`: the C caller's memory, the context's own life, and the library itself), and a rule that made an area of each would put them on the surface with nothing ever to be found under them.

### The names that do not carry their module

6 of them are `lib`'s, and `lib` is not a module in the sense the others are: it is the library itself, and `ts_sdk_version` would gain nothing by becoming `ts_lib_sdk_version`. The rule does not apply to it, and this page counts it as a miss rather than writing the exception into the rule — a rule with its exceptions inside it cannot be falsified by anything.

1 is an exception **declared in [`surface-areas.md`](surface-areas.md)** and still counted here for the same reason: `ts_context_new_with_provider`. It belongs to `context`'s family by the name a consumer calls and to `provider`'s by what it depends on, and the name that should read well is the one a consumer calls.

The remaining 7 are not accounted for: `ts_error_free`, `ts_almanac_disagreements`, `ts_almanac_published_reasons`, `ts_almanac_sankalpa`, `ts_almanac_ashauch`, `ts_almanac_first_year`, `ts_almanac_festival_calendar`.

## The areas the layer wires

**12 areas over 58 operations, and a root.** An area is a *value*: built once with the context, frozen, and destructurable, which is what makes the grouping worth having rather than merely tidy.

### The root — `Context`

| operation | reaches |
|---|---|
| `engine` | — |
| `profile` | `ts_context_profile` (context) |
| `settings` | — |
| `settingsJson` | `ts_context_settings_json` (context) |
| `settingsHash` | `ts_context_settings_hash` (context) |
| `recordedSettings` | `ts_recorded_settings` (recorded) |
| `positions` | `ts_frame_pack` (frame), `ts_positions` (positions) |
| `dispose` | `ts_context_free` (context) |

### `sdk.calendar` — `CalendarArea`

| operation | reaches |
|---|---|
| `dateOf` | `ts_calendar_from_fixed` (calendar) |
| `fixedOf` | `ts_calendar_to_fixed` (calendar) |
| `convert` | `ts_calendar_convert` (calendar) |
| `weekdayOf` | `ts_calendar_weekday` (calendar) |
| `monthLength` | `ts_calendar_month_length` (calendar) |
| `isLeap` | `ts_calendar_is_leap` (calendar) |
| `tibetanYear` | `ts_calendar_tibetan_year` (calendar) |

### `sdk.time` — `TimeArea`

| operation | reaches |
|---|---|
| `resolve` | `ts_time_resolve` (time) |
| `civilOf` | `ts_time_civil` (time) |
| `convert` | `ts_time_convert` (time) |
| `deltaT` | `ts_time_delta_t` (time) |

### `sdk.intl` — `IntlArea`

| operation | reaches |
|---|---|
| `locale` | `ts_intl_locale` (intl), `ts_intl_set_locale` (intl) |
| `render` | `ts_intl_render` (intl) |
| `has` | `ts_intl_has` (intl) |
| `transliterate` | `ts_intl_transliterate` (intl) |
| `inWords` | `ts_intl_in_words` (intl) |
| `entity` | `ts_intl_entity` (intl) |
| `messages` | — |
| `loadPack` | `ts_intl_load_pack` (intl) |

### `sdk.keys` — `KeysArea`

| operation | reaches |
|---|---|
| `id` | `ts_key_parse` (key) |
| `name` | `ts_key_name` (key) |

### `sdk.frame` — `FrameArea`

| operation | reaches |
|---|---|
| `canonical` | `ts_frame_canonical` (frame) |
| `pack` | `ts_frame_pack` (frame) |
| `unpack` | `ts_frame_unpack` (frame) |

### `sdk.chart` — `ChartArea`

| operation | reaches |
|---|---|
| `layout` | `ts_chart_layout_row` (chart) |
| `found` | — |
| `foundMany` | `ts_chart_found` (chart), `ts_rashifal` (rashifal) |
| `rashifal` | — |
| `rashifalMany` | `ts_rashifal` (rashifal) |

### `sdk.almanac` — `AlmanacArea`

| operation | reaches |
|---|---|
| `of` | `ts_panchanga_days` (panchanga) |
| `festivals` | — |
| `publishedFromSigned` | `ts_published_verify` (published) |
| `day` | `ts_pakshi` (pakshi) |
| `pakshi` | `ts_pakshi` (pakshi) |
| `sankalpa` | `ts_almanac_sankalpa` (panchanga) |
| `ashauch` | `ts_almanac_ashauch` (panchanga) |
| `firstYear` | `ts_almanac_first_year` (panchanga) |
| `disagreements` | `ts_almanac_disagreements` (panchanga) |
| `publishedReasons` | `ts_almanac_published_reasons` (panchanga) |
| `festivalCalendar` | `ts_almanac_festival_calendar` (panchanga) |

### `sdk.matching` — `MatchingArea`

| operation | reaches |
|---|---|
| `naam` | `ts_naam_milan` (naam) |

### `sdk.numerology` — `NumerologyArea`

| operation | reaches |
|---|---|
| `profile` | `ts_numerology_profile` (numerology) |

### `sdk.naming` — `NamingArea`

| operation | reaches |
|---|---|
| `syllables` | `ts_naming_syllables` (naming) |
| `read` | `ts_naming_read` (naming) |
| `check` | `ts_naming_check` (naming) |
| `of` | `ts_naming_of` (naming) |
| `load` | `ts_naming_load` (naming) |
| `suggest` | `ts_naming_suggest` (naming) |

### `sdk.research` — `ResearchArea`

| operation | reaches |
|---|---|
| `counts` | `ts_research` (research) |
| `compare` | `ts_research` (research) |
| `expected` | `ts_research` (research) |
| `timed` | `ts_research` (research) |

### `sdk.engine` — `Engine`

| operation | reaches |
|---|---|
| `manifestJson` | `ts_ephemeris_manifest` (ephemeris) |
| `manifest` | — |
| `names` | — |
| `signature` | — |
| `call` | — |
| `callJson` | `ts_ephemeris_call` (ephemeris) |

## What each boundary module holds

| module | entry points | reached from |
|---|---:|---|
| `blob` | 1 | — |
| `calendar` | 9 | `calendar` |
| `chart` | 2 | `chart` |
| `context` | 7 | `(root)` |
| `ephemeris` | 2 | `engine` |
| `frame` | 3 | `(root)`, `frame` |
| `intl` | 8 | `intl` |
| `key` | 2 | `keys` |
| `lib` | 6 | — |
| `naam` | 1 | `matching` |
| `naming` | 6 | `naming` |
| `numerology` | 1 | `numerology` |
| `pakshi` | 1 | `almanac` |
| `panchanga` | 7 | `almanac` |
| `positions` | 1 | `(root)` |
| `provider` | 3 | — |
| `published` | 1 | `almanac` |
| `rashifal` | 1 | `chart` |
| `recorded` | 1 | `(root)` |
| `research` | 1 | `research` |
| `string` | 1 | — |
| `time` | 4 | `time` |

**Entry points do not measure an area's size to a consumer.** `calendar` has the most of them and `chart`, `positions` and `panchanga` have one each — and those three are the operations the SDK exists for. One entry point serves a whole family there, because a chart request carries what would otherwise have been a dozen calls. An area sized by its entry points would rank the surface almost backwards, which is why the design page sizes none of them.

## What this does not measure

**The Dart and Python layers.** Only one of the three declares its surface and reaches the boundary by a name derived from the entry point's own, so only one can be read this way. What holds the other two to this one is `check-parity`, which compares a `surface.<area>.<operation>` line per operation from each runner: this page says what Node's shape *is*, and that gate says the other two share it.

**Whether the area names are the right ones.** This holds the properties a namespaced surface must keep. `almanac` over `panchanga`, and `engine` over `ephemeris`, are arguments and not counts, and [`surface-areas.md`](surface-areas.md) makes them.

**What the remaining phases add.** Every area here is one the SDK already has. Dashas, strengths, rules, interpretation and the application modules are what make a flat surface untenable, and an area invented before its operations exist is a slot that shapes the work to fit it.

