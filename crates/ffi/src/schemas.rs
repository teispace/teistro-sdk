//! The result blob schemas: what each blob-returning entry point writes,
//! declared once here, read by the encoder in this crate and by every
//! generated decoder through `idl/api.json`. A schema is appended to,
//! never reordered; a new section gets a new id.

use teistro_idl::model::{BlobSchema, ColumnDef, Scalar, SectionSchema};

/// The schema `ts_positions` fills.
pub const POSITIONS: &str = "positions";
/// The schema `ts_intl_render` fills.
pub const INTL_RENDER: &str = "intl_render";

/// Every schema, in id order.
#[must_use]
pub fn schemas() -> Vec<BlobSchema> {
    vec![positions(), intl_render()]
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
            SectionSchema::fixed(
                1,
                "summary",
                "The grid and the frame the values are in.",
                vec![
                    ColumnDef::new("frame_bits", Scalar::U32, "The frame the values are in, packed as the port packs it."),
                    ColumnDef::new("jd_count", Scalar::U32, "The number of instants."),
                    ColumnDef::new("body_count", Scalar::U32, "The number of bodies."),
                    ColumnDef::new("scale", Scalar::U32, "The time scale of the instants.").of_enum("TimeScale"),
                ],
            ),
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

#[cfg(test)]
mod tests {
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
    }
}
