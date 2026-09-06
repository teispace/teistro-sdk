//! The C layout of every boundary type: sizes, alignments and field
//! offsets by the platform C rules over the description alone, so the
//! header can assert what the library was built with and a test can hold
//! Rust's `size_of` and `offset_of` to the same numbers.

use core::fmt;

use crate::model::{Api, Scalar, StructDef, TypeRef};

/// What the layout depends on: the pointer width, which also decides
/// `size_t` and the alignment of 64-bit scalars on the targets the SDK
/// ships to (every one aligns `double` and `int64_t` to 8 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Target {
    /// The pointer width in bytes.
    pub pointer_width: usize,
}

impl Target {
    /// Every 64-bit target the SDK builds for.
    pub const LP64: Target = Target { pointer_width: 8 };
    /// The 32-bit targets (wasm32, 32-bit ARM).
    pub const ILP32: Target = Target { pointer_width: 4 };

    /// The target this build runs on.
    #[must_use]
    pub const fn host() -> Target {
        Target {
            pointer_width: core::mem::size_of::<usize>(),
        }
    }
}

/// A size and an alignment in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Layout {
    /// The size.
    pub size: usize,
    /// The alignment.
    pub align: usize,
}

/// A struct's layout: its size, alignment and every field's offset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructLayout {
    /// The size, padded to the alignment.
    pub size: usize,
    /// The alignment: the largest field alignment.
    pub align: usize,
    /// The offset of each field, in field order.
    pub offsets: Vec<usize>,
}

/// Why a type has no layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutError {
    /// A name the description does not define.
    Unknown(String),
    /// A type that only exists behind a pointer.
    Unsized(String),
}

impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LayoutError::Unknown(name) => write!(f, "`{name}` is not defined in the description"),
            LayoutError::Unsized(name) => {
                write!(f, "`{name}` has no size; it lives behind a pointer")
            }
        }
    }
}

impl std::error::Error for LayoutError {}

/// The layout of a type on a target.
///
/// # Errors
///
/// A name the description lacks, or a type with no size (`void`, an opaque
/// handle) outside a pointer.
pub fn layout_of(api: &Api, ty: &TypeRef, target: Target) -> Result<Layout, LayoutError> {
    match ty {
        TypeRef::Scalar { scalar } => Ok(scalar_layout(*scalar, target)),
        TypeRef::Enum { name } => api
            .enum_named(name)
            .map(|e| scalar_layout(e.repr, target))
            .ok_or_else(|| LayoutError::Unknown(name.clone())),
        TypeRef::Struct { name } => api
            .struct_named(name)
            .ok_or_else(|| LayoutError::Unknown(name.clone()))
            .and_then(|s| struct_layout(api, s, target))
            .map(|s| Layout {
                size: s.size,
                align: s.align,
            }),
        TypeRef::Opaque { name } => Err(LayoutError::Unsized(name.clone())),
        TypeRef::Callback { .. } | TypeRef::Pointer { .. } => Ok(Layout {
            size: target.pointer_width,
            align: target.pointer_width,
        }),
        TypeRef::Array { of, len } => {
            let element = layout_of(api, of, target)?;
            Ok(Layout {
                size: element.size * len,
                align: element.align,
            })
        }
        TypeRef::Void => Err(LayoutError::Unsized(String::from("void"))),
        TypeRef::Char => Ok(Layout { size: 1, align: 1 }),
    }
}

/// The layout of a `#[repr(C)]` struct on a target: each field at the
/// next offset aligned to its alignment, the whole padded to the largest.
///
/// # Errors
///
/// As [`layout_of`], for any field.
pub fn struct_layout(
    api: &Api,
    s: &StructDef,
    target: Target,
) -> Result<StructLayout, LayoutError> {
    let mut offset = 0usize;
    let mut align = 1usize;
    let mut offsets = Vec::with_capacity(s.fields.len());
    for field in &s.fields {
        let layout = layout_of(api, &field.ty, target)?;
        offset = round_up(offset, layout.align);
        offsets.push(offset);
        offset += layout.size;
        align = align.max(layout.align);
    }
    Ok(StructLayout {
        size: round_up(offset, align),
        align,
        offsets,
    })
}

const fn scalar_layout(scalar: Scalar, target: Target) -> Layout {
    let width = scalar.width(target.pointer_width);
    Layout {
        size: width,
        align: width,
    }
}

const fn round_up(value: usize, align: usize) -> usize {
    if align == 0 {
        value
    } else {
        value.div_ceil(align) * align
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;
    use crate::model::{EnumDef, EnumValue, FieldDef, Meta, StructRole};

    fn field(name: &str, ty: TypeRef) -> FieldDef {
        FieldDef {
            name: name.to_string(),
            ty,
            doc: String::new(),
            meta: Meta::default(),
        }
    }

    fn api() -> Api {
        Api {
            schema: crate::model::SCHEMA.to_string(),
            abi_version: 1,
            sdk_version: "0.0.0".into(),
            prefix: "ts_".into(),
            sources: vec![],
            constants: vec![],
            enums: vec![EnumDef {
                name: "Status".into(),
                doc: String::new(),
                repr: Scalar::I32,
                kind: None,
                values: vec![EnumValue {
                    name: "Ok".into(),
                    value: 0,
                    doc: String::new(),
                    key: None,
                    deprecated: false,
                }],
                source: String::new(),
            }],
            opaques: vec![],
            callbacks: vec![],
            structs: vec![
                StructDef {
                    name: "Inner".into(),
                    doc: String::new(),
                    role: StructRole::Object,
                    fields: vec![
                        field("a", TypeRef::scalar(Scalar::U8)),
                        field("b", TypeRef::scalar(Scalar::F64)),
                    ],
                    source: String::new(),
                },
                StructDef {
                    name: "Outer".into(),
                    doc: String::new(),
                    role: StructRole::Object,
                    fields: vec![
                        field("struct_size", TypeRef::scalar(Scalar::U32)),
                        field(
                            "status",
                            TypeRef::Enum {
                                name: "Status".into(),
                            },
                        ),
                        field(
                            "flags",
                            TypeRef::Array {
                                of: Box::new(TypeRef::scalar(Scalar::U8)),
                                len: 3,
                            },
                        ),
                        field(
                            "inner",
                            TypeRef::Struct {
                                name: "Inner".into(),
                            },
                        ),
                        field("text", TypeRef::pointer(TypeRef::Char, false)),
                        field("count", TypeRef::scalar(Scalar::Usize)),
                    ],
                    source: String::new(),
                },
            ],
            functions: vec![],
            blobs: vec![],
        }
    }

    #[test]
    fn structs_follow_the_c_rules_on_both_widths() {
        let api = api();
        let outer = api.struct_named("Outer").unwrap();
        let lp64 = struct_layout(&api, outer, Target::LP64).unwrap();
        // u32 at 0, i32 at 4, u8[3] at 8, Inner (align 8, size 16) at 16,
        // pointer at 32, size_t at 40: 48 bytes, aligned to 8.
        assert_eq!(lp64.offsets, [0, 4, 8, 16, 32, 40]);
        assert_eq!((lp64.size, lp64.align), (48, 8));
        let ilp32 = struct_layout(&api, outer, Target::ILP32).unwrap();
        // The same until the pointer: 4 bytes at 32, size_t 4 bytes at 36,
        // padded to the 8-byte alignment the inner double imposes.
        assert_eq!(ilp32.offsets, [0, 4, 8, 16, 32, 36]);
        assert_eq!((ilp32.size, ilp32.align), (40, 8));
    }

    #[test]
    fn unsized_and_unknown_types_are_refused() {
        let api = api();
        assert_eq!(
            layout_of(&api, &TypeRef::Void, Target::LP64),
            Err(LayoutError::Unsized("void".into()))
        );
        assert_eq!(
            layout_of(
                &api,
                &TypeRef::Opaque {
                    name: "TsContext".into()
                },
                Target::LP64
            ),
            Err(LayoutError::Unsized("TsContext".into()))
        );
        assert_eq!(
            layout_of(
                &api,
                &TypeRef::Struct {
                    name: "Nope".into()
                },
                Target::LP64
            ),
            Err(LayoutError::Unknown("Nope".into()))
        );
        assert_eq!(
            layout_of(&api, &TypeRef::pointer(TypeRef::Void, true), Target::ILP32).unwrap(),
            Layout { size: 4, align: 4 }
        );
        assert_eq!(Target::host().pointer_width, core::mem::size_of::<usize>());
    }
}
