//! A whole divisional chart of a founded moment, and the mixed-chart
//! axis.
//!
//! `03-design/varga-kernel.md` settles that
//! `(varga_for_planets, varga_for_lagna)` is a property of a **chart
//! context** and not of a varga: a caller may want the grahas in the
//! navamsha and the lagna still in the rashi, and neither chart changes
//! because of it. So it is a pair the caller sets, it rides on the
//! request, and the value carries it back.

use serde::{Deserialize, Serialize};
use teistro_chart::foundation::ChartFoundation;
use teistro_core::angle::Nas;
use teistro_core::catalogue::{Graha, Rashi, Varga};
use teistro_core::error::Error;
use teistro_core::quantity::Degrees;

use crate::evaluate::{Placement, place};
use crate::scheme::Scheme;

/// Which chart the grahas are read in, and which the lagna is.
///
/// Defaulting to the rashi for both, which is the plain chart.
///
/// ```
/// use teistro_core::catalogue::Varga;
/// use teistro_vargas::{Axis, Scheme};
///
/// assert_eq!(Axis::default(), Axis::of(Varga::D1));
/// assert_eq!(Axis::of(Varga::D9).lagna, Scheme::of(Varga::D9));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Axis {
    /// The chart the grahas are placed in.
    pub grahas: Scheme,
    /// The chart the lagna is placed in.
    pub lagna: Scheme,
}

impl Axis {
    /// One chart for both, which is what almost every caller wants.
    ///
    /// ```
    /// use teistro_core::catalogue::Varga;
    /// use teistro_vargas::Axis;
    ///
    /// let d9 = Axis::of(Varga::D9);
    /// assert_eq!(d9.grahas, d9.lagna);
    /// ```
    #[must_use]
    pub fn of(varga: Varga) -> Axis {
        let scheme = Scheme::of(varga);
        Axis {
            grahas: scheme,
            lagna: scheme,
        }
    }

    /// The grahas in one chart and the lagna in another.
    ///
    /// ```
    /// use teistro_core::catalogue::Varga;
    /// use teistro_vargas::{Axis, Scheme};
    ///
    /// // The grahas in the navamsha, the lagna still in the rashi.
    /// let axis = Axis::mixed(Scheme::of(Varga::D9), Scheme::of(Varga::D1));
    /// assert_ne!(axis, Axis::of(Varga::D9));
    /// ```
    #[must_use]
    pub const fn mixed(grahas: Scheme, lagna: Scheme) -> Axis {
        Axis { grahas, lagna }
    }

    /// What the axis is called, for a stamp: `D9` when both agree, and
    /// `D9/D1` when they do not.
    ///
    /// ```
    /// use teistro_core::catalogue::Varga;
    /// use teistro_vargas::{Axis, Scheme};
    ///
    /// assert_eq!(Axis::of(Varga::D9).key(), "D9");
    /// assert_eq!(Axis::mixed(Scheme::of(Varga::D9), Scheme::of(Varga::D1)).key(), "D9/D1");
    /// ```
    #[must_use]
    pub fn key(&self) -> String {
        if self.grahas == self.lagna {
            self.grahas.key()
        } else {
            format!("{}/{}", self.grahas.key(), self.lagna.key())
        }
    }
}

impl Default for Axis {
    fn default() -> Axis {
        Axis::of(Varga::D1)
    }
}

/// Where one graha stands in a divisional chart.
///
/// ```
/// use teistro_core::catalogue::{Graha, Varga};
/// use teistro_vargas::{Axis, chart};
///
/// let d9 = chart(&teistro_chart::sample::foundation()?, Axis::of(Varga::D9))?;
/// assert_eq!(d9.grahas[0].graha, Graha::Sun);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GrahaPlacement {
    /// Which graha.
    pub graha: Graha,
    /// Where it stands.
    pub at: Placement,
}

/// One divisional chart of one founded moment.
///
/// ```
/// use teistro_core::catalogue::Varga;
/// use teistro_vargas::{Axis, chart};
///
/// let foundation = teistro_chart::sample::foundation()?;
/// let d9 = chart(&foundation, Axis::of(Varga::D9))?;
/// assert_eq!(d9.grahas.len(), foundation.grahas.len());
/// assert_eq!(d9.lagna.rashi as u8, foundation.lagna_sign_index());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VargaChart {
    /// Which chart, or which pair of them.
    pub axis: Axis,
    /// The lagna's place, in the chart the axis names for it.
    pub lagna: Placement,
    /// Every graha the foundation carries, in its order.
    pub grahas: Vec<GrahaPlacement>,
}

impl VargaChart {
    /// Where a graha stands, if the chart carries it.
    ///
    /// ```
    /// use teistro_core::catalogue::{Graha, Varga};
    /// use teistro_vargas::{Axis, chart};
    ///
    /// let d9 = chart(&teistro_chart::sample::foundation()?, Axis::of(Varga::D9))?;
    /// assert!(d9.graha(Graha::Moon).is_some());
    /// assert!(d9.graha(Graha::Uranus).is_none());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn graha(&self, graha: Graha) -> Option<&Placement> {
        self.grahas
            .iter()
            .find(|placement| placement.graha == graha)
            .map(|placement| &placement.at)
    }

    /// The signs of the chart, lagna first, as an almanac prints them.
    ///
    /// ```
    /// use teistro_core::catalogue::Varga;
    /// use teistro_vargas::{Axis, chart};
    ///
    /// let d9 = chart(&teistro_chart::sample::foundation()?, Axis::of(Varga::D9))?;
    /// let signs = d9.signs();
    /// assert_eq!(signs.len(), 10);
    /// assert_eq!(signs[0], (String::from("LAGNA"), d9.lagna.sign));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn signs(&self) -> Vec<(String, Rashi)> {
        let mut signs = vec![(String::from("LAGNA"), self.lagna.sign)];
        signs.extend(
            self.grahas
                .iter()
                .map(|placement| (placement.graha.key().to_string(), placement.at.sign)),
        );
        signs
    }

    /// Which bodies keep the sign they already stood in.
    ///
    /// In the navamsha these are the **vargottama** bodies, which is what
    /// [`VargaChart::vargottama`] answers; in another chart the same fact
    /// has no special name.
    ///
    /// ```
    /// use teistro_core::catalogue::Varga;
    /// use teistro_vargas::{Axis, chart};
    ///
    /// // In the rashi chart every graha keeps its sign.
    /// let d1 = chart(&teistro_chart::sample::foundation()?, Axis::of(Varga::D1))?;
    /// assert_eq!(d1.keeping_their_sign().len(), 9);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn keeping_their_sign(&self) -> Vec<Graha> {
        self.grahas
            .iter()
            .filter(|placement| placement.at.keeps_its_sign())
            .map(|placement| placement.graha)
            .collect()
    }
}

/// Every divisional chart of a founded moment, on one axis.
///
/// ```
/// use teistro_core::catalogue::Varga;
/// use teistro_vargas::{Axis, chart};
///
/// let foundation = teistro_chart::sample::foundation()?;
/// let d1 = chart(&foundation, Axis::of(Varga::D1))?;
/// assert_eq!(d1.grahas[0].at.sign as u8, foundation.grahas[0].sign_index());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// Nothing here can fail once a foundation exists: every longitude it
/// carries is finite and every scheme has a row. The result is a
/// `Result` because a longitude the foundation could not have produced —
/// a NaN from a provider that got past the port — would be one, and
/// silently placing it in Aries is the failure mode this refuses.
pub fn chart(foundation: &ChartFoundation, axis: Axis) -> Result<VargaChart, Error> {
    let lagna = place(&axis.lagna, canonical(foundation.lagna_deg)?);
    let grahas = foundation
        .grahas
        .iter()
        .map(|position| {
            Ok(GrahaPlacement {
                graha: position.graha,
                at: place(&axis.grahas, canonical(position.longitude_deg)?),
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(VargaChart {
        axis,
        lagna,
        grahas,
    })
}

/// A chart in every catalogued varga, on one axis's charts.
///
/// The shape an application asks for when it draws a varga wheel:
/// twenty-one charts of one moment, computed from the same longitudes
/// and costing twenty-one lookups a body.
///
/// ```
/// use teistro_core::catalogue::Varga;
/// use teistro_vargas::every_chart;
///
/// let charts = every_chart(&teistro_chart::sample::foundation()?)?;
/// assert_eq!(charts.len(), Varga::ALL.len());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`chart`].
pub fn every_chart(foundation: &ChartFoundation) -> Result<Vec<VargaChart>, Error> {
    Varga::ALL
        .iter()
        .map(|varga| chart(foundation, Axis::of(*varga)))
        .collect()
}

/// A longitude the foundation carries, as the canonical angle.
fn canonical(degrees: f64) -> Result<Nas, Error> {
    Ok(Nas::from_degrees(Degrees::try_new(
        degrees.rem_euclid(360.0),
    )?))
}

impl VargaChart {
    /// Whether a graha is vargottama: it stands in the navamsha in the
    /// sign it already stood in.
    ///
    /// Answered only for a chart whose grahas are in the navamsha, so
    /// that "vargottama in D10" cannot be asked by accident; the general
    /// fact is [`VargaChart::keeping_their_sign`].
    ///
    /// ```
    /// use teistro_core::catalogue::{Graha, Varga};
    /// use teistro_vargas::{Axis, chart};
    ///
    /// let foundation = teistro_chart::sample::foundation()?;
    /// assert!(chart(&foundation, Axis::of(Varga::D9))?.vargottama(Graha::Sun).is_some());
    /// // Not asked of the dashamsha.
    /// assert_eq!(chart(&foundation, Axis::of(Varga::D10))?.vargottama(Graha::Sun), None);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn vargottama(&self, graha: Graha) -> Option<bool> {
        (self.axis.grahas.varga == Some(Varga::D9))
            .then(|| self.graha(graha).map(Placement::keeps_its_sign))
            .flatten()
    }

    /// Whether the **lagna** is vargottama, which the recording engine
    /// never reports and the texts do (entry 20 of the
    /// deliberate-difference registry).
    ///
    /// ```
    /// use teistro_core::catalogue::Varga;
    /// use teistro_vargas::{Axis, Scheme, chart};
    ///
    /// let foundation = teistro_chart::sample::foundation()?;
    /// let grahas_only = Axis::mixed(Scheme::of(Varga::D9), Scheme::of(Varga::D1));
    /// assert_eq!(chart(&foundation, grahas_only)?.lagna_vargottama(), None);
    /// assert!(chart(&foundation, Axis::of(Varga::D9))?.lagna_vargottama().is_some());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn lagna_vargottama(&self) -> Option<bool> {
        (self.axis.lagna.varga == Some(Varga::D9)).then(|| self.lagna.keeps_its_sign())
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "tests fail by panicking"
    )]

    use super::{Axis, canonical};
    use crate::scheme::Scheme;
    use teistro_core::catalogue::Varga;

    #[test]
    fn an_axis_is_one_chart_unless_it_is_two() {
        let plain = Axis::of(Varga::D9);
        assert_eq!(plain.key(), "D9");
        assert_eq!(plain.grahas, plain.lagna);

        let mixed = Axis::mixed(Scheme::of(Varga::D9), Scheme::of(Varga::D1));
        assert_eq!(mixed.key(), "D9/D1");
        assert_ne!(mixed.grahas, mixed.lagna);

        // The default is the plain rashi chart for both.
        assert_eq!(Axis::default().key(), "D1");
    }

    #[test]
    fn an_arbitrary_chart_names_itself_on_the_axis() {
        let axis = Axis::mixed(
            Scheme::cyclic(37).expect("inside the limit"),
            Scheme::of(Varga::D1),
        );
        assert_eq!(axis.key(), "D37/D1");
    }

    #[test]
    fn a_longitude_is_wrapped_before_it_is_canonical() {
        assert_eq!(canonical(370.0).unwrap(), canonical(10.0).unwrap());
        assert_eq!(canonical(-10.0).unwrap(), canonical(350.0).unwrap());
        assert!(canonical(f64::NAN).is_err(), "a NaN is refused, not placed");
    }
}
