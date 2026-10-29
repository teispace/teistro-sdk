# The compatibility shim: the baseline engine's inputs, forwarded

Status: `design`, 2026-10-28; items 1 to 4 (§7) built the same day,
items 5 to 7 on 2026-10-29, items 8 to 10 not. The last "not met"
row of the parity checklist
([`02-parity-requirements.md`](../01-research/baseline-engine/02-parity-requirements.md),
"migration guide") and the Phase 9 deliverable "the baseline migration
guide and compatibility shim"
([`07-roadmap/00-roadmap.md`](../07-roadmap/00-roadmap.md), "Phase 9").
The specification is the migration guide
([`09-guidelines/baseline-migration.md`](../09-guidelines/baseline-migration.md)):
it maps every baseline service and setting to the SDK, and the shim
carries out that map. The boundary rule comes from
[`ab-baseline.md`](ab-baseline.md) §1 and `CLEAN_ROOM.md`: the SDK never
names the baseline engine and never transliterates its class structure.
The baseline engine's public surface was read on the date above, from its
composition roots, its settings and birth types and the calls its
consuming application makes. This page describes that surface in its own
words.

## 1. Who it is for, and what it is not

It is for an application running on the baseline engine. In practice that
is the Node service that composes the baseline's engines once per process
and calls about thirty of their entry points: the chart foundation and
its slices, the daily panchanga, the transit services, the rashifal
context and score, remedies, rectification, milan, numerology,
Pancha Pakshi, the Bikram Sambat calendar, the zone resolver and the
locale helpers. Such an application keeps two things it cannot change in
one step:

- **its stored rows**: a birth record per person and a settings snapshot
  per chart (the fields `00-inventory.md` lists, "Settings snapshot");
- **its call sites**: hundreds of reads spread over its modules.

The shim lets it keep the first unchanged and move the second one call
at a time. A shim call takes the stored birth record and the stored
snapshot as they are, opens the SDK under the settings the snapshot
means, and makes the SDK call the guide names for that entry point. The
application ports the reads of one answer, ships, and moves on to the
next call. Both engines run side by side in one process the whole time
(guide §9, step 3).

What it is not:

- **Not the baseline's bytes.** Parity is on what a record says, not on
  its bytes. The rows the parity table records as "decided otherwise"
  stay decided: a birth record's name, gender and birth order are not
  inputs to a computation; the Tajika year follows Charak and not the
  baseline's implementation; the composers' text is the SDK's grammar;
  milan follows the *Kalaprakasika*'s tables. The shim does not reopen
  any of them. It forwards to the SDK's answer, and §7 of the guide is
  where that answer differs.
- **Not the baseline's answer shapes.** It returns the SDK's own results
  (§5).
- **Not a second API.** Every shim function is a documented SDK call with
  its inputs mapped. The generated map (§4.4) prints that call beside the
  function, so the final step, dropping the shim, is mechanical.
- **Not permanent.** It is a migration aid with a stated end (§6.3).

## 2. Where it lives (decided)

The shim has two parts:

| part | where | languages | what |
|---|---|---|---|
| the settings translation | `teistro_core::settings::recorded`, through the façade and the C boundary | every binding | a stored snapshot to a profile, a settings patch for each of three contexts, and the ephemeris it needs (§4) |
| the call forwarding | `bindings/node/compat`, the package **`@teistro/compat`** | Node | one function per baseline entry point a consumer uses, taking the stored birth record and answering the SDK's result (§3) |

Nothing is added to the baseline engine's repository except one export
on the A/B adapter that already lives there (§6.4).

**Why the forwarding is Node only.** The baseline engine is a set of
TypeScript packages with a native ephemeris module. Every application
that calls it runs Node. A forwarder in Python, Dart or Java would have
no caller, and each would be another copy of thirty functions to keep
in step. The SDK is reached from those languages directly. The
application that later moves a worker to Python moves it to the SDK, not
to a shim.

**Why the settings translation is language-neutral.** The stored
snapshots outlive the Node process. A data migration that recomputes
stored charts (guide §8.3) is often a script in another language, a SQL
job's helper or a Rust batch, and step 1 of that migration is "map its
stored settings snapshot to a profile and a patch". That step must give
the same answer in every language. Written once in Rust, it reaches every
binding through the generated boundary, and the conformance corpus's
Rust tests use the same function (§6.4).

**Rejected: a forwarding service graph.** The tempting shim mirrors the
baseline's composition roots: the same factory, the same service objects,
the same method names, each forwarding to the SDK, so the application
changes an import and nothing else. In the SDK repository that is a
transliteration of the baseline's class structure, which the clean-room
policy forbids, and it would name the baseline throughout. Kept in the
baseline's own repository, it only pays off if it also returns the
baseline's answer shapes, and §5 rejects that. Without reshaping, a
mirrored graph saves the application nothing over calling `@teistro/compat`
directly.

**Rejected: a subpath of `@teistro/sdk`.** A subpath would tie the shim's
end to the SDK's semantic version: removing it would be a breaking change
to the SDK. A separate package can be deprecated and then stop being
released (§6.3) without touching the SDK's version.

**The package.** `bindings/node/compat/`, named `@teistro/compat`, ES
module, `"private": true` until releases are cut (as every package is
today). Its only dependency is `@teistro/sdk`, as a peer at the exact
same version (§6.2). It has no native code. Its entry is one function:

```ts fragment
import { compat } from '@teistro/compat';
import teimeris from '@teistro/ephemeris-teimeris';

const shim = compat({ ephemeris: [teimeris({ dataDir: './ephe' })], locale: 'ne-Deva-NP' });
const chart = shim.chart(birthRecord, snapshot);   // the SDK's Chart
```

`compat(options)` holds a small cache of SDK contexts keyed by the
translated settings hash: one natal context per distinct snapshot, plus
a KP context opened on first use. Most stored
charts share a handful of snapshots, so the cache is bounded at 16 and
disposes what it evicts. `shim.dispose()` frees them all. The cache
belongs to the module instance, so each worker thread has its own, as
the SDK requires (guide §7, "threads").

Built (item 4): the translations are kept too, at most 64 by the
snapshot's text, so a snapshot costs one `recordedSettings` call however
often it is used (§6.5); two snapshots differing only in a field set
aside share one context. The locale a context opens with is the
snapshot's stored one, else the one `compat` was given. `profile` and
`settings` are refused as options, since each snapshot decides them. The
counts `calls()` reads belong to the module instance, so they are per
thread.

## 3. The call surface

Each baseline entry point a consumer uses, in the inventory's words
([`00-inventory.md`](../01-research/baseline-engine/00-inventory.md)),
with the shim function that takes it and the SDK call that answers. The
kinds:

- **forward**: one SDK call with the inputs mapped;
- **compose**: several SDK calls, or one call and a read across its
  answer;
- **refuse**: a function that throws at once, typed `never`, whose error
  names the guide section that explains why (§6.1). Typed `never` means
  the application's compiler marks the unreachable code after it, so a
  refused entry point is found at the call site and not in production.

Every chart function takes `(birth, snapshot, options?)`. `birth` is the
stored birth record, read as §3.7 says. `snapshot` is the stored settings
snapshot, translated as §4 says.

### 3.1 The core package

| baseline entry point | shim | SDK | kind |
|---|---|---|---|
| the astronomical backend, a position at an instant | `positions` | `ctx.positions({ instants, bodies, frame })` | forward |
| the engine's own operations (Julian day from a date, and the rest) | `engine` | `ctx.engine.call(name, args)` on a Teimeris context | forward |
| the ayanamsha at an instant | `ayanamsha` | `ctx.chart.found(...).ayanamsha` | compose |
| the sunrise service | `sunrise` | `ctx.almanac.day(...)`, its sunrise and sunset | compose |
| the timezone resolver: a civil time in a zone, LMT, DST rules | `resolve` | `ctx.time.resolve(civil, zone)` with `ianaZone`, `localMeanZone` | forward |
| the timezone resolver: a zone from coordinates | `zoneAt` | none | refuse: no geo port (guide §10) |
| ghati-pala, both reckonings | `ghatiPala` | `chart.timing`, read from the natal context and from a second context with the other `day.ghati_reckoning` | compose |
| the Bikram Sambat calendar: conversion, month lengths, leap tests | `bs` | `ctx.calendar.convert`, `dateOf`, `fixedOf`, `monthLength`, `isLeap` | forward |
| the calendar service's era numbers | `eras` | `ctx.almanac.day(...)` with `calendars.eras` | compose |
| the entity registry, a name by type and key | `entity` | `ctx.intl.entity(key)`. Built taking the catalogue's key (`graha.SUN`): the rule that builds it from the baseline's type and key (`crates/intl/src/migrate.rs`) is Rust only, and a JavaScript copy of it would be a second one (§8) | forward |
| numeral, date, degree and clock formatting | `format` | none | refuse: the SDK answers numbers and instants, and formatting them belongs to the application (`Intl.NumberFormat` with a `nu` extension does it in Node) (guide §2) |

### 3.2 The chart package

The baseline computes a foundation and then a slice per route. The SDK
founds once with every section asked for (guide §2). A shim function
asks for exactly the sections its baseline entry point returns. Where
the two engines cut their answers at different seams, it asks for the
larger, as the A/B does ([`ab-baseline.md`](ab-baseline.md) §2). The
first six rows below are A/B requests, and so are `day`, `month`,
`gochar`, `sadeSati`, `varsha`, `kp`, `milan` and `batch` further down.
Their SDK side already exists (§6.4).

| baseline entry point | shim | SDK | kind |
|---|---|---|---|
| the foundation | `chart` | `ctx.chart.found` with every varga, `aspects`, `points`, `houses`, `state`, `jaimini` and the Vimshottari | forward |
| the whole natal chart | `full` | `ctx.chart.found` with the A/B's `full` request: every section, the 18 dashas, every shipped rule set, `avakahada`, `rules.longevity` | forward |
| vargas and vargottama | `vargas` | `found` with `vargas` | forward |
| yogas, doshas | `yogas` | `found` with `rules: { shipped: [...], readings: 'RECORDING_ENGINE' }` | forward |
| Shadbala, Bhava bala, Ashtakavarga, Vimshopaka | `strength` | `found` with `shadbala`, `bhavaBala`, `ashtakavarga`, `vimshopaka` | forward |
| one dasha system, all 18, the active periods at an instant | `dashas` | `found` with the 18 `dashas`, depths from the snapshot (§4.2); the active chain is read from the periods' spans, which every answer carries | forward |
| houses, Bhava-Chalit | `houses` | `found` with `houses: true` | forward |
| avasthas, dignities, planetary war | `states` | `found` with `state: true` | forward |
| drishti | `aspects` | `found` with `aspects: true` | forward |
| upagrahas, special lagnas | `points` | `found` with `points: true` | forward |
| the Jaimini slice | `jaimini` | `found` with `jaimini: true` | forward |
| Ayurdaya, Maraka, longevity windows | `longevity` | `found` with `rules: { longevity: true }` | forward |
| birth timing: ishtakaal, bhayat, bhabhoga | `timing` | `found` with `moment: true` | forward |
| avakhada | `avakahada` | `found` with `avakahada: true` | forward |
| the calendar conversion of the birth | `birthCalendar` | `ctx.calendar.convert` on the birth's civil date | forward |
| the serialiser with presets | `dossier` | `found` with `dossier: { preset }` | forward |
| the gochar sidecar | `sidecar` | none in Node | refuse: the sidecar is Rust only; a binding asks for transits on the chart request (guide §8.2, §10) |
| chart layout geometry | `drawing` | `found` with `drawings` | forward |
| the dossier, day and dasha narrative composers | `narrative` | `found` with `interpret`, rendered by `ctx.intl.render` | compose |

### 3.3 The panchanga package

| baseline entry point | shim | SDK | kind |
|---|---|---|---|
| the daily panchanga | `day` | `ctx.almanac.day` | forward |
| a run of days | `month` | `ctx.almanac.of` | forward |
| next and previous limb changes | `changes` | the day's spans: a span carries its `whole` bounds (guide §4.3) | compose |
| the muhurta finder, windows, blackouts, natal tara and chandra bala | `muhurta` | `muhurta` on an almanac request, `ranking: 'BASELINE'` with the event rules named by the caller | forward |
| choghadiya, hora, Rahu kaal and the kaalas, muhurta yogas | `day` | the same almanac day (each is a member of it) | forward |
| the tithi calendar | `month` | `ctx.almanac.of`, its tithi spans | compose |
| Sarvatobhadra | `sarvatobhadra` | `found` with `sarvatobhadra` | forward |
| eclipses | `eclipses` | `ctx.almanac.of` with `eclipses: true` | forward |
| ingresses | `ingresses` | `ctx.almanac.of`, its sankrantis | compose |
| lunations | `lunations` | `ctx.almanac.of`, the ends of its tithi spans at 15 and 30 | compose |
| Pancha Pakshi | `pakshi` | `ctx.almanac.pakshi` | forward |

### 3.4 The predictive package

The transit functions answer on the natal context, whose
`gochar.transit_centre` casts them from the Earth's centre as the
baseline does (§4.2; guide §5.4; crux C778).

| baseline entry point | shim | SDK | kind |
|---|---|---|---|
| gochar overlays, transit aspects, phala with vedha, transit strength | `gochar` | `found` with `ashtakavarga` and `gochar: { instants, ashtakavarga: true }` | forward |
| Sade Sati | `sadeSati` | `found` with `sadeSati: { from, to }` | forward |
| the transit calendar | `transitCalendar` | `found` with `hits` | forward |
| the Tajika annual chart, muntha, sahams, Mudda dasha | `varsha` | `found` with `varsha` | forward |
| solar, lunar and planetary returns | `returns` | `found` with `hits: returnsRequest(from, to)`; the solar return through `varsha` | compose |
| secondary and tertiary progressions, solar arc directions | `progressions` | `found` with `progressions` | forward |

### 3.5 The applied package

| baseline entry point | shim | SDK | kind |
|---|---|---|---|
| KP: sub-lords, significators, ruling planets, Placidus with fallback | `kp` | `kp: {}` on the KP context (§4.3) | forward |
| milan: Ashta Koota, extended checks, Mangal match, marriage doshas | `milan` | `found` with `matching: { partner, partnerRole }` | forward |
| quick milan, akshar resolution | `naam` | `ctx.matching.naam(bride, groom)` | forward |
| rectification | `rectify` | `found` with `rectification: { baseline: {...} }`, the birth record's gender and time accuracy carried in (§3.7) | forward |
| prashna | `prashna` | `found` with `prashna: { rules: { timing: 'BASELINE', mook: 'BASELINE', score: 'BASELINE' } }` | forward |
| Lal Kitab | `lalkitab` | `found` with `lalkitab` | forward |
| numerology | `numerology` | `ctx.numerology.profile(name, date, rules)` with the baseline readings (guide §5.3) | forward |
| remedies | `remedies` | `found` with `remedies: { rules: { functional: { scheme: 'BASELINE' } } }` | forward |
| remedies: gemstone safety | `gemstoneSafety` | none | refuse: not built (parity table, Track B row) |
| namakarana: validation | `nameCheck` | `found` with `avakahada`, its star and pada given to `ctx.naming.check(names, birth)` (item 9 of `namakarana.md` landed, so the row composes rather than refuses) | compose |
| namakarana: suggestions, the offline corpus, the language-model path | `nameSuggestions` | none | refuse: no corpus ships, and the consumer loads its own name book (`namakarana.md` §3) |
| research: batch | `batch` | `ctx.chart.foundMany` | forward |
| research: synastry, composites | `synastry` | `found` with `synastry` | forward |
| research: statistics | `research` | `ctx.research.counts`, `compare`, `expected` | forward |

### 3.6 The rashifal and interpretation packages

| baseline entry point | shim | SDK | kind |
|---|---|---|---|
| rashifal context and score, one sign or all twelve, per period | `rashifal` | `ctx.chart.rashifal(request, period)` and `rashifalMany`, with `snapshot: { at: 'CLOCK', hour: 6, minute: 0 }` | forward |
| state tables, yoga and dosha readings | `reading` | `ctx.intl.render(key, params)` over the packs from `migrate readings` and `migrate states` | forward |
| the milan resolver and verdict composer | `milanVerdict` | `found` with `matching` and `interpret`, rendered | compose |
| the graha-bhava, tara chakra, yoga timing and namakarana note composers | `narrative` | as in §3.2 | compose |

### 3.7 The birth record

The shim reads the stored birth record. It maps the record's fields as
follows:

| the record holds | the shim |
|---|---|
| the instant, as a UTC date and as a UT Julian day | takes the date as the `instant`, the UT Julian day `2440587.5 + ms / 86 400 000`, and refuses the record when its Julian day stands more than 43.25 ms from it (§6.1) |
| latitude, longitude, altitude | the `place` |
| an IANA zone, or a local mean time flag | the clock's `utcOffsetSeconds`, from `ctx.time.civilOf(instant, zone)` with `ianaZone(name)` or `localMeanZone(longitude)` |
| the time's accuracy | read only by `rectify`. Every other call founds at the instant given, as the baseline does |
| gender | read only by `rectify` (its `sex`) |
| name | read only by `numerology` and `naam` |
| birth order, a Bikram Sambat date | ignored. They are the application's data, not a computation's input (parity table, "birth data", decided otherwise). The instant already fixes the date |

The record's spelling is a data format the application has stored, read
the way the corpus harness reads the exporter's files. It is not the
baseline's class structure. The type the package declares for it is
written from the stored rows, in the package and nowhere else.

## 4. The settings translation

### 4.1 The single source

Today the baseline's settings appear in four places: the guide's §5 and
§6 tables (prose), `conformance-baseline` (`crates/core/src/settings/profiles.rs`,
the default snapshot only), the A/B's `CONVENTIONS.same`
(`bindings/node/bench/ab.mjs`, the KP and transit contexts) and the
corpus tests that found a recorded variant (true node, geocentric,
Placidus, four ayanamshas and the rest), each with its own patch. The
shim would be a fifth. Instead there is one:

- **The table** is `teistro_core::settings::recorded::ROWS`, a Rust
  `const` slice. A row is a stored field, a stored value, and what it
  becomes: a `fn() -> SettingsPatch` (typed, checked by the compiler, and
  serialisable for the page), an ephemeris requirement, a note, or a
  refusal naming the guide anchor that explains it. Where the field's
  values form a catalogue set, the row names the attribute that lists
  them explicitly: the ayanamsha number is the `ayanamsha` member's
  `swiss_mode` (`catalogue/ayanamsha.yaml`). Where no attribute exists,
  the table lists the values. The 18 dasha systems' stored spellings are
  listed one by one (`DASHA_SYSTEMS`). Built, every one of them turned
  out to be its SDK key lower-cased, Padanadhamsa included, so a case
  rule would have mapped them; they stay listed so a spelling the SDK
  changes cannot be mapped wrongly in silence.
- **The function** is `recorded(snapshot) -> Result<Recorded, Refused>`.
  It folds the rows over the `conformance-baseline` profile. A stored
  field the table does not know is refused, never skipped, and so is a
  stored value it does not know.
- **The boundary** carries it as one JSON-in, JSON-out entry,
  `ts_recorded_settings(context, snapshot_json, out_json)`, generated
  into every binding and wrapped as the context's `recordedSettings(snapshot)`
  (Node, wasm, Dart, Java) and `recorded_settings` (Python); Rust calls
  `recorded` itself. It hangs off a context, as every JSON entry does, so
  a refusal crosses whole as the context's error record (field and crux);
  the translation reads none of the context's settings.
- **The page** is generated: `cargo xtask compat` renders the table into
  the generated map (§4.4), and the guide links that page instead of
  carrying the table. The guide's §5.1 knob table already comes from the
  profile's source. The new snapshot table comes from `ROWS`.
- **The checks**, all in `check-compat` (§6.4): the translation of the
  corpus's `default` snapshot is `conformance-baseline` with no knob
  changed except the dasha depth the snapshot stores (the gate compares
  the resolved settings knob by knob and prints any difference); each of the manifest's 13 recorded profiles
  (whose `settings_hash.canonical` is the stored snapshot, field for
  field) translates or is refused by a named row; the A/B's
  `CONVENTIONS.same` is deleted and the runner calls `recorded` on the
  default snapshot instead, so the A/B and the shim cannot drift apart.

`Recorded` holds `natal` and `kp`, each an `Opening`: a profile and a
settings patch over it, which `Opening::resolve` resolves. It also holds
`ephemeris`, a required entry such as `'SURYA_SIDDHANTA'` or none,
`locale` (a locale tag or none) and `notes`, the stored fields read and
deliberately ignored.

### 4.2 The rows

In the snapshot's own terms, as the inventory lists the fields:

| stored field | stored values | becomes |
|---|---|---|
| the ayanamsha | any of the 47 numbers, as the recording engine and the application's rectification rows store it; any of the 48 keys, as the application's charts and preferences store it | `frame.ayanamsha`: the member whose `swiss_mode` is the number, or the member the key stands for in `AYANAMSHA_KEYS` (the 47, and `suryasiddhanta`, an older spelling of `surya-siddhanta`). A number no member has, or text that is not one of the keys, is refused on `snapshot.ayanamsaId` with this table as its hint |
| the zodiac | sidereal | `frame.zodiac`, the profile's own |
| | tropical | a note: the recording engine casts the same sidereal chart under it. Measured: the corpus's four tropical variants equal their default charts field for field (positions, houses, vargas) |
| the node | mean, true | `frame.node` |
| topocentric | true, false | `frame.centre`: `TOPOCENTRIC`, `GEOCENTRIC` |
| the house system | the 22 names | `houses.placement_system`, the member whose key, lower-cased with hyphens, is the name. The chalit stays `VEHLOW` (registry entry 14). Any system but the profile's whole-sign also sets `houses.polar_policy` `FALLBACK_PORPHYRY`: the recording engine divides an undefined polar arc as Porphyry does, silently, as its Tromsø Placidus variant shows (reproduced to 4e-8 degrees) |
| the siddhanta | drik | the profile's own |
| | surya | `frame.siddhanta` `SURYA` with bija `NONE`, the modern sunrise and the text's planets; `frame.centre` `GEOCENTRIC` and `frame.ayanamsha` `SURYASIDDHANTA`, whatever the snapshot's centre and ayanamsha say; and `ephemeris: 'SURYA_SIDDHANTA'`. Each is what the recording computes, measured on its two surya variants: its sunrise is the modern one (reproduced to 1.5e-5 days), its places are the text's own zodiac (1.3 to 5.1 degrees off under Lahiri, 0.04 to 3.7 under the text's) and geocentric (the text has no observer) |
| the Surya bija | true | refused (C777) |
| the sunrise mode | apparent-refraction | the profile's own, `UPPER_LIMB_REFRACTION` (registry entry 12) |
| | geometric | `day.sunrise` `CENTRE_NO_REFRACTION` |
| | upper-limb, lower-limb | `day.sunrise` `CUSTOM` at the baseline's altitude, −0.5667° and −1.1° (C776) |
| the dasha balance | spatial, temporal | `dasha.balance`: `SPATIAL`, `TEMPORAL` |
| the dasha depth, overall and per system | 1 to 5; the 18 systems' spellings | `dasha.depth` for each system |
| the chara karakas | 7, 8 | `jaimini.chara_karakas`: `SEVEN`, `EIGHT` |
| the Ekadhipatya method | classical | the profile's own, `EMPTY_TO_ZERO` with `SARVA` (C59, C60) |
| | zero, transfer | refused: no text gives them and the baseline's own documentation calls them legacy; the SDK's two readings are BPHS's and the recorded one (C60) |
| the locale's language and script | ne, en, hi, sa | `locale`: `ne-Deva-NP`, `en-Latn`, `hi-Deva-IN`, `sa-Deva`. Any other pairing is refused |
| the locale's numeral system, calendar, clock | any | a note: presentation, which the shim does not do (§3.1, `format`) |
| the version triple | any | a note, kept for the stored chart's comparison (guide §8.3) |

Three settings the snapshot does not hold are the baseline's fixed
conventions, so `recorded` sets them on every translation. Until item 5
they lived in `ab.mjs`'s `CONVENTIONS.same`, which now reads them from
`recorded`:

- **the KP context:** `kp-default`, with the translated frame's node,
  centre and ayanamsha basis, the day's polar policy and the natal
  context's ruling planets (`kp.ruling_count` `WITH_SUBS`,
  `kp.day_lord_day` `CIVIL`, cruxes C150, C151) laid over it. This is
  what the A/B measured as 98.4% agreement on the four lord levels and
  100% on the ruling planets. A caller who wants KP under the chart's own ayanamsha passes
  `kp(birth, snapshot, { anyAyanamsha: true })`, which forwards
  `kp: { anyAyanamsha: true }` on the natal context instead (guide §5.3,
  C157);
- **the transits:** `gochar.transit_centre` `GEOCENTRIC`, already
  `conformance-baseline`'s, since the baseline casts transits from the
  Earth's centre whatever the snapshot says (guide §5.4, crux C778): one
  context, where the A/B opened a second;
- **the polar day:** the profile's `NEAREST_EVENT`.

### 4.3 Cruxes recorded

Two rows are choices where the baseline's names and its arithmetic
disagree. Both are added to
[`19-verification-cruxes.md`](../01-research/feature-universe/19-verification-cruxes.md):

- **C776, the sunrise modes.** The baseline drives every mode to a fixed
  geometric altitude of the Sun's centre (`apparent-refraction` −0.8333°,
  `geometric` 0°, `upper-limb` −0.5667°, `lower-limb` −1.1°). Two names
  say a limb the altitude does not mean: −0.5667° is the centre with 34′
  of refraction, and −1.1° is no limb with refraction at all (the lower
  limb with refraction is about −0.3°). Decided: translate by what the
  mode computes, not by its name. The two modes the corpus records
  (apparent-refraction, geometric) keep the named conventions their
  variant fixtures already hold, and the other two become `CUSTOM` at the
  baseline's own altitude. It is reopened by a recorded variant under
  either of the other two modes.
- **C777, the Surya bija.** The baseline's bija is one epoch-anchored
  correction of the mean node alone. The SDK's bija is a named or counted
  set of revolutions for every body (`siddhanta.md`; a bare switch is
  refused there), and none of the named sets is a node-only re-anchoring.
  Decided: refuse, naming the crux. It is reopened if a text or an almanac
  is found that applies a node-only correction. A consumer can then ask
  for a named set.

### 4.4 The generated map

`cargo xtask compat` writes `docs/09-guidelines/baseline-map.md`, held
by `check-compat`. It has three tables: §3's surface, from
`bindings/node/compat/surface.json`; §4.2's rows, from `ROWS`, each with
its patch as JSON; and the counts (§6.4). The guide's §4 concept map is
then replaced by a link to that page, so the guide and the shim cannot
disagree about a single row. The guide's prose sections (§2, §5.3 to
§10) stay hand-written.

## 5. Answer shapes (decided: the SDK's)

A shim function returns exactly what the SDK call returns: a `Chart`, an
`Almanac` day, a `Rashifal`. The map of fields is the generated page's
"reads" column, which is the guide's "result" column as it is today.

| | the SDK's shapes | the baseline's shapes, rebuilt |
|---|---|---|
| maintenance | none beyond the SDK's own generated types | a second declaration of every baseline result type (dozens, nested, keyed by its own names), hand-kept against an engine the SDK cannot run in its gates, so nothing reads the declarations to check them |
| type safety | the generated `.d.ts`, checked by `bindings/node/typecheck` | types written from reading another repository, unchecked |
| fidelity | provenance, spans and keys intact | rebuilding would invent what the SDK does not answer: four-language name blocks inside results (guide §2), a 1-to-30 tithi number (registry entry 19), the lagna's placeholder dignity (entry 22). Each invention is a copied defect or a fabrication |
| bit agreement | the shim's answer is the SDK's, so it can be held to the bit (§6.4) | unprovable: a reshaped answer is the shim's own product |
| clean room | nothing of the baseline's in the repository | its type structure transliterated |
| the consumer's cost | ports the reads of each call when it moves that call | ports nothing at first, then ports everything again when it drops the shim |

The last row decides it. Reshaping does not save the port. It postpones
it and makes the application do it twice. The shim saves the port of the
inputs: the stored rows, the settings snapshots and the contexts. That
is the part an application cannot do one call at a time.

One aid crosses the gap. `@teistro/compat` exports `facts(request,
answer, options)` and `FAMILIES`, the A/B's shared vocabulary
(`ab-baseline.md` §10), which reads the fields both engines answer as flat
named facts and says how each family compares. Built with item 6: it
moved from `bench/ab-agree.mjs` into `lib/facts.js`, and the A/B reads its
SDK side through it. `options.figures` names the yogas to report present
or absent (the A/B passes its joined list; left out, the present rules
alone are reported), and `options.window` is a Sade Sati answer's range.
An application running both engines side by side (guide §9, step 3)
compares the facts of the two answers instead of writing its own
comparison.

## 6. Errors, versions, the end, tests and budget

### 6.1 Errors

- An SDK error passes through untouched: the same `TeistroError`, the
  same record, never wrapped and never reclassified (the error carries
  its record).
- A refused entry point, stored field or stored value throws
  `CompatRefused` before any SDK call. Its `row` is the surface or
  settings row's id and its `guide` is the anchor in the migration guide
  or the generated map. The message says what to do instead, in one
  sentence.
- A birth record whose Julian day and UTC date disagree throws
  `CompatRefused` with row `birth.instant`. The date is the instant and
  the Julian day is checked against it, to 43.25 ms (settled with item
  5). The baseline derives the Julian day from the date wherever it
  stores both: the corpus's 115 records (its 55 charts and 60 variants)
  hold them at most 0.0195 ms apart, a double's rounding of a Julian day
  of this era, and the application founds every chart from the date,
  which its birth table keeps to the second, while it keeps the Julian
  day to six decimals, a step of 86.4 ms. A millisecond, the first
  tolerance, would have refused all but about one stored row in 43.
  So the tolerance is half that step, 43.2 ms, with 0.05 ms for the two
  roundings of a double (where the day was derived and where its decimal
  was read); founding at the stored Julian day instead would move a
  chart by up to 43 ms from the one the application computed. The
  refusal asks for the Julian day to be derived from the date again.
- A stored field or value the translation refuses reaches the shim as
  the SDK's `TeistroError` on `snapshot.<field>`. The shim throws it as
  `CompatRefused` with that field as its row and the SDK's error as its
  `cause`, record whole (built, item 4): the refusal is of the stored
  row, not of a computation, and the crux stays on `cause.hint`. Any other
  error the translation raises passes through untouched. Until the
  generated map exists (item 8), the guide anchor of a settings refusal
  is the guide's §8.3 and of a birth refusal its §8.2.

### 6.2 Versions

The shim's version is the SDK's. It is released in lockstep, with
`@teistro/sdk` as an exact peer dependency, so there is no compatibility
matrix. The table's rows change only when the SDK's settings do. A
profile version bump that changes a default changes the translated
settings hash, which a stored chart's comparison then shows (guide §8.2).

### 6.3 The end

The shim exists so that the baseline engine's application can delete
the baseline's packages. Its end is set in its README and CHANGELOG from
the first release:

- it is marked `@deprecated` in every export's documentation once the
  last baseline application reports no shim calls left;
- it stops being released at the first SDK minor version after that, or
  at the SDK's 2.0, whichever comes first;
- `recordedSettings` stays in the SDK. Stored snapshots outlive the
  migration, and reading one is a permanent need.

To make "no shim calls left" observable without telemetry, `shim.calls()`
returns, per row, how many times this process called it. An application
logs it once a day. When every count is zero, it has finished porting.

### 6.4 Tests and gates

- **Bit agreement (built with item 5).** The A/B's SDK side (`sdkSide`
  in `bench/ab.mjs`) moves into the package as the first fourteen
  forwards (`lib/forward.js`), and the A/B runner imports them from
  there. Every timed A/B request is then a shim call, and `check-ab` (not
  slower than the baseline) holds the shim too. The fifteenth request,
  `muhurta`, moved with item 6: the runner asks the shim's `muhurta`
  request on the context the baseline's search frame opens
  (`MUHURTA_FRAME`, C781).
  A new `bindings/node/compat/test/agree.test.mjs` asks each forward and
  compose function for every one of the corpus's 55 inputs and the
  documented SDK call directly, and asserts the two answers'
  `provenance.contentHash` are equal. The content hash is taken over the
  canonical bytes, so equal hashes are agreement to the bit. Built with
  item 6 for every answering row: an answer the SDK does not seal (a
  date, a zone's resolution, a composed record) is compared by its JSON,
  and the rows the A/B does not time are asked of one chart in five
  (rectification and research, the costliest, of one in eleven), which
  keeps the test near a minute. `engine` is held
  to the same refusal on both sides, since the built-in ephemeris has no
  operations of its own.
- **The translation reproduces the corpus.** A Rust test founds each
  recorded variant fixture under `recorded(its canonical snapshot)` and
  holds it to the golden tolerance. The ad hoc per-test variant patches
  are deleted in the same change, so the translation has a reader on all
  13 profiles.
- **Coverage, counted.** `surface.json` lists every row of §3 with its
  kind. `bindings/node/compat/test/surface.test.mjs` asserts the
  package's exports are exactly the rows' functions, that each refuse
  row throws `CompatRefused` with a guide anchor that exists, and that
  each forward and compose row is called by `agree.test.mjs`. The
  baseline's own list is counted from outside: the A/B adapter (in the
  baseline's repository) gains a `surface` export listing the entry
  points its engine offers by these row ids, and `measure-ab` records how
  many have no row. `check-compat` renders the generated map with the
  counts (forwarded, composed, refused, unmapped) and fails on any
  unmapped entry point, any row whose guide anchor is dead, and any
  difference between the generated page and the sources.
- **Typecheck.** `bindings/node/compat/typecheck/consumer.ts` uses
  every function once, including a refused one in a `never`-returning
  position, and runs under `check-node` beside the binding's own.

### 6.5 Size and speed

- **Size:** plain JavaScript with no dependency beyond the peer. The
  estimate was at most 40 KB unminified with its declarations; built
  with item 7 it is 110 KB (`lib/` and `surface.json`): 62 rows, their
  declarations, and the A/B's vocabulary (24 KB). It is recorded in
  `SIZES` like the other packages with item 10, which sets the bound
  from that measure.
- **Speed:** a shim call adds one input mapping and one cache lookup to
  the SDK call. Because the A/B times shim calls (§6.4), `check-ab`
  holds the shim to "not slower than the baseline engine". No separate
  overhead claim is made: one smaller than the A/B's measured spread
  would be refused by its results schema anyway.
- **Opening:** one `recordedSettings` call and one context per distinct
  snapshot, amortised by the cache. The A/B's cold-start figure (§6 of
  its page) is unchanged for an application with one snapshot.

## 7. Work items

In order. Each is one pull request.

| # | item | touches | estimate |
|---|---|---|---|
| 1 | built: `teistro_core::settings::recorded`: `ROWS`, `recorded()`, `Recorded`, `Refused`; unit tests over every row; crux rows C776 and C777 | core, cruxes | 1 day |
| 2 | built: the corpus founded through the translation (`ephemeris_kit::variants`, held on every built-in tier with its measured divergences), the ad hoc patches deleted, the `default` snapshot held to `conformance-baseline` knob by knob | sdk tests, ephemeris-kit | 1 day |
| 3 | built: the boundary, one JSON entry through the IDL, generated into every binding and wrapped as the context's `recordedSettings` in Node, wasm, Dart and Java and `recorded_settings` in Python; the parity example is not written | ffi, idl, bindings | 1 day |
| 4 | built: `@teistro/compat` skeleton: `compat()`, the context cache, the birth record (§3.7), `CompatRefused`, `calls()`, README with the end (§6.3), typecheck consumer; its tests and the consumer run under `check-node`, the README is held by `check-guides` | bindings/node/compat, xtask | 1 day |
| 5 | built: the A/B's SDK side moved into the package (`chart`, `full`, `vargas`, `yogas`, `strength`, `dashas`, `day`, `month`, `milan`, `varsha`, `kp`, `gochar`, `sadeSati`, `batch`); `CONVENTIONS.same` replaced by `recordedSettings`, the KP opening given the natal context's ruling planets; `agree.test.mjs` over the corpus's 55 charts and KP under the chart's own ayanamsha; the birth record founded at its date (§6.1) | node bench, compat | 1 day |
| 6 | built: the remaining forward and compose rows of §3 (62 rows in all: 46 forward, 11 compose, 5 refused), with `facts()` and `FAMILIES` moved from `ab-agree.mjs` into `lib/facts.js` and the A/B's `muhurta` onto the shim's request; a `variant` context per snapshot for the muhurta frame and the second ghati reckoning; each row in `agree.test.mjs` against the documented call | compat, node bench | 2 days |
| 7 | built: `surface.json`, read by the package for its rows and refusals; the refused rows typed `never` (`Refused`); `surface.test.mjs` holding the exports, the refusals' anchors, the declarations, §3 and `agree.test.mjs` to it | compat | 0.5 day |
| 8 | `cargo xtask compat` and `check-compat`: the generated `baseline-map.md`, the guide's §4 replaced by its link, the parity row updated, the gate added to fast-check | xtask, docs, CI | 1 day |
| 9 | the adapter's `surface` export (in the baseline's repository) and the unmapped count in `measure-ab` | node bench, ab-measured | 0.5 day |
| 10 | release wiring: the package in the release pipeline's install checks and `SIZES`, the CHANGELOG entry | release, sizes | 0.5 day |

About nine and a half days in all. Items 1 to 3 stand alone and are worth
having without the rest: they close guide §8.3 step 1 in every language.
The parity row "migration guide" is met when item 8's gate passes with
zero unmapped entry points.

## 8. Open questions

- **KP's ayanamsha in the application (read with item 5: no call).**
  The application registers the baseline's KP services and injects them
  nowhere, so no stored chart was cast through KP under any ayanamsha.
  The shim keeps the A/B's reading, Krishnamurti's ayanamsha on the
  chart's frame (§4.2), with `{ anyAyanamsha: true }` for the chart's
  own. Open only until an application on the baseline calls KP.
- **The surya variant (measured: it does not reproduce).** Under the
  §4.2 row the sunrise and sunset reproduce to 1.5e-5 days and the node
  to 0.015 degrees, but the star planets stand 0.04 to 0.95 degrees from
  the recording, the Moon 3.5 and 3.7, the lagna 0.66 and 1.37, and the
  speeds up to 1.75 degrees a day; the recording writes no latitude or
  distance (0). The recording engine's Surya Siddhanta is a different
  reading of the text, not a different setting, so the miss is
  registered (`surya-text-theory` in `ephemeris_kit::variants::KNOWN`)
  rather than patched. Open: which reading the application's stored
  surya charts should keep. A consumer migrating them sees the SDK's
  reading, and its own stored positions differ by the figures above.
- **The stored ayanamsha's spelling (settled: both are read).** The
  recording engine's settings and the application's rectification table
  hold `ayanamsaId` as the number; the application's chart settings and
  user preferences hold it as text, a key of up to 32 characters
  (`'lahiri'`, `'raman'`, `'krishnamurti'`). The row reads either. The
  keys were read from the application's own ayanamsha list, which maps
  each to the engine's number: 47 keys, one per number, and
  `suryasiddhanta`, an older spelling of `surya-siddhanta` its per-chart
  rows still hold, 48 in all. They map one to one onto the catalogue's
  47 members, the older spelling aside. Forty-three are the member's key
  lower-cased with hyphens and four are not (`de-luce`, `surya-siddhanta`,
  `surya-siddhanta-msun`, `galcent-gil-brand`), so they are listed
  (`AYANAMSHA_KEYS`), not derived. One snapshot holds one `ayanamsaId`,
  so a key and a number cannot disagree within it. A key is matched as
  stored, lower-cased; any other text, a number written as text
  included, is refused on `snapshot.ayanamsaId`. Item 8's generated map
  counts them.
- **Locale pairings.** The baseline lets a language be paired with
  another script, such as Nepali in Latin script. The SDK ships four
  locales. Whether any stored snapshot actually holds another pairing is
  unknown until the application's rows are counted. Until then they are
  refused.
- **The entity key from the baseline's type and key (item 6).** `entity`
  takes the catalogue's key. The rule that builds it from the baseline's
  entity type and key, with its 19 aliases, is `teistro_intl::migrate::catalogue_key`
  in Rust. Open: carry that rule through the boundary (one JSON entry, as
  `recordedSettings` is) so `entity(snapshot, type, key)` reads the
  baseline's own spelling, or leave the mapping to the application's
  one-time migration of its keys.
- **The milan verdict (item 6).** No composer says a milan: `milanVerdict`
  answers the milan with the doshas' rules read and their readings said
  (`interpret: { readings: true }`), beside the kootas. Open: whether a
  matching composer is wanted, which would replace the readings with
  the kootas' own plans.
- **The muhurta frame's home (item 6).** The baseline searches a muhurta
  from the Earth's centre with the true node (C781). The shim opens that
  context from `MUHURTA_FRAME` in `lib/forward.js`, a patch over the
  translated natal opening, as the A/B did. It is a fixed convention of
  the baseline, like the KP opening, so it may belong in `Recorded` as a
  third opening; that is a boundary change, left for item 8.
