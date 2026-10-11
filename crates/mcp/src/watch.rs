//! Cancellation and progress (`03-design/mcp-server.md` §7, P6): every
//! request a context makes of its engine passes a watch, which stops the
//! call once a transport has cancelled it and tells a caller holding a
//! progress token how much work is done.
//!
//! The watch sits on the provider, not in the SDK's loops, because every
//! computation reaches the engine and only the engine: a batch of charts,
//! a range of days and a research study all stop at the next position
//! they ask for, and an adapter's own operations stop with them.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use teistro_core::catalogue::Ayanamsha;
use teistro_core::quantity::{JulianDay, Ut1};
use teistro_port_ephemeris::capabilities::{Capabilities, Obliquity};
use teistro_port_ephemeris::{
    Angles, AnglesRequest, CrossingEvent, CrossingRequest, EphemerisProvider, HorizonRequest,
    PositionColumns, PositionRequest, ProviderError, TimeScale,
};

/// The least time between two progress notifications of one call, so a
/// call that asks its engine a million times sends a handful, not a
/// million.
const EVERY: Duration = Duration::from_millis(250);

/// How many requests cancelled before they ran are remembered; a
/// cancellation of a request that never arrives is forgotten after this
/// many others.
const REMEMBERED: usize = 256;

/// What reports a call's progress: the notification as a line of JSON.
pub type Notify = Arc<dyn Fn(String) + Send + Sync>;

/// A handle a transport keeps beside the server, on whatever thread
/// reads its input, to cancel a request while the server computes it.
#[derive(Clone)]
pub struct Interrupt {
    shared: Arc<Shared>,
}

impl core::fmt::Debug for Interrupt {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Interrupt").finish_non_exhaustive()
    }
}

impl Interrupt {
    /// Cancels the request `id`: stopped at its engine's next request if
    /// it is running, and never answered if it has not begun. An id the
    /// server has already answered changes nothing.
    pub fn cancel(&self, id: &Value) {
        self.shared.cancel(id);
    }

    /// Stops the request `id` if it is the one running, and otherwise
    /// changes nothing: what a closed connection does, which may close
    /// after its call has answered.
    #[cfg(not(target_family = "wasm"))]
    pub(crate) fn stop_running(&self, id: &Value) {
        let id = id.to_string();
        let calls = locked(&self.shared.calls);
        if calls.running.as_deref() == Some(id.as_str()) {
            self.shared.stop.store(true, Ordering::Relaxed);
        }
    }
}

/// What the server, its watch and its transport share.
pub(crate) struct Shared {
    /// Whether the running call is to stop.
    stop: AtomicBool,
    /// The running call and the requests cancelled before they ran, under
    /// one lock so a cancellation cannot fall between the two.
    calls: Mutex<Calls>,
    /// The engine requests the running call has made.
    work: AtomicU64,
    progress: Mutex<Option<Progress>>,
    notify: Mutex<Option<Notify>>,
}

impl core::fmt::Debug for Shared {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Watch")
            .field("running", &locked(&self.calls).running)
            .field("stopping", &self.stop.load(Ordering::Relaxed))
            .finish_non_exhaustive()
    }
}

#[derive(Default)]
struct Calls {
    /// The running call's id, as JSON text.
    running: Option<String>,
    cancelled: VecDeque<String>,
}

/// A running call's progress token and when it last heard.
struct Progress {
    token: Value,
    notify: Notify,
    last: Instant,
}

/// What a call's beginning decides.
pub(crate) enum Begun {
    /// The call runs; [`Shared::end`] says whether it was cancelled.
    Running,
    /// It was cancelled before it began, and is not answered.
    Cancelled,
}

fn locked<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Shared {
    pub(crate) fn new() -> Arc<Shared> {
        Arc::new(Shared {
            stop: AtomicBool::new(false),
            calls: Mutex::new(Calls::default()),
            work: AtomicU64::new(0),
            progress: Mutex::new(None),
            notify: Mutex::new(None),
        })
    }

    pub(crate) fn interrupt(self: &Arc<Shared>) -> Interrupt {
        Interrupt {
            shared: Arc::clone(self),
        }
    }

    /// Sends `notification` through the transport's sink, if it has one.
    pub(crate) fn say(&self, notification: String) {
        let notify = locked(&self.notify).clone();
        if let Some(notify) = notify {
            notify(notification);
        }
    }

    pub(crate) fn set_notify(&self, notify: Notify) {
        *locked(&self.notify) = Some(notify);
    }

    fn cancel(&self, id: &Value) {
        let id = id.to_string();
        let mut calls = locked(&self.calls);
        if calls.running.as_deref() == Some(id.as_str()) {
            self.stop.store(true, Ordering::Relaxed);
            return;
        }
        if calls.cancelled.len() == REMEMBERED {
            calls.cancelled.pop_front();
        }
        calls.cancelled.push_back(id);
    }

    /// Whether the running call is to stop.
    pub(crate) fn stopping(&self) -> bool {
        self.stop.load(Ordering::Relaxed)
    }

    /// The call `id` begins, heard by `token` when it carries one.
    pub(crate) fn begin(&self, id: &Value, token: Option<&Value>) -> Begun {
        let id = id.to_string();
        {
            let mut calls = locked(&self.calls);
            if let Some(at) = calls.cancelled.iter().position(|was| *was == id) {
                calls.cancelled.remove(at);
                return Begun::Cancelled;
            }
            calls.running = Some(id);
            self.stop.store(false, Ordering::Relaxed);
        }
        self.work.store(0, Ordering::Relaxed);
        let notify = locked(&self.notify).clone();
        *locked(&self.progress) = token.zip(notify).map(|(token, notify)| Progress {
            token: token.clone(),
            notify,
            last: Instant::now(),
        });
        Begun::Running
    }

    /// The running call ends; true when it was cancelled and its answer
    /// is not sent.
    pub(crate) fn end(&self) -> bool {
        locked(&self.calls).running = None;
        *locked(&self.progress) = None;
        self.stop.swap(false, Ordering::Relaxed)
    }

    /// One request of the engine: refused once the call is cancelled,
    /// and counted toward its progress.
    fn step(&self) -> Result<(), ProviderError> {
        if self.stop.load(Ordering::Relaxed) {
            return Err(ProviderError::Refused {
                detail: String::from("the call was cancelled"),
            });
        }
        let work = self.work.fetch_add(1, Ordering::Relaxed) + 1;
        // A lock every request would cost a batch; the clock is read
        // every so many.
        if work % 64 == 0 {
            if let Some(progress) = locked(&self.progress).as_mut() {
                if progress.last.elapsed() >= EVERY {
                    progress.last = Instant::now();
                    (progress.notify)(notification(&progress.token, work));
                }
            }
        }
        Ok(())
    }
}

/// A progress notification: the engine requests made so far, with no
/// total, since a call does not know beforehand how many it will make.
fn notification(token: &Value, work: u64) -> String {
    json!({
        "jsonrpc": "2.0",
        "method": "notifications/progress",
        "params": {
            "progressToken": token,
            "progress": work,
            "message": format!("{work} ephemeris requests answered"),
        },
    })
    .to_string()
}

/// The engine a context opened, behind the watch.
pub(crate) struct Watched {
    inner: Box<dyn EphemerisProvider>,
    shared: Arc<Shared>,
}

impl Watched {
    pub(crate) fn new(inner: Box<dyn EphemerisProvider>, shared: Arc<Shared>) -> Watched {
        Watched { inner, shared }
    }
}

impl EphemerisProvider for Watched {
    fn capabilities(&self) -> Capabilities {
        self.inner.capabilities()
    }

    fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
        self.shared.step()?;
        self.inner.positions(request)
    }

    fn obliquity(&self, jd: f64, scale: TimeScale) -> Result<Obliquity, ProviderError> {
        self.shared.step()?;
        self.inner.obliquity(jd, scale)
    }

    fn delta_t_seconds(&self, jd_ut1: f64) -> Result<f64, ProviderError> {
        self.shared.step()?;
        self.inner.delta_t_seconds(jd_ut1)
    }

    fn ayanamsha_deg(
        &self,
        jd: f64,
        scale: TimeScale,
        ayanamsha: Ayanamsha,
    ) -> Result<f64, ProviderError> {
        self.shared.step()?;
        self.inner.ayanamsha_deg(jd, scale, ayanamsha)
    }

    fn dut1_seconds(&self, jd_utc: f64) -> Result<f64, ProviderError> {
        self.shared.step()?;
        self.inner.dut1_seconds(jd_utc)
    }

    fn horizon_event(
        &self,
        request: &HorizonRequest,
    ) -> Result<Option<JulianDay<Ut1>>, ProviderError> {
        self.shared.step()?;
        self.inner.horizon_event(request)
    }

    fn angles(&self, request: &AnglesRequest) -> Result<Angles, ProviderError> {
        self.shared.step()?;
        self.inner.angles(request)
    }

    fn crossings(&self, request: &CrossingRequest) -> Result<Vec<CrossingEvent>, ProviderError> {
        self.shared.step()?;
        self.inner.crossings(request)
    }

    fn native_manifest(&self) -> Result<String, ProviderError> {
        self.inner.native_manifest()
    }

    fn native_call(&self, function: &str, arguments_json: &str) -> Result<String, ProviderError> {
        self.shared.step()?;
        self.inner.native_call(function, arguments_json)
    }
}
