//! The result blob (`TSRB`): the designed wire encoding of every tree- or
//! grid-shaped result (`docs/02-architecture/07-binding-architecture.md`,
//! ADR-0007). One schema-driven [`Writer`] encodes it in the C ABI crate;
//! this [`Reader`] and every binding's generated decoder read it the same
//! way, so a decoder can be checked against the encoder byte for byte.
//!
//! Layout, little-endian throughout:
//!
//! ```text
//! header   32 bytes   magic "TSRB", version, section count, total length,
//!                     schema id, three reserved words
//! table    16 bytes   per section: id, offset, length, count
//! sections 8-aligned  fixed: one 8-byte slot per field, in field order
//!                     columns: a directory of u32 offsets from the section
//!                              start, one per column, then the columns,
//!                              each 8-aligned, `count` elements each
//!                     bytes:   raw bytes, `count` of them
//! ```
//!
//! A decoder reads the version first and refuses one it was not generated
//! for with a typed error; the table lets it find a section by id without
//! knowing the others, so a section appended in a later minor version is
//! skipped by an older decoder.

use core::fmt;

use crate::model::{BlobSchema, Scalar, SectionKind, SectionSchema};

/// The bytes `TSRB` read as a little-endian word.
pub const MAGIC: u32 = 0x4252_5354;
/// The layout version this module writes and reads.
pub const VERSION: u32 = 1;
/// The header's length in bytes.
pub const HEADER_LEN: usize = 32;
/// A table entry's length in bytes.
pub const ENTRY_LEN: usize = 16;
/// The slot every fixed field occupies and every column starts on.
pub const SLOT: usize = 8;

/// What went wrong writing or reading a blob.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlobError {
    /// A section name the schema lacks.
    UnknownSection(String),
    /// A section written twice, or a fixed section given the wrong number
    /// of values.
    Shape(String),
    /// A value of the wrong scalar type for its field or column.
    Type {
        /// The section.
        section: String,
        /// The field or column.
        field: String,
        /// The scalar the schema expects.
        expected: Scalar,
    },
    /// A section the schema has that was never written.
    Missing(String),
    /// Bytes that are not a blob of this version and schema.
    Malformed(String),
}

impl fmt::Display for BlobError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlobError::UnknownSection(name) => write!(f, "no section `{name}` in the schema"),
            BlobError::Shape(detail) => write!(f, "{detail}"),
            BlobError::Type {
                section,
                field,
                expected,
            } => write!(
                f,
                "`{section}.{field}` takes {} values",
                expected.rust_name()
            ),
            BlobError::Missing(name) => write!(f, "section `{name}` was never written"),
            BlobError::Malformed(detail) => write!(f, "not a well-formed blob: {detail}"),
        }
    }
}

impl std::error::Error for BlobError {}

/// One scalar read from a blob, widened to the largest representation of
/// its family.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScalarValue {
    /// A signed integer.
    Int(i64),
    /// An unsigned integer.
    Uint(u64),
    /// A floating-point number.
    Float(f64),
}

impl ScalarValue {
    /// The value as `f64`, converting integers.
    #[must_use]
    #[allow(
        clippy::cast_precision_loss,
        reason = "a widening read for tests and tools"
    )]
    pub fn as_f64(self) -> f64 {
        match self {
            ScalarValue::Int(v) => v as f64,
            ScalarValue::Uint(v) => v as f64,
            ScalarValue::Float(v) => v,
        }
    }

    /// The value as `i64`; a float is truncated.
    #[must_use]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        reason = "a widening read for tests and tools"
    )]
    pub fn as_i64(self) -> i64 {
        match self {
            ScalarValue::Int(v) => v,
            ScalarValue::Uint(v) => v as i64,
            ScalarValue::Float(v) => v as i64,
        }
    }
}

/// A column's values, borrowed from the caller with their native type.
#[derive(Debug, Clone, Copy)]
pub enum ColumnData<'a> {
    /// `u8` values.
    U8(&'a [u8]),
    /// `u16` values.
    U16(&'a [u16]),
    /// `u32` values.
    U32(&'a [u32]),
    /// `u64` values.
    U64(&'a [u64]),
    /// `i8` values.
    I8(&'a [i8]),
    /// `i16` values.
    I16(&'a [i16]),
    /// `i32` values.
    I32(&'a [i32]),
    /// `i64` values.
    I64(&'a [i64]),
    /// `f32` values.
    F32(&'a [f32]),
    /// `f64` values.
    F64(&'a [f64]),
}

impl ColumnData<'_> {
    /// The scalar the data is.
    #[must_use]
    pub const fn scalar(&self) -> Scalar {
        match self {
            ColumnData::U8(_) => Scalar::U8,
            ColumnData::U16(_) => Scalar::U16,
            ColumnData::U32(_) => Scalar::U32,
            ColumnData::U64(_) => Scalar::U64,
            ColumnData::I8(_) => Scalar::I8,
            ColumnData::I16(_) => Scalar::I16,
            ColumnData::I32(_) => Scalar::I32,
            ColumnData::I64(_) => Scalar::I64,
            ColumnData::F32(_) => Scalar::F32,
            ColumnData::F64(_) => Scalar::F64,
        }
    }

    /// The element count.
    #[must_use]
    pub const fn len(&self) -> usize {
        match self {
            ColumnData::U8(v) => v.len(),
            ColumnData::U16(v) => v.len(),
            ColumnData::U32(v) => v.len(),
            ColumnData::U64(v) => v.len(),
            ColumnData::I8(v) => v.len(),
            ColumnData::I16(v) => v.len(),
            ColumnData::I32(v) => v.len(),
            ColumnData::I64(v) => v.len(),
            ColumnData::F32(v) => v.len(),
            ColumnData::F64(v) => v.len(),
        }
    }

    /// Whether the column is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn write_into(&self, buf: &mut Vec<u8>) {
        match self {
            ColumnData::U8(v) => buf.extend_from_slice(v),
            ColumnData::U16(v) => v
                .iter()
                .for_each(|x| buf.extend_from_slice(&x.to_le_bytes())),
            ColumnData::U32(v) => v
                .iter()
                .for_each(|x| buf.extend_from_slice(&x.to_le_bytes())),
            ColumnData::U64(v) => v
                .iter()
                .for_each(|x| buf.extend_from_slice(&x.to_le_bytes())),
            ColumnData::I8(v) => v
                .iter()
                .for_each(|x| buf.extend_from_slice(&x.to_le_bytes())),
            ColumnData::I16(v) => v
                .iter()
                .for_each(|x| buf.extend_from_slice(&x.to_le_bytes())),
            ColumnData::I32(v) => v
                .iter()
                .for_each(|x| buf.extend_from_slice(&x.to_le_bytes())),
            ColumnData::I64(v) => v
                .iter()
                .for_each(|x| buf.extend_from_slice(&x.to_le_bytes())),
            ColumnData::F32(v) => v
                .iter()
                .for_each(|x| buf.extend_from_slice(&x.to_le_bytes())),
            ColumnData::F64(v) => v
                .iter()
                .for_each(|x| buf.extend_from_slice(&x.to_le_bytes())),
        }
    }
}

/// The scalar a fixed field holds when a value is written: an integer or a
/// float, checked against the schema.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FixedValue {
    /// An integer, for any integer field.
    Int(i64),
    /// An unsigned integer, for any integer field.
    Uint(u64),
    /// A float, for `f32` and `f64` fields.
    Float(f64),
}

impl From<i64> for FixedValue {
    fn from(v: i64) -> FixedValue {
        FixedValue::Int(v)
    }
}

impl From<u64> for FixedValue {
    fn from(v: u64) -> FixedValue {
        FixedValue::Uint(v)
    }
}

impl From<u32> for FixedValue {
    fn from(v: u32) -> FixedValue {
        FixedValue::Uint(u64::from(v))
    }
}

impl From<u8> for FixedValue {
    fn from(v: u8) -> FixedValue {
        FixedValue::Uint(u64::from(v))
    }
}

impl From<f64> for FixedValue {
    fn from(v: f64) -> FixedValue {
        FixedValue::Float(v)
    }
}

/// Encodes one blob against its schema. Sections may be written in any
/// order and each exactly once; [`Writer::finish`] refuses a blob with a
/// section missing.
#[derive(Debug)]
pub struct Writer<'s> {
    schema: &'s BlobSchema,
    buf: Vec<u8>,
    written: Vec<bool>,
}

impl<'s> Writer<'s> {
    /// A writer with the header and the table reserved.
    #[must_use]
    pub fn new(schema: &'s BlobSchema) -> Writer<'s> {
        let table_len = schema.sections.len() * ENTRY_LEN;
        let mut buf = Vec::with_capacity(HEADER_LEN + table_len + 256);
        buf.extend_from_slice(&MAGIC.to_le_bytes());
        buf.extend_from_slice(&VERSION.to_le_bytes());
        buf.extend_from_slice(&len_u32(schema.sections.len()).to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(&schema.id.to_le_bytes());
        buf.resize(HEADER_LEN + table_len, 0);
        Writer {
            schema,
            buf,
            written: vec![false; schema.sections.len()],
        }
    }

    fn begin(
        &mut self,
        name: &str,
        kind: SectionKind,
    ) -> Result<(usize, &'s SectionSchema), BlobError> {
        let (index, section) = self
            .schema
            .section(name)
            .ok_or_else(|| BlobError::UnknownSection(name.to_string()))?;
        if section.kind != kind {
            return Err(BlobError::Shape(format!(
                "section `{name}` is {:?}, not {kind:?}",
                section.kind
            )));
        }
        if self.written.get(index).copied().unwrap_or(false) {
            return Err(BlobError::Shape(format!("section `{name}` written twice")));
        }
        self.align();
        Ok((index, section))
    }

    fn end(&mut self, index: usize, start: usize, count: usize) {
        let length = self.buf.len() - start;
        let at = HEADER_LEN + index * ENTRY_LEN;
        let id = self.schema.sections.get(index).map_or(0, |s| s.id);
        self.patch_u32(at, id);
        self.patch_u32(at + 4, len_u32(start));
        self.patch_u32(at + 8, len_u32(length));
        self.patch_u32(at + 12, len_u32(count));
        if let Some(slot) = self.written.get_mut(index) {
            *slot = true;
        }
    }

    /// Writes a fixed section: one value per field, in field order.
    ///
    /// # Errors
    ///
    /// An unknown or repeated section, a wrong value count, or a value of
    /// the wrong family for its field.
    pub fn fixed(&mut self, name: &str, values: &[FixedValue]) -> Result<(), BlobError> {
        let (index, section) = self.begin(name, SectionKind::Fixed)?;
        if values.len() != section.fields.len() {
            return Err(BlobError::Shape(format!(
                "section `{name}` has {} fields, {} values given",
                section.fields.len(),
                values.len()
            )));
        }
        let start = self.buf.len();
        for (field, value) in section.fields.iter().zip(values) {
            let bytes = match (field.scalar.is_float(), value) {
                (true, FixedValue::Float(v)) => v.to_le_bytes(),
                (false, FixedValue::Int(v)) => v.to_le_bytes(),
                (false, FixedValue::Uint(v)) => v.to_le_bytes(),
                _ => {
                    return Err(BlobError::Type {
                        section: name.to_string(),
                        field: field.name.clone(),
                        expected: field.scalar,
                    });
                }
            };
            self.buf.extend_from_slice(&bytes);
        }
        self.end(index, start, 1);
        Ok(())
    }

    /// Writes a column section: one column per field, each with `rows`
    /// elements of the field's scalar.
    ///
    /// # Errors
    ///
    /// An unknown or repeated section, a wrong column count, a column of
    /// the wrong length, or a column of the wrong scalar.
    pub fn columns(
        &mut self,
        name: &str,
        rows: usize,
        columns: &[ColumnData<'_>],
    ) -> Result<(), BlobError> {
        let (index, section) = self.begin(name, SectionKind::Columns)?;
        if columns.len() != section.fields.len() {
            return Err(BlobError::Shape(format!(
                "section `{name}` has {} columns, {} given",
                section.fields.len(),
                columns.len()
            )));
        }
        for (field, column) in section.fields.iter().zip(columns) {
            if column.scalar() != field.scalar {
                return Err(BlobError::Type {
                    section: name.to_string(),
                    field: field.name.clone(),
                    expected: field.scalar,
                });
            }
            if column.len() != rows {
                return Err(BlobError::Shape(format!(
                    "column `{name}.{}` has {} elements, not {rows}",
                    field.name,
                    column.len()
                )));
            }
        }
        let start = self.buf.len();
        let directory = start;
        self.buf.resize(start + columns.len() * 4, 0);
        for (i, column) in columns.iter().enumerate() {
            self.align();
            let offset = self.buf.len() - start;
            self.patch_u32(directory + i * 4, len_u32(offset));
            column.write_into(&mut self.buf);
        }
        self.end(index, start, rows);
        Ok(())
    }

    /// Writes a bytes section.
    ///
    /// # Errors
    ///
    /// An unknown or repeated section.
    pub fn bytes(&mut self, name: &str, data: &[u8]) -> Result<(), BlobError> {
        let (index, _) = self.begin(name, SectionKind::Bytes)?;
        let start = self.buf.len();
        self.buf.extend_from_slice(data);
        self.end(index, start, data.len());
        Ok(())
    }

    /// The bytes, once every section is written.
    ///
    /// # Errors
    ///
    /// A section of the schema that was never written.
    pub fn finish(mut self) -> Result<Vec<u8>, BlobError> {
        if let Some((index, _)) = self.written.iter().enumerate().find(|(_, w)| !**w) {
            let name = self
                .schema
                .sections
                .get(index)
                .map(|s| s.name.clone())
                .unwrap_or_default();
            return Err(BlobError::Missing(name));
        }
        self.align();
        let total = len_u32(self.buf.len());
        self.patch_u32(12, total);
        Ok(self.buf)
    }

    fn align(&mut self) {
        while self.buf.len() % SLOT != 0 {
            self.buf.push(0);
        }
    }

    fn patch_u32(&mut self, at: usize, value: u32) {
        if let Some(slot) = self.buf.get_mut(at..at + 4) {
            slot.copy_from_slice(&value.to_le_bytes());
        }
    }
}

fn len_u32(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}

/// One section as found in a blob's table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry {
    /// The section id.
    pub id: u32,
    /// The offset from the blob's start.
    pub offset: usize,
    /// The length in bytes.
    pub length: usize,
    /// The record, row or byte count.
    pub count: usize,
}

/// Decodes a blob against its schema, checking every offset before it is
/// read; the reference every generated decoder is held to.
#[derive(Debug)]
pub struct Reader<'b, 's> {
    bytes: &'b [u8],
    schema: &'s BlobSchema,
    entries: Vec<Entry>,
}

impl<'b, 's> Reader<'b, 's> {
    /// Parses the header and the table.
    ///
    /// # Errors
    ///
    /// A wrong magic, version or schema id, a length that disagrees with
    /// the bytes, or a table entry outside them.
    pub fn parse(bytes: &'b [u8], schema: &'s BlobSchema) -> Result<Reader<'b, 's>, BlobError> {
        let word = |at: usize| -> Result<u32, BlobError> {
            bytes
                .get(at..at + 4)
                .and_then(|b| b.try_into().ok())
                .map(u32::from_le_bytes)
                .ok_or_else(|| BlobError::Malformed(format!("{} bytes are too few", bytes.len())))
        };
        if word(0)? != MAGIC {
            return Err(BlobError::Malformed(String::from("the magic is not TSRB")));
        }
        let version = word(4)?;
        if version != VERSION {
            return Err(BlobError::Malformed(format!(
                "layout version {version}, this reader is version {VERSION}"
            )));
        }
        let count = word(8)? as usize;
        let total = word(12)? as usize;
        if total != bytes.len() {
            return Err(BlobError::Malformed(format!(
                "the header says {total} bytes, {} were given",
                bytes.len()
            )));
        }
        let schema_id = word(16)?;
        if schema_id != schema.id {
            return Err(BlobError::Malformed(format!(
                "schema id {schema_id}, expected {} (`{}`)",
                schema.id, schema.name
            )));
        }
        let mut entries = Vec::with_capacity(count);
        for i in 0..count {
            let at = HEADER_LEN + i * ENTRY_LEN;
            let entry = Entry {
                id: word(at)?,
                offset: word(at + 4)? as usize,
                length: word(at + 8)? as usize,
                count: word(at + 12)? as usize,
            };
            if entry
                .offset
                .checked_add(entry.length)
                .is_none_or(|end| end > total)
            {
                return Err(BlobError::Malformed(format!(
                    "section {} runs past the end",
                    entry.id
                )));
            }
            entries.push(entry);
        }
        Ok(Reader {
            bytes,
            schema,
            entries,
        })
    }

    /// The table entry of a section by name, when the blob has it.
    #[must_use]
    pub fn entry(&self, name: &str) -> Option<Entry> {
        let (_, section) = self.schema.section(name)?;
        self.entries.iter().copied().find(|e| e.id == section.id)
    }

    /// The count of a section: records, rows or bytes.
    #[must_use]
    pub fn count(&self, name: &str) -> Option<usize> {
        self.entry(name).map(|e| e.count)
    }

    /// The values of a fixed section, in field order.
    ///
    /// # Errors
    ///
    /// A section the schema lacks or the blob lacks, or a truncated one.
    pub fn fixed(&self, name: &str) -> Result<Vec<ScalarValue>, BlobError> {
        let (section, entry) = self.section(name, SectionKind::Fixed)?;
        section
            .fields
            .iter()
            .enumerate()
            .map(|(i, field)| self.scalar_at(entry.offset + i * SLOT, field.scalar, SLOT))
            .collect()
    }

    /// One column of a column section, decoded.
    ///
    /// # Errors
    ///
    /// A section or column the schema lacks, or a truncated one.
    pub fn column(&self, name: &str, column: &str) -> Result<Vec<ScalarValue>, BlobError> {
        let (section, entry) = self.section(name, SectionKind::Columns)?;
        let (index, field) = section
            .field(column)
            .ok_or_else(|| BlobError::UnknownSection(format!("{name}.{column}")))?;
        let offset = self
            .scalar_at(entry.offset + index * 4, Scalar::U32, 4)?
            .as_i64();
        let start = entry.offset + usize::try_from(offset).unwrap_or(usize::MAX);
        let width = field.scalar.width(8);
        (0..entry.count)
            .map(|row| self.scalar_at(start + row * width, field.scalar, width))
            .collect()
    }

    /// The bytes of a bytes section.
    ///
    /// # Errors
    ///
    /// A section the schema lacks or the blob lacks.
    pub fn bytes(&self, name: &str) -> Result<&'b [u8], BlobError> {
        let (_, entry) = self.section(name, SectionKind::Bytes)?;
        self.bytes
            .get(entry.offset..entry.offset + entry.count)
            .ok_or_else(|| BlobError::Malformed(format!("section `{name}` is truncated")))
    }

    /// The text of a bytes section.
    ///
    /// # Errors
    ///
    /// As [`Reader::bytes`], or bytes that are not UTF-8.
    pub fn text(&self, name: &str) -> Result<&'b str, BlobError> {
        core::str::from_utf8(self.bytes(name)?)
            .map_err(|_| BlobError::Malformed(format!("section `{name}` is not UTF-8")))
    }

    fn section(
        &self,
        name: &str,
        kind: SectionKind,
    ) -> Result<(&'s SectionSchema, Entry), BlobError> {
        let (_, section) = self
            .schema
            .section(name)
            .ok_or_else(|| BlobError::UnknownSection(name.to_string()))?;
        if section.kind != kind {
            return Err(BlobError::Shape(format!(
                "section `{name}` is {:?}, not {kind:?}",
                section.kind
            )));
        }
        let entry = self
            .entry(name)
            .ok_or_else(|| BlobError::Missing(name.to_string()))?;
        Ok((section, entry))
    }

    fn scalar_at(&self, at: usize, scalar: Scalar, width: usize) -> Result<ScalarValue, BlobError> {
        let bytes = self
            .bytes
            .get(at..at + width)
            .ok_or_else(|| BlobError::Malformed(format!("a read at {at} runs past the end")))?;
        let mut slot = [0u8; 8];
        if let Some(target) = slot.get_mut(..width) {
            target.copy_from_slice(bytes);
        }
        let raw = u64::from_le_bytes(slot);
        let bits = scalar.width(8) * 8;
        Ok(match scalar {
            Scalar::F64 => ScalarValue::Float(f64::from_bits(raw)),
            Scalar::F32 => ScalarValue::Float(f64::from(f32::from_bits(
                u32::try_from(raw & 0xFFFF_FFFF).unwrap_or(0),
            ))),
            s if s.is_signed() => {
                #[allow(
                    clippy::cast_possible_wrap,
                    reason = "a sign-extending reinterpretation"
                )]
                let signed = ((raw << (64 - bits)) as i64) >> (64 - bits);
                ScalarValue::Int(signed)
            }
            _ => ScalarValue::Uint(if bits == 64 {
                raw
            } else {
                raw & ((1u64 << bits) - 1)
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        clippy::float_cmp,
        reason = "tests fail by panicking and compare exact bits"
    )]

    use super::*;
    use crate::model::ColumnDef;

    fn schema() -> BlobSchema {
        BlobSchema {
            name: "demo".into(),
            id: 3,
            doc: String::new(),
            sections: vec![
                SectionSchema::fixed(
                    1,
                    "summary",
                    "",
                    vec![
                        ColumnDef::new("jd", Scalar::F64, ""),
                        ColumnDef::new("count", Scalar::U32, ""),
                        ColumnDef::new("delta", Scalar::I16, ""),
                    ],
                ),
                SectionSchema::columns(
                    2,
                    "rows",
                    "",
                    vec![
                        ColumnDef::new("lon", Scalar::F64, ""),
                        ColumnDef::new("status", Scalar::I32, ""),
                        ColumnDef::new("body", Scalar::U16, ""),
                    ],
                ),
                SectionSchema::bytes(3, "text", ""),
            ],
        }
    }

    #[test]
    fn a_blob_round_trips_with_every_section_aligned() {
        let schema = schema();
        let mut writer = Writer::new(&schema);
        writer.bytes("text", "namaste".as_bytes()).unwrap();
        writer
            .columns(
                "rows",
                3,
                &[
                    ColumnData::F64(&[1.5, 2.5, 3.5]),
                    ColumnData::I32(&[0, -1, -2]),
                    ColumnData::U16(&[7, 8, 9]),
                ],
            )
            .unwrap();
        writer
            .fixed(
                "summary",
                &[2_451_545.0.into(), 3u32.into(), FixedValue::Int(-5)],
            )
            .unwrap();
        let bytes = writer.finish().unwrap();
        assert_eq!(bytes.len() % SLOT, 0);
        let reader = Reader::parse(&bytes, &schema).unwrap();
        for name in ["summary", "rows"] {
            assert_eq!(reader.entry(name).unwrap().offset % SLOT, 0, "{name}");
        }
        assert_eq!(
            reader.fixed("summary").unwrap(),
            [
                ScalarValue::Float(2_451_545.0),
                ScalarValue::Uint(3),
                ScalarValue::Int(-5)
            ]
        );
        assert_eq!(reader.count("rows"), Some(3));
        assert_eq!(
            reader.column("rows", "status").unwrap(),
            [
                ScalarValue::Int(0),
                ScalarValue::Int(-1),
                ScalarValue::Int(-2)
            ]
        );
        assert_eq!(
            reader.column("rows", "body").unwrap(),
            [
                ScalarValue::Uint(7),
                ScalarValue::Uint(8),
                ScalarValue::Uint(9)
            ]
        );
        assert_eq!(reader.column("rows", "lon").unwrap()[2].as_f64(), 3.5);
        assert_eq!(reader.text("text").unwrap(), "namaste");
        let entry = reader.entry("rows").unwrap();
        let column_offset =
            u32::from_le_bytes(bytes[entry.offset..entry.offset + 4].try_into().unwrap());
        assert_eq!((entry.offset + column_offset as usize) % SLOT, 0);
    }

    #[test]
    fn the_writer_refuses_wrong_shapes_and_the_reader_wrong_bytes() {
        let schema = schema();
        let mut writer = Writer::new(&schema);
        assert!(matches!(
            writer.fixed("nope", &[]),
            Err(BlobError::UnknownSection(_))
        ));
        assert!(matches!(
            writer.fixed("summary", &[1.0.into()]),
            Err(BlobError::Shape(_))
        ));
        assert!(matches!(
            writer.fixed(
                "summary",
                &[FixedValue::Int(1), 3u32.into(), FixedValue::Int(0)]
            ),
            Err(BlobError::Type { .. })
        ));
        assert!(matches!(
            writer.columns(
                "rows",
                1,
                &[
                    ColumnData::F32(&[1.0]),
                    ColumnData::I32(&[0]),
                    ColumnData::U16(&[1])
                ]
            ),
            Err(BlobError::Type { .. })
        ));
        assert!(matches!(
            writer.columns(
                "rows",
                2,
                &[
                    ColumnData::F64(&[1.0]),
                    ColumnData::I32(&[0, 0]),
                    ColumnData::U16(&[1, 1])
                ]
            ),
            Err(BlobError::Shape(_))
        ));
        writer.bytes("text", b"").unwrap();
        assert!(matches!(
            writer.bytes("text", b""),
            Err(BlobError::Shape(_))
        ));
        assert!(matches!(writer.finish(), Err(BlobError::Missing(_))));

        assert!(matches!(
            Reader::parse(b"", &schema),
            Err(BlobError::Malformed(_))
        ));
        let mut writer = Writer::new(&schema);
        writer.bytes("text", b"x").unwrap();
        writer
            .columns(
                "rows",
                0,
                &[
                    ColumnData::F64(&[]),
                    ColumnData::I32(&[]),
                    ColumnData::U16(&[]),
                ],
            )
            .unwrap();
        writer
            .fixed("summary", &[0.0.into(), 0u32.into(), FixedValue::Uint(0)])
            .unwrap();
        let mut bytes = writer.finish().unwrap();
        let other = BlobSchema {
            id: 4,
            ..schema.clone()
        };
        assert!(matches!(
            Reader::parse(&bytes, &other),
            Err(BlobError::Malformed(_))
        ));
        bytes[4] = 9;
        assert!(matches!(
            Reader::parse(&bytes, &schema),
            Err(BlobError::Malformed(_))
        ));
        bytes[4] = 1;
        bytes.pop();
        assert!(matches!(
            Reader::parse(&bytes, &schema),
            Err(BlobError::Malformed(_))
        ));
    }
}
