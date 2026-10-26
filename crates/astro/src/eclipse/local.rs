//! An eclipse as one place sees it (`docs/03-design/eclipses.md` §4.5).
//!
//! A solar eclipse is read from the place itself: the Sun and the Moon of
//! the eclipse's own pair, less the station's geocentric position, so the
//! contacts are where the topocentric discs touch and the maximum is
//! where the magnitude is greatest. A lunar eclipse's contacts are the
//! same instants everywhere; a place adds only whether the Moon is up.
//! Each moment carries the body's topocentric altitude, and the view says
//! when, inside the eclipse, the body stands above a horizon convention:
//! what a sutak, observed only where the eclipse is seen, asks.

use serde::Serialize;
use teistro_core::catalogue::{Kind, write_in_full};
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::math;
use teistro_core::quantity::{JulianDay, Place, Ut1};
use teistro_port_ephemeris::{Body, Horizon};

use super::{
    COARSE_DAYS, CONTACT_REACH_DAYS, Eclipses, LunarEclipse, MOON_RADIUS, MOON_RADIUS_UMBRA, Pair,
    ShadowSource, SolarEclipse, SolarEclipseKind, Syzygy, TOLERANCE_DAYS, angle, sub, sun_radius,
};
use crate::iau::vector::{Vector3, pdp, pm, pn, sxp};
use crate::iau::{DAU, DEG2RAD, RAD2DEG};
use crate::rise_set::{AU_KM, Disc, EARTH_EQUATORIAL_RADIUS_KM, centre_altitude_deg};
use crate::scale::tt_of;
use crate::sky;
use crate::solve::{Caps, SolveError, minimum, refine};

/// How far either side of the greatest eclipse a place's own maximum is
/// looked for, days: the partial phase anywhere lasts under four hours
/// either side of it.
const LOCAL_REACH_DAYS: f64 = 0.2;

/// The step a visibility scan samples the body's altitude at, days: ten
/// minutes, under which neither the Sun nor the Moon rises and sets.
const VISIBILITY_STEP_DAYS: f64 = 10.0 / 1440.0;

/// One moment of an eclipse at a place.
///
/// ```
/// # use teistro_astro::eclipse::Eclipses;
/// # use teistro_astro::{Completion, DeltaTModel};
/// # use teistro_core::quantity::{JulianDay, Ut1};
/// # use teistro_core::settings::OverridePolicy;
/// # use teistro_port_ephemeris::TestProvider;
/// #
/// # // The test provider's sky: a geometry, not the real eclipses.
/// # let provider = TestProvider::new();
/// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let eclipses = Eclipses::new(&sky, DeltaTModel::TableThenModel);
/// # let j2000 = JulianDay::<Ut1>::J2000;
///
/// let lunar = eclipses.next_lunar(j2000)?;
/// # use teistro_core::quantity::Place;
/// # use teistro_port_ephemeris::Horizon;
/// #
/// # let kathmandu = Place::try_from_degrees(27.7172, 85.324, 1_400.0)?;
///
/// let view = eclipses.lunar_seen(&lunar, kathmandu, &Horizon::UPPER_LIMB_REFRACTION)?;
/// // The moment is the eclipse's own; the altitude is the place's.
/// assert_eq!(view.greatest.at, lunar.greatest);
/// assert!(view.greatest.altitude_deg.abs() <= 90.0);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct LocalMoment {
    /// When, UT1.
    pub at: JulianDay<Ut1>,
    /// The eclipsed body's centre above the place's horizon, degrees:
    /// topocentric and geometric, with no refraction.
    pub altitude_deg: f64,
}

/// The stretch of an eclipse its body stands above a horizon convention.
///
/// From the first instant to the last: a body that set and rose again
/// inside one eclipse, which only a place near a pole can see, is read as
/// up between.
///
/// ```
/// # use teistro_astro::eclipse::Eclipses;
/// # use teistro_astro::{Completion, DeltaTModel};
/// # use teistro_core::quantity::{JulianDay, Ut1};
/// # use teistro_core::settings::OverridePolicy;
/// # use teistro_port_ephemeris::TestProvider;
/// #
/// # // The test provider's sky: a geometry, not the real eclipses.
/// # let provider = TestProvider::new();
/// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let eclipses = Eclipses::new(&sky, DeltaTModel::TableThenModel);
/// # let j2000 = JulianDay::<Ut1>::J2000;
///
/// let lunar = eclipses.next_lunar(j2000)?;
/// # use teistro_core::quantity::Place;
/// # use teistro_port_ephemeris::Horizon;
/// #
/// # let kathmandu = Place::try_from_degrees(27.7172, 85.324, 1_400.0)?;
///
/// let view = eclipses.lunar_seen(&lunar, kathmandu, &Horizon::UPPER_LIMB_REFRACTION)?;
/// // What is seen lies inside the eclipse.
/// if let Some(seen) = view.seen {
///     assert!(view.p1.at.get() <= seen.from.get() && seen.to.get() <= view.p4.at.get());
/// }
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Visible {
    /// The first instant, UT1.
    pub from: JulianDay<Ut1>,
    /// The last instant, UT1.
    pub to: JulianDay<Ut1>,
}

/// A solar eclipse at one place.
///
/// ```
/// # use teistro_astro::eclipse::Eclipses;
/// # use teistro_astro::{Completion, DeltaTModel};
/// # use teistro_core::quantity::{JulianDay, Ut1};
/// # use teistro_core::settings::OverridePolicy;
/// # use teistro_port_ephemeris::TestProvider;
/// #
/// # // The test provider's sky: a geometry, not the real eclipses.
/// # let provider = TestProvider::new();
/// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # // The test provider places a body only as it is seen and every
/// # // body at one au, where the Moon's shadow never reaches the Earth:
/// # // this sky takes the placing it has and the Moon's mean distance.
/// # struct AsSeen<'a, 'p>(&'a Completion<'p, TestProvider>);
/// # impl teistro_astro::eclipse::ShadowSource for AsSeen<'_, '_> {
/// #     fn place(
/// #         &self,
/// #         body: teistro_port_ephemeris::Body,
/// #         ut1: JulianDay<Ut1>,
/// #         _: teistro_astro::eclipse::Seen,
/// #     ) -> Result<teistro_astro::sky::Apparent, teistro_core::error::Error> {
/// #         let apparent = teistro_astro::eclipse::Seen::Apparent;
/// #         let mut place = teistro_astro::eclipse::ShadowSource::place(self.0, body, ut1, apparent)?;
/// #         if body == teistro_port_ephemeris::Body::Moon {
/// #             place.distance_au = 0.002_57;
/// #         }
/// #         Ok(place)
/// #     }
/// #     fn describe(&self) -> String {
/// #         teistro_astro::eclipse::ShadowSource::describe(self.0)
/// #     }
/// # }
/// # let as_seen = AsSeen(&sky);
/// # let eclipses = Eclipses::new(&as_seen, DeltaTModel::TableThenModel);
/// # let j2000 = JulianDay::<Ut1>::J2000;
///
/// let solar = eclipses.next_solar(j2000)?;
/// # use teistro_core::quantity::Place;
/// # use teistro_port_ephemeris::Horizon;
/// #
/// # let kathmandu = Place::try_from_degrees(27.7172, 85.324, 1_400.0)?;
///
/// if let Some(view) = eclipses.solar_seen(&solar, kathmandu, &Horizon::UPPER_LIMB_REFRACTION)? {
///     // The maximum lies between the first contact and the fourth.
///     assert!(view.first.at.get() <= view.maximum.at.get() && view.maximum.at.get() <= view.fourth.at.get());
///     assert!(view.obscuration <= 1.0);
/// }
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SolarView {
    /// What the place sees at its maximum: partial, annular or total,
    /// never hybrid.
    pub kind: SolarEclipseKind,
    /// The fraction of the Sun's diameter the Moon covers at the
    /// maximum; in a total or annular phase, the ratio of the Moon's
    /// apparent diameter to the Sun's.
    pub magnitude: f64,
    /// The fraction of the Sun's disc the Moon covers at the maximum.
    pub obscuration: f64,
    /// The first contact: the discs touch outside, and the eclipse
    /// begins.
    pub first: LocalMoment,
    /// The second contact, when the eclipse is central here: totality or
    /// the ring begins.
    pub second: Option<LocalMoment>,
    /// The third contact, its end.
    pub third: Option<LocalMoment>,
    /// The fourth contact: the eclipse ends.
    pub fourth: LocalMoment,
    /// The maximum: the greatest magnitude.
    pub maximum: LocalMoment,
    /// When the Sun stands above the horizon convention between the
    /// first contact and the fourth; `None` when it never does.
    pub seen: Option<Visible>,
}

/// A lunar eclipse at one place: its contacts, each with the Moon's
/// altitude there.
///
/// ```
/// # use teistro_astro::eclipse::Eclipses;
/// # use teistro_astro::{Completion, DeltaTModel};
/// # use teistro_core::quantity::{JulianDay, Ut1};
/// # use teistro_core::settings::OverridePolicy;
/// # use teistro_port_ephemeris::TestProvider;
/// #
/// # // The test provider's sky: a geometry, not the real eclipses.
/// # let provider = TestProvider::new();
/// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let eclipses = Eclipses::new(&sky, DeltaTModel::TableThenModel);
/// # let j2000 = JulianDay::<Ut1>::J2000;
///
/// let lunar = eclipses.next_lunar(j2000)?;
/// # use teistro_core::quantity::Place;
/// # use teistro_port_ephemeris::Horizon;
/// #
/// # let kathmandu = Place::try_from_degrees(27.7172, 85.324, 1_400.0)?;
///
/// let view = eclipses.lunar_seen(&lunar, kathmandu, &Horizon::UPPER_LIMB_REFRACTION)?;
/// // The umbral part seen lies within the whole part seen.
/// if let (Some(umbral), Some(whole)) = (view.umbral_seen, view.seen) {
///     assert!(whole.from.get() <= umbral.from.get() && umbral.to.get() <= whole.to.get());
/// }
/// assert_eq!(view.u1.is_some(), lunar.contacts.u1.is_some());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct LunarView {
    /// The penumbra's first touch.
    pub p1: LocalMoment,
    /// The umbra's first touch, for a partial or total eclipse.
    pub u1: Option<LocalMoment>,
    /// Totality's beginning.
    pub u2: Option<LocalMoment>,
    /// The greatest eclipse.
    pub greatest: LocalMoment,
    /// Totality's end.
    pub u3: Option<LocalMoment>,
    /// The umbra's last touch.
    pub u4: Option<LocalMoment>,
    /// The penumbra's last touch.
    pub p4: LocalMoment,
    /// When the Moon stands above the horizon convention between the
    /// penumbra's first and last touch; `None` when it never does.
    pub seen: Option<Visible>,
    /// The same between the umbra's first and last touch: the part of the
    /// eclipse the eye sees, which a penumbral eclipse has none of and
    /// which the observances count by (*Dharmasindhu*: an eclipse's time
    /// lasts as long as it can be seen); `None` when the Moon is down
    /// throughout it, or the umbra never touches.
    pub umbral_seen: Option<Visible>,
}

/// A lunar eclipse and how one place sees it.
///
/// ```
/// # use teistro_astro::eclipse::Eclipses;
/// # use teistro_astro::{Completion, DeltaTModel};
/// # use teistro_core::quantity::{JulianDay, Ut1};
/// # use teistro_core::settings::OverridePolicy;
/// # use teistro_port_ephemeris::TestProvider;
/// #
/// # // The test provider's sky: a geometry, not the real eclipses.
/// # let provider = TestProvider::new();
/// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # // The test provider places a body only as it is seen and every
/// # // body at one au, where the Moon's shadow never reaches the Earth:
/// # // this sky takes the placing it has and the Moon's mean distance.
/// # struct AsSeen<'a, 'p>(&'a Completion<'p, TestProvider>);
/// # impl teistro_astro::eclipse::ShadowSource for AsSeen<'_, '_> {
/// #     fn place(
/// #         &self,
/// #         body: teistro_port_ephemeris::Body,
/// #         ut1: JulianDay<Ut1>,
/// #         _: teistro_astro::eclipse::Seen,
/// #     ) -> Result<teistro_astro::sky::Apparent, teistro_core::error::Error> {
/// #         let apparent = teistro_astro::eclipse::Seen::Apparent;
/// #         let mut place = teistro_astro::eclipse::ShadowSource::place(self.0, body, ut1, apparent)?;
/// #         if body == teistro_port_ephemeris::Body::Moon {
/// #             place.distance_au = 0.002_57;
/// #         }
/// #         Ok(place)
/// #     }
/// #     fn describe(&self) -> String {
/// #         teistro_astro::eclipse::ShadowSource::describe(self.0)
/// #     }
/// # }
/// # let as_seen = AsSeen(&sky);
/// # let eclipses = Eclipses::new(&as_seen, DeltaTModel::TableThenModel);
/// # let j2000 = JulianDay::<Ut1>::J2000;
///
/// # use teistro_core::quantity::Place;
/// # use teistro_port_ephemeris::Horizon;
/// #
/// # let kathmandu = Place::try_from_degrees(27.7172, 85.324, 1_400.0)?;
///
/// let here = eclipses.here_between(j2000, j2000.plus_days(365.25)?, kathmandu, &Horizon::UPPER_LIMB_REFRACTION)?;
/// let first = &here.lunar[0];
/// assert_eq!(first.here.greatest.at, first.eclipse.greatest);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct LunarHere {
    /// The eclipse, the same everywhere.
    pub eclipse: LunarEclipse,
    /// The place's view of it.
    pub here: LunarView,
    /// The vedha, the sutak: when food is given up before the eclipse
    /// until it ends as seen, under `panchanga.eclipse_vedha`. Read off
    /// the almanac's days, so the almanac fills it (`muhurta.md` §4.1.1);
    /// `None` where the place does not see the umbra, and here.
    pub vedha: Option<Interval>,
}

/// A solar eclipse and how one place sees it.
///
/// ```
/// # use teistro_astro::eclipse::Eclipses;
/// # use teistro_astro::{Completion, DeltaTModel};
/// # use teistro_core::quantity::{JulianDay, Ut1};
/// # use teistro_core::settings::OverridePolicy;
/// # use teistro_port_ephemeris::TestProvider;
/// #
/// # // The test provider's sky: a geometry, not the real eclipses.
/// # let provider = TestProvider::new();
/// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # // The test provider places a body only as it is seen and every
/// # // body at one au, where the Moon's shadow never reaches the Earth:
/// # // this sky takes the placing it has and the Moon's mean distance.
/// # struct AsSeen<'a, 'p>(&'a Completion<'p, TestProvider>);
/// # impl teistro_astro::eclipse::ShadowSource for AsSeen<'_, '_> {
/// #     fn place(
/// #         &self,
/// #         body: teistro_port_ephemeris::Body,
/// #         ut1: JulianDay<Ut1>,
/// #         _: teistro_astro::eclipse::Seen,
/// #     ) -> Result<teistro_astro::sky::Apparent, teistro_core::error::Error> {
/// #         let apparent = teistro_astro::eclipse::Seen::Apparent;
/// #         let mut place = teistro_astro::eclipse::ShadowSource::place(self.0, body, ut1, apparent)?;
/// #         if body == teistro_port_ephemeris::Body::Moon {
/// #             place.distance_au = 0.002_57;
/// #         }
/// #         Ok(place)
/// #     }
/// #     fn describe(&self) -> String {
/// #         teistro_astro::eclipse::ShadowSource::describe(self.0)
/// #     }
/// # }
/// # let as_seen = AsSeen(&sky);
/// # let eclipses = Eclipses::new(&as_seen, DeltaTModel::TableThenModel);
/// # let j2000 = JulianDay::<Ut1>::J2000;
///
/// # use teistro_core::quantity::Place;
/// # use teistro_port_ephemeris::Horizon;
/// #
/// # let kathmandu = Place::try_from_degrees(27.7172, 85.324, 1_400.0)?;
///
/// let here = eclipses.here_between(j2000, j2000.plus_days(365.25)?, kathmandu, &Horizon::UPPER_LIMB_REFRACTION)?;
/// // A view, when there is one, is of its own eclipse.
/// for solar in &here.solar {
///     if let Some(view) = solar.here {
///         assert!((view.maximum.at.get() - solar.eclipse.greatest.get()).abs() < 0.2);
///     }
/// }
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SolarHere {
    /// The eclipse, the same everywhere.
    pub eclipse: SolarEclipse,
    /// The place's view of it; `None` when the Moon's disc never touches
    /// the Sun's from there.
    pub here: Option<SolarView>,
    /// The vedha, the sutak, as a lunar eclipse's
    /// ([`LunarHere::vedha`]); `None` where the place does not see it.
    pub vedha: Option<Interval>,
}

/// Every eclipse of a window, each with one place's view of it: what an
/// almanac prints for its place.
///
/// ```
/// use teistro_astro::eclipse::EclipsesHere;
///
/// // None at all: nothing to see.
/// let none = EclipsesHere::default();
/// assert!(!none.any_seen());
/// assert!(none.lunar.is_empty() && none.solar.is_empty());
/// ```
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct EclipsesHere {
    /// The lunar eclipses, in order.
    pub lunar: Vec<LunarHere>,
    /// The solar eclipses, in order.
    pub solar: Vec<SolarHere>,
}

impl EclipsesHere {
    /// Where the eclipses name catalogue members, and of which kind: a
    /// dotted path through the value, a list reaching each element. What
    /// a boundary section writes in full; `crates/sdk/tests/eclipses.rs`
    /// holds it to serde, both ways. The shadow rule is a setting's
    /// value, written as it is.
    ///
    /// ```
    /// use teistro_astro::eclipse::EclipsesHere;
    /// use teistro_core::catalogue::Kind;
    ///
    /// assert!(EclipsesHere::MEMBERS.contains(&("lunar.eclipse.kind", Kind::LunarEclipseKind)));
    /// ```
    pub const MEMBERS: [(&'static str, Kind); 3] = [
        ("lunar.eclipse.kind", Kind::LunarEclipseKind),
        ("solar.eclipse.kind", Kind::SolarEclipseKind),
        ("solar.here.kind", Kind::SolarEclipseKind),
    ];

    /// The eclipses as JSON, every catalogue member written as its full
    /// key where [`EclipsesHere::MEMBERS`] says: what a boundary section
    /// carries and seals.
    ///
    /// ```
    /// # use teistro_astro::eclipse::Eclipses;
    /// # use teistro_astro::{Completion, DeltaTModel};
    /// # use teistro_core::quantity::{JulianDay, Ut1};
    /// # use teistro_core::settings::OverridePolicy;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // The test provider's sky: a geometry, not the real eclipses.
    /// # let provider = TestProvider::new();
    /// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// # // The test provider places a body only as it is seen and every
    /// # // body at one au, where the Moon's shadow never reaches the Earth:
    /// # // this sky takes the placing it has and the Moon's mean distance.
    /// # struct AsSeen<'a, 'p>(&'a Completion<'p, TestProvider>);
    /// # impl teistro_astro::eclipse::ShadowSource for AsSeen<'_, '_> {
    /// #     fn place(
    /// #         &self,
    /// #         body: teistro_port_ephemeris::Body,
    /// #         ut1: JulianDay<Ut1>,
    /// #         _: teistro_astro::eclipse::Seen,
    /// #     ) -> Result<teistro_astro::sky::Apparent, teistro_core::error::Error> {
    /// #         let apparent = teistro_astro::eclipse::Seen::Apparent;
    /// #         let mut place = teistro_astro::eclipse::ShadowSource::place(self.0, body, ut1, apparent)?;
    /// #         if body == teistro_port_ephemeris::Body::Moon {
    /// #             place.distance_au = 0.002_57;
    /// #         }
    /// #         Ok(place)
    /// #     }
    /// #     fn describe(&self) -> String {
    /// #         teistro_astro::eclipse::ShadowSource::describe(self.0)
    /// #     }
    /// # }
    /// # let as_seen = AsSeen(&sky);
    /// # let eclipses = Eclipses::new(&as_seen, DeltaTModel::TableThenModel);
    /// # let j2000 = JulianDay::<Ut1>::J2000;
    ///
    /// # use teistro_core::quantity::Place;
    /// # use teistro_port_ephemeris::Horizon;
    /// #
    /// # let kathmandu = Place::try_from_degrees(27.7172, 85.324, 1_400.0)?;
    ///
    /// let here = eclipses.here_between(j2000, j2000.plus_days(365.25)?, kathmandu, &Horizon::UPPER_LIMB_REFRACTION)?;
    /// let written = here.in_full()?;
    /// // The kind written as its full key: its catalogue kind, then the member.
    /// let kind = written["lunar"][0]["eclipse"]["kind"].as_str().unwrap_or_default();
    /// assert!(kind.starts_with("lunar_eclipse_kind."), "{kind}");
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INTERNAL` if the eclipses do not serialise, which a value this
    /// crate built cannot do.
    pub fn in_full(&self) -> Result<serde_json::Value, Error> {
        let mut value = serde_json::to_value(self)
            .map_err(|error| Error::internal(format!("an eclipse did not serialise: {error}")))?;
        for (path, kind) in Self::MEMBERS {
            write_in_full(&mut value, path, kind);
        }
        Ok(value)
    }

    /// Whether the place sees any of them.
    ///
    /// ```
    /// # use teistro_astro::eclipse::Eclipses;
    /// # use teistro_astro::{Completion, DeltaTModel};
    /// # use teistro_core::quantity::{JulianDay, Ut1};
    /// # use teistro_core::settings::OverridePolicy;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // The test provider's sky: a geometry, not the real eclipses.
    /// # let provider = TestProvider::new();
    /// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// # // The test provider places a body only as it is seen and every
    /// # // body at one au, where the Moon's shadow never reaches the Earth:
    /// # // this sky takes the placing it has and the Moon's mean distance.
    /// # struct AsSeen<'a, 'p>(&'a Completion<'p, TestProvider>);
    /// # impl teistro_astro::eclipse::ShadowSource for AsSeen<'_, '_> {
    /// #     fn place(
    /// #         &self,
    /// #         body: teistro_port_ephemeris::Body,
    /// #         ut1: JulianDay<Ut1>,
    /// #         _: teistro_astro::eclipse::Seen,
    /// #     ) -> Result<teistro_astro::sky::Apparent, teistro_core::error::Error> {
    /// #         let apparent = teistro_astro::eclipse::Seen::Apparent;
    /// #         let mut place = teistro_astro::eclipse::ShadowSource::place(self.0, body, ut1, apparent)?;
    /// #         if body == teistro_port_ephemeris::Body::Moon {
    /// #             place.distance_au = 0.002_57;
    /// #         }
    /// #         Ok(place)
    /// #     }
    /// #     fn describe(&self) -> String {
    /// #         teistro_astro::eclipse::ShadowSource::describe(self.0)
    /// #     }
    /// # }
    /// # let as_seen = AsSeen(&sky);
    /// # let eclipses = Eclipses::new(&as_seen, DeltaTModel::TableThenModel);
    /// # let j2000 = JulianDay::<Ut1>::J2000;
    ///
    /// # use teistro_core::quantity::Place;
    /// # use teistro_port_ephemeris::Horizon;
    /// #
    /// # let kathmandu = Place::try_from_degrees(27.7172, 85.324, 1_400.0)?;
    ///
    /// let here = eclipses.here_between(j2000, j2000.plus_days(365.25)?, kathmandu, &Horizon::UPPER_LIMB_REFRACTION)?;
    /// assert_eq!(here.any_seen(), here.lunar.iter().any(|lunar| lunar.here.seen.is_some()) || here.solar.iter().any(|solar| solar.here.is_some_and(|view| view.seen.is_some())));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn any_seen(&self) -> bool {
        self.lunar.iter().any(|e| e.here.seen.is_some())
            || self
                .solar
                .iter()
                .any(|e| e.here.is_some_and(|here| here.seen.is_some()))
    }
}

/// The pair from a place: each body's vector from the station, and the
/// place's zenith, all equatorial of date, Earth radii.
struct Topocentric {
    sun: Vector3,
    moon: Vector3,
    zenith: Vector3,
}

impl Topocentric {
    fn altitude_deg(&self, body: Body) -> f64 {
        let (_, unit) = pn(self.of(body));
        math::asin(pdp(&unit, &self.zenith).clamp(-1.0, 1.0)) * RAD2DEG
    }

    fn of(&self, body: Body) -> &Vector3 {
        if body == Body::Sun {
            &self.sun
        } else {
            &self.moon
        }
    }

    /// The centres' separation and the two semidiameters, radians, the
    /// Moon's at `moon_radius` Earth radii.
    fn discs(&self, moon_radius: f64) -> (f64, f64, f64) {
        let sun = math::asin(sun_radius() / pm(&self.sun));
        let moon = math::asin(moon_radius / pm(&self.moon));
        (angle(&self.sun, &self.moon), sun, moon)
    }
}

/// The fraction of a disc of radius `big` covered by one of radius
/// `small` whose centre is `d` away: the lens of two circles.
fn obscuration(d: f64, big: f64, small: f64) -> f64 {
    if d >= big + small {
        return 0.0;
    }
    if d <= (big - small).abs() {
        let ratio = small.min(big) / big;
        return ratio * ratio;
    }
    let part = |r: f64, other: f64| {
        r * r * math::acos(((d * d + r * r - other * other) / (2.0 * d * r)).clamp(-1.0, 1.0))
    };
    let kite = 0.5
        * ((-d + small + big) * (d + small - big) * (d - small + big) * (d + small + big))
            .max(0.0)
            .sqrt();
    (part(big, small) + part(small, big) - kite) / (core::f64::consts::PI * big * big)
}

impl<S: ShadowSource + ?Sized> Eclipses<'_, S> {
    /// A solar eclipse as `place` sees it, under `horizon` for when the
    /// Sun is up; `None` when the Moon's disc never touches the Sun's
    /// from there.
    ///
    /// The contacts are geometric, the Earth taken as transparent: a
    /// contact below the horizon is reported with its negative altitude,
    /// a place on the night side can have all four, and
    /// [`SolarView::seen`] says what of the eclipse is above the horizon,
    /// `None` when nothing is, which is whether the place sees it.
    ///
    /// ```
    /// # use teistro_astro::eclipse::Eclipses;
    /// # use teistro_astro::{Completion, DeltaTModel};
    /// # use teistro_core::quantity::{JulianDay, Ut1};
    /// # use teistro_core::settings::OverridePolicy;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // The test provider's sky: a geometry, not the real eclipses.
    /// # let provider = TestProvider::new();
    /// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// # // The test provider places a body only as it is seen and every
    /// # // body at one au, where the Moon's shadow never reaches the Earth:
    /// # // this sky takes the placing it has and the Moon's mean distance.
    /// # struct AsSeen<'a, 'p>(&'a Completion<'p, TestProvider>);
    /// # impl teistro_astro::eclipse::ShadowSource for AsSeen<'_, '_> {
    /// #     fn place(
    /// #         &self,
    /// #         body: teistro_port_ephemeris::Body,
    /// #         ut1: JulianDay<Ut1>,
    /// #         _: teistro_astro::eclipse::Seen,
    /// #     ) -> Result<teistro_astro::sky::Apparent, teistro_core::error::Error> {
    /// #         let apparent = teistro_astro::eclipse::Seen::Apparent;
    /// #         let mut place = teistro_astro::eclipse::ShadowSource::place(self.0, body, ut1, apparent)?;
    /// #         if body == teistro_port_ephemeris::Body::Moon {
    /// #             place.distance_au = 0.002_57;
    /// #         }
    /// #         Ok(place)
    /// #     }
    /// #     fn describe(&self) -> String {
    /// #         teistro_astro::eclipse::ShadowSource::describe(self.0)
    /// #     }
    /// # }
    /// # let as_seen = AsSeen(&sky);
    /// # let eclipses = Eclipses::new(&as_seen, DeltaTModel::TableThenModel);
    /// # let j2000 = JulianDay::<Ut1>::J2000;
    ///
    /// let solar = eclipses.next_solar(j2000)?;
    /// # use teistro_core::quantity::Place;
    /// # use teistro_port_ephemeris::Horizon;
    /// #
    /// # let kathmandu = Place::try_from_degrees(27.7172, 85.324, 1_400.0)?;
    ///
    /// // Seen or not, a view's contacts carry the Sun's altitude at Kathmandu.
    /// if let Some(view) = eclipses.solar_seen(&solar, kathmandu, &Horizon::UPPER_LIMB_REFRACTION)? {
    ///     assert_eq!(view.seen.is_some(), view.seen.is_some_and(|seen| seen.from.get() <= seen.to.get()));
    /// }
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// An instant the source cannot answer, or a Delta T model that
    /// cannot.
    pub fn solar_seen(
        &self,
        eclipse: &SolarEclipse,
        place: Place,
        horizon: &Horizon,
    ) -> Result<Option<SolarView>, Error> {
        let greatest = eclipse.greatest.get();
        let sought = || format!("the solar eclipse's maximum at {place}");
        // The maximum is the greatest magnitude, not the least separation:
        // the discs' topocentric sizes change as the bodies climb, and for
        // a shallow eclipse the two instants are seconds apart.
        let shortfall = |t: f64| {
            self.topocentric(t, Syzygy::New, place).map(|p| {
                let (d, sun, moon) = p.discs(MOON_RADIUS);
                (d - sun - moon) / (2.0 * sun)
            })
        };
        let coarse = minimum(
            shortfall,
            greatest - LOCAL_REACH_DAYS,
            greatest + LOCAL_REACH_DAYS,
            COARSE_DAYS,
            Caps::DEFAULT,
        )
        .map_err(|error| error.into_error(sought))?;
        let fine = minimum(
            shortfall,
            coarse.instant - coarse.width,
            coarse.instant + coarse.width,
            TOLERANCE_DAYS,
            Caps::DEFAULT,
        )
        .map_err(|error| error.into_error(sought))?;
        let at = fine.instant;
        let there = self.topocentric(at, Syzygy::New, place)?;
        let (d, sun, moon) = there.discs(MOON_RADIUS);
        if d >= sun + moon {
            return Ok(None);
        }
        let (_, _, moon_inner) = there.discs(MOON_RADIUS_UMBRA);
        let central = d < (moon_inner - sun).abs();
        let kind = match (central, moon_inner > sun) {
            (true, true) => SolarEclipseKind::Total,
            (true, false) => SolarEclipseKind::Annular,
            (false, _) => SolarEclipseKind::Partial,
        };
        // A contact: where the separation meets the discs' sum (outer) or
        // difference (inner), on one side of the maximum.
        let contact = |inner: bool, after: bool| -> Result<LocalMoment, Error> {
            let gap = |t: f64| {
                let here = self.topocentric(t, Syzygy::New, place)?;
                let gap = if inner {
                    let (d, sun, moon) = here.discs(MOON_RADIUS_UMBRA);
                    d - (moon - sun).abs()
                } else {
                    let (d, sun, moon) = here.discs(MOON_RADIUS);
                    d - (sun + moon)
                };
                Ok::<_, Error>(if after { gap } else { -gap })
            };
            let (lo, hi) = if after {
                (at, at + CONTACT_REACH_DAYS)
            } else {
                (at - CONTACT_REACH_DAYS, at)
            };
            let found = refine(gap, lo, hi, TOLERANCE_DAYS, Caps::DEFAULT).map_err(
                |error: SolveError<Error>| {
                    error.into_error(|| format!("the solar eclipse's contact at {place}"))
                },
            )?;
            self.moment(found.instant, Syzygy::New, place, Body::Sun)
        };
        let first = contact(false, false)?;
        let fourth = contact(false, true)?;
        Ok(Some(SolarView {
            kind,
            // Inside a central phase the Moon's disc is wholly over or
            // inside the Sun's, and the magnitude is the ratio of the two
            // diameters, as the global one is at greatest eclipse.
            magnitude: if central {
                moon_inner / sun
            } else {
                (sun + moon - d) / (2.0 * sun)
            },
            obscuration: obscuration(d, sun, moon),
            first,
            second: central.then(|| contact(true, false)).transpose()?,
            third: central.then(|| contact(true, true)).transpose()?,
            fourth,
            maximum: LocalMoment {
                at: JulianDay::try_new(at)?,
                altitude_deg: there.altitude_deg(Body::Sun),
            },
            seen: self.seen(Body::Sun, (first.at, fourth.at), place, horizon)?,
        }))
    }

    /// A lunar eclipse as `place` sees it, under `horizon` for when the
    /// Moon is up.
    ///
    /// ```
    /// # use teistro_astro::eclipse::Eclipses;
    /// # use teistro_astro::{Completion, DeltaTModel};
    /// # use teistro_core::quantity::{JulianDay, Ut1};
    /// # use teistro_core::settings::OverridePolicy;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // The test provider's sky: a geometry, not the real eclipses.
    /// # let provider = TestProvider::new();
    /// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// # let eclipses = Eclipses::new(&sky, DeltaTModel::TableThenModel);
    /// # let j2000 = JulianDay::<Ut1>::J2000;
    ///
    /// let lunar = eclipses.next_lunar(j2000)?;
    /// # use teistro_core::quantity::Place;
    /// # use teistro_port_ephemeris::Horizon;
    /// #
    /// # let kathmandu = Place::try_from_degrees(27.7172, 85.324, 1_400.0)?;
    ///
    /// let view = eclipses.lunar_seen(&lunar, kathmandu, &Horizon::UPPER_LIMB_REFRACTION)?;
    /// assert_eq!((view.p1.at, view.p4.at), (lunar.contacts.p1, lunar.contacts.p4));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`Eclipses::solar_seen`].
    pub fn lunar_seen(
        &self,
        eclipse: &LunarEclipse,
        place: Place,
        horizon: &Horizon,
    ) -> Result<LunarView, Error> {
        let moment = |at: JulianDay<Ut1>| self.moment(at.get(), Syzygy::Full, place, Body::Moon);
        let optional = |at: Option<JulianDay<Ut1>>| at.map(moment).transpose();
        let c = &eclipse.contacts;
        Ok(LunarView {
            p1: moment(c.p1)?,
            u1: optional(c.u1)?,
            u2: optional(c.u2)?,
            greatest: moment(eclipse.greatest)?,
            u3: optional(c.u3)?,
            u4: optional(c.u4)?,
            p4: moment(c.p4)?,
            seen: self.seen(Body::Moon, (c.p1, c.p4), place, horizon)?,
            umbral_seen: match (c.u1, c.u4) {
                (Some(u1), Some(u4)) => self.seen(Body::Moon, (u1, u4), place, horizon)?,
                _ => None,
            },
        })
    }

    /// Every eclipse whose greatest moment falls in `[from, to)`, each
    /// with `place`'s view of it under `horizon`.
    ///
    /// ```
    /// # use teistro_astro::eclipse::Eclipses;
    /// # use teistro_astro::{Completion, DeltaTModel};
    /// # use teistro_core::quantity::{JulianDay, Ut1};
    /// # use teistro_core::settings::OverridePolicy;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # // The test provider's sky: a geometry, not the real eclipses.
    /// # let provider = TestProvider::new();
    /// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// # // The test provider places a body only as it is seen and every
    /// # // body at one au, where the Moon's shadow never reaches the Earth:
    /// # // this sky takes the placing it has and the Moon's mean distance.
    /// # struct AsSeen<'a, 'p>(&'a Completion<'p, TestProvider>);
    /// # impl teistro_astro::eclipse::ShadowSource for AsSeen<'_, '_> {
    /// #     fn place(
    /// #         &self,
    /// #         body: teistro_port_ephemeris::Body,
    /// #         ut1: JulianDay<Ut1>,
    /// #         _: teistro_astro::eclipse::Seen,
    /// #     ) -> Result<teistro_astro::sky::Apparent, teistro_core::error::Error> {
    /// #         let apparent = teistro_astro::eclipse::Seen::Apparent;
    /// #         let mut place = teistro_astro::eclipse::ShadowSource::place(self.0, body, ut1, apparent)?;
    /// #         if body == teistro_port_ephemeris::Body::Moon {
    /// #             place.distance_au = 0.002_57;
    /// #         }
    /// #         Ok(place)
    /// #     }
    /// #     fn describe(&self) -> String {
    /// #         teistro_astro::eclipse::ShadowSource::describe(self.0)
    /// #     }
    /// # }
    /// # let as_seen = AsSeen(&sky);
    /// # let eclipses = Eclipses::new(&as_seen, DeltaTModel::TableThenModel);
    /// # let j2000 = JulianDay::<Ut1>::J2000;
    ///
    /// # use teistro_core::quantity::Place;
    /// # use teistro_port_ephemeris::Horizon;
    /// #
    /// # let kathmandu = Place::try_from_degrees(27.7172, 85.324, 1_400.0)?;
    ///
    /// // The eclipses the global search finds, each with Kathmandu's view.
    /// let to = j2000.plus_days(365.25)?;
    /// let here = eclipses.here_between(j2000, to, kathmandu, &Horizon::UPPER_LIMB_REFRACTION)?;
    /// assert_eq!(here.lunar.len(), eclipses.lunar_between(j2000, to)?.len());
    /// assert_eq!(here.solar.len(), eclipses.solar_between(j2000, to)?.len());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`Eclipses::lunar_between`] and [`Eclipses::solar_seen`].
    pub fn here_between(
        &self,
        from: JulianDay<Ut1>,
        to: JulianDay<Ut1>,
        place: Place,
        horizon: &Horizon,
    ) -> Result<EclipsesHere, Error> {
        let lunar = self
            .lunar_between(from, to)?
            .into_iter()
            .map(|eclipse| {
                Ok(LunarHere {
                    here: self.lunar_seen(&eclipse, place, horizon)?,
                    eclipse,
                    vedha: None,
                })
            })
            .collect::<Result<_, Error>>()?;
        let solar = self
            .solar_between(from, to)?
            .into_iter()
            .map(|eclipse| {
                Ok(SolarHere {
                    here: self.solar_seen(&eclipse, place, horizon)?,
                    eclipse,
                    vedha: None,
                })
            })
            .collect::<Result<_, Error>>()?;
        Ok(EclipsesHere { lunar, solar })
    }

    /// The pair from a place at an instant.
    fn topocentric(&self, at: f64, syzygy: Syzygy, place: Place) -> Result<Topocentric, Error> {
        let Pair { sun, moon } = self.pair(at, syzygy)?;
        let ut1 = JulianDay::<Ut1>::try_new(at)?;
        let (tt, _) = tt_of(ut1, self.delta_t)?;
        let station = sxp(
            DAU / 1000.0 / EARTH_EQUATORIAL_RADIUS_KM,
            &sky::observer(place, ut1, tt).position_au,
        );
        // The geodetic vertical: the place's latitude, at the station's
        // own right ascension.
        let (sin_lat, cos_lat) = math::sin_cos(place.latitude.get() * DEG2RAD);
        let ra = math::atan2(station[1], station[0]);
        let (sin_ra, cos_ra) = math::sin_cos(ra);
        Ok(Topocentric {
            sun: sub(&sun, &station),
            moon: sub(&moon, &station),
            zenith: [cos_lat * cos_ra, cos_lat * sin_ra, sin_lat],
        })
    }

    fn moment(
        &self,
        at: f64,
        syzygy: Syzygy,
        place: Place,
        body: Body,
    ) -> Result<LocalMoment, Error> {
        Ok(LocalMoment {
            at: JulianDay::try_new(at)?,
            altitude_deg: self.topocentric(at, syzygy, place)?.altitude_deg(body),
        })
    }

    /// The body's height above a horizon convention's line at an
    /// instant, degrees: positive when it is up.
    fn above(&self, body: Body, at: f64, place: Place, horizon: &Horizon) -> Result<f64, Error> {
        let syzygy = if body == Body::Sun {
            Syzygy::New
        } else {
            Syzygy::Full
        };
        let here = self.topocentric(at, syzygy, place)?;
        let distance_au = pm(here.of(body)) * EARTH_EQUATORIAL_RADIUS_KM / AU_KM;
        let disc = Disc::of(body, distance_au);
        // The convention's line is a geocentric altitude of the centre;
        // the altitude here is topocentric, so the parallax is taken off.
        let line = centre_altitude_deg(horizon, &disc, place.altitude) - disc.parallax_deg;
        Ok(here.altitude_deg(body) - line)
    }

    /// The first and last instants in `span` the body stands above the
    /// horizon convention: sampled every ten minutes, each change of side
    /// refined to the search's tolerance.
    fn seen(
        &self,
        body: Body,
        (from, to): (JulianDay<Ut1>, JulianDay<Ut1>),
        place: Place,
        horizon: &Horizon,
    ) -> Result<Option<Visible>, Error> {
        let (from, to) = (from.get(), to.get());
        let height = |t: f64| self.above(body, t, place, horizon);
        let steps = ((to - from) / VISIBILITY_STEP_DAYS).ceil().max(1.0);
        // Never more than a few dozen: an eclipse lasts hours.
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a positive count of ten-minute steps in a few hours"
        )]
        let count = steps as usize;
        let step = (to - from) / steps;
        let crossing = |lo: f64, hi: f64| -> Result<f64, Error> {
            let rising = height(lo)? < height(hi)?;
            let signed = |t: f64| height(t).map(|h| if rising { h } else { -h });
            refine(signed, lo, hi, TOLERANCE_DAYS, Caps::DEFAULT)
                .map(|found| found.instant)
                .map_err(|error: SolveError<Error>| {
                    error.into_error(|| format!("the {}'s horizon at {place}", body.key()))
                })
        };
        let mut first = None;
        let mut last = None;
        let mut previous = (from, height(from)?);
        if previous.1 > 0.0 {
            first = Some(from);
        }
        for n in 1..=count {
            #[allow(clippy::cast_precision_loss, reason = "a few dozen steps")]
            let t = if n == count {
                to
            } else {
                from + step * n as f64
            };
            let h = height(t)?;
            if (h > 0.0) != (previous.1 > 0.0) {
                let at = crossing(previous.0, t)?;
                if h > 0.0 {
                    first.get_or_insert(at);
                } else {
                    last = Some(at);
                }
            }
            previous = (t, h);
        }
        let Some(first) = first else {
            return Ok(None);
        };
        // Up at the end, or down since the last setting.
        let last = if previous.1 > 0.0 {
            to
        } else {
            last.unwrap_or(to)
        };
        Ok(Some(Visible {
            from: JulianDay::try_new(first)?,
            to: JulianDay::try_new(last)?,
        }))
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::obscuration;

    #[test]
    fn obscuration_is_the_lens_of_two_discs() {
        // Apart, inside and concentric.
        assert!(obscuration(3.0, 1.0, 1.0).abs() < 1e-15);
        assert!((obscuration(0.0, 1.0, 0.5) - 0.25).abs() < 1e-15);
        assert!((obscuration(0.0, 1.0, 1.2) - 1.0).abs() < 1e-15);
        // Two equal discs a radius apart overlap by 2π/3 − √3/2 of π.
        let lens = 2.0 * core::f64::consts::PI / 3.0 - 3.0_f64.sqrt() / 2.0;
        assert!((obscuration(1.0, 1.0, 1.0) - lens / core::f64::consts::PI).abs() < 1e-12);
    }
}
