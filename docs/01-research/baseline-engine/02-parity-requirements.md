# baseline engine: parity requirements

Status: `research`, 2026-09-04; measured against the code 2026-10-27
("State", below): 14 lines met, 16 partly, 2 not met. The checklist v1.0 must satisfy for the baseline engine to
delete its `packages/` folder. Each line will become a conformance test with
golden vectors exported from the baseline engine before migration.

## Inputs and settings

- [ ] Birth data: name, gender, UTC instant, Julian day, latitude, longitude,
      altitude, IANA zone, LMT flag, time accuracy, birth order, BS date.
- [x] Settings: ayanamsha (47 plus custom), zodiac mode, house system (22
      plus the named Bhava-Chalit variants), siddhanta (drik, Surya with
      bija), node type, sunrise mode, dasha balance method, chara karaka
      scheme, ekadhipatya method, topocentric flag, locale profile.
- [x] Profiles: "Nepali default" (Lahiri, sidereal, whole sign, drik,
      mean node, apparent-refraction sunrise, temporal balance, classical
      ekadhipatya, eight karakas, topocentric on, bija on).

## Computations (every row is a golden-vector set)

- [x] Positions with speeds, retrograde and combustion, nakshatra and pada,
      navamsa sign, dignities, avasthas (five families), planetary war.
- [x] Houses for all 22 systems; cusps and spans; degeneracy states.
- [x] Vargas: the baseline engine set with the same mapping; varga placements and
      vargottama.
- [ ] Aspects: graha drishti values, sphuta drishti, rashi drishti.
- [ ] Upagrahas, special lagnas, Bhrigu bindu, yogi points, mrityu and
      pushkara bhagas, marana karaka sthana. Pushkara navamshas and
      bhagas (*Vidyamadhaviyam* ch. 1 v. 18, C767) and marana karaka
      sthana (*Jataka Parijata* ch. 17 vv. 34–35, C768) are computed in
      every graha's state since 2026-10-27, on their verses: the corpus
      records none of them. Bhrigu bindu is catalogued and not computed,
      because no text defining it has been read (C769).
- [x] Yogas (562) and doshas (62): presence, strength, participants, houses,
      cancellations. Mechanical export of the rule corpus.
- [x] Shadbala with every sub-bala, Bhava Bala, Ashtakavarga (BAV, SAV,
      shodhana with three ekadhipatya methods, pindas, kakshya).
- [x] Dashas: all 18 systems, trees to the baseline depths, spatial and
      temporal balances, active chain at an instant, tiered persistence
      depths for the dossier.
- [ ] Jaimini: karakas (7 and 8), arudhas, three pairs.
- [ ] Ayurdaya (three methods, headline and spread), Maraka, longevity
      windows, Balarishta states.
- [x] Birth timing: Ishtakaal, bhayat, bhabhoga, ghati-pala both reckonings,
      ghati-pala to clock inverse.
- [x] Daily panchanga: every limb with exact transitions (true spans, not
      day-clipped), all derived timings and yogas, Moon sign, ayana, lunar
      month, eras.
- [ ] Muhurta search: identical ranking on the baseline regression set
      (activities, blackouts, gates, shuddhi adjustments, event rules,
      mahadosha caps, windows).
- [ ] Gochar: overlays, transit aspects, Sade Sati phases with dates,
      transit strength, phala with vedha, transit calendar.
- [ ] Tajika: annual chart, Muntha, aspects and yogas as implemented.
- [ ] KP: sub-lords at four levels, significators, ruling planets, Placidus
      with fallback.
- [ ] Milan: Ashta Koot scores and cancellations, extended checks, Mangal
      match, marriage doshas, recommendations, quick milan, akshar
      resolution.
- [x] Rectification: identical interval-first result on the baseline engine
      regression cases (stages, refinement, hold-out).
- [ ] Prashna: yes/no score, timing, topic, arudha, void-of-course.
- [ ] Lal Kitab, numerology, Pancha Pakshi, namakarana (validation and
      offline suggestions), remedies (gemstone safety, priorities),
      research (synastry, composites, statistics).
- [ ] Rashifal context and scoring for daily, weekly, monthly, yearly.

## Presentation

- [ ] Every state and rule key resolves to text in ne, en, sa, hi from packs
      exported from the baseline engine's interpretation records with citations.
- [ ] The composers produce byte-identical text to the baseline engine's for the same
      inputs (a translation-quality gate, not only a computation gate).
- [x] Chart serialiser: the date-free dossier with presets, byte-identical
      for the same chart, and the gochar sidecar.
- [x] Chart layout geometry for North, South and East Indian styles.

## Non-functional

- [x] the baseline engine's full natal compute (foundation plus every slice) on the SDK
      through the Node binding is not slower than the current engine on the
      same machine, measured by an interleaved benchmark.
- [x] The Node binding runs in worker threads without global-state hazards.
- [x] The same inputs give byte-identical JSON in Node, wasm, Python and
      Dart bindings.
- [ ] A migration guide maps every baseline engine service and type to the
      SDK API.

## State, measured 2026-10-27

A line is ticked only where a test or gate holds the SDK to the baseline
engine: the corpus (`fixtures/baseline`), a black-box run, or the A/B held
by `check-ab`. Code with nothing comparing it to the baseline is "partly".

| line | state | what holds it, or what is missing |
|---|---|---|
| birth data | decided otherwise | the SDK takes an instant and a place (`Place`, the civil time resolved as the baseline resolves it, `crates/time/tests/fixtures.rs`); a name, a gender and a birth order are the application's, not the computation's, and the Bikram Sambat date is `calendar.convert`. Decided 2026-10-27 |
| settings, profile | met | 47 ayanamshas and a custom one, 22 house systems and the chalit, every knob; `conformance-baseline` is the baseline's default, and every corpus test founds under it |
| positions, dignities, avasthas, war | met | `crates/ephemeris-builtin/tests/conformance.rs`, `crates/state/tests/baseline.rs` (four avastha families; the corpus records no fifth) |
| houses, vargas | met | `crates/astro/tests/baseline_houses.rs`, `crates/houses/tests/baseline.rs`, `crates/vargas/tests/baseline.rs` |
| aspects | partly | built (`crates/aspect`); the corpus records no drishti, so a baseline export of graha, sphuta and rashi drishti is owed |
| points | partly | upagrahas, yogi points, special lagnas, mrityu bhaga, pushkara navamshas and bhagas (C767) and marana karaka sthana (C768) held; Bhrigu bindu is catalogued and not computed: no text defining it has been read (C769) |
| yogas, doshas | met | 76 921 and 8 565 comparisons agree; the engine records no yoga strength, so none is compared; its corpus holds 605 yogas and 69 doshas, not the 562 and 62 above |
| balas, ashtakavarga | met | 11 431, 3 408 and 1 848 comparisons; kakshya by a unit test only |
| dashas | met | every recorded system, both balances, the chains and the dossier depths |
| Jaimini | partly | arudhas held; the whole chara karaka order under seven and eight karakas, 54 of 55 each, the miss a graha within an arc-second of a sign edge (`crates/sdk/tests/baseline_chara_karakas.rs`); the three pairs have no vector |
| ayurdaya, maraka, balarishta | partly | built (`crates/rules/src/longevity`); no baseline vectors |
| birth timing | met | `crates/chart/tests/baseline_day.rs` |
| daily panchanga | met | periods, classification, month, panchaka, ayana and shool held; muhurta yogas 253 of 265 flags (the 12 the recording's own weekday-and-nakshatra table), moonrise and moonset within 30 s over 104 events, and 271 limb changes within 10 s in TT (`crates/sdk/tests/baseline_panchanga_day.rs`) |
| muhurta ranking | partly | `Ranking::Baseline`, one regression; no exported regression set compared rank for rank |
| gochar | partly | longitude, house and bindus by the A/B; transit strength, vedha phala and the calendar have no vectors |
| Tajika | decided otherwise | the SDK follows Charak and the *Tajika Nilakanthi* and matches Charak's worked year in full; the A/B counts where the baseline differs (24.4% agreement, the sahams by other formulas). "As implemented" is not a goal; a baseline-reading mode is v1.x, if a consumer asks. Decided 2026-10-27 |
| KP | partly | the four lord levels 98.4% by the A/B; significators and ruling planets are not compared |
| milan | decided otherwise, partly | the SDK follows the *Kalaprakasika* (100% against it; 68.5% koota and 94.1% porutham agreement with the baseline, method differences); the recommendation bands wait on a rank 1 text, and the extended checks and marriage doshas have no vectors. Decided 2026-10-27 |
| rectification | met | the black-box run, 8 cases, intervals to a second (`crates/sdk/tests/rectify.rs`) |
| prashna, rashifal, Track B modules | partly | built and hand-worked; no baseline vectors; name suggestions and gemstone safety not built |
| text in ne, en, sa, hi | partly | 639 of 657 rules carry a reading; Hindi and Sanskrit messages wait on vetted translators (`locale-coverage-measured.md`) |
| composers byte-identical | decided otherwise | the SDK composes through Teistro Intl's own grammar, so the same bytes are not a goal; parity is on what a record says, and the baseline's records are migrated (`teistro-intl migrate baseline`). Decided 2026-10-27 |
| serialiser | met | the dossier and its presets, the same bytes for the same chart (`check-dossier`); the gochar sidecar in Rust (`GocharSidecar`, `03-design/gochar-sidecar.md`), checked against its chart and its settings, its transits the whole document's (`gochar-sidecar-measured.md`); a binding asks for transits on the chart request instead of storing one, decided 2026-10-28 |
| layouts | met | all three styles (`crates/render-svg/tests/render.rs`) |
| speed, workers, the same JSON | met | `check-ab`, `bindings/node/test/workers.test.mjs`, `check-parity` |
| migration guide | not met | being written (`09-guidelines/baseline-migration.md`); the compatibility shim is not built |
