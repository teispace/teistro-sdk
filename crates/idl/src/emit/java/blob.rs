//! The result-blob decoders, in `com.teispace.teistro.blob`: a class per
//! schema reading the `TSRB` layout, a class per column section whose
//! getters read a row straight out of the blob's bytes, and a record per
//! shape two blobs share. Their own package, so a section's name can never
//! meet an enum's.
//!
//! A decoded blob holds the copy `Calls` took of the library's bytes, so
//! nothing it answers reads memory the library has freed; a column is read
//! row by row from that copy rather than copied again.

use std::collections::BTreeSet;
use std::fmt::Write;

use super::values::carrier;
use super::{File, identifier, javadoc, literal, preamble};
use crate::model::{Api, BlobSchema, ColumnDef, Scalar, SectionKind, SectionSchema};
use crate::names::{pascal, snake};

/// The decoders' package.
pub(super) const BLOB_PACKAGE: &str = "com.teispace.teistro.blob";

/// Every decoder file: the reader, the exception, the shapes, a class per
/// schema.
pub(super) fn files(api: &Api) -> Vec<File> {
    let dir = BLOB_PACKAGE.replace('.', "/");
    let mut files = vec![
        File {
            path: format!("{dir}/BlobReader.java"),
            text: format!("{}{READER}", preamble(api, BLOB_PACKAGE)),
        },
        File {
            path: format!("{dir}/BlobFormatException.java"),
            text: format!("{}{EXCEPTION}", preamble(api, BLOB_PACKAGE)),
        },
    ];
    let mut shapes = BTreeSet::new();
    for schema in &api.blobs {
        for section in &schema.sections {
            let Some(shape) = section.shape.as_deref() else {
                continue;
            };
            if shapes.insert(shape.to_string()) {
                files.push(File {
                    path: format!("{dir}/{}.java", type_name(shape)),
                    text: render_shape(api, section),
                });
            }
        }
        files.push(File {
            path: format!("{dir}/{}.java", type_name(&schema.name)),
            text: render_schema(api, schema),
        });
    }
    files
}

/// A schema's or a shape's class name (`charts` is `Charts`).
fn type_name(name: &str) -> String {
    pascal(&snake(name))
}

/// The class a column section decodes into: its shape's, or one nested in
/// the blob's class.
fn column_type(section: &SectionSchema) -> String {
    section
        .shape
        .as_deref()
        .map_or_else(|| type_name(&section.name), type_name)
}

/// What reads one value of a scalar at a byte offset of `bytes`, carried in
/// its Java type.
fn read(scalar: Scalar, at: &str) -> String {
    match scalar {
        Scalar::U8 => format!("Byte.toUnsignedInt(bytes.get({at}))"),
        Scalar::I8 => format!("(int) bytes.get({at})"),
        Scalar::U16 => format!("Short.toUnsignedInt(bytes.getShort({at}))"),
        Scalar::I16 => format!("(int) bytes.getShort({at})"),
        Scalar::U32 => format!("Integer.toUnsignedLong(bytes.getInt({at}))"),
        Scalar::I32 => format!("bytes.getInt({at})"),
        Scalar::U64 | Scalar::I64 | Scalar::Usize | Scalar::Isize => {
            format!("bytes.getLong({at})")
        }
        Scalar::F32 => format!("bytes.getFloat({at})"),
        Scalar::F64 => format!("bytes.getDouble({at})"),
        Scalar::Bool => format!("bytes.get({at}) != 0"),
    }
}

/// A field's doc, saying when its value is an unsigned 64-bit number Java
/// carries in a `long`.
fn field_doc(column: &ColumnDef) -> String {
    let mut doc = column.doc.clone();
    if let Some(name) = &column.enum_name {
        let _ = write!(
            doc,
            "\n\nThe id of a {{@code {}}}.",
            crate::names::binding_type_name(name)
        );
    }
    if matches!(column.scalar, Scalar::U64 | Scalar::Usize) {
        doc.push_str("\n\nUnsigned: read it with `Long.toUnsignedString`.");
    }
    doc
}

/// A shape: a fixed section as a record, or a column section as a class,
/// one type wherever a blob carries it.
fn render_shape(api: &Api, section: &SectionSchema) -> String {
    let mut out = preamble(api, BLOB_PACKAGE);
    match section.kind {
        SectionKind::Columns => {
            out.push_str("import java.nio.ByteBuffer;\nimport java.util.Objects;\n\n");
            render_columns(&mut out, section, "", "public final class");
        }
        _ => render_fixed_record(&mut out, section),
    }
    out
}

/// A fixed section as a record, read eagerly: its fields are few.
fn render_fixed_record(out: &mut String, section: &SectionSchema) {
    let name = column_type(section);
    let mut doc = section.doc.clone();
    doc.push('\n');
    for field in &section.fields {
        let _ = write!(
            doc,
            "\n@param {} {}",
            identifier(&field.name),
            field_doc(field).replace("\n\n", " ")
        );
    }
    out.push_str(&javadoc(&doc, "").replace("&#64;param", "@param"));
    let components: Vec<String> = section
        .fields
        .iter()
        .map(|f| format!("{} {}", carrier(f.scalar), identifier(&f.name)))
        .collect();
    let reads: Vec<String> = section
        .fields
        .iter()
        .enumerate()
        .map(|(slot, f)| read_slot(f.scalar, slot))
        .collect();
    let _ = writeln!(
        out,
        "public record {name}(\n        {}) {{\n    static {name} of(BlobReader blob, int id, String name) {{\n        BlobReader.Section at = blob.fixed(id, name, {slots});\n        java.nio.ByteBuffer bytes = blob.bytes();\n        return new {name}(\n            {});\n    }}\n}}",
        components.join(",\n        "),
        reads.join(",\n            "),
        slots = section.fields.len()
    );
}

/// What reads the field in a slot of the fixed section at `at`.
fn read_slot(scalar: Scalar, slot: usize) -> String {
    read(scalar, &format!("at.offset() + {}", slot * 8))
}

/// A column section's class: the offset of each column, and a getter per
/// column reading one row.
fn render_columns(out: &mut String, section: &SectionSchema, indent: &str, head: &str) {
    let name = column_type(section);
    let whose = if section.shape.is_some() {
        "wherever a blob carries it"
    } else {
        "of this blob"
    };
    out.push_str(&javadoc(
        &format!(
            "The `{}` section {whose}: a getter per column, reading a row from the blob's bytes.\n\n{}",
            section.name, section.doc
        ),
        indent,
    ));
    let _ = writeln!(
        out,
        "{indent}{head} {name} {{\n{indent}    private final ByteBuffer bytes;\n{indent}    private final int length;"
    );
    for column in &section.fields {
        let _ = writeln!(
            out,
            "{indent}    private final int {}At;",
            identifier(&column.name)
        );
    }
    let _ = writeln!(
        out,
        "\n{indent}    {name}(BlobReader blob, int id, String name) {{\n{indent}        BlobReader.Section at = blob.section(id, name);\n{indent}        this.bytes = blob.bytes();\n{indent}        this.length = at.count();"
    );
    for (index, column) in section.fields.iter().enumerate() {
        let _ = writeln!(
            out,
            "{indent}        this.{}At = blob.column(at, {index}, {});",
            identifier(&column.name),
            column.scalar.width(8)
        );
    }
    let _ = writeln!(
        out,
        "{indent}    }}\n\n{indent}    /**\n{indent}     * The number of rows every column holds.\n{indent}     *\n{indent}     * @return the row count\n{indent}     */\n{indent}    public int length() {{\n{indent}        return length;\n{indent}    }}"
    );
    for column in &section.fields {
        let field = identifier(&column.name);
        let width = column.scalar.width(8);
        let doc = format!(
            "{}\n\n@param row the row, from 0\n@return the value in that row\n@throws IndexOutOfBoundsException for a row the section does not hold",
            field_doc(column)
        );
        out.push('\n');
        out.push_str(
            &javadoc(&doc, &format!("{indent}    "))
                .replace("&#64;param", "@param")
                .replace("&#64;return", "@return")
                .replace("&#64;throws", "@throws"),
        );
        let _ = writeln!(
            out,
            "{indent}    public {} {field}(int row) {{\n{indent}        return {};\n{indent}    }}",
            carrier(column.scalar),
            read(
                column.scalar,
                &format!("{field}At + Objects.checkIndex(row, length) * {width}")
            )
        );
    }
    let _ = writeln!(out, "{indent}}}");
}

/// A schema's class: a field per section (a fixed section without a
/// shape spreads its fields over the class), read when it is decoded.
fn render_schema(api: &Api, schema: &BlobSchema) -> String {
    let name = type_name(&schema.name);
    let mut out = preamble(api, BLOB_PACKAGE);
    out.push_str("import java.nio.ByteBuffer;\nimport java.nio.charset.StandardCharsets;\nimport java.util.Objects;\n\n");
    out.push_str(&javadoc(
        &format!("A decoded `{}` blob.\n\n{}", schema.name, schema.doc),
        "",
    ));
    let _ = writeln!(
        out,
        "public final class {name} {{\n    /** The schema's id, which the blob's header carries. */\n    public static final int SCHEMA_ID = {};\n",
        schema.id
    );
    // The fields, and what the constructor assigns them.
    let mut fields = String::new();
    let mut assigns = String::new();
    let mut getters = String::new();
    for section in &schema.sections {
        let label = literal(&section.name);
        let id = section.id;
        match (section.kind, section.shape.is_some()) {
            (SectionKind::Fixed, false) => {
                let _ = writeln!(
                    assigns,
                    "        BlobReader.Section {} = blob.fixed({id}, {label}, {});",
                    section_local(section),
                    section.fields.len()
                );
                for (slot, field) in section.fields.iter().enumerate() {
                    let f = identifier(&field.name);
                    let ty = carrier(field.scalar);
                    let _ = writeln!(fields, "    private final {ty} {f};");
                    let _ = writeln!(
                        assigns,
                        "        this.{f} = {};",
                        read(
                            field.scalar,
                            &format!("{}.offset() + {}", section_local(section), slot * 8)
                        )
                    );
                    getter(&mut getters, ty, &f, &field_doc(field));
                }
            }
            (SectionKind::Bytes, _) => {
                let f = identifier(&section.name);
                let _ = writeln!(fields, "    private final String {f};");
                let _ = writeln!(assigns, "        this.{f} = blob.text({id}, {label});");
                getter(&mut getters, "String", &f, &section.doc);
            }
            _ => {
                let f = identifier(&section.name);
                let ty = column_type(section);
                let _ = writeln!(fields, "    private final {ty} {f};");
                let construct = if section.kind == SectionKind::Fixed {
                    format!("{ty}.of(blob, {id}, {label})")
                } else {
                    format!("new {ty}(blob, {id}, {label})")
                };
                let _ = writeln!(assigns, "        this.{f} = {construct};");
                getter(&mut getters, &ty, &f, &section.doc);
            }
        }
    }
    let _ = writeln!(
        out,
        "{fields}\n    private {name}(BlobReader blob) {{\n        ByteBuffer bytes = blob.bytes();\n{assigns}    }}\n"
    );
    out.push_str(&javadoc(
        &format!(
            "Decodes a `{}` blob.\n\n@param raw the blob's bytes, which the result keeps and does not copy\n@return the decoded blob\n@throws BlobFormatException for bytes that are not a blob of this schema and layout version",
            schema.name
        ),
        "    ",
    ).replace("&#64;param", "@param").replace("&#64;return", "@return").replace("&#64;throws", "@throws"));
    let _ = writeln!(
        out,
        "    public static {name} decode(byte[] raw) {{\n        return new {name}(new BlobReader(Objects.requireNonNull(raw, \"raw\"), SCHEMA_ID, {}));\n    }}\n",
        literal(&schema.name)
    );
    out.push_str(&getters);
    for section in &schema.sections {
        if section.kind == SectionKind::Columns && section.shape.is_none() {
            out.push('\n');
            render_columns(&mut out, section, "    ", "public static final class");
        }
    }
    out.push_str("}\n");
    // A schema with no bytes section reads no text.
    if !schema.sections.iter().any(|s| s.kind == SectionKind::Bytes) {
        out = out.replace("import java.nio.charset.StandardCharsets;\n", "");
    }
    out
}

/// The local a fixed section's position is held in while it is read.
fn section_local(section: &SectionSchema) -> String {
    format!("{}Section", identifier(&section.name))
}

/// A getter.
fn getter(out: &mut String, ty: &str, name: &str, doc: &str) {
    let doc = format!("{doc}\n\n@return the {name}");
    out.push_str(&javadoc(&doc, "    ").replace("&#64;return", "@return"));
    let _ = writeln!(
        out,
        "    public {ty} {name}() {{\n        return {name};\n    }}\n"
    );
}

const EXCEPTION: &str = r"/**
 * Bytes that are not a result blob this decoder reads: too short, another
 * schema, another layout version, or a section that runs past the end.
 */
public final class BlobFormatException extends IllegalArgumentException {
    private static final long serialVersionUID = 1L;

    BlobFormatException(String message) {
        super(message);
    }
}
";

const READER: &str = r#"import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.util.HashMap;
import java.util.Map;

/**
 * A blob opened for reading: its bytes, little-endian, and its table of
 * contents, every offset checked against the blob's length before anything
 * is read through it.
 */
final class BlobReader {
    static final int MAGIC = 0x42525354;
    static final int VERSION = 1;
    static final int HEADER_LENGTH = 32;
    static final int ENTRY_LENGTH = 16;

    /** One section, as the table of contents describes it. */
    record Section(int offset, int length, int count) {}

    private final ByteBuffer bytes;
    private final Map<Integer, Section> sections = new HashMap<>();

    BlobReader(byte[] raw, int schemaId, String schemaName) {
        if (raw.length < HEADER_LENGTH) {
            throw new BlobFormatException("not a Teistro result blob: too short for a header");
        }
        // Read-only first: a view's byte order is its own.
        this.bytes = ByteBuffer.wrap(raw).asReadOnlyBuffer().order(ByteOrder.LITTLE_ENDIAN);
        int magic = bytes.getInt(0);
        int version = bytes.getInt(4);
        long count = Integer.toUnsignedLong(bytes.getInt(8));
        long total = Integer.toUnsignedLong(bytes.getInt(12));
        int identifier = bytes.getInt(16);
        if (magic != MAGIC) {
            throw new BlobFormatException("not a Teistro result blob");
        }
        if (version != VERSION) {
            throw new BlobFormatException("blob layout version " + version + ", this decoder reads version " + VERSION);
        }
        if (total != raw.length) {
            throw new BlobFormatException("the header says " + total + " bytes, " + raw.length + " were given");
        }
        if (identifier != schemaId) {
            throw new BlobFormatException("blob schema " + identifier + ", expected " + schemaId + " (" + schemaName + ")");
        }
        for (long index = 0; index < count; index += 1) {
            long at = HEADER_LENGTH + index * ENTRY_LENGTH;
            if (at + ENTRY_LENGTH > total) {
                throw new BlobFormatException("the table of contents runs past the end of the blob");
            }
            int id = bytes.getInt((int) at);
            long offset = Integer.toUnsignedLong(bytes.getInt((int) at + 4));
            long length = Integer.toUnsignedLong(bytes.getInt((int) at + 8));
            long rows = Integer.toUnsignedLong(bytes.getInt((int) at + 12));
            if (offset + length > total || rows > Integer.MAX_VALUE) {
                throw new BlobFormatException("section " + id + " runs past the end of the blob");
            }
            sections.put(id, new Section((int) offset, (int) length, (int) rows));
        }
    }

    ByteBuffer bytes() {
        return bytes;
    }

    /** The section with an id, or a refusal naming what is missing. */
    Section section(int id, String name) {
        Section found = sections.get(id);
        if (found == null) {
            throw new BlobFormatException("the blob carries no `" + name + "` section");
        }
        return found;
    }

    /** A fixed section, checked to hold a slot for each of its fields. */
    Section fixed(int id, String name, int slots) {
        Section at = section(id, name);
        if ((long) slots * 8 > at.length()) {
            throw new BlobFormatException("the `" + name + "` section holds " + at.length() + " bytes, not " + slots + " slots");
        }
        return at;
    }

    /** The byte offset of one column of a column section, checked to hold every row. */
    int column(Section at, int index, int width) {
        if ((long) (index + 1) * 4 > at.length()) {
            throw new BlobFormatException("a column's offset runs past the end of its section");
        }
        long offset = at.offset() + Integer.toUnsignedLong(bytes.getInt(at.offset() + index * 4));
        if (offset + (long) at.count() * width > (long) at.offset() + at.length()) {
            throw new BlobFormatException("a column runs past the end of its section");
        }
        return (int) offset;
    }

    /** A bytes section as the UTF-8 text the library wrote. */
    String text(int id, String name) {
        Section at = section(id, name);
        if (at.count() > at.length()) {
            throw new BlobFormatException("the `" + name + "` text runs past the end of its section");
        }
        byte[] text = new byte[at.count()];
        bytes.get(at.offset(), text);
        return new String(text, StandardCharsets.UTF_8);
    }
}
"#;
