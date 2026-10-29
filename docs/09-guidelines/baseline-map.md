# The baseline engine, mapped

Status: `generated` by `cargo xtask compat` from `bindings/node/compat/surface.json` and `teistro_core::settings::recorded::ROWS`, gated by `check-compat`. Do not edit. The design is [`03-design/compat-shim.md`](../03-design/compat-shim.md); the move itself, in prose, is the [migration guide](baseline-migration.md).

Every baseline entry point an application calls, with the `@teistro/compat` function that takes it and the SDK call that answers it (§2), and every stored settings field with what it becomes (§3). A shim function answers exactly what its SDK call answers, so dropping the shim is replacing each call by the one beside it.

## 1. Counts

| | count |
|---|---|
| shim rows | 62 |
| forwarded: one SDK call, its inputs mapped | 46 |
| composed: several SDK calls, or a read across one answer | 11 |
| refused: throws `CompatRefused` at once, typed `never` | 5 |
| stored settings rows | 36 |
| … becoming read | 4 |
| … becoming knobs | 15 |
| … becoming a note | 8 |
| … becoming knobs and an ephemeris | 1 |
| … becoming refused | 3 |
| … becoming the locale | 4 |
| … becoming the script | 1 |
| ayanamsha keys the application stores | 48 |
| dasha systems' stored spellings | 18 |

The baseline engine's own entry points, counted from outside by the A/B's adapter (`cargo xtask measure-ab`); the gate fails on any with no row:

- `macos-aarch64`: not counted, since its adapter exports no `surface` (§6.4 of the design)

## 2. The call surface

Every function takes the stored birth record and the stored snapshot, `(birth, snapshot, options?)`, unless its SDK call reads no birth. `ctx` is the context the snapshot opens; `found` is `ctx.chart.found`.

### 2.1 The core package

Design §3.1.

| baseline entry point | shim | SDK | kind |
|---|---|---|---|
| the astronomical backend, a position at an instant | `positions` | ctx.positions({ instants, bodies, frame }) | forward |
| the engine's own operations | `engine` | ctx.engine.call(name, args) on a Teimeris context | forward |
| the ayanamsha at an instant | `ayanamsha` | ctx.chart.found(...): its ayanamsha and offset | compose |
| the sunrise service | `sunrise` | ctx.almanac.day(...): its sunrise, sunset and next sunrise | compose |
| the timezone resolver: a civil time in a zone, LMT, DST rules | `resolve` | ctx.time.resolve(civil, zone) | forward |
| the timezone resolver: a zone from coordinates | `zoneAt` | none: no geo port | refuse: Give the zone as an IANA name or local mean time on the birth record; the SDK has no geo port. ([why](../09-guidelines/baseline-migration.md#10-what-is-not-available-yet)) |
| ghati-pala, both reckonings | `ghatiPala` | chart.timing on the natal context and on one with the other day.ghati_reckoning | compose |
| the Bikram Sambat calendar: conversion, month lengths, leap tests | `bs` | ctx.calendar: convert, dateOf, fixedOf, monthLength, isLeap | forward |
| the calendar service's era numbers | `eras` | ctx.almanac.of(...) with years and nepalSambat | compose |
| the entity registry, a name by type and key | `entity` | ctx.intl.entity(key) | forward |
| numeral, date, degree and clock formatting | `format` | none: formatting is the application's | refuse: Format the SDK's numbers and instants in the application, with Intl.NumberFormat and Intl.DateTimeFormat (a nu extension gives Devanagari digits). ([why](../09-guidelines/baseline-migration.md#2-what-is-different-in-kind)) |

### 2.2 The chart package

Design §3.2.

| baseline entry point | shim | SDK | kind |
|---|---|---|---|
| the foundation | `chart` | ctx.chart.found with every varga, aspects, points, houses, state, jaimini and the Vimshottari | forward |
| the whole natal chart | `full` | ctx.chart.found with every section, the 18 dashas, every shipped rule set, avakahada and the longevity | forward |
| vargas and vargottama | `vargas` | found with vargas | forward |
| yogas, doshas | `yogas` | found with rules.shipped | forward |
| Shadbala, Bhava bala, Ashtakavarga | `strength` | found with shadbala, bhavaBala, ashtakavarga | forward |
| one dasha system, all 18 | `dashas` | found with the 18 dashas, depths from the snapshot | forward |
| houses, Bhava-Chalit | `houses` | found with houses: true | forward |
| avasthas, dignities, planetary war | `states` | found with state: true | forward |
| drishti | `aspects` | found with aspects: true | forward |
| upagrahas, special lagnas | `points` | found with points: true | forward |
| the Jaimini slice | `jaimini` | found with jaimini: true | forward |
| Ayurdaya, Maraka, longevity windows | `longevity` | found with rules: { longevity: true, ayurdaya } | forward |
| birth timing: ishtakaal, bhayat, bhabhoga | `timing` | found with moment: true | forward |
| avakhada | `avakahada` | found with avakahada: true | forward |
| the calendar conversion of the birth | `birthCalendar` | ctx.calendar.convert on the birth's civil date | forward |
| the serialiser with presets | `dossier` | found with dossier | forward |
| the gochar sidecar | `sidecar` | none in Node: the sidecar is Rust only | refuse: Ask for the transits on the chart request instead: gochar, sadeSati or transitCalendar on the stored birth record. ([why](../09-guidelines/baseline-migration.md#10-what-is-not-available-yet)) |
| chart layout geometry | `drawing` | found with drawings | forward |
| the dossier, day, dasha, graha-bhava, tara chakra, yoga timing and namakarana note composers | `narrative` | found with interpret, each plan rendered by ctx.intl.render | compose |

### 2.3 The panchanga package

Design §3.3.

| baseline entry point | shim | SDK | kind |
|---|---|---|---|
| the daily panchanga; choghadiya, hora, Rahu kaal and the kaalas, muhurta yogas | `day` | ctx.almanac.day | forward |
| a run of days; the tithi calendar | `month` | ctx.almanac.of | forward |
| next and previous limb changes | `changes` | ctx.almanac.day: each limb span's whole bounds | compose |
| the muhurta finder, windows, blackouts, natal tara and chandra bala | `muhurta` | ctx.almanac.of with muhurta, ranking BASELINE, on the muhurta frame's context | forward |
| Sarvatobhadra | `sarvatobhadra` | found with sarvatobhadra | forward |
| eclipses | `eclipses` | ctx.almanac.of with eclipses: true | forward |
| ingresses | `ingresses` | ctx.almanac.of: each day's sankranti | compose |
| lunations | `lunations` | ctx.almanac.of: the ends of its tithi spans at 15 and 30 | compose |
| Pancha Pakshi | `pakshi` | ctx.almanac.pakshi | forward |

### 2.4 The predictive package

Design §3.4.

| baseline entry point | shim | SDK | kind |
|---|---|---|---|
| gochar overlays, transit aspects, phala with vedha, transit strength | `gochar` | found with ashtakavarga and gochar | forward |
| Sade Sati | `sadeSati` | found with sadeSati | forward |
| the transit calendar | `transitCalendar` | found with hits | forward |
| the Tajika annual chart, muntha, sahams, Mudda dasha | `varsha` | found with varsha | forward |
| solar, lunar and planetary returns | `returns` | found with hits: returnsRequest(from, to, grahas) | compose |
| secondary and tertiary progressions, solar arc directions | `progressions` | found with progressions | forward |

### 2.5 The applied package

Design §3.5.

| baseline entry point | shim | SDK | kind |
|---|---|---|---|
| KP: sub-lords, significators, ruling planets, Placidus with fallback | `kp` | kp on the KP context, or on the natal one with anyAyanamsha | forward |
| milan: Ashta Koota, extended checks, Mangal match, marriage doshas | `milan` | found with matching | forward |
| quick milan, akshar resolution | `naam` | ctx.matching.naam(bride, groom) | forward |
| rectification | `rectify` | found with rectification.baseline, the record's gender and time accuracy carried in | forward |
| prashna | `prashna` | found with prashna under the baseline's timing, mook and score | forward |
| Lal Kitab | `lalkitab` | found with lalkitab | forward |
| numerology | `numerology` | ctx.numerology.profile(name, date, 'BASELINE') | forward |
| remedies | `remedies` | found with remedies under the baseline's functional scheme | forward |
| remedies: gemstone safety | `gemstoneSafety` | none: not built | refuse: The SDK has no gemstone safety reading; keep the application's own until one is built. ([why](../01-research/baseline-engine/02-parity-requirements.md#state-measured-2026-10-27)) |
| namakarana: validation | `nameCheck` | found with avakahada, its star and pada given to ctx.naming.check | compose |
| namakarana: suggestions, the offline corpus, the language-model path | `nameSuggestions` | none: no corpus ships | refuse: Load the application's own name book with ctx.naming.load and search it with ctx.naming.suggest. ([why](../03-design/namakarana.md#3-the-name-book-what-ships-and-what-the-consumer-brings)) |
| research: batch | `batch` | ctx.chart.foundMany | forward |
| research: synastry, composites | `synastry` | found with synastry | forward |
| research: statistics | `research` | ctx.research.counts, compare, expected | forward |

### 2.6 The rashifal and interpretation packages

Design §3.6.

| baseline entry point | shim | SDK | kind |
|---|---|---|---|
| the dossier, day, dasha, graha-bhava, tara chakra, yoga timing and namakarana note composers | `narrative` | found with interpret, each plan rendered by ctx.intl.render | compose |
| rashifal context and score, one sign or all twelve, per period | `rashifal` | ctx.chart.rashifal(request, period), read at six on the clock | forward |
| state tables, yoga and dosha readings | `reading` | ctx.intl.render(key, params) | forward |
| the milan resolver and verdict composer | `milanVerdict` | found with matching, the doshas and interpret, rendered | compose |

## 3. The stored settings

Each stored field and value, folded in this order over `conformance-baseline`, a later row winning where two write one knob. A field or value not here is refused, never skipped. Every language binding reads a snapshot through `recordedSettings` (`recorded_settings` in Python, `recorded` in Rust).

| stored field | stored value | becomes | what |
|---|---|---|---|
| `ayanamsaId` | the number a catalogued ayanamsha's `swiss_mode` is, or the application's key for it (`AYANAMSHA_KEYS`) | read | read from the value; `ayanamsaId: "raman"` sets `{"frame":{"ayanamsha":{"id":"RAMAN"}}}` |
| `zodiacMode` | `sidereal` | knobs | `{"frame":{"zodiac":"SIDEREAL"}}` |
| `zodiacMode` | `tropical` | a note | the recording engine casts the same sidereal chart under it: the corpus's four tropical variants equal their default charts field for field |
| `nodeType` | `mean` | knobs | `{"frame":{"node":"MEAN"}}` |
| `nodeType` | `true` | knobs | `{"frame":{"node":"TRUE"}}` |
| `useTopocentric` | `true` | knobs | `{"frame":{"centre":"TOPOCENTRIC"}}` |
| `useTopocentric` | `false` | knobs | `{"frame":{"centre":"GEOCENTRIC"}}` |
| `houseSystem` | the 22 names, a house system's key lower-cased with hyphens | read | read from the value; `houseSystem: "placidus"` sets `{"houses":{"placement_system":"PLACIDUS","polar_policy":"FALLBACK_PORPHYRY"}}` |
| `siddhanta` | `drik` | knobs | `{"frame":{"siddhanta":{"kind":"DRIK"}}}` |
| `siddhanta` | `surya` | knobs and an ephemeris | `{"frame":{"ayanamsha":{"kind":"CATALOGUED","id":"SURYASIDDHANTA"},"centre":"GEOCENTRIC","siddhanta":{"kind":"SURYA","bija":{"kind":"NONE"},"sunrise":"MODERN","planets":{"kind":"TEXT"}}}}`, with `SURYA_SIDDHANTA` at the front of the natal chain |
| `suryaBija` | `false` | a note | the text's own revolutions, which `siddhanta: surya` already asks for |
| `suryaBija` | `true` | refused | C777: the baseline's bija re-anchors the mean node alone, and no named set of revolutions does that |
| `sunriseMode` | `apparent-refraction` | knobs | `{"day":{"sunrise":{"kind":"NAMED","which":"UPPER_LIMB_REFRACTION"}}}` |
| `sunriseMode` | `geometric` | knobs | `{"day":{"sunrise":{"kind":"NAMED","which":"CENTRE_NO_REFRACTION"}}}` |
| `sunriseMode` | `upper-limb` | knobs | `{"day":{"sunrise":{"kind":"CUSTOM","altitude_deg":-0.5667}}}` |
| `sunriseMode` | `lower-limb` | knobs | `{"day":{"sunrise":{"kind":"CUSTOM","altitude_deg":-1.1}}}` |
| `dashaBalanceMethod` | `spatial` | knobs | `{"dasha":{"balance":"SPATIAL"}}` |
| `dashaBalanceMethod` | `temporal` | knobs | `{"dasha":{"balance":"TEMPORAL"}}` |
| `dashaMaxLevel` | 1 to 5, each of the 18 systems | read | read from the value; `dashaMaxLevel: 2` sets `{"dasha":{"depth":{"VIMSHOTTARI":2,"ASHTOTTARI":2,"DWADASHOTTARI":2,"PANCHOTTARI":2,"SHATABDIKA":2,"CHATURASHITI_SAMA":2,"DWISAPTATI_SAMA":2,"YOGINI":2,"TRIBHAGI":2,"CHARA":2,"NARAYANA":2,"PADANADHAMSA":2,"TRIKONA":2,"DRIG":2,"SHOOLA":2,"NIRYANA_SHOOLA":2,"MANDOOKA":2,"KALACHAKRA":2}}}` |
| `dashaDepthBySystem.*` | 1 to 5, the system its key names | read | read from the value; `dashaDepthBySystem.yogini: 2` sets `{"dasha":{"depth":{"YOGINI":2}}}` |
| `charaKarakaSystem` | `7` | knobs | `{"jaimini":{"chara_karakas":"SEVEN"}}` |
| `charaKarakaSystem` | `8` | knobs | `{"jaimini":{"chara_karakas":"EIGHT"}}` |
| `ekadhipatyaMethod` | `classical` | knobs | `{"strength":{"ekadhipatya":"EMPTY_TO_ZERO","shodhana":"SARVA"}}` |
| `ekadhipatyaMethod` | `zero` | refused | C60: no text gives it and the baseline's own documentation calls it legacy |
| `ekadhipatyaMethod` | `transfer` | refused | C60: no text gives it and the baseline's own documentation calls it legacy |
| `locale.language` | `ne` | the locale | `ne-Deva-NP` |
| `locale.language` | `en` | the locale | `en-Latn` |
| `locale.language` | `hi` | the locale | `hi-Deva-IN` |
| `locale.language` | `sa` | the locale | `sa-Deva` |
| `locale.script` | the language's locale's script | the script | held to the language's locale's script; any other pairing is refused |
| `locale.numeralSystem` | any | a note | presentation: numbers are the application's to format |
| `locale.calendarSystem` | any | a note | presentation: dates are the application's to format |
| `locale.timeFormat24h` | any | a note | presentation: the clock is the application's to format |
| `engineVersion` | any | a note | kept for the stored chart's comparison (guide §8.3) |
| `swissephVersion` | any | a note | kept for the stored chart's comparison (guide §8.3) |
| `serializerVersion` | any | a note | kept for the stored chart's comparison (guide §8.3) |

### 3.1 The fixed conventions

What a snapshot that stores nothing opens: the settings the baseline fixes whatever a snapshot says. Every translation lays its rows over these.

| context | profile | patch |
|---|---|---|
| `natal` | `conformance-baseline` | `{}` |
| `kp` | `kp-default` | `{"frame":{"ayanamsha_basis":"TRUE","node":"MEAN","centre":"TOPOCENTRIC"},"day":{"polar_day_policy":"NEAREST_EVENT"},"kp":{"ruling_count":"WITH_SUBS","day_lord_day":"CIVIL"}}` |
| `muhurta` | `conformance-baseline` | `{"frame":{"node":"TRUE","centre":"GEOCENTRIC"}}` |

### 3.2 The ayanamsha keys

The application stores `ayanamsaId` as a catalogued member's `swiss_mode` number or as one of these keys, matched as stored.

| key | member |
|---|---|
| `lahiri` | `LAHIRI` |
| `raman` | `RAMAN` |
| `krishnamurti` | `KRISHNAMURTI` |
| `yukteshwar` | `YUKTESHWAR` |
| `jn-bhasin` | `JN_BHASIN` |
| `lahiri-1940` | `LAHIRI_1940` |
| `lahiri-vp285` | `LAHIRI_VP285` |
| `krishnamurti-vp291` | `KRISHNAMURTI_VP291` |
| `lahiri-icrc` | `LAHIRI_ICRC` |
| `surya-siddhanta` | `SURYASIDDHANTA` |
| `suryasiddhanta` | `SURYASIDDHANTA` |
| `surya-siddhanta-msun` | `SURYASIDDHANTA_MSUN` |
| `ss-citra` | `SS_CITRA` |
| `ss-revati` | `SS_REVATI` |
| `aryabhata` | `ARYABHATA` |
| `aryabhata-msun` | `ARYABHATA_MSUN` |
| `aryabhata-522` | `ARYABHATA_522` |
| `true-chitra` | `TRUE_CHITRA` |
| `true-revati` | `TRUE_REVATI` |
| `true-pushya` | `TRUE_PUSHYA` |
| `true-mula` | `TRUE_MULA` |
| `true-sheoran` | `TRUE_SHEORAN` |
| `aldebaran-15tau` | `ALDEBARAN_15TAU` |
| `fagan-bradley` | `FAGAN_BRADLEY` |
| `de-luce` | `DELUCE` |
| `ushashashi` | `USHASHASHI` |
| `djwhal-khul` | `DJWHAL_KHUL` |
| `galcent-0sag` | `GALCENT_0SAG` |
| `galcent-gil-brand` | `GALCENT_RGILBRAND` |
| `galcent-mula-wilhelm` | `GALCENT_MULA_WILHELM` |
| `galcent-cochrane` | `GALCENT_COCHRANE` |
| `galequ-iau1958` | `GALEQU_IAU1958` |
| `galequ-true` | `GALEQU_TRUE` |
| `galequ-mula` | `GALEQU_MULA` |
| `galequ-fiorenza` | `GALEQU_FIORENZA` |
| `galalign-mardyks` | `GALALIGN_MARDYKS` |
| `babyl-kugler1` | `BABYL_KUGLER1` |
| `babyl-kugler2` | `BABYL_KUGLER2` |
| `babyl-kugler3` | `BABYL_KUGLER3` |
| `babyl-huber` | `BABYL_HUBER` |
| `babyl-etpsc` | `BABYL_ETPSC` |
| `babyl-britton` | `BABYL_BRITTON` |
| `hipparchos` | `HIPPARCHOS` |
| `sassanian` | `SASSANIAN` |
| `valens-moon` | `VALENS_MOON` |
| `j2000` | `J2000` |
| `j1900` | `J1900` |
| `b1950` | `B1950` |

### 3.3 The dasha systems' spellings

The names `dashaDepthBySystem` keys a depth by.

| stored | system |
|---|---|
| `vimshottari` | `VIMSHOTTARI` |
| `yogini` | `YOGINI` |
| `tribhagi` | `TRIBHAGI` |
| `chara` | `CHARA` |
| `narayana` | `NARAYANA` |
| `ashtottari` | `ASHTOTTARI` |
| `kalachakra` | `KALACHAKRA` |
| `dwisaptati-sama` | `DWISAPTATI_SAMA` |
| `shatabdika` | `SHATABDIKA` |
| `chaturashiti-sama` | `CHATURASHITI_SAMA` |
| `dwadashottari` | `DWADASHOTTARI` |
| `panchottari` | `PANCHOTTARI` |
| `shoola` | `SHOOLA` |
| `niryana-shoola` | `NIRYANA_SHOOLA` |
| `padanadhamsa` | `PADANADHAMSA` |
| `mandooka` | `MANDOOKA` |
| `drig` | `DRIG` |
| `trikona` | `TRIKONA` |
