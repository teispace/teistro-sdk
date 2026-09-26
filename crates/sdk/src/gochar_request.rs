//! Gochar through the façade: the transits at many instants, each read
//! from the natal chart's reference sign (`03-design/gochar.md`).

use teistro_chart::foundation::ChartFoundation;
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_gochar::{GRAHAS, GocharReading, GocharRules, Transit, gochar};
use teistro_serial::Document;

/// What gochar counts the transits from (crux C139).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum GocharFrom {
    /// The natal Moon's sign, as Phaladeepika ch. 26 v. 1 counts them: "of
    /// all the lagnas, the Moon's".
    #[default]
    Moon,
    /// The natal lagna's sign, a second reference some software offers and
    /// the verse does not.
    Lagna,
}

/// The instants to read the transits at, and what to count them from.
///
/// ```
/// use teistro::quantity::{JulianDay, Utc};
/// use teistro::{GocharFrom, GocharRequest};
///
/// // A year of daily snapshots, counted from the natal Moon by default.
/// let days: Vec<_> = (0..365).map(|d| JulianDay::<Utc>::literal(2_460_676.5 + f64::from(d))).collect();
/// let year = GocharRequest::over(days);
/// assert_eq!(year.from(), GocharFrom::Moon);
/// let from_the_lagna = GocharRequest::at(JulianDay::literal(2_460_676.5)).counted_from(GocharFrom::Lagna);
/// assert_eq!(from_the_lagna.instants().len(), 1);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct GocharRequest {
    instants: Vec<JulianDay<Utc>>,
    from: GocharFrom,
}

impl GocharRequest {
    /// The transits at one instant.
    #[must_use]
    pub fn at(instant: JulianDay<Utc>) -> GocharRequest {
        GocharRequest::over([instant])
    }

    /// The transits at each of many instants, in the order given: one
    /// founding of the transit chart each, the natal reference read once.
    #[must_use]
    pub fn over(instants: impl IntoIterator<Item = JulianDay<Utc>>) -> GocharRequest {
        GocharRequest {
            instants: instants.into_iter().collect(),
            from: GocharFrom::Moon,
        }
    }

    /// Counted from another reference than the natal Moon.
    #[must_use]
    pub const fn counted_from(mut self, from: GocharFrom) -> GocharRequest {
        self.from = from;
        self
    }

    /// The instants asked for.
    #[must_use]
    pub fn instants(&self) -> &[JulianDay<Utc>] {
        &self.instants
    }

    /// What the transits are counted from.
    #[must_use]
    pub const fn from(&self) -> GocharFrom {
        self.from
    }
}

/// A sign from a sidereal longitude.
fn sign_of(longitude_deg: f64) -> Rashi {
    Transit::at_longitude(longitude_deg).sign
}

/// The natal chart's reference sign for `from`.
pub(crate) fn reference(natal: &ChartFoundation, from: GocharFrom) -> Result<Rashi, Error> {
    match from {
        GocharFrom::Lagna => Rashi::from_id(u16::from(natal.lagna_sign_index()))
            .ok_or_else(|| Error::internal("a lagna in no sign")),
        _ => natal
            .graha(Graha::Moon)
            .map(|moon| sign_of(moon.longitude_deg))
            .ok_or_else(|| Error::internal("a founded chart places the Moon")),
    }
}

/// One transit chart's gochar from `reference`.
pub(crate) fn reading(
    transit: &Document,
    reference: Rashi,
    rules: GocharRules,
) -> Result<GocharReading, Error> {
    let mut transits = [Transit::new(reference, 0.0); 9];
    for (slot, graha) in transits.iter_mut().zip(GRAHAS) {
        let at = transit
            .foundation
            .graha(graha)
            .ok_or_else(|| Error::internal(format!("a founded chart places {}", graha.key())))?;
        *slot = Transit::at_longitude(at.longitude_deg);
    }
    Ok(gochar(reference, &transits, rules))
}
