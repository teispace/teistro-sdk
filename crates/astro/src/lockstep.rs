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
    ///
    /// ```
    /// use teistro_astro::lockstep::Lockstep;
    ///
    /// // Nothing waits, so nothing is asked.
    /// let lockstep = Lockstep::new();
    /// let answer = |n: u8| async move { n };
    /// assert_eq!(lockstep.run([answer(1), answer(2)]), [1, 2]);
    /// ```
    #[must_use]
    pub const fn new() -> Lockstep<'s> {
        Lockstep {
            waits: RefCell::new(Vec::new()),
        }
    }

    /// The apparent positions of `body` at `instants` from `sky`, read in
    /// the round's one request of that source for that body.
    ///
    /// ```
    /// use teistro_astro::completion::Completion;
    /// use teistro_astro::delta_t::DeltaTModel;
    /// use teistro_astro::lockstep::Lockstep;
    /// use teistro_astro::sky::ApparentPositions;
    /// use teistro_core::quantity::{JulianDay, Ut1};
    /// use teistro_core::settings::OverridePolicy;
    /// use teistro_port_ephemeris::{Body, TestProvider};
    ///
    /// let provider = TestProvider::new();
    /// let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// let sky: &dyn ApparentPositions = &sky;
    /// let j2000 = JulianDay::<Ut1>::J2000;
    /// // Read in a round, the Sun is where it is read on the spot.
    /// let lockstep = Lockstep::new();
    /// let read = lockstep.run([lockstep.apparent(sky, Body::Sun, &[j2000])]);
    /// let sun = read.into_iter().next().expect("one search")?;
    /// assert_eq!(sun, [sky.apparent(Body::Sun, j2000)?]);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
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
    ///
    /// ```
    /// use teistro_astro::completion::Completion;
    /// use teistro_astro::delta_t::DeltaTModel;
    /// use teistro_astro::lockstep::Lockstep;
    /// use teistro_astro::sky::ApparentPositions;
    /// use teistro_core::quantity::{JulianDay, Ut1};
    /// use teistro_core::settings::OverridePolicy;
    /// use teistro_port_ephemeris::{Body, CountingProvider, TestProvider};
    ///
    /// let provider = CountingProvider::new(TestProvider::new());
    /// let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// let lockstep = Lockstep::new();
    /// // Three searches, each reading the Moon once: one request for all three.
    /// let days = [0.0, 1.0, 2.0].map(|d| JulianDay::<Ut1>::literal(2_460_000.5 + d));
    /// let sky: &dyn ApparentPositions = &sky;
    /// let read = lockstep.run(days.iter().map(|day| lockstep.apparent(sky, Body::Moon, core::slice::from_ref(day))));
    /// assert_eq!(read.len(), 3);
    /// assert!(read.iter().all(Result::is_ok));
    /// assert_eq!(provider.calls().positions, 1);
    /// ```
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

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::float_cmp,
        reason = "tests fail by panicking and compare the values they were handed"
    )]

    use core::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    /// A source answering each instant with its own Julian day as the
    /// right ascension, and counting its requests.
    #[derive(Default)]
    struct Echo {
        asked: AtomicUsize,
    }

    impl ApparentPositions for Echo {
        fn apparent(&self, _body: Body, ut1: JulianDay<Ut1>) -> Result<Apparent, Error> {
            Ok(Apparent {
                ra_deg: ut1.get(),
                dec_deg: 0.0,
                distance_au: 1.0,
            })
        }

        fn apparent_many(
            &self,
            body: Body,
            ut1: &[JulianDay<Ut1>],
            out: &mut Vec<Apparent>,
        ) -> Result<(), Error> {
            self.asked.fetch_add(1, Ordering::Relaxed);
            out.clear();
            for at in ut1 {
                out.push(self.apparent(body, *at)?);
            }
            Ok(())
        }

        fn describe(&self) -> String {
            String::from("an echo")
        }
    }

    /// A source refusing every request.
    struct Refuses;

    impl ApparentPositions for Refuses {
        fn apparent(&self, _body: Body, _ut1: JulianDay<Ut1>) -> Result<Apparent, Error> {
            Err(Error::unsupported("no sky here"))
        }

        fn describe(&self) -> String {
            String::from("a refusal")
        }
    }

    /// A source answering one instant fewer than it was asked for.
    struct Short;

    impl ApparentPositions for Short {
        fn apparent(&self, body: Body, ut1: JulianDay<Ut1>) -> Result<Apparent, Error> {
            Echo::default().apparent(body, ut1)
        }

        fn apparent_many(
            &self,
            body: Body,
            ut1: &[JulianDay<Ut1>],
            out: &mut Vec<Apparent>,
        ) -> Result<(), Error> {
            Echo::default().apparent_many(body, ut1, out)?;
            out.pop();
            Ok(())
        }

        fn describe(&self) -> String {
            String::from("a short answer")
        }
    }

    fn days(from: f64, n: u32) -> Vec<JulianDay<Ut1>> {
        (0..n)
            .map(|d| JulianDay::literal(from + f64::from(d)))
            .collect()
    }

    /// Two reads in a row, so each search waits twice.
    async fn twice<'s>(
        lockstep: &Lockstep<'s>,
        sky: &'s dyn ApparentPositions,
        body: Body,
        at: &[JulianDay<Ut1>],
    ) -> Result<Vec<f64>, Error> {
        let first = lockstep.apparent(sky, body, at).await?;
        let later: Vec<JulianDay<Ut1>> = first
            .iter()
            .map(|a| JulianDay::literal(a.ra_deg + 0.5))
            .collect();
        let second = lockstep.apparent(sky, body, &later).await?;
        Ok(second.iter().map(|a| a.ra_deg).collect())
    }

    #[test]
    fn each_search_gets_its_own_share_and_a_round_asks_once() {
        let echo = Echo::default();
        let lockstep = Lockstep::new();
        let starts = [days(10.0, 3), days(20.0, 1), days(30.0, 2)];
        let answers = lockstep.run(
            starts
                .iter()
                .map(|at| twice(&lockstep, &echo, Body::Moon, at)),
        );
        let answers: Vec<Vec<f64>> = answers.into_iter().map(Result::unwrap).collect();
        assert_eq!(
            answers,
            vec![vec![10.5, 11.5, 12.5], vec![20.5], vec![30.5, 31.5]]
        );
        // Two rounds, one body, one source: two requests for six searches'
        // twelve readings.
        assert_eq!(echo.asked.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn a_round_asks_each_source_and_body_apart() {
        let (a, b) = (Echo::default(), Echo::default());
        let lockstep = Lockstep::new();
        let at = days(5.0, 2);
        let searches = [
            (&a, Body::Moon),
            (&a, Body::Sun),
            (&b, Body::Moon),
            (&a, Body::Moon),
        ];
        let answers = lockstep.run(
            searches
                .iter()
                .map(|&(sky, body)| twice(&lockstep, sky, body, &at)),
        );
        assert!(
            answers
                .iter()
                .all(|answer| answer.as_ref().unwrap() == &[5.5, 6.5])
        );
        // Each round asks `a` for the Moon and the Sun and `b` for the Moon.
        assert_eq!(a.asked.load(Ordering::Relaxed), 4);
        assert_eq!(b.asked.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn a_refusal_reaches_every_search_waiting_on_it() {
        let (echo, refuses) = (Echo::default(), Refuses);
        let lockstep = Lockstep::new();
        let at = days(1.0, 2);
        let answers = lockstep.run([
            twice(&lockstep, &refuses, Body::Moon, &at),
            twice(&lockstep, &echo, Body::Moon, &at),
            twice(&lockstep, &refuses, Body::Moon, &at),
        ]);
        let no_sky = Err(Error::unsupported("no sky here"));
        assert_eq!(answers, [no_sky.clone(), Ok(vec![1.5, 2.5]), no_sky]);
    }

    #[test]
    fn a_short_answer_is_refused_rather_than_shared_out() {
        let lockstep = Lockstep::new();
        let at = days(1.0, 3);
        let answers = lockstep.run([twice(&lockstep, &Short, Body::Moon, &at)]);
        let error = answers.into_iter().next().unwrap().unwrap_err();
        assert_eq!(error.status, teistro_core::error::Status::Internal);
        assert!(error.message.contains("2 of 3"), "{}", error.message);
    }

    #[test]
    fn a_search_read_on_the_spot_finishes_unless_it_waits() {
        assert_eq!(finished(async { 7 }), Some(7));
        let echo = Echo::default();
        let lockstep = Lockstep::default();
        let at = days(1.0, 1);
        assert!(finished(lockstep.apparent(&echo, Body::Moon, &at)).is_none());
        assert_eq!(format!("{lockstep:?}"), "Lockstep { waiting: 1, .. }");
    }

    #[test]
    fn the_test_sources_name_themselves() {
        let names = [
            Echo::default().describe(),
            Refuses.describe(),
            Short.describe(),
        ];
        assert_eq!(names, ["an echo", "a refusal", "a short answer"]);
    }
}
