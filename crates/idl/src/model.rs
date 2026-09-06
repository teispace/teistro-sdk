//! The API description: what the extractor produces from the boundary
//! crates and what every emitter reads. Serialised as `idl/api.json` so
//! the file ships inside every package, is diffed in review and is gated
//! by `cargo xtask check-ffi`.

use serde::{Deserialize, Serialize};

/// The schema of the description file.
pub const SCHEMA: &str = "teistro-api/1";

/// The whole description of the C ABI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Api {
    /// [`SCHEMA`].
    pub schema: String,
    /// The ABI version the library's `ts_abi_version` returns.
    pub abi_version: u32,
    /// The SDK version the description was extracted from.
    pub sdk_version: String,
    /// The symbol prefix (`ts_`).
    pub prefix: String,
    /// The repository-relative sources the items came from.
    pub sources: Vec<String>,
    /// Exported integer constants (flags, versions).
    pub constants: Vec<ConstantDef>,
    /// C-like enums with explicit discriminants, the catalogue's kinds
    /// among them.
    pub enums: Vec<EnumDef>,
    /// Opaque handle types, only ever behind a pointer.
    pub opaques: Vec<OpaqueDef>,
    /// Function-pointer types.
    pub callbacks: Vec<CallbackDef>,
    /// `#[repr(C)]` structs.
    pub structs: Vec<StructDef>,
    /// Exported functions.
    pub functions: Vec<FunctionDef>,
    /// The result blob schemas, one per blob-returning entry point.
    pub blobs: Vec<BlobSchema>,
}

impl Api {
    /// The enum with a Rust name.
    #[must_use]
    pub fn enum_named(&self, name: &str) -> Option<&EnumDef> {
        self.enums.iter().find(|e| e.name == name)
    }

    /// The struct with a Rust name.
    #[must_use]
    pub fn struct_named(&self, name: &str) -> Option<&StructDef> {
        self.structs.iter().find(|s| s.name == name)
    }

    /// The opaque type with a Rust name.
    #[must_use]
    pub fn opaque_named(&self, name: &str) -> Option<&OpaqueDef> {
        self.opaques.iter().find(|o| o.name == name)
    }

    /// The callback type with a Rust name.
    #[must_use]
    pub fn callback_named(&self, name: &str) -> Option<&CallbackDef> {
        self.callbacks.iter().find(|c| c.name == name)
    }

    /// The blob schema with a name.
    #[must_use]
    pub fn blob_named(&self, name: &str) -> Option<&BlobSchema> {
        self.blobs.iter().find(|b| b.name == name)
    }

    /// The function with a symbol name.
    #[must_use]
    pub fn function_named(&self, name: &str) -> Option<&FunctionDef> {
        self.functions.iter().find(|f| f.name == name)
    }
}

/// A machine scalar at the boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scalar {
    /// `u8`.
    U8,
    /// `u16`.
    U16,
    /// `u32`.
    U32,
    /// `u64`.
    U64,
    /// `i8`.
    I8,
    /// `i16`.
    I16,
    /// `i32`.
    I32,
    /// `i64`.
    I64,
    /// `f32`.
    F32,
    /// `f64`.
    F64,
    /// `usize`, the target's pointer width.
    Usize,
    /// `isize`, the target's pointer width.
    Isize,
    /// `bool`, one byte.
    Bool,
}

impl Scalar {
    /// Every scalar.
    pub const ALL: [Scalar; 13] = [
        Scalar::U8,
        Scalar::U16,
        Scalar::U32,
        Scalar::U64,
        Scalar::I8,
        Scalar::I16,
        Scalar::I32,
        Scalar::I64,
        Scalar::F32,
        Scalar::F64,
        Scalar::Usize,
        Scalar::Isize,
        Scalar::Bool,
    ];

    /// The scalar with a Rust spelling.
    #[must_use]
    pub fn from_rust(name: &str) -> Option<Scalar> {
        Scalar::ALL.iter().copied().find(|s| s.rust_name() == name)
    }

    /// The Rust spelling.
    #[must_use]
    pub const fn rust_name(self) -> &'static str {
        match self {
            Scalar::U8 => "u8",
            Scalar::U16 => "u16",
            Scalar::U32 => "u32",
            Scalar::U64 => "u64",
            Scalar::I8 => "i8",
            Scalar::I16 => "i16",
            Scalar::I32 => "i32",
            Scalar::I64 => "i64",
            Scalar::F32 => "f32",
            Scalar::F64 => "f64",
            Scalar::Usize => "usize",
            Scalar::Isize => "isize",
            Scalar::Bool => "bool",
        }
    }

    /// The C spelling (`<stdint.h>`, `<stddef.h>`, `<stdbool.h>`).
    #[must_use]
    pub const fn c_name(self) -> &'static str {
        match self {
            Scalar::U8 => "uint8_t",
            Scalar::U16 => "uint16_t",
            Scalar::U32 => "uint32_t",
            Scalar::U64 => "uint64_t",
            Scalar::I8 => "int8_t",
            Scalar::I16 => "int16_t",
            Scalar::I32 => "int32_t",
            Scalar::I64 => "int64_t",
            Scalar::F32 => "float",
            Scalar::F64 => "double",
            Scalar::Usize => "size_t",
            Scalar::Isize => "ptrdiff_t",
            Scalar::Bool => "bool",
        }
    }

    /// The width in bytes on a target; `None` for the pointer-sized ones
    /// when no target is given.
    #[must_use]
    pub const fn width(self, pointer_width: usize) -> usize {
        match self {
            Scalar::U8 | Scalar::I8 | Scalar::Bool => 1,
            Scalar::U16 | Scalar::I16 => 2,
            Scalar::U32 | Scalar::I32 | Scalar::F32 => 4,
            Scalar::U64 | Scalar::I64 | Scalar::F64 => 8,
            Scalar::Usize | Scalar::Isize => pointer_width,
        }
    }

    /// Whether the scalar is a floating-point number.
    #[must_use]
    pub const fn is_float(self) -> bool {
        matches!(self, Scalar::F32 | Scalar::F64)
    }

    /// Whether the scalar is a signed integer.
    #[must_use]
    pub const fn is_signed(self) -> bool {
        matches!(
            self,
            Scalar::I8 | Scalar::I16 | Scalar::I32 | Scalar::I64 | Scalar::Isize
        )
    }
}

/// A type as it appears at the boundary.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TypeRef {
    /// A machine scalar.
    Scalar {
        /// Which one.
        scalar: Scalar,
    },
    /// A named enum, passed as its `repr` integer.
    Enum {
        /// The Rust name.
        name: String,
    },
    /// A named `#[repr(C)]` struct.
    Struct {
        /// The Rust name.
        name: String,
    },
    /// An opaque handle type.
    Opaque {
        /// The Rust name.
        name: String,
    },
    /// A nullable function pointer.
    Callback {
        /// The callback type's Rust name.
        name: String,
    },
    /// A pointer to something.
    Pointer {
        /// What it points at.
        to: Box<TypeRef>,
        /// `*mut` rather than `*const`.
        mutable: bool,
    },
    /// A fixed-size array inside a struct.
    Array {
        /// The element type.
        of: Box<TypeRef>,
        /// The element count.
        len: usize,
    },
    /// `c_void`, only ever behind a pointer.
    Void,
    /// `c_char`, only ever behind a pointer.
    Char,
}

impl TypeRef {
    /// A scalar.
    #[must_use]
    pub const fn scalar(scalar: Scalar) -> TypeRef {
        TypeRef::Scalar { scalar }
    }

    /// A pointer to a type.
    #[must_use]
    pub fn pointer(to: TypeRef, mutable: bool) -> TypeRef {
        TypeRef::Pointer {
            to: Box::new(to),
            mutable,
        }
    }

    /// What a pointer points at, through one level.
    #[must_use]
    pub fn pointee(&self) -> Option<&TypeRef> {
        match self {
            TypeRef::Pointer { to, .. } => Some(to),
            _ => None,
        }
    }

    /// The scalar, when the type is one.
    #[must_use]
    pub const fn as_scalar(&self) -> Option<Scalar> {
        match self {
            TypeRef::Scalar { scalar } => Some(*scalar),
            _ => None,
        }
    }

    /// The name of the enum, struct, opaque or callback the type names.
    #[must_use]
    pub fn named(&self) -> Option<&str> {
        match self {
            TypeRef::Enum { name }
            | TypeRef::Struct { name }
            | TypeRef::Opaque { name }
            | TypeRef::Callback { name } => Some(name),
            _ => None,
        }
    }
}

/// The metadata an `api:` doc line carries (ADR-0023): units, ranges and
/// examples on fields, the enum an integer stands for, nullability, and
/// on a function the blob schema it fills.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Meta {
    /// The unit, such as `deg` or `deg/day`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// The quantity the number carries, where two of the same type would
    /// otherwise be swappable: a latitude is not a longitude. A binding
    /// that can express a distinct type for it does (ADR-0023).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brand: Option<String>,
    /// The range in interval notation, such as `[1,5]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<String>,
    /// An example value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
    /// The enum the integer stands for, by Rust name.
    #[serde(default, rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_name: Option<String>,
    /// Whether a pointer may be null.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub nullable: bool,
    /// The blob schema a function fills, by name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
    /// The ABI version an item appeared in, when later than the first.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<u32>,
    /// An integer that carries a boolean (`api: flag`).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub flag: bool,
    /// An integer that carries a set of an enum's members, bit `n` for the
    /// member with value `n` (`api: bitset=Name`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bitset: Option<String>,
    /// The field holding this pointer's element count (`api: len=field`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub len: Option<String>,
    /// The flag field that says whether this one is present
    /// (`api: present_if=field`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub present_if: Option<String>,
}

/// An exported integer constant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstantDef {
    /// The Rust name.
    pub name: String,
    /// The doc comment.
    pub doc: String,
    /// The integer type.
    pub ty: Scalar,
    /// The value.
    pub value: i64,
    /// The repository-relative source.
    pub source: String,
}

/// A C-like enum.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumDef {
    /// The Rust name.
    pub name: String,
    /// The doc comment.
    pub doc: String,
    /// The `repr` integer.
    pub repr: Scalar,
    /// The catalogue kind the enum is, for the catalogue's kinds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// The members in declaration order.
    pub values: Vec<EnumValue>,
    /// The repository-relative source.
    pub source: String,
}

/// One enum member.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumValue {
    /// The Rust variant name.
    pub name: String,
    /// The discriminant.
    pub value: i64,
    /// The doc comment.
    pub doc: String,
    /// The catalogue key, for a catalogue member.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// Whether the member is deprecated.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub deprecated: bool,
}

/// What a struct is for, which decides how every binding marshals it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StructRole {
    /// A plain value object.
    #[default]
    Object,
    /// A library-owned string with a length and a capacity, freed by the
    /// library.
    OwnedString,
    /// A borrowed string view: a pointer and a length the library owns.
    BorrowedString,
    /// A library-owned result blob, freed by the library.
    Blob,
    /// A table of function pointers a host implements.
    Vtable,
    /// Caller-allocated arrays the library or a provider writes into,
    /// with a capacity beside them.
    Columns,
}

/// A `#[repr(C)]` struct.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructDef {
    /// The Rust name.
    pub name: String,
    /// The doc comment.
    pub doc: String,
    /// What the struct is for.
    #[serde(default)]
    pub role: StructRole,
    /// The fields in order.
    pub fields: Vec<FieldDef>,
    /// The repository-relative source.
    pub source: String,
}

/// One struct field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldDef {
    /// The name, unit-suffixed where it holds a quantity.
    pub name: String,
    /// The type.
    pub ty: TypeRef,
    /// The doc comment without the `api:` line.
    pub doc: String,
    /// Units, ranges, examples and links from the `api:` line.
    #[serde(default)]
    pub meta: Meta,
}

/// A function-pointer type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallbackDef {
    /// The Rust type alias name.
    pub name: String,
    /// The doc comment.
    pub doc: String,
    /// The parameters in order.
    pub params: Vec<ParamDef>,
    /// The return type.
    pub returns: TypeRef,
    /// The repository-relative source.
    pub source: String,
}

/// An opaque handle type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpaqueDef {
    /// The Rust name.
    pub name: String,
    /// The doc comment.
    pub doc: String,
    /// The repository-relative source.
    pub source: String,
}

/// An exported function.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionDef {
    /// The symbol name.
    pub name: String,
    /// The doc comment without the Safety section.
    pub doc: String,
    /// The Safety section, when the function is `unsafe`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safety: Option<String>,
    /// The parameters in order.
    pub params: Vec<ParamDef>,
    /// The return type, `None` for `void`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub returns: Option<TypeRef>,
    /// The blob schema and version metadata.
    #[serde(default)]
    pub meta: Meta,
    /// The repository-relative source.
    pub source: String,
}

/// One parameter with its inferred role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParamDef {
    /// The name.
    pub name: String,
    /// The type.
    pub ty: TypeRef,
    /// What the parameter is for.
    pub role: Role,
    /// Units, ranges, examples and enum links, from an `api:` line of the
    /// function's documentation that names this parameter
    /// (`` `api: calendar: enum=Calendar` ``).
    #[serde(default)]
    pub meta: Meta,
}

/// The role of a parameter, inferred from its type and name
/// ([`crate::rules::infer_role`]); it decides how every binding marshals
/// the parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// A scalar or enum passed by value.
    Value,
    /// An opaque handle.
    Handle,
    /// A slot that receives a new handle.
    HandleOut,
    /// A struct read by the library.
    StructIn,
    /// A struct written by the library; the caller sets `struct_size`.
    StructOut,
    /// A provider vtable, nullable.
    VtableIn,
    /// The opaque pointer handed back to callbacks.
    UserData,
    /// A blob descriptor written by the library.
    BlobOut,
    /// A blob descriptor consumed by the library.
    BlobFree,
    /// A NUL-terminated UTF-8 string read by the library.
    StringIn,
    /// An owned string written by the library.
    StringOut,
    /// An owned string consumed by the library.
    StringFree,
    /// A borrowed string view written by the library.
    StrOut,
    /// Bytes read by the library, with the following `Length` parameter.
    BytesIn,
    /// An array of scalars read by the library, with the following
    /// `Length` parameter.
    ArrayIn,
    /// The element count of the pointer parameter before it.
    Length,
    /// A scalar written by the library.
    ScalarOut,
}

/// The layout of one result blob: the sections a blob-returning entry
/// point writes, in the order they appear in the table of contents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlobSchema {
    /// The schema's name, which the function's `api: blob=` names.
    pub name: String,
    /// The schema's stable id, written in the blob header.
    pub id: u32,
    /// What the blob carries.
    pub doc: String,
    /// The sections.
    pub sections: Vec<SectionSchema>,
}

impl BlobSchema {
    /// The section with a name.
    #[must_use]
    pub fn section(&self, name: &str) -> Option<(usize, &SectionSchema)> {
        self.sections
            .iter()
            .enumerate()
            .find(|(_, s)| s.name == name)
    }
}

/// How a section's bytes are laid out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SectionKind {
    /// One record: every field in an 8-byte slot, in field order.
    Fixed,
    /// A directory of 32-bit column offsets, then one 8-aligned column per
    /// field with `count` elements each.
    Columns,
    /// Raw bytes, UTF-8 where the doc says text; `count` is the length.
    Bytes,
}

/// One section of a blob.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionSchema {
    /// The section id in the table of contents, unique within the schema.
    pub id: u32,
    /// The section name.
    pub name: String,
    /// What the section holds.
    pub doc: String,
    /// The layout.
    pub kind: SectionKind,
    /// The fields of a fixed section or the columns of a column section;
    /// empty for bytes.
    #[serde(default)]
    pub fields: Vec<ColumnDef>,
}

impl SectionSchema {
    /// The field or column with a name.
    #[must_use]
    pub fn field(&self, name: &str) -> Option<(usize, &ColumnDef)> {
        self.fields.iter().enumerate().find(|(_, f)| f.name == name)
    }
}

/// A field of a fixed section or a column of a column section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnDef {
    /// The name.
    pub name: String,
    /// The scalar type.
    pub scalar: Scalar,
    /// What the values are.
    pub doc: String,
    /// The enum the integer stands for, by Rust name.
    #[serde(default, rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_name: Option<String>,
}

impl ColumnDef {
    /// A column.
    #[must_use]
    pub fn new(name: &str, scalar: Scalar, doc: &str) -> ColumnDef {
        ColumnDef {
            name: name.to_string(),
            scalar,
            doc: doc.to_string(),
            enum_name: None,
        }
    }

    /// The same column standing for an enum.
    #[must_use]
    pub fn of_enum(mut self, enum_name: &str) -> ColumnDef {
        self.enum_name = Some(enum_name.to_string());
        self
    }
}

impl SectionSchema {
    /// A fixed section.
    #[must_use]
    pub fn fixed(id: u32, name: &str, doc: &str, fields: Vec<ColumnDef>) -> SectionSchema {
        SectionSchema {
            id,
            name: name.to_string(),
            doc: doc.to_string(),
            kind: SectionKind::Fixed,
            fields,
        }
    }

    /// A column section.
    #[must_use]
    pub fn columns(id: u32, name: &str, doc: &str, fields: Vec<ColumnDef>) -> SectionSchema {
        SectionSchema {
            id,
            name: name.to_string(),
            doc: doc.to_string(),
            kind: SectionKind::Columns,
            fields,
        }
    }

    /// A bytes section.
    #[must_use]
    pub fn bytes(id: u32, name: &str, doc: &str) -> SectionSchema {
        SectionSchema {
            id,
            name: name.to_string(),
            doc: doc.to_string(),
            kind: SectionKind::Bytes,
            fields: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    #[test]
    fn scalars_round_trip_their_spellings_and_widths() {
        for scalar in Scalar::ALL {
            assert_eq!(Scalar::from_rust(scalar.rust_name()), Some(scalar));
            assert!(scalar.width(8) >= 1 && scalar.width(8) <= 8);
        }
        assert_eq!(Scalar::Usize.width(4), 4);
        assert_eq!(Scalar::F64.c_name(), "double");
        assert!(Scalar::F32.is_float() && !Scalar::U8.is_float());
        assert!(Scalar::I16.is_signed() && !Scalar::U16.is_signed());
    }

    #[test]
    fn the_description_serialises_with_stable_tags() {
        let ty = TypeRef::pointer(
            TypeRef::Opaque {
                name: "TsContext".into(),
            },
            true,
        );
        let json = serde_json::to_string(&ty).unwrap();
        assert_eq!(
            json,
            r#"{"kind":"pointer","to":{"kind":"opaque","name":"TsContext"},"mutable":true}"#
        );
        assert_eq!(serde_json::from_str::<TypeRef>(&json).unwrap(), ty);
        assert_eq!(ty.pointee().and_then(TypeRef::named), Some("TsContext"));
        let meta = Meta {
            unit: Some("deg".into()),
            nullable: true,
            ..Meta::default()
        };
        assert_eq!(
            serde_json::to_string(&meta).unwrap(),
            r#"{"unit":"deg","nullable":true}"#
        );
        assert_eq!(
            serde_json::to_string(&Role::StructOut).unwrap(),
            "\"struct_out\""
        );
    }
}
