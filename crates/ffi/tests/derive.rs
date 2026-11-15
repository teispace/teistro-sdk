//! `ts_context_derive` as a C caller meets it: a child context under a
//! per-call settings patch and locale, sharing its parent's engine, and
//! each refusal with its record (`docs/03-design/context-and-config.md`).

#![allow(
    unsafe_code,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "tests cross the boundary and fail by panicking"
)]

use core::ffi::{CStr, c_char};
use core::ptr;

use teistro_core::Status;
use teistro_ffi::context::{TsContext, ts_context_derive, ts_context_free, ts_error_free};

mod common;
use common::{Ctx, Record, blank_error, read_record};

/// A child of `parent` under `patch` and `locale`, or the record of why
/// not; the child is freed on drop.
fn derive(
    parent: *const TsContext,
    patch: Option<&CStr>,
    locale: Option<&CStr>,
) -> Result<Ctx, Record> {
    let text = |text: Option<&CStr>| text.map_or(ptr::null(), CStr::as_ptr);
    let mut child: *mut TsContext = ptr::null_mut();
    let mut error = blank_error();
    // SAFETY: `parent` is null or live, the texts are null or
    // NUL-terminated, and both out slots are writable.
    let status = unsafe {
        ts_context_derive(
            parent,
            text(patch),
            text(locale),
            &raw mut child,
            &raw mut error,
        )
    };
    if status == Status::Ok {
        assert!(!child.is_null(), "an answered derive writes its handle");
        return Ok(Ctx { handle: child });
    }
    assert!(child.is_null(), "a refused derive writes no handle");
    let record = read_record(&error);
    // SAFETY: the record the refusal wrote, freed once.
    unsafe { ts_error_free(&raw mut error) };
    Err(record)
}

/// The settings a context resolved.
fn settings(ctx: &Ctx) -> teistro_core::settings::Settings {
    // SAFETY: a live handle from `ts_context_new` or `ts_context_derive`.
    let context = unsafe { &*ctx.handle };
    context.settings().clone()
}

#[test]
fn a_patch_changes_only_the_child() {
    let parent = Ctx::defaults();
    let before = settings(&parent);
    let child = derive(
        parent.handle,
        Some(c"{\"frame\": {\"node\": \"TRUE\"}}"),
        None,
    )
    .unwrap();
    assert_ne!(settings(&child), before, "the patch reaches the child");
    assert_eq!(settings(&parent), before, "and leaves the parent as it was");
    // A child is a context in its own right: it outlives nothing it needs.
    let grandchild = derive(child.handle, None, None).unwrap();
    assert_eq!(settings(&grandchild), settings(&child));
}

#[test]
fn no_patch_is_the_parents_settings_and_a_locale_is_the_childs() {
    let parent = Ctx::defaults();
    let child = derive(parent.handle, None, Some(c"ne-Deva-NP")).unwrap();
    assert_eq!(settings(&child), settings(&parent));
    // SAFETY: both handles are live.
    let (parent_ref, child_ref) = unsafe { (&*parent.handle, &*child.handle) };
    assert_eq!(child_ref.intl().locale(), "ne-Deva-NP");
    assert_ne!(parent_ref.intl().locale(), "ne-Deva-NP");
    assert_eq!(child_ref.profile(), parent_ref.profile());
}

#[test]
fn a_null_parent_is_refused_by_name() {
    let (status, _, field, _, _) = derive(ptr::null(), None, None).unwrap_err();
    assert_eq!(status, Status::InvalidArg);
    assert_eq!(field.as_deref(), Some("context"));
}

#[test]
fn a_malformed_or_unknown_patch_is_refused_and_the_parent_lives_on() {
    let parent = Ctx::defaults();
    let (status, message, _, _, _) =
        derive(parent.handle, Some(c"{\"frame\": "), None).unwrap_err();
    assert_ne!(status, Status::Ok);
    assert!(!message.is_empty(), "a refusal says why");
    let (status, _, field, _, _) = derive(
        parent.handle,
        Some(c"{\"frame\": {\"node\": \"SIDEWAYS\"}}"),
        None,
    )
    .unwrap_err();
    assert_ne!(status, Status::Ok);
    assert!(
        field.as_deref().is_some_and(|field| field.contains("node")),
        "the refusal names the field: {field:?}"
    );
    // The parent is untouched by its children's refusals.
    assert!(derive(parent.handle, None, None).is_ok());
}

#[test]
fn an_unknown_locale_is_refused() {
    let parent = Ctx::defaults();
    let (status, _, _, _, _) = derive(parent.handle, None, Some(c"xx-Nope-ZZ")).unwrap_err();
    assert_ne!(status, Status::Ok);
}

#[test]
fn text_that_is_not_utf8_is_refused_by_name() {
    let parent = Ctx::defaults();
    let bytes: [u8; 3] = [0xff, 0xfe, 0];
    let mut child: *mut TsContext = ptr::null_mut();
    let mut error = blank_error();
    // SAFETY: a live parent, a NUL-terminated (if invalid UTF-8) patch, a
    // null locale and writable slots.
    let status = unsafe {
        ts_context_derive(
            parent.handle,
            bytes.as_ptr().cast::<c_char>(),
            ptr::null(),
            &raw mut child,
            &raw mut error,
        )
    };
    assert_ne!(status, Status::Ok);
    let (_, _, field, _, _) = read_record(&error);
    assert_eq!(field.as_deref(), Some("patch_json"));
    // SAFETY: the record the refusal wrote, freed once; the null child is
    // ignored.
    unsafe {
        ts_error_free(&raw mut error);
        ts_context_free(child);
    }
}
