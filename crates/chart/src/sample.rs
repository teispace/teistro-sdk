//! A chart to try things on: Kathmandu at 05:45 local time on 21 June
//! 2024 (0h UTC), founded on the test provider under the default profile.
//!
//! Every crate built on a foundation documents itself with a chart, and
//! each would otherwise carry the thirty lines that found one in every
//! example. This is those lines once, founded the way the façade founds a
//! chart from its settings.
//!
//! It is **not** an astrological answer. The test provider computes no
//! astronomy: its elements are a fixture
//! ([`teistro_port_ephemeris::TestProvider`]), so the chart is
//! deterministic and needs no engine, and its longitudes are not the
//! sky's.
//!
//! ```
//! use teistro_chart::sample;
//!
//! let foundation = sample::foundation()?;
//! assert_eq!(foundation.grahas.len(), 9);
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::precession::PrecessionModel;
use teistro_calendar::Gregorian;
use teistro_calendar::solar::drik::DrikSun;
use teistro_core::catalogue::{Ayanamsha, ChartKind};
use teistro_core::error::Error;
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro_core::settings::{AyanamshaChoice, DEFAULT_PROFILE, Profile, Resolved, SettingsPatch};
use teistro_core::time::UtcOffset;
use teistro_port_ephemeris::TestProvider;

use crate::foundation::{ChartFoundation, Founder};

/// The instant: 0h UTC on 21 June 2024.
///
/// ```
/// use teistro_chart::sample;
///
/// assert_eq!(sample::INSTANT.get(), 2_460_482.5);
/// ```
pub const INSTANT: JulianDay<Utc> = JulianDay::literal(2_460_482.5);

/// Kathmandu: 27.7172° N, 85.3240° E, 1400 m.
///
/// ```
/// use teistro_chart::sample;
///
/// assert_eq!(sample::place().latitude.get(), 27.7172);
/// ```
#[must_use]
pub fn place() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.3240),
        Altitude::literal(1400.0),
    )
}

/// The default profile, resolved with nothing patched.
///
/// ```
/// use teistro_chart::sample;
/// use teistro_core::settings::DEFAULT_PROFILE;
///
/// assert_eq!(sample::resolved().profile.as_str(), DEFAULT_PROFILE);
/// ```
///
/// # Panics
///
/// Never: the default profile resolves, which its own tests hold.
#[must_use]
pub fn resolved() -> Resolved {
    #[expect(
        clippy::expect_used,
        reason = "the default profile resolving is a tested invariant"
    )]
    Profile::shipped(DEFAULT_PROFILE)
        .expect("the default profile ships")
        .resolve(&SettingsPatch::default())
        .expect("the default profile resolves")
}

/// The provider the sample founds on: the test provider, which every
/// sample shares.
static PROVIDER: TestProvider = TestProvider::new();

/// What a founder over the sample borrows: [`resolved`]'s settings and
/// the sun they name. [`Sample::founder`] lends them, so an example that
/// asks a founder for more than one chart — a transit, a lagna table, a
/// batch — needs no construction of its own.
///
/// ```
/// use teistro_chart::sample::{self, Sample};
/// use teistro_core::catalogue::ChartKind;
///
/// let sample = Sample::new();
/// let founder = sample.founder();
/// let chart = founder.found_one(sample::INSTANT, &sample::place(), ChartKind::Natal)?;
/// assert_eq!(chart.value, sample::foundation()?);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Debug)]
pub struct Sample {
    /// The settings, owned so the founder can borrow them.
    resolved: Resolved,
    /// The sun of the settings' ayanamsha, sunrise and override policy.
    sun: DrikSun<'static, TestProvider>,
}

impl Sample {
    /// The sample's parts, under [`resolved`]'s settings as the façade
    /// founds a chart from them.
    ///
    /// ```
    /// use teistro_chart::sample::Sample;
    ///
    /// let sample = Sample::new();
    /// assert_eq!(sample.resolved().profile.as_str(), teistro_core::settings::DEFAULT_PROFILE);
    /// ```
    #[must_use]
    pub fn new() -> Sample {
        let resolved = resolved();
        let settings = &resolved.settings;
        let ayanamsha = match settings.frame.ayanamsha {
            AyanamshaChoice::Catalogued { id } => id,
            AyanamshaChoice::Custom { .. } => Ayanamsha::Lahiri,
        };
        let sun = DrikSun::new(
            &PROVIDER,
            ayanamsha,
            settings.day.sunrise,
            settings.provider.overrides,
            DeltaTModel::default(),
        );
        Sample { resolved, sun }
    }

    /// The settings the sample founds under.
    ///
    /// ```
    /// use teistro_chart::sample::{self, Sample};
    ///
    /// assert_eq!(Sample::new().resolved(), &sample::resolved());
    /// ```
    #[must_use]
    pub const fn resolved(&self) -> &Resolved {
        &self.resolved
    }

    /// A founder over the test provider: the settings' sun, the Gregorian
    /// calendar, Nepal's +05:45 and the default precession and ΔT.
    ///
    /// ```
    /// use teistro_chart::sample::{self, Sample};
    ///
    /// let sample = Sample::new();
    /// let day = sample.founder().day_at(sample::INSTANT, &sample::place())?;
    /// assert!(day.lagna_sunrise().get() <= sample::INSTANT.get());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn founder(&self) -> Founder<'_, TestProvider> {
        Founder::new(
            &PROVIDER,
            &self.resolved,
            &self.sun,
            &Gregorian,
            &CLOCK,
            PrecessionModel::default(),
            DeltaTModel::default(),
        )
    }
}

impl Default for Sample {
    fn default() -> Sample {
        Sample::new()
    }
}

/// Nepal's clock, which the sample's local dates are read on.
const CLOCK: UtcOffset = UtcOffset::literal(5, 45, 0);

/// The sample chart, founded by [`Sample::founder`].
///
/// ```
/// use teistro_chart::sample;
///
/// let foundation = sample::foundation()?;
/// assert_eq!(foundation.place, sample::place());
/// assert!((0.0..360.0).contains(&foundation.lagna_deg));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`Founder::found_one`]; the test provider answers for the instant,
/// so in practice none.
pub fn foundation() -> Result<ChartFoundation, Error> {
    Ok(Sample::new()
        .founder()
        .found_one(INSTANT, &place(), ChartKind::Natal)?
        .value)
}
