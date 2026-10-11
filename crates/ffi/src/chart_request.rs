//! The chart request at the C boundary, compiled into every build: the
//! generated glue builds one whichever families a build has, and a build
//! without the chart area refuses it unread (`03-design/wasm-profiles.md`).

use core::ffi::c_char;

use crate::support::c_struct;

/// What a chart is founded on: when, where, what kind, and the clock its
/// day is reckoned in.
///
/// Everything else is the context's settings, which is what makes two
/// calls under one context comparable and what the settings hash is for.
/// The clock is here because nothing else knows it: a chart's day runs
/// from a local sunrise and its date is a civil date, and a longitude
/// gives local *mean* time rather than a civil offset
/// (`03-design/chart-at-the-boundary.md` §5).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TsChartRequest {
    /// `sizeof(ts_chart_request)` as the caller compiled it.
    pub struct_size: u32,
    /// What kind of chart to found.
    /// `api: enum=ChartKind example=0`
    pub kind: u16,
    /// Reserved; write zero.
    pub reserved: u16,
    /// The instants, as Julian days on the UTC scale: one chart each.
    ///
    /// A grid, not a scalar, because the founder shares the settings and
    /// the solar model across a batch and a rectification pass wants a
    /// hundred charts (`03-design/chart-at-the-boundary.md` §3a). A
    /// caller wanting one passes a grid of one, as `ts_positions` takes
    /// a grid of one instant.
    /// `api: len=instant_count unit=jd`
    pub instants: *const f64,
    /// How many instants `instants` points at.
    pub instant_count: usize,
    /// The place's latitude, degrees north.
    /// `api: unit=deg range=[-90,90] example=27.7172`
    pub latitude_deg: f64,
    /// The place's longitude, degrees east.
    /// `api: unit=deg range=[-180,180] example=85.324`
    pub longitude_deg: f64,
    /// The place's altitude, metres above the ellipsoid.
    /// `api: unit=m range=[-500,9000] example=1400`
    pub altitude_m: f64,
    /// The local clock's offset from UTC in seconds, east positive: the
    /// clock the day's date is read in.
    /// `api: unit=s range=[-64800,64800] example=20700`
    pub utc_offset_seconds: i32,
    /// Reserved; write zero.
    pub reserved_tail: i32,
    /// Which of the document's sections to compute beside the
    /// foundation, as a bit set: 1 the day's almanac, 2 the planetary
    /// states, 4 the aspects, 8 the derived points, 16 the houses
    /// service, 32 the Ashtakavarga, 64 the Vimshopaka, 128 the Shadbala, 256 the Bhava bala, 512 the Vaiseshikamsa, 1024 the dasha phala. Zero for the foundation alone, which is what every
    /// caller compiled against an earlier header passes by not passing
    /// it at all.
    ///
    /// A bit set here and a named option in every ergonomic layer, which
    /// is the split `ts_frame_pack` already has: nothing but a generated
    /// layer writes bits (`03-design/chart-reading.md` §5).
    /// `api: example=0`
    pub sections: u32,
    /// Reserved; write zero.
    pub reserved_sections: u32,
    /// Which divisional charts to compute, as catalogue ids, in the
    /// order they should be answered in; null with a count of zero for
    /// none, as `instants` takes a grid of none.
    ///
    /// **Not `nullable`**, and that is the description's word rather
    /// than a promise about the pointer: `nullable` makes the generated
    /// field an `Option` of the whole parameter, and an optional *array
    /// of enum members* is a shape no emitter has been shown — it mapped
    /// the option's contents where it meant to map the array's. An empty
    /// array says "none" without needing one, which is what `instants`
    /// already does.
    /// `api: len=varga_count enum=Varga`
    pub vargas: *const u16,
    /// How many divisional charts `vargas` points at.
    pub varga_count: usize,
    /// Which charts to draw, and in which layouts, in the order they should
    /// be answered in: each `layout_id << 16 | varga_id`, a `chart_layout`
    /// catalogue id and a `Varga` id, `D1` for the founded chart. Null with a
    /// count of zero for none.
    ///
    /// Packed, as `sections` is a bit set, so the request carries one array
    /// and one count rather than two arrays that must agree; every ergonomic
    /// layer takes named pairs and writes the bits (`03-design/chart-geometry.md`).
    /// `api: len=drawing_count`
    pub drawings: *const u32,
    /// How many drawings `drawings` points at.
    pub drawing_count: usize,
    /// Which dashas to compute, in the order they should be answered in: each
    /// a `DashaSystem` catalogue id, or the id `ts_key_parse` gives a system
    /// the context registered (`0x8000` and up). Each one's balance and its
    /// periods to its depth. Null with a count of zero for none.
    ///
    /// Ids and not an enum, as `drawings` carries layout ids: every ergonomic
    /// layer takes a catalogue member or a registered key and writes the id.
    /// `api: len=dasha_count`
    pub dashas: *const u16,
    /// How many dashas `dashas` points at.
    pub dasha_count: usize,
    /// A theme to write every drawing as SVG in, as JSON: an object of
    /// `style` and `content` naming only what it changes, over the light
    /// theme or the shipped one its `extends` names (`{"extends": "DARK"}`).
    /// The SVGs come back in the blob's `svgs` section, in the context's
    /// locale. Null for none, which costs nothing
    /// (`03-design/render-svg.md`).
    /// `api: nullable example={"extends":"DARK"}`
    pub theme_json: *const c_char,
    /// Rules to answer over every chart, as JSON: `shipped` names the
    /// kernel's sets, `rules` a consumer's own in the rule format, with
    /// `readings`, `houses` and `longevity` choosing what else comes back
    /// (`03-design/rules-at-the-boundary.md`). The answers come back in the
    /// blob's `rules` section, and the sections the rules read are computed
    /// whether or not `sections` asked for them. Null for none, which costs
    /// nothing.
    /// `api: nullable example={"shipped":["NABHASAS"]}`
    pub rules_json: *const c_char,
    /// Narrative plans to compose over every chart, as JSON: an object
    /// naming the composers to run — `placements`, `readings`, `strength`,
    /// `houses`, `positions`, `aspects`, `conditions` and `karakas` — each
    /// false by default. The plans come back in the blob's `plans`
    /// section, holding no words at all — an item's params are the JSON
    /// `ts_intl_render` takes, so a binding says one by handing it
    /// straight back, in any locale and in as many as it likes
    /// (`03-design/plans-at-the-boundary.md`). `readings` says what the
    /// rules answered, so it needs `rules_json` beside it; the sections
    /// the other composers read are computed for them, as the rules' are.
    /// Null for none, which costs nothing.
    /// `api: nullable example={"placements":true}`
    pub interpret_json: *const c_char,
    /// The annual charts to answer for every chart in the batch, as a JSON
    /// object: `reading` — `"SIDEREAL"` (the tradition's), `"TROPICAL"`
    /// (the Western solar return) or `"MEAN"` (a whole sidereal year each
    /// time) — and `through`, the last year of life wanted, 1 to 200. The
    /// instants come back in the `praveshas` section, ragged by
    /// `cast.pravesha_count`; an ephemeris that ends first answers fewer
    /// than asked for rather than refusing. Null for none
    /// (`03-design/annual-chart.md`). Refusals are named from the record
    /// every binding calls `varsha`, as `varsha.through`.
    /// `api: nullable`
    pub varsha_json: *const c_char,
    /// The transits to read against every chart in the batch, as a JSON
    /// object: `instants`, UTC Julian days, at least one; `from` —
    /// `"MOON"` (Phaladeepika ch. 26 v. 1's, the default) or `"LAGNA"`; and
    /// `ashtakavarga`, true to judge the seven by the natal bindus too, in
    /// the `gochar_ashtakavarga` section.
    /// Each chart's readings come back in the `gochar` section, a row an
    /// instant, and its grahas in `gochar_grahas`, under the settings'
    /// `gochar` group. Null for none (`03-design/gochar.md`). Refusals are
    /// named from the record every binding calls `gochar`, as
    /// `gochar.instants`.
    /// `api: nullable example={"instants":[2460676.5],"from":"MOON"}`
    pub gochar_json: *const c_char,
    /// The transit hit list to search against every chart in the batch,
    /// as a JSON object: the window `from` and `to`, UTC Julian days, and
    /// optionally `grahas` (keys, the nine by default), `kinds`
    /// (`"SIGN_INGRESS"`, `"NAKSHATRA_INGRESS"`, `"STATION"`, `"ASPECT"`;
    /// all by default), `points` (the natal points aspected: a graha's key
    /// or `"LAGNA"`, or an answer's `to`; the nine and the lagna by
    /// default), `aspects` (angles, whole degrees to 180; 0 and 180 by
    /// default, C145) and `orbDeg` (more than 0, under 15 and under half
    /// the step between the aspects' lines, for the windows' edges; exact
    /// only by default, C146).
    /// Each chart's hits come back in the `hits` section, `cast.hit_count`
    /// rows a chart, the sky searched once for the batch. Null for none
    /// (`03-design/transit-hit-list.md`). Refusals are named from the
    /// record every binding calls `hits`, as `hits.to`.
    /// `api: nullable example={"from":2460676.5,"to":2461041.5,"grahas":["SATURN"]}`
    pub hits_json: *const c_char,
    /// Sade Sati and the smaller spells of Saturn to find for every chart
    /// in the batch, as a JSON object: `from`, a UTC Julian day, and
    /// optionally `to` (the window's end, `from` by default), `countedFrom`
    /// (`"MOON"`, the default, or `"LAGNA"`; C139), `reckoning` (`"SIGN"`,
    /// the default, or `"DEGREE"`; C147) and `spells` (houses 3 to 11, the
    /// 4th and the 8th by default; C149). Every period reaching into the
    /// window comes back whole in the `sade_sati` and `sade_sati_visits`
    /// sections, the sky searched once for the batch. Null for none
    /// (`03-design/sade-sati.md`). Refusals are named from the record every
    /// binding calls `sadeSati`, as `sadeSati.to`.
    /// `api: nullable example={"from":2460676.5,"to":2464329.0,"reckoning":"SIGN"}`
    pub sade_sati_json: *const c_char,
    /// Every chart read as KP (Krishnamurti Paddhati), as a JSON object,
    /// every member optional: `number`, the querent's horary number 1 to
    /// 249, which casts the cusps from it (C156); `clock`, seconds east of
    /// UT that the civil day lord is the weekday on, this request's own
    /// when absent (C151); and `anyAyanamsha`, true to read a chart whose
    /// zodiac is not Krishnamurti's (C157). Each chart's reading — its
    /// cusps and planets to the sub-sub lord, its significators and the
    /// ruling planets of its moment — comes back in the `kp` section,
    /// under the settings' `kp` group. Null for none, which costs nothing
    /// (`03-design/kp.md`). Refusals are named from the record every
    /// binding calls `kp`, as `kp.number`.
    /// `api: nullable example={"number":74}`
    pub kp_json: *const c_char,
    /// Every chart's essential dignities, as a JSON object, every member
    /// optional: `sectRule` (`"HORIZON"`, the Sun's centre above the true
    /// horizon and the default; `"DAYLIGHT"`, the chart's own sunrise to
    /// sunset; or `"DAY"` or `"NIGHT"` outright; C209), `rules`
    /// (`{terms, triplicities}`: the terms `"PTOLEMAIC_LILLY"`, the
    /// default, `"EGYPTIAN"`, `"PTOLEMAIC_ASHMAND"`, `"CHALDEAN"` or
    /// `{"TABLE": [...]}`, twelve signs of five `{lord, end}` from Aries;
    /// the triplicities `"LILLY"`, the default, or `"PTOLEMY"`; C208) and
    /// `scores` (`house`, `exaltation`, `triplicity`, `term`, `face`,
    /// `detriment`, `fall`, `peregrine`, Lilly's by default). The sect and
    /// what was applied come back in the `dignities` section and the seven
    /// planets in `dignity_planets`. Null for none, which costs nothing
    /// (`03-design/essential-dignities.md`). Refusals are named from the
    /// record every binding calls `dignities`, as `dignities.sectRule`.
    /// `api: nullable example={"sectRule":"HORIZON","rules":{"terms":"EGYPTIAN"}}`
    pub dignities_json: *const c_char,
    /// Every chart's accidental fortitudes beside its essential dignities
    /// (Lilly, p. 115), as a JSON object, every member optional:
    /// `dignities` (the record `dignities_json` takes), `rules`
    /// (`combustionDeg` 8.5, `combustionInSign` true, `beamsDeg` 17,
    /// `cazimiDeg` 17′, `cuspOrbDeg` 5, `starOrbDeg` 5, `partile`
    /// `"SAME_DEGREE"` or `{"WITHIN": {"orbDeg": …}}` (C216), `siege`
    /// `"SAME_SIGN"` or `{"WITHIN": {"spanDeg": …}}` (C215) and
    /// `meanMotionDeg`, seven in the Chaldean order) and `scores` (the
    /// twelve `houses` and each line by name), Lilly's by default. The
    /// essential half comes back in the sections `dignities_json` fills, so
    /// asking for both is refused; the accidental half in `fortitudes`,
    /// `fortitude_houses`, `fortitude_planets` and `fortitude_accidents`.
    /// The houses are Regiomontanus's unless a profile names another
    /// division for the `hellenistic` module. Null for none, which costs
    /// nothing (`03-design/essential-dignities.md` §Accidental fortitudes).
    /// Refusals are named from the record every binding calls
    /// `fortitudes`, as `fortitudes.rules.beamsDeg`.
    /// `api: nullable example={"rules":{"partile":{"WITHIN":{"orbDeg":1}}},"scores":{"regulus":5}}`
    pub fortitudes_json: *const c_char,
    /// Every chart's lots, all fourteen Valens gives, as a JSON object,
    /// every member optional: `sectRule` (the record `dignities_json`
    /// names it in, Valens's `"HORIZON"` by default) and `fortune`, how
    /// the Part of Fortune is taken by night: `"REVERSED_BY_NIGHT"`
    /// (Valens II.22, the default), `"DAY_AND_NIGHT"` (Lilly) or
    /// `"REVERSED_WHILE_MOON_UP"` (Valens III.11, C221). What was applied
    /// comes back in the `lots` section and the fourteen in `lot_places`.
    /// Null for none, which costs nothing
    /// (`03-design/hellenistic-lots.md`). Refusals are named from the
    /// record every binding calls `lots`, as `lots.fortune`.
    /// `api: nullable example={"fortune":"REVERSED_WHILE_MOON_UP"}`
    pub lots_json: *const c_char,
    /// Every chart's considerations before judgement (Lilly, *Christian
    /// Astrology* I.XIX), as a JSON object, every member optional:
    /// `moonLateFromDeg` (27 by default, C229) and `orbsDeg`, the seven
    /// whole orbs in the Chaldean order whose halves make an application
    /// (Lilly's p. 107 by default, C230). The fortitudes they read are
    /// `fortitudes_json`'s, or Lilly's when it is null. The clauses come
    /// back in `considerations`, the Moon's two perfections in
    /// `consideration_perfections` and the orbs applied in
    /// `consideration_orbs`. Null for none, which costs nothing
    /// (`03-design/hellenistic-considerations.md`). Refusals are named from
    /// the record every binding calls `considerations`, as
    /// `considerations.moonLateFromDeg`.
    /// `api: nullable example={"moonLateFromDeg":25}`
    pub considerations_json: *const c_char,
    /// Whether a horary matter is brought to pass (Lilly, *Christian
    /// Astrology* pp. 107–113 and 125–127), as a JSON object:
    /// `querent` and `quesited`, the two significators by key, or
    /// `house`, the house of the matter, whose cusp's lord signifies the
    /// quesited, the querent's being the Ascendant's lord unless named;
    /// and `rules`, every member optional: `orbsDeg` (Lilly's p. 107),
    /// `horizonDays` (unset, until the swifter significator leaves its
    /// sign, C232) and `withinSign` (true: a third planet's contact counts
    /// only before the applier leaves its sign, C234). The houses and dignities it weighs are
    /// `fortitudes_json`'s, or Lilly's when it is null; the timeline is
    /// searched on the ephemeris. The relations come back in
    /// `perfection`, `perfection_impediments`, `perfection_translations`
    /// and `perfection_collections`, and the orbs applied in
    /// `perfection_orbs`. Null for none, which costs nothing
    /// (`03-design/hellenistic-perfection.md`). Refusals are named from
    /// the record every binding calls `perfection`, as
    /// `perfection.quesited`.
    /// `api: nullable example={"house":7}`
    pub perfection_json: *const c_char,
    /// The progressions to read every chart's birth through, as a JSON
    /// object, every field optional but one of `at` and `contacts`: `at`,
    /// the instant of life (a UTC Julian day) the progressed chart and the
    /// direction are read for; `rate` (`{"sky": "DAY", "life": "YEAR"}` by
    /// default; a span is `"DAY"`, `"SYNODIC_MONTH"`, `"SIDEREAL_MONTH"`,
    /// `"YEAR"` or `{"DAYS": n}`); `year` (`"TROPICAL"` by default,
    /// `"JULIAN"`, or Leo's `"NOON_SIDEREAL_TIME"`, C236); `angles` (how
    /// the progressed midheaven moves, `"NAIBOD_RIGHT_ASCENSION"` by
    /// default, C237); `direction` (`"SOLAR"` by default, `"NAIBOD"`,
    /// `"PTOLEMY"` or `{"PER_YEAR": degrees}`); and `contacts`, a window of
    /// life `{from, to}` with the progressed `grahas` (the seven by
    /// default), the radical `points` (the seven and the lagna) and the
    /// `aspects` (Leo's table, p. 48), spelled as `hits_json` spells them.
    /// The progressed chart is founded at the request's place. The answers
    /// come back in `progressions`, `progressed_grahas`, `directed_grahas`
    /// and `progressed_contacts`. Null for none, which costs nothing
    /// (`03-design/western-progressions.md`). Refusals are named from the
    /// record every binding calls `progressions`, as `progressions.year`.
    /// `api: nullable example={"at":2460676.5}`
    pub progressions_json: *const c_char,
    /// Every chart's Western aspect table, as a JSON object, every field
    /// optional: `aspects`, the keys looked for (`"CONJUNCTION"`,
    /// `"SEMI_SEXTILE"`, `"SEMI_SQUARE"`, `"SEXTILE"`, `"SQUARE"`,
    /// `"TRINE"`, `"SESQUIQUADRATE"`, `"QUINCUNX"`, `"OPPOSITION"`; Leo's
    /// nine when left out), and `orbs`, the model: `{"model": "LEO"}` by
    /// default (C240), `{"model": "MOIETIES", "orbs": [{"graha": "SUN",
    /// "orbDeg": 17}, …]}`, or `{"model": "BY_ASPECT", "orbs": [{"aspect":
    /// "TRINE", "orbDeg": 6}, …]}`. The pairs are the chart's planets: the
    /// seven, and the outer three when `TS_CHART_OUTER` placed them. The
    /// answers come back in `western_aspects` and `western_aspect_rows`.
    /// Null for none, which costs nothing (`03-design/western-aspects.md`).
    /// Refusals are named from the record every binding calls
    /// `westernAspects`, as `westernAspects.orbs.orbs`.
    /// `api: nullable example={"aspects":["TRINE","SQUARE"]}`
    pub western_aspects_json: *const c_char,
    /// Every chart's synastry with one partner, as a JSON object:
    /// `partner`, the second birth, `{"instant": jd, "place": {"latitude",
    /// "longitude", "altitude"}, "utcOffsetSeconds"}`, founded once under
    /// the context's settings with the outer planets when
    /// `TS_CHART_OUTER` placed them; and beside it, every field optional,
    /// `aspects` and `orbs` as `western_aspects_json` spells them, `lagna`
    /// (true: each side's lagna is read beside its planets, C242),
    /// `zodiac` (`"TROPICAL"`, the default, or `"CHARTS"`, C241) and
    /// `parallels` (`{"orbDeg": 1}` as `parallels_json` spells it: the
    /// parallels across the two, none when left out) and `antiscia`
    /// (`{"orbs": {"model": "LEO"}}` as `antiscia_json` spells it: the antiscia
    /// across the two, none when left out), `midpoints` (`{"orbDeg": 0.5}`
    /// as `midpoints_json` spells it: the equal distances across the two,
    /// each chart's planets on the partner's pairs and the partner's on
    /// the chart's, none when left out), `composite` (true: each chart's
    /// composite with the partner, C247) and `davison` (true: each chart's
    /// Davison birth with the partner, the chart's read on this request's
    /// clock, C248). Each chart is read against the
    /// partner, the chart's point first. The answers come back in
    /// `synastry`, `synastry_rows`, `synastry_parallel_rows`,
    /// `synastry_antiscion_rows`, `synastry_composites`,
    /// `synastry_composite_rows`, `synastry_davisons`, `synastry_midpoints`
    /// and `synastry_midpoint_rows`. Null for none,
    /// which costs nothing
    /// (`03-design/western-synastry.md`). Refusals are
    /// named from the record every binding calls `synastry`, as
    /// `synastry.partner.place.latitude`.
    /// `api: nullable example={"partner":{"instant":2403113.4993,"place":{"latitude":51.5058,"longitude":-0.1878,"altitude":0}}}`
    pub synastry_json: *const c_char,
    /// Every chart's declinations and the parallels among its planets, as
    /// a JSON object, every field optional: `orbDeg`, how close two
    /// distances from the equator must stand, Leo's 1° by default and at
    /// most 10°. A pair on either side of the equator is a parallel
    /// (C243). The pairs are the chart's planets: the seven, and the outer
    /// three when `TS_CHART_OUTER` placed them. The answers come back in
    /// `declinations`, `declination_rows` and `parallel_rows`. Null for
    /// none, which costs nothing (`03-design/western-declinations.md`).
    /// Refusals are named from the record every binding calls
    /// `parallels`, as `parallels.orbDeg`.
    /// `api: nullable example={"orbDeg":1}`
    pub parallels_json: *const c_char,
    /// Every chart's antiscia, as a JSON object, every field optional:
    /// `orbs`, as `western_aspects_json` spells them, read at the
    /// conjunction, Lilly's moieties by default (C244). Each planet is
    /// reflected about the solstices and the equinoxes from its tropical
    /// longitude, and a pair whose longitudes sum to 180° or 0° within the
    /// orb stands in antiscion or contrantiscion. The planets are the
    /// seven, and the outer three when `TS_CHART_OUTER` placed them; one
    /// the orbs give none is reflected and stands in no pair. The answers
    /// come back in `antiscia`, `antiscion_points` and `antiscion_rows`.
    /// Null for none, which costs nothing
    /// (`03-design/western-antiscia.md`). Refusals are named from the
    /// record every binding calls `antiscia`, as `antiscia.orbs.orbs`.
    /// `api: nullable example={"orbs":{"model":"LEO"}}`
    pub antiscia_json: *const c_char,
    /// Every chart's equal distances, as a JSON object, every field
    /// optional: `orbDeg`, how far from the axis through two planets'
    /// midpoint a third may stand, 0.5° by default (C245) and at most 10°.
    /// A planet stands on the axis when it is equally distant from the
    /// two, on the shorter arc's midpoint or opposite it (C246). The
    /// planets are the seven, and the outer three when `TS_CHART_OUTER`
    /// placed them. The answers come back in `midpoints` and
    /// `midpoint_rows`. Null for none, which costs nothing
    /// (`03-design/western-midpoints.md`). Refusals are named from the
    /// record every binding calls `midpoints`, as `midpoints.orbDeg`.
    /// `api: nullable example={"orbDeg":1}`
    pub midpoints_json: *const c_char,
    /// Every chart's Western houses, as a JSON object, every field
    /// optional: `system`, the division (`"KOCH"`), else the profile's
    /// `houses.module_overrides.western`, else Placidus, the division
    /// Leo's figures are cast in (C249). Each planet is counted by the
    /// cusps alone, and flagged when Leo reads it with the ascendant, up
    /// to the degree that rose one sidereal hour before the birth (C250).
    /// The answers come back in `western_houses`, `western_house_cusps`
    /// and `western_house_planets`. Null for none, which costs nothing
    /// (`03-design/western-houses.md`). Refusals are named from the
    /// record every binding calls `westernHouses`, as
    /// `westernHouses.system`.
    /// `api: nullable example={"system":"KOCH"}`
    pub western_houses_json: *const c_char,
    /// Every chart's harmonic chart, as a JSON object: `number`, the
    /// harmonic, a whole number from 1 to 360 every longitude is
    /// multiplied by, and `orbDeg`, how close two points meet in it, 12°
    /// by default (C252), at most 30°. The planets, the ascendant and the
    /// midheaven are multiplied in the chart's own zodiac (C253), each in
    /// its equal house from the harmonic ascendant (C254). The answers
    /// come back in `harmonics`, `harmonic_points` and `harmonic_rows`.
    /// Null for none, which costs nothing
    /// (`03-design/western-harmonics.md`). Refusals are named from the
    /// record every binding calls `harmonic`, as `harmonic.number`.
    /// `api: nullable example={"number":9}`
    pub harmonic_json: *const c_char,
    /// Every chart matched with one partner's birth by the Ashta Koota of
    /// *Muhurta Chintamani* VI.21–34, the ten considerations and the Kuja
    /// dosha, as a JSON object: `partner`,
    /// `{"instant": jd, "place": {"latitude", "longitude", "altitude"},
    /// "utcOffsetSeconds"}`, founded once under the context's sidereal
    /// profile; `partnerRole`, `"BRIDE"` or `"GROOM"`, every chart standing
    /// on the other side; and `rules`, every field optional: `equalVarna`
    /// (`WHOLE` or `HALF`), `devaBride` (`FOUR` or `THREE`),
    /// `bhakootLift` (`ANY_ONE` or `GARGA`) and `nadiDosha` (`ANY` or
    /// `MIDDLE_ONLY`); `porutham`, the ten considerations of
    /// *Kalaprakasika* XIII, every field optional: `twoSignStar`
    /// (`GROOM_EARLIER` or `BRIDE_FIRST_SIGN`), `deerghaBeyond`
    /// (`THIRTEENTH` or `SEVENTH`) and `lordsFriendship` (`MUTUAL` or
    /// `ONE_WAY`); and `kuja`, the Kuja dosha of *Manasagari*, every field
    /// optional: `houses` (`MANASAGARI` or `WITH_SECOND`) and `from`
    /// (`LAGNA` or `LAGNA_MOON_VENUS`). The answers come back in
    /// `matchings`, `matching_kootas`, `poruthams`, `porutham_rows` and
    /// `kujas`, with every dosha the three report gathered in
    /// `marriage_doshas` and `marriage_dosha_rows`. Null for none,
    /// which costs nothing
    /// (`03-design/matching.md`). Refusals are named from the record every
    /// binding calls `matching`, as `matching.partnerRole`.
    /// `api: nullable example={"partner":{"instant":2447892.5,"place":{"latitude":27.7172,"longitude":85.324,"altitude":1400}},"partnerRole":"BRIDE"}`
    pub matching_json: *const c_char,
    /// Every chart read as a prashna, the chart of the moment a question
    /// was asked, as *Shatpanchashika* and Tajika Nilakanthi print it, as
    /// a JSON object, every member optional: `question` (`house`, the
    /// matter's house 1 to 12, which the verdict's I.3 clauses and the
    /// Tajika links read; `number`, the querent's 1 to 108, read only by
    /// the baseline engine's unsourced rule, C340) and `rules` (`pisces`,
    /// `timing`, `mook`, `moon` `{kshina}` and `score`, the texts' own by
    /// default). A prashna reads the seven's Shadbala, so asking for one
    /// asks for the `shadbala` sections too. Each chart's reading comes
    /// back in the `prashna` section. Null for none, which costs nothing
    /// (`03-design/prashna.md`). Refusals are named from the record every
    /// binding calls `prashna`, as `prashna.question.house`.
    /// `api: nullable example={"question":{"house":7},"rules":{"mook":"MOON_HOUSE"}}`
    pub prashna_json: *const c_char,
    /// Every chart's remedies, as BPHS, *Laghu Parashari* and
    /// *Yājñavalkya* prescribe them, as a JSON object, every member
    /// optional: `at` (the Julian day UTC whose running Vimśottarī
    /// mahādaśā and antardaśā name subjects and bring the antardaśā's
    /// printed śānti; none reads no daśā) and `rules` (`functional`
    /// `{scheme}`, `shanti` `{rik}`, `devata` `{sunWithKetu}`, the texts'
    /// own by default). A record with `at` asks for the Vimśottarī daśā
    /// too. Each chart's remedies come back in the `remedies` section.
    /// Null for none, which costs nothing (`03-design/remedies.md`).
    /// Refusals are named from the record every binding calls `remedies`,
    /// as `remedies.rules.devata`.
    /// `api: nullable example={"at":2460676.5,"rules":{"shanti":{"rik":"YAJNAVALKYA"}}}`
    pub remedies_json: *const c_char,
    /// Every chart read as a birth time to rectify, the chart's instant
    /// the time on record, as a JSON object, every member optional:
    /// `purify` (`minutes` either side of the chart's instant, more than
    /// none and at most 1080, and `rules`, the purifier of BPHS ch. 2
    /// vv. 67–78), `conception` (the pranapada's house, the nisheka and
    /// the conception Moon, with its rules), `circumstance` (`facts` the
    /// family remembers, `fatherPresent`, `presentation`, `oil`, `wick`
    /// and `attendants`, and `rules`, *Brihat Jataka* ch. V) and `baseline`
    /// (the baseline engine's unsourced cascade: `uncertaintyMinutes` 1
    /// to 720, `accuracy`, dated `events`, `sex`, `coverage` and `dasha`).
    /// Each chart's readings come back in the `rectification` section,
    /// one member for each reading asked. Null for none, which costs
    /// nothing (`03-design/rectification.md`). Refusals are named from
    /// the record every binding calls `rectification`, as
    /// `rectification.purify.minutes`.
    /// `api: nullable example={"purify":{"minutes":30},"circumstance":{"facts":{"fatherPresent":false}}}`
    pub rectification_json: *const c_char,
    /// Every chart read as Lal Kitab reads it (the 1952 edition), as a JSON
    /// object, every member optional: `cycle` (`{planet, year}`, where the
    /// 35-year cycle starts, the book's general table from Saturn in the
    /// first year when left out), `year` (a year of life from 1, the year
    /// from birth to the first birthday, to read its ruler, its thirds and
    /// its annual teva) and `varshphal` (`{rows}`, the 120-year list the
    /// annual teva is read from, which the SDK does not ship and checks
    /// row by row). Each chart's reading comes back in the `lalkitab`
    /// section. Null for none, which costs nothing
    /// (`03-design/lalkitab.md`). Refusals are named from the record every
    /// binding calls `lalkitab`, as `lalkitab.cycle.year`.
    /// `api: nullable example={"cycle":{"planet":"VENUS","year":17},"year":30}`
    pub lalkitab_json: *const c_char,
}

// **The handshake, which this struct carried and nothing read.**
// `struct_size` is documented as "`sizeof(ts_chart_request)` as the caller compiled
// it", and the entry point below dereferenced the pointer raw: a caller
// compiled against an older header passed a shorter struct and the
// library read past it, which is undefined behaviour rather than the
// `SCHEMA_VERSION` refusal the field exists to give. Eleven of the
// thirteen boundary structs with the field were registered here; these
// two were not, and they are the two biggest requests.
// `check-lints`' `handshake-is-checked` holds the class now.
c_struct!(TsChartRequest);
