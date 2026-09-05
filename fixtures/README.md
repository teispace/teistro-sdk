# Fixtures

Golden vectors for the conformance harness (ADR-0022): recorded outputs of
other implementations that the SDK must reproduce within a stated
tolerance, each carrying the complete settings profile it was computed
under, the hash of that profile, and every version that influenced the
numbers.

This directory is the seed of the separate conformance repository
(`teispace/teistro-conformance`, CC0-1.0). It moves there before Phase 1
exits and is mounted back here as a pinned submodule. Until the move the
files are under this repository's licence.

| directory | source | evidence rank (ADR-0018) | state |
|---|---|---|---|
| `baseline/` | the baseline engine, version 1.8.0 | 2 | 115 fixtures over 55 charts, exported 2026-09-04 (spike 1) |
| `pyjhora/` | PyJHora | 3 | planned |
| `texts/` | classical texts and printed almanacs, hand-entered with citations | 1 | planned |
| `official/` | the authority's own publications: the national panchanga committee's yearly panchanga for BS 2082 and 2083, read from its published files | 1 for the official calendar | 24 sankranti instants, 4 rows of printed places, 22 days of sunrise and sunset, 8 tithi ends (2026-09-05) |
| `teimeris/` | Teimeris | 2 | `ayanamsha.json`: the engine's mean ayanamsha for every epoch-defined member at 24 Julian epochs from −700 to 2500, 1044 rows, written by the adapter's `ayanamsha-table` binary; `houses.json`: the engine's cusps and angles for twenty-one house systems at ten latitudes, two longitudes and three instants, 1260 rows, by its `houses-table` binary (teimeris 0.1.0, 2026-09-05) |

`tolerances.json` is the one central tolerance file, keyed by field and
provider class, never per fixture. It is provisional until the harness
exists (`docs/05-testing/01-golden-vectors.md`).

`cargo xtask check-fixtures` is the gate: every fixture parses, declares
its schema, carries the settings hash of the profile it claims, holds
every section it lists, is listed in the manifest with nothing unlisted
beside it, and contains no forbidden term.

## `official/`

`npns-2082-2083.json` holds what the national panchanga committee
(`npns.gov.np`) printed in its Rashtriya Panchangam for BS 2082 and
2083, read from the page images of the published PDFs (they carry no
text layer; each sankranti line was read at two magnifications), with
the file URLs, page numbers and retrieval date in the file: the twelve
sankranti instants of each year to the minute in Nepal time, printed
under the panchanga day (sunrise to sunrise, so an hour past 24 is the
small hours of the following civil day); four rows of the "planets at
sunrise" table (sign:degree:minute:second, sidereal); the printed
sunrise and sunset of 22 days; eight tithi ends. It is rank 1 for the
official calendar and for what the committee computes. What it shows
(`docs/calendars/bikram-sambat.md`, R2): the committee's Sun is the
text's within three arcseconds and its 24 sankrantis reproduce within
1.6 minutes; every month start follows the shipped rule; its Moon is
the text's with the apsis making four revolutions fewer in an age; its
star planets are modern positions in the Lahiri frame. Tests:
`crates/calendar/tests/official.rs`, `crates/siddhanta/tests/official.rs`.

## `baseline/`

```text
baseline/
  manifest.json      provenance, the thirteen settings profiles with their hashes, every fixture
  charts/            55 fixtures under the default profile, one per chart: <id>-<place>-<date>.json
  variants/          60 fixtures of the same charts under twelve alternative profiles: <id>-<place>-<date>--<profile>.json
```

The export script lives in the baseline engine's own repository, in that
repository's language, and reads the engine's built packages and its
ephemeris data files. It is not part of this repository. To regenerate:
run it with `--out <a scratch directory>`, copy `charts/`, `variants/`
and `manifest.json` here, and run `cargo xtask check-fixtures`. A
regeneration under a newer engine version is a corpus version bump, never
a silent overwrite (ADR-0022).

### Provenance

Recorded in `manifest.json` and repeated in every fixture:

- engine version 1.8.0; Swiss Ephemeris 2.10.03 through its Node
  binding, with the three `.se1` files that cover 1800 to 2399 and their
  SHA-256 hashes; calculation flags `SEFLG_SWIEPH | SEFLG_SPEED |
  SEFLG_SIDEREAL`, plus `SEFLG_TOPOCTR` when the profile is topocentric;
  Delta T from the library's default model;
- time zones from tzdb 2026a as compiled into the Node runtime's ICU, the
  zone looked up from the coordinates; local mean time from the longitude
  when the input says so;
- the runtime (Node 26) and the decimal library the engine classifies
  with.

### The chart set

Fifty-five charts, chosen to be adversarial rather than typical. The
`tags` field of each fixture says why it is there.

| group | charts | what they exercise |
|---|---|---|
| Nepal and India | c001 to c012 | the default profile's home ground: pre-sunrise and post-sunset births, civil midnight, the +05:30 to +05:45 change of 1986 (and the fifteen minutes that did not exist), a leap day, local-mean-time eras (+05:41:16, +05:21:10), explicit LMT |
| unusual and half-hour zones | c013 to c015, c017, c022, c034 | +06:30, +04:30, +07:30, -10:30, +03:30 |
| southern hemisphere and the equator | c017, c018 to c021, c032, c033, c040 to c042, c044, c045 | southern seasons, the equator at 0.18°S, the date line at +13 and +14, the day Samoa skipped |
| Europe and history | c023 to c026, c030, c038 | LMT London 1830, British double summer time, Paris mean time at the 1900 century turn, 1945 summer time, the day Turkey stopped changing clocks, New York on local mean time |
| daylight-saving edges | c014, c031, c035 to c037, c045 | the first summer-time hour, the last half hour before a gap, an ambiguous local time resolved both ways, a southern fall-back ambiguity |
| high latitude and polar | c027 to c029, c043, c044 | 64°N at midsummer midnight, polar day and polar night at 69.6°N, 64.8°N, 54.8°S |
| altitude | c039, c041, c042 | 2 240 m, 3 640 m and 2 850 m: the topocentric Moon |
| the range of the data files | c046 to c048 | 2350, the first day of 1800, the last days of 2399 |
| found by search | c049 to c055 | instants placed to the second at a boundary, in the topocentric frame the natal chart uses: the Moon at a nakshatra edge and at a pada edge where the geocentric Moon is still on the other side (so the dasha lord or the pada depends on the frame), the last second before and the first after the Sun's sidereal Aries ingress of 2020, the first second after a Mercury station, the ascendant at a sign edge, the first second after a new moon |

Every chart is inside 1800 to 2399, so every position comes from the
`.se1` files; `foundation.ephemeris_source` proves it per fixture.

### The settings profiles

`default` is the baseline engine's shipped default: Lahiri, sidereal,
whole-sign houses, drik, the mean node, sunrise at the refracted disc
edge, the spatial dasha balance, eight chara karakas, the classical
ekadhipatya reduction, topocentric positions. Twelve variants change one
thing each (or the ayanamsha) and export only the sections that thing
affects:

| profile | change | sections | charts |
|---|---|---|---|
| `temporal` | dasha balance by the Moon's real entry and exit instants | foundation, dashas | 10 |
| `true-node` | the true node | positions, vargas | 6 |
| `geocentric` | no topocentric correction | positions, vargas, dashas | 6 |
| `placidus` | Placidus houses | houses | 8 |
| `raman`, `krishnamurti`, `true-chitra`, `fagan-bradley` | the ayanamsha | all | 4 each |
| `tropical` | the tropical zodiac | positions, houses, vargas | 4 |
| `surya-bija` | the classical Surya Siddhanta engine with the node correction | foundation, positions, vargas, panchanga, dashas | 4 |
| `surya` | the same engine, uncorrected | as above | 2 |
| `sunrise-geometric` | sunrise at the disc centre on the true horizon | foundation | 4 |

### The fixture schema (`teistro-conformance/baseline-chart/1`)

Top level: `schema`, `id`, `chart`, `title`, `tags`, `why`, `provenance`,
`input`, `settings`, `settings_hash`, `sections`, then one object per
listed section.

- `input.place` (coordinates, altitude in metres), `input.local` (the
  civil date and time as entered, the LMT flag, the daylight-saving
  choice), `input.search` (for the searched charts: the body, the target
  longitude, the crossing instant found and the whole second chosen) and
  `input.resolved` (the UTC instant, the Julian Day in UT, the zone
  offset in whole minutes, the zone id, the resolver's source and era,
  its warning codes).
- `settings` names every knob; `settings_hash.canonical` is the sorted
  `key=value` list the engine hashes and `settings_hash.value` its
  SHA-256 truncated to 16 hex characters, so the recipe is reproducible.
- `foundation`: the Julian Day, Delta T, the ayanamsha value, the
  ephemeris source, sunrise and sunset for the local civil day and its
  neighbours, the sunrise that anchors the Lagna, the Lagna, the
  panchanga day (which half-arc holds the birth, its bounds, the weekday),
  the hora and abda lords, the upagrahas, the arudha and navamsha Lagna
  signs, the birth timing (ishtakaal, bhayat, bhabhoga in ghati and pala)
  and the engine's own version block with its profile hash.
- `positions.bodies`: the ten bodies in canonical order, each with the
  sidereal and tropical longitudes, latitude, speed, distance, degrees in
  sign, sign, whole-sign house, nakshatra, its lord, pada and akshara,
  retrograde, dignity, moolatrikona, combustion, distance from the Sun,
  vargottama, the three friendship views, the baladi avastha, the
  boundary flags and the jagradadi, deeptadi and lajjitadi avasthas with
  planetary war. `positions.outer`: Uranus, Neptune and Pluto, geocentric.
- `houses`: the selected system's cusps, ascendant, MC and degeneracy
  flag, the same for all 22 systems the engine offers (`all_systems`),
  the Bhava Chalit result, the special lagnas, the arudha padas and the
  chara karakas under both schemes.
- `vargas`: for each of the 21 divisional charts, the sign of every body.
- `panchanga`: the five limbs at the birth instant, from the natal
  (topocentric) Sun and Moon.
- `panchanga_day`: the daily panchanga of the local civil date at the
  place: sunrise and sunset, every tithi, nakshatra, yoga and karana with
  its start and end instants, the inauspicious periods, the choghadiya and
  hora sequences, Abhijit and Brahma muhurta, the lunar month under both
  conventions, panchaka, moonrise and moonset, the Moon and Sun signs,
  the ayana, the disha shool and the five muhurta yogas, plus the Bikram
  Sambat date.
- `dashas`: Vimshottari with the year length (365.25 days), the sequence,
  the starting lord and the Moon's elapsed nakshatra fraction, then per
  balance method (`spatial` and `temporal` under the default profile): the
  remaining fraction, the nakshatra's real entry and exit instants
  (temporal), the balance breakdown, the tree as pre-order rows `[path,
  lord, start_jd, end_jd]` to depth 3 (spatial) or 2 (temporal), the
  direct children at chosen paths down to level 5, and the active chain
  of five levels at two reference instants (birth plus 10 000 days, and
  2026-09-04T00:00:00Z).

Conventions: angles are degrees as computed, `f64`, never rounded;
instants are Julian Days in UT; sign index 0 is Aries; nakshatra index 0
is Ashwini; pada 1 to 4; houses 1 to 12; `weekday_swe` counts Monday as
0. Longitudes are sidereal unless the field says tropical. A body's
nakshatra, pada and sign in the fixture are the engine's classification
of its own `f64`, made with decimal arithmetic and lower-inclusive
boundaries; the harness classifies the same `f64` with the SDK's integer
path (ADR-0016) and expects the same answer.

### Baseline conventions to register, not to copy

Observed while exporting. Each becomes a row of the deliberate-difference
registry the harness reads, so a divergence here is expected and
explained rather than a failure or a silent adoption.

1. The natal panchanga uses the topocentric Moon (c055 shows tithi 1 a
   second after the topocentric new moon); the daily panchanga is
   geocentric like every almanac. The SDK's day-boundary and frame knobs
   decide this explicitly (`02-architecture/05-data-model-identifiers.md`).
2. Local mean time is rounded to the whole minute (c023, c047: a
   longitude of -0.13° gives -1 minute, not -31 seconds), and
   `tz_offset_min` is whole minutes while the resolved instant keeps the
   seconds of historical offsets (c005, c011, c025, c038).
3. Polar day and polar night have no horizon crossing; the engine
   synthesises sunrise and sunset as the bounds of the civil day (c028:
   a 24-hour day) or collapses them (c029: a zero-length day and a
   post-sunset birth). The SDK's polar policy is a setting (ADR-0016's
   companion pages); these two fixtures are recorded for comparison only.
4. Sunrise ignores the observer's altitude; the altitude reaches only the
   topocentric positions.
5. The Placidus cusps at 69.6°N (c028, c029) are returned without a
   degeneracy flag; the library falls back internally above the polar
   circle and the engine's heuristic does not notice. The SDK's house
   module must flag them (`03-design/` house page, planned).
6. Ketu is Rahu plus 180° in the same frame; the nodes carry no
   latitude, speed or distance of their own beyond what the library
   reports for the node.
7. The dasha balance breakdown uses 365.25-day years and 30.4375-day
   months with integer floors and a rounded final minute.
8. Zone ids come from the coordinates through a lookup library and
   canonical tzdb names, so Tromsø resolves to `Europe/Berlin` (a tzdb
   link); the SDK reports the id its own provider gives.
9. The engine stores no tropical longitude; the exporter asked the
   library for it in the positions' frame, and for the Lagna added the
   nutated ayanamsha to the sidereal value (the two agree to a
   milliarcsecond for the planets).
10. The `outer` planets are geocentric even under a topocentric profile.
11. A zone resolution's era is labelled by comparing the applied offset
    with the offset in force when the export ran, a northern summer, so a
    seasonal offset the zone still applies every year reads `historical`
    (c018 Sydney and c019 Auckland in their summer time, c029 Berlin and
    c035 New York in standard time, c037 the later occurrence of the New
    York fold); the SDK compares with the offsets the zone applies in the
    database's own year and reads no clock, so it calls those `current`
    (cruxes register C33).

12. Sunrise and sunset. The baseline's library computed refraction from
    its standard atmosphere; the SDK's standard refraction is the
    almanac's 34 arcminutes with the semidiameter and the horizontal
    parallax from the distance, so the SDK's sunrise under the same
    convention differs from the fixtures' by up to 2.5 s below 60° of
    latitude and 9.8 s at Fairbanks on the solstice (cruxes C34); the
    band on `foundation.sunrise.*_jd` is set from that spread when the
    harness lands. The day's arc in the SDK is the civil date's sunrise
    and the sunset that follows it, as the baseline reckons for c043,
    whose sunset falls after the next civil midnight. For c022, c025 and
    c039 the baseline's `foundation.sunrise` block holds the previous
    day's events and its `next_day` block the civil date's own (C35): a
    comparison reads the latter there.

13. Planetary hours. The baseline reckons twelve horas over the day from
    sunrise to sunset and twelve over the night (`hora_reckoning:
    PROPORTIONAL`); `crates/time/tests/hora_fixtures.rs` reproduces the
    lord at the birth instant for every chart except c022 and c039, whose
    day-early sunrise block (convention twelve) places the birth in
    another day's horas, and c028, whose polar day the baseline
    synthesises. The equal reckoning (sixty-minute horas from sunrise)
    disagrees with the fixtures on many charts, so they decide the
    default.
