//! The API description: what the extractor produces from the C ABI crate
//! and what every generator reads. Serialised as `api.json` so the file can
//! ship inside a package and be diffed in review.

use serde::{Deserialize, Serialize};

/// The whole description of one C ABI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Api {
    /// The ABI version the library's handshake returns.
    pub abi_version: u32,
    /// The symbol prefix (`tsp_`).
    pub prefix: String,
    /// The Rust source the description was extracted from.
    pub source: String,
    /// C-like enums with explicit discriminants.
    pub enums: Vec<EnumDef>,
    /// `#[repr(C)]` structs.
    pub structs: Vec<StructDef>,
    /// Function-pointer callback types.
    pub callbacks: Vec<CallbackDef>,
    /// Opaque handle types.
    pub opaques: Vec<OpaqueDef>,
    /// Exported functions.
    pub functions: Vec<FunctionDef>,
    /// The result blob layout.
    pub blob: BlobSchema,
}

/// A C-like enum.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EnumDef {
    /// The Rust and C name.
    pub name: String,
    /// The `repr` integer type.
    pub repr: String,
    /// The doc comment.
    pub doc: String,
    /// The variants in declaration order.
    pub values: Vec<EnumValue>,
}

/// One enum variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EnumValue {
    /// The variant name.
    pub name: String,
    /// The discriminant.
    pub value: i64,
    /// The doc comment.
    pub doc: String,
}

/// A `#[repr(C)]` struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct StructDef {
    /// The name.
    pub name: String,
    /// The doc comment.
    pub doc: String,
    /// The handshake field name when the struct carries `struct_size`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handshake: Option<String>,
    /// The blob format name when the struct describes a result blob.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
    /// The fields in order.
    pub fields: Vec<FieldDef>,
}

/// One struct field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FieldDef {
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

/// The metadata an `api:` doc line carries (ADR-0023).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct Meta {
    /// The unit, such as `deg` or `deg/day`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// The range in interval notation, such as `[1,5]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<String>,
    /// An example value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
    /// The enum the integer field stands for.
    #[serde(default, rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_name: Option<String>,
    /// A role such as `struct_size`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// The callback name on a function-pointer type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub callback: Option<String>,
    /// The handshake kind on a struct.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handshake: Option<String>,
    /// The blob format on a blob struct.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
}

/// A type as it appears at the boundary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum TypeRef {
    /// A machine scalar: `u8`, `u32`, `i32`, `i64`, `f64`, `usize`, `bool`.
    Scalar {
        /// The Rust spelling.
        name: String,
    },
    /// A named enum.
    Enum {
        /// The enum name.
        name: String,
    },
    /// A named struct.
    Struct {
        /// The struct name.
        name: String,
    },
    /// An opaque handle type.
    Opaque {
        /// The type name.
        name: String,
    },
    /// A pointer to something.
    Pointer {
        /// What it points at.
        to: Box<TypeRef>,
        /// `*mut` rather than `*const`.
        mutable: bool,
    },
    /// An optional function pointer.
    Callback {
        /// The callback type name.
        name: String,
    },
    /// `c_void`.
    Void,
    /// `c_char`.
    Char,
}

/// A function-pointer type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CallbackDef {
    /// The type alias name.
    pub name: String,
    /// The doc comment.
    pub doc: String,
    /// The parameters in order.
    pub params: Vec<ParamDef>,
    /// The return type.
    pub returns: TypeRef,
}

/// An opaque handle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OpaqueDef {
    /// The type name.
    pub name: String,
    /// The doc comment.
    pub doc: String,
}

/// An exported function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FunctionDef {
    /// The symbol name.
    pub name: String,
    /// The doc comment without the Safety section.
    pub doc: String,
    /// The Safety section, when the function is unsafe.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safety: Option<String>,
    /// The parameters in order.
    pub params: Vec<ParamDef>,
    /// The return type, `None` for `void`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub returns: Option<TypeRef>,
}

/// One parameter with its inferred role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ParamDef {
    /// The name.
    pub name: String,
    /// The type.
    pub ty: TypeRef,
    /// What the parameter is for, inferred from its type and name.
    pub role: Role,
}

/// The role of a parameter, which decides how every binding marshals it.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Role {
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
    BlobInOut,
}

/// The result blob layout, from the schema constants of the ABI crate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BlobSchema {
    /// The magic number.
    pub magic: u32,
    /// The layout version.
    pub version: u32,
    /// The sections in id order.
    pub sections: Vec<SectionSchema>,
}

/// One section of the blob.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SectionSchema {
    /// The section id in the table of contents.
    pub id: u32,
    /// The section name.
    pub name: String,
    /// `fixed` (one record) or `columns` (a directory of columns).
    pub kind: String,
    /// The fields of a fixed section or the columns of a column section.
    pub fields: Vec<ColumnDef>,
}

/// A field or column with its scalar type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ColumnDef {
    /// The name.
    pub name: String,
    /// The scalar type: `u8`, `i32`, `u32`, `i64` or `f64`.
    pub ty: String,
}

impl ColumnDef {
    /// The byte width of the scalar.
    #[must_use]
    pub(crate) fn width(&self) -> usize {
        match self.ty.as_str() {
            "u8" | "i8" => 1,
            "u16" | "i16" => 2,
            "u32" | "i32" | "f32" => 4,
            _ => 8,
        }
    }
}

/// `TspSettings` to `Settings`, `tsp_context_new` to `context_new`.
#[must_use]
pub(crate) fn strip_prefix(name: &str, prefix: &str) -> String {
    let lower = prefix.to_ascii_lowercase();
    let pascal = {
        let mut chars = lower.trim_end_matches('_').chars();
        let first = chars
            .next()
            .map(|c| c.to_ascii_uppercase())
            .unwrap_or_default();
        let rest: String = chars.collect();
        format!("{first}{rest}")
    };
    name.strip_prefix(&lower)
        .or_else(|| name.strip_prefix(&pascal))
        .unwrap_or(name)
        .to_string()
}

/// `dasha_depth` to `dashaDepth`.
#[must_use]
pub(crate) fn camel(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut upper = false;
    for c in name.chars() {
        if c == '_' {
            upper = true;
        } else if upper {
            out.push(c.to_ascii_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// `dasha_depth` to `DashaDepth`.
#[must_use]
pub(crate) fn pascal(name: &str) -> String {
    let c = camel(name);
    let mut chars = c.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
        None => String::new(),
    }
}

/// `TspStatus` to `TSP_STATUS`, `DepthOutOfRange` to `DEPTH_OUT_OF_RANGE`.
#[must_use]
pub(crate) fn screaming(name: &str) -> String {
    let mut out = String::with_capacity(name.len() + 4);
    let mut previous_lower = false;
    for c in name.chars() {
        if c.is_ascii_uppercase() && previous_lower {
            out.push('_');
        }
        previous_lower = c.is_ascii_lowercase() || c.is_ascii_digit();
        out.push(c.to_ascii_uppercase());
    }
    out
}
