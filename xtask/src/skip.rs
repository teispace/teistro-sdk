//! A skip is not a pass where the run says so.
//!
//! A gate that needs a tool the machine lacks says so and moves on, so a
//! contributor without Dart can still run the Node gate (ADR-0014). In
//! verify and in a release that same skip is a hole: a runner that lost
//! its Python reports green having checked nothing of Python. Under
//! `TEISTRO_STRICT` every [`skip`] fails the command, after it has run
//! everything else, so the log names every hole at once.
//!
//! Some skips are the design rather than the machine: a check made on one
//! row of the matrix only, or a gap a page admits. Those are [`excused`],
//! each saying why, and a strict run lets them through. An excuse is
//! written where the skip is, so adding one is a change a reviewer sees.

use std::fmt::Display;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The variable that makes a skip a failure. Set, and not `0` or empty,
/// in verify and in the release.
const STRICT: &str = "TEISTRO_STRICT";

/// How many skips this command made that a strict run refuses.
static SKIPPED: AtomicUsize = AtomicUsize::new(0);

/// A part of a gate that did not run because the machine lacks what it
/// needs: printed, and counted against a strict run.
pub(crate) fn skip(why: impl Display) {
    println!("skip  {why}");
    SKIPPED.fetch_add(1, Ordering::Relaxed);
}

/// A part of a gate that does not run here by design, and says why: a
/// strict run lets it through.
pub(crate) fn excused(why: impl Display, because: &str) {
    println!("skip  {why} (excused: {because})");
}

/// Whether this run makes a skip a failure.
fn strict() -> bool {
    std::env::var_os(STRICT).is_some_and(|value| !value.is_empty() && value != "0")
}

/// The command's exit code once its skips are counted: unchanged unless
/// the run is strict and something was skipped.
pub(crate) fn verdict(code: i32) -> i32 {
    let (code, message) = counted(code, SKIPPED.load(Ordering::Relaxed), strict());
    if let Some(message) = message {
        println!("{message}");
    }
    code
}

/// [`verdict`] without the process's state: the code, and what to print.
fn counted(code: i32, skipped: usize, strict: bool) -> (i32, Option<String>) {
    if skipped == 0 || !strict {
        return (code, None);
    }
    let message = format!(
        "FAIL  {skipped} part(s) of this gate skipped, and {STRICT} makes a skip a failure: \
         install what each `skip` above names"
    );
    (if code == 0 { 1 } else { code }, Some(message))
}

#[cfg(test)]
mod tests {
    use super::counted;

    #[test]
    fn a_skip_fails_only_a_strict_run() {
        assert_eq!(counted(0, 2, false), (0, None));
        assert_eq!(counted(0, 0, true), (0, None));
        let (code, message) = counted(0, 2, true);
        assert_eq!(code, 1);
        assert!(message.is_some_and(|text| text.starts_with("FAIL  2 part(s)")));
    }

    #[test]
    fn a_failure_keeps_its_own_code() {
        assert_eq!(counted(3, 1, true).0, 3);
        assert_eq!(counted(3, 0, true).0, 3);
    }
}
