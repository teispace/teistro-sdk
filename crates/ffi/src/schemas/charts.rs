//! The chart and naam blob schemas: every section a chart call writes,
//! compiled with the chart area (`03-design/wasm-profiles.md`).

use teistro_idl::model::{BlobSchema, ColumnDef, Scalar, SectionSchema};

use super::{
    CHARTS, DAY_SHAPE, MATCHING_KOOTAS_SHAPE, MATCHINGS_SHAPE, NAAM, PORUTHAM_ROWS_SHAPE,
    PORUTHAMS_SHAPE, content_hashes_section, day_section,
};

/// One row per graha per chart, charts outermost.
#[must_use]
fn chart_grahas_section(id: u32) -> SectionSchema {
    chart_placed_section(
        id,
        "grahas",
        "One row per graha per chart, charts outermost: row `i * graha_count + j` is chart `i`, graha `j`, grahas in the catalogue's order. `house_*` is the bhava for \"which house is it in\"; `placement_*` is the chart's chalit, which is a different question and often a different answer.",
    )
}

/// Uranus, Neptune and Pluto, where the request asked for them, in the
/// shape of the nine (`03-design/western-outer-planets.md`).
#[must_use]
fn chart_outer_section(id: u32) -> SectionSchema {
    chart_placed_section(
        id,
        "outer",
        "Uranus, Neptune and Pluto beside the nine, **the same number of rows a chart**, charts outermost and each chart's in the catalogue's order, the columns `grahas` has: three a chart when `TS_CHART_OUTER` asked for them, and empty when it did not. A reader divides the rows by `chart_count`. They are placed as the nine are, in the chart's zodiac and from its centre.",
    )
}

/// A section of placed bodies, a row a body: the columns `grahas` and
/// `outer` share.
#[must_use]
fn chart_placed_section(id: u32, name: &str, doc: &str) -> SectionSchema {
    SectionSchema::columns(
        id,
        name,
        doc,
        vec![
            ColumnDef::new("graha", Scalar::U16, "Which graha.").of_enum("Graha"),
            ColumnDef::new(
                "longitude_deg",
                Scalar::F64,
                "Its longitude in the chart's zodiac, degrees.",
            ),
            ColumnDef::new(
                "tropical_deg",
                Scalar::F64,
                "Its longitude in the tropical zodiac, degrees.",
            ),
            ColumnDef::new("latitude_deg", Scalar::F64, "Its latitude, degrees."),
            ColumnDef::new(
                "distance_au",
                Scalar::F64,
                "Its distance in astronomical units; zero for a point that has none.",
            ),
            ColumnDef::new(
                "speed_deg_per_day",
                Scalar::F64,
                "Its longitude speed, degrees per day; negative when retrograde.",
            ),
            ColumnDef::new(
                "house_bhava",
                Scalar::U8,
                "The bhava it stands in, 1 to 12.",
            ),
            ColumnDef::new(
                "house_method",
                Scalar::U16,
                "The house system that produced that bhava.",
            )
            .of_enum("HouseSystem"),
            ColumnDef::new(
                "house_through",
                Scalar::F64,
                "How far through the bhava it stands, 0 to 1.",
            ),
            ColumnDef::new(
                "house_from_madhya_deg",
                Scalar::F64,
                "Its distance from the bhava's madhya, degrees.",
            ),
            ColumnDef::new(
                "placement_bhava",
                Scalar::U8,
                "The bhava of the chart's chalit it stands in, 1 to 12.",
            ),
            ColumnDef::new(
                "placement_method",
                Scalar::U16,
                "The house system that produced the chalit.",
            )
            .of_enum("HouseSystem"),
            ColumnDef::new(
                "placement_through",
                Scalar::F64,
                "How far through that bhava it stands, 0 to 1.",
            ),
            ColumnDef::new(
                "placement_from_madhya_deg",
                Scalar::F64,
                "Its distance from that bhava's madhya, degrees.",
            ),
        ],
    )
}

/// Where in its day the moment falls, in the reckonings the settings named.
#[must_use]
fn chart_timing_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "timing",
        "Where in its day each moment falls, in the reckonings the settings named: one row per chart.",
        vec![
            ColumnDef::new(
                "ghati",
                Scalar::U8,
                "The ishtakaal's ghatis since sunrise, 0 to 59.",
            ),
            ColumnDef::new("pala", Scalar::U8, "Its palas, 0 to 59."),
            ColumnDef::new("vipala", Scalar::U8, "Its vipalas, 0 to 59."),
            ColumnDef::new(
                "ghati_reckoning",
                Scalar::U8,
                "How the ghatis were measured.",
            )
            .of_enum("TsGhatiReckoning"),
            ColumnDef::new(
                "hora_number",
                Scalar::U8,
                "Which hora of the day holds the instant, 1 to 24.",
            ),
            ColumnDef::new("hora_lord", Scalar::U16, "The graha that rules it.").of_enum("Graha"),
            ColumnDef::new(
                "hora_start",
                Scalar::F64,
                "When that hora began, as a Julian day (UTC).",
            ),
            ColumnDef::new(
                "hora_end",
                Scalar::F64,
                "When it ends, as a Julian day (UTC).",
            ),
            ColumnDef::new("hora_reckoning", Scalar::U8, "How the horas were measured.")
                .of_enum("TsHoraReckoning"),
        ],
    )
}

/// One row per chart: what changes from instant to instant.
///
/// What a batch decides once — the place, the kind, the frame, the
/// house systems, the solar model, the completion steps — is written
/// once, in the sections around this one. What an instant decides is
/// here. Named for the casting rather than for the chart, so the
/// decoded type does not stutter (`ChartCast`, beside the positions
/// blob's `PositionsCells`).
#[must_use]
fn chart_cast_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "cast",
        "One row per chart, in the order the instants were asked for.",
        vec![
            ColumnDef::new(
                "instant",
                Scalar::F64,
                "The instant the chart is cast for, as a Julian day (UTC).",
            ),
            ColumnDef::new(
                "lagna_deg",
                Scalar::F64,
                "The lagna at the instant, in the chart's zodiac, degrees.",
            ),
            ColumnDef::new(
                "day_lagna_deg",
                Scalar::F64,
                "The lagna at the sunrise that opened the day, degrees.",
            ),
            ColumnDef::new(
                "ayanamsha_offset_deg",
                Scalar::F64,
                "The ayanamsha applied at this instant, degrees; zero for a tropical chart.",
            ),
            ColumnDef::new(
                "day_part",
                Scalar::U8,
                "Which arc of its day the instant falls in.",
            )
            .of_enum("TsDayPart"),
            ColumnDef::new(
                "day_elapsed",
                Scalar::F64,
                "How far through that arc the instant is, 0 to 1.",
            ),
            ColumnDef::new(
                "point_count",
                Scalar::U32,
                "How many rows of the `points` section belong to this chart. Zero when the derived points were not asked for.\n\nRagged for the reason `aspect_count` is: a chart's points depend on what its day allows — Saturn's eighth needs an arc to divide, which a polar day has not — so the count is a per-chart fact and not a batch one.",
            ),
            ColumnDef::new(
                "aspect_count",
                Scalar::U32,
                "How many rows of the `aspects` section belong to this chart. Zero when the aspects were not asked for.\n\nA **per-chart count and not one for the batch**, because a chart's drishti are a function of where the bodies stand rather than of how many there are: two charts of the same nine grahas at one place hold 47 relations and 40. The rows are concatenated charts outermost and a reader prefix-sums these counts, which is the panchanga blob's own rule for a ragged list.",
            ),
            ColumnDef::new(
                "pravesha_count",
                Scalar::U32,
                "How many rows of the `praveshas` section belong to this chart. Zero when no annual charts were asked for.\n\nRagged for a reason of its own: the request settles how many returns are wanted, and an ephemeris that ends first settles how many there are (`03-design/annual-chart.md`). Fewer than asked for is the answer, so a reader takes this count and never the number it requested.",
            ),
            ColumnDef::new(
                "natal_saham_count",
                Scalar::U32,
                "How many rows of the `natal_sahams` section belong to this chart: the sahams `varsha_json.sahams` asked for, 0 to 41.",
            ),
            ColumnDef::new(
                "hit_count",
                Scalar::U32,
                "How many rows of the `hits` section belong to this chart. Zero when no hit list was asked for.\n\nRagged because a chart's aspects are its own: the sky's ingresses and stations are every chart's alike, but how often a transit crosses a natal point depends on where the point stands.",
            ),
            ColumnDef::new(
                "sade_sati_visit_count",
                Scalar::U32,
                "How many rows of the `sade_sati_visits` section belong to this chart. Zero when no Sade Sati was asked for.\n\nRagged because a chart's periods are its own: where Saturn crosses into them depends on where the natal Moon stands, and how often it steps back out depends on where its stations fall.",
            ),
        ],
    )
}

/// Which house system produced each set of bhavas, and which bound each is read against.
#[must_use]
fn chart_readings_section(id: u32) -> SectionSchema {
    SectionSchema::fixed(
        id,
        "readings",
        "Which house system produced each set of bhavas, and which bound each is read against.",
        vec![
            ColumnDef::new(
                "houses_method",
                Scalar::U16,
                "The system the houses were computed under.",
            )
            .of_enum("HouseSystem"),
            ColumnDef::new(
                "houses_source",
                Scalar::U16,
                "The system its cusps came from.",
            )
            .of_enum("HouseSystem"),
            ColumnDef::new(
                "houses_reading",
                Scalar::U8,
                "Which bound the houses are read against.",
            )
            .of_enum("TsReading"),
            ColumnDef::new(
                "chalit_method",
                Scalar::U16,
                "The system the chalit was computed under.",
            )
            .of_enum("HouseSystem"),
            ColumnDef::new(
                "chalit_source",
                Scalar::U16,
                "The system its cusps came from.",
            )
            .of_enum("HouseSystem"),
            ColumnDef::new(
                "chalit_reading",
                Scalar::U8,
                "Which bound the chalit is read against.",
            )
            .of_enum("TsReading"),
        ],
    )
}

/// A founded chart: where every graha stands, in which bhava under both
/// readings, in which zodiac, on which day, at what time of that day.
///
/// The one columnar group is the grahas — everything else is one value
/// per chart — which is why a `columns` section is right for them and
/// wrong for the rest (`03-design/chart-at-the-boundary.md` §2).
#[must_use]
pub fn charts() -> BlobSchema {
    BlobSchema {
        name: CHARTS.to_string(),
        id: 3,
        doc: "A batch of founded charts at one place: the grahas placed, the bhavas under both readings, the zodiac, the day and the timing. Every per-chart section runs charts outermost, and a batch of one is the ordinary case.".to_string(),
        sections: vec![
            chart_summary_section(1),
            chart_cast_section(2),
            chart_grahas_section(3),
            chart_readings_section(4),
            chart_cusps_section(
                5,
                "houses",
                "The twelve bhavas for \"which house is it in\", charts outermost: row `i * 12 + j` is chart `i`, bhava `j`, first to twelfth.",
            ),
            chart_cusps_section(
                6,
                "chalit",
                "The twelve bhavas of each chart's chalit, the same shape as `houses`.",
            ),
            chart_zodiac_section(7),
            day_section(8).of_shape(DAY_SHAPE),
            chart_timing_section(9),
            SectionSchema::bytes(
                10,
                "model",
                "UTF-8 text: the solar model that reckoned the days, as it describes itself.",
            ),
            SectionSchema::bytes(
                11,
                "steps",
                "UTF-8 JSON: an array of strings, the completion steps applied in order, each `name:Implementation`. The positions blob carries the same steps as objects and spells the implementation differently (`PASS_THROUGH` against `PassThrough`); which of the two every blob should use is an open question (`03-design/chart-at-the-boundary.md` §8).",
            ),
            SectionSchema::bytes(
                12,
                "provenance_json",
                "UTF-8 JSON: the provenance envelope of the result, canonical.",
            ),
            chart_vargas_section(13),
            chart_varga_grahas_section(14),
            chart_aspects_section(15),
            chart_drishti_table_section(16),
            chart_points_section(17),
            chart_bhavas_section(18),
            chart_states_section(19),
        ]
        .into_iter()
        .chain(chart_text_sections(20))
        .chain([
            chart_dashas_section(23),
            chart_dasha_periods_section(24),
            chart_ashtakavarga_section(25),
            chart_ashtakavarga_bindus_section(26),
            chart_sarvashtakavarga_section(27),
            chart_vimshopaka_section(28),
            chart_shadbala_section(29),
            chart_bhava_bala_section(30),
            chart_vaiseshikamsa_section(31),
            chart_dasha_phala_section(32),
            chart_rules_section(33),
            chart_plans_section(34),
        ])
        .chain(chart_annual_sections())
        .chain([chart_content_hashes_section(50)])
        .chain(chart_jaimini_sections(51))
        .chain(chart_gochar_sections(53))
        .chain([chart_hits_section(56)])
        .chain(chart_sade_sati_sections(57))
        .chain([chart_kp_section(59)])
        .chain(chart_dignity_sections(60))
        .chain(chart_fortitude_sections(63))
        .chain(chart_lot_sections(67))
        .chain(chart_consideration_sections(69))
        .chain(chart_perfection_sections(72))
        .chain(chart_progression_sections(77))
        .chain([chart_outer_section(81)])
        .chain(chart_western_aspect_sections(82))
        .chain(chart_synastry_sections(84))
        .chain(chart_declination_sections(86))
        .chain(chart_synastry_parallel_sections(89))
        .chain(chart_antiscia_sections(91))
        .chain(chart_synastry_antiscia_sections(94))
        .chain(chart_midpoint_sections(96))
        .chain(chart_composite_sections(98))
        .chain(chart_synastry_midpoint_sections(101))
        .chain(chart_western_house_sections(103))
        .chain(chart_harmonic_sections(108))
        .chain(chart_matching_sections(111))
        .chain(chart_marriage_dosha_sections(116))
        .chain(chart_avakahada_sections(118))
        .chain([
            chart_prashna_section(120),
            chart_remedies_section(121),
            chart_rectification_section(122),
        ])
        .collect(),
    }
}

/// The zodiac a chart batch is measured in, once for the batch.
fn chart_zodiac_section(id: u32) -> SectionSchema {
    SectionSchema::fixed(
        id,
        "zodiac",
        "The zodiac the batch is measured in. The offset itself moves with the instant, so it is a column of `charts` rather than a field here.",
        vec![
            ColumnDef::new(
                "frame_bits",
                Scalar::U32,
                "The frame the positions were asked for, packed as the port packs it.",
            ),
            ColumnDef::new(
                "ayanamsha_kind",
                Scalar::U8,
                "0 for none, 1 for a catalogued ayanamsha, 2 for one the settings define.",
            ),
            ColumnDef::new(
                "ayanamsha",
                Scalar::U16,
                "Which catalogued ayanamsha, when the kind is 1.",
            )
            .of_enum("Ayanamsha"),
        ],
    )
}

/// What a chart batch decided once: where, what kind, and how many of what.
fn chart_summary_section(id: u32) -> SectionSchema {
    SectionSchema::fixed(
        id,
        "summary",
        "What the batch decided once: where, what kind, and how many of what.",
        vec![
            ColumnDef::new("kind", Scalar::U16, "What kind of chart these are.")
                .of_enum("ChartKind"),
            ColumnDef::new(
                "chart_count",
                Scalar::U32,
                "How many charts the batch holds, and how many rows the `cast`, `day` and `timing` sections each hold.",
            ),
            ColumnDef::new(
                "graha_count",
                Scalar::U32,
                "How many grahas each chart holds; the `grahas` section holds `chart_count * graha_count` rows.",
            ),
            ColumnDef::new(
                "varga_count",
                Scalar::U32,
                "How many divisional charts were asked for, in the order asked; zero when none were. The `vargas` section holds `chart_count * varga_count` rows and `varga_grahas` holds `chart_count * varga_count * graha_count`.",
            ),
            ColumnDef::new(
                "dasha_count",
                Scalar::U32,
                "How many dashas were asked for, in the order asked; zero when none were. The `dashas` section holds `chart_count * dasha_count` rows.",
            ),
            ColumnDef::new(
                "latitude_deg",
                Scalar::F64,
                "The place's latitude, degrees north.",
            ),
            ColumnDef::new(
                "longitude_deg",
                Scalar::F64,
                "The place's longitude, degrees east.",
            ),
            ColumnDef::new("altitude_m", Scalar::F64, "The place's altitude, metres."),
        ],
    )
}

/// The three sections a chart carries as text rather than columns, from
/// `first`: the combustion table, the drawings and their SVGs.
fn chart_text_sections(first: u32) -> [SectionSchema; 3] {
    [
        SectionSchema::bytes(
            first,
            "combustion_orbs",
            "UTF-8 text: the combustion table the settings named, which every `burning` above was judged against. Empty when the states were not asked for.",
        ),
        SectionSchema::bytes(
            first + 1,
            "drawings",
            "UTF-8 JSON, canonical: an array with one entry per chart, each the array of that chart's drawings in the order asked for, every drawing `{varga, placed}` exactly as the document schema describes `Drawing` (`03-design/chart-geometry.md`). Empty when no drawings were asked for.",
        ),
        SectionSchema::bytes(
            first + 2,
            "svgs",
            "UTF-8 JSON, canonical: an array with one entry per chart, each the array of that chart's drawings written as SVG strings, in the order asked for, in the request's theme and the context's locale (`03-design/render-svg.md`). Empty when no theme was given.",
        ),
    ]
}

/// Jaimini's significators from `first`: a row a chart, and a row a graha.
fn chart_jaimini_sections(first: u32) -> [SectionSchema; 2] {
    [
        chart_jaimini_section(first),
        chart_jaimini_grahas_section(first + 1),
    ]
}

/// The transits read against every chart, from `first`: the readings and
/// their grahas (`03-design/gochar.md`).
fn chart_gochar_sections(first: u32) -> [SectionSchema; 3] {
    [
        chart_gochar_section(first),
        chart_gochar_grahas_section(first + 1),
        chart_gochar_ashtakavarga_section(first + 2),
    ]
}

/// Every chart's Sade Sati, from `first`: a row a chart, and a row a visit
/// (`03-design/sade-sati.md`).
fn chart_sade_sati_sections(first: u32) -> [SectionSchema; 2] {
    [
        SectionSchema::columns(
            first,
            "sade_sati",
            "What every chart's Sade Sati was reckoned from, a row a chart in the `cast` section's order. Empty when `sade_sati_json` asked for none, and then `sade_sati_visits` is too.",
            vec![
                ColumnDef::new(
                    "reference",
                    Scalar::U16,
                    "The sign the houses are counted from: the natal Moon's, or the lagna's when `counted_from` says so.",
                )
                .of_enum("Rashi"),
                ColumnDef::new(
                    "counted_from",
                    Scalar::U8,
                    "Which natal point `reference` is, `sade_sati_json.countedFrom` (C139).",
                )
                .of_enum("TsGocharFrom"),
                ColumnDef::new(
                    "reckoning",
                    Scalar::U8,
                    "What the houses were reckoned in, `sade_sati_json.reckoning` (C147).",
                )
                .of_enum("TsReckoning"),
            ],
        ),
        SectionSchema::columns(
            first + 1,
            "sade_sati_visits",
            "Every stay of Saturn's in a house of a period reaching into the window, concatenated in the `cast` section's order and **ragged** by its `sade_sati_visit_count`. A chart's rows are its periods in turn, numbered by `period`: its Sade Satis first (houses 12, 1 and 2), then its smaller spells (C149), each group in time order; within a Sade Sati its phases' rows in the order 12, 1, 2; and each house's visits in time order, a retrograde re-entry a visit of its own (C148). A period is **whole**, however far its bounds fall outside the window. The sky is searched **once for the batch**.",
            vec![
                ColumnDef::new(
                    "period",
                    Scalar::U16,
                    "Which of the chart's periods the visit belongs to, counted from 0: the rows of one Sade Sati, or of one smaller spell, share it.",
                ),
                ColumnDef::new(
                    "house",
                    Scalar::U8,
                    "The house Saturn stays in, 1 to 12 from `sade_sati.reference`: 12, 1 or 2 in a Sade Sati (rising, peak and setting), otherwise a smaller spell's.",
                ),
                ColumnDef::new(
                    "from",
                    Scalar::F64,
                    "When Saturn entered the house, a UTC Julian day; NaN when that is before the ephemeris's coverage.",
                ),
                ColumnDef::new(
                    "to",
                    Scalar::F64,
                    "When Saturn left it, a UTC Julian day, the visit half-open; NaN when that is after the ephemeris's coverage.",
                ),
            ],
        ),
    ]
}

/// Every chart read as KP (`03-design/kp.md`).
fn chart_kp_section(id: u32) -> SectionSchema {
    SectionSchema::bytes(
        id,
        "kp",
        "UTF-8 JSON, canonical: an array with one entry per chart, each `{chart, significators, ruling}` (`03-design/kp.md`). `chart` is `{system, cusps, planets}`, every cusp `{house, longitude, lords}` and every planet `{graha, longitude, retrograde, house, lords}`, a longitude in **nanoarcseconds** of the sidereal zodiac, exact, and `lords` `{sign, star, sub, subSub}`, each level below the sign `{lord, span: {start, end}}` in nanoarcseconds, half-open. For a horary number the cusps are the number's (C156). `significators` is `{houses, nodes}`: twelve `{house, inOccupantsStars, occupants, inLordsStar, lord, conjoined, aspected, intercepted}` in Reader VI's order, and each node's `{node, conjoined, starLord, aspecting, signLord}` (C155). `ruling` is `{rulers, rules}`, each ruler `{graha, reasons, retrograde, rejectedBy, rejectedBySub}` and each reason `{kind}`, an `AGENT`'s with the ruler it stands for, `of`, and `by`; the ruling planets are the moment's own, even for a horary number. Empty when `kp_json` asked for none.",
    )
}

/// Every chart's essential dignities (`03-design/essential-dignities.md`):
/// what was applied a chart, the seven planets a chart, then its
/// receptions.
fn chart_dignity_sections(first: u32) -> [SectionSchema; 3] {
    let score = |name: &'static str, what: &str| {
        ColumnDef::new(
            name,
            Scalar::I8,
            &format!("What {what} scores, `dignities_json.scores`, Lilly's (p. 115) by default."),
        )
    };
    let flag = |name: &'static str, what: &str| {
        ColumnDef::new(name, Scalar::U8, &format!("1 when {what}, else 0."))
    };
    [
        SectionSchema::columns(
            first,
            "dignities",
            "Every chart's sect and the rules its dignities were read under, a row a chart in the `cast` section's order. Filled from `dignities_json`, or from `fortitudes_json`'s essential half; empty when neither asked, and then `dignity_planets` is too.",
            vec![
                ColumnDef::new(
                    "sect",
                    Scalar::U8,
                    "Whether the chart is of the day or of the night, as `sect_rule` reads it.",
                )
                .of_enum("TsSect"),
                ColumnDef::new(
                    "sect_rule",
                    Scalar::U8,
                    "How the sect was read, `dignities_json.sectRule` (C209).",
                )
                .of_enum("TsSectRule"),
                ColumnDef::new(
                    "terms",
                    Scalar::U8,
                    "The system of terms, `dignities_json.rules.terms` (C208); `TABLE` is the table the request gave.",
                )
                .of_enum("TsTerms"),
                ColumnDef::new(
                    "triplicities",
                    Scalar::U8,
                    "Who rules each triplicity, `dignities_json.rules.triplicities`.",
                )
                .of_enum("TsTriplicities"),
                score("score_house", "a planet in its own house"),
                score("score_exaltation", "a planet in its exaltation"),
                score("score_triplicity", "a planet ruling its sign's triplicity"),
                score("score_term", "a planet in its own term"),
                score("score_face", "a planet in its own face"),
                score("score_detriment", "a planet in its detriment"),
                score("score_fall", "a planet in its fall"),
                score("score_peregrine", "a planet in none of its five dignities"),
                ColumnDef::new(
                    "reception_count",
                    Scalar::U8,
                    "How many rows of the `dignity_receptions` section belong to this chart.\n\nRagged because which pairs receive each other depends on where each planet stands.",
                ),
            ],
        ),
        SectionSchema::columns(
            first + 1,
            "dignity_planets",
            "The seven planets' essential dignities, **seven rows a chart** in the `cast` section's order, each chart's in the Chaldean order: Saturn, Jupiter, Mars, the Sun, Venus, Mercury, the Moon. A planet is peregrine when none of the first five flags is set. Empty when neither `dignities_json` nor `fortitudes_json` asked.",
            vec![
                ColumnDef::new("planet", Scalar::U16, "The planet.").of_enum("Graha"),
                ColumnDef::new(
                    "longitude",
                    Scalar::F64,
                    "Where it stands, in degrees of the chart's zodiac.",
                ),
                flag("house", "the sign is its house"),
                flag("exaltation", "the sign is its exaltation"),
                flag(
                    "triplicity",
                    "it rules the sign's triplicity in a chart of the sect",
                ),
                flag("term", "the degree lies in its own term"),
                flag("face", "the degree lies in its own face"),
                flag("detriment", "the sign is opposite its house"),
                flag("fall", "the sign is opposite its exaltation"),
                ColumnDef::new(
                    "score",
                    Scalar::I16,
                    "Its flags read by the `dignities` row's scores.",
                ),
                ColumnDef::new(
                    "reception",
                    Scalar::I16,
                    "What Lilly's table adds for mutual reception (p. 115): `score_house` when it is received by house, `score_exaltation` when by exaltation, nothing for a mixed reception or one by a lesser dignity (C210). Kept apart from `score`, since a planet in reception is still peregrine; a total is `score + reception`.",
                ),
            ],
        ),
        dignity_receptions_section(first + 2),
    ]
}

/// The `fortitudes` section's columns: the accidental sky, the rules and
/// a score a line.
fn fortitude_chart_columns() -> Vec<ColumnDef> {
    let degrees = |name: &'static str, what: &str| ColumnDef::new(name, Scalar::F64, what);
    let mut chart = vec![
        ColumnDef::new(
            "houses",
            Scalar::U16,
            "The division the houses were counted in: Regiomontanus, Lilly's, unless a profile names another for the `hellenistic` module.",
        )
        .of_enum("HouseSystem"),
        degrees(
            "ascendant",
            "The ascendant, from the chart's angles, in degrees of the chart's zodiac: whole-sign and equal houses do not put it on a cusp.",
        ),
        degrees(
            "midheaven",
            "The midheaven, from the chart's angles: only a quadrant division puts it on the tenth cusp.",
        ),
        degrees("north_node", "The North Node's longitude, in degrees of the chart's zodiac."),
        degrees("regulus", "Regulus's apparent longitude of date, in degrees of the chart's zodiac."),
        degrees("spica", "Spica's, likewise."),
        degrees("algol", "Algol's, likewise."),
        degrees(
            "combustion_orb",
            "Combust within this many degrees of the Sun, `fortitudes_json.rules.combustionDeg`.",
        ),
        ColumnDef::new(
            "combustion_in_sign",
            Scalar::U8,
            "1 when combustion also asks for the Sun's sign (C211), `fortitudes_json.rules.combustionInSign`, else 0.",
        ),
        degrees(
            "beams_orb",
            "Under the beams within this many degrees, `fortitudes_json.rules.beamsDeg` (C212).",
        ),
        degrees(
            "cazimi_orb",
            "Cazimi within this many degrees, `fortitudes_json.rules.cazimiDeg`.",
        ),
        degrees(
            "cusp_orb",
            "A planet this near the next cusp is in its house (p. 33), `fortitudes_json.rules.cuspOrbDeg` (C214).",
        ),
        degrees(
            "star_orb",
            "With a star within this many degrees, `fortitudes_json.rules.starOrbDeg`.",
        ),
        ColumnDef::new(
            "partile",
            Scalar::U8,
            "When two planets are in partile aspect, `fortitudes_json.rules.partile` (C216).",
        )
        .of_enum("TsPartile"),
        degrees("partile_orb", "The orb of `WITHIN`; 0 for `SAME_DEGREE`."),
        ColumnDef::new(
            "siege",
            Scalar::U8,
            "When a planet is besieged, `fortitudes_json.rules.siege` (C215).",
        )
        .of_enum("TsSiege"),
        degrees("siege_span", "The span of `WITHIN`; 0 for `SAME_SIGN`."),
        ColumnDef::new(
            "almuten_place",
            Scalar::U8,
            "What of a place its almuten's dignities are counted from, `fortitudes_json.almuten.place` (C218).",
        )
        .of_enum("TsPlaceReading"),
        ColumnDef::new(
            "almuten_fortune",
            Scalar::U8,
            "How the Part of Fortune is taken by night, `fortitudes_json.almuten.fortune` (C220).",
        )
        .of_enum("TsFortuneRule"),
        degrees(
            "fortune",
            "The Part of Fortune, one of the five places `fortitude_planets`' `places` sums over.",
        ),
    ];
    chart.extend(crate::chart::ACCIDENTAL_LINES.map(|line| {
        ColumnDef::new(
            &format!("score_{line}"),
            Scalar::I8,
            &format!(
                "What the line `{line}` scores, `fortitudes_json.scores`, Lilly's (p. 115) by default."
            ),
        )
    }));
    chart
}

/// The seven in the Chaldean order: each almuten column's name, and the
/// planet as its description names it.
const CHALDEAN_NAMES: [(&str, &str); 7] = [
    ("saturn", "Saturn"),
    ("jupiter", "Jupiter"),
    ("mars", "Mars"),
    ("sun", "the Sun"),
    ("venus", "Venus"),
    ("mercury", "Mercury"),
    ("moon", "the Moon"),
];

/// Every chart's accidental fortitudes (`03-design/essential-dignities.md`
/// §Accidental fortitudes): what was applied a chart, its twelve houses,
/// the seven planets, then their accidents.
fn chart_fortitude_sections(first: u32) -> [SectionSchema; 4] {
    let degrees = |name: &'static str, what: &str| ColumnDef::new(name, Scalar::F64, what);
    let empty = "Empty when `fortitudes_json` asked for none.";
    [
        SectionSchema::columns(
            first,
            "fortitudes",
            &format!(
                "Every chart's accidental sky and the rules and scores its fortitudes were read under, a row a chart in the `cast` section's order. The essential half is in `dignities`. {empty}"
            ),
            fortitude_chart_columns(),
        ),
        SectionSchema::columns(
            first + 1,
            "fortitude_houses",
            &format!(
                "Every chart's twelve houses, **twelve rows a chart** in the `cast` section's order, the first to the twelfth. {empty}"
            ),
            vec![
                degrees(
                    "cusp",
                    "Where the house begins, in degrees of the chart's zodiac.",
                ),
                ColumnDef::new(
                    "score",
                    Scalar::I8,
                    "What a planet in the house scores, `fortitudes_json.scores.houses`.",
                ),
            ]
            .into_iter()
            .chain(CHALDEAN_NAMES.map(|(column, planet)| {
                ColumnDef::new(
                    &format!("almuten_{column}"),
                    Scalar::I16,
                    &format!(
                        "The dignities {planet} holds at the cusp, read under `almuten_place`: the house's almuten is the planet with the most."
                    ),
                )
            }))
            .collect(),
        ),
        SectionSchema::columns(
            first + 2,
            "fortitude_planets",
            &format!(
                "The seven planets' accidental fortitudes, **seven rows a chart** in the `cast` section's order, each chart's in the Chaldean order. Lilly's net is `dignity_planets`' `score + reception` and this row's `fortitude - debility`, and the planet with the greatest is his almuten of the figure. {empty}"
            ),
            vec![
                ColumnDef::new("planet", Scalar::U16, "The planet.").of_enum("Graha"),
                degrees(
                    "speed",
                    "Its daily motion in longitude, in degrees; negative when retrograde.",
                ),
                degrees(
                    "mean_motion",
                    "The mean daily motion its speed is judged swift or slow against, `fortitudes_json.rules.meanMotionDeg`.",
                ),
                ColumnDef::new(
                    "house",
                    Scalar::U8,
                    "Its house, 1 to 12, under the five-degree rule.",
                ),
                ColumnDef::new(
                    "fortitude",
                    Scalar::I16,
                    "The sum of its positive lines, its house's included.",
                ),
                ColumnDef::new(
                    "debility",
                    Scalar::I16,
                    "The sum of its negative lines, its house's included, as a positive number, the way Lilly prints it.",
                ),
                ColumnDef::new(
                    "places",
                    Scalar::I16,
                    "Its essential dignities summed over the ascendant, midheaven, Sun, Moon and Part of Fortune: Chapter CV's almuten is the planet with the most. Lilly's almuten of the figure is the greatest net.",
                ),
                ColumnDef::new(
                    "accident_count",
                    Scalar::U8,
                    "How many rows of the `fortitude_accidents` section belong to this planet.\n\nRagged because which lines a planet meets depends on its sky.",
                ),
            ],
        ),
        SectionSchema::columns(
            first + 3,
            "fortitude_accidents",
            &format!(
                "Every planet's accidental lines beyond its house, concatenated in `fortitude_planets`' order and **ragged** by its `accident_count`, each planet's in `TsAccident`'s order. {empty}"
            ),
            vec![
                ColumnDef::new("accident", Scalar::U8, "The line it meets.").of_enum("TsAccident"),
                ColumnDef::new(
                    "points",
                    Scalar::I8,
                    "What the line scores for this planet: orientality scores Saturn, Jupiter and Mars one way and Venus and Mercury the other.",
                ),
            ],
        ),
    ]
}

/// Every chart's lots (`03-design/hellenistic-lots.md`): what was applied
/// a chart, then the fourteen.
fn chart_lot_sections(first: u32) -> [SectionSchema; 2] {
    let empty = "Empty when `lots_json` asked for none.";
    [
        SectionSchema::columns(
            first,
            "lots",
            &format!(
                "Every chart's sect and the rules its lots were read under, a row a chart in the `cast` section's order. {empty}"
            ),
            vec![
                ColumnDef::new(
                    "sect",
                    Scalar::U8,
                    "Whether the chart is of the day or of the night, as `sect_rule` reads it: a night chart takes each lot's night arc.",
                )
                .of_enum("TsSect"),
                ColumnDef::new(
                    "sect_rule",
                    Scalar::U8,
                    "How the sect was read, `lots_json.sectRule` (C209).",
                )
                .of_enum("TsSectRule"),
                ColumnDef::new(
                    "fortune",
                    Scalar::U8,
                    "How the Part of Fortune is taken by night, `lots_json.fortune` (C221).",
                )
                .of_enum("TsFortuneRule"),
                ColumnDef::new(
                    "fortune_reversed",
                    Scalar::U8,
                    "1 when Fortune was counted from the Moon to the Sun, and Daimon from the Sun to the Moon; 0 otherwise.",
                ),
            ],
        ),
        SectionSchema::columns(
            first + 1,
            "lot_places",
            &format!(
                "Every chart's fourteen lots, **fourteen rows a chart** in the `cast` section's order, each chart's in `TsLot`'s order. {empty}"
            ),
            vec![
                ColumnDef::new("lot", Scalar::U8, "Which.").of_enum("TsLot"),
                ColumnDef::new(
                    "longitude_deg",
                    Scalar::F64,
                    "Where it fell, degrees of the chart's zodiac in [0, 360).",
                ),
                ColumnDef::new("sign", Scalar::U16, "The sign it fell in.").of_enum("Rashi"),
                ColumnDef::new(
                    "lord",
                    Scalar::U16,
                    "That sign's lord, the lot's ruler, which Valens reads it by.",
                )
                .of_enum("Graha"),
                ColumnDef::new(
                    "house",
                    Scalar::U8,
                    "Its place, 1 to 12, counted in whole signs from the ascendant's sign.",
                ),
            ],
        ),
    ]
}

/// Every chart's considerations before judgement
/// (`03-design/hellenistic-considerations.md`): the clauses a chart, the
/// Moon's two perfections a chart, then the seven orbs applied.
fn chart_consideration_sections(first: u32) -> [SectionSchema; 3] {
    let empty = "Empty when `considerations_json` asked for none.";
    [
        consideration_clauses_section(first, empty),
        SectionSchema::columns(
            first + 1,
            "consideration_perfections",
            &format!(
                "The Moon's course, **two rows a chart** in the `cast` section's order: the first Ptolemaic aspect she perfects with one of the other six before she leaves her sign, then the first already within the two planets' moieties of orb (C230). A row with `present` 0 says she is void by that reading. {empty}"
            ),
            vec![
                ColumnDef::new(
                    "present",
                    Scalar::U8,
                    "1 when there is such a perfection; 0 when she is void by this reading, and then the other columns are 0 and NaN.",
                ),
                ColumnDef::new("planet", Scalar::U16, "The planet she perfects it with.")
                    .of_enum("Graha"),
                ColumnDef::new("aspect", Scalar::U8, "The aspect.").of_enum("TsPtolemaicAspect"),
                ColumnDef::new(
                    "days",
                    Scalar::F64,
                    "Days until it is exact, at the motions of the moment.",
                ),
                ColumnDef::new(
                    "gap_deg",
                    Scalar::F64,
                    "How far it is from exact now, degrees.",
                ),
            ],
        ),
        SectionSchema::columns(
            first + 2,
            "consideration_orbs",
            &format!(
                "The orbs the moieties were taken from, **seven rows a chart** in the `cast` section's order, each chart's in the Chaldean order, `considerations_json.orbsDeg`. {empty}"
            ),
            vec![ColumnDef::new(
                "orb_deg",
                Scalar::F64,
                "The planet's whole orb, degrees; half of it counts toward an application.",
            )],
        ),
    ]
}

/// What a dignity bit set holds, for a column carrying one.
const DIGNITY_BITS: &str = "as a bit set in `EssentialDignity`'s order: bit 0 house, 1 exaltation, 2 triplicity, 3 term, 4 face, 5 detriment, 6 fall";

/// A column naming a graha by catalogue id.
fn graha_column(name: &str, doc: &str) -> ColumnDef {
    ColumnDef::new(name, Scalar::U16, doc).of_enum("Graha")
}

/// A column naming a Ptolemaic aspect.
fn aspect_column(name: &str, doc: &str) -> ColumnDef {
    ColumnDef::new(name, Scalar::U8, doc).of_enum("TsPtolemaicAspect")
}

/// A column holding a planet's essential dignities as a bit set.
fn dignity_column(name: &str, doc: &str) -> ColumnDef {
    ColumnDef::new(name, Scalar::U8, &format!("{doc}, {DIGNITY_BITS}."))
}

/// The five sections a perfection crosses as, from `first`: a row a chart,
/// its impediments, translations and collections ragged under it, and the
/// orbs it was read with (`03-design/hellenistic-perfection.md`).
fn chart_perfection_sections(first: u32) -> [SectionSchema; 5] {
    let empty = "Empty when `perfection_json` asked for none.";
    [
        perfection_section(first, empty),
        SectionSchema::columns(
            first + 1,
            "perfection_impediments",
            &format!(
                "What stops or hinders the significators' application (pp. 110–113), concatenated in the `cast` section's order and **ragged** by `perfection.impediment_count`, each chart's in time order. {empty}"
            ),
            vec![
                ColumnDef::new("kind", Scalar::U8, "What it is.").of_enum("TsImpedimentKind"),
                graha_column(
                    "significator",
                    "The significator it falls on: the one a prohibiting third reaches, the one that reaches a frustrating third, or the one that stations.",
                ),
                ColumnDef::new(
                    "third_present",
                    Scalar::U8,
                    "1 when a third planet takes part; 0 for a refranation, and then `third` is 0.",
                ),
                graha_column("third", "The third planet."),
                aspect_column(
                    "aspect",
                    "The aspect the third perfects, or the one refrained from.",
                ),
                ColumnDef::new(
                    "days",
                    Scalar::F64,
                    "Days until it happens: the contact, or the station.",
                ),
            ],
        ),
        SectionSchema::columns(
            first + 2,
            "perfection_translations",
            &format!(
                "Every translation of light between the significators (p. 111), concatenated in the `cast` section's order and **ragged** by `perfection.translation_count`. {empty}"
            ),
            vec![
                graha_column("translator", "The lighter planet carrying the light."),
                graha_column("from", "The significator it separates from."),
                graha_column("to", "The significator it applies to next."),
                aspect_column("separating_aspect", "The aspect it separates from."),
                ColumnDef::new(
                    "separating_past_deg",
                    Scalar::F64,
                    "How far past exact that separation is, degrees.",
                ),
                aspect_column("aspect", "The aspect it applies by."),
                ColumnDef::new("days", Scalar::F64, "Days until that application is exact."),
                dignity_column(
                    "received",
                    "The dignities of `from` the translator stands in: how it is received, by house, triplicity or term (p. 126)",
                ),
            ],
        ),
        perfection_collections_section(first + 3, empty),
        SectionSchema::columns(
            first + 4,
            "perfection_orbs",
            &format!(
                "The orbs the moieties were taken from, **seven rows a chart** in the `cast` section's order, each chart's in the Chaldean order, `perfection_json.rules.orbsDeg`. {empty}"
            ),
            vec![ColumnDef::new(
                "orb_deg",
                Scalar::F64,
                "The planet's whole orb, degrees; half of it counts toward an application.",
            )],
        ),
    ]
}

/// The two sections the Western aspects cross as, from `first`: a row a
/// chart saying how many it holds, and the aspects ragged under it
/// (`03-design/western-aspects.md`).
fn chart_western_aspect_sections(first: u32) -> [SectionSchema; 2] {
    let empty = "Empty when `western_aspects_json` asked for none.";
    [
        SectionSchema::columns(
            first,
            "western_aspects",
            &format!(
                "Every chart's Western aspect table, a row a chart in the `cast` section's order: how many rows of `western_aspect_rows` are its. {empty}"
            ),
            vec![ColumnDef::new(
                "count",
                Scalar::U32,
                "How many aspects the chart's planets hold under the record's orbs; the chart's rows follow the earlier charts' in `western_aspect_rows`.",
            )],
        ),
        SectionSchema::columns(
            first + 1,
            "western_aspect_rows",
            &format!(
                "Every chart's aspects, concatenated in the `cast` section's order and **ragged** by `western_aspects.count`, each chart's closest first: a pair of its planets (the seven, and the outer three when `TS_CHART_OUTER` placed them) at one of the record's aspects, inside the orb its model allows (Leo's by aspect by default, C240). {empty}"
            ),
            vec![
                graha_column(
                    "first",
                    "The first planet of the pair, in the catalogue's order.",
                ),
                graha_column("second", "The second."),
            ]
            .into_iter()
            .chain(western_aspect_measures())
            .chain([ColumnDef::new(
                "applying",
                Scalar::U8,
                "1 when the gap is closing on the aspect, 0 when it is leaving it.",
            )])
            .collect(),
        ),
    ]
}

/// The four columns a Western aspect row measures, whichever two points
/// stand at it: the aspect, the gap, the distance from exact and the orb.
fn western_aspect_measures() -> [ColumnDef; 4] {
    [
        ColumnDef::new("aspect", Scalar::U8, "Which aspect.").of_enum("TsWesternAspect"),
        ColumnDef::new(
            "apart_deg",
            Scalar::F64,
            "How far apart the two stand, degrees, 0 to 180.",
        ),
        ColumnDef::new(
            "from_exact_deg",
            Scalar::F64,
            "How far from exact, degrees; the smaller, the stronger.",
        ),
        ColumnDef::new(
            "orb_deg",
            Scalar::F64,
            "The orb the model allowed this pair at this aspect, degrees.",
        ),
    ]
}

/// The two columns a natal point crosses as, named for `side`: whether it
/// is the lagna, and which graha when it is not.
fn point_columns(side: &str, whose: &str) -> [ColumnDef; 2] {
    [
        ColumnDef::new(
            &format!("{side}_lagna"),
            Scalar::U8,
            &format!("1 when {whose} point is the lagna, 0 when it is a graha."),
        ),
        ColumnDef::new(
            &format!("{side}_graha"),
            Scalar::U16,
            &format!("Which graha {whose} point is (a `Graha` id); 0 for the lagna."),
        ),
    ]
}

/// The two sections a synastry with a partner crosses as, from `first`: a
/// row a chart, and the contacts ragged under it
/// (`03-design/western-synastry.md`).
fn chart_synastry_sections(first: u32) -> [SectionSchema; 2] {
    let empty = "Empty when `synastry_json` asked for none.";
    [
        SectionSchema::columns(
            first,
            "synastry",
            &format!(
                "Every chart's synastry with the record's partner, a row a chart in the `cast` section's order: how many rows of `synastry_rows` are its. {empty}"
            ),
            vec![ColumnDef::new(
                "count",
                Scalar::U32,
                "How many aspects stand between the chart's points and the partner's; the chart's rows follow the earlier charts' in `synastry_rows`.",
            )],
        ),
        SectionSchema::columns(
            first + 1,
            "synastry_rows",
            &format!(
                "Every chart's synastry, concatenated in the `cast` section's order and **ragged** by `synastry.count`, each chart's closest first: a point of the chart (its planets, the outer three when `TS_CHART_OUTER` placed them, and its lagna unless the record leaves it out) against a point of the partner's at one of the record's aspects, inside the orb its model allows (Leo's by default, C240; the lagna stands as a planet, C242), compared in the tropical zodiac unless the record asks for each chart's own (C241). {empty}"
            ),
            point_columns("first", "the chart's")
                .into_iter()
                .chain(point_columns("second", "the partner's"))
                .chain(western_aspect_measures())
                .collect(),
        ),
    ]
}

/// The two sections the parallels across a chart and the synastry's
/// partner cross as, from `first`: a row a chart, and the parallels ragged
/// under it (`03-design/western-declinations.md`). Their own row, not a
/// column of `synastry`'s, so a record that asked for none reads apart
/// from one whose charts hold none.
fn chart_synastry_parallel_sections(first: u32) -> [SectionSchema; 2] {
    let empty = "Empty when `synastry_json` asked for no `parallels`.";
    [
        SectionSchema::columns(
            first,
            "synastry_parallels",
            &format!(
                "Every chart's parallels with the synastry's partner, a row a chart in the `cast` section's order: how many rows of `synastry_parallel_rows` are its. {empty}"
            ),
            vec![ColumnDef::new(
                "count",
                Scalar::U32,
                "How many parallels stand between the chart's points and the partner's; the chart's rows follow the earlier charts' in `synastry_parallel_rows`.",
            )],
        ),
        SectionSchema::columns(
            first + 1,
            "synastry_parallel_rows",
            &format!(
                "Every chart's parallels with the synastry's partner, concatenated in the `cast` section's order and **ragged** by `synastry_parallels.count`, each chart's closest first: a point of the chart (its planets and, unless the record leaves it out, its lagna) the same distance from the equator as a point of the partner's, within the orb of the record's `parallels` (Leo's 1° by default), on either side of it (C243). {empty}"
            ),
            point_columns("first", "the chart's")
            .into_iter()
            .chain(point_columns("second", "the partner's"))
            .chain([
                ColumnDef::new(
                    "contrary",
                    Scalar::U8,
                    "1 when the two stand on opposite sides of the equator, the contra-parallel; 0 when on one side.",
                ),
                ColumnDef::new(
                    "apart_deg",
                    Scalar::F64,
                    "How far apart their distances from the equator are, degrees.",
                ),
                ColumnDef::new("orb_deg", Scalar::F64, "The orb the record allowed, degrees."),
            ])
            .collect(),
        ),
    ]
}

/// The three sections the antiscia cross as, from `first`: a row a chart,
/// each planet's reflections, and the pairs, both ragged under the row
/// (`03-design/western-antiscia.md`).
fn chart_antiscia_sections(first: u32) -> [SectionSchema; 3] {
    let empty = "Empty when `antiscia_json` asked for none.";
    let degrees = |name: &str, doc: &str| ColumnDef::new(name, Scalar::F64, doc);
    [
        SectionSchema::columns(
            first,
            "antiscia",
            &format!(
                "Every chart's antiscia, a row a chart in the `cast` section's order: how many rows of `antiscion_points` and of `antiscion_rows` are its. {empty}"
            ),
            vec![
                ColumnDef::new(
                    "point_count",
                    Scalar::U32,
                    "How many planets' reflections are the chart's in `antiscion_points`.",
                ),
                ColumnDef::new(
                    "pair_count",
                    Scalar::U32,
                    "How many pairs are the chart's in `antiscion_rows`.",
                ),
                ColumnDef::new(
                    "cusp_count",
                    Scalar::U32,
                    "How many reflections upon a cusp are the chart's in `antiscion_cusp_rows`; 0 when the record asked for no `cusps`.",
                ),
                ColumnDef::new(
                    "cusp_system",
                    Scalar::U16,
                    "The division the cusps were read in, Lilly's Regiomontanus unless the record's `cusps` named another, or the one a polar policy fell back to; `0xFFFF` when the record asked for no `cusps`.",
                )
                .of_enum("HouseSystem"),
            ],
        ),
        SectionSchema::columns(
            first + 1,
            "antiscion_points",
            &format!(
                "Every chart's planets reflected, concatenated in the `cast` section's order and **ragged** by `antiscia.point_count`: the seven, and the outer three when `TS_CHART_OUTER` placed them, in the catalogue's order, each from its tropical longitude (Lilly, *Christian Astrology*, pp. 90–92). {empty}"
            ),
            vec![
                graha_column("graha", "Which planet."),
                degrees(
                    "antiscion_deg",
                    "Its antiscion, the reflection about the solstices: 180° less its tropical longitude, degrees.",
                ),
                degrees(
                    "contrantiscion_deg",
                    "Its contrantiscion, the reflection about the equinoxes: 360° less it, degrees.",
                ),
                ColumnDef::new(
                    "paired",
                    Scalar::U8,
                    "1 when the record's orbs give the planet one, so it can stand in a pair; 0 when they give it none, as Lilly's moieties give the outer three.",
                ),
            ],
        ),
        SectionSchema::columns(
            first + 2,
            "antiscion_rows",
            &format!(
                "Every chart's pairs in antiscion, concatenated in the `cast` section's order and **ragged** by `antiscia.pair_count`, each chart's closest first: two planets whose longitudes sum to 180°, or to 0° for the contrantiscion, within the record's orb read at the conjunction (Lilly's moieties by default, C244). {empty}"
            ),
            antiscion_row_columns(
                "The first planet of the pair, in the catalogue's order.",
                "The second.",
            ),
        ),
    ]
}

/// The columns of a pair in antiscion, one chart's own or across two.
fn antiscion_row_columns(first: &str, second: &str) -> Vec<ColumnDef> {
    let degrees = |name: &str, doc: &str| ColumnDef::new(name, Scalar::F64, doc);
    vec![
        graha_column("first", first),
        graha_column("second", second),
        ColumnDef::new(
            "contrary",
            Scalar::U8,
            "1 for the contrantiscion, the reflection about the equinoxes; 0 for the antiscion.",
        ),
        degrees(
            "apart_deg",
            "How far the one's reflection stands from the other, degrees.",
        ),
        degrees("orb_deg", "The orb the record allowed the pair, degrees."),
    ]
}

/// The two sections the antiscia across a chart and the synastry's
/// partner cross as, from `first`: a row a chart, and the pairs ragged
/// under it (`03-design/western-antiscia.md`).
fn chart_synastry_antiscia_sections(first: u32) -> [SectionSchema; 2] {
    let empty = "Empty when `synastry_json` asked for no `antiscia`.";
    [
        SectionSchema::columns(
            first,
            "synastry_antiscia",
            &format!(
                "Every chart's antiscia with the synastry's partner, a row a chart in the `cast` section's order: how many rows of `synastry_antiscion_rows` are its. {empty}"
            ),
            vec![ColumnDef::new(
                "count",
                Scalar::U32,
                "How many pairs stand in antiscion between the chart's planets and the partner's; the chart's rows follow the earlier charts' in `synastry_antiscion_rows`.",
            )],
        ),
        SectionSchema::columns(
            first + 1,
            "synastry_antiscion_rows",
            &format!(
                "Every chart's pairs in antiscion with the synastry's partner, concatenated in the `cast` section's order and **ragged** by `synastry_antiscia.count`, each chart's closest first: a planet of the chart (the seven, and the outer three when `TS_CHART_OUTER` placed them) and one of the partner's whose tropical longitudes sum to 180°, or to 0° for the contrantiscion, within the orb of the record's `antiscia` read at the conjunction (Lilly's moieties by default, C244). {empty}"
            ),
            antiscion_row_columns("The chart's planet.", "The partner's planet."),
        ),
    ]
}

/// The two sections the equal distances cross as, from `first`: a row a
/// chart, and the rows ragged under it (`03-design/western-midpoints.md`).
fn chart_midpoint_sections(first: u32) -> [SectionSchema; 2] {
    let empty = "Empty when `midpoints_json` asked for none.";
    [
        SectionSchema::columns(
            first,
            "midpoints",
            &format!(
                "Every chart's equal distances, a row a chart in the `cast` section's order: how many rows of `midpoint_rows` are its. {empty}"
            ),
            vec![ColumnDef::new(
                "count",
                Scalar::U32,
                "How many planets stand equally distant from two others; the chart's rows follow the earlier charts' in `midpoint_rows`.",
            )],
        ),
        SectionSchema::columns(
            first + 1,
            "midpoint_rows",
            &format!(
                "Every chart's equal distances, concatenated in the `cast` section's order and **ragged** by `midpoints.count`, each chart's closest first: a planet (the seven, and the outer three when `TS_CHART_OUTER` placed them) within the record's orb of the axis through two others' midpoint, 0.5° by default (C245), on the shorter arc's midpoint or opposite it (C246; Leo, *How to Judge a Nativity*, pp. 47–48). {empty}"
            ),
            midpoint_row_columns(
                "The first planet of the pair, in the catalogue's order.",
                "The planet equally distant from the two.",
                false,
            ),
        ),
    ]
}

/// The columns of an equal distance, one chart's own or, with
/// `partners_pair`, across a synastry.
fn midpoint_row_columns(first: &str, middle: &str, across: bool) -> Vec<ColumnDef> {
    let degrees = |name: &str, doc: &str| ColumnDef::new(name, Scalar::F64, doc);
    let mut columns = vec![
        graha_column("first", first),
        graha_column("second", "The second."),
        graha_column("middle", middle),
    ];
    if across {
        columns.push(ColumnDef::new(
            "partners_pair",
            Scalar::U8,
            "1 when the pair is the partner's and the planet between it the chart's; 0 when the pair is the chart's and the planet between the partner's.",
        ));
    }
    columns.extend([
        ColumnDef::new(
            "far",
            Scalar::U8,
            "1 when it stands opposite the midpoint of the pair's shorter arc, on the longer arc's midpoint; 0 on the shorter's.",
        ),
        degrees(
            "distance_deg",
            "How far it stands from each of the two, the mean of the two arcs, degrees.",
        ),
        degrees(
            "from_axis_deg",
            "How far it stands from the nearer point of the axis, degrees: half what its two distances differ by.",
        ),
        degrees("orb_deg", "The orb the record allowed, degrees."),
    ]);
    columns
}

/// The two sections the equal distances across a chart and the synastry's
/// partner cross as, from `first`: a row a chart, and the rows ragged
/// under it (`03-design/western-midpoints.md`, decision 9).
fn chart_synastry_midpoint_sections(first: u32) -> [SectionSchema; 2] {
    let empty = "Empty when `synastry_json` asked for no `midpoints`.";
    [
        SectionSchema::columns(
            first,
            "synastry_midpoints",
            &format!(
                "Every chart's equal distances with the synastry's partner, a row a chart in the `cast` section's order: how many rows of `synastry_midpoint_rows` are its. {empty}"
            ),
            vec![ColumnDef::new(
                "count",
                Scalar::U32,
                "How many equal distances stand across the chart and the partner's; the chart's rows follow the earlier charts' in `synastry_midpoint_rows`.",
            )],
        ),
        SectionSchema::columns(
            first + 1,
            "synastry_midpoint_rows",
            &format!(
                "Every chart's equal distances with the synastry's partner, concatenated in the `cast` section's order and **ragged** by `synastry_midpoints.count`, each chart's closest first: a planet of one chart within the record's orb of the axis through the midpoint of two of the other's (0.5° by default, C245), on the shorter arc's midpoint or opposite it (C246), in the record's zodiac; the planets are the seven, and the outer three when `TS_CHART_OUTER` placed them. {empty}"
            ),
            midpoint_row_columns(
                "The first planet of the pair, in its chart's order.",
                "The planet of the other chart equally distant from the two.",
                true,
            ),
        ),
    ]
}

/// The three sections a synastry's composites and Davison births cross
/// as, from `first`: a composite a chart, its planets ragged under it, and
/// a Davison birth a chart (`03-design/western-composites.md`).
fn chart_composite_sections(first: u32) -> [SectionSchema; 3] {
    let empty = |field: &str| format!("Empty when `synastry_json` asked for no `{field}`.");
    let degrees = |name: &str, doc: &str| ColumnDef::new(name, Scalar::F64, doc);
    [
        SectionSchema::columns(
            first,
            "synastry_composites",
            &format!(
                "Every chart's composite with the synastry's partner, a row a chart in the `cast` section's order: its angles, each the near midpoint of the two charts', and how many rows of `synastry_composite_rows` are its, in the record's zodiac (C247). {}",
                empty("composite")
            ),
            vec![
                degrees(
                    "lagna_deg",
                    "The composite lagna, degrees: the near midpoint of the two lagnas, turned by 180° when `lagna_turned` says.",
                ),
                degrees(
                    "midheaven_deg",
                    "The composite midheaven, degrees: the near midpoint of the two midheavens.",
                ),
                ColumnDef::new(
                    "lagna_turned",
                    Scalar::U8,
                    "1 when the near midpoint of the two lagnas stood before the midheaven and was turned by 180° to stand after it, as a lagna does; 0 otherwise.",
                ),
                ColumnDef::new(
                    "count",
                    Scalar::U32,
                    "How many planets the composite places; the chart's rows follow the earlier charts' in `synastry_composite_rows`.",
                ),
                ColumnDef::new(
                    "cusp_count",
                    Scalar::U8,
                    "12 when both charts carry cusps in the `western` module's division, whose near midpoints the composite's cusps are in `synastry_composite_cusps`; 0 where the profile's polar policy refuses the division at either birthplace.",
                ),
            ],
        ),
        SectionSchema::columns(
            first + 1,
            "synastry_composite_rows",
            &format!(
                "Every chart's composite planets, concatenated in the `cast` section's order and **ragged** by `synastry_composites.count`, in the chart's order: each planet (the seven, and the outer three when `TS_CHART_OUTER` placed them) at the near midpoint of its places in the chart and the partner's, moving at the mean of its two speeds (Townley; Astrolog). {}",
                empty("composite")
            ),
            vec![
                graha_column("graha", "The planet."),
                degrees(
                    "longitude_deg",
                    "Its composite longitude, degrees, in the record's zodiac.",
                ),
                degrees(
                    "speed_deg_per_day",
                    "The mean of its two speeds, degrees a day; negative when retrograde.",
                ),
            ],
        ),
        SectionSchema::columns(
            first + 2,
            "synastry_davisons",
            &format!(
                "Every chart's Davison birth with the synastry's partner, a row a chart in the `cast` section's order (C248): the mean of the two instants, of the two latitudes and altitudes, and of the two longitudes the shorter way round, and the mean of the two clocks, the chart's read on the request's. Found it with any chart request, as a birth is. {}",
                empty("davison")
            ),
            vec![
                degrees(
                    "instant",
                    "The mean instant, a Julian day on the UTC scale.",
                ),
                degrees("latitude_deg", "The mean latitude, degrees north."),
                degrees(
                    "longitude_deg",
                    "The mean longitude the shorter way round, degrees east.",
                ),
                degrees("altitude_m", "The mean altitude, metres."),
                ColumnDef::new(
                    "utc_offset_seconds",
                    Scalar::I32,
                    "The mean of the two clocks, seconds east of UTC: it names only the civil day.",
                ),
            ],
        ),
    ]
}

/// The sections a Western chart's houses add, from `first`: a row a chart,
/// its twelve cusps and its planets' houses, then the composite's cusps
/// and the antiscia upon a cusp, which read them
/// (`03-design/western-houses.md`).
fn chart_western_house_sections(first: u32) -> [SectionSchema; 5] {
    let empty = "Empty when `western_houses_json` asked for none.";
    let degrees = |name: &str, doc: &str| ColumnDef::new(name, Scalar::F64, doc);
    [
        SectionSchema::columns(
            first,
            "western_houses",
            &format!(
                "Every chart's Western houses, a row a chart in the `cast` section's order, in the chart's zodiac: the division, the ascendant, the degree that rose one sidereal hour before the birth, and how many rows of `western_house_planets` are its. {empty}"
            ),
            vec![
                ColumnDef::new(
                    "system",
                    Scalar::U16,
                    "The division the cusps are of: the record's `system`, else the profile's `houses.module_overrides.western`, else Placidus (C249); or the one a polar policy fell back to.",
                )
                .of_enum("HouseSystem"),
                degrees(
                    "ascendant_deg",
                    "The ascendant, degrees: the first cusp in every quadrant division.",
                ),
                degrees(
                    "reach_deg",
                    "The degree that rose one sidereal hour before the birth, degrees: the limit of the ascendant's reach (Leo, p. 90; C250).",
                ),
                ColumnDef::new(
                    "planet_count",
                    Scalar::U32,
                    "How many planets are counted; the chart's rows follow the earlier charts' in `western_house_planets`.",
                ),
            ],
        ),
        SectionSchema::columns(
            first + 1,
            "western_house_cusps",
            &format!(
                "Every chart's twelve cusps, charts outermost: row `i * 12 + j` is chart `i`, cusp `j`, first to twelfth. {empty}"
            ),
            vec![degrees("cusp_deg", "The cusp, degrees of the chart's zodiac.")],
        ),
        SectionSchema::columns(
            first + 2,
            "western_house_planets",
            &format!(
                "Every chart's planets counted in its houses, concatenated in the `cast` section's order and **ragged** by `western_houses.planet_count`, in the catalogue's order: the seven, and the outer three when `TS_CHART_OUTER` placed them. {empty}"
            ),
            vec![
                graha_column("graha", "Which planet."),
                ColumnDef::new(
                    "house",
                    Scalar::U8,
                    "The house whose cusp it has passed and whose next cusp it has not, 1 to 12.",
                ),
                ColumnDef::new(
                    "with_ascendant",
                    Scalar::U8,
                    "1 when Leo reads it with the ascendant: in the first house, or above the ascendant no further than `reach_deg`; 0 otherwise. The house is never moved for it.",
                ),
            ],
        ),
        SectionSchema::columns(
            first + 3,
            "synastry_composite_cusps",
            "Every chart's composite cusps, concatenated in the `cast` section's order and **ragged** by `synastry_composites.cusp_count`, first to twelfth: each the near midpoint of the two charts' same cusp, turned by 180° when more than 90° from the midheaven plus 30° a house from the tenth (Astrolog; C247), in the record's zodiac. Empty when `synastry_json` asked for no `composite`.",
            vec![degrees("cusp_deg", "The cusp, degrees.")],
        ),
        SectionSchema::columns(
            first + 4,
            "antiscion_cusp_rows",
            "Every chart's reflections upon a cusp, concatenated in the `cast` section's order and **ragged** by `antiscia.cusp_count`, in the planets' order and then the houses': a planet's antiscion or contrantiscion in the cusp's own sign and whole degree, \"the very degree\" (Lilly, p. 165; C251). Empty when `antiscia_json` asked for no `cusps`.",
            vec![
                graha_column("graha", "Which planet."),
                ColumnDef::new(
                    "house",
                    Scalar::U8,
                    "The house whose cusp its reflection falls on, 1 to 12.",
                ),
                ColumnDef::new(
                    "contrary",
                    Scalar::U8,
                    "1 for the contrantiscion, the reflection about the equinoxes; 0 for the antiscion.",
                ),
            ],
        ),
    ]
}

/// The three sections a chart's harmonic crosses as, from `first`: a row a
/// chart, its points and the points meeting in it, both ragged under the
/// row (`03-design/western-harmonics.md`).
fn chart_harmonic_sections(first: u32) -> [SectionSchema; 3] {
    let empty = "Empty when `harmonic_json` asked for none.";
    let angle = |name: &str, whose: &str| {
        ColumnDef::new(
            name,
            Scalar::U8,
            &format!(
                "Which {whose} it is: 0 for a planet, named in the graha column beside it; 1 for the ascendant and 2 for the midheaven."
            ),
        )
    };
    let graha = |name: &str| {
        graha_column(
            name,
            "The planet when the angle beside it is 0; 0 for an angle.",
        )
    };
    [
        SectionSchema::columns(
            first,
            "harmonics",
            &format!(
                "Every chart's harmonic chart, a row a chart in the `cast` section's order: which harmonic, and how many rows of `harmonic_points` and `harmonic_rows` are its. {empty}"
            ),
            vec![
                ColumnDef::new(
                    "number",
                    Scalar::U16,
                    "Which harmonic, 1 to 360: the number every longitude is multiplied by.",
                ),
                ColumnDef::new(
                    "point_count",
                    Scalar::U32,
                    "How many points the harmonic chart places; the chart's rows follow the earlier charts' in `harmonic_points`.",
                ),
                ColumnDef::new(
                    "row_count",
                    Scalar::U32,
                    "How many pairs meet in it; the chart's rows follow the earlier charts' in `harmonic_rows`.",
                ),
            ],
        ),
        SectionSchema::columns(
            first + 1,
            "harmonic_points",
            &format!(
                "Every chart's harmonic points, concatenated in the `cast` section's order and **ragged** by `harmonics.point_count`: the planets the chart places in the catalogue's order, then the ascendant and the midheaven, each at its longitude multiplied in the chart's zodiac (Addey; C253). {empty}"
            ),
            vec![
                angle("angle", "point"),
                graha("graha"),
                ColumnDef::new(
                    "longitude_deg",
                    Scalar::F64,
                    "Its longitude multiplied by the harmonic, degrees in `[0, 360)`.",
                ),
                ColumnDef::new(
                    "house",
                    Scalar::U8,
                    "Its equal house from the harmonic ascendant, 1 to 12 (C254).",
                ),
            ],
        ),
        SectionSchema::columns(
            first + 2,
            "harmonic_rows",
            &format!(
                "Every chart's pairs meeting in its harmonic chart, concatenated in the `cast` section's order and **ragged** by `harmonics.row_count`, closest first: the two points within the orb of each other there (C252). {empty}"
            ),
            vec![
                angle("first_angle", "earlier point"),
                graha("first_graha"),
                angle("second_angle", "later point"),
                graha("second_graha"),
                ColumnDef::new(
                    "apart_deg",
                    Scalar::F64,
                    "How far apart they stand in the harmonic chart, degrees.",
                ),
                ColumnDef::new(
                    "multiple",
                    Scalar::U16,
                    "Which multiple of the harmonic's aspect they stand at in the chart itself: k for k × 360° / n, 0 to half the harmonic.",
                ),
                ColumnDef::new(
                    "orb_deg",
                    Scalar::F64,
                    "The orb the record allowed, degrees.",
                ),
            ],
        ),
    ]
}

/// The Bhakoot's columns in `matchings`: how far apart the signs stand,
/// the dosha, its five exceptions as clauses and whether they lift it
/// (VI.31–33).
/// The gana dosha and its lift, after the two ganas (VI.33, VI.36).
fn gana_dosha_columns() -> [ColumnDef; 2] {
    let flag = |name: &str, doc: &str| ColumnDef::new(name, Scalar::U8, doc);
    [
        flag(
            "gana_dosha",
            "1 when a Rakshasa stands beside another gana, the gana dosha.",
        ),
        flag(
            "gana_lifted",
            "1 when the gana dosha is lifted: the sign lords or the navamsha lords befriended (VI.33), or one sign with two stars or one star across two signs (VI.36, C283).",
        ),
    ]
}

fn bhakoot_columns() -> [ColumnDef; 8] {
    let flag = |name: &str, doc: &str| ColumnDef::new(name, Scalar::U8, doc);
    [
        flag(
            "bhakoot_apart",
            "The groom's Moon sign counted from the bride's, 1 to 12 (VI.31).",
        ),
        ColumnDef::new(
            "bhakoot_dosha",
            Scalar::U8,
            "The bad Bhakoot the two signs stand at, or none.",
        )
        .of_enum("TsBhakootDosha"),
        flag(
            "bhakoot_one_lord",
            "1 when one lord rules both signs: the first exception of VI.32–33, reported whether or not there is a dosha.",
        ),
        flag(
            "bhakoot_lords_friends",
            "1 when the two sign lords are each other's friends.",
        ),
        flag(
            "bhakoot_navamsha_lords_friends",
            "1 when the lords of the two Moons' navamshas are one or each other's friends.",
        ),
        flag("bhakoot_tara_pure", "1 when the tara is pure both ways."),
        flag("bhakoot_vashya", "1 when one sign is vashya to the other."),
        flag(
            "bhakoot_lifted",
            "1 when the exceptions lift the dosha under the record's `bhakootLift` (C263); 0 with no dosha.",
        ),
    ]
}

/// The `matchings` section's columns: the total, then each koota's reading
/// in the verse's order (`03-design/matching.md`).
fn matching_columns() -> Vec<ColumnDef> {
    let flag = |name: &str, doc: &str| ColumnDef::new(name, Scalar::U8, doc);
    let sides = |what: &str, kind: &str, doc: &str| {
        [
            ColumnDef::new(
                &format!("bride_{what}"),
                Scalar::U16,
                &format!("The bride's {doc}."),
            )
            .of_enum(kind),
            ColumnDef::new(
                &format!("groom_{what}"),
                Scalar::U16,
                &format!("The groom's {doc}."),
            )
            .of_enum(kind),
        ]
    };
    let mut columns = vec![ColumnDef::new(
        "total",
        Scalar::F64,
        "The eight kootas' points, out of 36, a multiple of a half.",
    )];
    columns.extend(sides(
        "varna",
        "Varna",
        "varna, by her or his Moon's sign (VI.22)",
    ));
    columns.push(
        ColumnDef::new(
            "vashya",
            Scalar::U8,
            "How the two Moon signs stand in Vashya (VI.23, C260).",
        )
        .of_enum("TsVashyaRelation"),
    );
    columns.extend([
        flag(
            "tara_bride_to_groom",
            "The tara counted from the bride's nakshatra to the groom's, 1 to 9 (VI.24); the 3rd, 5th and 7th are bad.",
        ),
        flag(
            "tara_groom_to_bride",
            "The tara counted from the groom's nakshatra to the bride's, 1 to 9.",
        ),
    ]);
    columns.extend(sides(
        "yoni",
        "Yoni",
        "yoni, by her or his Moon's nakshatra (VI.25–26)",
    ));
    columns.push(
        ColumnDef::new("yoni", Scalar::U8, "How the two yonis stand (C261).")
            .of_enum("TsYoniRelation"),
    );
    columns.extend([
        graha_column(
            "bride_lord",
            "The lord of the bride's Moon sign (VI.27–28).",
        ),
        graha_column("groom_lord", "The lord of the groom's Moon sign."),
        ColumnDef::new(
            "maitri",
            Scalar::U8,
            "How the two lords stand by the natural friendships.",
        )
        .of_enum("TsMaitriRelation"),
        flag(
            "maitri_lifted",
            "1 when the lords' enmity is lifted by a good Bhakoot (VI.33, C283).",
        ),
    ]);
    columns.extend(sides(
        "gana",
        "Gana",
        "gana, by her or his Moon's nakshatra (VI.29–30)",
    ));
    columns.extend(gana_dosha_columns());
    columns.extend(bhakoot_columns());
    columns.extend(sides(
        "nadi",
        "Nadi",
        "nadi, by her or his Moon's nakshatra (VI.34)",
    ));
    columns.push(flag(
        "nadi_dosha",
        "1 when the shared nadi is a dosha under the record's `nadiDosha` (C264).",
    ));
    columns.push(flag(
        "nadi_lifted",
        "1 when the nadi dosha is lifted by one sign with two stars, one star across two signs, or one star in two padas (VI.36, C284).",
    ));
    columns
}

/// The `poruthams` section's columns: the counts, the p. 76 exception's
/// clauses, then what each of the ten read in Kalaprakasika's order
/// (`03-design/matching.md`).
fn porutham_columns() -> Vec<ColumnDef> {
    let flag = |name: &str, doc: &str| ColumnDef::new(name, Scalar::U8, doc);
    let sides = |what: &str, kind: &str, scalar: Scalar, doc: &str| {
        [
            ColumnDef::new(
                &format!("bride_{what}"),
                scalar,
                &format!("The bride's {doc}."),
            )
            .of_enum(kind),
            ColumnDef::new(
                &format!("groom_{what}"),
                scalar,
                &format!("The groom's {doc}."),
            )
            .of_enum(kind),
        ]
    };
    let mut columns = vec![
        flag(
            "agreeing",
            "How many of the ten agree, a lift included; the chapter asks \"at least five\" (p. 76).",
        ),
        flag(
            "chief_agreeing",
            "How many of the chief five agree: Dhinam, Ganam, Yoni, Rasi and Rajju.",
        ),
        flag(
            "one_lord",
            "1 when one lord rules both Moon signs: the p. 76 exception's first clause (C277).",
        ),
        flag(
            "lords_friendly",
            "1 when the two sign lords are friendly on the chapter's own table, as the record's `lordsFriendship` reads it (C273).",
        ),
        flag("opposite", "1 when the two Moon signs are opposite."),
        flag(
            "count",
            "The groom's nakshatra counted from the bride's, 1 to 27, which Dhinam, Mahendra and Sthree-Dheergham read.",
        ),
        ColumnDef::new(
            "dhinam_rule",
            Scalar::U8,
            "Which of the chapter's rules decided Dhinam (pp. 69–72).",
        )
        .of_enum("TsDhinamRule"),
    ];
    columns.extend(sides(
        "gana",
        "Gana",
        Scalar::U16,
        "gana, by her or his Moon's nakshatra (p. 72)",
    ));
    columns.push(flag(
        "gana_diminished",
        "1 when a Rakshasa stands beside another gana and the bride's star is beyond the 14th from the groom's: the evil \"diminishes\", the disagreement stands (C279).",
    ));
    columns.extend(sides(
        "yoni",
        "Yoni",
        Scalar::U16,
        "yoni on the chapter's own table, Uttarashadha the cow (p. 73, C278)",
    ));
    columns.extend([
        flag("yoni_hostile", "1 when the two yonis are among the chapter's eight enmities."),
        flag("apart", "The groom's Moon sign counted from the bride's, 1 to 12, which Rasi reads (pp. 73–74)."),
        graha_column("bride_lord", "The lord of the bride's Moon sign."),
        graha_column("groom_lord", "The lord of the groom's Moon sign."),
        flag(
            "bride_calls_friend",
            "1 when the bride's lord calls the groom's a friend on the chapter's own table (pp. 74–75); a lord is its own.",
        ),
        flag("groom_calls_friend", "1 when the groom's lord calls the bride's a friend."),
        flag(
            "bride_to_groom",
            "1 when the bride's Moon sign is concordant to the groom's on p. 75's Vasyam table, never a sign to itself (C274).",
        ),
        flag("groom_to_bride", "1 when the groom's Moon sign is concordant to the bride's."),
    ]);
    columns.extend(sides(
        "rajju",
        "TsRajju",
        Scalar::U8,
        "Rajju division, by her or his Moon's nakshatra (p. 75, C275)",
    ));
    columns.push(flag(
        "pierced",
        "1 when the two nakshatras are a Vedhai pair of p. 76 (C276).",
    ));
    columns
}

/// A koota's points: which koota, its points and the most it gives.
fn matching_koota_columns() -> Vec<ColumnDef> {
    vec![
        ColumnDef::new("koota", Scalar::U16, "Which koota.").of_enum("Koota"),
        ColumnDef::new("points", Scalar::F64, "Its points, a multiple of a half."),
        ColumnDef::new(
            "max_points",
            Scalar::F64,
            "The most it gives, 1 for Varna to 8 for Nadi.",
        ),
    ]
}

/// One of the ten considerations: which, whether it agrees and whether
/// only by the exception.
fn porutham_row_columns() -> Vec<ColumnDef> {
    vec![
        ColumnDef::new(
            "koota",
            Scalar::U16,
            "Which consideration, a catalogue koota (C282).",
        )
        .of_enum("Koota"),
        ColumnDef::new("agrees", Scalar::U8, "1 when it agrees, a lift included."),
        ColumnDef::new(
            "lifted",
            Scalar::U8,
            "1 when it agrees only by the p. 76 exception: Ganam, Rasi, Rajju and Vedhai (C277).",
        ),
    ]
}

/// The four sections a match crosses as, from `first`: a row a chart with
/// what each koota read, each koota's points (eight rows a chart in the
/// verse's order), a row a chart with what each of the ten considerations
/// read, and whether each agrees (ten rows a chart in the chapter's order)
/// (`03-design/matching.md`).
fn chart_matching_sections(first: u32) -> [SectionSchema; 5] {
    let empty = "Empty when `matching_json` asked for none.";
    [
        SectionSchema::columns(
            first,
            "matchings",
            &format!(
                "Every chart matched with the record's partner by the Ashta Koota of *Muhurta Chintamani* VI.21–34, a row a chart in the `cast` section's order: what each koota read between the bride's Moon and the groom's, the chart on the side `partnerRole` leaves it. Never a verdict: the doshas and their exceptions are clauses. {empty}"
            ),
            matching_columns(),
        )
        .of_shape(MATCHINGS_SHAPE),
        SectionSchema::columns(
            first + 1,
            "matching_kootas",
            &format!(
                "Every chart's eight kootas, eight rows a chart in the `cast` section's order and the verse's: Varna, Vashya, Tara, Yoni, Graha Maitri, Gana, Bhakoot, Nadi. {empty}"
            ),
            matching_koota_columns(),
        )
        .of_shape(MATCHING_KOOTAS_SHAPE),
        SectionSchema::columns(
            first + 2,
            "poruthams",
            &format!(
                "Every chart matched with the record's partner by the ten considerations of *Kalaprakasika* XIII, a row a chart in the `cast` section's order: how many agree, the p. 76 exception's clauses, and what each of the ten read, on the chapter's own tables. Never a verdict. {empty}"
            ),
            porutham_columns(),
        )
        .of_shape(PORUTHAMS_SHAPE),
        SectionSchema::columns(
            first + 3,
            "porutham_rows",
            &format!(
                "Every chart's ten considerations, ten rows a chart in the `cast` section's order and the chapter's: Dhinam (`TARA`), Ganam, Mahendra, Sthree-Dheergham, Yoni, Rasi (`BHAKOOT`), Rasyadhipathi (`GRAHA_MAITRI`), Vasyam (`VASHYA`), Rajju, Vedhai. {empty}"
            ),
            porutham_row_columns(),
        )
        .of_shape(PORUTHAM_ROWS_SHAPE),
        SectionSchema::columns(
            first + 4,
            "kujas",
            &format!(
                "Every chart's Kuja dosha beside the record's partner's (*Manasagari*, jāyābhāva v. 4), a row a chart in the `cast` section's order: Mars's house by sign from the lagna, the Moon and Venus on each side, whether each side carries the dosha under `matching.kuja`, and whether both do. Never lifted (C288). {empty}"
            ),
            kuja_columns(),
        ),
    ]
}

/// The two sections a match's marriage doshas cross as, from `first`: a
/// count a chart, and the entries ragged under it (C289, C290).
fn chart_marriage_dosha_sections(first: u32) -> [SectionSchema; 2] {
    let empty = "Empty when `matching_json` asked for none.";
    [
        SectionSchema::columns(
            first,
            "marriage_doshas",
            &format!(
                "How many marriage doshas each chart's match carries, a row a chart in the `cast` section's order; the entries are `marriage_dosha_rows`, **ragged** by `count` (C289). {empty}"
            ),
            vec![ColumnDef::new(
                "count",
                Scalar::U32,
                "The number of entries, every dosha the three readings report.",
            )],
        ),
        SectionSchema::columns(
            first + 1,
            "marriage_dosha_rows",
            &format!(
                "Every chart's marriage doshas, concatenated in the `cast` section's order and **ragged** by `marriage_doshas.count`, each chart's in the answers' own order: the Ashta Koota's Bhakoot, Nadi, Gana and the lords' enmity, each of the ten that disagrees or agrees by the p. 76 exception, then each side's Kuja dosha, the bride's first. Never a severity (C290). {empty}"
            ),
            vec![
                ColumnDef::new("system", Scalar::U8, "Which reading it comes from.")
                    .of_enum("TsDoshaSystem"),
                ColumnDef::new(
                    "koota",
                    Scalar::U16,
                    "The koota or consideration it is; read only when `system` is not `KUJA`.",
                )
                .of_enum("Koota"),
                ColumnDef::new(
                    "side",
                    Scalar::U8,
                    "The side carrying it; read only when `system` is `KUJA`.",
                )
                .of_enum("TsMatchRole"),
                ColumnDef::new(
                    "lifted",
                    Scalar::U8,
                    "1 when an exception the source names lifts it.",
                ),
            ],
        ),
    ]
}

/// Every chart read as a prashna (`03-design/prashna.md`).
fn chart_prashna_section(id: u32) -> SectionSchema {
    SectionSchema::bytes(
        id,
        "prashna",
        "UTF-8 JSON, canonical: an array with one entry per chart, each `{rules, verdict, change, timing, mook, links, moon, score, numberSign}` (`03-design/prashna.md`). `verdict` is `{clauses, outcome}`, each clause `{kind, graha, favour}` naming its verse by `kind`, and `outcome` `SUCCEEDS`, `WITH_DIFFICULTY` or `FAILS` (*Shatpanchashika* I.4, C337). `change` is `STAYS` or `CHANGES`. `timing` is `{rule, graha, tie, count, multiplier, amount, unit, between}`, `amount` null where the rule gives none. `mook` is `{rule, graha, tie, house, person, thought}`, `person` null but under `SHATPANCHASHIKA`. `links` is the Tajika yogas between the lagna lord and the asked house's lord, as a year's `matters` carry them, null when no house was asked. `moon` is `{rules, clauses}`, each clause a key of the Samjna Tantra vv. 73-74 (C352). `score` is the baseline engine's `{points, answer, factors, void, applyingTo}`, null unless `rules.score` is `BASELINE`; `numberSign` the sign of the querent's number, null unless one was given. Empty when `prashna_json` asked for none.",
    )
}

/// Every chart's remedies (`03-design/remedies.md`).
fn chart_remedies_section(id: u32) -> SectionSchema {
    SectionSchema::bytes(
        id,
        "remedies",
        "UTF-8 JSON, canonical: an array with one entry per chart, each `{rules, functional, subjects, shantis, ishtaDevata}` (`03-design/remedies.md`). `rules` is `{functional, shanti, devata}`, every member filled. `functional` is the lagna's natures under *Laghu Parashari*: `{lagna, scheme, rows, yogakarakas, marakas, badhaka}`, each of the seven's row `{graha, houses, clauses, nature}` with every clause `{kind, house}` that made it, and `badhaka` `{house, lord}`. `subjects` is `{subjects, antardasha}`: each graha with at least one reason, `{graha, reasons}`, ranked by nothing (C351), and `antardasha` the running Vimshottari antardasha's printed shanti `{shanti, holds}`, `shanti` being `{mahadasha, antardasha, chapter, verses, page, conditions, remedies}` and `holds` each condition true, false or null where the verse leaves it open, the whole null unless the record named `at`. `shantis` is each subject's graha-shanti, `{graha, image, rik, japaThousands, samidh, food, dakshina, gem, substance, direction, mandala}`. `ishtaDevata` is `{atmakaraka, karakamsha, inRasi, inNavamsha}`, each chart's `{rules, sign, devotions, minor}`: every graha in the 12th from the karakamsha as `{graha, deities, verse, withKetu}`, and Saturn or Venus there in a malefic's sign (C354 to C356). Empty when `remedies_json` asked for none.",
    )
}

/// Every chart read as a birth time to rectify
/// (`03-design/rectification.md`, step 8).
fn chart_rectification_section(id: u32) -> SectionSchema {
    SectionSchema::bytes(
        id,
        "rectification",
        "UTF-8 JSON, canonical: an array with one entry per chart, each with a member for every reading the record asked and none for one it did not. `purified` is what the purifier of BPHS ch. 2 vv. 67-78 leaves standing of the window around the chart's instant, `{intervals, removed, edges, grid}`: each interval `{from, to, verdict}` with the clauses that held, each removed run naming the clause that failed. `conception` is `{birth, pranapadaHouse, nisheka, moon}` at the chart's instant (BPHS ch. 3 vv. 25-29, *Brihat Jataka* IV.21). `circumstance` is `{sky, father, presentation, lamp, attending, weights}` (*Brihat Jataka* ch. V), each fact given one weight. `baseline` is the baseline engine's unsourced cascade around the chart's instant, `{window, sunrise, intervals, intervalWidthMinutes, resolutionMinutes, suggested, concentration, candidates, stages, eventsUsed, eventsHeldOut, holdOut}`. Instants are Julian days UTC. Empty when `rectification_json` asked for none.",
    )
}

/// Every chart's avakahada from `first`: the Moon's readings a row a
/// chart, and the syllables it is named by as text (`03-design/matching.md`,
/// C297 to C301).
fn chart_avakahada_sections(first: u32) -> [SectionSchema; 2] {
    let empty = "Empty when the avakahada was not asked for.";
    let read =
        |name: &str, doc: &str, kind: &str| ColumnDef::new(name, Scalar::U16, doc).of_enum(kind);
    [
        SectionSchema::columns(
            first,
            "avakahada",
            &format!(
                "Each chart's avakahada, a row a chart in the `cast` section's order: what a janma-patrika prints of the Moon, each reading the one the Ashta Koota takes of the same Moon (C301). Vashya, paya, disha and tatwa are not here (C300). {empty}"
            ),
            vec![
                read("nakshatra", "The Moon's nakshatra.", "Nakshatra"),
                ColumnDef::new("pada", Scalar::U8, "Its pada, 1 to 4."),
                read("rashi", "The Moon's sign.", "Rashi"),
                read(
                    "nakshatra_lord",
                    "The nakshatra's lord, the Vimshottari dasha's.",
                    "Graha",
                ),
                read(
                    "rashi_lord",
                    "The sign's lord, the one Graha Maitri reads.",
                    "Graha",
                ),
                read(
                    "varna",
                    "The sign's varna, as Varna koota reads it (VI.22).",
                    "Varna",
                ),
                read("yoni", "The nakshatra's yoni.", "Yoni"),
                read("gana", "The nakshatra's gana.", "Gana"),
                read("nadi", "The nakshatra's nadi.", "Nadi"),
                ColumnDef::new(
                    "cell",
                    Scalar::U8,
                    "The birth syllable's place among the śatapada cakra's 112 cells, 0 for a, Krittika's first (C297).",
                ),
                ColumnDef::new("varga", Scalar::U8, "The birth syllable's varga (C295).")
                    .of_enum("TsNameVarga"),
            ],
        ),
        SectionSchema::bytes(
            first + 1,
            "avakahada_syllables",
            &format!(
                "UTF-8 JSON, canonical: an array with one entry per row of `avakahada`, each `[devanagari, iast]`, the birth pada's syllable as *Muhurta Chintamani* p. 173 prints it and its IAST (C299). {empty}"
            ),
        ),
    ]
}

/// The `kujas` section's columns: the bride's side, the groom's, and
/// whether both carry the dosha (`03-design/matching.md`).
fn kuja_columns() -> Vec<ColumnDef> {
    let side = |who: &str, whose: &str| {
        let mut columns: Vec<ColumnDef> = ["lagna", "moon", "venus"]
            .iter()
            .map(|from| {
                ColumnDef::new(
                    &format!("{who}_{from}_house"),
                    Scalar::U8,
                    &format!("Mars's house by sign from {whose} {from}, 1 to 12 (C287)."),
                )
            })
            .collect();
        columns.extend(["lagna", "moon", "venus"].iter().map(|from| {
            ColumnDef::new(
                &format!("{who}_{from}_in_houses"),
                Scalar::U8,
                &format!(
                    "1 when that house from {whose} {from} is one of the rules' houses (C285)."
                ),
            )
        }));
        columns.push(ColumnDef::new(
            &format!("{who}_dosha"),
            Scalar::U8,
            &format!("1 when {whose} Mars stands in one of the rules' houses from a reference the rules count (C286)."),
        ));
        columns
    };
    let mut columns = side("bride", "the bride's");
    columns.extend(side("groom", "the groom's"));
    columns.push(ColumnDef::new(
        "both",
        Scalar::U8,
        "1 when both carry the dosha, the fact the popular cancellation reads; nothing is lifted (C288).",
    ));
    columns
}

/// The three sections the declinations and the parallels cross as, from
/// `first`: a row a chart, each planet's declination, and the parallels,
/// both ragged under the row (`03-design/western-declinations.md`).
fn chart_declination_sections(first: u32) -> [SectionSchema; 3] {
    let empty = "Empty when `parallels_json` asked for none.";
    let degrees = |name: &str, doc: &str| ColumnDef::new(name, Scalar::F64, doc);
    [
        SectionSchema::columns(
            first,
            "declinations",
            &format!(
                "Every chart's distances from the equator, a row a chart in the `cast` section's order: the obliquity they were turned by, the angles', and how many rows of `declination_rows` and `parallel_rows` are its. {empty}"
            ),
            vec![
                degrees(
                    "obliquity_deg",
                    "The true obliquity at the chart's instant, degrees.",
                ),
                degrees(
                    "lagna_deg",
                    "The lagna's declination, degrees north: the Sun's at that degree (Leo, p. 141).",
                ),
                degrees(
                    "midheaven_deg",
                    "The midheaven's declination, degrees north, read the same way.",
                ),
                ColumnDef::new(
                    "graha_count",
                    Scalar::U32,
                    "How many planets' declinations are the chart's in `declination_rows`.",
                ),
                ColumnDef::new(
                    "parallel_count",
                    Scalar::U32,
                    "How many parallels are the chart's in `parallel_rows`.",
                ),
            ],
        ),
        SectionSchema::columns(
            first + 1,
            "declination_rows",
            &format!(
                "Every chart's planets' declinations, concatenated in the `cast` section's order and **ragged** by `declinations.graha_count`: the seven, and the outer three when `TS_CHART_OUTER` placed them, in the catalogue's order, from each one's tropical longitude, ecliptic latitude and the true obliquity. {empty}"
            ),
            vec![
                graha_column("graha", "Which planet."),
                degrees("declination_deg", "Its declination, degrees north."),
            ],
        ),
        SectionSchema::columns(
            first + 2,
            "parallel_rows",
            &format!(
                "Every chart's parallels, concatenated in the `cast` section's order and **ragged** by `declinations.parallel_count`, each chart's closest first: a pair of its planets the same distance from the equator within the record's orb (Leo's 1° by default, p. 47), on either side of it (C243). {empty}"
            ),
            vec![
                graha_column(
                    "first",
                    "The first planet of the pair, in the catalogue's order.",
                ),
                graha_column("second", "The second."),
                ColumnDef::new(
                    "contrary",
                    Scalar::U8,
                    "1 when the two stand on opposite sides of the equator, the contra-parallel; 0 when on one side.",
                ),
                degrees(
                    "apart_deg",
                    "How far apart their distances from the equator are, degrees.",
                ),
                degrees("orb_deg", "The orb the record allowed, degrees."),
            ],
        ),
    ]
}

/// The four sections progressions cross as, from `first`: a row a chart,
/// the progressed and directed planets, and the contacts ragged under the
/// row (`03-design/western-progressions.md`).
fn chart_progression_sections(first: u32) -> [SectionSchema; 4] {
    let empty = "Empty when `progressions_json` asked for none.";
    [
        progressions_section(first, empty),
        SectionSchema::columns(
            first + 1,
            "progressed_grahas",
            &format!(
                "The progressed planets, **the same number of rows a chart** in the `cast` section's order, each chart's in the catalogue's order: the chart founded at `progressions.sky`, the nine and then the outer three when the birth placed them (`TS_CHART_OUTER`). A reader divides the rows by `chart_count`. Empty when the record named no `at`. {empty}"
            ),
            vec![
                graha_column("graha", "Which planet."),
                ColumnDef::new(
                    "longitude_deg",
                    Scalar::F64,
                    "Its progressed longitude in the chart's zodiac, degrees.",
                ),
                ColumnDef::new(
                    "tropical_deg",
                    Scalar::F64,
                    "Its progressed tropical longitude, degrees.",
                ),
                ColumnDef::new(
                    "speed_deg_per_day",
                    Scalar::F64,
                    "Its speed at the instant of sky, degrees a day; below zero when retrograde.",
                ),
            ],
        ),
        SectionSchema::columns(
            first + 2,
            "directed_grahas",
            &format!(
                "The birth's planets moved by the direction's arc, **as many rows a chart as `progressed_grahas`** in the `cast` section's order: the nine, and the outer three when the birth placed them. Empty when the record named no `at`. {empty}"
            ),
            vec![
                graha_column("graha", "Which planet."),
                ColumnDef::new(
                    "longitude_deg",
                    Scalar::F64,
                    "Its directed longitude in the chart's zodiac, degrees.",
                ),
            ],
        ),
        progressed_contacts_section(first + 3, empty),
    ]
}

/// The `progressions` section: a row a chart.
fn progressions_section(id: u32, empty: &str) -> SectionSchema {
    let no_at = "NaN when the record named no `at`.";
    let degrees = |name: &str, doc: &str| {
        ColumnDef::new(name, Scalar::F64, &format!("{doc}, degrees. {no_at}"))
    };
    SectionSchema::columns(
        id,
        "progressions",
        &format!(
            "Every chart's progressions (Leo, *The Progressed Horoscope*), a row a chart in the `cast` section's order: the progressed chart and the direction at the record's `at`, and how many contacts its window holds. {empty}"
        ),
        vec![
            ColumnDef::new(
                "life",
                Scalar::F64,
                &format!("The instant of life asked for, a Julian day (UTC). {no_at}"),
            ),
            ColumnDef::new(
                "sky",
                Scalar::F64,
                &format!(
                    "The instant of sky that measures it, a Julian day (UTC): where the progressed planets are read. {no_at}"
                ),
            ),
            degrees(
                "armc_deg",
                "The progressed meridian's right ascension, by `angles` (C237)",
            ),
            degrees(
                "ascendant_deg",
                "The progressed ascendant, in the chart's zodiac",
            ),
            degrees(
                "midheaven_deg",
                "The progressed midheaven, in the chart's zodiac",
            ),
            degrees(
                "arc_deg",
                "The direction's arc: the solar arc, signed, or the measure's degrees for the years of life",
            ),
            degrees(
                "directed_ascendant_deg",
                "The birth's ascendant moved by the arc",
            ),
            degrees(
                "directed_midheaven_deg",
                "The birth's midheaven moved by the arc",
            ),
            ColumnDef::new(
                "contact_count",
                Scalar::U32,
                "How many rows of the `progressed_contacts` section belong to this chart; 0 when the record named no `contacts`.",
            ),
            ColumnDef::new(
                "contacts_asked",
                Scalar::U8,
                "1 when the record named `contacts`, so a count of 0 is a window holding none; 0 when it named none.",
            ),
        ],
    )
}

/// The `progressed_contacts` section: every contact, ragged under the
/// `progressions` row.
fn progressed_contacts_section(id: u32, empty: &str) -> SectionSchema {
    SectionSchema::columns(
        id,
        "progressed_contacts",
        &format!(
            "Every exact aspect a progressed planet makes to a radical point in the record's window (Leo's Appendix V), concatenated in the `cast` section's order and **ragged** by `progressions.contact_count`, each chart's in the order they fall due. {empty}"
        ),
        vec![
            ColumnDef::new(
                "life",
                Scalar::F64,
                "The instant of life it falls due, a Julian day (UTC).",
            ),
            ColumnDef::new(
                "sky",
                Scalar::F64,
                "The instant of sky the aspect is exact at, a Julian day (UTC).",
            ),
            graha_column("graha", "The progressed planet."),
            ColumnDef::new(
                "to_lagna",
                Scalar::U8,
                "1 when the radical point is the lagna, 0 when it is a radical graha.",
            ),
            ColumnDef::new(
                "to_graha",
                Scalar::U16,
                "The radical graha aspected (a `Graha` id); 0 for the lagna.",
            ),
            ColumnDef::new(
                "angle",
                Scalar::U16,
                "The aspect's angle, a whole degree 0 to 180, either side of the radical point.",
            ),
            ColumnDef::new(
                "motion",
                Scalar::U8,
                "Which way the progressed planet was moving.",
            )
            .of_enum("TsMotion"),
        ],
    )
}

/// The `perfection_collections` section: every collection of light,
/// ragged under the `perfection` row.
fn perfection_collections_section(id: u32, empty: &str) -> SectionSchema {
    SectionSchema::columns(
        id,
        "perfection_collections",
        &format!(
            "Every collection of light (p. 112): a heavier planet both significators apply to, concatenated in the `cast` section's order and **ragged** by `perfection.collection_count`. Who must receive whom is C233. {empty}"
        ),
        vec![
            graha_column("collector", "The heavier planet."),
            aspect_column(
                "from_querent_aspect",
                "The aspect the querent's significator applies by.",
            ),
            ColumnDef::new("from_querent_days", Scalar::F64, "Days until it is exact."),
            aspect_column(
                "from_quesited_aspect",
                "The aspect the quesited's significator applies by.",
            ),
            ColumnDef::new("from_quesited_days", Scalar::F64, "Days until it is exact."),
            dignity_column(
                "collector_in_querent",
                "The querent's significator's dignities the collector stands in",
            ),
            dignity_column(
                "collector_in_quesited",
                "The quesited's significator's dignities the collector stands in",
            ),
            dignity_column(
                "querent_in_collector",
                "The collector's dignities the querent's significator stands in",
            ),
            dignity_column(
                "quesited_in_collector",
                "The collector's dignities the quesited's significator stands in",
            ),
        ],
    )
}

/// The `perfection` section: a row a chart, the significators'
/// application, separation and standing, and the ways it holds.
#[expect(
    clippy::too_many_lines,
    reason = "one declaration per fact Lilly weighs; splitting it would hide the shape it exists to show"
)]
fn perfection_section(id: u32, empty: &str) -> SectionSchema {
    let flag = |name: &str, doc: &str| {
        ColumnDef::new(name, Scalar::U8, &format!("1 when {doc}; 0 otherwise."))
    };
    let count = |name: &str, section: &str| {
        ColumnDef::new(
            name,
            Scalar::U32,
            &format!("How many rows of the `{section}` section belong to this chart."),
        )
    };
    SectionSchema::columns(
        id,
        "perfection",
        &format!(
            "Whether a horary matter is brought to pass (Lilly, *Christian Astrology* pp. 107–113 and 125–127), a row a chart in the `cast` section's order: the significators' application and separation, where each stands, and which of the seven ways of perfection the figure holds, never a verdict. The future is the ephemeris searched from the chart's instant up to the horizon. {empty}"
        ),
        vec![
            graha_column(
                "querent",
                "The querent's significator: the Ascendant's lord unless named.",
            ),
            graha_column(
                "quesited",
                "The quesited's significator: as named, or the lord of the asked house's cusp.",
            ),
            ColumnDef::new(
                "horizon_days",
                Scalar::F64,
                "How far ahead the timeline was searched, days: the rules' horizon, or until the swifter significator leaves its sign (C232), at most ten years.",
            ),
            ColumnDef::new(
                "horizon_rule_days",
                Scalar::F64,
                "`perfection_json.rules.horizonDays` as asked; NaN when unset.",
            ),
            flag(
                "within_sign_rule",
                "1 when `perfection_json.rules.withinSign` held, as by default: a third planet's contact counted only before the applier left its sign (C234).",
            ),
            ColumnDef::new(
                "application_present",
                Scalar::U8,
                "1 when the significators apply within the horizon; 0 otherwise, and then the application's columns are 0 and NaN.",
            ),
            aspect_column("application_aspect", "The aspect they apply by."),
            ColumnDef::new("application_days", Scalar::F64, "Days until it is exact."),
            graha_column("applying", "The significator whose motion closes it."),
            ColumnDef::new(
                "application_kind",
                Scalar::U8,
                "Which of the three kinds (p. 107).",
            )
            .of_enum("TsApplicationKind"),
            ColumnDef::new(
                "gap_deg",
                Scalar::F64,
                "How far it is from exact now, degrees.",
            ),
            flag(
                "within_moieties",
                "the gap is already within the two planets' moieties of orb",
            ),
            ColumnDef::new(
                "separation_present",
                Scalar::U8,
                "1 when the significators are separating within their moieties at the figure (p. 110); 0 otherwise, and then the separation's columns are 0 and NaN.",
            ),
            aspect_column("separation_aspect", "The aspect they separate from."),
            ColumnDef::new(
                "separation_past_deg",
                Scalar::F64,
                "How far past exact it is, degrees.",
            ),
            ColumnDef::new(
                "querent_house",
                Scalar::U8,
                "The house the querent's significator is in, 1 to 12, under the fortitudes' houses.",
            ),
            dignity_column(
                "querent_dignity",
                "The querent's significator's essential dignities at its degree",
            ),
            ColumnDef::new(
                "quesited_house",
                Scalar::U8,
                "The house the quesited's significator is in, 1 to 12.",
            ),
            dignity_column(
                "quesited_dignity",
                "The quesited's significator's essential dignities at its degree",
            ),
            flag(
                "mutual_by_house",
                "each significator stands in the other's house",
            ),
            ColumnDef::new(
                "infortunes_between",
                Scalar::U8,
                "Saturn and Mars when among the thirds that come between the significators before they perfect, as a bit set: bit `n` is the graha with catalogue id `n`.",
            ),
            flag(
                "moon_relays",
                "the Moon, neither significator, separates from the quesited's and comes next to the querent's (p. 126, the opposition)",
            ),
            flag(
                "quesited_in_ascendant",
                "the quesited's significator is in the first house",
            ),
            ColumnDef::new(
                "ways_held",
                Scalar::U8,
                "The ways of perfection the figure holds (pp. 125–127), as a bit set over `TsWay`: bit `n` is the member with code `n`. 0 when it holds none.",
            ),
            count("impediment_count", "perfection_impediments"),
            count("translation_count", "perfection_translations"),
            count("collection_count", "perfection_collections"),
        ],
    )
}

/// The `considerations` section: a row a chart, each clause with the
/// facts it rests on.
#[expect(
    clippy::too_many_lines,
    reason = "one declaration per clause Lilly lists; splitting it would hide the shape it exists to show"
)]
fn consideration_clauses_section(id: u32, empty: &str) -> SectionSchema {
    let flag = |name: &str, doc: &str| {
        ColumnDef::new(name, Scalar::U8, &format!("1 when {doc}; 0 otherwise."))
    };
    SectionSchema::columns(
        id,
        "considerations",
        &format!(
            "Every chart's considerations before judgement (Lilly, *Christian Astrology* I.XIX), a row a chart in the `cast` section's order: each clause with the facts it rests on, never a verdict. {empty}"
        ),
        vec![
            ColumnDef::new(
                "hour_lord",
                Scalar::U16,
                "The lord of the chart's planetary hour, under the settings' `hora_reckoning`.",
            )
            .of_enum("Graha"),
            ColumnDef::new(
                "ascendant_lord",
                Scalar::U16,
                "The lord of the rising sign.",
            )
            .of_enum("Graha"),
            ColumnDef::new(
                "radical_grounds",
                Scalar::U8,
                "Why the figure is radical, as a bit set over `TsRadicalGround`: bit `n` is the member with code `n` (p. 122). 0 when it is not radical.",
            ),
            ColumnDef::new("ascendant_sign", Scalar::U16, "The rising sign.").of_enum("Rashi"),
            ColumnDef::new(
                "ascendant_degree",
                Scalar::F64,
                "The Ascendant's degree within its sign, [0, 30).",
            ),
            flag(
                "ascendant_early",
                "fewer than 3 degrees rise, too early to judge",
            ),
            flag(
                "ascendant_late",
                "27 degrees or more rise, too late to judge",
            ),
            flag(
                "short_ascension",
                "the rising sign is one of short ascension, Capricorn to Gemini",
            ),
            ColumnDef::new("moon_sign", Scalar::U16, "The Moon's sign.").of_enum("Rashi"),
            ColumnDef::new(
                "moon_degree",
                Scalar::F64,
                "The Moon's degree within her sign, [0, 30).",
            ),
            flag(
                "moon_late",
                "the Moon is in the later degrees of her sign, from `moon_late_from_deg` (C229)",
            ),
            flag(
                "moon_late_sign",
                "the Moon is in Gemini, Scorpio or Capricorn, where Lilly says lateness matters most",
            ),
            flag(
                "via_combusta",
                "the Moon is in the via combusta, Libra 15° to Scorpio 15°",
            ),
            ColumnDef::new(
                "days_in_sign",
                Scalar::F64,
                "Days until the Moon leaves her sign, at her motion of the moment.",
            ),
            flag(
                "eased",
                "the Moon is in Taurus, Cancer, Sagittarius or Pisces, where void of course \"somewhat she performes\"",
            ),
            ColumnDef::new(
                "seventh_cusp_deg",
                Scalar::F64,
                "The seventh cusp, degrees of the chart's zodiac.",
            ),
            ColumnDef::new(
                "seventh_lord",
                Scalar::U16,
                "The lord of the sign on the seventh cusp.",
            )
            .of_enum("Graha"),
            ColumnDef::new(
                "seventh_infortunes",
                Scalar::U8,
                "Saturn and Mars when counted in the seventh house, as a bit set: bit `n` is the graha with catalogue id `n` (C231).",
            ),
            flag(
                "seventh_lord_retrograde",
                "the seventh's lord is retrograde",
            ),
            flag("seventh_lord_combust", "the seventh's lord is combust"),
            flag("seventh_lord_in_fall", "the seventh's lord is in his fall"),
            flag(
                "seventh_lord_in_infortune_term",
                "the seventh's lord is in the terms of Saturn or Mars",
            ),
            ColumnDef::new(
                "seventh_lord_net",
                Scalar::I16,
                "The seventh's lord's net strength, his essential and accidental fortitudes less his debilities.",
            ),
            ColumnDef::new(
                "saturn_house",
                Scalar::U8,
                "The house Saturn is counted in, 1 to 12, by the fortitudes' five-degree rule.",
            ),
            flag("saturn_retrograde", "Saturn is retrograde"),
            flag("ascendant_lord_combust", "the Ascendant's lord is combust"),
            ColumnDef::new(
                "moon_late_from_deg",
                Scalar::F64,
                "The degree the Moon's lateness was counted from, `considerations_json.moonLateFromDeg`.",
            ),
        ],
    )
}

/// Every chart's receptions, a row a pair (`essential-dignities.md`
/// §Reception).
fn dignity_receptions_section(id: u32) -> SectionSchema {
    // One side of a reception: the other planet's dignities where this one
    // stands, in `dignity_planets`' order of flags.
    let received = |side: &str| {
        let other = if side == "first" { "second" } else { "first" };
        [
            "house",
            "exaltation",
            "triplicity",
            "term",
            "face",
            "detriment",
            "fall",
        ]
        .map(|kind| {
            ColumnDef::new(
                &format!("{side}_in_{kind}"),
                Scalar::U8,
                &format!("1 when `{side}` stands in `{other}`'s {kind}, else 0."),
            )
        })
    };
    SectionSchema::columns(
        id,
        "dignity_receptions",
        "Every pair of the seven each standing in at least one of the other's five dignities (Lilly, p. 112), concatenated in the `cast` section's order and **ragged** by `dignities.reception_count`, each chart's in the Chaldean order of `first` and then `second`. Each side is reported whole, so a reception by the same dignity both ways (mutual) and one by different dignities (mixed) are read off the same row. Empty when neither `dignities_json` nor `fortitudes_json` asked.",
        {
            let mut columns = vec![
                ColumnDef::new(
                    "first",
                    Scalar::U16,
                    "The first of the two, in the Chaldean order.",
                )
                .of_enum("Graha"),
                ColumnDef::new("second", Scalar::U16, "The second.").of_enum("Graha"),
            ];
            columns.extend(received("first"));
            columns.extend(received("second"));
            columns
        },
    )
}

/// Every chart's transit hit list, a row a hit.
fn chart_hits_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "hits",
        "Every chart's transit hit list, concatenated in the `cast` section's order and **ragged** by its `hit_count`, each chart's sorted by instant, then graha, then kind (`03-design/transit-hit-list.md`). Each sign and nakshatra is the one a chart founded at that instant gives. The sky is searched **once for the batch**: a chart's ingresses and stations are every chart's, and only its aspects are its own. Empty when `hits_json` asked for none.",
        vec![
            ColumnDef::new("instant", Scalar::F64, "When, as a Julian day (UTC)."),
            ColumnDef::new("graha", Scalar::U16, "The transiting graha.").of_enum("Graha"),
            ColumnDef::new("kind", Scalar::U8, "What happened, which says which columns below mean something.")
                .of_enum("TsHitKind"),
            ColumnDef::new(
                "into",
                Scalar::U16,
                "The sign (a `Rashi` id) entered by a sign ingress, or the nakshatra (a `Nakshatra` id) entered by a nakshatra ingress; 0 for any other kind. A retrograde ingress enters the division before the line it crossed.",
            ),
            ColumnDef::new(
                "motion",
                Scalar::U8,
                "Which way the graha was moving through the line, or, for a station, the motion it turned to.",
            )
            .of_enum("TsMotion"),
            ColumnDef::new(
                "to_lagna",
                Scalar::U8,
                "For an aspect, 1 when the natal point aspected is the lagna, 0 when it is a natal graha; 0 for any other kind.",
            ),
            ColumnDef::new(
                "to_graha",
                Scalar::U16,
                "For an aspect to a natal graha, which (a `Graha` id); 0 otherwise.",
            ),
            ColumnDef::new(
                "angle",
                Scalar::U16,
                "For an aspect, its angle, 0 to 180 degrees, either side of the natal point (C145); 0 for any other kind.",
            ),
            ColumnDef::new(
                "phase",
                Scalar::U8,
                "For an aspect, where in its window: entering or leaving the orb, or exact (C146); read only when `kind` is an aspect.",
            )
            .of_enum("TsAspectPhase"),
        ],
    )
}

/// Every transit's seven judged by the natal Ashtakavarga, a row a graha.
fn chart_gochar_ashtakavarga_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "gochar_ashtakavarga",
        "Each transit's seven, the Sun to Saturn, judged by the natal Ashtakavarga (Phaladeepika ch. 23; `03-design/gochar-ashtakavarga.md`): row `r * 7 + g` is row `r` of `gochar`, graha `g`. **Empty unless `gochar_json.ashtakavarga` asked**, and then every row of `gochar` has its seven; the nodes have no Ashtakavarga.",
        vec![
            ColumnDef::new("graha", Scalar::U16, "Which graha.").of_enum("Graha"),
            ColumnDef::new(
                "bindus",
                Scalar::U8,
                "The bindus its own Ashtakavarga put in the sign it transits, 0 to 8, unreduced (v. 11; C143).",
            ),
            ColumnDef::new(
                "good",
                Scalar::U8,
                "1 when they reach `gochar.ashtakavarga_good_from`, else 0 (C141).",
            ),
            ColumnDef::new(
                "kakshya",
                Scalar::U8,
                "The eighth of the sign it stands in, 1 to 8, 3°45′ each (v. 16).",
            ),
            ColumnDef::new(
                "kakshya_lord",
                Scalar::U8,
                "That eighth's lord, in the orbits' order from the sign's start (vv. 18 and 19).",
            )
            .of_enum("TsKakshyaLord"),
            ColumnDef::new(
                "kakshya_bindu",
                Scalar::U8,
                "1 when that lord gave a bindu to the sign in this graha's Ashtakavarga, so that a bindu bears its fruit now; else 0.",
            ),
            ColumnDef::new(
                "sarva",
                Scalar::U16,
                "The sign's sarvashtakavarga, the seven's bindus together.",
            ),
            ColumnDef::new(
                "sarva_standing",
                Scalar::U8,
                "Where it stands against 28 (v. 20; C142).",
            )
            .of_enum("TsSarvaStanding"),
        ],
    )
}

/// Every chart's transits, a row an instant.
fn chart_gochar_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "gochar",
        "Each chart's transits at the instants `gochar_json.instants` named, charts outermost and each chart's in the order asked: with `n` instants, row `i * n + k` is chart `i` at instant `k`. **Fixed, not ragged**: the request settles `n` for every chart, so `n` is this section's rows over `summary.chart_count`. Read under the settings' `gochar` group (Phaladeepika ch. 26). Empty when no transits were asked for.",
        vec![
            ColumnDef::new(
                "instant",
                Scalar::F64,
                "The instant the transits were read at, a UTC Julian day.",
            ),
            ColumnDef::new(
                "reference",
                Scalar::U16,
                "The sign the houses are counted from: the natal Moon's by v. 1, or the lagna's when `counted_from` says so.",
            )
            .of_enum("Rashi"),
            ColumnDef::new(
                "counted_from",
                Scalar::U8,
                "Which natal point `reference` is, `gochar_json.from` (C139).",
            )
            .of_enum("TsGocharFrom"),
            ColumnDef::new(
                "node_vedha",
                Scalar::U8,
                "The nodes' vedha the transits were judged under, `gochar.node_vedha` (C136).",
            )
            .of_enum("TsNodeVedha"),
            ColumnDef::new(
                "node_obstruction",
                Scalar::U8,
                "Whom the nodes obstruct, `gochar.node_obstruction` (C137, C140).",
            )
            .of_enum("TsNodeObstruction"),
            ColumnDef::new(
                "ashtakavarga_good_from",
                Scalar::U8,
                "How many bindus make a transit good in `gochar_ashtakavarga`, `gochar.ashtakavarga_good_from` (C141).",
            )
            .of_enum("TsAshtakavargaGoodFrom"),
        ],
    )
}

/// Every transit's grahas, a row a graha.
fn chart_gochar_grahas_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "gochar_grahas",
        "Each transit's nine grahas, the Sun to Ketu: row `r * 9 + g` is row `r` of `gochar`, graha `g`.",
        vec![
            ColumnDef::new("graha", Scalar::U16, "Which graha.").of_enum("Graha"),
            ColumnDef::new("sign", Scalar::U16, "The sign it transits.").of_enum("Rashi"),
            ColumnDef::new(
                "degrees",
                Scalar::F64,
                "Its degrees within the sign, 0 to 30.",
            ),
            ColumnDef::new(
                "house",
                Scalar::U8,
                "Its house from `gochar.reference`, 1 to 12.",
            ),
            ColumnDef::new(
                "good_house",
                Scalar::U8,
                "1 when v. 2 makes a transit of this house good, else 0.",
            ),
            ColumnDef::new(
                "vedha_house",
                Scalar::U8,
                "The house whose occupant obstructs it (vv. 3 to 8), 1 to 12; 0 when the house is not good or, for a node under `node_vedha = NONE`, nothing obstructs it.",
            ),
            ColumnDef::new(
                "obstructed_by",
                Scalar::U16,
                "The grahas standing in the vedha house that obstruct it, the verses' exemptions left out, a bit set: bit `n` is the graha with id `n`.",
            ),
            ColumnDef::new("verdict", Scalar::U8, "What the transit comes to.")
                .of_enum("TsGocharVerdict"),
            ColumnDef::new(
                "fruition",
                Scalar::U8,
                "The decanate in which its transit bears fruit (v. 25).",
            )
            .of_enum("TsFruition"),
            ColumnDef::new(
                "fruitful_now",
                Scalar::U8,
                "1 when it stands in that decanate now, else 0.",
            ),
        ],
    )
}

/// Every chart's Jaimini significators, a row a chart that asked for them
/// (`03-design/jaimini-significators.md`).
fn chart_jaimini_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "jaimini",
        "Jaimini's significators, a row a chart, charts outermost: the karakamsha and the Brahma graha under the settings' `jaimini` group (BPHS ch. 33 v. 1, ch. 46 vv. 170 to 173). Empty when they were not asked for.",
        vec![
            ColumnDef::new(
                "atmakaraka",
                Scalar::U16,
                "The Atmakaraka, under `jaimini.chara_karakas`.",
            )
            .of_enum("Graha"),
            ColumnDef::new(
                "karakamsha",
                Scalar::U16,
                "The karakamsha: the Atmakaraka's navamsha sign.",
            )
            .of_enum("Rashi"),
            ColumnDef::new(
                "brahma_rule",
                Scalar::U8,
                "The rule the Brahma graha was sought under, `jaimini.brahma`.",
            )
            .of_enum("TsBrahmaRule"),
            ColumnDef::new(
                "counted_from",
                Scalar::U16,
                "The stronger of the lagna and the 7th, which the rule counts from.",
            )
            .of_enum("Rashi"),
            ColumnDef::new(
                "qualified",
                Scalar::U16,
                "The planets that met the rule's marks, a bit set: bit `n` is the graha with id `n`.",
            ),
            ColumnDef::new(
                "brahma",
                Scalar::U16,
                "The Brahma graha; read only when `brahma_outcome` is `FOUND`.",
            )
            .of_enum("Graha"),
            ColumnDef::new(
                "brahma_outcome",
                Scalar::U8,
                "Whether the Brahma graha was found, and when not, why.",
            )
            .of_enum("TsBrahmaOutcome"),
            ColumnDef::new(
                "passed_from",
                Scalar::U16,
                "Saturn or the node that qualified and passed Brahma-hood to the planet in the 6th from it (C127); read only when `passed_from_present` is 1.",
            )
            .of_enum("Graha"),
            ColumnDef::new(
                "passed_from_present",
                Scalar::U8,
                "1 when Brahma-hood was passed on, else 0.",
            ),
        ],
    )
}

/// Every chart's grahas as Jaimini reads them, a row a graha: the house from
/// the karakamsha in both charts, and the graha's arudha.
fn chart_jaimini_grahas_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "jaimini_grahas",
        "Each graha as Jaimini reads it, the Sun to Ketu, charts outermost: row `i * 9 + g` is the `i`th chart in `jaimini`, graha `g`. Its house from the karakamsha in the rasi chart and in the navamsha, since the schools part on which (C130), and its arudha (BPHS ch. 29 vv. 6 and 7).",
        vec![
            ColumnDef::new("graha", Scalar::U16, "Which graha.").of_enum("Graha"),
            ColumnDef::new(
                "in_rasi",
                Scalar::U8,
                "Its house from the karakamsha in the rasi chart, 1 to 12.",
            ),
            ColumnDef::new(
                "in_navamsha",
                Scalar::U8,
                "Its house from the karakamsha in the navamsha, 1 to 12.",
            ),
            ColumnDef::new(
                "arudha",
                Scalar::U16,
                "Its arudha under `jaimini.graha_arudha_exception`; read only when `arudha_present` is 1.",
            )
            .of_enum("Rashi"),
            ColumnDef::new(
                "arudha_present",
                Scalar::U8,
                "1 when it has an arudha; 0 for a node that owns no sign under `jaimini.node_co_lordship` (C133).",
            ),
        ],
    )
}

/// Each chart's own content hash.
fn chart_content_hashes_section(id: u32) -> SectionSchema {
    content_hashes_section(
        id,
        "chart",
        "its document, with what it answers by rule where rules were asked",
    )
}

/// The annual charts a batch's births open and everything Tajika reads
/// from them, in id order: fifteen sections ragged under one another, so
/// they are declared together rather than scattered through the rest.
fn chart_annual_sections() -> [SectionSchema; 15] {
    [
        chart_praveshas_section(35),
        chart_annual_charts_section(36),
        chart_year_claims_section(37),
        chart_year_yogas_section(38),
        chart_year_matters_section(39),
        chart_matter_yogas_section(40),
        chart_matter_legs_section(41),
        saham_section(
            42,
            "year_sahams",
            "Every annual chart's sahams, concatenated in the `annual_charts` section's order and **ragged** by its `saham_count`, each year's in the order `varsha_json.sahams` named them. A saham is a − b + c from the year's own chart, carried a sign further where c does not fall between b and a, each read under `varsha_json.sahamRules` (`03-design/tajika-sahams.md`), and judged for strength under the year's own lord (`03-design/tajika-saham-strength.md`). Whether the year opened by day, which chooses each saham's night formula, is `annual_charts.daylight`. Empty unless sahams were asked for and a place given.",
        ),
        saham_seven_section(43, "year_saham_seven", "year_sahams"),
        chart_year_harsha_section(44),
        saham_section(
            45,
            "natal_sahams",
            "Every birth chart's own sahams, concatenated in the `cast` section's order and **ragged** by its `natal_saham_count`, each chart's in the order `varsha_json.sahams` named them, with their strength — which has no year lord, so `with_year_lord` never holds here. The source reads a year's sahams beside the birth's: \"only those Sahams which are strong in the birth chart can produce results during a given year\". Answered with or without a place; empty unless sahams were asked for.",
        ),
        saham_seven_section(46, "natal_saham_seven", "natal_sahams"),
        chart_year_dashas_section(47),
        chart_year_dasha_shares_section(48),
        dasha_periods_section(
            49,
            "year_dasha_periods",
            "Every annual dasha's periods, concatenated in the `year_dashas` section's order and **ragged** by its `period_count`, each depth first in time order from the year's return to its close: a mahadasha, then its antardashas, then the next mahadasha, to `varsha_json.dashaRules.depth` levels. A period that runs for no time is not listed, and its place is kept in the others' `index`.",
            SignedBy::Period,
        ),
    ]
}

/// What every chart answered by rule, as canonical JSON.
fn chart_rules_section(id: u32) -> SectionSchema {
    SectionSchema::bytes(
        id,
        "rules",
        "UTF-8 JSON, canonical: an array with one entry per chart, each the rules the request's `rules_json` named that held on it — `present`, each `{rule, result}` with the rule by key — with `houses` and `longevity` when asked, and `unreadable` naming an input a rule named that the chart could not have (`03-design/rules-at-the-boundary.md`). Empty when no rules were asked for.",
    )
}

/// What every chart has to say, as narrative plans holding no words.
fn chart_plans_section(id: u32) -> SectionSchema {
    SectionSchema::bytes(
        id,
        "plans",
        "UTF-8 JSON, canonical: an array with one entry per chart, each an object carrying the narrative plans the request's `interpret_json` asked for — `placements`, `readings`, `strength`, `houses`, `positions`, `aspects` — and only those. A plan is the array of its items, each `{key, params}`, and its params are the very JSON `ts_intl_render` takes, so a binding says an item by handing it straight back (`03-design/plans-at-the-boundary.md`). Empty when no composer was asked for.",
    )
}

/// Every chart's annual charts: the instants the Sun returns to where it
/// stood at birth.
/// Each year's own chart, when `varsha_json.place` asked for them.
fn chart_annual_charts_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "annual_charts",
        "Each return's own chart, founded where `varsha_json.place` said — `\"birth\"` or a residence — and read down to what Tajika reads from it: row for row beside the `praveshas` section when a place was asked for, and **empty** when none was, never partly filled. The Muntha's lord, the first office-bearer, is `praveshas.muntha_lord` and is not repeated here (`03-design/muntha.md`).",
        vec![
            ColumnDef::new(
                "lagna_deg",
                Scalar::F64,
                "The annual chart's lagna, sidereal degrees, at the place it was cast for.",
            ),
            ColumnDef::new(
                "daylight",
                Scalar::U8,
                "1 when the return falls between sunrise and sunset at that place, 0 when by night: what chooses the Tri-Rashi and Dina-Ratri lords.",
            ),
            ColumnDef::new(
                "janma_lagna_lord",
                Scalar::U16,
                "The birth lagna's lord, a `graha` id: the Janmesha.",
            ),
            ColumnDef::new(
                "varsha_lagna_lord",
                Scalar::U16,
                "The annual lagna's lord, a `graha` id: the Varsha Lagnesha.",
            ),
            ColumnDef::new(
                "tri_rashi_lord",
                Scalar::U16,
                "The annual lagna's Tri-Rashi lord for the part of the day, a `graha` id: the Dorothean triplicity lords under the source's positional rule (crux C108).",
            ),
            ColumnDef::new(
                "dina_ratri_lord",
                Scalar::U16,
                "The lord of the Sun's sign by day or the Moon's by night, a `graha` id: the Dina-Ratri Pati.",
            ),
            ColumnDef::new(
                "year_lord",
                Scalar::U16,
                "The **Varshesha**, lord of the year, a `graha` id: the strongest office-bearer that aspects the annual lagna, with the source's fallbacks (`03-design/varshesha.md`).",
            ),
            ColumnDef::new(
                "year_lord_chosen",
                Scalar::U8,
                "Which step of the chain decided the year's lord. A year lord reached by a fallback is a different statement about the year from one chosen on strength, and the planet alone cannot say so.",
            )
            .of_enum("TsVarsheshaChosen"),
            ColumnDef::new(
                "year_lord_vishwa",
                Scalar::I32,
                "The year lord's five-fold strength, exact, in **sub-sub units** of which a unit holds 3600 — an integer because two office-bearers a sub-sub unit apart decide a year between them.",
            ),
            ColumnDef::new(
                "moon_passed_over",
                Scalar::U8,
                "1 when the Moon led on strength and stepped aside, being \"unable to govern\"; 0 otherwise.",
            ),
            ColumnDef::new(
                "claim_count",
                Scalar::U8,
                "How many rows of the `year_claims` section belong to this year: one to five, the distinct office-bearers.",
            ),
            ColumnDef::new(
                "yoga_count",
                Scalar::U8,
                "How many rows of the `year_yogas` section belong to this year: the pairs of the seven that make an Ithasala or an Ishrafa, 0 to 21.",
            ),
            ColumnDef::new(
                "retrograde",
                Scalar::U8,
                "The seven that are retrograde in this year's chart, as a bit set: bit `n` is the graha with catalogue id `n`. What the matters' yogas were judged on.",
            ),
            ColumnDef::new(
                "combust",
                Scalar::U8,
                "The seven that are combust in this year's chart, under the context's combustion table, as a bit set like `retrograde`.",
            ),
            ColumnDef::new(
                "matter_count",
                Scalar::U8,
                "How many rows of the `year_matters` section belong to this year: the matters `varsha_json.matters` asked about, 0 to 12.",
            ),
            ColumnDef::new(
                "saham_count",
                Scalar::U8,
                "How many rows of the `year_sahams` section belong to this year: the sahams `varsha_json.sahams` asked for, 0 to 41.",
            ),
            ColumnDef::new(
                "dasha_count",
                Scalar::U8,
                "How many rows of the `year_dashas` section belong to this year: the annual dashas `varsha_json.dashas` asked for, 0 to 3.",
            ),
        ],
    )
}

/// Every year's annual dashas: which, what seeds it, and where its ring
/// opens.
fn chart_year_dashas_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "year_dashas",
        "Every annual chart's annual dashas, concatenated in the `annual_charts` section's order and **ragged** by its `dasha_count`, each year's in the order `varsha_json.dashas` named them, read under `varsha_json.dashaRules` (`03-design/annual-dashas.md`). Each runs round a ring of lords (the next `share_count` rows of `year_dasha_shares`) from `first`, and its periods are the next `period_count` rows of `year_dasha_periods`. Empty unless annual dashas were asked for and a place given.",
        vec![
            ColumnDef::new(
                "system",
                Scalar::U16,
                "Which: the Patyayini, the Mudda or the Varsha Yogini.",
            )
            .of_enum("DashaSystem"),
            ColumnDef::new(
                "seeded",
                Scalar::U8,
                "1 when the birth nakshatra seeds the year and `seed` names it: the Mudda and the Varsha Yogini; 0 for the Patyayini, which is read from the year's own chart, and `seed` is zero.",
            ),
            ColumnDef::new(
                "seed",
                Scalar::U16,
                "The birth Moon's nakshatra, when `seeded`.",
            )
            .of_enum("Nakshatra"),
            ColumnDef::new(
                "first",
                Scalar::U8,
                "The place in the ring the year opens with, from 0: for a nakshatra year the birth nakshatra's lord advanced one for each completed year.",
            ),
            ColumnDef::new(
                "remaining",
                Scalar::F64,
                "How much of the first lord's share was still to run when the year opened, 0 to 1; the rest closes the year. NaN when the first lord runs its whole share from the return and the year ends with the lord before it: the Patyayini, and a balance of `whole`.",
            ),
            ColumnDef::new(
                "from_jd",
                Scalar::F64,
                "When the year opens: its return, a Julian day (UTC).",
            ),
            ColumnDef::new(
                "to_jd",
                Scalar::F64,
                "When the year closes, a Julian day (UTC): under the default clock the next return, as the Sun's own crossing of its return longitude.",
            ),
            ColumnDef::new(
                "share_count",
                Scalar::U8,
                "How many rows of `year_dasha_shares` are this dasha's ring: 9 for the Mudda, 8 for the Varsha Yogini and the Patyayini.",
            ),
            ColumnDef::new(
                "period_count",
                Scalar::U32,
                "How many rows of `year_dasha_periods` are this dasha's.",
            ),
        ],
    )
}

/// The lords a year's dasha runs round, and each one's share of it.
fn chart_year_dasha_shares_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "year_dasha_shares",
        "Every annual dasha's ring, concatenated in the `year_dashas` section's order and **ragged** by its `share_count`, in the order the ring runs: a lord's share of the year is its weight over the ring's.",
        vec![
            ColumnDef::new(
                "lord",
                Scalar::U16,
                "Its lord: the graha, or the lord of the sign when the share is a sign's.",
            )
            .of_enum("Graha"),
            ColumnDef::new(
                "has_sign",
                Scalar::U8,
                "1 when the share is a sign's and `sign` names it: the Patyayini's lagna; 0 for a planet's, and `sign` is zero.",
            ),
            ColumnDef::new("sign", Scalar::U16, "The sign, when `has_sign`.").of_enum("Rashi"),
            ColumnDef::new(
                "weight",
                Scalar::F64,
                "Its weight: a nakshatra year's lord's natal years, or a Patyayini share's patyamsha — its krishamsha less the one before it — in nanoarcseconds, exactly. Only the ratios matter. 0 for a lord tied with the one before it, which runs for no time and has no period.",
            ),
        ],
    )
}

/// A chart's sahams: where each fell, what it fell in, and its strength
/// clause by clause. One shape for the years' and the births', so every
/// binding decodes a saham in one place.
fn saham_section(id: u32, name: &str, doc: &str) -> SectionSchema {
    SectionSchema::columns(
        id,
        name,
        doc,
        vec![
            ColumnDef::new("saham", Scalar::U8, "Which of the forty-one.").of_enum("TsSaham"),
            ColumnDef::new(
                "longitude_deg",
                Scalar::F64,
                "Where it fell, sidereal degrees in [0, 360).",
            ),
            ColumnDef::new("sign", Scalar::U16, "The sign it fell in, a `rashi` id."),
            ColumnDef::new(
                "lord",
                Scalar::U16,
                "That sign's lord, a `graha` id: the saham's lord, by whose strength the source judges it.",
            ),
            ColumnDef::new(
                "house",
                Scalar::U8,
                "The house it fell in, 1 to 12, counted from the chart's lagna by whole signs. The 6th, 8th and 12th are where the source calls a saham handicapped.",
            ),
            ColumnDef::new(
                "added_sign",
                Scalar::U8,
                "1 when it was carried a sign further because c did not fall between b and a, under the request's `addSign` rule; 0 otherwise.",
            ),
            ColumnDef::new(
                "strong",
                Scalar::U16,
                "The clauses of the source's strong list that hold, as a bit set: bit `n` is the `TsSahamStrong` with id `n`. Reported and never weighed: the source judges in words and gives no score (`03-design/tajika-saham-strength.md`).",
            ),
            ColumnDef::new(
                "weak",
                Scalar::U8,
                "The clauses of the source's weak list that hold, as a bit set over `TsSahamWeak`. A saham may meet clauses on both lists, and three in five do.",
            ),
            ColumnDef::new(
                "lord_vishwa",
                Scalar::I32,
                "The saham lord's Panchavargiya Vishwa bala, exact, in sub-sub units of which a unit holds 3600.",
            ),
            ColumnDef::new(
                "lord_harsha",
                Scalar::U8,
                "The saham lord's Harsha bala grade.",
            )
            .of_enum("TsHarshaGrade"),
            ColumnDef::new(
                "node_axis",
                Scalar::U8,
                "1 when the saham's sign is Rahu's or Ketu's, which the source's forty-sixth year counts against a saham; 0 when not; 2 when the chart placed no nodes to read.",
            ),
        ],
    )
}

/// The seven planets' facts under each saham of `of`: seven rows a saham,
/// the catalogue's order, so a reader indexes them by the saham's row.
fn saham_seven_section(id: u32, name: &str, of: &str) -> SectionSchema {
    SectionSchema::columns(
        id,
        name,
        &format!("Seven rows under each row of `{of}`, one for each of the seven in the catalogue's order — **fixed, not ragged**, so a saham's rows start at its row times seven: how each planet stands to the saham, which is what the strength clauses were read from."),
        vec![
            ColumnDef::new("graha", Scalar::U16, "Which of the seven, a `graha` id."),
            ColumnDef::new(
                "drishti",
                Scalar::U8,
                "The Tajika aspect its sign casts on the saham's; one in the saham's own sign casts the inimical aspect and is also its company.",
            )
            .of_enum("TsTajikaDrishti"),
            ColumnDef::new(
                "relation",
                Scalar::U8,
                "How it stands to the saham's lord, under the request's friendship.",
            )
            .of_enum("TsTajikaRelation"),
            ColumnDef::new(
                "company",
                Scalar::U8,
                "1 when it stands in the saham's sign: the saham's company.",
            ),
        ],
    )
}

/// Every founded year's Harsha bala, seven rows a year.
fn chart_year_harsha_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "year_harsha",
        "Seven rows under each row of `annual_charts`, one for each of the seven in the catalogue's order — **fixed, not ragged**: each planet's Harsha bala, four places it is \"happy\" in, five units each (`03-design/tajika-harsha.md`), read under `varsha_json.harshaRules`. Empty when no place was asked for.",
        vec![
            ColumnDef::new("graha", Scalar::U16, "Which of the seven, a `graha` id."),
            ColumnDef::new(
                "house",
                Scalar::U8,
                "The house it stands in, whole signs from the annual lagna.",
            ),
            ColumnDef::new(
                "sthana",
                Scalar::U8,
                "1 in its house of joy: the first part.",
            ),
            ColumnDef::new(
                "uchcha_swakshetra",
                Scalar::U8,
                "1 in its exaltation or own sign: the second part.",
            ),
            ColumnDef::new(
                "stri_purusha",
                Scalar::U8,
                "1 in a house of its own gender, Tajika's genders: the third part.",
            ),
            ColumnDef::new(
                "dina_ratri",
                Scalar::U8,
                "1 in a year opening at its own part of the day: the fourth part.",
            ),
            ColumnDef::new(
                "total",
                Scalar::U8,
                "The parts held, five units each: 0 to 20.",
            ),
            ColumnDef::new("grade", Scalar::U8, "What the source calls that total.")
                .of_enum("TsHarshaGrade"),
        ],
    )
}

/// The seven columns every pair the matter sections carry is written in:
/// `year_yogas`' own, with a presence flag on the yoga, because a pair
/// here may make none. `prefix` names them where a section carries a
/// pair beside other fields.
fn pair_columns(prefix: &str) -> Vec<ColumnDef> {
    let named = |name: &str| format!("{prefix}{name}");
    vec![
        ColumnDef::new(
            &named("faster"),
            Scalar::U16,
            "The faster of the two by the tradition's ranking — Moon, Mercury, Venus, Sun, Mars, Jupiter, Saturn — a `graha` id.",
        ),
        ColumnDef::new(&named("slower"), Scalar::U16, "The slower of the two, a `graha` id."),
        ColumnDef::new(
            &named("drishti"),
            Scalar::U8,
            "The Tajika aspect between the signs they stand in; `NONE` where they stand in the neutral houses.",
        )
        .of_enum("TsTajikaDrishti"),
        ColumnDef::new(
            &named("yoga"),
            Scalar::U8,
            "What they are doing, read only when the yoga is present.",
        )
        .of_enum("TsTajikaYoga"),
        ColumnDef::new(
            &named("yoga_present"),
            Scalar::U8,
            "1 when they make an Ithasala or an Ishrafa; 0 when they make neither.",
        ),
        ColumnDef::new(
            &named("orb_deg"),
            Scalar::F64,
            "The orb governing the pair, degrees: the mean of their two deeptamshas.",
        ),
        ColumnDef::new(
            &named("apart_deg"),
            Scalar::F64,
            "How far apart they stand within their signs, degrees: positive when the faster is behind the slower and coming to it, negative when it is past.",
        ),
    ]
}

/// Every year's matters: the question each asked, and the pair it names.
fn chart_year_matters_section(id: u32) -> SectionSchema {
    let mut fields = vec![
        ColumnDef::new(
            "house",
            Scalar::U8,
            "The house asked about, 1 to 12, counted from the annual lagna by whole signs.",
        ),
        ColumnDef::new(
            "sign",
            Scalar::U16,
            "The sign that house falls in, a `rashi` id.",
        ),
        ColumnDef::new(
            "lagnesha",
            Scalar::U16,
            "The lord of the annual lagna, a `graha` id.",
        ),
        ColumnDef::new(
            "karyesha",
            Scalar::U16,
            "The lord of the house asked about, a `graha` id.",
        ),
        ColumnDef::new(
            "same_lord",
            Scalar::U8,
            "1 when one planet is both lords — always so of the first house — and there is no pair to judge; the `pair_*` columns are then read not at all.",
        ),
    ];
    fields.extend(pair_columns("pair_"));
    fields.extend([
        ColumnDef::new(
            "unanswered",
            Scalar::U16,
            "The yogas this call could not answer for, as a bit set: bit `n` is the `TsYearYoga` with id `n`. A yoga absent from `matter_yogas` did not hold **only** if it is not here.",
        ),
        ColumnDef::new(
            "held_count",
            Scalar::U8,
            "How many rows of the `matter_yogas` section belong to this matter: the yogas that hold, one row each time one holds.",
        ),
    ]);
    SectionSchema::columns(
        id,
        "year_matters",
        "Every annual chart's matters, concatenated in the `annual_charts` section's order and **ragged** by its `matter_count`, each year's in the order `varsha_json.matters` named them. Fourteen of the sixteen Tajika yogas are judgements about the lagnesha and the karyesha, so each row is the question as well as where its answer starts (`03-design/tajika-yogas.md`). Empty unless matters were asked for.",
        fields,
    )
}

/// Every matter's yogas that hold, and what made each hold.
fn chart_matter_yogas_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "matter_yogas",
        "Every matter's yogas that hold, concatenated in the `year_matters` section's order and **ragged** by its `held_count`. A yoga may hold more than once in a matter, once for each third planet that makes it.",
        vec![
            ColumnDef::new("yoga", Scalar::U8, "Which of the sixteen.").of_enum("TsYearYoga"),
            ColumnDef::new(
                "by_pair",
                Scalar::U8,
                "1 when the lords' own relation, the matter's `pair_*`, is what made it: an Ithasala or an Ishrafa, and the judgements upon an Ithasala.",
            ),
            ColumnDef::new(
                "through",
                Scalar::U16,
                "The third planet it turns on, a `graha` id, read only when `through_present`: the one that carried or gathered the light, the malefic, the Moon, or the strong planet a lord is drawn to.",
            ),
            ColumnDef::new(
                "through_present",
                Scalar::U8,
                "1 when there is a third planet.",
            ),
            ColumnDef::new(
                "entering",
                Scalar::U16,
                "The planet judged on entering the next sign, a `graha` id, read only when `entering_present`: Gairi-Kamboola's Moon or Tambira's lord at a sign's end. Its legs are then read from the next sign's first degree.",
            ),
            ColumnDef::new(
                "entering_present",
                Scalar::U8,
                "1 when a planet was judged on entering the next sign.",
            ),
            ColumnDef::new(
                "afflictions_present",
                Scalar::U8,
                "1 when the lords' afflictions are what made it: Rudda and Durapha.",
            ),
            ColumnDef::new(
                "lagnesha_afflictions",
                Scalar::U8,
                "The lagnesha's afflictions, as a bit set: bit `n` is the `TsAffliction` with id `n`. Read only when `afflictions_present`.",
            ),
            ColumnDef::new(
                "karyesha_afflictions",
                Scalar::U8,
                "The karyesha's afflictions, as a bit set like `lagnesha_afflictions`.",
            ),
            ColumnDef::new(
                "leg_count",
                Scalar::U8,
                "How many rows of the `matter_legs` section belong to this yoga: none, or two — how the third planet stands to each of the pair.",
            ),
        ],
    )
}

/// Every held yoga's legs: how its third planet stands to each of the pair.
fn chart_matter_legs_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "matter_legs",
        "Every held yoga's legs, concatenated in the `matter_yogas` section's order and **ragged** by its `leg_count`: how the third planet stands to each of the pair, or, for a planet entering the next sign, to its partner and to the strong third it reaches, read from where it will stand.",
        pair_columns(""),
    )
}

/// Every annual chart's pairs that make a Tajika yoga.
fn chart_year_yogas_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "year_yogas",
        "Every annual chart's pairs of the seven that make a yoga — an Ithasala in one of its three kinds, coming together, or an Ishrafa, drawing apart — concatenated in the `annual_charts` section's order and **ragged** by its `yoga_count`. Empty when no place was asked for. The pairs that make none are the rest of the twenty-one and do not cross; a Rust caller has `sdk.chart().drishtis` for all of them (`03-design/tajika-aspects.md`).",
        vec![
            ColumnDef::new(
                "faster",
                Scalar::U16,
                "The faster of the two by the tradition's ranking — Moon, Mercury, Venus, Sun, Mars, Jupiter, Saturn — a `graha` id.",
            ),
            ColumnDef::new("slower", Scalar::U16, "The slower of the two, a `graha` id."),
            ColumnDef::new(
                "drishti",
                Scalar::U8,
                "The Tajika aspect between the signs they stand in. A pair in the neutral houses makes no yoga however close, so this is never `NONE` here.",
            )
            .of_enum("TsTajikaDrishti"),
            ColumnDef::new(
                "yoga",
                Scalar::U8,
                "What they are doing: one of the Ithasala's three kinds, coming together, or an Ishrafa, drawing apart.",
            )
            .of_enum("TsTajikaYoga"),
            ColumnDef::new(
                "orb_deg",
                Scalar::F64,
                "The orb governing the pair, degrees: the **mean** of their two deeptamshas.",
            ),
            ColumnDef::new(
                "apart_deg",
                Scalar::F64,
                "How far apart they stand **within their signs**, degrees, the completed signs deleted as the tradition counts them: positive when the faster is behind the slower and coming to it, negative when it is past.",
            ),
        ],
    )
}

/// Every year's claimants on the lordship, and what each was judged on.
fn chart_year_claims_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "year_claims",
        "Every annual chart's claimants on the year's lordship, concatenated in the `annual_charts` section's order and **ragged** by its `claim_count`, each year's ranked strongest first. Empty when no place was asked for. This is the reckoning the year lord came out of, so a reader can see the decision rather than take it on trust (`03-design/varshesha.md`).",
        vec![
            ColumnDef::new("graha", Scalar::U16, "The claimant, a `graha` id."),
            ColumnDef::new(
                "vishwa",
                Scalar::I32,
                "Its five-fold strength, exact, in sub-sub units of which a unit holds 3600.",
            ),
            ColumnDef::new(
                "portfolios",
                Scalar::U8,
                "How many of the five offices it holds, 1 to 5: the tie-break when two are level on strength.",
            ),
            ColumnDef::new(
                "aspects_lagna",
                Scalar::U8,
                "1 when it gives the Tajika aspect to the annual lagna, which it must to hold the year; 0 when it stands in a neutral house — 2, 6, 8 or 12 — and is disqualified however strong.",
            ),
        ],
    )
}

fn chart_praveshas_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "praveshas",
        "Every chart's annual-chart instants, concatenated in the `cast` section's order and **ragged** by its `pravesha_count`, each chart's in year order. The reading is the batch's, from the request's `varsha_json`, as the dashas asked for are (`03-design/annual-chart.md`). Empty when no annual charts were asked for.",
        vec![
            ColumnDef::new(
                "year",
                Scalar::U16,
                "How many years the native has completed at this instant: 1 is the first return, a year after birth. Counted in returns and not in years of life, because the two namings differ by one and both are in use.",
            ),
            ColumnDef::new(
                "jd",
                Scalar::F64,
                "The instant, a Julian day (UTC). A chart cast for it is the annual chart; the place is the caller's, which is why the boundary answers the instant and not the chart.",
            ),
            ColumnDef::new(
                "muntha_sign",
                Scalar::U16,
                "The Muntha's sign at this return, a `rashi` id: the birth lagna's sign advanced one sign for each completed year. Both readings of the Muntha's degree give this same sign.",
            ),
            ColumnDef::new(
                "muntha_lord",
                Scalar::U16,
                "The lord of the Muntha's sign, a `graha` id: the Munthesha, first of the annual chart's five office-bearers and the one that takes the year's lordship when no other qualifies.",
            ),
            ColumnDef::new(
                "muntha_deg",
                Scalar::F64,
                "The Muntha's longitude at this return, degrees, under the `muntha` reading the request asked for. It advances 30 degrees over the year, so a caller timing within the year interpolates from here.",
            ),
        ],
    )
}

/// Twelve bhavas a chart, each its centre and opening cusp: the `houses` and
/// the `chalit` share the shape.
fn chart_cusps_section(id: u32, name: &str, doc: &str) -> SectionSchema {
    SectionSchema::columns(
        id,
        name,
        doc,
        vec![
            ColumnDef::new("madhya_deg", Scalar::F64, "The bhava's centre, degrees."),
            ColumnDef::new(
                "sandhi_deg",
                Scalar::F64,
                "The bhava's opening cusp, degrees.",
            ),
        ],
    )
}

/// Every chart's dashas: one row a chart a system, charts outermost and the
/// systems in the order asked (`03-design/dasha-kernels.md`).
fn chart_dashas_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "dashas",
        "Every chart's dashas, charts outermost and then the systems in the order asked: row `i * dasha_count + j` is chart `i`'s `j`th. Each row's periods are the next `period_count` rows of `dasha_periods`, in the same order. Empty when no dashas were asked for.",
        vec![
            ColumnDef::new(
                "system",
                Scalar::U16,
                "Which system: a catalogue id, or at `0x8000` and up the id of a system the context registered, which `ts_key_name` names.",
            )
            .of_enum("DashaSystem"),
            ColumnDef::new(
                "seeded",
                Scalar::U8,
                "1 when a nakshatra seeds the dasha and it has a balance at birth: then `seed`, `overflow` and the balance columns are its; 0 for a sign-based dasha, whose first period runs whole from birth, and those columns are zero.",
            ),
            ColumnDef::new(
                "signed",
                Scalar::U8,
                "1 when every period is a sign's, and `dasha_periods.sign` names it; 0 when the periods are their lords' and that column is zero.",
            ),
            ColumnDef::new(
                "seed",
                Scalar::U16,
                "The nakshatra the Moon stood in, which seeds it; zero unless `seeded`.",
            )
            .of_enum("Nakshatra"),
            ColumnDef::new("first_lord", Scalar::U16, "The lord it starts with.").of_enum("Graha"),
            ColumnDef::new(
                "overflow",
                Scalar::U8,
                "1 when the seed lay outside a conditional system's nakshatras and started at the first lord because the settings let it.",
            ),
            ColumnDef::new("balance", Scalar::U8, "How the balance was measured.")
                .of_enum("TsBalance"),
            ColumnDef::new(
                "remaining",
                Scalar::F64,
                "The fraction of the first lord's period still to run at birth, 0 to 1.",
            ),
            ColumnDef::new(
                "balance_days",
                Scalar::F64,
                "That fraction of the first lord's years, in days.",
            ),
            ColumnDef::new(
                "balance_years",
                Scalar::U32,
                "The balance's whole years of the year length.",
            ),
            ColumnDef::new(
                "balance_months",
                Scalar::U8,
                "Its whole months of a twelfth of the year length.",
            ),
            ColumnDef::new("balance_day_count", Scalar::U8, "Its whole days."),
            ColumnDef::new(
                "balance_hours",
                Scalar::U8,
                "Its hours, the rest rounded to the minute.",
            ),
            ColumnDef::new("balance_minutes", Scalar::U8, "Its minutes, rounded."),
            ColumnDef::new(
                "moon_span_from",
                Scalar::F64,
                "When the Moon entered its nakshatra, a Julian day (UTC); NaN when the balance was spatial and read no span.",
            ),
            ColumnDef::new(
                "moon_span_to",
                Scalar::F64,
                "When it left, a Julian day (UTC); NaN when no span was read.",
            ),
            ColumnDef::new(
                "depth",
                Scalar::U8,
                "How many levels the periods go down, 1 to 6.",
            ),
            ColumnDef::new(
                "period_count",
                Scalar::U32,
                "How many rows of `dasha_periods` are this dasha's.",
            ),
        ],
    )
}

/// Every chart's Ashtakavarga, a row a graha.
fn chart_ashtakavarga_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "ashtakavarga",
        "Every chart's Ashtakavarga, a row a graha, Sun to Saturn, charts outermost: row `i * 7 + g` is chart `i`'s `g`th graha. Its bindus are the `ashtakavarga_bindus` rows `(i * 7 + g) * 12` to the next eleven, and its chart's sums the `sarvashtakavarga` rows `i * 12` to the next eleven. Empty when the Ashtakavarga was not asked for (`03-design/ashtakavarga-measured.md`).",
        vec![
            ColumnDef::new("graha", Scalar::U16, "Which graha.").of_enum("Graha"),
            ColumnDef::new(
                "shodhana",
                Scalar::U8,
                "Where the reductions and pindas were made; `reduced` in `ashtakavarga_bindus` is zero unless in each graha's own.",
            )
            .of_enum("TsShodhana"),
            ColumnDef::new(
                "ekadhipatya",
                Scalar::U8,
                "How a co-ruled sign beside an occupied one was reduced.",
            )
            .of_enum("TsEkadhipatya"),
            ColumnDef::new("rashi_pinda", Scalar::U32, "Its rashi pinda."),
            ColumnDef::new("graha_pinda", Scalar::U32, "Its graha pinda."),
            ColumnDef::new("yoga_pinda", Scalar::U32, "Its yoga pinda, the two together."),
        ],
    )
}

/// Every graha's bindus by sign.
fn chart_ashtakavarga_bindus_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "ashtakavarga_bindus",
        "Every graha's bindus by sign, Aries to Pisces, in the `ashtakavarga` section's order: twelve rows a graha. Empty when the Ashtakavarga was not asked for.",
        vec![
            ColumnDef::new("bindus", Scalar::U8, "Its bindus in the sign, 0 to 8."),
            ColumnDef::new(
                "reduced",
                Scalar::U8,
                "The same after both reductions, when they were made in each graha's own Ashtakavarga; zero otherwise.",
            ),
        ],
    )
}

/// Every chart's sarvashtakavarga by sign.
fn chart_sarvashtakavarga_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "sarvashtakavarga",
        "Every chart's sums by sign, Aries to Pisces, charts outermost: twelve rows a chart. Empty when the Ashtakavarga was not asked for.",
        vec![
            ColumnDef::new(
                "sarva",
                Scalar::U16,
                "The seven grahas' bindus in the sign.",
            ),
            ColumnDef::new("trikona", Scalar::U16, "The sum after the trine reduction."),
            ColumnDef::new("reduced", Scalar::U16, "The sum after both reductions."),
        ],
    )
}

/// Every chart's Vimshopaka, a row a graha.
fn chart_vimshopaka_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "vimshopaka",
        "Every chart's Vimshopaka, a row a graha, Sun to Saturn, charts outermost: row `i * 7 + g` is chart `i`'s `g`th graha, each score out of 20. Empty when the Vimshopaka was not asked for (`03-design/vimshopaka-measured.md`).",
        vec![
            ColumnDef::new("graha", Scalar::U16, "Which graha.").of_enum("Graha"),
            ColumnDef::new("scoring", Scalar::U8, "How each varga was scored.")
                .of_enum("TsVimshopakaScoring"),
            ColumnDef::new("shadvarga", Scalar::F64, "Over the six vargas."),
            ColumnDef::new("saptavarga", Scalar::F64, "Over the seven."),
            ColumnDef::new("dashavarga", Scalar::F64, "Over the ten."),
            ColumnDef::new("shodashavarga", Scalar::F64, "Over the sixteen."),
        ],
    )
}

/// Every chart's Shadbala, a row a graha, its value columns from the table
/// the writer reads.
fn chart_shadbala_section(id: u32) -> SectionSchema {
    let mut columns = vec![ColumnDef::new("graha", Scalar::U16, "Which graha.").of_enum("Graha")];
    columns.extend(
        crate::chart::SHADBALA_COLUMNS
            .iter()
            .map(|(name, doc, _)| ColumnDef::new(name, Scalar::F64, doc)),
    );
    columns.push(ColumnDef::new(
        "strong",
        Scalar::U8,
        "1 when the rupas reach the requirement, else 0.",
    ));
    SectionSchema::columns(
        id,
        "shadbala",
        "Every chart's Shadbala in virupas, a row a graha, Sun to Saturn, charts outermost: row `i * 7 + g` is chart `i`'s `g`th graha. Read under the context's `strength.*` settings, which the provenance carries. Empty when the Shadbala was not asked for (`03-design/shadbala-measured.md`).",
        columns,
    )
}

/// Every chart's Bhava bala, a row a bhava, its value columns from the table
/// the writer reads.
fn chart_bhava_bala_section(id: u32) -> SectionSchema {
    let mut columns = vec![
        ColumnDef::new(
            "lord",
            Scalar::U16,
            "The lord of the sign its madhya falls in.",
        )
        .of_enum("Graha"),
    ];
    columns.extend(
        crate::chart::BHAVA_BALA_COLUMNS
            .iter()
            .map(|(name, doc, _)| ColumnDef::new(name, Scalar::F64, doc)),
    );
    SectionSchema::columns(
        id,
        "bhava_bala",
        "Every chart's Bhava bala in virupas, a row a bhava, the first to the twelfth, charts outermost: row `i * 12 + h` is chart `i`'s bhava `h + 1`. Read under the context's `strength.bhava_*` settings, which the provenance carries. Empty when the Bhava bala was not asked for (`03-design/bhava-bala-measured.md`).",
        columns,
    )
}

/// Every chart's dasha phala, a row a graha: the Subhanka in each of the
/// seven vargas, their totals, and what ch. 47 reads of the placement.
fn chart_dasha_phala_section(id: u32) -> SectionSchema {
    let mut columns = vec![ColumnDef::new("graha", Scalar::U16, "Which graha.").of_enum("Graha")];
    columns.extend(
        teistro::strength::shadbala::SAPTAVARGAJA_VARGAS
            .iter()
            .enumerate()
            .map(|(k, varga)| {
                let out_of = if k == 0 { 60 } else { 30 };
                ColumnDef::new(
                    &format!("subhanka_{}", varga.key().to_lowercase()),
                    Scalar::F64,
                    &format!(
                        "Its Subhanka in the {}, out of {out_of}: the points of its dignity there (BPHS ch. 28 vv. 7 to 9).",
                        varga.key()
                    ),
                )
            }),
    );
    columns.extend([
        ColumnDef::new("subhanka", Scalar::F64, "The seven Subhankas together, out of 240."),
        ColumnDef::new("asubhanka", Scalar::F64, "Their complements together, out of 240."),
        ColumnDef::new(
            "nature",
            Scalar::U16,
            "Whether its rasi place is auspicious, neutral or inauspicious (v. 10).",
        )
        .of_enum("Nature"),
        ColumnDef::new(
            "phase",
            Scalar::U8,
            "Where in its dasha its effects come, by its decanate and reversed when retrograde and for the nodes (ch. 47 vv. 3 and 4).",
        )
        .of_enum("TsDashaPhase"),
        ColumnDef::new(
            "favourable",
            Scalar::U8,
            "1 when it is in the lagna, exaltation, its own sign or a Shant sign (ch. 47 v. 5).",
        ),
        ColumnDef::new(
            "unfavourable",
            Scalar::U8,
            "1 when it is in the sixth, eighth or twelfth, debilitation or an inimical sign (v. 6); both flags can stand.",
        ),
    ]);
    SectionSchema::columns(
        id,
        "dasha_phala",
        "Every chart's dasha phala, a row a graha, Sun to Ketu, charts outermost: row `i * 9 + g` is chart `i`'s `g`th graha. Read under the context's `dasha.shanta_sign`. Empty when the dasha phala was not asked for.",
        columns,
    )
}

/// Every chart's Vaiseshikamsa, a row a graha, a count and a name a scheme.
fn chart_vaiseshikamsa_section(id: u32) -> SectionSchema {
    let mut columns = vec![
        ColumnDef::new("graha", Scalar::U16, "Which graha.").of_enum("Graha"),
        ColumnDef::new(
            "impaired",
            Scalar::U8,
            "1 when it is combust, defeated in war or in Shayana, its names then not auspicious, else 0.",
        ),
    ];
    for (scheme, _) in crate::chart::VAISESHIKAMSA_SCHEMES {
        columns.push(ColumnDef::new(
            &format!("{scheme}_good"),
            Scalar::U8,
            &format!("How many of the {scheme}'s vargas are good for it."),
        ));
        columns.push(
            ColumnDef::new(
                &format!("{scheme}_name"),
                Scalar::U16,
                &format!(
                    "The name the {scheme} count earns; read only when that count is 2 or more."
                ),
            )
            .of_enum("Vaiseshikamsa"),
        );
    }
    SectionSchema::columns(
        id,
        "vaiseshikamsa",
        "Every chart's Vaiseshikamsa, a row a graha, Sun to Saturn, charts outermost: row `i * 7 + g` is chart `i`'s `g`th graha (BPHS ch. 6 vv. 42 to 53). Empty when the Vaiseshikamsa was not asked for.",
        columns,
    )
}

/// Whether a period section says row by row that a period is a sign's:
/// a birth dasha is all signs' or all lords', which `dashas.signed` says
/// once, where a year's Patyayini runs one sign among seven planets.
#[derive(Clone, Copy)]
pub(crate) enum SignedBy {
    /// Once, by `dashas.signed`.
    Dasha,
    /// For each period, by its `has_sign`.
    Period,
}

/// Every birth dasha's periods, depth first in time order.
fn chart_dasha_periods_section(id: u32) -> SectionSchema {
    dasha_periods_section(
        id,
        "dasha_periods",
        "Every dasha's periods of its birth cycle, concatenated in the `dashas` section's order and **ragged** by its `period_count`, each dasha's depth first in time order: a mahadasha, then its antardashas and theirs, then the next mahadasha. A period's path is its `index` below the nearest earlier period one `level` up.",
        SignedBy::Dasha,
    )
}

/// A dasha's periods, depth first in time order: one layout for the
/// births' `dasha_periods` and the years' `year_dasha_periods`, so every
/// binding decodes a period in one place.
fn dasha_periods_section(id: u32, name: &str, doc: &str, signed_by: SignedBy) -> SectionSchema {
    let mut columns = vec![
        ColumnDef::new("level", Scalar::U8, "How deep: 1 for a mahadasha."),
        ColumnDef::new(
            "index",
            Scalar::U8,
            "Its place in its parent's sequence, from 0; under the elapsed reading of the birth period the first may not be 0.",
        ),
    ];
    let sign = match signed_by {
        SignedBy::Dasha => {
            "The sign it is the period of, when its dasha is `signed`; zero otherwise."
        }
        SignedBy::Period => {
            columns.push(ColumnDef::new(
                "has_sign",
                Scalar::U8,
                "1 when it is a sign's period and `sign` names it: the Patyayini's lagna; 0 for a planet's.",
            ));
            "The sign it is the period of, when `has_sign`; zero otherwise."
        }
    };
    columns.extend([
        ColumnDef::new("sign", Scalar::U16, sign).of_enum("Rashi"),
        ColumnDef::new("lord", Scalar::U16, "Its lord.").of_enum("Graha"),
        ColumnDef::new(
            "from_jd",
            Scalar::F64,
            "When it begins, a Julian day (UTC).",
        ),
        ColumnDef::new("to_jd", Scalar::F64, "When it ends, a Julian day (UTC)."),
    ]);
    SectionSchema::columns(id, name, doc, columns)
}

/// Every chart's drishti: which body looks at which, how strongly, and
/// how near each end stands to a boundary.
///
/// **Ragged**, and `cast.aspect_count` is what says where each chart's
/// rows begin: a chart's relations are a function of where the bodies
/// stand rather than of how many there are, so two charts of the same
/// nine grahas hold 47 and 40 of them.
#[must_use]
fn chart_aspects_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "aspects",
        "Every chart's drishti, concatenated charts outermost and **ragged**: chart `i`'s rows begin at the sum of every earlier chart's `cast.aspect_count` and run for its own, ordered by the looking body and then by the body looked at, in the foundation's own order. Empty when the aspects were not asked for. `from_*` and `to_*` say how near each end stands to a boundary, which is what an ayanamsha that moved would change.",
        vec![
            ColumnDef::new("from", Scalar::U16, "The body looking.").of_enum("Graha"),
            ColumnDef::new("to", Scalar::U16, "The body looked at.").of_enum("Graha"),
            ColumnDef::new(
                "houses",
                Scalar::U8,
                "Which house of the first's sign the second stands in, counting inclusively from one.",
            ),
            ColumnDef::new("strength", Scalar::U8, "How strongly.").of_enum("TsStrength"),
            ColumnDef::new(
                "from_sign_deg",
                Scalar::F64,
                "How near the looking body stands to a sign edge, degrees.",
            ),
            ColumnDef::new(
                "from_nakshatra_deg",
                Scalar::F64,
                "How near it stands to a nakshatra edge, degrees.",
            ),
            ColumnDef::new(
                "from_pada_deg",
                Scalar::F64,
                "How near it stands to a pada edge, degrees.",
            ),
            ColumnDef::new(
                "to_sign_deg",
                Scalar::F64,
                "How near the body looked at stands to a sign edge, degrees.",
            ),
            ColumnDef::new(
                "to_nakshatra_deg",
                Scalar::F64,
                "How near it stands to a nakshatra edge, degrees.",
            ),
            ColumnDef::new(
                "to_pada_deg",
                Scalar::F64,
                "How near it stands to a pada edge, degrees.",
            ),
        ],
    )
}

/// The drishti table every relation was read under, which is one to a
/// batch because it is a setting.
#[must_use]
fn chart_drishti_table_section(id: u32) -> SectionSchema {
    SectionSchema::bytes(
        id,
        "drishti_table",
        "UTF-8 text: the drishti table the settings named, which every aspect above was read under. Empty when the aspects were not asked for.",
    )
}

/// Every chart's derived points: the upagrahas and the special lagnas.
///
/// **Ragged**, as the drishti are, and for a reason of its own: Saturn's
/// eighth needs an arc to divide, so a chart whose day has none carries
/// two points fewer.
#[must_use]
fn chart_points_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "points",
        "Every chart's derived points — the upagrahas and the special lagnas — concatenated charts outermost and **ragged**: chart `i`'s rows begin at the sum of every earlier chart's `cast.point_count` and run for its own. Empty when the points were not asked for. Gulika and Mandi are Saturn's eighth of the day's arc and are the two a chart with no arc to divide cannot have.",
        vec![
            ColumnDef::new("point", Scalar::U16, "Which point.").of_enum("Point"),
            ColumnDef::new(
                "longitude_deg",
                Scalar::F64,
                "Its longitude in the chart's zodiac, degrees.",
            ),
            ColumnDef::new("sign", Scalar::U16, "The sign it falls in.").of_enum("Rashi"),
            ColumnDef::new(
                "sign_deg",
                Scalar::F64,
                "How near it stands to a sign edge, degrees.",
            ),
            ColumnDef::new(
                "nakshatra_deg",
                Scalar::F64,
                "How near it stands to a nakshatra edge, degrees.",
            ),
            ColumnDef::new(
                "pada_deg",
                Scalar::F64,
                "How near it stands to a pada edge, degrees.",
            ),
        ],
    )
}

/// One row per divisional chart per chart: which chart, and where its
/// lagna falls.
#[must_use]
fn chart_vargas_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "vargas",
        "One row per divisional chart per chart, charts outermost: row `i * varga_count + v` is chart `i`, the `v`th chart asked for. Empty when none were asked for, which is unambiguous because a divisional chart that *was* asked for always has a lagna (`03-design/chart-reading.md` §5).",
        vec![
            ColumnDef::new("varga", Scalar::U16, "Which divisional chart.").of_enum("Varga"),
            ColumnDef::new(
                "lagna_rashi",
                Scalar::U16,
                "The sign the lagna stands in, in the rashi chart.",
            )
            .of_enum("Rashi"),
            ColumnDef::new(
                "lagna_part",
                Scalar::U16,
                "Which part of that sign the lagna falls in, counted from zero.",
            ),
            ColumnDef::new(
                "lagna_sign",
                Scalar::U16,
                "The sign the divisional chart puts the lagna in.",
            )
            .of_enum("Rashi"),
        ],
    )
}

/// Every graha of every divisional chart.
#[must_use]
fn chart_varga_grahas_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "varga_grahas",
        "One row per graha per divisional chart per chart, charts outermost then charts asked for: row `(i * varga_count + v) * graha_count + j` is chart `i`, the `v`th divisional chart, graha `j` in the `grahas` section's own order. Empty when no divisional chart was asked for.",
        vec![
            ColumnDef::new(
                "rashi",
                Scalar::U16,
                "The sign the graha stands in, in the rashi chart.",
            )
            .of_enum("Rashi"),
            ColumnDef::new(
                "part",
                Scalar::U16,
                "Which part of that sign it falls in, counted from zero.",
            ),
            ColumnDef::new(
                "sign",
                Scalar::U16,
                "The sign the divisional chart puts it in.",
            )
            .of_enum("Rashi"),
        ],
    )
}

/// The twelve bhavas of each chart, as the houses service reads them.
///
/// **What is here is what is not elsewhere.** The madhya and the sandhi
/// are already in `houses` and `chalit`; which bhava each body falls in
/// is already in `grahas`; the systems are already in `readings`. What
/// only this service computes is the sign a bhava's *middle* falls in —
/// which under an unequal division is not the sign it begins in — its
/// lord, and which third of the wheel it stands in
/// (`03-design/chart-at-the-boundary.md` §3: describe each shape once).
///
/// Fixed at twelve per chart, so no count is needed: a chart that has
/// bhavas has twelve of them, which is what makes an empty section
/// unambiguously "not asked for".
#[must_use]
fn chart_bhavas_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "bhavas",
        "Twelve rows per chart, charts outermost: row `i * 12 + j` is chart `i`, bhava `j + 1`. Empty when the houses were not asked for, which is unambiguous because a chart that has bhavas has twelve. The madhya and the sandhi are in `houses` and `chalit`; this is what only the houses service computes.",
        vec![
            ColumnDef::new(
                "sign",
                Scalar::U16,
                "The sign the bhava's **middle** falls in, which is the sign the tradition means by \"the house's sign\": under an unequal division a house can begin in one sign and be centred in another.",
            )
            .of_enum("Rashi"),
            ColumnDef::new("lord", Scalar::U16, "The lord of that sign.").of_enum("Graha"),
            ColumnDef::new(
                "quadrant",
                Scalar::U8,
                "Which third of the wheel it stands in.",
            )
            .of_enum("TsQuadrant"),
        ],
    )
}

/// What each graha **is**, as opposed to where it is.
///
/// One row per graha per chart, the same stride as `grahas`, because
/// every chart of a batch carries the same bodies: a state is a reading
/// of a placement, so there is exactly one per placement and no count is
/// needed.
///
/// **The three lajjitadi lists are bit sets**, one bit per member of a
/// six-member enum, rather than three ragged sections with three counts.
/// A set over a small closed enum is a set; making it a list would put
/// three prefix sums in every decoder for a value that fits in a byte
/// (`03-design/chart-reading.md` §5).
///
/// Every `*_present` column beside a value is the panchanga blob's own
/// rule: a value a row may not have crosses as a flag beside it, because
/// an absent distance and a distance of zero are different facts and no
/// sentinel tells them apart.
#[must_use]
#[expect(
    clippy::too_many_lines,
    reason = "one declaration per column of the widest section in the blob; splitting it would hide the shape it exists to show"
)]
fn chart_states_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "states",
        "One row per graha per chart, charts outermost: row `i * graha_count + j` is chart `i`, graha `j`, in the `grahas` section's own order. Empty when the states were not asked for, which is unambiguous because a chart that has states has one per graha.\n\nThe **motion** is not here: `grahas.speed_deg_per_day` already carries it and retrograde is its sign, and describing a shape twice is what `03-design/chart-at-the-boundary.md` §3 exists to prevent.",
        vec![
            ColumnDef::new("graha", Scalar::U16, "Which graha.").of_enum("Graha"),
            ColumnDef::new("sign", Scalar::U16, "The sign it stands in.").of_enum("Rashi"),
            ColumnDef::new(
                "house",
                Scalar::U8,
                "The bhava it falls in, under the chart's placement system.",
            ),
            ColumnDef::new("dignity", Scalar::U16, "Its dignity.").of_enum("Dignity"),
            ColumnDef::new(
                "natural",
                Scalar::U16,
                "How it stands to its dispositor by the table's own reading.",
            )
            .of_enum("Relationship"),
            ColumnDef::new(
                "temporary",
                Scalar::U16,
                "How it stands to its dispositor by where that body stands.",
            )
            .of_enum("Relationship"),
            ColumnDef::new(
                "compound",
                Scalar::U16,
                "The five-fold compound of the two.",
            )
            .of_enum("Relationship"),
            ColumnDef::new(
                "has_dispositor",
                Scalar::U8,
                "1 when the sign has a lord; 0 only for a body the catalogue gives no sign.",
            ),
            ColumnDef::new(
                "dispositor",
                Scalar::U16,
                "The lord of the sign, which all three relationships are with.",
            )
            .of_enum("Graha"),
            ColumnDef::new("burning", Scalar::U8, "What the Sun does to it.").of_enum("TsBurning"),
            ColumnDef::new(
                "has_from_sun",
                Scalar::U8,
                "1 when the chart carries a Sun to measure from; 0 when it does not, in which case nothing is burnt and this says why rather than claiming the sky is clear.",
            ),
            ColumnDef::new(
                "from_sun_deg",
                Scalar::F64,
                "How far from the Sun it stands, degrees; read only when `has_from_sun`.",
            ),
            ColumnDef::new(
                "has_orbs",
                Scalar::U8,
                "1 when the table gives this body an orb; 0 for a body that does not burn at all.",
            ),
            ColumnDef::new(
                "orb_deg",
                Scalar::F64,
                "Combust inside this, degrees; read only when `has_orbs`.",
            ),
            ColumnDef::new(
                "has_deep_orb",
                Scalar::U8,
                "1 when the table gives a deeper orb as well.",
            ),
            ColumnDef::new(
                "deep_orb_deg",
                Scalar::F64,
                "Deeply combust inside this, degrees; read only when `has_deep_orb`.",
            ),
            ColumnDef::new("age", Scalar::U16, "Which fifth of its sign it stands in.")
                .of_enum("AvasthaBaladi"),
            ColumnDef::new("wakefulness", Scalar::U16, "Awake, dreaming or asleep.")
                .of_enum("AvasthaJagradadi"),
            ColumnDef::new(
                "has_deeptadi",
                Scalar::U8,
                "1 when the SDK can decide a bright state.",
            ),
            ColumnDef::new(
                "deeptadi",
                Scalar::U16,
                "The bright state; read only when `has_deeptadi`.",
            )
            .of_enum("AvasthaDeeptadi"),
            ColumnDef::new(
                "lajjitadi_holding",
                Scalar::U32,
                "The lajjitadi that hold, as a bit set: bit `n` is the member with catalogue id `n`.",
            ),
            ColumnDef::new(
                "lajjitadi_ruled_out",
                Scalar::U32,
                "The lajjitadi that certainly do not hold, because the tradition's own necessary condition fails, as a bit set.",
            ),
            ColumnDef::new(
                "lajjitadi_undecided",
                Scalar::U32,
                "The lajjitadi nothing decides: the necessary condition holds and what narrows it further is not in the chart. A bit set, so a caller can tell a short list from an empty one.",
            ),
            ColumnDef::new(
                "has_war",
                Scalar::U8,
                "1 when the body is in a planetary war.",
            ),
            ColumnDef::new(
                "war_opponent",
                Scalar::U16,
                "The other body; read only when `has_war`.",
            )
            .of_enum("Graha"),
            ColumnDef::new(
                "war_won",
                Scalar::U8,
                "1 when this body won it; read only when `has_war`.",
            ),
            ColumnDef::new(
                "war_apart_deg",
                Scalar::F64,
                "How far apart they stand, degrees; read only when `has_war`.",
            ),
            ColumnDef::new(
                "sign_deg",
                Scalar::F64,
                "How near it stands to a sign edge, degrees.",
            ),
            ColumnDef::new(
                "nakshatra_deg",
                Scalar::F64,
                "How near it stands to a nakshatra edge, degrees.",
            ),
            ColumnDef::new(
                "pada_deg",
                Scalar::F64,
                "How near it stands to a pada edge, degrees.",
            ),
            ColumnDef::new(
                "has_sayanadi",
                Scalar::U8,
                "1 for the nine grahas, which BPHS ch. 45 numbers; 0 for the outer planets, and for every body of a chart with no Moon.",
            ),
            ColumnDef::new(
                "sayanadi",
                Scalar::U16,
                "The Sayanadi state; read only when `has_sayanadi`.",
            )
            .of_enum("AvasthaSayanadi"),
        ]
        .into_iter()
        .chain(teistro_state::Anka::ALL.map(|anka| {
            ColumnDef::new(
                &format!("cheshta_{}", anka.get()),
                Scalar::U16,
                &format!(
                    "The Sayanadi sub-state under a name whose first syllable's anka is {}; read only when `has_sayanadi`.",
                    anka.get()
                ),
            )
            .of_enum("AvasthaCheshta")
        }))
        .collect(),
    )
}

/// Two names matched star to star: each name's first syllable, their
/// vargas, and the Ashta Koota and the ten considerations of the two name
/// stars, in the sections a chart's match crosses as (`matching.md`,
/// C291 to C296).
#[must_use]
pub fn naam() -> BlobSchema {
    let one = "One row: the two names' match.";
    BlobSchema {
        name: NAAM.to_string(),
        id: 5,
        doc: "Two names matched star to star (naam milan): each name's first syllable in the śatapada cakra, the varga koota, and the Ashta Koota and the ten considerations read from the two name stars, in the same sections and shapes a chart's match crosses as.".to_string(),
        sections: vec![
            SectionSchema::columns(
                1,
                "naam_names",
                "Each name's first syllable, two rows: the bride's, then the groom's.",
                vec![
                    ColumnDef::new("cell", Scalar::U8, "Its place among the cakra's 112 cells, 0 for a, Krittika's first."),
                    ColumnDef::new("nakshatra", Scalar::U16, "Its star; read only when `abhijit` is 0.").of_enum("Nakshatra"),
                    ColumnDef::new("abhijit", Scalar::U8, "1 when the syllable is Abhijit's, which is none of the 27 (C292)."),
                    ColumnDef::new("quarter", Scalar::U8, "Which of the star's four syllables, 1 to 4: the pada, for one of the 27."),
                    ColumnDef::new("varga", Scalar::U8, "The varga of the name's first letter as written (C295).").of_enum("TsNameVarga"),
                ],
            ),
            SectionSchema::fixed(
                2,
                "naam_varga",
                "The varga koota of *Muhurta Chintamani* VI.35: how the two names' vargas stand. Never points.",
                vec![
                    ColumnDef::new("relation", Scalar::U8, "One varga, enemies (each the 5th from the other) or neither.").of_enum("TsVargaRelation"),
                ],
            ),
            SectionSchema::columns(3, "matchings", &format!("The Ashta Koota of the two name stars, as a chart's match reads two Moons. {one}"), matching_columns()).of_shape(MATCHINGS_SHAPE),
            SectionSchema::columns(4, "matching_kootas", "The eight kootas' points, in the verse's order.", matching_koota_columns()).of_shape(MATCHING_KOOTAS_SHAPE),
            SectionSchema::columns(5, "poruthams", &format!("The ten considerations of the two name stars. {one}"), porutham_columns()).of_shape(PORUTHAMS_SHAPE),
            SectionSchema::columns(6, "porutham_rows", "The ten considerations, in the chapter's order.", porutham_row_columns()).of_shape(PORUTHAM_ROWS_SHAPE),
        ],
    }
}
