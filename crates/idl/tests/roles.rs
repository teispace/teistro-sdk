//! A parameter's role and a field's, inferred from the type alone where
//! the description does not say: every pointer a C signature can take, a
//! count that follows or does not follow what it counts, and a writable
//! pointer in a struct of columns and in one that is not.

#![allow(clippy::unwrap_used, reason = "a test fails by panicking")]

use std::path::Path;

use teistro_idl::model::{Api, ParamDef, Role, StructRole};
use teistro_idl::rules::{self, FieldRole, infer_role};
use teistro_idl::{Scalar, TypeRef};

fn api() -> Api {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../idl/api.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// The first struct of a role, by name.
fn of_role(api: &Api, role: StructRole) -> TypeRef {
    let s = api.structs.iter().find(|s| s.role == role).unwrap();
    TypeRef::Struct {
        name: s.name.clone(),
    }
}

#[test]
fn each_pointer_a_signature_takes_has_its_role() {
    let api = api();
    let role = |name: &str, ty: TypeRef, mutable: bool| {
        infer_role(&api, name, &TypeRef::pointer(ty, mutable), None)
    };
    let opaque = TypeRef::Opaque {
        name: String::from("TsContext"),
    };
    assert_eq!(
        role("out", TypeRef::pointer(opaque.clone(), true), true),
        Role::HandleOut
    );
    assert_eq!(role("ctx", opaque, true), Role::Handle);
    assert_eq!(role("user_data", TypeRef::Void, true), Role::UserData);
    assert_eq!(role("name", TypeRef::Char, false), Role::StringIn);
    let cases = [
        (StructRole::Vtable, "vtable", Role::VtableIn),
        (StructRole::Blob, "out_blob", Role::BlobOut),
        (StructRole::Blob, "blob", Role::BlobFree),
        (StructRole::OwnedString, "out_text", Role::StringOut),
        (StructRole::OwnedString, "text", Role::StringFree),
        (StructRole::BorrowedString, "out_str", Role::StrOut),
        (StructRole::BorrowedString, "str", Role::StrOut),
        (StructRole::Object, "out_request", Role::StructOut),
        (StructRole::Columns, "out_columns", Role::StructOut),
        (StructRole::Error, "out_error", Role::StructOut),
        (StructRole::Error, "error", Role::ErrorFree),
        (StructRole::Object, "request", Role::StructIn),
        (StructRole::Columns, "columns", Role::StructIn),
    ];
    for (struct_role, name, expected) in cases {
        assert_eq!(
            role(name, of_role(&api, struct_role), true),
            expected,
            "{struct_role:?} {name}"
        );
    }
    // A struct the description does not declare is an object.
    let unknown = TypeRef::Struct {
        name: String::from("TsNowhere"),
    };
    assert_eq!(role("out", unknown.clone(), true), Role::StructOut);
    assert_eq!(role("unknown", unknown, false), Role::StructIn);
}

#[test]
fn a_scalar_pointer_is_an_output_bytes_an_array_or_a_value() {
    let api = api();
    let role = |name: &str, ty: TypeRef, mutable: bool| {
        infer_role(&api, name, &TypeRef::pointer(ty, mutable), None)
    };
    let f64_ = TypeRef::scalar(Scalar::F64);
    let kind = TypeRef::Enum {
        name: String::from("Kind"),
    };
    assert_eq!(role("out_count", f64_.clone(), true), Role::ScalarOut);
    assert_eq!(
        role("out", TypeRef::scalar(Scalar::U8), true),
        Role::ScalarOut
    );
    // Writable, but not named for an output: a plain value.
    assert_eq!(role("values", f64_.clone(), true), Role::Value);
    assert_eq!(role("kinds", kind.clone(), true), Role::Value);
    assert_eq!(
        role("bytes", TypeRef::scalar(Scalar::U8), false),
        Role::BytesIn
    );
    assert_eq!(role("values", f64_, false), Role::ArrayIn);
    assert_eq!(role("kinds", kind, false), Role::ArrayIn);
    let callback = TypeRef::Callback {
        name: String::from("TsLog"),
    };
    assert_eq!(role("log", callback, false), Role::Value);
}

#[test]
fn a_count_is_a_length_only_after_what_it_counts() {
    let api = api();
    let count = TypeRef::scalar(Scalar::Usize);
    let after = |role: Role| -> ParamDef {
        let mut param = api
            .functions
            .iter()
            .flat_map(|f| &f.params)
            .next()
            .unwrap()
            .clone();
        param.role = role;
        param
    };
    for name in ["len", "count", "jd_count", "bytes_len"] {
        assert!(rules::is_length_name(name), "{name}");
        for counted in [Role::BytesIn, Role::ArrayIn] {
            assert_eq!(
                infer_role(&api, name, &count, Some(&after(counted))),
                Role::Length,
                "{name} after {counted:?}"
            );
        }
        assert_eq!(
            infer_role(&api, name, &count, Some(&after(Role::StringIn))),
            Role::Value,
            "{name} after a string"
        );
    }
    for name in ["length_deg", "lens", "counter"] {
        assert!(!rules::is_length_name(name), "{name}");
        assert_eq!(
            infer_role(&api, name, &count, Some(&after(Role::ArrayIn))),
            Role::Value,
            "{name}"
        );
    }
}

#[test]
fn a_writable_pointer_is_a_column_only_in_a_struct_of_columns() {
    let api = api();
    let columns = api
        .structs
        .iter()
        .find(|s| s.role == StructRole::Columns)
        .unwrap();
    // A writable column the description says nothing more of.
    let mut columns = columns.clone();
    let mut column = columns.fields.last().unwrap().clone();
    column.name = String::from("extra");
    column.ty = TypeRef::pointer(TypeRef::scalar(Scalar::F64), true);
    column.meta = teistro_idl::model::Meta::default();
    columns.fields.push(column);
    let at = columns.fields.len() - 1;
    let roles = rules::field_roles(&api, &columns);
    assert_eq!(roles.get(at), Some(&FieldRole::Column));
    let mut object = columns.clone();
    object.role = StructRole::Object;
    assert_eq!(
        rules::field_roles(&api, &object).get(at),
        Some(&FieldRole::Value)
    );
}
