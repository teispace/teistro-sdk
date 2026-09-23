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
                "Which sunrise convention the arc was reckoned by; `0xFF` for a custom altitude.",
            )
            .of_enum("TsSunrise"),
            ColumnDef::new(
                "convention_value",
                Scalar::F64,
                "The altitude in degrees when the convention is custom; zero otherwise.",
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
                "provenance",
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
    SectionSchema::columns(
        id,
        "grahas",
        "One row per graha per chart, charts outermost: row `i * graha_count + j` is chart `i`, graha `j`, grahas in the catalogue's order. `house_*` is the bhava for \"which house is it in\"; `placement_*` is the chart's chalit, which is a different question and often a different answer.",
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
            SectionSchema::fixed(
                1,
                "summary",
                "What the batch decided once: where, what kind, and how many of what.",
                vec![
                    ColumnDef::new("kind", Scalar::U16, "What kind of chart these are.").of_enum("ChartKind"),
                    ColumnDef::new("chart_count", Scalar::U32, "How many charts the batch holds, and how many rows the `cast`, `day` and `timing` sections each hold."),
                    ColumnDef::new("graha_count", Scalar::U32, "How many grahas each chart holds; the `grahas` section holds `chart_count * graha_count` rows."),
                    ColumnDef::new("varga_count", Scalar::U32, "How many divisional charts were asked for, in the order asked; zero when none were. The `vargas` section holds `chart_count * varga_count` rows and `varga_grahas` holds `chart_count * varga_count * graha_count`."),
                    ColumnDef::new("dasha_count", Scalar::U32, "How many dashas were asked for, in the order asked; zero when none were. The `dashas` section holds `chart_count * dasha_count` rows."),
                    ColumnDef::new("latitude_deg", Scalar::F64, "The place's latitude, degrees north."),
                    ColumnDef::new("longitude_deg", Scalar::F64, "The place's longitude, degrees east."),
                    ColumnDef::new("altitude_m", Scalar::F64, "The place's altitude, metres."),
                ],
            ),
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
                "provenance",
                "UTF-8 JSON: the provenance envelope of the result, canonical.",
            ),
            chart_vargas_section(13),
            chart_varga_grahas_section(14),
            chart_aspects_section(15),
            chart_drishti_table_section(16),
            chart_points_section(17),
            chart_bhavas_section(18),
            chart_states_section(19),
            SectionSchema::bytes(
                20,
                "combustion_orbs",
                "UTF-8 text: the combustion table the settings named, which every `burning` above was judged against. Empty when the states were not asked for.",
            ),
            SectionSchema::bytes(
                21,
                "drawings",
                "UTF-8 JSON, canonical: an array with one entry per chart, each the array of that chart's drawings in the order asked for, every drawing `{varga, placed}` exactly as the document schema describes `Drawing` (`03-design/chart-geometry.md`). Empty when no drawings were asked for.",
            ),
            SectionSchema::bytes(
                22,
                "svgs",
                "UTF-8 JSON, canonical: an array with one entry per chart, each the array of that chart's drawings written as SVG strings, in the order asked for, in the request's theme and the context's locale (`03-design/render-svg.md`). Empty when no theme was given.",
            ),
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
        ]
        .into_iter()
        .chain(chart_annual_sections())
        .collect(),
    }
}

/// The annual charts a batch's births open and everything Tajika reads
/// from them, in id order: seven sections ragged under one another, so
/// they are declared together rather than scattered through the rest.
fn chart_annual_sections() -> [SectionSchema; 7] {
    [
        chart_praveshas_section(35),
        chart_annual_charts_section(36),
        chart_year_claims_section(37),
        chart_year_yogas_section(38),
        chart_year_matters_section(39),
        chart_matter_yogas_section(40),
        chart_matter_legs_section(41),
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

/// Every dasha's periods, depth first in time order.
fn chart_dasha_periods_section(id: u32) -> SectionSchema {
    SectionSchema::columns(
        id,
        "dasha_periods",
        "Every dasha's periods of its birth cycle, concatenated in the `dashas` section's order and **ragged** by its `period_count`, each dasha's depth first in time order: a mahadasha, then its antardashas and theirs, then the next mahadasha. A period's path is its `index` below the nearest earlier period one `level` up.",
        vec![
            ColumnDef::new("level", Scalar::U8, "How deep: 1 for a mahadasha."),
            ColumnDef::new(
                "index",
                Scalar::U8,
                "Its place in its parent's sequence, from 0; under the elapsed reading of the birth period the first may not be 0.",
            ),
            ColumnDef::new(
                "sign",
                Scalar::U16,
                "The sign it is the period of, when its dasha is `signed`; zero otherwise.",
            )
            .of_enum("Rashi"),
            ColumnDef::new("lord", Scalar::U16, "Its lord.").of_enum("Graha"),
            ColumnDef::new(
                "from_jd",
                Scalar::F64,
                "When it begins, a Julian day (UTC).",
            ),
            ColumnDef::new("to_jd", Scalar::F64, "When it ends, a Julian day (UTC)."),
        ],
    )
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
                "provenance",
                "UTF-8 JSON: the provenance envelope of the result, canonical.",
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
