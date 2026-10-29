//! A shape two blobs share is one type in every binding, so the sections
//! naming it must agree: each way two can differ is refused, naming the
//! shape, both sections and the difference.

use teistro_idl::model::check_shapes;
use teistro_idl::{BlobSchema, ColumnDef, Scalar, SectionSchema};

fn blob(name: &str, id: u32, section: SectionSchema) -> BlobSchema {
    BlobSchema {
        name: name.into(),
        id,
        doc: String::from("an example"),
        sections: vec![
            SectionSchema::bytes(9, "text", "unshaped, so never compared"),
            section,
        ],
    }
}

fn day(fields: Vec<ColumnDef>) -> SectionSchema {
    SectionSchema::columns(1, "day", "one day", fields).of_shape("day")
}

fn jd() -> ColumnDef {
    ColumnDef::new("jd", Scalar::F64, "the instant")
}

fn weekday() -> ColumnDef {
    ColumnDef::new("weekday", Scalar::U8, "the weekday")
}

fn refused(other: SectionSchema) -> String {
    check_shapes(&[
        blob("charts", 3, day(vec![jd(), weekday()])),
        blob("panchanga", 4, other),
    ])
    .err()
    .unwrap_or_default()
}

const WHERE: &str = "shape `day`: `charts.day` and `panchanga.day`";

#[test]
fn two_sections_of_one_shape_agree_or_are_refused_for_how_they_differ() {
    assert_eq!(
        check_shapes(&[
            blob("charts", 3, day(vec![jd(), weekday()])),
            blob("panchanga", 4, day(vec![jd(), weekday()])),
            blob("naam", 5, day(vec![jd(), weekday()])),
        ]),
        Ok(())
    );
    let fixed = SectionSchema::fixed(1, "day", "one day", vec![jd(), weekday()]).of_shape("day");
    assert_eq!(
        refused(fixed),
        format!("{WHERE} are Columns and Fixed; a shape is one kind")
    );
    assert_eq!(
        refused(day(vec![jd()])),
        format!("{WHERE} have 2 and 1 fields; a shape is one set")
    );
    assert_eq!(
        refused(day(vec![weekday(), jd()])),
        format!("{WHERE} order their fields differently: `jd` against `weekday`")
    );
    let mut narrow = weekday();
    narrow.scalar = Scalar::U16;
    assert_eq!(
        refused(day(vec![jd(), narrow])),
        format!("{WHERE} disagree on `weekday`: U8 against U16")
    );
    let mut named = weekday();
    named.enum_name = Some(String::from("TsVara"));
    assert_eq!(
        refused(day(vec![jd(), named])),
        format!("{WHERE} disagree on what `weekday` names: None against Some(\"TsVara\")")
    );
}
