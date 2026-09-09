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

/// A rendered message: the text, where it resolved from, and the warnings.
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
                    ColumnDef::new("latitude_deg", Scalar::F64, "The place's latitude, degrees north."),
                    ColumnDef::new("longitude_deg", Scalar::F64, "The place's longitude, degrees east."),
                    ColumnDef::new("altitude_m", Scalar::F64, "The place's altitude, metres."),
                ],
            ),
            chart_cast_section(2),
            chart_grahas_section(3),
            chart_readings_section(4),
            SectionSchema::columns(
                5,
                "houses",
                "The twelve bhavas for \"which house is it in\", charts outermost: row `i * 12 + j` is chart `i`, bhava `j`, first to twelfth.",
                vec![
                    ColumnDef::new("madhya_deg", Scalar::F64, "The bhava's centre, degrees."),
                    ColumnDef::new("sandhi_deg", Scalar::F64, "The bhava's opening cusp, degrees."),
                ],
            ),
            SectionSchema::columns(
                6,
                "chalit",
                "The twelve bhavas of each chart's chalit, the same shape as `houses`.",
                vec![
                    ColumnDef::new("madhya_deg", Scalar::F64, "The bhava's centre, degrees."),
                    ColumnDef::new("sandhi_deg", Scalar::F64, "The bhava's opening cusp, degrees."),
                ],
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
        ],
    }
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
