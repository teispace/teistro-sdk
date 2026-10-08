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
/// The naam milan blob's name.
pub const NAAM: &str = "naam";

#[cfg(feature = "chart")]
mod charts;
#[cfg(feature = "chart")]
pub(crate) use charts::SignedBy;
#[cfg(feature = "chart")]
pub use charts::{charts, naam};

/// Every schema this build writes, in id order: a build without the chart
/// area writes no chart or naam blob.
#[must_use]
pub fn schemas() -> Vec<BlobSchema> {
    let mut all = vec![positions(), intl_render()];
    #[cfg(feature = "chart")]
    all.push(charts());
    all.push(panchanga());
    #[cfg(feature = "chart")]
    all.push(naam());
    all
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

/// The name every section holding a day arc declares, so that two blobs
/// carrying the same day decode into **one** type in each binding rather
/// than two identical ones (`03-design/chart-at-the-boundary.md` §8).
pub const DAY_SHAPE: &str = "day";

/// The shapes a match crosses as, shared by a chart's match with its
/// partner and two names' match, so each binding reads both through one
/// type and one reader (`03-design/matching.md`).
pub const MATCHINGS_SHAPE: &str = "matchings";
/// See [`MATCHINGS_SHAPE`].
pub const MATCHING_KOOTAS_SHAPE: &str = "matching_kootas";
/// See [`MATCHINGS_SHAPE`].
pub const PORUTHAMS_SHAPE: &str = "poruthams";
/// See [`MATCHINGS_SHAPE`].
pub const PORUTHAM_ROWS_SHAPE: &str = "porutham_rows";

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
