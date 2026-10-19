//! Long work as a task (`03-design/mcp-server.md` §7, P7): the
//! `io.modelcontextprotocol/tasks` extension (SEP-2663). A call whose
//! client declares the extension is computed on a thread of its own; it is
//! answered as usual if it finishes within the server's patience, and as a
//! task handle otherwise, which `tasks/get` polls and `tasks/cancel` stops.
//!
//! The store is the process's: every worker of the HTTP transport shares
//! one, so a poll reaches the task whichever worker serves it. A task is
//! kept an hour after it ends, and the oldest ended ones go first once
//! [`MOST_KEPT`] have ended.

use std::collections::HashMap;
#[cfg(not(target_family = "wasm"))]
use std::fmt::Write as _;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value, json};

use crate::watch::Interrupt;

/// The extension's identifier, under `extensions` in both sides'
/// capabilities.
pub(crate) const EXTENSION: &str = "io.modelcontextprotocol/tasks";

/// How long the server waits on a call before handing back a task: a
/// call quicker than this is answered as if no task had been possible.
pub(crate) const PATIENCE: Duration = Duration::from_secs(2);

/// How often a client is asked to poll: the progress a task reports
/// moves no faster than every quarter of a second.
const POLL_MS: u64 = 1_000;

/// How long an ended task stays readable.
const KEPT: Duration = Duration::from_secs(60 * 60);

/// The most tasks working at once; a call past it is answered inline,
/// which the extension allows of any call.
pub(crate) const MOST_WORKING: usize = 8;

/// The most ended tasks kept at once.
const MOST_KEPT: usize = 256;

/// Where a task stands.
#[derive(Clone, Debug)]
pub(crate) enum Status {
    Working,
    /// The call's result, a `CallToolResult` (`isError` included).
    Completed(Value),
    /// The JSON-RPC error the call ended with.
    Failed(Value),
    Cancelled,
}

#[derive(Debug)]
struct Entry {
    created: SystemTime,
    updated: SystemTime,
    status: Status,
    message: Option<String>,
    interrupt: Interrupt,
    /// When it ended, for [`KEPT`].
    ended: Option<Instant>,
}

/// Every task the process holds.
#[derive(Debug, Default)]
pub(crate) struct Store {
    tasks: Mutex<HashMap<String, Entry>>,
}

/// What a task method was refused for.
pub(crate) enum Refused {
    /// No task by that id: `-32602`.
    Unknown(String),
}

impl Store {
    pub(crate) fn shared() -> Arc<Store> {
        Arc::new(Store::default())
    }

    fn locked(&self) -> std::sync::MutexGuard<'_, HashMap<String, Entry>> {
        let mut tasks = self.tasks.lock().unwrap_or_else(PoisonError::into_inner);
        sweep(&mut tasks);
        tasks
    }

    /// A new working task stopped through `interrupt`, or `None` when
    /// [`MOST_WORKING`] are working or no unguessable id can be drawn.
    pub(crate) fn begin(&self, interrupt: Interrupt) -> Option<String> {
        let mut tasks = self.locked();
        let working = tasks
            .values()
            .filter(|task| matches!(task.status, Status::Working))
            .count();
        if working >= MOST_WORKING {
            return None;
        }
        let id = fresh_id()?;
        let now = SystemTime::now();
        tasks.insert(
            id.clone(),
            Entry {
                created: now,
                updated: now,
                status: Status::Working,
                message: None,
                interrupt,
                ended: None,
            },
        );
        Some(id)
    }

    /// The working task's latest word on its progress.
    pub(crate) fn report(&self, id: &str, message: String) {
        if let Some(task) = self.locked().get_mut(id) {
            if matches!(task.status, Status::Working) {
                task.message = Some(message);
                task.updated = SystemTime::now();
            }
        }
    }

    /// The task ends as `status`.
    pub(crate) fn end(&self, id: &str, status: Status) {
        if let Some(task) = self.locked().get_mut(id) {
            task.status = status;
            task.message = None;
            task.updated = SystemTime::now();
            task.ended = Some(Instant::now());
        }
    }

    /// The ended task taken out of the store, for a call answered within
    /// the server's patience; `None` while it works.
    pub(crate) fn take_ended(&self, id: &str) -> Option<Status> {
        let mut tasks = self.locked();
        let ended = tasks.get(id).is_some_and(|task| task.ended.is_some());
        ended
            .then(|| tasks.remove(id))
            .flatten()
            .map(|task| task.status)
    }

    /// The task as `tasks/get` and `CreateTaskResult` say it.
    pub(crate) fn view(&self, id: &str) -> Result<Map<String, Value>, Refused> {
        let tasks = self.locked();
        let task = tasks.get(id).ok_or_else(|| unknown(id))?;
        Ok(view(id, task))
    }

    /// Asks the task to stop: cooperative, so it may still complete.
    pub(crate) fn cancel(&self, id: &str) -> Result<(), Refused> {
        let tasks = self.locked();
        let task = tasks.get(id).ok_or_else(|| unknown(id))?;
        if matches!(task.status, Status::Working) {
            task.interrupt.cancel(&Value::String(id.to_owned()));
        }
        Ok(())
    }

    /// Whether the task is held, for `tasks/update`, which has nothing to
    /// update: no task of this server ever asks its client for input.
    pub(crate) fn known(&self, id: &str) -> Result<(), Refused> {
        self.locked()
            .contains_key(id)
            .then_some(())
            .ok_or_else(|| unknown(id))
    }
}

fn unknown(id: &str) -> Refused {
    Refused::Unknown(format!(
        "no task `{id}`: it was never created here, or ended over an hour ago"
    ))
}

/// Drops the tasks ended more than [`KEPT`] ago, then the oldest ended
/// past [`MOST_KEPT`].
fn sweep(tasks: &mut HashMap<String, Entry>) {
    tasks.retain(|_, task| task.ended.is_none_or(|ended| ended.elapsed() < KEPT));
    let mut ended: Vec<(Instant, String)> = tasks
        .iter()
        .filter_map(|(id, task)| task.ended.map(|at| (at, id.clone())))
        .collect();
    if ended.len() > MOST_KEPT {
        ended.sort_unstable();
        for (_, id) in ended.iter().take(ended.len() - MOST_KEPT) {
            tasks.remove(id);
        }
    }
}

fn view(id: &str, task: &Entry) -> Map<String, Value> {
    let (status, payload) = match &task.status {
        Status::Working => ("working", None),
        Status::Completed(result) => ("completed", Some(("result", result.clone()))),
        Status::Failed(error) => ("failed", Some(("error", error.clone()))),
        Status::Cancelled => ("cancelled", None),
    };
    // Unlimited while it works; once it ends, its life so far and the
    // hour it is kept for, counted as the extension counts it, from its
    // creation.
    let ttl = task.ended.map_or(Value::Null, |_| {
        let lived = task
            .updated
            .duration_since(task.created)
            .unwrap_or_default();
        json!(u64::try_from((lived + KEPT).as_millis()).unwrap_or(u64::MAX))
    });
    let mut view = Map::new();
    view.insert(String::from("taskId"), json!(id));
    view.insert(String::from("status"), json!(status));
    if let Some(message) = &task.message {
        view.insert(String::from("statusMessage"), json!(message));
    }
    view.insert(String::from("createdAt"), json!(iso(task.created)));
    view.insert(String::from("lastUpdatedAt"), json!(iso(task.updated)));
    view.insert(String::from("ttlMs"), ttl);
    view.insert(String::from("pollIntervalMs"), json!(POLL_MS));
    if let Some((key, value)) = payload {
        view.insert(String::from(key), value);
    }
    view
}

/// 128 bits from the operating system, as 32 hex digits: a task id is
/// the only thing standing between one caller and another's task.
#[cfg(not(target_family = "wasm"))]
fn fresh_id() -> Option<String> {
    let mut bytes = [0_u8; 16];
    getrandom::fill(&mut bytes).ok()?;
    Some(
        bytes
            .iter()
            .fold(String::with_capacity(32), |mut id, byte| {
                let _ = write!(id, "{byte:02x}");
                id
            }),
    )
}

/// None on wasm32, which has no thread to run a task on.
#[cfg(target_family = "wasm")]
fn fresh_id() -> Option<String> {
    None
}

/// An instant as ISO 8601 in UTC, to the millisecond.
fn iso(at: SystemTime) -> String {
    let since = at.duration_since(UNIX_EPOCH).unwrap_or_default();
    let seconds = since.as_secs();
    let (days, rest) = (seconds / 86_400, seconds % 86_400);
    let (year, month, day) = civil(i64::try_from(days).unwrap_or(i64::MAX));
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{:03}Z",
        rest / 3_600,
        rest % 3_600 / 60,
        rest % 60,
        since.subsec_millis()
    )
}

/// The Gregorian date `days` after 1970-01-01 (Hinnant's `civil_from_days`).
fn civil(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    (
        year,
        u32::try_from(month).unwrap_or(1),
        u32::try_from(day).unwrap_or(1),
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "a test fails by panicking")]
mod tests {
    use super::*;

    #[test]
    fn an_instant_reads_as_iso_8601_in_utc() {
        assert_eq!(iso(UNIX_EPOCH), "1970-01-01T00:00:00.000Z");
        let leap = UNIX_EPOCH + Duration::from_millis(951_782_400_000 + 3_723_004);
        assert_eq!(iso(leap), "2000-02-29T01:02:03.004Z");
        assert_eq!(civil(-1), (1969, 12, 31));
    }

    #[test]
    fn ids_are_long_and_distinct() {
        let (a, b) = (fresh_id().unwrap(), fresh_id().unwrap());
        assert_eq!(a.len(), 32);
        assert_ne!(a, b);
    }
}
