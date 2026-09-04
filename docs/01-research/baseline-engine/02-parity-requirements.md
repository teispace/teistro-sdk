# baseline engine: parity requirements

Status: `research`, 2026-09-04. The checklist v1.0 must satisfy for the baseline engine to
delete its `packages/` folder. Each line will become a conformance test with
golden vectors exported from the baseline engine before migration.

## Inputs and settings

- [ ] Birth data: name, gender, UTC instant, Julian day, latitude, longitude,
      altitude, IANA zone, LMT flag, time accuracy, birth order, BS date.
- [ ] Settings: ayanamsha (47 plus custom), zodiac mode, house system (22
      plus the named Bhava-Chalit variants), siddhanta (drik, Surya with
      bija), node type, sunrise mode, dasha balance method, chara karaka
      scheme, ekadhipatya method, topocentric flag, locale profile.
- [ ] Profiles: "Nepali default" (Lahiri, sidereal, whole sign, drik,
      mean node, apparent-refraction sunrise, temporal balance, classical
      ekadhipatya, eight karakas, topocentric on, bija on).

## Computations (every row is a golden-vector set)

- [ ] Positions with speeds, retrograde and combustion, nakshatra and pada,
      navamsa sign, dignities, avasthas (five families), planetary war.
- [ ] Houses for all 22 systems; cusps and spans; degeneracy states.
- [ ] Vargas: the baseline engine set with the same mapping; varga placements and
      vargottama.
- [ ] Aspects: graha drishti values, sphuta drishti, rashi drishti.
- [ ] Upagrahas, special lagnas, Bhrigu bindu, yogi points, mrityu and
      pushkara bhagas, marana karaka sthana.
- [ ] Yogas (562) and doshas (62): presence, strength, participants, houses,
      cancellations. Mechanical export of the rule corpus.
- [ ] Shadbala with every sub-bala, Bhava Bala, Ashtakavarga (BAV, SAV,
      shodhana with three ekadhipatya methods, pindas, kakshya).
- [ ] Dashas: all 18 systems, trees to the baseline depths, spatial and
      temporal balances, active chain at an instant, tiered persistence
      depths for the dossier.
- [ ] Jaimini: karakas (7 and 8), arudhas, three pairs.
- [ ] Ayurdaya (three methods, headline and spread), Maraka, longevity
      windows, Balarishta states.
- [ ] Birth timing: Ishtakaal, bhayat, bhabhoga, ghati-pala both reckonings,
      ghati-pala to clock inverse.
- [ ] Daily panchanga: every limb with exact transitions (true spans, not
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
- [ ] Rectification: identical interval-first result on the baseline engine
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
- [ ] Chart serialiser: the date-free dossier with presets, byte-identical
      for the same chart, and the gochar sidecar.
- [ ] Chart layout geometry for North, South and East Indian styles.

## Non-functional

- [ ] the baseline engine's full natal compute (foundation plus every slice) on the SDK
      through the Node binding is not slower than the current engine on the
      same machine, measured by an interleaved benchmark.
- [ ] The Node binding runs in worker threads without global-state hazards.
- [ ] The same inputs give byte-identical JSON in Node, wasm, Python and
      Dart bindings.
- [ ] A migration guide maps every baseline engine service and type to the
      SDK API.
