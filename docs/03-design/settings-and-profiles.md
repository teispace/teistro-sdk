# Settings and profiles

Status: `draft`, written 2026-09-05 as a Phase 1 design page; revised
when `core::settings` is built. Derives from
`02-architecture/05-data-model-identifiers.md` (the knob table),
`06-api-conventions.md` (requests declare what to compute; coherence is
validated at the boundary), ADR-0013 (the override policy default),
ADR-0018 (named variants, never a silent school), ADR-0020 (the settings
hash in every result) and `core-types-and-catalogue.md` (every knob
value is a catalogue key or a validated newtype). The knob inventory is
a superset of the settings the baseline engine persists per chart
(`01-research/baseline-engine/00-inventory.md`) and of the thirteen
profiles spike 1's fixtures were exported under.

## 1. Purpose and scope

A computation never has a hidden default. Every module reads one
complete `Settings` value, built by applying an explicit patch to a
named, versioned profile; the value has a canonical serialisation and a
hash, the hash goes into every result, and a fixture that asserts the
hash cannot pass under a substituted default. This page settles the
knob inventory and its types, the profile model and its resolution, the
coherence rules, the canonical form and the hash, the shipped profiles,
and how a request overrides a context.

## 2. Inputs, settings and ports

Inputs are a profile id, an optional patch, and the provider's
capabilities (for the coherence rules that need them). No port; no
environment. The settings model is the one thing in the SDK that every
other module's design page adds a row to.

## 3. The data model

### 3.1 The knobs

`Settings` is a struct of groups; every field is a closed union, a
catalogue key type or a validated newtype; every field has a value in
every shipped profile. The v1 inventory:

| group | knob | type | values |
|---|---|---|---|
| frame | `zodiac` | enum | `TROPICAL`, `SIDEREAL` |
| frame | `ayanamsha` | `AyanamshaKey` or `Custom { epoch_jd_tt, value_deg, rate_deg_per_year }` | the 47 catalogued, or custom |
| frame | `ayanamsha_basis` | enum | `MEAN`, `TRUE` (with nutation) |
| frame | `node` | enum | `MEAN`, `TRUE` |
| frame | `centre` | enum | `GEOCENTRIC`, `TOPOCENTRIC` |
| frame | `positions` | enum | `APPARENT`, `TRUE` (no light time, aberration or deflection) |
| frame | `siddhanta` | enum | `DRIK`, `SURYA { bija: bool }` |
| frame | `nakshatra_scheme` | enum | `TWENTY_SEVEN`, `TWENTY_EIGHT` |
| houses | `placement_system` | `HouseSystemKey` | any registered system |
| houses | `chalit_system` | `HouseSystemKey` | `SRIPATI`, `VEHLOW`, `PORPHYRY`, `KP` |
| houses | `module_overrides` | map module to system | `kp: PLACIDUS` |
| houses | `polar_policy` | enum | `ERROR`, `FALLBACK_WHOLE_SIGN`, `FALLBACK_PORPHYRY`, `CLAMP` |
| day | `sunrise` | enum | `CENTRE_NO_REFRACTION`, `UPPER_LIMB_REFRACTION`, `LOWER_LIMB_REFRACTION`, `CUSTOM { altitude_deg }` |
| day | `day_boundary` | enum | `MIDNIGHT`, `SUNRISE`, `SUNSET`, `NOON` |
| day | `polar_day_policy` | enum | `UNDEFINED`, `NEAREST_EVENT`, `CIVIL_MIDNIGHT` |
| day | `ghati_reckoning` | enum | `CIVIL`, `PROPORTIONAL` |
| day | `hora_reckoning` | enum | `PROPORTIONAL`, `EQUAL` |
| time | `dst_gap` | enum | `ERROR`, `SHIFT_FORWARD` |
| time | `dst_overlap` | enum | `EARLIER`, `LATER`, `ERROR` |
| time | `unknown_time` | enum | `REFUSE`, `NOON`, `SUNRISE`, `MIDNIGHT` |
| time | `delta_t` | enum | `TABLE_THEN_MODEL` (the default; `time-and-timezone.md`), a named model, `PROVIDER` |
| dasha | `balance` | enum | `SPATIAL`, `TEMPORAL` |
| dasha | `year_length` | map system to `YearLengthKey` | `JULIAN_365_25`, `SAVANA_360`, `SIDEREAL`, `TROPICAL`, `LUNAR`, `NAKSHATRA_324` (the defaults per system are crux C6) |
| dasha | `depth` | map system to `Depth` | 1 to 6 |
| dasha | `seed_overflow` | enum | `WRAP_TO_START`, `REJECT` |
| jaimini | `chara_karakas` | enum | `SEVEN`, `EIGHT` |
| jaimini | `node_co_lordship` | enum | `NONE`, `STRONGER_LORD`, `BOTH` |
| aspect | `node_aspects` | enum | `NONE`, `FIVE_SEVEN_NINE`, `THREE_SEVEN_ELEVEN` |
| aspect | `drishti_table` | key | the aspect model's tables |
| state | `combustion_orbs` | key | the cited orb tables (`BPHS`, `SURYA_SIDDHANTA`) |
| strength | `bala_scheme` | `BalaSchemeKey` | `PARASHARA`, `PARASHARA_EXTENDED` |
| strength | `ekadhipatya` | enum | `CLASSICAL`, `ZERO`, `TRANSFER` |
| varga | `unattested_dn` | enum | `CYCLIC`, or a named scheme |
| calendar | `civil_calendar` | `CalendarKey` | `GREGORIAN`, `BIKRAM_SAMBAT`, … |
| calendar | `lunar_month` | enum | `AMANTA`, `PURNIMANTA` |
| calendar | `eras` | set of `EraKey` | which era numbers a date carries |
| provider | `overrides` | enum | `PREFER_NATIVE`, `SDK_ONLY`, `NATIVE_ONLY` (ADR-0013) |
| provider | `tier` | enum | `COMPACT`, `STANDARD`, `FULL`, `REFERENCE` for the built-in ephemeris |
| output | `precision` | per field family | the rounding contract (`exact-arithmetic.md`) |

A module that adds a knob adds a row here, a default to every shipped
profile and a note to the changelog (`09-guidelines/03-adding-a-module.md`).
Limits (batch sizes, ranges, iteration caps, cache bytes) are context
configuration, not settings: they do not move a number, so they are
stamped in provenance when they bite and are outside the hash.

### 3.2 Profiles

```rust
pub struct Profile {
    pub id: ProfileId,               // `nepali-default`, `kp-default`, ...
    pub version: u16,                // bumps when a default changes (a major release)
    pub base: Option<ProfileId>,     // one level of inheritance
    pub patch: SettingsPatch,        // what this profile sets over its base
    pub sources: Vec<Citation>,      // why each default
    pub mark: Mark,                  // V, T or S per ADR-0018
}
pub struct SettingsPatch { /* every knob as Option<...> */ }
```

The root profile is `sdk-root`: every knob set, cited, and never
selected directly. A shipped profile is a patch over it; a consumer
profile is a patch over a shipped one. Resolution applies root, then the
base, then the profile, then the request's patch, and validates the
result once. Only the resolved `Settings` is hashed, never the profile
id: two consumers with the same resolved value get the same hash
whatever their profiles are called, and a profile version bump that
changes a default changes the hash of every result under it, which is
what makes the change visible.

Shipped in v1, each with its sources in the profile file:

| profile | for | the defaults that define it |
|---|---|---|
| `nepali-default` | the product's charts | sidereal, `LAHIRI`, mean node, topocentric, `WHOLE_SIGN` placements, the chalit system the baseline engine measures as (`VEHLOW`; documented as Sripati, measured otherwise, so the profile says what it does), `CENTRE_NO_REFRACTION` sunrise, `SUNRISE` day boundary, `SPATIAL` balance, seven chara karakas, node aspects `NONE`, `AMANTA` months, eras Vikrama, Shaka, Kali, Nepal Sambat |
| `parashari-classical` | the texts as read | as above with `SRIPATI` chalit, `PROPORTIONAL` ghatis, eight chara karakas, `SURYA_SIDDHANTA` orbs where the text gives them |
| `kp-default` | Krishnamurti Paddhati | sidereal, `KRISHNAMURTI`, true node, `PLACIDUS` placements with the cusp as the house start, `kp: PLACIDUS` module override, node aspects `NONE`, Vimshottari at 365.25 |
| `western-tropical-default` | Western and Hellenistic modules | tropical, `PLACIDUS`, true node, `MIDNIGHT` day boundary, geocentric apparent positions |
| `conformance-baseline` | the golden-vector runs | the baseline engine's defaults exactly, including every convention the deliberate-difference registry records, so the fixtures' settings hashes reproduce |

Spike 1's thirteen fixture profiles are patches over
`conformance-baseline`, checked in beside the fixtures, and the harness
asserts that each resolves to the settings hash the fixture carries.

### 3.3 Coherence rules

Validation runs once on the resolved value and returns every finding,
never the first:

| rule | outcome |
|---|---|
| `SIDEREAL` without an ayanamsha, or a custom ayanamsha without an epoch | error `INVALID_ARG`, fields named |
| `TROPICAL` with an ayanamsha set | warning: the ayanamsha is ignored |
| `TOPOCENTRIC` and a request without a place | error at the request |
| `TWENTY_EIGHT` nakshatras with a dasha seeded on 27 | error: the seed map has no Abhijit row |
| a dasha depth beyond the system's supported depth | error |
| `CLAMP` polar policy with `WHOLE_SIGN` placements | warning: the policy never applies |
| a house system the provider declares it cannot compute natively under `NATIVE_ONLY` | error `CAPABILITY` |
| `SURYA` siddhanta with `TOPOCENTRIC` | warning: the model is geocentric; the correction is applied on top and stamped |
| a year length that is not the system's classical one | warning, recorded in `applied_conventions` |
| `REFUSE` unknown time with a request lacking a time | error, with the fallbacks that would have applied |

Warnings travel in the result's provenance; errors stop the request
with every field involved.

### 3.4 The canonical form and the hash

The canonical serialisation is JSON with keys in Unicode code-point
order, no whitespace, integers as integers, floats in the shortest
round-trip form, enums as their catalogue keys, maps as sorted objects,
and every knob present. The hash is SHA-256 of those bytes, rendered as
64 hex digits. The form is versioned by `settings_schema: 1` inside the
document, so a later knob appends and an old document still hashes the
same under the old schema; a hash is never recomputed under a newer
schema without saying so.

## 4. Algorithms

Resolution is a fold of patches with `Option::or`; validation is a list
of rules over the resolved value and the capabilities; hashing is the
canonical serialiser plus SHA-256. Nothing iterates; nothing depends on
the request beyond its patch.

## 5. The API

Rust: `Profile::shipped(id)`, `Profile::resolve(&self, patch:
&SettingsPatch, capabilities: &Capabilities) -> Result<Settings,
Diagnostics>`, `Settings::hash()`, `Settings::canonical_json()`,
`Settings::patch(&self, patch) -> Settings` for per-request overrides,
and a typed builder for `SettingsPatch` in every binding (`settings()
.ayanamsha(Ayanamsha::Raman).node(Node::True)`). C ABI:
`ts_settings_resolve(ctx, profile_id, const ts_settings_patch*,
ts_settings*)` with the patch struct carrying a presence bitmask beside
its fields and `struct_size`, `ts_settings_hash(const ts_settings*,
char[65])`, `ts_settings_json(...)`. Bindings expose the resolved value
as a readonly typed object and the patch builder; a request takes an
optional patch, never a profile id, so the context's profile is the one
source of defaults.

## 6. Errors and degenerate states

Unknown profile: `UNSUPPORTED` with the shipped ids. A knob value that
is not a catalogue member: `UNSUPPORTED` with the key. A registered but
unsourced variant: `UNSUPPORTED (UNSOURCED)`. Coherence errors:
`INVALID_ARG` with the rule id and the fields. A settings document from
a newer schema: `SCHEMA_VERSION`.

## 7. Performance budget

| operation | budget |
|---|---:|
| resolve a profile with a patch | 5 µs, allocation only for maps |
| canonical JSON and hash | 10 µs for a 1 KB document |
| `Settings` clone | under 1 KB copied |

## 8. Tests

- Completeness: a test enumerates every knob by reflection over the
  patch struct and asserts every shipped profile resolves without a
  `None`.
- Hash stability: the canonical JSON and the hash of every shipped
  profile are snapshots; a change requires a profile version bump and a
  calculation version entry.
- The coherence matrix: every rule fired and not fired by a pair of
  settings documents, with the fields asserted.
- Fixture parity: each of spike 1's thirteen profiles resolves to the
  hash in its manifest; the conformance harness asserts the hash on
  every fixture (ADR-0020).
- Round trip: JSON to `Settings` to JSON is a fixed point; the patch
  builder in every binding produces the same canonical document (the
  shared corpus).
- `trybuild`: a knob cannot take a bare integer or string.

## 9. Localisation

`sdk.settings` holds every knob's name and every value's label and
description so a consumer can render a settings screen from the
catalogue; the SDK renders none itself.

## 10. Open questions

- `nepali-default`'s node type and chalit system are taken from the
  baseline engine's measured behaviour; the maintainer confirms both
  before the profile ships (the chalit finding is in the
  deliberate-difference registry).
- The year-length default per dasha system (crux C6) and the combustion
  orb table default are resolved by reading the texts before Phase 5;
  until then both are marked T in the root profile and stamped in
  `applied_conventions`.
- Whether `precision` belongs in settings (it moves serialised digits,
  not values) or in the serialiser's options; the default is settings,
  so the hash captures it.
