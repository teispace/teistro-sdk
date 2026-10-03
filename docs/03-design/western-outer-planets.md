# The outer planets in a chart (the `western` module)

Status: `building`, 2026-10-03 — written from the corpus and the code
before any change. Steps 2 to 4 are built: the chart layer and the
SDK place the three when asked, held to the corpus in
[`outer-planets-measured.md`](outer-planets-measured.md); the transit
search, the contacts and the later charts read them; and they cross
the boundary into every binding.

A chart places nine grahas: the seven planets and the two nodes. The
catalogue names three more, Uranus, Neptune and Pluto, and the built-in
ephemeris computes all three. No chart carries them, so every Western
reading that needs them stops short. Leo's lunar year (p. 41) lists the
progressed Moon quincunx Uranus in April 1907, and step 4 of
[`western-progressions.md`](western-progressions.md) had to leave those
contacts out. A Western aspect table, synastry and the wheel all wait on
the same thing. The feature register lists the three as P0, "partial
(positions only)".

## What the corpus records

Each of the corpus's 55 charts records the three under `positions.outer`,
apart from the nine under `positions.bodies`:

- `sidereal_longitude_deg`, `latitude_deg` and `speed_deg_per_day`, and
  nothing else (no house, sign or nakshatra);
- `"frame": "geocentric"`, although the corpus's settings say
  `"topocentric": true` and the nine carry no such flag. The recording
  engine places the outer three from the Earth's centre even where it
  places the nine from the observer.

That is the acceptance test: every chart's three, under the conformance
profile, against what the engine recorded. The bound is the built-in
ephemeris's own. [`builtin-ephemeris-measured.md`](builtin-ephemeris-measured.md)
measures Uranus at 4.73″ and Neptune at 6.57″ worst against a modern
ephemeris (VSOP87's theory is the limit, not the table), and
[`pluto-measured.md`](pluto-measured.md) Pluto's fit under a tenth of an
arcsecond. A chart is read to the arcminute, so these are no obstacle,
but the measured page states them per body instead of one figure.

## Decisions

1. **Asked for, not by default.** `ChartRequest::with_outer_planets()`
   adds them, a section like the rest. A chart that does not ask is the
   chart it was, to the byte and to the hash.
2. **Beside the nine, not among them.** They go in
   `ChartFoundation::outer`, a list in the catalogue's order, omitted
   from the document when empty. They do not join `grahas`, since every
   reader of `grahas` counts nine. The Vedic tables index by graha, and
   the boundary's `cast` section is nine rows a chart. The outer three
   are what the corpus keeps apart too.
3. **One request.** The founder asks for them in the same position grid
   as the nine, so a chart that wants them costs no second request to the
   provider. The cost is three more cells per instant.
4. **The chart's own frame (C239).** They are placed as the nine are:
   in the chart's zodiac, from the chart's centre. The corpus's
   geocentric outer three under a topocentric chart is the engine's
   habit, not a reading anyone asked for. At Uranus's distance the
   parallax is under an arcsecond, so the difference is measured on the
   page, not mirrored.
5. **Each one is a full position.** The type is the nine's
   `GrahaPosition`: longitude in the chart's zodiac and tropical,
   latitude, distance, speed, and its house under both divisions. A
   reader asks a Western question of a body the same way whatever its
   list.
6. **A graha names its body once.** The founder looks a graha's body up
   by its ordinal in the list it asked for, which works for the first
   eight. One function, `body_of`, answers for every graha: Ketu as
   Rahu turned, the node as the settings name it, and the outer three.
   The transit search, the contact search and the progressions then
   accept the outer three where a chart carries them.

## What building it found

- **The corpus holds every bound.** Placed from the Earth's centre, the
  three agree with the recording within 5.11″ (Uranus), 6.14″
  (Neptune) and 0.893″ (Pluto). The bounds are built from the
  ephemeris's parts, not read off a sample. The theory's floor seen
  from the Earth, plus the standard tier's arcsecond, gives 6″ and 8″.
  A first bound taken from the built-in page's sample worst (4.73″) was
  falsified at 5.11″, which is why the bound is now derived.
- **C239 is under an arcsecond.** Placing the three from the observer
  moves them at most 0.559″: 0.51″ of parallax at Uranus's nearest, and
  diurnal aberration.
- **A graha found its body by its ordinal.** The transit search took
  `bodies[graha as usize]`, which reaches the eighth body and no
  further. `body_of` replaces it, so a search can reach Uranus.
- **Leo's April holds.** Founded with the three, his birth's progressed
  Moon makes one contact to Uranus in his forty-seventh year: the
  quincunx, on 16 April 1907, in the month he prints. An earlier draft
  of this page called it a sesquiquadrate; his list says quincunx.
- **A later chart places what the birth placed.** The progressed chart
  and the solar arc's are founded with the three whenever the birth
  carries them, whatever the caller's request asks, so a contact or a
  direction never finds a body the birth had and the later chart lacks.
  The directed planets are twelve then, in the foundation's order.
- **The boundary keeps one count a batch.** `TS_CHART_OUTER` (bit 4096)
  asks for them, and section 81, `outer`, carries them in the columns
  `grahas` has. The summary is unchanged: the section holds the same
  number a chart, three or none, and a reader divides its rows by the
  batch's charts, as the progressed planets already were read. The
  encoder refuses a batch whose charts differ, which only the SDK
  could build. `progressed_grahas` and `directed_grahas` carry the
  outer three after the nine whenever the birth placed them, so the
  two stay the same length.
- **The test provider answers them.** Its elements covered the seven
  and the mean node, so the parity runners could not ask for the three
  at all. Each now has its heliocentric mean motion and its orbit's
  equation of the centre as the one periodic term; no answer the
  provider gave before changed.

## What this does not decide

- **Rulership.** Whether Uranus rules Aquarius, Neptune Pisces and
  Pluto Scorpio is a modern Western claim that no text read so far
  makes. The dignities stay the seven's, as Lilly's are.
- **The Vedic readings.** Vargas, the drishti and the yogas read the
  nine and go on reading them. A divisional place for an outer planet is
  arithmetic, but no source read here uses one.
- **Names.** `sdk.entity` does not yet name the three in every locale
  (`entity-names.md`). A section can carry their keys without names,
  and vetted names are their own task.

## Order of work

1. This page, and C239.
2. The chart layer and the SDK: `body_of`, the founder asking for the
   three in the same grid, `ChartFoundation::outer`, and
   `ChartRequest::with_outer_planets()`. Held to the corpus's 55 charts
   in `outer-planets-measured.md`, which states each body's worst
   difference and the parallax C239 sets aside. **Built.**
3. The readers: a transit's hit list, a progression's contacts and a
   progressed chart accept the three where the birth carries them.
   Leo's Uranus contact on p. 41 becomes a test. **Built.**
4. The boundary and every binding, with the parity gate: `outerPlanets`
   in Node and Dart, `outer_planets` in Python, read back as
   `chart.outer`, and `chart-{i}-outer-{j}` in all four runners.
   **Built.**
5. The wheel draws them (`render-svg.md`, "Outer planets on the
   wheel"). **Waits on names.** A wheel labels a body by its locale
   form, and no strict locale names the three (`entity-names.md`; the
   `UNNAMED` list in `xtask/src/intl.rs`). The baseline engine's tables
   stop at the nine, so the Nepali names need a vetted source before
   the wheel can say anything but the bare key.
