//! The conventions every question to an engine states
//! (`docs/03-design/ephemeris-port-and-adapters.md` §12): the Delta T model
//! a UT1 instant is carried to TT by, and the precession, nutation, frame
//! bias and sidereal time an answer is computed under.
//!
//! The SDK's settings define each convention once, with its citation
//! (`time.delta_t`, `frame.precession`, `frame.nutation`,
//! `frame.frame_bias`, `time.sidereal_time`); a request carries the values
//! a context resolved, and an engine answers under exactly them or returns
//! [`ProviderError::Unsupported`](crate::ProviderError::Unsupported). No
//! convention is left to an engine's default.
//!
//! ```
//! use teistro_core::settings::{Nutation, root};
//! use teistro_port_ephemeris::{DeltaTModel, Models};
//!
//! // What the root settings name, which every request states.
//! let models = Models::of(&root());
//! assert_eq!(models, Models::STANDARD);
//! assert_eq!(models.nutation, Nutation::Iau2000B);
//! assert_eq!(DeltaTModel::from_knob(root().time.delta_t), Some(DeltaTModel::TableThenModel));
//! ```

use core::fmt;

use serde::{Deserialize, Serialize};
use teistro_core::settings::{FrameBias, Nutation, Precession, Settings, SiderealTime};

/// Which Delta T carries a UT1 instant to TT.
///
/// ```
/// use teistro_port_ephemeris::DeltaTModel;
///
/// // Written with its kind.
/// let custom: DeltaTModel = serde_json::from_str(r#"{"kind": "CUSTOM", "seconds": 69.2}"#)?;
/// assert_eq!(custom, DeltaTModel::Custom { seconds: 69.2 });
/// assert_eq!(DeltaTModel::default(), DeltaTModel::TableThenModel);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeltaTModel {
    /// The IERS table where measured (1956 to the present), the model
    /// either side, continuous at the seams: the default, and the SDK's
    /// own definition, which an engine answers only with the same table.
    #[default]
    TableThenModel,
    /// The polynomial fits of Espenak and Meeus (2006) alone, for
    /// reproducing older software.
    EspenakMeeus2006,
    /// Stephenson, Morrison and Hohenkerk (2016), *Proc. R. Soc. A* 472:
    /// the spline of the historical record, with its published σ.
    StephensonMorrisonHohenkerk2016,
    /// A value the consumer supplies.
    Custom {
        /// Delta T in seconds.
        seconds: f64,
    },
}

impl DeltaTModel {
    /// Every model's key, in id order: what the C boundary's id indexes.
    pub const KEYS: [&'static str; 4] = [
        "TABLE_THEN_MODEL",
        "ESPENAK_MEEUS_2006",
        "STEPHENSON_MORRISON_HOHENKERK_2016",
        "CUSTOM",
    ];

    /// The model a settings knob names; `None` for the provider's own
    /// (`PROVIDER`), which the context resolves to the default.
    ///
    /// ```
    /// use teistro_core::settings::DeltaT;
    /// use teistro_port_ephemeris::DeltaTModel;
    ///
    /// assert_eq!(DeltaTModel::from_knob(DeltaT::EspenakMeeus2006), Some(DeltaTModel::EspenakMeeus2006));
    /// assert_eq!(DeltaTModel::from_knob(DeltaT::Provider), None);
    /// ```
    #[must_use]
    pub const fn from_knob(knob: teistro_core::settings::DeltaT) -> Option<DeltaTModel> {
        use teistro_core::settings::DeltaT as Knob;
        match knob {
            Knob::TableThenModel => Some(DeltaTModel::TableThenModel),
            Knob::EspenakMeeus2006 => Some(DeltaTModel::EspenakMeeus2006),
            Knob::StephensonMorrisonHohenkerk2016 => {
                Some(DeltaTModel::StephensonMorrisonHohenkerk2016)
            }
            // The provider's own, and any knob value core adds before this
            // crate learns it: the context supplies those.
            _ => None,
        }
    }

    /// The key stamped in provenance.
    ///
    /// ```
    /// use teistro_port_ephemeris::DeltaTModel;
    ///
    /// assert_eq!(DeltaTModel::TableThenModel.key(), "TABLE_THEN_MODEL");
    /// assert_eq!(DeltaTModel::Custom { seconds: 1.0 }.key(), "CUSTOM");
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            DeltaTModel::TableThenModel => "TABLE_THEN_MODEL",
            DeltaTModel::EspenakMeeus2006 => "ESPENAK_MEEUS_2006",
            DeltaTModel::StephensonMorrisonHohenkerk2016 => "STEPHENSON_MORRISON_HOHENKERK_2016",
            DeltaTModel::Custom { .. } => "CUSTOM",
        }
    }

    /// The stable id at the C boundary; a custom value crosses beside it.
    ///
    /// ```
    /// use teistro_port_ephemeris::DeltaTModel;
    ///
    /// assert_eq!(DeltaTModel::StephensonMorrisonHohenkerk2016.id(), 2);
    /// assert_eq!(DeltaTModel::KEYS[usize::from(DeltaTModel::EspenakMeeus2006.id())], "ESPENAK_MEEUS_2006");
    /// ```
    ///
    /// ```
    /// use teistro_port_ephemeris::DeltaTBand;
    ///
    /// assert_eq!(DeltaTBand::Unstated.id(), 0);
    /// assert_eq!(DeltaTBand::StephensonMorrisonHohenkerk2016.id(), 3);
    /// ```
    #[must_use]
    pub const fn id(self) -> u8 {
        match self {
            DeltaTModel::TableThenModel => 0,
            DeltaTModel::EspenakMeeus2006 => 1,
            DeltaTModel::StephensonMorrisonHohenkerk2016 => 2,
            DeltaTModel::Custom { .. } => 3,
        }
    }

    /// The custom value, seconds, or zero: what crosses beside the id.
    ///
    /// ```
    /// use teistro_port_ephemeris::DeltaTModel;
    ///
    /// assert_eq!(DeltaTModel::Custom { seconds: 69.2 }.custom_seconds(), 69.2);
    /// assert_eq!(DeltaTModel::TableThenModel.custom_seconds(), 0.0);
    /// ```
    #[must_use]
    pub const fn custom_seconds(self) -> f64 {
        match self {
            DeltaTModel::Custom { seconds } => seconds,
            _ => 0.0,
        }
    }

    /// The model with an id, and the value a custom one carries (read only
    /// for id 3).
    ///
    /// ```
    /// use teistro_port_ephemeris::DeltaTModel;
    ///
    /// for model in [DeltaTModel::TableThenModel, DeltaTModel::EspenakMeeus2006, DeltaTModel::Custom { seconds: 64.0 }] {
    ///     assert_eq!(DeltaTModel::from_id(model.id(), model.custom_seconds()), Some(model));
    /// }
    /// assert_eq!(DeltaTModel::from_id(4, 0.0), None);
    /// ```
    #[must_use]
    pub const fn from_id(id: u8, seconds: f64) -> Option<DeltaTModel> {
        match id {
            0 => Some(DeltaTModel::TableThenModel),
            1 => Some(DeltaTModel::EspenakMeeus2006),
            2 => Some(DeltaTModel::StephensonMorrisonHohenkerk2016),
            3 => Some(DeltaTModel::Custom { seconds }),
            _ => None,
        }
    }
}

impl fmt::Display for DeltaTModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeltaTModel::Custom { seconds } => write!(f, "CUSTOM({seconds} s)"),
            other => f.write_str(other.key()),
        }
    }
}

/// Where a Delta T's uncertainty comes from.
///
/// ```
/// use teistro_port_ephemeris::DeltaTBand;
///
/// assert_eq!(DeltaTBand::StephensonMorrisonHohenkerk2016.key(), "STEPHENSON_MORRISON_HOHENKERK_2016");
/// assert_eq!(DeltaTBand::from_id(DeltaTBand::Measured.id()), Some(DeltaTBand::Measured));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeltaTBand {
    /// No uncertainty is stated: a custom value.
    Unstated,
    /// The IERS's measured series, good to milliseconds.
    Measured,
    /// Morrison and Stephenson's (2004) standard errors, from −500 to the
    /// IERS era (their Table 1; `0.8 t²` seconds with `t` in centuries
    /// from 1820 where only the parabola stands): the band the SDK's model
    /// takes over the span its data covers.
    #[serde(rename = "MORRISON_STEPHENSON_2004")]
    MorrisonStephenson2004,
    /// The published σ of Stephenson, Morrison and Hohenkerk (2016).
    #[serde(rename = "STEPHENSON_MORRISON_HOHENKERK_2016")]
    StephensonMorrisonHohenkerk2016,
    /// Huber's (2000) random walk in the length of day, beyond the data:
    /// `σ = 365.25 N √(N Q / 3 · (1 + N / M)) / 1000` seconds with `N`
    /// years from the last value known, `Q = 0.058 ms²/yr` and `M = 2500`
    /// years (Espenak and Meeus 2006, "Uncertainty in Delta T", Table 3).
    #[serde(rename = "HUBER_2000")]
    Huber2000,
}

impl DeltaTBand {
    /// Every band, in id order.
    pub const ALL: [DeltaTBand; 5] = [
        DeltaTBand::Unstated,
        DeltaTBand::Measured,
        DeltaTBand::MorrisonStephenson2004,
        DeltaTBand::StephensonMorrisonHohenkerk2016,
        DeltaTBand::Huber2000,
    ];

    /// The key stamped in provenance.
    ///
    /// ```
    /// use teistro_port_ephemeris::DeltaTBand;
    ///
    /// assert_eq!(DeltaTBand::Measured.key(), "MEASURED");
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            DeltaTBand::Unstated => "UNSTATED",
            DeltaTBand::Measured => "MEASURED",
            DeltaTBand::MorrisonStephenson2004 => "MORRISON_STEPHENSON_2004",
            DeltaTBand::StephensonMorrisonHohenkerk2016 => "STEPHENSON_MORRISON_HOHENKERK_2016",
            DeltaTBand::Huber2000 => "HUBER_2000",
        }
    }

    /// The stable id at the C boundary.
    ///
    /// ```
    /// use teistro_port_ephemeris::DeltaTBand;
    ///
    /// assert_eq!(DeltaTBand::Measured.id(), 1);
    /// assert_eq!(DeltaTBand::from_id(DeltaTBand::Measured.id()), Some(DeltaTBand::Measured));
    /// ```
    #[must_use]
    pub const fn id(self) -> u8 {
        match self {
            DeltaTBand::Unstated => 0,
            DeltaTBand::Measured => 1,
            DeltaTBand::MorrisonStephenson2004 => 2,
            DeltaTBand::StephensonMorrisonHohenkerk2016 => 3,
            DeltaTBand::Huber2000 => 4,
        }
    }

    /// The band with an id.
    ///
    /// ```
    /// use teistro_port_ephemeris::DeltaTBand;
    ///
    /// for band in DeltaTBand::ALL {
    ///     assert_eq!(DeltaTBand::from_id(band.id()), Some(band));
    /// }
    /// assert_eq!(DeltaTBand::from_id(5), None);
    /// ```
    #[must_use]
    pub const fn from_id(id: u8) -> Option<DeltaTBand> {
        match id {
            0 => Some(DeltaTBand::Unstated),
            1 => Some(DeltaTBand::Measured),
            2 => Some(DeltaTBand::MorrisonStephenson2004),
            3 => Some(DeltaTBand::StephensonMorrisonHohenkerk2016),
            4 => Some(DeltaTBand::Huber2000),
            _ => None,
        }
    }
}

/// An engine's Delta T under a model: the value, its one-sigma uncertainty
/// and where that uncertainty comes from.
///
/// ```
/// use teistro_port_ephemeris::{DeltaTBand, DeltaTEstimate};
///
/// let measured = DeltaTEstimate { seconds: 63.83, sigma_seconds: Some(0.005), band: DeltaTBand::Measured };
/// assert!(measured.is_valid());
/// assert!(!DeltaTEstimate { seconds: f64::NAN, ..measured }.is_valid());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DeltaTEstimate {
    /// TT less UT1, seconds.
    pub seconds: f64,
    /// The one-sigma uncertainty, seconds, when the model states one.
    pub sigma_seconds: Option<f64>,
    /// Where the uncertainty comes from.
    pub band: DeltaTBand,
}

impl DeltaTEstimate {
    /// Whether the value is finite and the uncertainty, when given, finite
    /// and not negative.
    ///
    /// ```
    /// use teistro_port_ephemeris::{DeltaTBand, DeltaTEstimate};
    ///
    /// let stated = DeltaTEstimate { seconds: 69.2, sigma_seconds: Some(0.1), band: DeltaTBand::Measured };
    /// assert!(stated.is_valid());
    /// assert!(!DeltaTEstimate { seconds: f64::NAN, ..stated }.is_valid());
    /// assert!(!DeltaTEstimate { sigma_seconds: Some(-1.0), ..stated }.is_valid());
    /// ```
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.seconds.is_finite() && self.sigma_seconds.is_none_or(|s| s.is_finite() && s >= 0.0)
    }
}

/// The models an answer is computed under: the precession, the nutation,
/// the frame bias and the sidereal time.
///
/// ```
/// use teistro_core::settings::{Nutation, Precession};
/// use teistro_port_ephemeris::Models;
///
/// let older = Models { precession: Precession::Iau1976, nutation: Nutation::Iau1980, ..Models::STANDARD };
/// assert_eq!(older.key(), "IAU_1976/IAU_1980/IAU_2006/IERS_2010");
/// assert_eq!(Models::from_ids(older.ids()), Some(older));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Models {
    /// The precession.
    pub precession: Precession,
    /// The nutation series.
    pub nutation: Nutation,
    /// The frame bias.
    pub frame_bias: FrameBias,
    /// The sidereal time.
    pub sidereal_time: SiderealTime,
}

/// A knob value's index among its set's members: its id at the C boundary.
pub(crate) fn index_of<T: PartialEq>(all: &[T], value: &T) -> u8 {
    all.iter()
        .position(|member| member == value)
        .and_then(|i| u8::try_from(i).ok())
        .unwrap_or(u8::MAX)
}

impl Models {
    /// The root settings' models, which are the SDK's own: Vondrák 2011,
    /// IAU 2000B, the IAU 2006 bias and IERS 2010.
    pub const STANDARD: Models = Models {
        precession: Precession::Vondrak2011,
        nutation: Nutation::Iau2000B,
        frame_bias: FrameBias::Iau2006,
        sidereal_time: SiderealTime::Iers2010,
    };

    /// The models a context's settings name.
    ///
    /// ```
    /// use teistro_core::settings::{Precession, root};
    /// use teistro_port_ephemeris::Models;
    ///
    /// let mut settings = root();
    /// settings.frame.precession = Precession::Iau2006;
    /// assert_eq!(Models::of(&settings).precession, Precession::Iau2006);
    /// ```
    #[must_use]
    pub const fn of(settings: &Settings) -> Models {
        Models {
            precession: settings.frame.precession,
            nutation: settings.frame.nutation,
            frame_bias: settings.frame.frame_bias,
            sidereal_time: settings.time.sidereal_time,
        }
    }

    /// The key stamped in provenance.
    ///
    /// ```
    /// use teistro_port_ephemeris::Models;
    ///
    /// assert_eq!(Models::STANDARD.key(), "VONDRAK_2011/IAU_2000B/IAU_2006/IERS_2010");
    /// ```
    #[must_use]
    pub fn key(&self) -> String {
        format!(
            "{}/{}/{}/{}",
            self.precession.key(),
            self.nutation.key(),
            self.frame_bias.key(),
            self.sidereal_time.key()
        )
    }

    /// The four ids at the C boundary, each the value's index among its
    /// knob's members.
    ///
    /// ```
    /// use teistro_port_ephemeris::Models;
    ///
    /// // The standard models are each first in their list.
    /// assert_eq!(Models::STANDARD.ids(), [0, 0, 0, 0]);
    /// ```
    #[must_use]
    pub fn ids(&self) -> [u8; 4] {
        [
            index_of(Precession::ALL, &self.precession),
            index_of(Nutation::ALL, &self.nutation),
            index_of(FrameBias::ALL, &self.frame_bias),
            index_of(SiderealTime::ALL, &self.sidereal_time),
        ]
    }

    /// The models with four ids.
    ///
    /// ```
    /// use teistro_port_ephemeris::Models;
    ///
    /// assert_eq!(Models::from_ids(Models::STANDARD.ids()), Some(Models::STANDARD));
    /// assert_eq!(Models::from_ids([0, 0, 0, u8::MAX]), None);
    /// ```
    #[must_use]
    pub fn from_ids([p, n, b, s]: [u8; 4]) -> Option<Models> {
        Some(Models {
            precession: *Precession::ALL.get(usize::from(p))?,
            nutation: *Nutation::ALL.get(usize::from(n))?,
            frame_bias: *FrameBias::ALL.get(usize::from(b))?,
            sidereal_time: *SiderealTime::ALL.get(usize::from(s))?,
        })
    }
}

impl Default for Models {
    fn default() -> Models {
        Models::STANDARD
    }
}

impl fmt::Display for Models {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.key())
    }
}

impl Models {
    /// The four ids packed into one word, precession in the low byte: how
    /// the C boundary carries them.
    ///
    /// ```
    /// use teistro_port_ephemeris::Models;
    ///
    /// assert_eq!(Models::STANDARD.to_bits(), 0);
    /// assert_eq!(Models::from_bits(Models::STANDARD.to_bits()), Some(Models::STANDARD));
    /// ```
    #[must_use]
    pub fn to_bits(&self) -> u32 {
        u32::from_le_bytes(self.ids())
    }

    /// The models a packed word names.
    ///
    /// ```
    /// use teistro_port_ephemeris::Models;
    ///
    /// assert_eq!(Models::from_bits(Models::STANDARD.to_bits()), Some(Models::STANDARD));
    /// assert_eq!(Models::from_bits(u32::MAX), None);
    /// ```
    #[must_use]
    pub fn from_bits(bits: u32) -> Option<Models> {
        Models::from_ids(bits.to_le_bytes())
    }
}

/// The conventions a question states: the Delta T model a UT1 instant is
/// carried to TT by, and the models the answer is computed under.
///
/// Every request on the port carries one (the positions, the obliquity,
/// the ayanamsha, a rise or set, the crossings, the angles), so an engine
/// is never left to its own defaults. A classical astronomy has no Delta T
/// and no nutation, and answers as its text does whatever these say.
///
/// ```
/// use teistro_core::settings::{DeltaT, SettingsPatch, root};
/// use teistro_port_ephemeris::{Conventions, DeltaTModel, Models};
///
/// let mut patch = SettingsPatch::default();
/// patch.time.delta_t = Some(DeltaT::EspenakMeeus2006);
/// let stated = Conventions::of(&root().patched(&patch));
/// assert_eq!(stated, Conventions { delta_t: DeltaTModel::EspenakMeeus2006, models: Models::STANDARD });
/// assert_eq!(Conventions::STANDARD.key(), "TABLE_THEN_MODEL;VONDRAK_2011/IAU_2000B/IAU_2006/IERS_2010");
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Conventions {
    /// The Delta T model.
    pub delta_t: DeltaTModel,
    /// The precession, nutation, frame bias and sidereal time.
    pub models: Models,
}

impl Conventions {
    /// The root settings' conventions.
    pub const STANDARD: Conventions = Conventions {
        delta_t: DeltaTModel::TableThenModel,
        models: Models::STANDARD,
    };

    /// The conventions a context's settings name; the provider's own Delta
    /// T (`PROVIDER`) resolves to the default model, as the context
    /// resolves it.
    ///
    /// ```
    /// use teistro_core::settings::root;
    /// use teistro_port_ephemeris::Conventions;
    ///
    /// assert_eq!(Conventions::of(&root()), Conventions::STANDARD);
    /// ```
    #[must_use]
    pub fn of(settings: &Settings) -> Conventions {
        Conventions {
            delta_t: DeltaTModel::from_knob(settings.time.delta_t).unwrap_or_default(),
            models: Models::of(settings),
        }
    }

    /// The same conventions under another Delta T model.
    ///
    /// ```
    /// use teistro_port_ephemeris::{Conventions, DeltaTModel, Models};
    ///
    /// let stated = Conventions::STANDARD.with_delta_t(DeltaTModel::EspenakMeeus2006);
    /// assert_eq!((stated.delta_t, stated.models), (DeltaTModel::EspenakMeeus2006, Models::STANDARD));
    /// ```
    #[must_use]
    pub const fn with_delta_t(self, delta_t: DeltaTModel) -> Conventions {
        Conventions { delta_t, ..self }
    }

    /// The key stamped in provenance.
    ///
    /// ```
    /// use teistro_port_ephemeris::Conventions;
    ///
    /// assert_eq!(Conventions::STANDARD.key(), "TABLE_THEN_MODEL;VONDRAK_2011/IAU_2000B/IAU_2006/IERS_2010");
    /// ```
    #[must_use]
    pub fn key(&self) -> String {
        format!("{};{}", self.delta_t, self.models)
    }

    /// The conventions to the bit, for a memo's key: two answers under
    /// different conventions are different answers.
    ///
    /// ```
    /// use teistro_port_ephemeris::{Conventions, DeltaTModel};
    ///
    /// let custom = |seconds| Conventions::STANDARD.with_delta_t(DeltaTModel::Custom { seconds });
    /// assert_ne!(custom(69.0).bits(), custom(70.0).bits());
    /// assert_ne!(Conventions::STANDARD.bits(), custom(0.0).bits());
    /// ```
    #[must_use]
    pub fn bits(&self) -> [u64; 2] {
        [
            u64::from(self.models.to_bits()) | (u64::from(self.delta_t.id()) << 32),
            self.delta_t.custom_seconds().to_bits(),
        ]
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    #[test]
    fn an_estimate_is_valid_only_with_a_finite_value_and_a_sound_sigma() {
        let stated = DeltaTEstimate {
            seconds: 69.2,
            sigma_seconds: Some(0.1),
            band: DeltaTBand::Measured,
        };
        for (estimate, valid) in [
            (stated, true),
            (
                DeltaTEstimate {
                    sigma_seconds: None,
                    ..stated
                },
                true,
            ),
            (
                DeltaTEstimate {
                    sigma_seconds: Some(0.0),
                    ..stated
                },
                true,
            ),
            (
                DeltaTEstimate {
                    seconds: f64::NAN,
                    ..stated
                },
                false,
            ),
            (
                DeltaTEstimate {
                    seconds: f64::INFINITY,
                    sigma_seconds: None,
                    ..stated
                },
                false,
            ),
            (
                DeltaTEstimate {
                    sigma_seconds: Some(-1.0),
                    ..stated
                },
                false,
            ),
            (
                DeltaTEstimate {
                    sigma_seconds: Some(f64::NAN),
                    ..stated
                },
                false,
            ),
            (
                DeltaTEstimate {
                    sigma_seconds: Some(f64::INFINITY),
                    ..stated
                },
                false,
            ),
        ] {
            assert_eq!(estimate.is_valid(), valid, "{estimate:?}");
        }
    }

    #[test]
    fn every_id_round_trips() {
        for (index, key) in DeltaTModel::KEYS.iter().enumerate() {
            let model = DeltaTModel::from_id(u8::try_from(index).unwrap(), 1.5).unwrap();
            assert_eq!(model.key(), *key);
            assert_eq!(usize::from(model.id()), index);
        }
        for band in DeltaTBand::ALL {
            assert_eq!(DeltaTBand::from_id(band.id()), Some(band));
        }
        for precession in Precession::ALL {
            for nutation in Nutation::ALL {
                let models = Models {
                    precession: *precession,
                    nutation: *nutation,
                    ..Models::STANDARD
                };
                assert_eq!(Models::from_ids(models.ids()), Some(models));
            }
        }
        assert_eq!(Models::from_ids([9, 0, 0, 0]), None);
        assert_eq!(Models::default().to_string(), Models::STANDARD.key());
        assert_eq!(
            DeltaTModel::Custom { seconds: 2.0 }.to_string(),
            "CUSTOM(2 s)"
        );
    }

    #[test]
    fn a_bands_serde_spelling_is_its_key() {
        // A year stays its own word, as the provenance's key spells it.
        for band in DeltaTBand::ALL {
            assert_eq!(
                serde_json::to_value(band).unwrap(),
                serde_json::Value::from(band.key())
            );
        }
    }
}
