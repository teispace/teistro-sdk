# Reckonings: one extension point for every classical sky

Status: `plan`, written 2026-11-17 for the Ketaki lane. The maintainer's instruction:
"we have to make scalable, later we may need to add other too". Nothing on this page is
built. Cruxes C2192 and C2194 to C2197 hold its decisions. The first new member is Ketaki
(`03-design/ketaki.md`). The later ones are the consumer's addendum 8 §6 list.

## 1. Purpose and scope

A **reckoning** is a way of computing the sky and the day that an almanac tradition
follows. Today the SDK has three:

- drik: a modern engine (Builtin or Teimeris) and the root's conventions;
- the Surya Siddhanta: `crates/siddhanta`'s `SiddhantaProvider`, with its bijas
  (`NEPAL_COMMITTEE`, Makaranda);
- the Tamil vakya: `VakyaSky`, the Vakyakarana's Sun and Moon.

The last two ride on the same chain entry, `SURYA_SIDDHANTA`, and are chosen by
`frame.siddhanta`. That entry is the precedent this page generalises. The goal is that
adding Grahalaghava or Parahita means three things: data, a provider module and a
profile. It does not mean edits across the SDK, the FFI, the bindings or the MCP.

A reckoning has three layers, and each has one home:

| layer | what it is | home | example |
|---|---|---|---|
| 1. sky | positions, a sunrise if the text gives one, the text's own ayanamsha; behind the ephemeris port, declaring its capabilities | a provider in `crates/siddhanta` (one module per text family) | `SiddhantaProvider`, `VakyaSky`, the Ketaki karana |
| 2. conventions | the zodiac, the month scheme, the era, the day boundary, sunrise, the node | settings knobs; already generic | `frame.ayanamsha`, `calendar.lunar_month`, `day.sunrise` |
| 3. profile | layer 2's values with citations, plus which layer-1 sky (if any) it needs | `profiles.rs` and a builder entry | `nepali-committee`, `tamil-vakya`, `ketaki` |

Some reckonings need only layer 3. Ketaki's modern form, the Rashtriya Panchang and the
Tilak panchang are profiles over a modern sky. Only a reckoning with its own planetary
text needs layer 1.

## 2. Audit: how drik and the Surya Siddhanta are wired today

Read at `99b9b1eff`.

### 2.1 Already generic

| part | where | why it is generic |
|---|---|---|
| the port's capabilities | `crates/port-ephemeris/src/capabilities.rs`: `astronomy` (`Modern` or `Classical`, l. 501), `ayanamshas` (the members a provider defines), `distance_unit` (`MeanDistances`, l. 379), `speeds`, `speed_model`, `jd_range` | a classical provider says what it is. It does not say which text it is |
| completion | `astro/completion.rs`: `defines_ayanamsha`, `defines(...)` over `Astronomy::Classical` | reads the capability, not the provider |
| the classical branches downstream | `chart/foundation.rs:1106`, `astro/completion.rs` (905, 940, 1036, 1153), `sdk/area/chart.rs` (1980, 3354), `sky_view_request.rs:270`, `mundane.rs:633`, `muhurta/sources.rs:399`, `context.rs:492` | each tests `Astronomy::Classical`. None names Surya |
| the calendar | `calendar/src/solar/mod.rs:129` `SolarModel`, `lunisolar/mod.rs:67` `LunarModel`, with a drik model over any provider (`solar/drik.rs`) | the SDK's calendar reads a provider. The direct `impl SolarModel for SuryaSiddhanta` (`solar/siddhanta.rs`) is a library convenience the SDK does not route through |
| composition | `crates/siddhanta/src/modern.rs` `ModernParts` | wraps any text sky with a modern sunrise and modern star planets from the chain's next entry |
| profiles | `profiles.rs`: a patch over a base, `Citation`s, `Mark`, a version | `mithila-makaranda` and `tamil-vakya` are profiles on the one classical entry already |
| the settings tag | `Siddhanta` is `#[serde(tag = "kind")]` | a new variant is additive on the wire |

### 2.2 Surya-specific (what blocks a third text)

| # | what | where | the effect |
|---|---|---|---|
| S1 | the chain entry is named for one text | `sdk/src/ephemeris.rs:39` `Ephemeris::SuryaSiddhanta`; its name l. 118; `ffi/src/context.rs:63` `TsEphemeris::SuryaSiddhanta = 3`, generated into every binding; `mcp/src/lib.rs:208` `Engine::NAMES`; `core/settings/recorded.rs:274` `RequiredEphemeris::SuryaSiddhanta` | a Ketaki karana opened under `SURYA_SIDDHANTA` reads wrongly. A new entry per text means an ABI member, a binding regeneration and an MCP edit for each |
| S2 | the provider is built by a hand match | `ephemeris.rs:226–290` `open_before`: `if siddhanta == Siddhanta::Vakya { ModernParts … VakyaSky }`, then `let Siddhanta::Surya { bija, sunrise, planets, trig }`, then `SiddhantaProvider::with_bija(…).with_trig(…)` and the `ModernParts` steps | every text adds an arm in the SDK |
| S3 | only that entry receives the rest of the chain | `ephemeris.rs:376` `if matches!(entry, Ephemeris::SuryaSiddhanta)` | a second classical entry would lose its modern parts |
| S4 | messages name the two texts | `context.rs:1304` `astronomy_coherence` matches `Surya{..} | Vakya` and hints "open the context over 'SURYA_SIDDHANTA'"; `context.rs:1343` `classical_name` returns one of two strings | a third text gets the wrong name in a refusal |
| S5 | the setting has a variant per text, and its option types are named for one | `core/settings/mod.rs:603` `Siddhanta { Drik, Surya { bija, sunrise: SuryaSunrise, planets: SuryaPlanets, trig: SuryaTrig }, Vakya }`; the `siddhanta-topocentric` warning (l. 2288) matches both; `siddhanta-text-sunrise-convention` (l. 2307) is Surya with `sunrise: Text` | each text adds a variant, warnings and their tests |
| S6 | the conformance kit assumes the Surya ayanamsha for every classical provider | `ephemeris-kit/src/kit.rs:990–1015` `check_ayanamsha`: a classical provider must give SURYASIDDHANTA = 20°24′39″ at Burgess's instant; `kit/conventions.rs:208` skips the value check for a classical provider | the kit would fail a correct Ketaki karana (C2192) |
| S7 | one builder per tradition | `context.rs:1113` `nepal_official(modern)` = the profile plus `once(SuryaSiddhanta).chain(modern)` | each tradition adds a builder method, and each builder reaches every binding by hand |

Everything in 2.2 is the SDK choosing a text. The fix is to let a registry choose it.

## 3. The extension point

### 3.1 Layer 1: a `Reckoning` behind a registry

A trait in `crates/siddhanta/src/reckoning.rs`. The crate already depends on the port and
on core's settings types.

```rust
/// A classical text that gives a sky.
pub trait Reckoning: Sync {
    /// Its key in the `ReckoningKey` catalogue: "SURYA", "VAKYA", "KETAKI_GRAHAGANITA".
    fn key(&self) -> ReckoningKey;
    /// Its name in a message: "the Surya Siddhanta's".
    fn possessive(&self) -> &'static str;
    /// The provider, under the options frame.siddhanta carries, with the chain's rest
    /// for any modern parts the options ask for.
    fn open(&self, options: &TextOptions, modern: &mut dyn FnMut(&'static str)
        -> Result<Box<dyn EphemerisProvider>, Error>) -> Result<Box<dyn EphemerisProvider>, Error>;
    /// The member the text defines and the value the kit checks it against (R1).
    fn text_ayanamsha(&self) -> Option<TextAyanamsha>;
    /// Warnings this text adds to a settings document (R4).
    fn diagnose(&self, options: &TextOptions, settings: &Settings, out: &mut Vec<Diagnostic>);
    /// Its sources, for the generated docs and the profile's citations.
    fn citations(&self) -> &'static [Citation];
}

pub static RECKONINGS: &[&dyn Reckoning] = &[&SURYA, &VAKYA /*, &KETAKI_GRAHAGANITA, … */];
```

`TextOptions` holds what every text shares: `sunrise: TextSunrise` (`TEXT`, `BHUJANTARA`,
`MODERN`), `planets: TextPlanets` (`TEXT`, `MODERN { ayanamsha }`) and `trig: TextTrig`.
These are today's `SuryaSunrise`, `SuryaPlanets` and `SuryaTrig` under neutral names (R7).
A reckoning refuses an option it cannot honour, with the existing refusal record. A
choice particular to one text, such as Surya's bija, is not an open option. It is a
catalogue key: `SURYA` stays a variant with its bija for compatibility, and a new text
that has editions gets one key per edition (`a-kind-is-a-key-space-not-a-vocabulary`).

### 3.2 Layer 1 in the setting

```text
Siddhanta = Drik
          | Surya { bija, sunrise, planets, trig }     // unchanged on the wire
          | Vakya                                     // unchanged on the wire
          | Text  { reckoning: ReckoningKey, sunrise, planets, trig }   // new, additive
```

`Surya` and `Vakya` keep their spellings and their settings hashes. Every recorded
document and the conformance corpus still read. `Siddhanta::reckoning()` maps all three
classical variants to a key: `Surya` → `SURYA`, `Vakya` → `VAKYA`, `Text` → its key.
Every site that matches `Surya{..} | Vakya` today calls `is_classical()` or `reckoning()`
instead (R3, R4). Whether `Surya` and `Vakya` should one day be spelled as `Text` is
C2195. This page says no: a spelling change costs every stored document a migration and
buys nothing.

### 3.3 The chain entry

`Ephemeris::SuryaSiddhanta` becomes **the classical entry**: "the sky `frame.siddhanta`
names". The Rust variant, its ABI number 3, `TsEphemeris`, `RequiredEphemeris` and the
`SURYA_SIDDHANTA` wire name do not change, so no binding regenerates. A second accepted
spelling, `CLASSICAL`, is added where names are parsed (the MCP's `Engine::from_name`,
the settings reader). The output keeps `SURYA_SIDDHANTA`. This keeps stored output
byte-stable. A rename to `Classical`, with `SURYA_SIDDHANTA` as the alias, is a
1.0-or-later decision (C2196).

### 3.4 Layer 3: one builder for every tradition

```rust
impl ContextBuilder {
    /// A shipped reckoning profile over its sky: the profile, and the classical entry
    /// before `modern` when the profile names a text.
    pub fn reckoning(self, profile: &str, modern: impl IntoIterator<Item = Ephemeris>)
        -> Result<ContextBuilder, Error>;
}
```

The builder reads the profile. If the profile's `frame.siddhanta` is classical, it puts
the classical entry before `modern`. Otherwise it opens `modern` alone. `nepal_official`
becomes `reckoning(NEPAL_OFFICIAL_PROFILE, modern)`, and its output is unchanged. The FFI
and the bindings expose `reckoning` **once**, taking a profile id string, so a new
reckoning needs no binding code. The MCP's context tool takes `profile` already, and gains
the classical entry's insertion in the same way.

### 3.5 What reaches the bindings and the MCP, and how

| item | crosses as | generated? | per reckoning |
|---|---|---|---|
| a profile | its id string (`SHIPPED_PROFILES`) | the list is generated from `profiles.rs` | one entry in `profiles.rs` |
| `frame.siddhanta` | the settings JSON (`{"kind": "TEXT", "reckoning": "KETAKI_GRAHAGANITA"}`) | the schema is generated by schemars | none |
| `ReckoningKey` | a catalogue member, listed by the existing catalogue API and the MCP's catalogue tool | the IDL catalogue table, as the ayanamsha members are | one row in the catalogue source |
| the chain entry | `TsEphemeris` 3, unchanged | already | none |
| the builder | `reckoning(profile, modern)`, once | once, in F4 | none |

### 3.6 What a new reckoning supplies

1. A research page in `docs/01-research/` and its cruxes.
2. If it has its own sky: a module in `crates/siddhanta/src/<family>/` with its typed
   tables (each entry citing its page) and an `impl Reckoning`; one line in `RECKONINGS`;
   one `ReckoningKey` member. The kit runs over it unchanged (R1).
3. A profile in `profiles.rs` with its citations, and its id in `SHIPPED_PROFILES`.
4. Tests: the text's worked examples step by step, a printed almanac's days where one
   exists, and a measured page against drik.

No edit in `crates/sdk`, `crates/ffi`, the bindings or `crates/mcp`.

## 4. The refactors, ranked

Each is behaviour-preserving. Each is proved by the same three gates: the existing test
suites of the crates it touches, the settings-hash goldens (a changed hash is a failure),
and the conformance corpus read back bit-identically (`gates/ci-mirror.sh` and the corpus
check). They are ranked by what they unblock for the least risk.

| rank | refactor | touches | proof beyond the gates | size |
|---|---|---|---|---|
| R1 | the kit reads the ayanamsha a classical provider defines from `Reckoning::text_ayanamsha` (or the provider's own `ayanamshas`), not a fixed SURYASIDDHANTA | `ephemeris-kit/src/kit.rs` `check_ayanamsha` | the kit's run over `SiddhantaProvider` and `VakyaSky` gives the same verdicts; break it (a wrong value) and see it fail (`break-it-to-find-the-gate`) | 0.5 day |
| R2 | `Reckoning` trait and `RECKONINGS`; `SURYA` and `VAKYA` implemented by moving `open_before`'s two arms into them verbatim; `open_before` dispatches on `siddhanta.reckoning()` | `crates/siddhanta` (new `reckoning.rs`), `sdk/src/ephemeris.rs` | the providers are built by the same calls in the same order. A test opens both ways and compares 10,000 positions bit for bit | 1.5 days |
| R3 | `open()` passes the chain's rest to any classical entry; `is_classical()` replaces the `matches!` | `sdk/src/ephemeris.rs:376` | no behaviour change while only one classical entry exists | 0.25 day |
| R4 | messages and warnings from the registry: `astronomy_coherence`, `classical_name`, `siddhanta-topocentric`, `siddhanta-text-sunrise-convention` | `sdk/src/context.rs`, `core/settings/mod.rs` | the diagnostic snapshot tests: every message byte-identical for Surya and Vakya | 1 day |
| R5 | `Siddhanta::Text { … }` added; `reckoning()` and `is_classical()` accessors | `core/settings/mod.rs`, the schema | the hash goldens unchanged; a round trip of every shipped profile | 1 day |
| R6 | `ContextBuilder::reckoning`; `nepal_official` delegates; one FFI function and one per binding | `sdk/src/context.rs`, `ffi`, generated bindings | `nepal_official`'s tests and the official-calendar corpus unchanged; the new path gives the same context hash | 1.5 days |
| R7 | `SuryaSunrise`, `SuryaPlanets`, `SuryaTrig` renamed `TextSunrise`, `TextPlanets`, `TextTrig`, with type aliases for the old names; the wire spellings unchanged | `core/settings`, the schema's titles | the schema diff shows titles only | 0.5 day, optional |
| R8 | `ReckoningKey` catalogue in the IDL; listings in the MCP | `crates/core` catalogue, the IDL extractor | the generated catalogue pages gain one table; `sweep-every-generated-page` | 1 day |
| R9 | the `CLASSICAL` alias where names are parsed | `mcp/src/lib.rs`, settings reader | both spellings parse to the same entry; output unchanged | 0.25 day |

R1 to R4 are the framework proper. R5 and R8 are what a new text needs on the wire. R6
is what makes it reachable from a binding without code. R7 and R9 are naming.

## 5. How the later methods fit

| method | layer 1 (sky) | layers 2–3 | fit | notes |
|---|---|---|---|---|
| **Ketaki** (modern) | none: a modern engine | profile `ketaki`, catalogue member `KETAKI` | fits now | `ketaki.md` K1 |
| **Ketaki Grahaganitam** (1899 karana) | a karana provider, `karana/ketaki.rs` | profile `ketaki-grahaganita` | fits | the first `Text` member; `ketaki.md` K4 |
| **Tilak** (Revati) | none | profile `tilak-revati` | fits now | C2184 |
| **Rashtriya Panchang** (1957) | none | profile: the RCRC's Lahiri (true basis, 23°15′ at 1956-03-21), the national solar months from Chaitra 1 on 22 March, Saka era | fits now | its civil calendar is a `civil_calendar` member; the RCRC report is in hand |
| **Grahalaghava** (Ganesha Daivajna, 1520) | a karana provider in the same family as Ketaki's: an epoch, cycles of 4016 days, rational motions, small tables | profile `grahalaghava`, its own ayanamsha member (the text's, about 58″ a year; RCRC survey 23°08′ to 23°10′ in 1954) | fits | the Ketaki karana is built in the Grahalaghava's form (KGG v. 2), so the family's arithmetic is shared |
| **Arya paksha, Brahma paksha** | `SiddhantaProvider` with another `Parameters` set: the Aryabhatiya's and the Brahmasphutasiddhanta's revolution numbers and apsides | profiles `arya-paksha`, `brahma-paksha` | fits | the model is the same four steps with other constants. `Parameters::TEXT` becomes one of several `const`s, each a `ReckoningKey` |
| **Parahita** (Kerala, AD 683) | the Aryabhatiya's parameters with the Parahita corrections (*śakābda saṃskāra*) | profile `parahita` | fits | as Arya paksha, plus a correction term in `mean.rs` |
| **Kerala vakya** | a `VakyaSky` variant: the *Vākyakaraṇa* with the Kerala tables and Kerala's sunrise place | profile `kerala-vakya` | fits | extends `crates/siddhanta/src/vakya` |
| **Tamil vakya star planets** | extend `VakyaSky` with the star-planet vakyas (today it borrows Surya's) | the existing `tamil-vakya` | fits | an option within `VAKYA` or a key `VAKYA_PLANETS` |
| **Tibetan (Phugpa, Tsurphu), Bhutanese** | none that fits: these calendars are arithmetic (the *Kālacakra*'s *gza' dhru*, the mean and true weekday, the lunar day by table) and give no sky | a calendar system in `crates/calendar` (`CalendarSystem`, `date.rs:348`) and a profile choosing it | **does not fit layer 1** | they are calendars, not skies. Forcing them behind the ephemeris port would invent positions their texts never give. They belong beside the Bikram Sambat engine (C2197) |

## 6. Phases and estimates

| phase | contents | estimate | gate |
|---|---|---|---|
| F1 | R1, R3 | 1 day | the kit and the SDK suites; corpus bit-identical |
| F2 | R2, R4 | 2.5 days | diagnostics snapshots; the 10,000-position comparison |
| F3 | R5, R8 (and R7 if taken) | 2–2.5 days | hash goldens; generated-page sweep |
| F4 | R6, R9 | 1.5–2 days | `nepal_official` corpus; every binding's smoke test |
| **framework total** | | **4–6 days** (with review and CI) | |
| K1–K2 | Ketaki modern, Tilak (`ketaki.md`) | 4–6 days | independent of F1–F4; may go first |
| K3–K4 | Ketaki karana | 8–11 days | after F4 |
| N1 | Rashtriya Panchang profile | about 2 days | none |
| N2 | Arya and Brahma pakshas | 3–5 days | F2 (a `Parameters` set per key) |
| N3 | Grahalaghava | about 2 weeks | K4 (the karana family) |
| N4 | Parahita | 1–2 weeks | N2 |
| N5 | Kerala vakya; Tamil vakya star planets | about 2 weeks; about 1 week | F2 |
| N6 | Tibetan Phugpa, Tsurphu; Bhutanese | about 1 week each | none of F1–F4; the calendar crate |

Each N phase begins with its research page, as Ketaki's did.

## 7. Risks

| risk | effect | mitigation |
|---|---|---|
| a refactor changes a bit | a stored chart or the official calendar changes | the corpus read back bit-identically on every F phase; a settings-hash golden failure blocks |
| the registry becomes a second settings system | options live in two places | `TextOptions` holds only what all texts share; text-specific choices are keys |
| the `CLASSICAL` alias and the old name both appear in output | consumers compare strings | output keeps `SURYA_SIDDHANTA` until C2196 decides |
| a text that is half calendar, half sky (Tibetan) is forced into layer 1 | invented positions | C2197: such a text is a calendar system |

## 8. Open questions

C2192 (the kit), C2194 (a catalogue member per reckoning's ayanamsha), C2195 (keeping the
`Surya` and `Vakya` spellings), C2196 (the entry's name), C2197 (the Tibetan calendars)
in `01-research/feature-universe/19-verification-cruxes.md`.
