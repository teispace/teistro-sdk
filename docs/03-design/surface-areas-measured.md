# The surface areas, measured

Status: `generated` by `cargo xtask areas`, gated by `check-areas`. Do not edit. Read from `idl/api.json`, the boundary's own description, and from `bindings/node/lib/index.js`, the one ergonomic layer whose members and the entry points they reach can both be read from one file.

ADR-0030 decides that the surface becomes `sdk.<area>.<operation>` and says the areas are "derived from the boundary modules the reference site already groups by". That is a proposal about a grouping, and this is the measurement of it — taken before four binding layers, four reference surfaces and every example are restructured around it, because the change is cheap now and breaking after v1.

## The rules

| proposed rule | verdict | measured |
|---|---|---|
| every boundary module is an area a consumer sees | falsified | 5 of 14 disagree; never reached: `blob`, `context`, `lib`, `provider`, `string` |
| every member of the context reaches the boundary | falsified | 10 of 35 disagree; each of them is a cached value, a decoded result or a delegate |
| a member that reaches the boundary reaches one module | falsified | 1 of 25 disagree |
| an entry point's name already carries its own module | falsified | 7 of 46 disagree; the exceptions are `ts_abi_version`, `ts_sdk_version`, `ts_catalogue_version`, `ts_default_profile`, `ts_build_info`, `ts_status_message`, `ts_context_new_with_provider` |

**The grouping is real and the rule that derives it is not.** 5 of the 14 boundary modules are never reached by anything a consumer calls, and they are not an oversight: each exists for the C caller's memory or for the context's own life, which is not an operation anybody namespaces. A rule that made an area of every module would put 5 of them on the surface with nothing ever to be found under them.

**1 of the 25 members that reach the boundary at all reaches more than one entry point**, and that is where a grouping derived from the boundary does fail: `positions` reaches `ts_frame_pack`, `ts_positions`. The failure is one of kind rather than of grouping — `ts_frame_pack` marshals the frame the request is expressed in and is not an operation a consumer would look for under an area — but the rule as ADR-0030 words it does not know that, so the areas cannot be *derived* and have to be **chosen with the derivation as evidence**.

### The names that do not carry their module

6 of them are `lib`'s, and `lib` is not a module in the sense the others are: it is the library itself, and `ts_sdk_version` would gain nothing by becoming `ts_lib_sdk_version`. The rule does not apply to it, and this page counts it as a miss rather than writing the exception into the rule — a rule with its exceptions inside it cannot be falsified by anything.

1 is an exception **declared in [`surface-areas.md`](surface-areas.md)** and still counted here for the same reason: `ts_context_new_with_provider`. It belongs to `context`'s family by the name a consumer calls and to `provider`'s by what it depends on, and the name that should read well is the one a consumer calls.

Nothing else. The three that were inconsistencies were all the same one — a function named in the singular in a file named in the plural — and were fixed by renaming the **file**, which is not an ABI symbol, rather than the function, which is.

## What each boundary module holds

| module | entry points | members that reach it | an area |
|---|---:|---:|---|
| `blob` | 1 | 0 |  |
| `calendar` | 8 | 6 | **yes** |
| `chart` | 1 | 1 | **yes** |
| `context` | 6 | 0 |  |
| `ephemeris` | 2 | 2 | **yes** |
| `frame` | 3 | 2 | **yes** |
| `intl` | 7 | 6 | **yes** |
| `key` | 2 | 2 | **yes** |
| `lib` | 6 | 0 |  |
| `panchanga` | 1 | 1 | **yes** |
| `positions` | 1 | 1 | **yes** |
| `provider` | 3 | 0 |  |
| `string` | 1 | 0 |  |
| `time` | 4 | 4 | **yes** |

**Entry points are not a measure of an area's size to a consumer, and that is the second finding.** `calendar` has the most of them and `chart`, `positions` and `panchanga` have one each — and those three are the operations the SDK exists for. One entry point serves a whole family there, because a chart request carries what would otherwise have been a dozen calls. An area sized by its entry points would rank the surface almost backwards.

## Where each member sits today

| member | reaches | module |
|---|---|---|
| `constructor` | — |  |
| `ephemeris` | — |  |
| `ephemerisManifestJson` | `ts_ephemeris_manifest` | `ephemeris` |
| `ephemerisCallJson` | `ts_ephemeris_call` | `ephemeris` |
| `profile` | — |  |
| `settings` | — |  |
| `settingsJson` | — |  |
| `settingsHash` | — |  |
| `locale` | `ts_intl_locale` | `intl` |
| `locale` | `ts_intl_set_locale` | `intl` |
| `canonicalFrame` | `ts_frame_canonical` | `frame` |
| `positions` | `ts_frame_pack`, `ts_positions` | `frame`, `positions` |
| `found` | — |  |
| `foundMany` | `ts_chart_found` | `chart` |
| `almanac` | `ts_panchanga_days` | `panchanga` |
| `almanacDay` | — |  |
| `render` | `ts_intl_render` | `intl` |
| `has` | `ts_intl_has` | `intl` |
| `transliterate` | `ts_intl_transliterate` | `intl` |
| `entity` | `ts_intl_entity` | `intl` |
| `messages` | — |  |
| `loadPack` | `ts_intl_load_pack` | `intl` |
| `dateOf` | `ts_calendar_from_fixed` | `calendar` |
| `fixedOf` | `ts_calendar_to_fixed` | `calendar` |
| `convert` | `ts_calendar_convert` | `calendar` |
| `weekdayOf` | `ts_calendar_weekday` | `calendar` |
| `monthLength` | `ts_calendar_month_length` | `calendar` |
| `isLeap` | `ts_calendar_is_leap` | `calendar` |
| `resolve` | `ts_time_resolve` | `time` |
| `civilOf` | `ts_time_civil` | `time` |
| `convertTime` | `ts_time_convert` | `time` |
| `deltaT` | `ts_time_delta_t` | `time` |
| `keyId` | `ts_key_parse` | `key` |
| `keyName` | `ts_key_name` | `key` |
| `dispose` | — |  |

**6 of them already spell their own area inside their own name** — `ephemerisManifestJson`, `ephemerisCallJson`, `canonicalFrame`, `convertTime`, `keyId`, `keyName` — which is what a flat surface costs when two areas want the same verb: `convert` was taken by the calendar, so the time's became `convertTime`. Under `sdk.<area>.<operation>` each of those loses the half that is now the namespace, and none of them needs a new word invented for it.

## What this does not measure

**The Dart and Python layers.** Only one of the three declares its surface and reaches the boundary by a name derived from the entry point's own, so only one can be read this way. `check-parity` already holds all three to the same values; what it does not hold them to is the same *shape*, which is the gap this namespacing closes and a thing a later pass should gate.

**Whether these are the right area names.** This says which groupings the boundary supports and which it does not. `ephemeris` is the clearest case of a name the measurement cannot settle: ADR-0030 renames it `engine` on an argument about what a consumer needs to be warned of, and no count decides that.

**What the remaining phases add.** Every area here is one the SDK already has. Dashas, strengths, rules, interpretation and the application modules are what make a flat surface untenable, and they are not in the description yet — so this page measures the case for namespacing at its weakest, which is the honest time to make it.

