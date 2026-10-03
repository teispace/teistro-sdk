//! The result blob schemas: what each blob-returning entry point writes,
//! declared once here, read by the encoder in this crate and by every
//! generated decoder through `idl/api.json`. A schema is appended to,
//! never reordered; a new section gets a new id.

use teistro_idl::model::{BlobSchema, ColumnDef, Scalar, SectionSchema};

/// The schema `ts_positions` fills.
pub const POSITIONS: &str = "positions";
/// The schema `ts_intl_render` fills.
pub const INTL_RENDER: &str = "intl_render";
/// The schema `ts_chart_found` fills.
pub const CHARTS: &str = "charts";

/// The panchanga blob's name.
pub const PANCHANGA: &str = "panchanga";

/// Every schema, in id order.
#[must_use]
pub fn schemas() -> Vec<BlobSchema> {
    vec![positions(), intl_render(), charts(), panchanga()]
}

/// The name every section holding a day arc declares, so that two blobs
/// carrying the same day decode into **one** type in each binding rather
/// than two identical ones (`03-design/chart-at-the-boundary.md` §8).
pub const DAY_SHAPE: &str = "day";

/// The day an instant belongs to, which is not always its civil date.
///
/// Declared once and used by every blob that carries a day. A chart's
/// day and a panchanga's day are the same eighteen fields with the same
/// values on the same chart, so this is the section §3 exists to stop
/// being written twice.
///
/// The place is **not** here. A chart has one place and its summary
/// carries it; repeating it inside the day would be the same repetition
/// in a smaller box.
///
/// Neither are the two fields that belong to an **instant** rather than
/// to the day. `part` — which arc the instant falls in — and `elapsed`
/// — how far through that arc it is — were here while a chart was the
/// only blob carrying a day, and a panchanga cannot fill them: its day
/// has no single instant. They live in the chart's `cast` section, with
/// the other things an instant decides. The design page had said all
/// along that the shared section is **eighteen** fields; it was twenty
/// until the panchanga asked for it.
///
/// Two fields are a tagged enum split into a kind and a payload, which
/// is how a variant carrying data crosses a boundary that has only
/// scalars: `state` has none today, and `convention` puts a `TsSunrise`
/// id in its value for a named convention and an altitude in degrees for
/// a custom one.
#[must_use]
pub fn day_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "day",
        "The day each instant belongs to: its arc, its date and how it was reckoned. One row per row of the blob's own grid.",
        vec![
            ColumnDef::new(
                "sunrise",
                Scalar::F64,
                "The sunrise that opened the day, as a Julian day (UTC).",
            ),
            ColumnDef::new(
                "sunset",
                Scalar::F64,
                "The sunset that closed its daylight, as a Julian day (UTC).",
            ),
            ColumnDef::new(
                "next_sunrise",
                Scalar::F64,
                "The sunrise that closes it, as a Julian day (UTC).",
            ),
            ColumnDef::new("vara", Scalar::U16, "The weekday the day carries.").of_enum("Vara"),
            ColumnDef::new("calendar", Scalar::U16, "The calendar the date is in.")
                .of_enum("Calendar"),
            ColumnDef::new("era", Scalar::U16, "The era the date's year is counted in.")
                .of_enum("Era"),
            ColumnDef::new("year", Scalar::I32, "The astronomical year; 1 BCE is 0."),
            ColumnDef::new("era_year", Scalar::I32, "The year as the era counts it."),
            ColumnDef::new("month", Scalar::U8, "The month, 1 to 12 or 13."),
            ColumnDef::new("day_of_month", Scalar::U8, "The day of the month."),
            ColumnDef::new("resolution", Scalar::U8, "How the date was resolved.")
                .of_enum("TsResolution"),
            ColumnDef::new(
                "computed_month",
                Scalar::U8,
                "The engine's month where it differs from the table's; zero otherwise.",
            ),
            ColumnDef::new(
                "computed_day",
                Scalar::U8,
                "The engine's day where it differs from the table's; zero otherwise.",
            ),
            ColumnDef::new(
                "state_kind",
                Scalar::U8,
                "Whether the day had a sunrise at all.",
            )
            .of_enum("TsDayState"),
            ColumnDef::new(
                "state_polar_kind",
                Scalar::U8,
                "Which polar state it was, when it had none; zero otherwise.",
            )
            .of_enum("TsPolarKind"),
            ColumnDef::new(
                "state_polar_policy",
                Scalar::U8,
                "Which policy synthesised its bounds, when it had none; zero otherwise.",
            )
            .of_enum("TsPolarDayPolicy"),
            ColumnDef::new(
                "convention_kind",
                Scalar::U8,
                "Which sunrise convention the arc was reckoned by, or the one an atmospheric convention gave its air to; `0xFF` for a custom altitude.",
            )
            .of_enum("TsSunrise"),
            ColumnDef::new(
                "convention_value",
                Scalar::F64,
                "The altitude in degrees when the convention is custom; zero otherwise.",
            ),
            ColumnDef::new(
                "air_pressure_hpa",
                Scalar::F64,
                "The air's pressure the arc was refracted through, hectopascals, resolved at the place, when the convention is atmospheric; zero otherwise.",
            ),
            ColumnDef::new(
                "air_temperature_c",
                Scalar::F64,
                "The air's temperature the arc was refracted through, degrees Celsius, when the convention is atmospheric; zero otherwise.",
            ),
        ],
    )
}

/// The schema with a name.
#[must_use]
pub fn schema(name: &str) -> Option<BlobSchema> {
    schemas().into_iter().find(|s| s.name == name)
}

/// Positions over a grid of instants and bodies, completed into the
/// requested frame, with the steps that produced them and the provenance.
#[must_use]
pub fn positions() -> BlobSchema {
    BlobSchema {
        name: POSITIONS.to_string(),
        id: 1,
        doc: "Positions over a grid of instants and bodies, instants outermost: cell `i * body_count + j` is instant `i`, body `j`.".to_string(),
        sections: vec![
            positions_summary_section(1),
            SectionSchema::columns(
                2,
                "instants",
                "The instants of the request, in order.",
                vec![ColumnDef::new("jd", Scalar::F64, "A Julian day on the request's scale.")],
            ),
            SectionSchema::columns(
                3,
                "bodies",
                "The bodies of the request, in order.",
                vec![ColumnDef::new("body", Scalar::U16, "A body id.").of_enum("Body")],
            ),
            SectionSchema::columns(
                4,
                "cells",
                "One row per cell, instants outermost; a cell whose status is not zero carries no value.",
                vec![
                    ColumnDef::new("lon", Scalar::F64, "Longitude in degrees, 0 to 360."),
                    ColumnDef::new("lat", Scalar::F64, "Latitude in degrees."),
                    ColumnDef::new("dist", Scalar::F64, "Distance in the provider's unit."),
                    ColumnDef::new("lon_speed", Scalar::F64, "Longitude speed in degrees per day; zero when speeds were not asked for."),
                    ColumnDef::new("lat_speed", Scalar::F64, "Latitude speed in degrees per day."),
                    ColumnDef::new("dist_speed", Scalar::F64, "Distance speed per day."),
                    ColumnDef::new("status", Scalar::I32, "The cell's status code; zero is a value."),
                    ColumnDef::new("source", Scalar::U32, "What computed the cell, packed as the port packs it."),
                ],
            ),
            SectionSchema::bytes(
                5,
                "steps",
                "UTF-8 JSON: the completion steps applied, in order, each `{\"name\", \"implementation\"}`.",
            ),
            SectionSchema::bytes(
                6,
                "provenance_json",
                "UTF-8 JSON: the provenance envelope of the result, canonical.",
            ),
        ],
    }
}

/// A rendered message: the text, its parts, where it resolved from, and the
/// warnings.
#[must_use]
pub fn intl_render() -> BlobSchema {
    BlobSchema {
        name: INTL_RENDER.to_string(),
        id: 2,
        doc: "A rendered message.".to_string(),
        sections: vec![
            SectionSchema::fixed(
                1,
                "flags",
                "How the message resolved.",
                vec![
                    ColumnDef::new(
                        "is_fallback",
                        Scalar::U8,
                        "Non-zero when a fallback locale answered.",
                    ),
                    ColumnDef::new(
                        "is_override",
                        Scalar::U8,
                        "Non-zero when a runtime override answered.",
                    ),
                    ColumnDef::new("warning_count", Scalar::U32, "The number of warnings."),
                ],
            ),
            SectionSchema::bytes(2, "text", "UTF-8: the plain text, markup stripped."),
            SectionSchema::bytes(
                3,
                "resolved_from",
                "UTF-8: the locale whose message answered; empty when none had it.",
            ),
            SectionSchema::bytes(
                4,
                "warnings",
                "UTF-8 JSON: an array of strings, one per problem met.",
            ),
            SectionSchema::bytes(
                5,
                "parts",
                "UTF-8 JSON: the message's parts, for a rich renderer \u{2014} `{\"type\": \"text\", \"value\": ...}` or `{\"type\": \"markup\", \"kind\": \"open\"|\"close\"|\"standalone\", \"name\": ..., \"options\": {...}}`, adjacent text in one part. **Empty when the message has no markup**, the whole of it being `text`; a reader turns that into the one text part rather than asking for it twice.",
            ),
        ],
    }
}

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

/// The grid and the frame the positions are in.
#[must_use]
fn positions_summary_section(id: u32) -> SectionSchema {
    SectionSchema::fixed(
        id,
        "summary",
        "The grid and the frame the values are in.",
        vec![
            ColumnDef::new(
                "frame_bits",
                Scalar::U32,
                "The frame the values are in, packed as the port packs it.",
            ),
            ColumnDef::new("jd_count", Scalar::U32, "The number of instants."),
            ColumnDef::new("body_count", Scalar::U32, "The number of bodies."),
            ColumnDef::new("scale", Scalar::U32, "The time scale of the instants.")
                .of_enum("TimeScale"),
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
            SectionSchema::fixed(
                7,
                "zodiac",
                "The zodiac the batch is measured in. The offset itself moves with the instant, so it is a column of `charts` rather than a field here.",
                vec![
                    ColumnDef::new("frame_bits", Scalar::U32, "The frame the positions were asked for, packed as the port packs it."),
                    ColumnDef::new("ayanamsha_kind", Scalar::U8, "0 for none, 1 for a catalogued ayanamsha, 2 for one the settings define."),
                    ColumnDef::new("ayanamsha", Scalar::U16, "Which catalogued ayanamsha, when the kind is 1.").of_enum("Ayanamsha"),
                ],
            ),
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
        .collect(),
    }
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

/// Each item's own content hash, where a batch's provenance hashes the
/// list: what an item handed out alone is stamped with.
fn content_hashes_section(id: u32, item: &str, hashed: &str) -> SectionSchema {
    SectionSchema::bytes(
        id,
        "content_hashes",
        &format!(
            "UTF-8 text: each {item}'s own content hash — {hashed}, canonical — as sixty-four lowercase hex digits, a {item} after the other in the batch's order with nothing between them, so {item} `i` is bytes `64 * i` to `64 * i + 64`. The provenance's `content_hash` is the list's; a {item} handed out alone carries its own (`03-design/serial-and-the-envelope.md` §3)."
        ),
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

/// A list of spans of one catalogue's members, clipped to a day.
///
/// Seven of the panchanga's lists are this shape — the four moving limbs,
/// panchaka, and the signs the Moon and the Sun stood in — so the five
/// columns are declared once here.
///
/// They deliberately do **not** carry a `shape` name. A shape makes two
/// sections decode to one type, which is right when they are the same
/// section in two blobs (a chart's day and a panchanga's) and wrong here:
/// a span of tithis and a span of nakshatras have the same structure and
/// different meanings, and one type for both would let a caller pass
/// either where the other is wanted. The repetition worth removing is in
/// the declaration, which this removes; the distinction worth keeping is
/// in the type, which this keeps.
#[must_use]
fn span_section(id: u32, name: &str, doc: &str, member: &str, enum_name: &str) -> SectionSchema {
    SectionSchema::columns(
        id,
        name,
        doc,
        vec![
            ColumnDef::new("member", Scalar::U16, member).of_enum(enum_name),
            ColumnDef::new(
                "whole_from",
                Scalar::F64,
                "When the member itself began, as a Julian day (UTC), whether or not that is inside the day.",
            ),
            ColumnDef::new(
                "whole_to",
                Scalar::F64,
                "When the member itself ended, as a Julian day (UTC), whether or not that is inside the day.",
            ),
            ColumnDef::new(
                "inside_from",
                Scalar::F64,
                "Where the part inside the day begins: what an almanac row prints.",
            ),
            ColumnDef::new(
                "inside_to",
                Scalar::F64,
                "Where the part inside the day ends.",
            ),
            ColumnDef::new(
                "sunrises",
                Scalar::U8,
                "Which of the day's two sunrises the member was running at: its own, the next day's, both (vriddhi: it names two days) or neither (kshaya: it names none).",
            )
            .of_enum("TsSunrises"),
            ColumnDef::new(
                "ends_ghati",
                Scalar::U8,
                "When the member ended, in ghatis from the day's sunrise under `day.ghati_reckoning`; a member outlasting the day reads as the day's whole count.",
            ),
            ColumnDef::new("ends_pala", Scalar::U8, "And palas, 0 to 59."),
            ColumnDef::new("ends_vipala", Scalar::U8, "And vipalas, 0 to 59."),
        ],
    )
}

/// An instant a day's rows are bounded by, as a column pair.
#[must_use]
fn interval_columns(from: &str, to: &str, what: &str) -> Vec<ColumnDef> {
    vec![
        ColumnDef::new(
            from,
            Scalar::F64,
            &format!("When {what} begins, as a Julian day (UTC)."),
        ),
        ColumnDef::new(
            to,
            Scalar::F64,
            &format!("When {what} ends, as a Julian day (UTC)."),
        ),
    ]
}

/// How many rows of each ragged section belong to each day.
///
/// A day's lists are ragged and the measurement says by how much: ten of
/// the fifteen vary, and a rectangular layout wastes 78.1% of its rows
/// once one polar day joins a batch, because that day sets the stride for
/// every other (`03-design/panchanga-at-the-boundary-measured.md` §2). So
/// every list is concatenated across the batch and this says where each
/// day's share of it is — the same rule for all thirteen sections, since
/// two layouts in one blob is two things for a reader to learn and the
/// five fixed lists lose nothing by it.
#[must_use]
fn panchanga_counts_section(id: u32) -> SectionSchema {
    let count = |name: &str, of: &str| {
        ColumnDef::new(
            name,
            Scalar::U32,
            &format!("How many rows of `{of}` belong to this day."),
        )
    };
    SectionSchema::columns(
        id,
        "counts",
        "How many rows of each per-day section belong to each day, in the order the days run. A day's rows begin where the sum of every earlier day's count leaves off.",
        vec![
            count("tithi", "tithi"),
            count("nakshatra", "nakshatra"),
            count("yoga", "yoga"),
            count("karana", "karana"),
            count("panchaka", "panchaka"),
            count("moon_signs", "moon_signs"),
            count("sun_signs", "sun_signs"),
            count("kaalas", "kaalas"),
            count("choghadiya", "choghadiya"),
            count("horas", "horas"),
            count("muhurtas", "muhurtas"),
            count("moon_events", "moon_events"),
            count("muhurta_yogas", "muhurta_yogas"),
        ],
    )
}

/// What each day is, beside the day it belongs to.
#[must_use]
fn panchanga_days_section(id: u32) -> SectionSchema {
    let mut fields = interval_columns(
        "window_from",
        "window_to",
        "the window the day's spans are clipped to",
    );
    fields.extend([
        ColumnDef::new("month", Scalar::U16, "The lunar month under the profile's own convention.").of_enum("Masa"),
        ColumnDef::new("amanta", Scalar::U16, "The amanta month: new moon to new moon.").of_enum("Masa"),
        ColumnDef::new("purnimanta", Scalar::U16, "The purnimanta month: full moon to full moon.").of_enum("Masa"),
        ColumnDef::new("paksha", Scalar::U16, "The fortnight the day opens in.").of_enum("Paksha"),
        ColumnDef::new("month_kind", Scalar::U8, "Whether the month is ordinary, intercalary or omitted. The month's *name* needs no case for the intercalary one — the Sun stands in the same sign at an adhika month's new moon as at the following nija month's, so `month` names both — and this is the mark beside it.").of_enum("TsMonthKind"),
        ColumnDef::new("ayana", Scalar::U16, "Which half of the year the day falls in.").of_enum("Ayana"),
        ColumnDef::new("ritu", Scalar::U16, "Which season the day falls in, under `panchanga.ritu`: by default the season of the sidereal solar month the day belongs to, two signs each from Capricorn (Surya Siddhanta XIV.10), its first day placed by `panchanga.solar_month_start` (`03-design/ritu-measured.md`).").of_enum("Ritu"),
        ColumnDef::new("disha_shool", Scalar::U16, "The direction not to travel in, which is the vara's.").of_enum("Direction"),
        ColumnDef::new("has_sankranti", Scalar::U8, "1 when the Sun entered a new sign inside the day, 0 otherwise."),
        ColumnDef::new("sankranti", Scalar::F64, "When it did, as a Julian day (UTC); zero when it did not, which `has_sankranti` is what distinguishes from midnight."),
        ColumnDef::new("has_abhijit", Scalar::U8, "1 when the day has an Abhijit muhurta, 0 on a day with no daylight."),
    ]);
    fields.extend(interval_columns("abhijit_from", "abhijit_to", "Abhijit"));
    fields.extend([
        ColumnDef::new(
            "abhijit_effective",
            Scalar::U8,
            "1 when Abhijit is effective, which it is on every day but a Wednesday.",
        ),
        ColumnDef::new(
            "has_brahma",
            Scalar::U8,
            "1 when the night that ends at this day's sunrise is known, 0 in the polar case.",
        ),
    ]);
    fields.extend(interval_columns(
        "brahma_from",
        "brahma_to",
        "Brahma muhurta",
    ));
    fields.extend(interval_columns(
        "moon_window_from",
        "moon_window_to",
        "the window the Moon's rises and sets were looked for in",
    ));
    SectionSchema::columns(
        id,
        "days",
        "One row per day: what the day is, beside the `day` section's account of the day it belongs to. Three values a day may not have — the sankranti, Abhijit and Brahma muhurta — carry a presence flag beside them rather than a sentinel, because an absent instant and midnight are both nought.",
        fields,
    )
}

/// The `kaalas` section: one of the panchanga's ragged lists.
#[must_use]
fn panchanga_kaalas_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "kaalas",
        "The inauspicious eighths of the daylight each day has.",
        {
            let mut fields =
                vec![ColumnDef::new("kaala", Scalar::U16, "Which one.").of_enum("Kaala")];
            fields.extend(interval_columns("from", "to", "it"));
            fields
        },
    )
}

/// The `choghadiya` section: one of the panchanga's ragged lists.
#[must_use]
fn panchanga_choghadiya_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "choghadiya",
        "Eight choghadiya of the daylight and eight of the night, when the day has both.",
        {
            let mut fields = vec![
                ColumnDef::new("choghadiya", Scalar::U16, "Which choghadiya.")
                    .of_enum("Choghadiya"),
                ColumnDef::new("lord", Scalar::U16, "The graha that rules it.").of_enum("Graha"),
            ];
            fields.extend(interval_columns("from", "to", "it"));
            fields.push(ColumnDef::new(
                "daytime",
                Scalar::U8,
                "1 when it is one of the eight of the daylight, 0 for one of the night.",
            ));
            fields
        },
    )
}

/// The `horas` section: one of the panchanga's ragged lists.
#[must_use]
fn panchanga_horas_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "horas",
        "The twenty-four horas of each day, from sunrise.",
        vec![
            ColumnDef::new(
                "number",
                Scalar::U8,
                "The hora's number, 1 to 24 from sunrise.",
            ),
            ColumnDef::new("lord", Scalar::U16, "The graha that rules it.").of_enum("Graha"),
            ColumnDef::new(
                "start",
                Scalar::F64,
                "When it begins, as a Julian day (UTC).",
            ),
            ColumnDef::new("end", Scalar::F64, "When it ends, as a Julian day (UTC)."),
        ],
    )
}

/// The `muhurtas` section: one of the panchanga's ragged lists.
#[must_use]
fn panchanga_muhurtas_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "muhurtas",
        "The thirty muhurtas of each day: fifteen of the daylight and fifteen of the night that follows it, in order. Abhijit and Brahma muhurta are named in `days` rather than repeated here.",
        {
            let mut fields = interval_columns("from", "to", "it");
            fields.push(ColumnDef::new(
                "daylight",
                Scalar::U8,
                "1 when it is one of the fifteen of the daylight, 0 for one of the night.",
            ));
            fields
        },
    )
}

/// The `moon_events` section: one of the panchanga's ragged lists.
#[must_use]
fn panchanga_moon_events_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "moon_events",
        "Every moonrise and moonset inside each day's moon window, in order.",
        vec![
            ColumnDef::new("kind", Scalar::U8, "Whether the Moon rose or set.")
                .of_enum("TsMoonEvent"),
            ColumnDef::new("instant", Scalar::F64, "When, as a Julian day (UTC)."),
        ],
    )
}

/// The `muhurta_yogas` section: one of the panchanga's ragged lists.
#[must_use]
fn panchanga_muhurta_yogas_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "muhurta_yogas",
        "The muhurta yogas that held, with what made each hold. `because_*` is a tagged enum split into a kind and the payload fields of its widest variant, so a `VARA_NAKSHATRA` cause leaves `because_tithi` at zero.",
        {
            let mut fields =
                vec![ColumnDef::new("yoga", Scalar::U16, "Which yoga.").of_enum("MuhurtaYoga")];
            fields.extend(interval_columns("from", "to", "it"));
            fields.extend([
                ColumnDef::new("because_kind", Scalar::U8, "What made it hold.")
                    .of_enum("TsYogaCause"),
                ColumnDef::new(
                    "because_vara",
                    Scalar::U16,
                    "The vara that makes it; every cause has one.",
                )
                .of_enum("Vara"),
                ColumnDef::new(
                    "because_tithi",
                    Scalar::U16,
                    "The tithi that makes it, when the cause has one; zero otherwise.",
                )
                .of_enum("Tithi"),
                ColumnDef::new(
                    "because_nakshatra",
                    Scalar::U16,
                    "The nakshatra that makes it; every cause has one.",
                )
                .of_enum("Nakshatra"),
            ]);
            fields
        },
    )
}

/// A batch of almanacs: one day at one place, many times over.
#[must_use]
pub fn panchanga() -> BlobSchema {
    BlobSchema {
        name: PANCHANGA.to_string(),
        id: 4,
        doc: "A batch of daily panchangas at one place: the day, the four moving limbs, the periods, the lunar month, what the Moon and the Sun did, and what the day is said to be. Every per-day list is concatenated across the batch, with `counts` saying how many rows are each day's.".to_string(),
        sections: vec![
            SectionSchema::fixed(
                1,
                "summary",
                "What the batch decided once: where, in which calendar, and how many days.",
                vec![
                    ColumnDef::new("day_count", Scalar::U32, "How many days the batch holds, and how many rows the `days`, `counts` and `day` sections each hold."),
                    ColumnDef::new("latitude_deg", Scalar::F64, "The place's latitude, degrees north."),
                    ColumnDef::new("longitude_deg", Scalar::F64, "The place's longitude, degrees east."),
                    ColumnDef::new("altitude_m", Scalar::F64, "The place's altitude, metres."),
                    ColumnDef::new("calendar", Scalar::U16, "The civil calendar the days' dates are read in.").of_enum("Calendar"),
                    ColumnDef::new("lunar_month", Scalar::U8, "Which lunar-month convention `days.month` leads with.").of_enum("TsLunarMonth"),
                ],
            ),
            panchanga_days_section(2),
            panchanga_counts_section(3),
            day_section(4).of_shape(DAY_SHAPE),
            span_section(5, "tithi", "The tithis that touch each day.", "Which tithi ran.", "Tithi"),
            span_section(6, "nakshatra", "The nakshatras the Moon was in.", "Which nakshatra the Moon was in.", "Nakshatra"),
            span_section(7, "yoga", "The nitya yogas.", "Which nitya yoga ran.", "Yoga"),
            span_section(8, "karana", "The karanas; half-tithis, so there are three or four on an ordinary day.", "Which karana ran.", "Karana"),
            span_section(9, "panchaka", "Panchaka, while the Moon is in the last five nakshatras.", "Which panchaka held.", "Panchaka"),
            span_section(10, "moon_signs", "The signs the Moon stood in, with when it entered and left each.", "Which sign the Moon was in.", "Rashi"),
            span_section(11, "sun_signs", "The signs the Sun stood in; two only on a sankranti day.", "Which sign the Sun was in.", "Rashi"),
            panchanga_kaalas_section(12),
            panchanga_choghadiya_section(13),
            panchanga_horas_section(14),
            panchanga_muhurtas_section(15),
            panchanga_moon_events_section(16),
            panchanga_muhurta_yogas_section(17),
            SectionSchema::bytes(
                18,
                "model",
                "UTF-8 text: the solar model that reckoned the days, as it describes itself.",
            ),
            SectionSchema::bytes(
                19,
                "provenance_json",
                "UTF-8 JSON: the provenance envelope of the result, canonical.",
            ),
            content_hashes_section(20, "day", "its panchanga"),
            SectionSchema::bytes(
                21,
                "muhurta",
                "UTF-8 JSON, canonical: the muhurta search `muhurta_json` asked for over these days, as its envelope `{value, provenance}` (`03-design/muhurta-at-the-boundary.md`). `value` is `{windows, closed, daysJudged, daysCut, windowsBlackedOut, ranking, unjudged}`: each window `{at, clauses, barredBy, score}`, an interval `{from, to}` in UTC Julian days, every clause `{clause, at, ...}` tagged by its kind with that kind's fields beside the tag, `barredBy` the bars that struck it (a clause's key or a clause), and `score` the baseline's `{value, factors, cappedAt}` under the `BASELINE` ranking, else null; each closed day `{date, by}`, the blackouts that closed it. Every catalogue member is written as its full key (`nakshatra.ROHINI`), and the envelope is sealed over exactly this value. Empty when `muhurta_json` asked for none.",
            ),
            SectionSchema::bytes(
                22,
                "festivals",
                "UTF-8 JSON, canonical: the days the rules `festivals_json` asked for fall on over these days, as the envelope `{value, provenance}` (`03-design/festival-rules.md` §7). `value` is `{observances, ekadashis, unjudged}`: each observance `{rule, day, tithi, month, adhika, case, extents, decidedBy, choice}`, the rule's key, the calendar date it falls on, the occurrence judged `{from, to}` in UTC Julian days (the tithi's, or the nakshatra's for a rule kept on one), its amanta month as a full key and whether that month is adhika, the case between its two days (`EARLIER_ONLY`, `LATER_ONLY`, `BOTH`, `NEITHER`, `EQUAL_PARTS`, `UNEQUAL_PARTS`), each day's extent `{day, window, held}`, `decidedBy` `{by: GUARD, index}`, `{by: OTHERWISE}` or, for a rule counted from another's day, `{by: AFTER, rule, days}` (its tithi, case and extents then the counted-from observance's), and the choice that decided (`EARLIER`, `LATER` or `BY_YUGMA`, which `day` resolves); each Ekadashi fast `{rule, tithi, month, adhika, tithis, days, piercedAt, pierced, excess, choice, day}`, the rule's key, the 11th and its lunar month as full keys, whether that month is adhika, the 10th, the 11th and the 12th as `{from, to}`, the 11th's own day and the next, where the 10th reached into the 11th's day (`SUNRISE`, `ARUNODAYA` or null), whether that counts under the rule's vedha, which of the 11th and the 12th holds the next sunrise (`ELEVENTH`, `TWELFTH`, `BOTH`, `NEITHER`), and the choice (`EARLIER` or `LATER`, which `day` resolves); each unjudged occurrence `{rule, tithi, why}`. A date's calendar and era are written as full keys (`calendar.GREGORIAN`), and the envelope is sealed over exactly this value; its provenance names the widened days as `festival.days`. Empty when `festivals_json` asked for none.",
            ),
            SectionSchema::bytes(
                23,
                "years",
                "UTF-8 JSON, canonical: the lunar years these days fall in when `sections` asked for `TS_PANCHANGA_YEARS`, as the envelope `{value, provenance}` (`03-design/calendar-indian-lunisolar.md` §10). `value` is a list of years in order, each `{samvatsara, count, vikrama, shaka, opened, began, ended, jovian, lupta}`: the name the year carries as a full key (`samvatsara.PRAMADICHA`), the count that named it under `calendars.samvatsara` (`BARHASPATYA`, `BARHASPATYA_RUNNING`, `CHANDRAMANA`), its Vikrama and Shaka numbers, the new moon that opened its first Chaitra, the sunrise of Chaitra Shukla Pratipada that began it and the next year's that ended it, in UTC Julian days; the Jovian years that ran in it, each `{member, count, from, to}` with its member in full, the signs mean Jupiter had crossed since the Kali age began, counted from 0, and its bounds in UTC Julian days; and the Jovian year expunged in it, in full, or null. Consecutive years abut, `ended` to `began`. Empty when `sections` did not ask for them.",
            ),
            SectionSchema::bytes(
                24,
                "eclipses",
                "UTF-8 JSON, canonical: the eclipses whose greatest moment falls between the first day's local midnight and the midnight after the last, when `sections` asked for `TS_PANCHANGA_ECLIPSES`, as the envelope `{value, provenance}` (`03-design/eclipses.md`). `value` is `{lunar, solar}`, each a list in order of `{eclipse, here}`. A lunar `eclipse` is `{greatest, kind, gamma, umbralMagnitude, penumbralMagnitude, contacts, shadow}`: the kind `PENUMBRAL`, `PARTIAL` or `TOTAL`, gamma in Earth radii, the contacts `{p1, u1, u2, u3, u4, p4}` with the umbral ones null where the eclipse lacks them, and the shadow rule it was read under (`DANJON`, `CHAUVENET`, from `panchanga.eclipse_shadow`); its `here` is `{p1, u1, u2, greatest, u3, u4, p4, seen, umbralSeen}`, each moment `{at, altitudeDeg}` with the Moon's topocentric geometric altitude there, and `umbralSeen` the stretch of the umbral phase seen, or null. A solar `eclipse` is `{greatest, kind, gamma, magnitude, point}`: the kind `PARTIAL`, `ANNULAR`, `TOTAL` or `HYBRID`, and the point of greatest eclipse `{latitude, longitude}` in degrees; its `here` is null where the Moon's disc never touches the Sun's from the place, else `{kind, magnitude, obscuration, first, second, third, fourth, maximum, seen}`, the place's own contacts with the Sun's altitude at each, the second and third null outside a central path. `seen` is `{from, to}`, the stretch of the eclipse the body stands above `panchanga.eclipse_horizon`'s horizon, or null where the place does not see it. Every instant is a UT1 Julian day, UTC to within a second. Empty when `sections` did not ask for them.",
            ),
            SectionSchema::bytes(
                25,
                "nepal_sambat",
                "UTF-8 JSON, canonical: each day's Nepal Sambat date when `sections` asked for `TS_PANCHANGA_NEPAL_SAMBAT`, as the envelope `{value, provenance}` (`03-design/calendar-indian-lunisolar.md` §11). `value` is a list in the days' order, each `{year, month, kind, paksha}`: the year, which opens at Kachhala's first day (1146 from 2025-10-22); the month, 1 for Kachhala (amanta Kartika) to 12 for Kaula (amanta Ashwina), an adhika month keeping the number of the month it repeats; the month's kind (`NIJA`, `ADHIKA`, which is Anala, or `KSHAYA`); and the half as a full key, `paksha.SHUKLA` (thwa) or `paksha.KRISHNA` (ga). The provenance is the days' own. Empty when `sections` did not ask for them.",
            ),
        ],
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::expect_used,
        reason = "a test fails by panicking, and the message names the schema at fault"
    )]

    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn schema_ids_and_section_ids_are_unique() {
        let all = schemas();
        let ids: BTreeSet<u32> = all.iter().map(|s| s.id).collect();
        assert_eq!(ids.len(), all.len());
        for schema in &all {
            let sections: BTreeSet<u32> = schema.sections.iter().map(|s| s.id).collect();
            assert_eq!(sections.len(), schema.sections.len(), "{}", schema.name);
            assert!(schema.sections.iter().all(|s| !s.doc.is_empty()));
        }
        assert!(schema(POSITIONS).is_some() && schema("nope").is_none());
        // A field name becomes an identifier in four generated surfaces,
        // so a repeat is a defect that reaches a binding rather than a
        // compile error here.
        for schema in &all {
            for section in &schema.sections {
                let names: BTreeSet<&str> =
                    section.fields.iter().map(|f| f.name.as_str()).collect();
                assert_eq!(
                    names.len(),
                    section.fields.len(),
                    "{}.{} repeats a field name",
                    schema.name,
                    section.name
                );
            }
        }
    }

    /// The day section is the shared one, and says so.
    ///
    /// The shape is what makes two blobs carrying a day decode into one
    /// type in each binding rather than two identical ones; a day
    /// section that lost its shape would still work and would quietly
    /// double the surface (`03-design/chart-at-the-boundary.md` §8).
    #[test]
    fn the_day_section_declares_its_shape() {
        let chart = schema(CHARTS).expect("the charts schema");
        let (_, day) = chart.section("day").expect("a day section");
        assert_eq!(day.shape.as_deref(), Some(DAY_SHAPE));
    }
}
