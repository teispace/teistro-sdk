//! A context crossing threads at the C ABI, for the thread sanitizer
//! (`verify.yml`'s sanitizers job) as much as for its answers: one
//! context moved from thread to thread, as the header allows, answers
//! the same wherever it is used; and contexts founded at once on several
//! threads, each first to ask for the tables built once (the festival
//! rules, the transliteration tables, the built-in ephemeris's
//! segments), answer as one founded alone.
#![cfg(feature = "full")]
#![allow(
    unsafe_code,
    clippy::unwrap_used,
    reason = "tests cross the boundary and fail by panicking"
)]

use core::ptr;
use std::ffi::CString;
use std::sync::Barrier;
use std::thread;

use teistro_core::Status;
use teistro_ffi::context::TsEphemeris;
use teistro_ffi::intl::ts_intl_transliterate;
use teistro_ffi::panchanga::ts_almanac_festival_calendar;
use teistro_ffi::string::{TsStr, TsString};

mod common;

use common::{Ctx, lent as lent_text, owned};

/// The answers a context gives: a month of Nepal's festivals and a name
/// transliterated, each through tables built once for the process.
fn answers(ctx: &Ctx) -> Vec<String> {
    let calendar = CString::new(
        r#"{"first": {"year": 2025, "month": 10, "day": 1}, "last": {"year": 2025, "month": 10, "day": 31}, "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1337, "utcOffsetSeconds": 20700, "festivals": {"rules": "NEPAL"}, "stampUnixSeconds": 1767225600, "name": "Nepal", "refreshMinutes": 0}"#,
    )
    .unwrap();
    let mut answer = TsString::empty();
    // SAFETY: a live context, a NUL-terminated request and a slot.
    let status =
        unsafe { ts_almanac_festival_calendar(ctx.handle, calendar.as_ptr(), &raw mut answer) };
    assert_eq!(status, Status::Ok, "{:?}", ctx.last_error());
    let (text, from, to) = (c"सूर्य", c"deva", c"iast");
    let mut lent = TsStr {
        data: ptr::null(),
        len: 0,
    };
    // SAFETY: a live context, NUL-terminated strings and a slot.
    let status = unsafe {
        ts_intl_transliterate(
            ctx.handle,
            text.as_ptr(),
            from.as_ptr(),
            to.as_ptr(),
            &raw mut lent,
        )
    };
    assert_eq!(status, Status::Ok, "{:?}", ctx.last_error());
    vec![owned(answer), lent_text(lent)]
}

fn builtin() -> Ctx {
    Ctx::with_ephemeris(0, TsEphemeris::Builtin, None, None, None).unwrap()
}

/// A context handed to another thread whole: the header's "used by one
/// thread at a time; it may be moved between threads".
struct Moved(Ctx);

// SAFETY: a context may move between threads, and each is used by one
// thread at a time here, the one it was moved to.
unsafe impl Send for Moved {}

#[test]
fn contexts_founded_at_once_answer_as_one_founded_alone() {
    const THREADS: usize = 4;
    let start = Barrier::new(THREADS);
    let each: Vec<Vec<String>> = thread::scope(|scope| {
        let workers: Vec<_> = (0..THREADS)
            .map(|_| {
                scope.spawn(|| {
                    start.wait();
                    answers(&builtin())
                })
            })
            .collect();
        workers.into_iter().map(|w| w.join().unwrap()).collect()
    });
    let alone = answers(&builtin());
    for answered in each {
        assert_eq!(answered, alone);
    }
}

#[test]
fn a_context_moved_between_threads_answers_the_same_on_each() {
    let here = builtin();
    let first = answers(&here);
    let moved = Moved(here);
    let (moved, there) = thread::spawn(move || {
        let moved = moved;
        let there = answers(&moved.0);
        (moved, there)
    })
    .join()
    .unwrap();
    assert_eq!(there, first);
    // Back on the thread it was founded on, and freed there.
    assert_eq!(answers(&moved.0), first);
}
