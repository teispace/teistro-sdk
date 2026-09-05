# Core types and the catalogue

Status: `draft`, written 2026-09-05 as the first Phase 1 design page and
revised the same day when `crates/core` was built: fifty-three kinds
in `catalogue/`, the generator and its gate, the newtypes with
compile-fail proofs, the exact angle and rationals, the envelope and
the error, registries and limits, and the settings module, all measured
(section 7). Derives from `02-architecture/05-data-model-identifiers.md`,
`06-api-conventions.md`, `08-extensibility.md`, `exact-arithmetic.md`,
ADR-0003, ADR-0016, ADR-0018, ADR-0020 and ADR-0023, and from the shapes
the spikes fixed: the `api:` description lines (spike 2), the packing of
kinds and ids at the C boundary and the reserved status codes (spike 3),
the `sdk.entity` namespace and its record shape (spike 4). The catalogue
content is inventoried from the baseline engine's entity data (rank 2,
ADR-0018) and is cited row by row before it ships.

## 1. Purpose and scope

`core` is L0: the vocabulary every other crate speaks and no crate may
redefine. This page settles the identifier model (kinds, keys, ids and
their packing), the catalogue (its sources, schema, generation, and the
three rules for what it holds and refuses to hold), the validated
quantity newtypes, the closed unions and their form in every binding,
the result envelope and the error model, the registries and limits, and
the tests that hold all of it. It leaves to their own pages what they
own: `Nas` and `Ratio` (`exact-arithmetic.md`), instants and civil time
(`time-and-timezone.md`), settings knobs and profiles
(`settings-and-profiles.md`); `core` hosts those types and this page
names where they sit.

## 2. Inputs, settings and ports

`core` reads no settings, no environment, no clock and no file. The
catalogue is compiled in; registries are filled at context creation and
immutable afterwards. Dependencies: the standard library and `serde`
(derive) for the canonical serialisation; nothing else. No lookup
allocates.

## 3. The data model

### 3.1 Kinds

A kind is a family of entities sharing one key type and one attribute
schema. Kinds are numbered once, appended only, and the number is part
of the C boundary. The v1 inventory, with the key convention each kind
follows (the baseline engine's identifiers wherever they exist, for
continuity with the fixtures and the packs; romanised Sanskrit without
diacritics elsewhere):

| # | kind | members | keys | attributes (section 3.4) |
|---:|---|---:|---|---|
| 1 | `graha` | 12 | `SUN`, `MOON`, `MARS`, `MERCURY`, `JUPITER`, `VENUS`, `SATURN`, `RAHU`, `KETU`, `URANUS`, `NEPTUNE`, `PLUTO` | body class, nature, gender, element, guna, varna, direction, exaltation, debilitation, moolatrikona, own signs, natural relationships |
| 2 | `rashi` | 12 | `ARIES` to `PISCES` | lord, element, modality, parity, rising type |
| 3 | `nakshatra` | 27 | `ASHWINI` to `REVATI` | Vimshottari lord, deity, gana, nadi, yoni with its sex, varna, element, muhurta nature, four padas with their aksharas; Abhijit is a span of the 28-scheme's table, not a member |
| 4 | `tithi` | 30 | `SHUKLA_PRATIPADA` to `PURNIMA`, `KRISHNA_PRATIPADA` to `AMAVASYA` | paksha, number, deity, class (nanda, bhadra, jaya, rikta, purna) |
| 5 | `karana` | 11 | `BAVA`, `BALAVA`, `KAULAVA`, `TAITILA`, `GARIJA`, `VANIJA`, `VISHTI`, `SHAKUNI`, `CHATUSHPADA`, `NAGA`, `KIMSTUGHNA` | movable or fixed, deity |
| 6 | `yoga` | 27 | `VISHKAMBHA` to `VAIDHRITI` | deity, auspiciousness class |
| 7 | `vara` | 7 | `RAVIVARA` to `SHANIVARA` | lord, weekday number |
| 8 | `masa` | 12 | `CHAITRA` to `PHALGUNA` | order, solar counterpart |
| 9 | `ritu` | 6 | `VASANTA` to `SHISHIRA` | the two masas |
| 10 | `ayana` | 2 | `UTTARAYANA`, `DAKSHINAYANA` | |
| 11 | `paksha` | 2 | `SHUKLA`, `KRISHNA` | |
| 12 | `samvatsara` | 60 | `PRABHAVA` to `AKSHAYA` | order in the Jovian cycle |
| 13 | `tatwa` | 5 | `PRITHVI`, `JALA`, `AGNI`, `VAYU`, `AKASHA` | |
| 14 | `varna` | 4 | `BRAHMANA`, `KSHATRIYA`, `VAISHYA`, `SHUDRA` | |
| 15 | `gana` | 3 | `DEVA`, `MANUSHYA`, `RAKSHASA` | |
| 16 | `nadi` | 3 | `ADI`, `MADHYA`, `ANTYA` | |
| 17 | `yoni` | 14 | `ASHWA` to `SIMHA` | the hostile yoni |
| 18 | `deity` | about 45 | `ASHWINI_KUMARA` and the rest | |
| 19 | `dignity` | 11 | `DEEP_EXALTED` to `DEEP_DEBILITATED` | rank |
| 20 | `relationship` | 5 | `GREAT_FRIEND`, `FRIEND`, `NEUTRAL`, `ENEMY`, `GREAT_ENEMY` | rank |
| 21, 51 to 54 | `avastha_baladi`, `avastha_jagradadi`, `avastha_deeptadi`, `avastha_lajjitadi`, `avastha_sayanadi` | 5, 3, 9, 6, 12 | `BALA` to `MRITA`; `JAGRAT` to `SUSHUPTI`; the nine Deeptadi; the six Lajjitadi; the twelve Sayanadi; one kind per family because two families share a member name (`MUDITA`) | |
| 22 | `state` | 12 | `RETROGRADE`, `STATIONARY`, `COMBUST`, `PLANETARY_WAR`, `GANDANTA`, `SANDHI`, `VARGOTTAMA`, `PUSHKARA_NAVAMSA`, `PUSHKARA_BHAGA`, `MRITYU_BHAGA`, `MARANA_KARAKA_STHANA`, `ECLIPSED` | |
| 23 | `ayanamsha` | 47 | `LAHIRI`, `RAMAN`, `KRISHNAMURTI`, `TRUE_CHITRA` and the rest of the Swiss catalogue | the Swiss sidereal-mode number, the defining epoch and value where formula-defined |
| 24 | `house_system` | 26 | `WHOLE_SIGN`, `PLACIDUS`, `KOCH`, `REGIOMONTANUS`, `CAMPANUS`, `EQUAL`, `PORPHYRY`, `SRIPATI`, `VEHLOW`, `ALCABITIUS`, `MORINUS`, `TOPOCENTRIC` and the rest | the Swiss letter, degeneracy behaviour |
| 25 | `varga` | 21 shipped, open | `D1` to `D60`, `D150`; named variants `D2_KASHINATHA`, `D9_KALACHAKRA`; custom `D_N` registered | divisions; the kernel row lives in `vargas` |
| 26 | `dasha_system` | 56 catalogued, open | `VIMSHOTTARI`, `ASHTOTTARI`, `YOGINI`, `CHARA`, `KALACHAKRA` and the rest | family; the kernel row lives in `dasha` |
| 27 | `bala_scheme` | 2 shipped | `PARASHARA`, `PARASHARA_EXTENDED` | the scheme row lives in `strength` |
| 28 | `koota` | 12 | `VARNA` to `MAHENDRA` | maximum points |
| 29 | `chara_karaka` | 8 | `ATMAKARAKA` to `PITRIKARAKA` | rank |
| 30 | `chart_kind` | 7 | `NATAL`, `TRANSIT`, `EVENT`, `PRASHNA`, `RETURN`, `RELOCATED`, `COMPOSITE` | |
| 31 | `point` | about 60, open | `LAGNA`, `MC`, `BHAVA_LAGNA`, `HORA_LAGNA`, `GHATI_LAGNA`, `GULIKA`, `MANDI`, `BHRIGU_BINDU`, `YOGI`, the sphutas, the arudhas `A1` to `A12`, the sahamas; consumer points registered | formula reference |
| 32 | `rule` | open | pack keys (`GAJA_KESARI`) | none in core |
| 33 | `muhurta_nature` | 7 | `DHRUVA`, `CHARA`, `UGRA`, `MISHRA`, `KSHIPRA`, `MRIDU`, `TIKSHNA` | |
| 34 | `modality` | 3 | `CHARA`, `STHIRA`, `DVISVABHAVA` | |
| 35 | `nature` | 3 | `BENEFIC`, `MALEFIC`, `CONDITIONAL` | |
| 36 | `guna` | 3 | `SATTVA`, `RAJAS`, `TAMAS` | |
| 37 | `direction` | 8 | `EAST` to `NORTH_EAST` | |
| 38 | `gender` | 3 | `MALE`, `FEMALE`, `NEUTER` | |
| 39 | `calendar` | 6 shipped, open | `GREGORIAN`, `JULIAN`, `MIXED`, `ISO_WEEK`, `BIKRAM_SAMBAT`, `INDIAN_LUNISOLAR` | in `calendar` |
| 40 | `era` | 7 | `VIKRAMA`, `SHAKA`, `KALI`, `NEPAL_SAMBAT`, `BUDDHA`, `KOLLAM`, `BENGALI` | in `calendar` |

The lagna is a point, not a graha: the baseline engine files it under
grahas for convenience, and the SDK does not, because a point has a
formula and no ephemeris body. Kinds 41 to 55 are the small value sets
the attributes use (body class, parity, rising type, sex, tithi class,
auspiciousness, degeneracy, the ayanamsha category, the dasha family,
the point family); everything enumerable is a kind so every value has a
key a pack can name. Kinds 25 to 27, 31, 32 and 39 hold the
identity of their members here and the definition rows in the crate
that owns the kernel (ADR-0017); `core` never holds a kernel table.

### 3.2 Keys

A key is `<kind>.<NAME>`: the kind's lowercase name, a full stop, and a
name matching `[A-Z][A-Z0-9_]{0,47}`, unique within the kind. Keys are
ASCII, romanised without diacritics, and never renamed: a member that
must change name keeps its id, the new name becomes the key, and the old
one stays an alias that resolves with `deprecated` set in the result's
provenance for at least one minor version. Numbered families keep the
baseline engine's form (`D9`; a variant is `D9_KALACHAKRA`). A custom
member registered by a consumer follows the same grammar and carries a
namespace prefix chosen at registration (`ACME_D7`), so a consumer key
never collides with a future SDK key.

### 3.3 Ids

Every member has a dense `u16` id from 0 in catalogue order, stable
forever; a deprecated member keeps its id and its slot. At the C boundary
a key travels as `KeyId(u32) = (kind << 16) | id`, with `0xFFFF` in the
low half meaning "none". Ids `0x8000` and above are runtime-registered
(custom vargas, consumer dasha systems, consumer points, pack rules),
scoped to the context that registered them, and serialised only beside
their key, never bare. Ids below `0x8000` are the catalogue's and appear
in generated enums.

### 3.4 The catalogue

Sources live in `catalogue/<kind>.yaml`, one file per kind, and are the
only hand-edited form. A generator (`cargo xtask gen catalogue`) produces
`crates/core/src/catalogue/*.rs` (checked in; a gate refuses a source
that does not match its output), `catalogue.json` for tooling and the
docs, the `sdk.entity` skeleton the intl sources are validated against
(keys, glyphs, genders), and the enum members of the API description
with their documentation. One entry:

```yaml
kind: graha
version: 1
members:
  - key: SUN
    id: 0
    glyph: "☉"
    attributes:
      body: LUMINARY
      nature: MALEFIC
      gender: MALE
      element: AGNI
      guna: SATTVA
      varna: KSHATRIYA
      direction: EAST
      exaltation: { sign: ARIES, degree: 10 }
      debilitation: { sign: LIBRA, degree: 10 }
      moolatrikona: { sign: LEO, from: 0, to: 20 }
      own: [LEO]
      friends: [MOON, MARS, JUPITER]
      neutrals: [MERCURY]
      enemies: [VENUS, SATURN]
    sources:
      - { text: BPHS, ref: "3.44-56" }
    mark: V
```

Every member carries `sources` and a confidence mark (V, T or S,
ADR-0018); an attribute a text does not settle carries its own mark and
the value the SDK follows by default with the alternatives named in the
cruxes register. Three rules decide what an attribute is:

1. **A fact the texts agree on is an attribute**: a sign's lord, a
   planet's exaltation sign and degree, a nakshatra's gana, nadi and
   yoni, a tithi's deity.
2. **A value the schools dispute is a row in the kernel table that uses
   it**, never an attribute: combustion orbs (a cited table in `state`,
   selectable), dasha years (the system's row in `dasha`), naisargika
   bala and the dig bala house (the scheme in `strength`), the nodes'
   co-lordship of Aquarius and Scorpio (a settings knob over an
   attribute marked T), special aspects (the aspect model's table).
3. **Presentation and application facts are not catalogue
   attributes**: names, abbreviations, transliterations and prose forms
   live in the intl packs (`sdk.entity`, spike 4); gemstones, colours
   and metals live in the remedies module's data; interpretation text
   lives in interpretation packs.

The baseline engine's graha record carries all three kinds mixed; the
catalogue keeps the first, moves the second to its kernel and the third
to its pack, so a school choice can never hide inside a "fact".

The attribute schemas, generated as Rust structs with `&'static` tables:

```rust
pub struct GrahaAttributes {
    pub body: BodyClass,                 // Luminary | Planet | Node | Outer
    pub nature: Nature,
    pub gender: Gender,
    pub element: Tatwa,
    pub guna: Guna,
    pub varna: Varna,
    pub direction: Direction,
    pub exaltation: Option<SignDegree>,  // sign and the degree of deepest exaltation
    pub debilitation: Option<SignDegree>,
    pub moolatrikona: Option<SignSpan>,  // sign, from, to in whole degrees
    pub own: &'static [Rashi],
    pub friends: &'static [Graha],
    pub neutrals: &'static [Graha],
    pub enemies: &'static [Graha],
}

pub struct RashiAttributes {
    pub lord: Graha,
    pub co_lord: Option<Graha>,          // Rahu for Aquarius, Ketu for Scorpio; marked T
    pub element: Tatwa,
    pub modality: Modality,
    pub parity: Parity,                  // Odd | Even
    pub rising: Rising,                  // Sirshodaya | Prishtodaya | Ubhayodaya
}

pub struct NakshatraAttributes {
    pub vimshottari_lord: Graha,         // named for what it is; other systems map in their rows
    pub deity: Deity,
    pub gana: Gana,
    pub nadi: Nadi,
    pub yoni: Yoni,
    pub yoni_sex: Sex,
    pub varna: Varna,
    pub element: Tatwa,
    pub muhurta_nature: MuhurtaNature,
    pub padas: [Pada; 4],                // akshara (romanised) per pada; the navamsa sign is derived
}

pub struct TithiAttributes { pub paksha: Paksha, pub number: u8, pub deity: Deity, pub class: TithiClass }
pub struct KaranaAttributes { pub movable: bool, pub deity: Deity }
pub struct YogaAttributes { pub deity: Deity, pub auspicious: Auspiciousness }
pub struct VaraAttributes { pub lord: Graha, pub weekday: u8 }
pub struct AyanamshaAttributes { pub swiss_mode: u8, pub definition: AyanamshaDefinition }
pub struct HouseSystemAttributes { pub swiss_letter: u8, pub polar: Degeneracy }
```

What is computable is derived by a `const fn` and never stored, so it
cannot drift from its definition: the navamsa sign of pada `p` of
nakshatra `n` is `(4n + p) mod 12`; a tithi's paksha and number are
`t / 15` and `t mod 15 + 1`; the karana of half `h` of tithi `t` is
`k = 2t + h`, with `k = 0` Kimstughna, `k` of 57 to 59 Shakuni,
Chatushpada and Naga, and the rest `movable[(k − 1) mod 7]`; the lord of
weekday `w` is the seven-planet ladder from the Sun. Which division of
the circle a longitude falls in is `Nas::division_index`, never a table.

Nakshatra spans are a scheme, not an attribute of the member: the
27-scheme is 27 equal spans; the 28-scheme inserts Abhijit from 276°40′
to 280°53′20″ and shortens Uttara Ashadha and Shravana around it. Both
are span tables in `Nas` (`exact-arithmetic.md`), selected by a settings
knob; Abhijit is the 28th span's flag on the panchanga result, never a
member, so no attribute needs an option for it.

### 3.5 Quantity newtypes

Every domain quantity is a newtype validated once at construction
(ADR-0023). No bare primitive appears in a public signature above the C
ABI; at the C ABI the field name carries the unit.

| type | inner | accepts | C field |
|---|---|---|---|
| `Latitude` | `f64` | finite, −90 to 90 | `lat_deg` |
| `Longitude` | `f64` | finite, −180 to 180, east positive | `lon_deg` |
| `Altitude` | `f64` | finite, −500 to 12 000 m | `alt_m` |
| `Degrees` | `f64` | finite; an open angle, any range | `_deg` suffix |
| `Nas` | `i64` | 0 to the circle exclusive (`exact-arithmetic.md`) | `_nas` suffix |
| `JulianDay<S>` | `f64` | finite; `S` is `Ut1`, `Tt` or `Utc` as a type parameter, so a UT1 day cannot be passed where TT is expected | `jd_ut1`, `jd_tt`, `jd_utc` |
| `SignIndex` | `u8` | 0 to 11 | `sign` |
| `HouseNumber` | `u8` | 1 to 12 | `house` |
| `NakshatraIndex` | `u8` | 0 to 26, or 27 in the 28-scheme | `nakshatra` |
| `PadaIndex` | `u8` | 0 to 3 | `pada` |
| `TithiIndex` | `u8` | 0 to 29 | `tithi` |
| `VargaDivisions` | `u16` | 1 to 300 | `divisions` |
| `Depth` | `u8` | 1 to 6 | `depth` |
| `Ratio` | `i128` pair | lowest terms, non-negative (`exact-arithmetic.md`) | numerator and denominator fields |
| `Percent`, `Rupa`, `Shashtiamsha` | `f64` | finite, non-negative | named |

Construction is `T::try_new(value) -> Result<T, InvalidValue>` where
`InvalidValue` names the type, the value and the accepted range and
carries the field name when the caller supplies one; `const fn
new_unchecked` exists for generated tables only and is `pub(crate)`.
Every newtype implements `Display`, `Serialize` as its inner value with
its unit in the field name, ordering where the quantity is ordered, and
nothing that converts implicitly: there is no `From<f64>`, and a
`trybuild` test proves `place(lon, lat)` does not compile.

### 3.6 Closed unions

Every kind's Rust enum is `#[non_exhaustive]` with explicit
discriminants equal to the ids; members are only appended. The
description carries every member's documentation from the catalogue's
sources, and the generators emit, per binding: a TypeScript string union
(`'graha.SUN' | …`) plus an `unknown` arm for a value from a newer
library; a Dart enum with a `key` field and an `unknown` member; a
Python `Literal` union with `str` fallback; a C enum whose members are
the ids with `_UNKNOWN = -1`. Spike 4 measured the TypeScript and Dart
forms and proved a wrong member is a compile error.

### 3.7 The envelope and the errors

```rust
pub struct Envelope<T> { pub value: T, pub provenance: Provenance }

pub struct Provenance {
    pub sdk_version: Version,
    pub module_versions: &'static [(ModuleId, Version)],
    pub calculation_version: u32,             // ADR-0020
    pub catalogue_version: u32,
    pub profile: ProfileId,
    pub settings_hash: Hash,
    pub input_hash: Hash,
    pub provider: ProviderStamp,               // name, version, data version, data hashes, tier, frame, flags used
    pub packs: Vec<PackStamp>,                 // id, version, content hash
    pub time: TimeStamp,                       // Delta T model, leap table, tzdb version, time basis applied, uncertainty
    pub calendar: Option<CalendarResolution>,  // tabular, computed, divergent
    pub deviation: Option<Deviation>,          // when a classical model answered
    pub applied_conventions: Vec<Convention>,  // knob, value, reason
    pub confidence: Confidence,                // Verified | Unverified
    pub fallbacks_used: Vec<Fallback>,
    pub warnings: Vec<Warning>,                // code, message key, slots
    pub content_hash: Hash,
}
```

`Hash` is a SHA-256 in canonical hex; hashes are computed over the
canonical serialisation `serial` defines, so two bindings that agree on
the JSON agree on the hash. The cache key every binding documents is
`(input_hash, settings_hash, calculation_version)`.

Errors are one closed status with a stable code, a detail code, an
English message that names the field and the range, and, behind one
pointer so a `Result` stays small on the success path, the field, a
hint (`did you mean ...`) and an optional message key with slots for
localisation; a success never carries a message:

| status | code | when |
|---|---:|---|
| `OK` | 0 | |
| `INVALID_ARG` | −1 | a value refused at construction or a request that contradicts itself |
| `OUT_OF_RANGE` | −2 | an instant or place outside the provider's or the calendar's coverage |
| `CAPABILITY` | −3 | the settings need something the provider does not declare |
| `PROVIDER` | −4 | the provider failed; its own code and message are carried |
| `NOT_CONVERGED` | −5 | a search hit its cap |
| `UNSUPPORTED` | −6 | a registered but unimplemented variant (`detail: UNSOURCED`, ADR-0018) or an unknown key |
| `PACK` | −7 | a pack failed validation or targets another catalogue version |
| `LIMIT` | −8 | a batch, range or cache limit exceeded |
| `SCHEMA_VERSION` | −9 | a struct or blob from an incompatible version |
| `INTERNAL` | −10 | a panic caught at the boundary; never expected |

The port codes spike 3 reserved (−1 to −6 inside the provider boundary)
map onto these at the port; a provider's own codes stay offset out of
this range. Degenerate outcomes are never errors: they are typed states
on the result (`undefined { reason }`), and every convention the SDK had
to choose to terminate is listed in `applied_conventions`.

### 3.8 Registries, capabilities and limits

```rust
pub struct Registry<K: Kind> { /* catalogue members + registered definitions */ }
impl<K: Kind> Registry<K> {
    pub fn get(&self, key: &K::Key) -> Option<&K::Definition>;
    pub fn by_id(&self, id: KeyId) -> Option<&K::Definition>;
    pub fn register(&mut self, definition: K::Definition) -> Result<RegisteredKey<K>, Error>;  // before the context is sealed
}
```

A context builds one registry per open kind at creation, validates every
consumer definition with the kind's whole-table invariants (ADR-0017)
and seals; a registration after sealing is `INVALID_ARG`. `Capabilities`
is the shape every provider declares and every port refines (identity,
version, data hashes, coverage, what it can compute), so provenance
stamps them uniformly. `Limits` is per context: batch sizes, instant and
place ranges, iteration caps per solver, cache bytes; every limit has a
default in the profile and is stamped when it bites (`LIMIT`).

## 4. Algorithms

`core` computes almost nothing, on purpose. Key parsing splits at the
first full stop, checks the grammar, and resolves the name by binary
search over the kind's sorted static key table, then the alias table;
formatting is the reverse. Id packing and unpacking are shifts. The
derivations in section 3.4 are `const fn`s with unit tests against the
texts and a property test that they agree with the tables they replace
(the baseline engine's pada and karana tables are fixtures). Everything
that classifies a longitude is `core::angle` and is specified in
`exact-arithmetic.md`.

## 5. The API

Rust: `core::catalogue::{Graha, Rashi, Nakshatra, …}` enums with
`key() -> &'static str`, `id() -> KeyId`, `attributes() -> &'static
…Attributes`, `from_key(&str)`, `from_id(KeyId)` and `ALL`;
`core::quantity::{Latitude, Longitude, …}`; `core::envelope::{Envelope,
Provenance, Error, Status}`; `core::registry::Registry`. C ABI:
`ts_key_parse(const char*, ts_key_id*)`, `ts_key_name(ts_key_id, char*,
size_t)`, `ts_catalogue_version()`, and every kind as a typed enum in
the generated header; the API description gains `kind=` on enum members
so the generators know which union a member belongs to. Bindings: the
generated unions of section 3.6, the branded quantity types with
validating constructors, and `catalogue.json` shipped for tooling that
needs attributes without a native call.

## 6. Errors and degenerate states

A refused quantity is `INVALID_ARG` naming the field, the value and the
range. An unknown key is `UNSUPPORTED` with the key and the nearest
known key when one is within an edit distance of two, as a hint in the
message. A deprecated key resolves and adds `DEPRECATED_KEY` to the
warnings with the replacement. A pack built against another catalogue
version is `PACK` with both versions. A runtime-registered id that
reaches serialisation without its key is `INTERNAL`, because the SDK
never emits one.

## 7. Performance budget and benchmark

| operation | budget | measured (Apple Silicon, release) |
|---|---:|---:|
| key parse and resolve | 100 ns, no allocation | 40 ns (`graha.MARS`); 20 ns inside a kind |
| an unknown key with a suggestion | 10 µs | 5.4 µs |
| id to attributes | 5 ns (a static table index) | 0.7 ns |
| `Nas::from_degrees`; sign, nakshatra and pada | 20 ns | 3.7 ns; 1.7 ns |
| settings resolution from a profile | 5 µs | 1.9 µs |
| canonical JSON and hash of the shipped document | 10 µs per KB | 15 µs for the 2 KB document with its two forty-entry dasha maps |
| envelope construction | one allocation per vector present | |
| the catalogue's static tables | under 64 KB in `.rodata` across every kind | |

The benchmark is `core`'s criterion set with an instruction-count row
per operation in the pull-request gate (`05-testing/01-quality-bar.md`).

## 8. Tests

- Generated-table invariants in one test: ids dense and unique per kind,
  every key matching the grammar, every reference (a lord, a sign, a
  deity) resolving, exaltation and debilitation opposite by six signs
  for the seven classical grahas, own signs reciprocal with the sign's
  lord, the four padas of every nakshatra present, the twelve signs'
  lords covering the seven grahas twice each except the luminaries once.
- Golden vectors: the baseline engine's entity data exported as rank-2
  fixtures for every attribute the catalogue keeps, with the
  deliberate-difference registry recording what moved to a kernel or a
  pack; a text citation upgrades a row to rank 1.
- `trybuild` compile-fail tests for every newtype swap the API
  conventions forbid; unit tests of every constructor's range with the
  boundary values on both sides; the shared valid-and-invalid input
  corpus every binding's validators run.
- Property tests for the derivations of section 3.4 against the tables
  they replace; snapshot tests of `catalogue.json` and the generated
  description members; cross-binding parity of the generated unions
  (every member, every id, every doc string).

## 9. Localisation

`core` emits keys only. The catalogue generates the `sdk.entity`
skeleton (keys, glyphs, genders) that spike 4's validator holds every
shipped locale to; `sdk.state`, `sdk.dignity` and `sdk.avastha` follow
the same rule for their kinds. Nothing in `core` renders text.

## 10. Open questions

- Whether padas become a kind of their own (108 keys) if a module needs
  to address one by key; the default is derived (`NakshatraIndex` plus
  `PadaIndex`) until a use appears.
- The nodes' co-lordship of Aquarius and Scorpio: an attribute marked T
  until a text is cited; the cruxes register carries it.
- The `state` kind's closure in v1: the twelve members above are the
  ones a P0 module emits; a later module adds members by appending.
- Deity keys: the baseline engine stores deities as strings; the
  catalogue lifts them to a kind so packs localise them once; the exact
  member list is fixed when the nakshatra, tithi, karana and yoga files
  are authored and cited.
