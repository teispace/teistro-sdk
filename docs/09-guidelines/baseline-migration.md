# Moving from the baseline engine

Status: `draft`, 2026-10-27. The Phase 9 migration guide
([`07-roadmap/00-roadmap.md`](../07-roadmap/00-roadmap.md), "Phase 9";
the last line of
[`01-research/baseline-engine/02-parity-requirements.md`](../01-research/baseline-engine/02-parity-requirements.md)).
Every API name below was checked against the source on the date above.
Every figure is a link to the generated page that gates it, never a
copy, so this page cannot drift from it.

## 1. Who this is for

You run an application on the baseline engine: the Node packages with a
core, a chart pipeline, a panchanga, a predictive layer, an applied layer
(KP, milan, rectification, prashna and the rest) and a rashifal layer
([`00-inventory.md`](../01-research/baseline-engine/00-inventory.md)). You
want the same answers from the Teistro SDK, in Node or in another
language, and you want to know where the answers will differ and why.

This page gives you five things:

- a map from each baseline service to the SDK call that replaces it;
- a map from each setting the baseline stores to the SDK knob;
- the way to reproduce the baseline's numbers, and where you cannot;
- what happens to charts you have already stored;
- a plan for running both engines side by side and then switching.

The SDK is not a drop-in replacement. No compatibility shim exists yet
(§10). You change your calls, not only an import.

## 2. What is different in kind

Four changes apply to every call you port.

1. **One context holds the settings.** A context is opened once with a
   profile, an optional settings patch, a locale and an ephemeris. Every
   request on it uses those settings. A request names what to compute,
   never which school to compute it under
   ([`03-design/settings-and-profiles.md`](../03-design/settings-and-profiles.md) §5).
2. **The SDK answers keys, not names.** A result carries catalogue keys
   (`graha.SUN`, `rashi.LEO`). The locale layer turns a key into text
   (`ctx.intl.entity(key)`, `ctx.intl.render(key, params)`). The baseline
   puts four-language name blocks inside its results; the SDK does not
   ([`01-pain-points.md`](../01-research/baseline-engine/01-pain-points.md)).
3. **A chart is founded once and asked for sections by name.** The
   baseline computes a foundation and then a slice per route. The SDK's
   `ctx.chart.found(request)` takes the instant, the place and every
   section you want, and computes them in one call. A section you do not
   ask for costs nothing.
4. **Every result carries its provenance.** The settings hash, the input
   hash, the content hash, the provider and the versions travel with the
   answer (ADR-0020,
   [`08-decisions/adr-0020-calculation-version-and-provenance.md`](../08-decisions/adr-0020-calculation-version-and-provenance.md)).

The SDK is also not Node-only. The same calls exist in Python, Dart,
Java, Rust and C, and over WebAssembly. The site's surface page lists the
spelling in each binding
([`site/content/docs/surface.mdx`](../../site/content/docs/surface.mdx)).
The examples here are Node's, because the baseline engine is Node.

## 3. Opening a context

The baseline reads Swiss Ephemeris files through a native module with
global state. The SDK takes an ephemeris as a context option (ADR-0029,
[`08-decisions/adr-0029-the-provider-plugin-model.md`](../08-decisions/adr-0029-the-provider-plugin-model.md)).
Teimeris reads the same `.se1` file family for 1800 to 2399
([`adapters/README.md`](../../adapters/README.md)):

```ts title="adapters/ephemeris-teimeris/node/typecheck/consumer.ts"
import { Context } from '@teistro/sdk';
import teimeris from '@teistro/ephemeris-teimeris';

const sdk = new Context({
  profile: 'parashari-classical',
  ephemeris: [teimeris({ dataDir: './ephe' }), 'BUILTIN'],
});
```

The list is a chain, tried in order. `'BUILTIN'` is the SDK's own
analytic ephemeris, which needs no files. Name it only if you accept it
as the fallback.

Nothing is published yet. The SDK's version is `0.0.0` and every package
is marked unpublishable until a release is cut. Until then, build from
source ([`site/content/docs/install.mdx`](../../site/content/docs/install.mdx),
"From source").

## 4. The concept map

The left column is the baseline's package and service, as the inventory
names them. The right column is the Node call or the chart request field
that replaces it, and the result member it fills. `ctx` is a `Context`.

### 4.1 The core package

| baseline | SDK |
|---|---|
| astronomical backend over the native ephemeris | the `ephemeris` option of `new Context(...)`; `ctx.positions({ instants, bodies, frame })` for a grid of positions; `ctx.engine` for the engine's own operations (`ctx.engine.call(name, args)`) |
| siddhanta router, `drik` or `surya` with bija | the `frame.siddhanta` knob; the `surya-siddhanta` and `nepali-committee` profiles; the `'SURYA_SIDDHANTA'` ephemeris; `nepalOfficial(modern)` opens the committee's reckoning in one call |
| 47 ayanamshas | the `frame.ayanamsha` knob: a catalogued key (`ayanamsha.LAHIRI`, `ayanamsha.RAMAN`, `ayanamsha.KRISHNAMURTI`, …) or a custom definition |
| timezone resolver | `ctx.time.resolve(civil, zone)` with `ianaZone(name)`, `fixedZone(seconds)` or `localMeanZone(longitude)`; `whenUnknown(day)` for a birth with no recorded time |
| sunrise service and its modes | the `day.sunrise` knob; every almanac day and every chart reckons its day from it |
| ghati-pala, civil and proportional | the `day.ghati_reckoning` knob; a result's `GhatiPala` fields; `chart.timing` for a chart's ishtakaal and hora |
| BS calendar | `ctx.calendar.convert(date(Calendar.BikramSambat, y, m, d), Calendar.Gregorian)`, and `dateOf`, `fixedOf`, `monthLength`, `isLeap` |
| entity registry with four-language names | `ctx.intl.entity(key)` in the context's locale; the names came from the baseline's tables (`teistro-intl migrate baseline`, [`crates/intl/src/migrate.rs`](../../crates/intl/src/migrate.rs)) |
| locale and numeral services | `ctx.intl.render(key, params)`, `ctx.intl.transliterate(text)`; locales `en-Latn`, `ne-Deva-NP`, `hi-Deva-IN`, `sa-Deva` |
| calendar service with era numbers | the `calendars.eras` knob; `nepalSambat: true` on an almanac request |

### 4.2 The chart package

Every row is a field of the request to `ctx.chart.found(...)` (or
`ctx.chart.foundMany(...)` for many instants at one place) and the member
of the `Chart` it fills.

| baseline | request field | result |
|---|---|---|
| foundation | `instant`, `place`, `utcOffsetSeconds`, `kind` | `chart.grahas`, `chart.lagnaDeg`, `chart.ayanamsha`, `chart.day` |
| house systems, Bhava-Chalit | `houses: true` for the houses service's reading | `chart.bhavas`; `chart.houses` and `chart.chalit` are always there |
| vargas | `vargas: [Varga.D9, …]` | `chart.vargas` |
| dignities and avasthas | `state: true` | `chart.states` |
| aspects | `aspects: true` | `chart.aspects` |
| upagrahas, special lagnas | `points: true` | `chart.points` |
| yogas and doshas | `rules: { shipped: ['YOGAS', 'DOSHAS'] }` | `chart.rules` |
| Shadbala | `shadbala: true` | `chart.shadbala` |
| Ashtakavarga | `ashtakavarga: true` | `chart.ashtakavarga` |
| Bhava Bala | `bhavaBala: true` | `chart.bhavaBala` |
| Vimshopaka | `vimshopaka: true` | `chart.vimshopaka` |
| the 18 dasha systems and their trees | `dashas: [DashaSystem.…]` | `chart.dashas` |
| Jaimini slice | `jaimini: true` | `chart.jaimini` |
| Ayurdaya, Maraka, longevity | `rules: { longevity: true }` | `chart.rules` |
| birth timing: ishtakaal, bhayat, bhabhoga | `moment: true` | `chart.timing`, `chart.moment` |
| avakhada | `avakahada: true` | `chart.avakahada` |
| serialiser with presets | `dossier: { preset: 'FULL' }` | `chart.dossier` |
| chart layout geometry | `drawings: [{ layout, varga }]` | `chart.drawings` |
| transit service | `gochar: { instants }` | `chart.gochar` |

### 4.3 The panchanga package

| baseline | SDK |
|---|---|
| daily panchanga | `ctx.almanac.day({ date, place, utcOffsetSeconds })` |
| a run of days | `ctx.almanac.of({ from, to, place, utcOffsetSeconds })` |
| next and previous change finders | each limb is a list of spans; a span carries its `whole` bounds, inside the day or not, and the part `inside` it |
| muhurta finder, window assembler, blackouts | `muhurta: {...}` on an almanac request, answered as `Almanac.muhurta`, with the closed days and the reasons |
| event rules | `muhurta.rules`: a shipped set by name (`'RAMAN_MARRIAGE'`, `'BASELINE_MARRIAGE'`, …) or `ActivityRules` spelt out |
| Pancha Pakshi | `ctx.almanac.pakshi(request)` |

### 4.4 The predictive package

| baseline | request field | result |
|---|---|---|
| gochar overlays, transit aspects, phala with vedha | `gochar: { instants }` | `chart.gochar` |
| Sade Sati | `sadeSati: { from, to }` | `chart.sadeSati` |
| transit strength by Ashtakavarga | `gochar: { instants, ashtakavarga: true }` | `chart.gochar` |
| transit calendar | `hits: {...}` | `chart.hits` |
| Tajika annual chart | `varsha: {...}` | `chart.praveshas`, `chart.sahams` |

### 4.5 The applied package

| baseline | SDK |
|---|---|
| KP | `kp: {...}` on a chart request, on the `kp-default` profile; `chart.kp` |
| milan, Ashta Koot, extended checks, Mangal match | `matching: { partner, partnerRole }`; `chart.matching`, `chart.porutham`, `chart.kuja`, `chart.marriageDoshas` |
| quick milan, akshar resolution | `ctx.matching.naam(bride, groom)` |
| rectification | `rectification: {...}`; `chart.rectification` |
| prashna | `prashna: {...}`; `chart.prashna` |
| Lal Kitab | `lalkitab: {...}`; `chart.lalkitab` |
| numerology | `ctx.numerology.profile(name, date, rules)` |
| remedies | `remedies: {...}`; `chart.remedies` |
| research: batch, statistics | `ctx.chart.foundMany(...)`; `ctx.research.counts(...)` and the rest of `ctx.research` |
| research: synastry, composites | `synastry: {...}`; `chart.synastry`, `chart.synastryComposite` |
| namakarana | the syllables in `chart.avakahada` and `ctx.matching.naam`; in Rust, `teistro::matching::{read_name, name_check, syllables, names_of}`; the rest designed, not built (§10) |

### 4.6 The rashifal and interpretation packages

| baseline | SDK |
|---|---|
| rashifal context and scoring | `ctx.chart.rashifal(request, period)` and `ctx.chart.rashifalMany(requests, period)`; the second argument asks for the baseline's score (§5.3) |
| state tables, yoga and dosha readings | the reading packs in [`packs/`](../../packs/README.md), built from the baseline's records (`teistro-intl migrate readings`, `migrate states`) and loaded with `ctx.intl.loadPack(bytes)` |
| graha-bhava, dasha, day and muhurta composers | `interpret: {...}` on a chart request; `chart.plans`, rendered by `ctx.intl.render` ([`site/content/docs/interpretation.mdx`](../../site/content/docs/interpretation.mdx)) |

## 5. Reproducing the baseline's numbers

### 5.1 The `conformance-baseline` profile

`conformance-baseline` sets every knob the conformance corpus measures
to the value the baseline engine applies by default
([`crates/core/src/settings/profiles.rs`](../../crates/core/src/settings/profiles.rs),
`conformance_baseline`). It exists so that the conformance corpus's
charts reproduce. It is also the closest the SDK comes to the baseline's
answers:

```js
const same = new Context({
  profile: 'conformance-baseline',
  ephemeris: [teimeris({ dataDir: './ephe' })],
});
```

Its knobs, each with the crux or registry entry that measured it:

| knob | value | evidence |
|---|---|---|
| `frame.centre` | `TOPOCENTRIC` | the baseline's `useTopocentric` |
| `frame.ayanamsha_basis` | `TRUE` | registry entry 16: the baseline applies the nutated ayanamsha |
| `houses.chalit_system` | `VEHLOW` | registry entry 14: documented as Sripati, measured as Vehlow |
| `day.sunrise` | `UPPER_LIMB_REFRACTION` | the baseline's `apparent-refraction` |
| `day.polar_day_policy` | `NEAREST_EVENT` | registry entry 3 |
| `time.dst_gap`, `time.dst_overlap` | `ERROR`, `EARLIER` | the baseline throws in a gap and takes the earlier of a fold |
| `calendars.civil_calendar` | `BIKRAM_SAMBAT` | the baseline's civil calendar |
| `panchanga.moon_events` | `CIVIL_DAY` | registry entry 18 |
| `panchanga.panchaka_start` | `DHANISHTHA` | C158 |
| `strength.ekadhipatya`, `strength.shodhana` | `EMPTY_TO_ZERO`, `SARVA` | C59, C60 |
| `strength.vimshopaka` | `SAPTAVARGAJA_VIRUPAS` | C63 |
| the Shadbala knobs (`saptavargaja`, `nathonnatha`, `pre_dawn_night`, `sun_ayana`, `moon_paksha`, `luminary_cheshta`, `kranti`, `kaala_lords`, `dig`, `drik`, `naisargika`, `required_rupas`, `drekkana`, `benefics`, `cheshta`, `yuddha`, `ishta_kashta`) | the baseline's reading of each | C64 to C72, C76 |
| the Bhava bala knobs (`bhava_division`, `bhava_dig`, `bhava_drishti`, `bhava_special_rules`) | the baseline's reading of each | C73 to C75 |
| `dasha.ashtottari_grouping` | `THREE_EACH` | C5 |
| `dasha.dual_lord`, `dasha.rashi_start` | `KENDRA`, `LAGNA` | C51, C53 |
| `kp.ruling_count`, `kp.day_lord_day` | `WITH_SUBS`, `CIVIL` | C150, C151: seven ruling planets, the clock's weekday |
| `jaimini.chara_karakas` | `EIGHT` | the recorded `chara_karaka: 8` ([`03-design/jaimini-measured.md`](../03-design/jaimini-measured.md)) |
| `jaimini.eight_karaka_order` | `PITRIKARAKA_LAST` | C101: the least of eight is named the Pitrikaraka |
| `gochar.transit_centre` | `GEOCENTRIC` | C778: a transit from the Earth's centre under a topocentric birth |

The registry entries are in
[`05-testing/01-golden-vectors.md`](../05-testing/01-golden-vectors.md),
"The baseline's conventions, registered rather than copied". The cruxes
are in
[`01-research/feature-universe/19-verification-cruxes.md`](../01-research/feature-universe/19-verification-cruxes.md).
The full knob list, with every value, is
[`03-design/settings-and-profiles.md`](../03-design/settings-and-profiles.md) §3.1.

How close it comes is measured, not claimed.
[`05-testing/CONFORMANCE.md`](../05-testing/CONFORMANCE.md) scores the
SDK against the baseline's recorded answers, section by section, under
this profile. [`05-testing/ab-measured.md`](../05-testing/ab-measured.md)
compares the two engines answer for answer; its `same` column is this
profile. Read those pages for the current figures.

### 5.2 Patching what the profile does not set

A settings patch is laid over the profile. Group and knob names are the
settings document's own; values are their keys:

```js title="bindings/node/bench/ab.mjs"
// KP on the baseline's frame: its mean node, topocentric and nutated.
const kp = new Context({
  profile: 'kp-default',
  ephemeris,
  settings: {
    frame: { node: 'MEAN', centre: 'TOPOCENTRIC', ayanamsha_basis: 'TRUE' },
    day: { polar_day_policy: 'NEAREST_EVENT' },
  },
});
```

A catalogued ayanamsha in a patch is written
`{ kind: 'CATALOGUED', id: 'RAMAN' }`.

`conformance-baseline` carries the baseline's KP and Jaimini readings
(§5.1), but a context opened on another profile does not. A KP context
on `kp-default`, as above, reads Krishnamurti's own rules; patch these
two knobs when you want the baseline's ruling planets:

```js
settings: { kp: { ruling_count: 'WITH_SUBS', day_lord_day: 'CIVIL' } },
```

### 5.3 The baseline's readings, asked for per request

Some baseline behaviour is a reading of a rule with no text behind it.
The SDK keeps each one as a named value, marked unsourced, and computes
it only when a request asks
([`08-decisions/adr-0018-evidence-ranks-and-mark-and-continue.md`](../08-decisions/adr-0018-evidence-ranks-and-mark-and-continue.md)).

| area | ask for | design page |
|---|---|---|
| yogas and doshas | `rules: { shipped: ['YOGAS', 'DOSHAS'], readings: 'RECORDING_ENGINE' }` | [`rules-engine.md`](../03-design/rules-engine.md) |
| muhurta ranking | `muhurta: { rules: 'BASELINE_MARRIAGE', ranking: 'BASELINE' }` | [`muhurta.md`](../03-design/muhurta.md) §4.5 |
| rashifal score | `ctx.chart.rashifal(request, 'DAILY' \| 'WEEKLY' \| 'MONTHLY' \| 'YEARLY')`; the answer's `baseline` holds each sign's score | [`rashifal.md`](../03-design/rashifal.md) |
| rashifal snapshot | `snapshot: { at: 'CLOCK', hour: 6, minute: 0 }` on the request, the baseline's 06:00; the SDK reads sunrise by default (C358) | [`rashifal.md`](../03-design/rashifal.md) |
| prashna | `prashna: { rules: { timing: 'BASELINE', mook: 'BASELINE', score: 'BASELINE' } }` | [`prashna.md`](../03-design/prashna.md) |
| numerology | `rules`: `{ masters: 'ELEVEN_TWENTY_TWO_THIRTY_THREE', nameReduction: 'WHOLE', chaldeanCompound: 'LETTER_TOTAL', birthReduction: 'DIGIT_SUM', nonLatin: 'SKIP' }`; the answer then carries `baseline` | [`numerology.md`](../03-design/numerology.md) |
| functional nature for remedies | `remedies: { rules: { functional: { scheme: 'BASELINE' } } }` | [`remedies.md`](../03-design/remedies.md) |
| rectification | `rectification: { baseline: { uncertaintyMinutes: 60, sex: 'MALE', events } }`; `chart.rectification.baseline` | [`rectification.md`](../03-design/rectification.md) |
| Kuja dosha houses | `matching: { …, kuja: { houses: 'WITH_SECOND' } }` | [`matching.md`](../03-design/matching.md), "The Kuja dosha, decided" |
| Stree-deergha | `matching: { …, porutham: { deerghaBeyond: 'TENTH_OR_SAME_STAR' } }`: beyond the 10th, the same star agreeing (C779) | [`matching.md`](../03-design/matching.md) |
| KP under any ayanamsha | `kp: { anyAyanamsha: true }`; the SDK refuses KP under a non-KP ayanamsha otherwise (C157), where the baseline warns | [`kp.md`](../03-design/kp.md) |

In Rust, `PrashnaRules::baseline()`, `NumerologyRules::baseline()` and
`FunctionalRules::baseline()` set every baseline value of their record at
once. Dart has `NumerologyRules.baseline()`. Node, Python and Java spell
each value, as in the table.

### 5.4 Transits

The baseline casts a natal chart from the birthplace and a transit from
the Earth's centre. `conformance-baseline` does the same: its
`gochar.transit_centre` is `GEOCENTRIC`, so one context answers the
natal requests topocentric and the transit requests (`gochar`,
`sadeSati`, `hits`) geocentric (crux C778). On another profile, set it
yourself:

```js
settings: { gochar: { transit_centre: 'GEOCENTRIC' } },
```

## 6. Where the SDK's default differs on purpose

The default profile is `parashari-classical`, the texts as read
(ADR-0024,
[`08-decisions/adr-0024-the-default-profile.md`](../08-decisions/adr-0024-the-default-profile.md)).
It is not the baseline's default. Each difference below is a choice, and
each has a knob that takes the baseline's value.

| knob | the baseline | `parashari-classical` | why |
|---|---|---|---|
| `frame.centre` | topocentric | `GEOCENTRIC` | the texts and PyJHora cast from the Earth's centre; the Moon moves up to about a degree between the two, which moves dasha starts |
| `frame.ayanamsha_basis` | nutated | `MEAN` | the Lahiri definition is the mean value (registry entry 16) |
| `houses.chalit_system` | Vehlow | `SRIPATI` | BPHS; the baseline documents Sripati and computes Vehlow (registry entry 14, [`chart-bhava-chalit.md`](../03-design/chart-bhava-chalit.md)) |
| `day.ghati_reckoning` | records both reckonings | `PROPORTIONAL` (`CIVIL` in `conformance-baseline`) | a context answers the one reckoning its settings name; open a second context for the other ([`settings-and-profiles.md`](../03-design/settings-and-profiles.md) §3.2) |
| `day.polar_day_policy` | synthesised from the civil day | `UNDEFINED` | a day with no sunrise is refused rather than invented |
| `state.combustion_orbs` | deep orbs | `SURYA_SIDDHANTA` | the text gives no deeper orb, so no body is deeply combust (registry entry 23) |
| the Shadbala and Bhava bala knobs | the baseline's readings | Sripati's, as Raman works them | Raman's Example 56 reproduces under Sripati's readings and not under the baseline's (C64 to C75, [`shadbala-measured.md`](../03-design/shadbala-measured.md)) |
| `strength.ekadhipatya`, `strength.shodhana`, `strength.vimshopaka` | the baseline's reductions | `BPHS`, `EACH_GRAHA`, `BPHS` | BPHS chs. 67 to 69 and ch. 7 (C59, C60, C63) |
| `dasha.ashtottari_grouping` | three each | `FOUR_AND_THREE` | BPHS ch. 46 vv. 17 to 20 (C5) |
| `dasha.dual_lord`, `dasha.rashi_start` | kendra, lagna | `BPHS`, `STRONGER` | BPHS ch. 46 (C51, C53) |
| `panchanga.moon_events` | from civil midnight | `WINDOW` | the Moon's events over the same sunrise-to-sunrise window as the limbs (registry entry 18) |
| `panchanga.panchaka_start` | Dhanishtha | `AQUARIUS` | C158 |
| `calendars.civil_calendar` | Bikram Sambat | `GREGORIAN` | no country's practice in the default (ADR-0024) |
| `kp.ruling_count`, `kp.day_lord_day` | seven, civil weekday | `FIVE`, `SUNRISE` | KP Reader VI (C150, C151) |
| yoga readings | the baseline's | the texts' | Kemadruma's cancellations, the Nabhasa shapes, Gaja Kesari and Chamara as their texts' clauses read ([`yogas-measured.md`](../03-design/yogas-measured.md)) |
| muhurta ranking | weighted score | `TEXTS` | the sources' own order, fewest doshas then most favourable clauses (C162) |

`nepali-default` is the profile for a Nepali product. It keeps the
baseline's topocentric centre, Vehlow chalit and Bikram Sambat. It
differs from the baseline in its sunrise: `CENTRE_REFRACTION`, the
convention nearest Nepal's printed daily sunrise (C39). The baseline's
sunrise is `UPPER_LIMB_REFRACTION`.

## 7. Differences no setting removes

These are differences of method, or defects in the baseline the SDK does
not copy. A knob cannot bring them back. Each is registered with the
evidence.

| difference | the baseline | the SDK | where |
|---|---|---|---|
| local mean time | rounded to the whole minute | to the second | registry entry 2 |
| sunrise refraction | the library's standard atmosphere | the almanac's 34′ with semidiameter and parallax; seconds apart | registry entry 12, C34 |
| a day placed wrongly | three recorded births read in the previous Hindu day | the civil date's sunrise | C35 |
| the night's length | 24 h less the daylight | the real interval to the next sunrise | registry entry 15 |
| Brahma muhurta | sized from the following night | from the night it is in | registry entry 17 |
| tithi number | 1 to 30 | the paksha and 1 to 15 | registry entry 19 |
| vargottama | grahas only | anything asked about | registry entry 20 |
| a deeply debilitated graha's wakefulness | dreaming | asleep | registry entry 21 |
| the lagna's dignity | placeholder values | none: the lagna is not a graha | registry entry 22 |
| special lagnas | a clock up to 1.6 minutes from its own ishtakaal | the chart's ishtakaal | registry entry 25 |
| Placidus at high latitude | a silent fallback, flagged by a heuristic | an outcome that says which system answered; `houses.polar_policy` decides | registry entries 5 and 26 |
| limb boundaries | found to 0.001° (up to 7.6 s) | converged in time | [`01-pain-points.md`](../01-research/baseline-engine/01-pain-points.md) |
| the years an ephemeris covers | an analytic fallback past the files | refused at the data's edge | [`ab-measured.md`](../05-testing/ab-measured.md), "Refused" |
| Ashta Koota | its own Vashya, Gana, Yoni and Bhakoot tables | the sources' tables, with knobs for the printed variants (`matching.rules`, a `KootaRules`) | [`ab-measured.md`](../05-testing/ab-measured.md), `milan` |
| Tajika year lord and sahams | its own | Charak's worked year | [`ab-measured.md`](../05-testing/ab-measured.md), `varsha` |
| era numbers in the application layer | fixed offsets from the year | computed with sankranti-aware rollover | [`01-pain-points.md`](../01-research/baseline-engine/01-pain-points.md) |
| threads | crashes in worker threads (global ephemeris state) | one context per thread, no shared state | [`ab-measured.md`](../05-testing/ab-measured.md), "Not measured, and why" |

The A/B page names the dominant cause of every remaining difference,
request by request, and says which side a reference neither engine
computed agrees with ([`ab-measured.md`](../05-testing/ab-measured.md),
"Agreement and accuracy"). Where a rank-1 or rank-2 reference exists
(Raman for Shadbala, Charak for Tajika, *Kalaprakasika* for milan), read
that column before deciding which answer your users should see.

## 8. Stored charts

### 8.1 What the baseline stored

The baseline persists a settings snapshot, a content hash over it and a
date-free dossier from its serialiser, with a gochar sidecar for
transits. None of those hashes can be reproduced by the SDK. The SDK's
settings hash has another recipe, over the full resolved settings
([`settings-and-profiles.md`](../03-design/settings-and-profiles.md) §3.4).
The SDK's dossier is another text format
([`03-design/dossier.md`](../03-design/dossier.md)).

### 8.2 What to store with the SDK

Store the input and the provenance, and recompute the answer when you
need it.

1. **The input.** The instant, the place and the clock's offset. Keep
   the zone resolution beside them: `ctx.time.resolve` answers the rule
   it used and `tzdbVersion`.
2. **The settings.** `ctx.profile`, `ctx.settingsJson` (the canonical
   settings document) and `ctx.settingsHash`. A profile version bump
   that changes a default changes the hash, so a changed default is
   visible.
3. **The provenance.** `chart.provenance` carries `contentHash`,
   `inputHash`, `settingsHash`, `sdkVersion`, `calculationVersion` and
   the provider. `chart.batch.provenanceJson` is the batch's envelope as
   canonical JSON, the same bytes in every binding.
4. **The text, if you keep text.** `dossier: { preset: 'FULL' }` writes
   a date-free dossier. Give it `at` for a dated one.
5. **The gochar sidecar, in Rust, if you keep text and read transits
   later.** The baseline's `gocharInputs` column becomes
   `GocharSidecar::cut(&read.value, &read.provenance)?.to_text()`: the
   chart's foundation and panchanga, tied to the chart's content hash and
   settings hash. On a later day `GocharSidecar::read` takes it back in
   any JSON spelling (a `jsonb` column reorders keys), `check_chart`
   refuses it beside a chart recomputed since, and `sdk.chart().natal`
   opens it for `gochar`, `sade_sati`, `hits` and `sarvatobhadra` under
   the settings it was cut with
   ([`03-design/gochar-sidecar.md`](../03-design/gochar-sidecar.md)).
   The baseline's sidecar cannot be read: it holds signs and tables
   where the SDK's holds the foundation, so cut a new one when the chart
   is recomputed (§8.3).

To decide whether a stored answer is still valid, compare the answer key
the serial page defines: the SHA-256 of the canonical JSON of the
provenance's versions, settings hash, input hash, provider and packs
([`03-design/serial-and-the-envelope.md`](../03-design/serial-and-the-envelope.md),
"The answer key"). Rust has `Provenance::answer_key()`. A binding builds
the same object from `chart.provenance` and hashes it.

In Rust, a value's canonical bytes are `teistro_serial::to_hash_form`,
read back by `canonical::from_hash_form`. The chart document's JSON
Schema is `teistro::schema::DOCUMENT`
([`03-design/document-schema.md`](../03-design/document-schema.md)). No
binding hands out the chart document's JSON yet; each decodes the
result blob (§10).

### 8.3 Moving the stored charts

For each stored chart:

1. Map its stored settings snapshot to a profile and a patch (§5).
2. Recompute it on the SDK from its stored input.
3. Store the SDK's settings hash and content hash beside the baseline's.
4. Compare the fields your application shows. A difference is either a
   setting you have not mapped or a row of §7.

The conformance harness does the same for the corpus: it maps each
recorded profile to an SDK profile, asserts the SDK's own hash and keeps
the baseline's beside it
([`05-testing/01-golden-vectors.md`](../05-testing/01-golden-vectors.md),
"Design choices worth keeping").

## 9. A migration plan

1. **Inventory your calls.** List every baseline service you call and
   the settings you persist. Find each in §4 and §5.
2. **Choose a profile.** `conformance-baseline` for the closest
   numbers; `nepali-default` or `parashari-classical` with a patch for a
   product. Write the choice down with its patch: it is part of your
   stored provenance.
3. **Run both side by side.** Open an SDK context next to the baseline
   in the same process. Ask both for the same births. Compare the fields
   you show, not whole objects: the SDK's results are keys and spans
   where the baseline's are names and numbers.
4. **Use the A/B harness.** `bindings/node/bench/ab.mjs` runs both
   engines interleaved on the golden corpus's 55 births. It compares
   their answers field by field and holds each to a third reference. It
   loads the baseline through an adapter module you write to the
   contract in [`03-design/ab-baseline.md`](../03-design/ab-baseline.md)
   §1 (`version`, `sun`, one export a request, `facts`, `conventions`),
   named by `TEISTRO_AB_BASELINE`. `cargo xtask measure-ab` runs it and
   writes the record; `cargo xtask check-ab` renders and gates it.
5. **Compare two SDK reckonings.** `ctx.almanac.disagreements(other,
   request)` lists the days on which two contexts' limbs or festivals
   differ, and why. Use it to see what moving from
   `conformance-baseline` to your chosen profile changes:

   ```js
   const { days } = chosen.almanac.disagreements(same, {
     from: date('calendar.GREGORIAN', 2025, 11, 1),
     to: date('calendar.GREGORIAN', 2025, 11, 30),
     place: { latitude: 27.7172, longitude: 85.324, altitude: 1400 },
     utcOffsetSeconds: 20700,
   });
   ```

6. **Move the stored charts** (§8.3).
7. **Switch.** Route reads to the SDK. Keep the baseline's stored hashes
   until no reader needs them.

## 10. What is not available yet

- **The compatibility shim is not built.** The roadmap lists it as a
  Phase 9 deliverable beside this guide. There is no module that accepts
  the baseline's calls and settings and forwards them to the SDK. You
  port each call.
- **Namakarana.** The phonetic matcher, the name validation and the
  offline name corpus have no equivalent in a binding yet. In Rust,
  `read_name` reads a Latin name under a declared scheme or every
  `CASUAL` way, `name_check` checks a name with its clauses,
  `syllables` gives a pada's, a star's or a sign's syllables, and
  `NameBook::load` and `suggest` search a book you load
  (`namakarana.md` §10). The design is
  [`03-design/namakarana.md`](../03-design/namakarana.md): a `naming`
  area that reads a Latin name under a declared scheme or reports every
  casual reading rather than picking one, checks a name as facts and
  clauses rather than a score, and searches a name book the consumer
  loads. No corpus ships, so the baseline's corpus migrates as your own
  book (§3 of the page). Until it is built, `chart.avakahada` gives the
  Moon's syllable and `ctx.matching.naam` reads a name's star.
- **The gochar sidecar in a binding.** Rust has it (§8.2); Node,
  Python, Dart and Java do not. Ask for the transits on the chart
  request (its `gochar`, `sadeSati` and `dossier` records) from the
  stored natal input, which founds the natal chart and reads its
  transits in one call.
- **The chart document in a binding.** Rust has the document and its
  schema; Node, Python, Dart and Java decode the result blob and do not
  hand out the document's JSON
  ([`document-schema.md`](../03-design/document-schema.md) §8).
- **A zone from coordinates.** The baseline infers the zone from the
  place. The SDK's geo port is optional and not built; give an IANA zone
  or local mean time
  ([`03-design/time-and-timezone.md`](../03-design/time-and-timezone.md) §2).
- **Byte-identical narrative text.** The composers say a chart through
  plans of message keys. Their text is not the baseline's composers'
  text, and no gate compares the two
  ([`interpret-composers.md`](../03-design/interpret-composers.md) §2).
- **The baseline's koota tables, Stree-deergha count and Tajika year
  lord** have no knob (§7).
