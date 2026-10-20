//! Searches run side by side, each step of every one asked for in one
//! request.
//!
//! A search is serial: each instant it reads is chosen from the answer to
//! the last, so a search alone asks its source once a step. Many
//! searches are not: the next instant of each is known as soon as each
//! has its last answer, so a round can ask for all of them together. A
//! search written as an `async fn` over a [`Lockstep`] waits for its
//! round's answer instead of asking alone, and [`Lockstep::run`] polls
//! every search, asks once for everything they wait on, and hands each
//! its share; a hundred searches of twenty steps are twenty requests
//! instead of two thousand.
//!
//! Each search takes the steps it takes alone, on the readings it would
//! read alone, so what it answers is its own answer to the bit: lockstep
//! changes how many requests the source sees, never a value. The same
//! code read on the spot ([`finished`]) is the search's ordinary form.

use core::cell::RefCell;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use std::rc::Rc;

use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Ut1};
use teistro_port_ephemeris::Body;

use crate::sky::{Apparent, ApparentPositions};

/// A future polled once with nothing to wake it: its output when it
/// finished in that poll, which a search reading on the spot always does,
/// and `None` when it waited on something.
///
/// It is how a search written once, as an `async fn` that reads either
/// on the spot or in a round, is run on the spot.
///
/// ```
/// assert_eq!(teistro_astro::lockstep::finished(async { 7 }), Some(7));
/// ```
pub fn finished<T>(future: impl Future<Output = T>) -> Option<T> {
    let mut future = core::pin::pin!(future);
    match future
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(value) => Some(value),
        Poll::Pending => None,
    }
}

/// Where a round leaves one wait's answer: empty until the round is
/// asked.
type Slot = Rc<RefCell<Option<Result<Vec<Apparent>, Error>>>>;

/// One search's wait: the instants it asked for and, once the round is
/// asked, their answer.
struct Wait<'s> {
    sky: &'s dyn ApparentPositions,
    body: Body,
    instants: Vec<JulianDay<Ut1>>,
    answer: Slot,
}

/// The searches of one run, each waiting on the source it reads.
///
/// ```
/// use teistro_astro::completion::Completion;
/// use teistro_astro::delta_t::DeltaTModel;
/// use teistro_astro::lockstep::Lockstep;
/// use teistro_astro::rise_set::Solver;
/// use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Ut1};
/// use teistro_core::settings::OverridePolicy;
/// use teistro_port_ephemeris::{Body, CountingProvider, Horizon, HorizonEventKind, TestProvider};
///
/// let provider = CountingProvider::new(TestProvider::new());
/// let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// let place = Place::new(Latitude::literal(27.7), Longitude::literal(85.3), Altitude::literal(1400.0));
/// let solver = Solver::new(&sky, Body::Moon, place, Horizon::UPPER_LIMB_REFRACTION, DeltaTModel::TableThenModel);
/// let days: Vec<JulianDay<Ut1>> = (0..30).map(|d| JulianDay::literal(2_460_000.5 + f64::from(d))).collect();
///
/// let alone: Vec<_> = days.iter().map(|day| solver.event(HorizonEventKind::Rise, *day, 1.0)).collect::<Result<_, _>>()?;
/// let asked_alone = provider.calls().positions;
/// provider.reset();
/// let lockstep = Lockstep::new();
/// let together = lockstep.run(days.iter().map(|day| solver.event_in(&lockstep, HorizonEventKind::Rise, *day, 1.0)));
/// let together: Vec<_> = together.into_iter().collect::<Result<_, _>>()?;
/// assert_eq!(alone, together);
/// assert!(provider.calls().positions * 4 < asked_alone);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
pub struct Lockstep<'s> {
    waits: RefCell<Vec<Wait<'s>>>,
}

impl core::fmt::Debug for Lockstep<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Lockstep")
            .field("waiting", &self.waits.borrow().len())
            .finish_non_exhaustive()
    }
}

impl Default for Lockstep<'_> {
    fn default() -> Self {
        Lockstep::new()
    }
}

impl<'s> Lockstep<'s> {
    /// A run with no search in it yet.
    #[must_use]
    pub const fn new() -> Lockstep<'s> {
        Lockstep {
            waits: RefCell::new(Vec::new()),
        }
    }

    /// The apparent positions of `body` at `instants` from `sky`, read in
    /// the round's one request of that source for that body.
    ///
    /// # Errors
    ///
    /// The source's refusal for the round, which every search waiting on
    /// it receives.
    pub async fn apparent(
        &self,
        sky: &'s dyn ApparentPositions,
        body: Body,
        instants: &[JulianDay<Ut1>],
    ) -> Result<Vec<Apparent>, Error> {
        let answer = Rc::new(RefCell::new(None));
        self.waits.borrow_mut().push(Wait {
            sky,
            body,
            instants: instants.to_vec(),
            answer: Rc::clone(&answer),
        });
        Answered { answer }.await
    }

    /// Run every search to its end, a round at a time: each round polls
    /// the searches still going, asks each source once a body for every
    /// instant they wait on, and hands each its share. The outputs come
    /// back in the order the searches were given.
    pub fn run<'f, T, F>(&self, searches: impl IntoIterator<Item = F>) -> Vec<T>
    where
        F: Future<Output = T> + 'f,
        T: 'f,
    {
        let mut going: Vec<Option<Pin<Box<dyn Future<Output = T> + 'f>>>> = searches
            .into_iter()
            .map(|search| Some(Box::pin(search) as Pin<Box<dyn Future<Output = T> + 'f>>))
            .collect();
        let mut done: Vec<Option<T>> = going.iter().map(|_| None).collect();
        let mut cx = Context::from_waker(Waker::noop());
        loop {
            for (slot, out) in going.iter_mut().zip(done.iter_mut()) {
                if let Some(search) = slot.as_mut()
                    && let Poll::Ready(value) = search.as_mut().poll(&mut cx)
                {
                    *out = Some(value);
                    *slot = None;
                }
            }
            let waits = core::mem::take(&mut *self.waits.borrow_mut());
            if waits.is_empty() {
                break;
            }
            Self::answer(&waits);
        }
        // A search that neither finished nor waited cannot be resumed:
        // nothing it awaits is anything a round answers. Every search
        // here awaits only its rounds, so none is left.
        done.into_iter().flatten().collect()
    }

    /// Ask each source once a body for every instant the round waits on,
    /// and hand each wait its share.
    fn answer(waits: &[Wait<'s>]) {
        // The sources and bodies the round waits on, in the order first
        // asked; two waits share a request only when they read the same
        // source, compared by address.
        let mut asks: Vec<(&'s dyn ApparentPositions, Body)> = Vec::new();
        for wait in waits {
            if !asks
                .iter()
                .any(|(sky, body)| core::ptr::addr_eq(*sky, wait.sky) && *body == wait.body)
            {
                asks.push((wait.sky, wait.body));
            }
        }
        for (sky, body) in asks {
            let mine: Vec<&Wait<'s>> = waits
                .iter()
                .filter(|wait| core::ptr::addr_eq(sky, wait.sky) && wait.body == body)
                .collect();
            let instants: Vec<JulianDay<Ut1>> = mine
                .iter()
                .flat_map(|wait| wait.instants.iter().copied())
                .collect();
            let mut apparent = Vec::with_capacity(instants.len());
            let asked = sky
                .apparent_many(body, &instants, &mut apparent)
                .and_then(|()| {
                    if apparent.len() == instants.len() {
                        Ok(())
                    } else {
                        Err(Error::internal(format!(
                            "a source answered {} of {} instants asked for in lockstep",
                            apparent.len(),
                            instants.len()
                        )))
                    }
                });
            let mut at = 0;
            for wait in mine {
                let share = match &asked {
                    Ok(()) => Ok(apparent
                        .get(at..at + wait.instants.len())
                        .map(<[Apparent]>::to_vec)
                        .unwrap_or_default()),
                    Err(error) => Err(error.clone()),
                };
                at += wait.instants.len();
                *wait.answer.borrow_mut() = Some(share);
            }
        }
    }
}

/// A wait's answer, ready once its round has been asked.
struct Answered {
    answer: Slot,
}

impl Future for Answered {
    type Output = Result<Vec<Apparent>, Error>;

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        match self.answer.borrow_mut().take() {
            Some(answer) => Poll::Ready(answer),
            None => Poll::Pending,
        }
    }
}
