//! The value types: a checked record per branded quantity, a record per
//! boundary struct a binding shows, and `Values`, the conversions they
//! share. Each record writes itself into the C struct a call takes
//! (`into`) and reads itself out of one the library filled (`of`), so a
//! caller never sees a handshake, a pad, a count or a presence flag.
//!
//! Java has no unsigned integers, so an unsigned field is carried one size
//! wider (`u8` and `u16` as `int`, `u32` as `long`) and checked against
//! its C range on the way in, naming the field; a `u64` stays a `long`,
//! documented as unsigned.

use std::fmt::Write;

use super::{File, PACKAGE, identifier, javadoc, literal, preamble, unplaced};
use crate::emit::{DocStyle, field_doc};
use crate::model::{Api, FieldDef, Scalar, StructDef, StructRole, TypeRef};
use crate::names::{binding_type_name, pascal, screaming};
use crate::rules::{FieldRole, field_roles, has_handshake};

/// The structs the hand-written layer owns: the context's options, whose
/// flags it spells as booleans, and the error record, which is the
/// exception. No record is generated for either.
const HAND_WRITTEN: [&str; 2] = ["TsContextOptions", "TsError"];

/// Whether a struct is shown as a record: the rule every emitter uses (a
/// plain object or a set of caller-allocated columns, with a field of its
/// own and no array of structs), less what the hand-written layer owns.
pub(super) fn shown(api: &Api, s: &StructDef) -> bool {
    matches!(s.role, StructRole::Object | StructRole::Columns)
        && !HAND_WRITTEN.contains(&s.name.as_str())
        && s.fields
            .iter()
            .any(|f| f.name != "struct_size" && !f.name.starts_with("reserved"))
        && !s.fields.iter().zip(field_roles(api, s)).any(|(f, role)| {
            matches!(role, FieldRole::Array { .. })
                && matches!(f.ty.pointee(), Some(TypeRef::Struct { .. }))
        })
}

/// Every value file: `Values`, a record per brand, a record per shown
/// struct.
pub(super) fn files(api: &Api, dir: &str) -> Vec<File> {
    let mut files = vec![File {
        path: format!("{dir}/Values.java"),
        text: render_values(api),
    }];
    for (brand, field) in crate::emit::ts::brands(api) {
        files.push(File {
            path: format!("{dir}/{}.java", pascal(&brand)),
            text: render_brand(api, &brand, field),
        });
    }
    for s in api.structs.iter().filter(|s| shown(api, s)) {
        files.push(File {
            path: format!("{dir}/{}.java", binding_type_name(&s.name)),
            text: render_record(api, s),
        });
    }
    files
}

// ── Types ──────────────────────────────────────────────────────────────────

/// The Java type a scalar is carried in: one size wider when it is
/// unsigned, so every C value has a Java one.
const fn carrier(scalar: Scalar) -> &'static str {
    match scalar {
        Scalar::U8 | Scalar::U16 | Scalar::I8 | Scalar::I16 | Scalar::I32 => "int",
        Scalar::U32 | Scalar::U64 | Scalar::I64 | Scalar::Usize | Scalar::Isize => "long",
        Scalar::F32 => "float",
        Scalar::F64 => "double",
        Scalar::Bool => "boolean",
    }
}

/// A carrier's boxed type, for a value that may be absent.
const fn boxed(carrier: &str) -> &str {
    match carrier.as_bytes() {
        b"int" => "Integer",
        b"long" => "Long",
        b"float" => "Float",
        b"double" => "Double",
        b"boolean" => "Boolean",
        _ => "Object",
    }
}

/// The raw layout's Java type for a scalar, which a `VarHandle` reads and
/// writes exactly.
const fn raw(scalar: Scalar) -> &'static str {
    match scalar {
        Scalar::U8 | Scalar::I8 => "byte",
        Scalar::U16 | Scalar::I16 => "short",
        Scalar::U32 | Scalar::I32 => "int",
        Scalar::U64 | Scalar::I64 | Scalar::Usize | Scalar::Isize => "long",
        Scalar::F32 => "float",
        Scalar::F64 => "double",
        Scalar::Bool => "boolean",
    }
}

/// The scalar a field is stored as.
fn stored(api: &Api, field: &FieldDef) -> Scalar {
    match &field.ty {
        TypeRef::Scalar { scalar } => *scalar,
        TypeRef::Enum { name } => api.enum_named(name).map_or(Scalar::I32, |e| e.repr),
        _ => Scalar::U8,
    }
}

/// The element scalar of an array field.
fn element(field: &FieldDef) -> Scalar {
    field
        .ty
        .pointee()
        .and_then(TypeRef::as_scalar)
        .unwrap_or(Scalar::U8)
}

/// The enum a field stands for, by its binding name.
fn enum_of(field: &FieldDef) -> Option<String> {
    field.meta.enum_name.as_deref().map(binding_type_name).or_else(|| match &field.ty {
        TypeRef::Enum { name } => Some(binding_type_name(name)),
        _ => None,
    })
}

/// Whether a field may hold no value.
fn absent(field: &FieldDef, role: &FieldRole) -> bool {
    field.meta.nullable || matches!(role, FieldRole::Optional { .. })
}

/// A field's Java type, from its role.
fn java_type(api: &Api, field: &FieldDef, role: &FieldRole) -> String {
    match role {
        FieldRole::Flag => String::from("boolean"),
        FieldRole::BitSet { enum_name } => format!("Set<{}>", binding_type_name(enum_name)),
        FieldRole::Array { .. } | FieldRole::Column => match enum_of(field) {
            Some(name) => format!("List<{name}>"),
            None => format!("{}[]", carrier(element(field))),
        },
        FieldRole::FixedBytes { .. } => String::from("byte[]"),
        FieldRole::Text => String::from("String"),
        FieldRole::Nested { name } => binding_type_name(name),
        FieldRole::Optional { .. } => match &field.ty {
            TypeRef::Struct { name } => binding_type_name(name),
            _ => boxed(&plain(api, field)).to_string(),
        },
        _ => {
            let plain = plain(api, field);
            if field.meta.nullable && !plain.chars().next().is_some_and(char::is_uppercase) {
                boxed(&plain).to_string()
            } else {
                plain
            }
        }
    }
}

/// A plain value's type: its brand, its enum, or its carrier.
fn plain(api: &Api, field: &FieldDef) -> String {
    if let Some(brand) = &field.meta.brand {
        return pascal(brand);
    }
    if let Some(name) = enum_of(field) {
        return name;
    }
    carrier(stored(api, field)).to_string()
}

/// Whether a type is a reference, which a record must hold non-null unless
/// the field may be absent.
fn is_reference(ty: &str) -> bool {
    ty.chars().next().is_some_and(char::is_uppercase) || ty.ends_with("[]")
}

// ── Values ─────────────────────────────────────────────────────────────────

/// `Values`: the checks and copies every record shares.
fn render_values(api: &Api) -> String {
    let mut out = preamble(api, PACKAGE);
    out.push_str(VALUES);
    out
}

const VALUES: &str = r#"import static java.lang.foreign.ValueLayout.JAVA_DOUBLE;
import static java.lang.foreign.ValueLayout.JAVA_INT;
import static java.lang.foreign.ValueLayout.JAVA_SHORT;

import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Collections;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Set;
import java.util.function.IntFunction;

import com.teispace.teistro.ffi.Native;

/**
 * The conversions every record shares: an unsigned field checked against
 * its C range on the way in, naming the field, and the copies an array or a
 * string makes across the boundary.
 */
final class Values {
    private Values() {}

    private static IllegalArgumentException outside(String field, long value, String range) {
        return new IllegalArgumentException("`" + field + "` is " + range + ", and " + value + " is not");
    }

    static byte u8(String field, long value) {
        if (value < 0 || value > 0xFF) {
            throw outside(field, value, "0 to 255");
        }
        return (byte) value;
    }

    static short u16(String field, long value) {
        if (value < 0 || value > 0xFFFF) {
            throw outside(field, value, "0 to 65535");
        }
        return (short) value;
    }

    static int u32(String field, long value) {
        if (value < 0 || value > 0xFFFF_FFFFL) {
            throw outside(field, value, "0 to 4294967295");
        }
        return (int) value;
    }

    static byte i8(String field, long value) {
        if (value < Byte.MIN_VALUE || value > Byte.MAX_VALUE) {
            throw outside(field, value, "-128 to 127");
        }
        return (byte) value;
    }

    static short i16(String field, long value) {
        if (value < Short.MIN_VALUE || value > Short.MAX_VALUE) {
            throw outside(field, value, "-32768 to 32767");
        }
        return (short) value;
    }

    /** A member's id, refusing the one this build does not know, which no call can carry. */
    static int id(String field, Member member) {
        if (member.id() < 0) {
            throw new IllegalArgumentException("`" + field + "` cannot carry " + member.key()
                    + ": a member this build does not know has no id to send");
        }
        return member.id();
    }

    /** A set of members as the bits of their ids. */
    static long bits(String field, Set<? extends Member> members) {
        long bits = 0;
        for (Member member : members) {
            int id = id(field, member);
            if (id >= 32) {
                throw new IllegalArgumentException("`" + field + "` holds ids below 32, and "
                        + member.key() + " is " + id);
            }
            bits |= 1L << id;
        }
        return bits;
    }

    /** The members whose ids are the set bits. */
    static <E> Set<E> members(long bits, IntFunction<E> of) {
        Set<E> members = new LinkedHashSet<>();
        for (int bit = 0; bit < 32; bit += 1) {
            if ((bits >>> bit & 1L) != 0) {
                members.add(of.apply(bit));
            }
        }
        return Collections.unmodifiableSet(members);
    }

    /** A string as the library takes one: NUL-terminated UTF-8, or NULL for null. */
    static MemorySegment cString(Arena arena, String text) {
        return text == null ? MemorySegment.NULL : arena.allocateFrom(text, StandardCharsets.UTF_8);
    }

    /** A string the library wrote, or null (or an empty string, for a field that is never absent). */
    @SuppressWarnings("restricted")
    static String text(MemorySegment pointer, boolean nullable) {
        if (pointer.equals(MemorySegment.NULL)) {
            return nullable ? null : "";
        }
        return pointer.reinterpret(Long.MAX_VALUE).getString(0, StandardCharsets.UTF_8);
    }

    static MemorySegment doubles(Arena arena, double[] values) {
        return arena.allocateFrom(JAVA_DOUBLE, values);
    }

    static MemorySegment i32s(Arena arena, int[] values) {
        return arena.allocateFrom(JAVA_INT, values);
    }

    static MemorySegment u32s(Arena arena, String field, long[] values) {
        MemorySegment written = arena.allocate(JAVA_INT, Math.max(values.length, 1));
        for (int i = 0; i < values.length; i += 1) {
            written.setAtIndex(JAVA_INT, i, u32(field, values[i]));
        }
        return written;
    }

    static MemorySegment u16s(Arena arena, String field, int[] values) {
        MemorySegment written = arena.allocate(JAVA_SHORT, Math.max(values.length, 1));
        for (int i = 0; i < values.length; i += 1) {
            written.setAtIndex(JAVA_SHORT, i, u16(field, values[i]));
        }
        return written;
    }

    static MemorySegment u16Members(Arena arena, String field, List<? extends Member> members) {
        MemorySegment written = arena.allocate(JAVA_SHORT, Math.max(members.size(), 1));
        for (int i = 0; i < members.size(); i += 1) {
            written.setAtIndex(JAVA_SHORT, i, u16(field, id(field, members.get(i))));
        }
        return written;
    }

    @SuppressWarnings("restricted")
    private static MemorySegment sized(MemorySegment pointer, long count, long width) {
        return pointer.reinterpret(count * width);
    }

    static double[] doubles(MemorySegment pointer, long count) {
        if (count == 0 || pointer.equals(MemorySegment.NULL)) {
            return new double[0];
        }
        return sized(pointer, count, 8).toArray(JAVA_DOUBLE);
    }

    static int[] i32s(MemorySegment pointer, long count) {
        if (count == 0 || pointer.equals(MemorySegment.NULL)) {
            return new int[0];
        }
        return sized(pointer, count, 4).toArray(JAVA_INT);
    }

    static long[] u32s(MemorySegment pointer, long count) {
        long[] read = new long[(int) count];
        if (count == 0 || pointer.equals(MemorySegment.NULL)) {
            return new long[0];
        }
        MemorySegment from = sized(pointer, count, 4);
        for (int i = 0; i < read.length; i += 1) {
            read[i] = Integer.toUnsignedLong(from.getAtIndex(JAVA_INT, i));
        }
        return read;
    }

    static int[] u16s(MemorySegment pointer, long count) {
        int[] read = new int[(int) count];
        if (count == 0 || pointer.equals(MemorySegment.NULL)) {
            return new int[0];
        }
        MemorySegment from = sized(pointer, count, 2);
        for (int i = 0; i < read.length; i += 1) {
            read[i] = Short.toUnsignedInt(from.getAtIndex(JAVA_SHORT, i));
        }
        return read;
    }

    static <E> List<E> u16Members(MemorySegment pointer, long count, IntFunction<E> of) {
        List<E> read = new ArrayList<>();
        for (int id : u16s(pointer, count)) {
            read.add(of.apply(id));
        }
        return Collections.unmodifiableList(read);
    }

    /** A member, or null for the id a nullable catalogue field holds for none. */
    static <E> E memberOrNull(int id, IntFunction<E> of) {
        return id == NO_MEMBER ? null : of.apply(id);
    }

    /** What a nullable catalogue field holds for none. */
    static final int NO_MEMBER = Native.NO_MEMBER;
}
"#;

// ── Brands ─────────────────────────────────────────────────────────────────

/// A branded quantity: its own type, so a latitude cannot be passed where a
/// longitude is wanted, and checked against the range the description
/// states (ADR-0023).
fn render_brand(api: &Api, brand: &str, field: &FieldDef) -> String {
    let name = pascal(brand);
    let unit = field.meta.unit.as_deref().unwrap_or("");
    let range = field.meta.range.as_deref().unwrap_or("");
    let mut out = preamble(api, PACKAGE);
    let doc = format!(
        "A {brand}{}{}.\n\nIts own type, so it cannot be passed where another quantity is wanted.\n\n@param value the {brand}{}",
        if unit.is_empty() { String::new() } else { format!(" in {unit}") },
        if range.is_empty() { String::new() } else { format!(", {range}") },
        if unit.is_empty() { String::new() } else { format!(" in {unit}") },
    );
    // The `@param` tag must reach Javadoc as a tag.
    out.push_str(&javadoc(&doc, "").replace("&#64;param", "@param"));
    let _ = writeln!(out, "public record {name}(double value) {{");
    if let Some((low, high)) = crate::emit::ts::brand_range(field) {
        let _ = writeln!(
            out,
            "    /**\n     * A {brand}, checked.\n     *\n     * @throws IllegalArgumentException for a value outside {range}, or not a number\n     */\n    public {name} {{\n        if (!({low} <= value && value <= {high})) {{\n            throw new IllegalArgumentException({} + value + {});\n        }}\n    }}",
            literal(&format!("a {brand} is {range}, and ")),
            literal(" is not")
        );
    }
    out.push_str("}\n");
    out
}

// ── Records ────────────────────────────────────────────────────────────────

/// One record: its components, a compact constructor that copies what is
/// mutable and refuses a missing value, and `into` and `of`.
fn render_record(api: &Api, s: &StructDef) -> String {
    let name = binding_type_name(&s.name);
    let roles = field_roles(api, s);
    let shown: Vec<(&FieldDef, &FieldRole)> = s
        .fields
        .iter()
        .zip(&roles)
        .filter(|(_, role)| role.is_shown())
        .collect();
    let mut out = preamble(api, PACKAGE);
    out.push_str("import java.lang.foreign.Arena;\nimport java.lang.foreign.MemorySegment;\nimport java.util.List;\nimport java.util.Objects;\nimport java.util.Set;\n\nimport com.teispace.teistro.ffi.Native;\n\n");
    let mut doc = s.doc.clone();
    doc.push('\n');
    for (f, _) in &shown {
        let _ = write!(
            doc,
            "\n@param {} {}",
            identifier(&f.name),
            field_doc(f, DocStyle::Prose).replace('\n', " ")
        );
    }
    out.push_str(&javadoc(&doc, "").replace("&#64;param", "@param"));
    let components: Vec<String> = shown
        .iter()
        .map(|(f, role)| format!("{} {}", java_type(api, f, role), identifier(&f.name)))
        .collect();
    let _ = writeln!(
        out,
        "@SuppressWarnings(\"unused\")\npublic record {name}(\n        {}) {{",
        components.join(",\n        ")
    );
    render_compact(&mut out, api, &name, &shown);
    render_into(&mut out, api, s, &roles);
    render_of(&mut out, api, s, &roles, &name);
    out.push_str("}\n");
    out
}

/// The compact constructor: a missing value refused by its name, an array
/// copied in, a list or a set made unmodifiable.
fn render_compact(out: &mut String, api: &Api, name: &str, shown: &[(&FieldDef, &FieldRole)]) {
    let mut body = String::new();
    for (f, role) in shown {
        let field = identifier(&f.name);
        let ty = java_type(api, f, role);
        if !absent(f, role) && is_reference(&ty) {
            let _ = writeln!(body, "        Objects.requireNonNull({field}, \"{field}\");");
        }
        if ty.ends_with("[]") {
            let _ = writeln!(body, "        {field} = {field}.clone();");
        } else if ty.starts_with("List<") {
            let _ = writeln!(body, "        {field} = List.copyOf({field});");
        } else if ty.starts_with("Set<") {
            let _ = writeln!(body, "        {field} = Set.copyOf({field});");
        }
    }
    let _ = writeln!(
        out,
        "    /** The value, checked: what may not be absent is present, and what is mutable is copied. */\n    public {name} {{\n{body}    }}\n"
    );
    // An array component is handed out as a copy, so the record stays a
    // value however its caller uses what it gets.
    for (f, role) in shown {
        let ty = java_type(api, f, role);
        if ty.ends_with("[]") {
            let field = identifier(&f.name);
            let _ = writeln!(
                out,
                "    /**\n     * A copy of the {}.\n     *\n     * @return the {}\n     */\n    @Override\n    public {ty} {field}() {{\n        return {field}.clone();\n    }}\n",
                f.name, f.name
            );
        }
    }
}

/// The handle of a field in the raw layer.
fn handle(s: &StructDef, field: &FieldDef) -> String {
    format!("Native.{}.{}", s.name, screaming(&field.name))
}

/// What writes a plain value into its field.
fn write_plain(api: &Api, s: &StructDef, f: &FieldDef, value: &str) -> String {
    let h = handle(s, f);
    let label = literal(&identifier(&f.name));
    let scalar = stored(api, f);
    let carried = if f.meta.brand.is_some() {
        format!("{value}.value()")
    } else if enum_of(f).is_some() {
        if f.meta.nullable {
            format!("({value} == null ? Values.NO_MEMBER : Values.id({label}, {value}))")
        } else {
            format!("Values.id({label}, {value})")
        }
    } else {
        value.to_string()
    };
    let raw_value = match scalar {
        Scalar::U8 => format!("Values.u8({label}, {carried})"),
        Scalar::U16 => format!("Values.u16({label}, {carried})"),
        Scalar::U32 => format!("Values.u32({label}, {carried})"),
        Scalar::I8 => format!("Values.i8({label}, {carried})"),
        Scalar::I16 => format!("Values.i16({label}, {carried})"),
        _ => {
            let from = if f.meta.brand.is_some() {
                "double"
            } else if enum_of(f).is_some() {
                "int"
            } else {
                carrier(scalar)
            };
            if from == raw(scalar) {
                carried
            } else {
                format!("({}) {carried}", raw(scalar))
            }
        }
    };
    format!("{h}.set(raw, 0L, {raw_value});")
}

/// `into`: the value written into the C struct a call takes, which may be
/// one held inside another. What it points at is allocated in `arena`.
fn render_into(out: &mut String, api: &Api, s: &StructDef, roles: &[FieldRole]) {
    let mut body = String::new();
    if has_handshake(s) {
        let _ = writeln!(
            body,
            "        Native.{0}.STRUCT_SIZE.set(raw, 0L, (int) Native.{0}.SIZE);",
            s.name
        );
    }
    for (f, role) in s.fields.iter().zip(roles) {
        let field = identifier(&f.name);
        let h = handle(s, f);
        let label = literal(&identifier(&f.name));
        let line = match role {
            FieldRole::Handshake | FieldRole::Reserved => continue,
            FieldRole::Flag => format!("{h}.set(raw, 0L, ({}) ({field} ? 1 : 0));", raw(stored(api, f))),
            FieldRole::BitSet { .. } => format!(
                "{h}.set(raw, 0L, Values.u32({label}, Values.bits({label}, {field})));"
            ),
            FieldRole::Count { of } => {
                let size = match s.fields.iter().find(|x| &x.name == of) {
                    Some(array) if enum_of(array).is_some() => format!("{}.size()", identifier(of)),
                    _ => format!("{}.length", identifier(of)),
                };
                format!("{h}.set(raw, 0L, ({}) {size});", raw(stored(api, f)))
            }
            FieldRole::Presence { of } => {
                format!("{h}.set(raw, 0L, ({}) ({} == null ? 0 : 1));", raw(stored(api, f)), identifier(of))
            }
            FieldRole::Array { .. } | FieldRole::Column => {
                let written = match (enum_of(f), element(f)) {
                    (Some(_), Scalar::U16) => format!("Values.u16Members(arena, {label}, {field})"),
                    (None, Scalar::F64) => format!("Values.doubles(arena, {field})"),
                    (None, Scalar::I32) => format!("Values.i32s(arena, {field})"),
                    (None, Scalar::U32) => format!("Values.u32s(arena, {label}, {field})"),
                    (None, Scalar::U16) => format!("Values.u16s(arena, {label}, {field})"),
                    _ => unplaced(&format!("{}.{}", s.name, f.name), &crate::layout::LayoutError::Unknown(String::from("an array of this element"))),
                };
                format!("{h}.set(raw, 0L, {written});")
            }
            FieldRole::FixedBytes { len } => format!(
                "if ({field}.length != {len}) {{\n            throw new IllegalArgumentException({} + {field}.length);\n        }}\n        MemorySegment.copy(MemorySegment.ofArray({field}), 0L, raw, Native.{}.{}_OFFSET, {len});",
                literal(&format!("`{}` is {len} bytes, not ", f.name)),
                s.name,
                screaming(&f.name)
            ),
            FieldRole::Text => format!("{h}.set(raw, 0L, Values.cString(arena, {field}));"),
            FieldRole::Nested { name } => format!(
                "{field}.into(raw.asSlice(Native.{}.{}_OFFSET, Native.{name}.SIZE), arena);",
                s.name,
                screaming(&f.name)
            ),
            FieldRole::Optional { .. } => match &f.ty {
                TypeRef::Struct { name } => format!(
                    "if ({field} != null) {{\n            {field}.into(raw.asSlice(Native.{}.{}_OFFSET, Native.{name}.SIZE), arena);\n        }}",
                    s.name,
                    screaming(&f.name)
                ),
                _ => format!(
                    "if ({field} != null) {{\n            {}\n        }}",
                    write_plain(api, s, f, &field)
                ),
            },
            FieldRole::Value => write_plain(api, s, f, &field),
        };
        let _ = writeln!(body, "        {line}");
    }
    let _ = writeln!(
        out,
        "    /** Writes this value into the C struct at {{@code raw}}; what it points at is allocated in {{@code arena}}. */\n    void into(MemorySegment raw, Arena arena) {{\n{body}    }}\n\n    /** This value as a fresh C struct in {{@code arena}}, ready to be passed by pointer. */\n    MemorySegment toC(Arena arena) {{\n        MemorySegment raw = arena.allocate(Native.{}.LAYOUT);\n        into(raw, arena);\n        return raw;\n    }}\n",
        s.name
    );
}

/// A field's stored scalar read out and carried as its Java type: an
/// unsigned one widened without its sign.
fn read_carried(api: &Api, s: &StructDef, f: &FieldDef) -> String {
    let scalar = stored(api, f);
    let got = format!("(({}) {}.get(raw, 0L))", raw(scalar), handle(s, f));
    match scalar {
        Scalar::U8 => format!("Byte.toUnsignedInt{got}"),
        Scalar::U16 => format!("Short.toUnsignedInt{got}"),
        Scalar::U32 => format!("Integer.toUnsignedLong{got}"),
        Scalar::I8 | Scalar::I16 => format!("(int) {got}"),
        _ => got,
    }
}

/// What reads a plain value out of its field.
fn read_plain(api: &Api, s: &StructDef, f: &FieldDef) -> String {
    let scalar = stored(api, f);
    let widened = read_carried(api, s, f);
    if let Some(brand) = &f.meta.brand {
        return format!("new {}({widened})", pascal(brand));
    }
    if let Some(name) = enum_of(f) {
        let id = if matches!(scalar, Scalar::U32) {
            format!("(int) {widened}")
        } else {
            widened
        };
        if f.meta.nullable {
            return format!("Values.memberOrNull({id}, {name}::of)");
        }
        return format!("{name}.of({id})");
    }
    widened
}

/// `of`: the value read out of a C struct the library filled.
fn render_of(out: &mut String, api: &Api, s: &StructDef, roles: &[FieldRole], name: &str) {
    let mut reads: Vec<String> = Vec::new();
    for (f, role) in s.fields.iter().zip(roles).filter(|(_, r)| r.is_shown()) {
        let h = handle(s, f);
        let read = match role {
            FieldRole::Flag => format!("(({}) {h}.get(raw, 0L)) != 0", raw(stored(api, f))),
            FieldRole::BitSet { enum_name } => format!(
                "Values.members(Integer.toUnsignedLong((int) {h}.get(raw, 0L)), {}::of)",
                binding_type_name(enum_name)
            ),
            FieldRole::Array { count } => {
                let count_field = s.fields.iter().find(|x| &x.name == count);
                let n = count_field.map_or_else(
                    || String::from("0L"),
                    |c| read_carried(api, s, c),
                );
                let pointer = format!("(MemorySegment) {h}.get(raw, 0L)");
                match (enum_of(f), element(f)) {
                    (Some(e), Scalar::U16) => format!("Values.u16Members({pointer}, {n}, {e}::of)"),
                    (None, Scalar::F64) => format!("Values.doubles({pointer}, {n})"),
                    (None, Scalar::I32) => format!("Values.i32s({pointer}, {n})"),
                    (None, Scalar::U32) => format!("Values.u32s({pointer}, {n})"),
                    (None, Scalar::U16) => format!("Values.u16s({pointer}, {n})"),
                    _ => unplaced(&format!("{}.{}", s.name, f.name), &crate::layout::LayoutError::Unknown(String::from("an array of this element"))),
                }
            }
            // A column is the caller's to allocate and the library's to
            // fill; read back on its own, it holds nothing yet.
            FieldRole::Column => match enum_of(f) {
                Some(_) => String::from("List.of()"),
                None => format!("new {}[0]", carrier(element(f))),
            },
            FieldRole::FixedBytes { len } => format!(
                "raw.asSlice(Native.{}.{}_OFFSET, {len}).toArray(java.lang.foreign.ValueLayout.JAVA_BYTE)",
                s.name,
                screaming(&f.name)
            ),
            FieldRole::Text => format!(
                "Values.text((MemorySegment) {h}.get(raw, 0L), {})",
                f.meta.nullable
            ),
            FieldRole::Nested { name } => format!(
                "{}.of(raw.asSlice(Native.{}.{}_OFFSET, Native.{name}.SIZE))",
                binding_type_name(name),
                s.name,
                screaming(&f.name)
            ),
            FieldRole::Optional { flag } => {
                let present = s
                    .fields
                    .iter()
                    .find(|x| &x.name == flag)
                    .map_or_else(
                        || String::from("false"),
                        |x| format!("(({}) {}.get(raw, 0L)) != 0", raw(stored(api, x)), handle(s, x)),
                    );
                let value = match &f.ty {
                    TypeRef::Struct { name } => format!(
                        "{}.of(raw.asSlice(Native.{}.{}_OFFSET, Native.{name}.SIZE))",
                        binding_type_name(name),
                        s.name,
                        screaming(&f.name)
                    ),
                    _ => read_plain(api, s, f),
                };
                format!("{present} ? {value} : null")
            }
            _ => read_plain(api, s, f),
        };
        reads.push(read);
    }
    let _ = writeln!(
        out,
        "    /** The value the library wrote into the C struct at {{@code raw}}. */\n    static {name} of(MemorySegment raw) {{\n        return new {name}(\n            {});\n    }}\n",
        reads.join(",\n            ")
    );
}
