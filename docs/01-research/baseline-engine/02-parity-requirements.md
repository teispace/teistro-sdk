# baseline engine: parity requirements

Status: `research`, 2026-09-04; measured against the code 2026-10-27,
the aspects, prashna and rashifal 2026-10-29 ("State", below): 16 lines
met, 14 partly, 2 not met. The checklist v1.0 must satisfy for the baseline engine to
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
- [x] Aspects: graha drishti values, sphuta drishti, rashi drishti.
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
- [x] Jaimini: karakas (7 and 8), arudhas, three pairs.
- [ ] Ayurdaya (three methods, headline and spread), Maraka, longevity
      windows, Balarishta states.
- [x] Birth timing: Ishtakaal, bhayat, bhabhoga, ghati-pala both reckonings,
      ghati-pala to clock inverse.
- [x] Daily panchanga: every limb with exact transitions (true spans, not
      day-clipped), all derived timings and yogas, Moon sign, ayana, lunar
      month, eras.
- [ ] Muhurta search: identical ranking on the baseline regression set
      (activities, blackouts, gates, shuddhi adjustments, event rules,
      mahadosha caps, windows). Not identical: by the A/B's `muhurta`
      under `same`, 20 of 54 marriage rankings agree rank for rank (19
      of them empty on both sides) and 46.7% of the facts; every miss is
      the baseline's day-level gates and sampling (crux C783), its
      weekday (C782), its special-yoga tables (C784), Chaturmas (C166) or
      asta (C44, C164), none the SDK's (row "muhurta ranking" below).
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
- [x] Prashna: yes/no score, timing, topic, arudha, void-of-course.
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
| aspects | met | by the A/B under `same`, every chart's full aspects (`drishti`), every graha's graded aspect on every other in percent (`drishtiGrade`, 99.7% under `timed`, the misses a Moon a few arcminutes from a sign's edge) and every body's rashi drishti (`rashiDrishti`, 99.8% under `timed`, the same Moons) agree at 100% (`ab-measured.md`). The sphuta drishti by degree is in every chart's aspects since 2026-10-29 (`Aspects::sphuta`, every binding's `sphutaDrishti`); the baseline computes none, so it is held by Raman's worked example (`crates/aspect`, Sripati's rule, C45) |
| points | partly | upagrahas, yogi points, special lagnas, mrityu bhaga, pushkara navamshas and bhagas (C767) and marana karaka sthana (C768) held; Bhrigu bindu is catalogued and not computed: no text defining it has been read (C769) |
| yogas, doshas | met | 76 921 and 8 565 comparisons agree; the engine records no yoga strength, so none is compared; its corpus holds 605 yogas and 69 doshas, not the 562 and 62 above |
| balas, ashtakavarga | met | 11 431, 3 408 and 1 848 comparisons; kakshya by a unit test only |
| dashas | met | every recorded system, both balances, the chains and the dossier depths |
| Jaimini | met | arudhas held; the whole chara karaka order under seven and eight karakas, 54 of 55 each, the miss a graha within an arc-second of a sign edge (`crates/sdk/tests/baseline_chara_karakas.rs`); the three pairs by the A/B (`jaiminiPair`, 81.7% under `same`): the lagna lord with the eighth lord and the lagna with the hora lagna agree on every chart, and the second pair is BPHS ch. 43 v. 33's Saturn and Moon where the baseline pairs the lagna and the Moon, which alone moves the class they decide |
| ayurdaya, maraka, balarishta | partly | by the A/B under `same`: Pindayu, Amsayu and Nisargayu year by year 73.4% (`ayurdaya`), every miss a reading: v. 11's Moon a benefic in the visible half, vv. 20 to 22's Amsayu multipliers, Nisargayu's listed years unreduced; the marakas by lordship 100% (`maraka`); Balarishta 98.1%, the misses its cancellations (`balarishta`). The longevity windows are not compared |
| birth timing | met | `crates/chart/tests/baseline_day.rs` |
| daily panchanga | met | periods, classification, month, panchaka, ayana and shool held; muhurta yogas 253 of 265 flags (the 12 the recording's own weekday-and-nakshatra table), moonrise and moonset within 30 s over 104 events, and 271 limb changes within 10 s in TT (`crates/sdk/tests/baseline_panchanga_day.rs`) |
| muhurta ranking | partly | `Ranking::Baseline`, one regression; by the A/B (`muhurta`, a marriage over 25 days at Kathmandu for each of 54 births, with and without the native) under `same`, 20 of 54 rankings identical rank for rank, 19 of them empty on both sides, and 46.7% of the facts (44.5% under `timed`). A window's score agrees wherever its day's does, its sky geocentric with the true node (C781); the rest is the day: the baseline bars a whole day by the star and solar month at sunrise and samples its seasons at 0h UT where the SDK judges each window (C783), reads the weekday off sunrise's UT date (C782), scores its own special-yoga tables (C784), anchors Chaturmas by the Sun's sign (C166) and sees asta heliacally (C44, C164). No miss is an SDK defect |
| gochar | partly | longitude, house, bindus (each transit sign's own and sarva) and transit strength by the A/B; the good houses and a good transit's vedha 96.7% under `same` (`gochar`), every miss Phaladeepika ch. 26's pairs and exemptions (vv. 2 to 8, cruxes C136, C140); the calendar has no vectors |
| Tajika | decided otherwise | the SDK follows Charak and the *Tajika Nilakanthi* and matches Charak's worked year in full; the A/B counts where the baseline differs (24.4% agreement, the sahams by other formulas). "As implemented" is not a goal; a baseline-reading mode is v1.x, if a consumer asks. Decided 2026-10-27 |
| KP | met | by the A/B under `same`: the four lord levels 98.4%, the significators' four levels 95.2% and the ruling planets 100%, `same` asking the baseline's ruling count and civil day lord (cruxes C150, C151); every miss an intercepted sign's lord, which the baseline counts a significator and the SDK reports and does not count (crux C154), or the frame. Under `timed` the day lord also differs, KP Reader VI's day turning at sunrise (`ab-measured.md`) |
| milan | decided otherwise, partly | the SDK follows the *Kalaprakasika* (100% against it; 68.5% koota and 94.1% porutham agreement with the baseline, method differences); the recommendation bands wait on a rank 1 text; by the A/B under `same`, the Kuja dosha 90.9% (`kuja`, the misses the baseline's own cancellations) and the Ashta Koota's marriage doshas 100% (`marriageDosha`). Decided 2026-10-27 |
| rectification | met | the black-box run, 8 cases, intervals to a second (`crates/sdk/tests/rectify.rs`) |
| prashna | met | by the A/B (`prashna`, `ab-measured.md`): the answer and its points, the timing's unit and count, the topic's house and graha, the querent's number's sign and the Moon void of course and applying agree on every birth under `same`, 100% of 477 (c047 and c048, the data's edges, unanswered as `strength` is under `same`: its Shadbala asks for the Sun past the data); under `timed` 96.6% of 495, every miss the frame: the baseline casts a question's chart from the Earth's centre on the true node, the birth's from its place on the mean node (C800). The SDK is 2.97× the baseline's time, a founded chart against nine longitudes and a lagna (`SLOWER`, `xtask/src/ab.rs`) |
| rashifal | decided otherwise, partly | by the A/B (`rashifal`, `ab-measured.md`) for a day, a week, a month and a year: 88.4% under `same` (the readings 95.6%, the scores 74.1% within a point), 84.4% under `timed`, which reads at sunrise and reports every graha's events (C358, C360). Under `same` the transits agree to the arcsecond; the readings' misses are the baseline's vedha tables (C801, the nodes' C136 and C140) and its scan's events (no node's ingress, none in a scan's last partial step, C360), each decided for the text; the score is the baseline's formula (C361) over those readings, and where a sign's readings agree its scores agree within a point on 99.5%, the rest by two (C802, open) |
| Track B modules | partly | built and hand-worked; no baseline vectors; gemstone safety not built; names: the reading (declared schemes and `CASUAL`), the check with its clauses, a pada's syllables, and the consumer's name book loaded and searched, built 2026-10-28 in Rust and as `naming` in every binding and as MCP tools (`03-design/namakarana.md` §10); no corpus ships |
| text in ne, en, sa, hi | partly | 639 of 657 rules carry a reading; Hindi and Sanskrit messages wait on vetted translators (`locale-coverage-measured.md`) |
| composers byte-identical | decided otherwise | the SDK composes through Teistro Intl's own grammar, so the same bytes are not a goal; parity is on what a record says, and the baseline's records are migrated (`teistro-intl migrate baseline`). Decided 2026-10-27 |
| serialiser | met | the dossier and its presets, the same bytes for the same chart (`check-dossier`); the gochar sidecar in Rust (`GocharSidecar`, `03-design/gochar-sidecar.md`), checked against its chart and its settings, its transits the whole document's (`gochar-sidecar-measured.md`); a binding asks for transits on the chart request instead of storing one, decided 2026-10-28 |
| layouts | met | all three styles (`crates/render-svg/tests/render.rs`) |
| speed, workers, the same JSON | met | `check-ab`, `bindings/node/test/workers.test.mjs`, `check-parity` |
| migration guide | partly | the guide (`09-guidelines/baseline-migration.md`) and its generated map (`09-guidelines/baseline-map.md`, held by `check-compat`, its §1 counting the shim's rows by kind and every stored settings row); the shim built (`bindings/node/compat`, `03-design/compat-shim.md`). Met when `check-compat` passes with zero unmapped entry points: the gate counts them from the A/B adapter's `surface` export, which the adapter does not have yet, so the count is not taken (`ab-measured.md`) |
