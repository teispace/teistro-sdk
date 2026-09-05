//! The ayanamsha catalogue as a computation
//! (`docs/03-design/astro-ayanamsha-catalogue.md`): every catalogued
//! ayanamsha has a definition, most an epoch and the value there, carried
//! to any date by precession; twelve are anchored to a star or the galactic
//! centre and wait for the star table; four are frames rather than angles.
//! The value is mean (the offset a sidereal longitude subtracts) or true
//! (with the nutation in longitude added), as the `ayanamsha_basis` knob
//! chooses.
//!
//! The construction is the one every published table of these values was
//! computed from: the vernal point of the date, carried back to the
//! definition's epoch by the precession model in use, read as a longitude
//! on the ecliptic of that epoch, subtracted from the value there, less
//! a correction for the precession model the constant was fitted with
//! (Fagan and Bradley with Newcomb's, Lahiri with IAU 1976's).
//!
//! ```
//! use teistro_astro::ayanamsha::{Basis, mean_deg, value_deg};
//! use teistro_astro::delta_t::DeltaTModel;
//! use teistro_astro::precession::PrecessionModel;
//! use teistro_core::catalogue::Ayanamsha;
//! use teistro_core::quantity::{JulianDay, Tt};
//!
//! let j2000 = JulianDay::<Tt>::literal(2_451_545.0);
//! let lahiri = mean_deg(&Ayanamsha::Lahiri.into(), j2000, PrecessionModel::default(), DeltaTModel::TableThenModel)
//!     .expect("an epoch-defined ayanamsha");
//! assert!((lahiri - 23.857).abs() < 0.001);
//! let nutated = value_deg(&Ayanamsha::Lahiri.into(), j2000, Basis::True, PrecessionModel::default(), DeltaTModel::TableThenModel).unwrap();
//! assert!(((nutated - lahiri) * 3600.0 + 13.9).abs() < 0.2);
//! ```

use teistro_core::angle::normalise_deg;
use teistro_core::catalogue::Ayanamsha;
use teistro_core::error::{Error, Status};
use teistro_core::quantity::{JulianDay, Tt, Ut1};
pub use teistro_core::settings::AyanamshaBasis as Basis;
use teistro_core::settings::AyanamshaChoice;

use crate::delta_t::{DeltaTModel, delta_t};
use crate::iau::vector::c2s;
use crate::iau::{DJ00, DJY, RAD2DEG, nut00b};
use crate::precession::{self, PrecessionModel};
use crate::scale::tt_from_ut1;

/// The time scale a definition's epoch is stated in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EpochScale {
    /// Terrestrial Time (the modern definitions).
    Tt,
    /// Universal Time (the historical definitions, whose authors reckoned
    /// in civil time); Delta T of the epoch is applied.
    Ut,
}

/// The precession model a published constant was fitted with, when it was
/// not the model in use: the value is corrected for the difference so the
/// published number keeps its meaning under any model.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Fitted {
    /// Fitted with whatever model the SDK uses; no correction.
    Current,
    /// Fitted with IAU 1976 precession.
    Iau1976,
    /// Fitted with Newcomb's precession.
    Newcomb,
}

/// An epoch-defined ayanamsha: the value at a reference epoch, carried to
/// any date by precession.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Epoch {
    /// The reference epoch, a Julian day.
    pub jd: f64,
    /// The scale the epoch is stated in.
    pub scale: EpochScale,
    /// The ayanamsha at the epoch, degrees.
    pub value_deg: f64,
    /// The precession model the value was fitted with.
    pub fitted: Fitted,
}

/// How a catalogued ayanamsha is defined.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Definition {
    /// A value at an epoch carried by precession.
    Epoch(Epoch),
    /// A frame rather than an angle: the sidereal zodiac is the ecliptic
    /// and equinox of the epoch itself (J2000, J1900, B1950, Mardyks). The
    /// value is the precession since the epoch; positions referred to the
    /// ecliptic of the epoch are a completion step, not an angle.
    Frame(Epoch),
    /// Anchored to where a body is: the star's or the galactic centre's
    /// longitude of date less a fixed sidereal longitude. Needs the star
    /// table and an ephemeris for the aberration.
    Object {
        /// The anchor as the star table names it.
        anchor: &'static str,
        /// The sidereal longitude the anchor is fixed at, degrees.
        fixed_deg: f64,
    },
    /// A member the catalogue registers that this build has no definition
    /// for: refused as unsourced, never approximated.
    Unsourced,
}

const J1900: f64 = 2_415_020.0;
const B1950: f64 = 2_433_282.423_459_05;

const fn tt(jd: f64, value_deg: f64, fitted: Fitted) -> Definition {
    Definition::Epoch(Epoch {
        jd,
        scale: EpochScale::Tt,
        value_deg,
        fitted,
    })
}

const fn ut(jd: f64, value_deg: f64, fitted: Fitted) -> Definition {
    Definition::Epoch(Epoch {
        jd,
        scale: EpochScale::Ut,
        value_deg,
        fitted,
    })
}

const fn frame(jd: f64, value_deg: f64) -> Definition {
    Definition::Frame(Epoch {
        jd,
        scale: EpochScale::Tt,
        value_deg,
        fitted: Fitted::Current,
    })
}

const fn object(anchor: &'static str, fixed_deg: f64) -> Definition {
    Definition::Object { anchor, fixed_deg }
}

/// The definition of a catalogued ayanamsha: the published epochs and
/// values as the Swiss Ephemeris documents them and Teimeris carries them
/// (`docs/03-design/astro-ayanamsha-catalogue.md` lists each with its
/// author's source).
#[must_use]
pub const fn definition(id: Ayanamsha) -> Definition {
    match id {
        Ayanamsha::FaganBradley => tt(B1950, 24.042_044_444, Fitted::Newcomb),
        Ayanamsha::Lahiri => tt(2_435_553.5, 23.250_182_778 - 0.004_658_035, Fitted::Iau1976),
        Ayanamsha::Deluce => ut(1_721_057.5, 0.0, Fitted::Current),
        Ayanamsha::Raman => tt(J1900, 360.0 - 338.985_56, Fitted::Newcomb),
        Ayanamsha::Ushashashi => tt(J1900, 360.0 - 341.339_04, Fitted::Current),
        Ayanamsha::Krishnamurti => tt(J1900, 360.0 - 337.636_111, Fitted::Newcomb),
        Ayanamsha::DjwhalKhul => tt(J1900, 360.0 - 333.036_902_4, Fitted::Current),
        Ayanamsha::Yukteshwar => tt(J1900, 360.0 - 338.917_778, Fitted::Current),
        Ayanamsha::JnBhasin => tt(J1900, 360.0 - 338.634_444, Fitted::Current),
        Ayanamsha::BabylKugler1 => ut(1_684_532.5, -5.666_67, Fitted::Current),
        Ayanamsha::BabylKugler2 => ut(1_684_532.5, -4.266_67, Fitted::Current),
        Ayanamsha::BabylKugler3 => ut(1_684_532.5, -3.416_67, Fitted::Current),
        Ayanamsha::BabylHuber => ut(1_684_532.5, -4.466_67, Fitted::Current),
        Ayanamsha::BabylEtpsc => ut(1_673_941.0, -5.079_167, Fitted::Current),
        Ayanamsha::Aldebaran15tau => ut(1_684_532.5, -4.441_385_98, Fitted::Current),
        Ayanamsha::Hipparchos => ut(1_674_484.0, -9.333_33, Fitted::Current),
        Ayanamsha::Sassanian => ut(1_927_135.874_779_3, 0.0, Fitted::Current),
        Ayanamsha::Galcent0sag => object("Sgr A*", 240.0),
        Ayanamsha::J2000 => frame(DJ00, 0.0),
        Ayanamsha::J1900 => frame(J1900, 0.0),
        Ayanamsha::B1950 => frame(B1950, 0.0),
        Ayanamsha::Suryasiddhanta => ut(1_903_396.812_865_4, 0.0, Fitted::Current),
        Ayanamsha::SuryasiddhantaMsun => ut(1_903_396.812_865_4, -0.214_633_95, Fitted::Current),
        Ayanamsha::Aryabhata => ut(1_903_396.789_532_1, 0.0, Fitted::Current),
        Ayanamsha::AryabhataMsun => ut(1_903_396.789_532_1, -0.237_632_38, Fitted::Current),
        Ayanamsha::SsRevati => ut(1_903_396.812_865_4, -0.791_670_46, Fitted::Current),
        Ayanamsha::SsCitra => ut(1_903_396.812_865_4, 2.110_704_44, Fitted::Current),
        Ayanamsha::TrueChitra => object("Spica", 180.0),
        Ayanamsha::TrueRevati => object("zeta Piscium", 359.833_333_333_3),
        Ayanamsha::TruePushya => object("delta Cancri", 106.0),
        Ayanamsha::GalcentRgilbrand => object("Sgr A*", 210.0 + 90.0 * 0.381_966_011_3),
        Ayanamsha::GalequIau1958 => object("galactic pole (IAU 1958)", 150.0),
        Ayanamsha::GalequTrue => object("galactic pole", 150.0),
        Ayanamsha::GalequMula => object("galactic pole", 150.0 + 6.666_666_666_7),
        Ayanamsha::GalalignMardyks => frame(2_451_079.734_892, 30.0),
        Ayanamsha::TrueMula => object("lambda Scorpii", 240.0),
        Ayanamsha::GalcentMulaWilhelm => object("Sgr A*", 246.666_666_666_7),
        Ayanamsha::Aryabhata522 => ut(1_911_797.740_782_065, 0.0, Fitted::Current),
        Ayanamsha::BabylBritton => ut(1_721_057.5, -3.2, Fitted::Current),
        Ayanamsha::TrueSheoran => object("delta Cancri", 103.492_642_216_25),
        Ayanamsha::GalcentCochrane => object("Sgr A*", 270.0),
        Ayanamsha::GalequFiorenza => ut(2_451_544.5, 25.0, Fitted::Current),
        Ayanamsha::ValensMoon => ut(1_775_845.5, -2.9422, Fitted::Current),
        Ayanamsha::Lahiri1940 => tt(J1900, 22.445_972_22, Fitted::Newcomb),
        Ayanamsha::LahiriVp285 => tt(1_825_235.245_851_302_8, 0.0, Fitted::Current),
        Ayanamsha::KrishnamurtiVp291 => tt(1_827_424.752_255_678, 0.0, Fitted::Current),
        Ayanamsha::LahiriIcrc => tt(2_435_553.5, 23.25 - 0.004_642_07, Fitted::Newcomb),
        // A member the catalogue adds before this crate learns it.
        _ => Definition::Unsourced,
    }
}

/// Whether the SDK computes a catalogued ayanamsha itself: the epoch and
/// frame definitions yes, the object-anchored ones once the star table
/// exists.
#[must_use]
pub const fn is_computable(id: Ayanamsha) -> bool {
    matches!(definition(id), Definition::Epoch(_) | Definition::Frame(_))
}

/// The epoch of an epoch or frame definition in TT, with Delta T applied
/// to an epoch stated in Universal Time.
fn epoch_tt(epoch: &Epoch, delta_t_model: DeltaTModel) -> Result<JulianDay<Tt>, Error> {
    match epoch.scale {
        EpochScale::Tt => Ok(JulianDay::try_new(epoch.jd)?),
        EpochScale::Ut => {
            let ut1 = JulianDay::<Ut1>::try_new(epoch.jd)?;
            let dt = delta_t(ut1, delta_t_model)?;
            Ok(tt_from_ut1(ut1, &dt))
        }
    }
}

/// The longitude, degrees, of a mean equatorial unit vector of an epoch
/// read on the ecliptic of that epoch under a model.
fn ecliptic_longitude_deg(v: [f64; 3], model: PrecessionModel, at: JulianDay<Tt>) -> f64 {
    let ecliptic = precession::equatorial_to_ecliptic(v, precession::mean_obliquity_rad(model, at));
    let (lon, _) = c2s(&ecliptic);
    lon * RAD2DEG
}

/// The correction for the precession model a constant was fitted with:
/// the vernal point of the epoch carried to J2000.0 by the model in use
/// and back with the fitted model, read on the ecliptic of the epoch.
fn fitted_correction_deg(epoch: &Epoch, t0: JulianDay<Tt>, model: PrecessionModel) -> f64 {
    let fitted = match epoch.fitted {
        Fitted::Current => return 0.0,
        Fitted::Iau1976 => PrecessionModel::Iau1976,
        Fitted::Newcomb => PrecessionModel::Newcomb,
    };
    if t0.get() == DJ00 {
        return 0.0;
    }
    let x = precession::to_j2000(model, t0, [1.0, 0.0, 0.0]);
    let x = precession::to_date(fitted, t0, x);
    let correction = normalise_deg(ecliptic_longitude_deg(x, model, t0));
    // A residual near zero seen from the other side.
    if correction > 350.0 {
        correction - 360.0
    } else {
        correction
    }
}

/// The mean value of an epoch or frame definition at a TT instant.
fn epoch_value_deg(
    epoch: &Epoch,
    at: JulianDay<Tt>,
    model: PrecessionModel,
    delta_t_model: DeltaTModel,
) -> Result<f64, Error> {
    let t0 = epoch_tt(epoch, delta_t_model)?;
    // The vernal point of the date, a unit vector along x by construction,
    // carried to J2000.0 and on to the epoch, read on the epoch's ecliptic.
    let mut x = [1.0, 0.0, 0.0];
    if at.get() != DJ00 {
        x = precession::to_j2000(model, at, x);
    }
    x = precession::to_date(model, t0, x);
    let longitude_at_t0 = ecliptic_longitude_deg(x, model, t0);
    let value = -longitude_at_t0 + epoch.value_deg - fitted_correction_deg(epoch, t0, model);
    Ok(normalise_deg(value))
}

fn unsupported_object(id: Ayanamsha, anchor: &str) -> Error {
    Error::new(
        Status::Unsupported,
        format!(
            "the {} ayanamsha is anchored to {anchor}, which needs the star table; choose an epoch-defined ayanamsha such as LAHIRI, or a provider that declares the AYANAMSHA override",
            id.key()
        ),
    )
    .with_field("frame.ayanamsha")
}

/// The mean ayanamsha at a TT instant, degrees: the offset a sidereal
/// longitude subtracts from a tropical one.
///
/// # Errors
///
/// An object-anchored ayanamsha (`UNSUPPORTED`, naming the anchor), a
/// custom definition that is not finite (`INVALID_ARG`), or an epoch the
/// Delta T model cannot answer for.
pub fn mean_deg(
    choice: &AyanamshaChoice,
    at: JulianDay<Tt>,
    model: PrecessionModel,
    delta_t_model: DeltaTModel,
) -> Result<f64, Error> {
    match choice {
        AyanamshaChoice::Catalogued { id } => match definition(*id) {
            Definition::Epoch(epoch) | Definition::Frame(epoch) => {
                epoch_value_deg(&epoch, at, model, delta_t_model)
            }
            Definition::Object { anchor, .. } => Err(unsupported_object(*id, anchor)),
            Definition::Unsourced => Err(Error::new(
                Status::Unsupported,
                format!(
                    "the {} ayanamsha has no definition in this build (unsourced)",
                    id.key()
                ),
            )
            .with_field("frame.ayanamsha")),
        },
        AyanamshaChoice::Custom {
            epoch_jd_tt,
            value_deg,
            rate_deg_per_year,
        } => {
            if !(epoch_jd_tt.is_finite() && value_deg.is_finite() && rate_deg_per_year.is_finite())
            {
                return Err(Error::invalid_arg(
                    "a custom ayanamsha needs a finite epoch, value and rate",
                )
                .with_field("frame.ayanamsha"));
            }
            let years = (at.get() - epoch_jd_tt) / DJY;
            Ok(normalise_deg(value_deg + rate_deg_per_year * years))
        }
    }
}

/// The ayanamsha at a TT instant under a basis: the mean value, or the
/// mean value with the IAU 2000B nutation in longitude added (what a
/// tropical longitude of the true equinox subtracts).
///
/// # Errors
///
/// As [`mean_deg`].
pub fn value_deg(
    choice: &AyanamshaChoice,
    at: JulianDay<Tt>,
    basis: Basis,
    model: PrecessionModel,
    delta_t_model: DeltaTModel,
) -> Result<f64, Error> {
    let mean = mean_deg(choice, at, model, delta_t_model)?;
    Ok(match basis {
        Basis::True => {
            let (date1, date2) = at.split();
            normalise_deg(mean + nut00b(date1, date2).dpsi * RAD2DEG)
        }
        // The mean value, and a basis core adds before this crate learns it.
        _ => mean,
    })
}

/// The rate of the mean ayanamsha, degrees per day, by a central
/// difference over a day: the general precession, about 50″ a year.
///
/// # Errors
///
/// As [`mean_deg`].
pub fn speed_deg_per_day(
    choice: &AyanamshaChoice,
    at: JulianDay<Tt>,
    model: PrecessionModel,
    delta_t_model: DeltaTModel,
) -> Result<f64, Error> {
    let before = mean_deg(choice, at.plus_days(-0.5)?, model, delta_t_model)?;
    let after = mean_deg(choice, at.plus_days(0.5)?, model, delta_t_model)?;
    Ok(teistro_core::angle::difference_deg(after, before))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    const J2000: JulianDay<Tt> = JulianDay::literal(DJ00);
    const MODEL: PrecessionModel = PrecessionModel::Vondrak2011;
    const DELTA_T: DeltaTModel = DeltaTModel::TableThenModel;

    fn mean(id: Ayanamsha, at: JulianDay<Tt>) -> f64 {
        mean_deg(&id.into(), at, MODEL, DELTA_T).unwrap()
    }

    #[test]
    fn the_published_values_at_j2000_are_reproduced() {
        // The kit's published values (Lahiri from the Indian Astronomical
        // Ephemeris; Raman and Krishnamurti from their authors' tables), and
        // Fagan and Bradley's synetic vernal point at 24.74°.
        assert!((mean(Ayanamsha::Lahiri, J2000) - 23.857).abs() < 0.002);
        assert!((mean(Ayanamsha::Raman, J2000) - 22.41).abs() < 0.01);
        assert!((mean(Ayanamsha::Krishnamurti, J2000) - 23.76).abs() < 0.01);
        assert!((mean(Ayanamsha::FaganBradley, J2000) - 24.74).abs() < 0.01);
        // A frame definition's value is the precession since its epoch:
        // zero at J2000 for J2000, a century's worth for J1900.
        assert!(mean(Ayanamsha::J2000, J2000).abs() < 1e-9);
        assert!((mean(Ayanamsha::J1900, J2000) - 1.396).abs() < 0.002);
    }

    #[test]
    fn the_value_at_the_epoch_is_the_published_constant() {
        for id in [
            Ayanamsha::Lahiri,
            Ayanamsha::FaganBradley,
            Ayanamsha::Raman,
            Ayanamsha::Lahiri1940,
        ] {
            let Definition::Epoch(epoch) = definition(id) else {
                panic!("{id:?} is epoch-defined");
            };
            let at = JulianDay::<Tt>::literal(epoch.jd);
            let value = mean(id, at);
            // The fitted-model correction moves the value at its own epoch
            // by a small residual only.
            assert!(
                (value - normalise_deg(epoch.value_deg)).abs() < 0.001,
                "{id:?}: {value} against {}",
                epoch.value_deg
            );
        }
        // A UT epoch takes Delta T: the Babylonian definitions at their own
        // epoch return their constant modulo 360 within the day's precession.
        let Definition::Epoch(kugler) = definition(Ayanamsha::BabylKugler1) else {
            panic!("epoch-defined");
        };
        let value = mean(Ayanamsha::BabylKugler1, JulianDay::literal(kugler.jd));
        assert!(
            (value - normalise_deg(kugler.value_deg)).abs() < 0.01,
            "{value}"
        );
    }

    #[test]
    fn nutation_custom_speed_and_refusals() {
        let lahiri: AyanamshaChoice = Ayanamsha::Lahiri.into();
        let mean = mean_deg(&lahiri, J2000, MODEL, DELTA_T).unwrap();
        let nutated = value_deg(&lahiri, J2000, Basis::True, MODEL, DELTA_T).unwrap();
        // The IAU 2000B nutation in longitude at J2000.0: −13.9″.
        assert!(
            ((nutated - mean) * 3600.0 + 13.93).abs() < 0.2,
            "{}",
            (nutated - mean) * 3600.0
        );
        let speed = speed_deg_per_day(&lahiri, J2000, MODEL, DELTA_T).unwrap();
        assert!((speed * 365.25 * 3600.0 - 50.29).abs() < 0.1, "{speed}");
        let custom = AyanamshaChoice::Custom {
            epoch_jd_tt: DJ00,
            value_deg: 23.0,
            rate_deg_per_year: 50.29 / 3600.0,
        };
        let later = mean_deg(&custom, JulianDay::literal(DJ00 + 36_525.0), MODEL, DELTA_T).unwrap();
        assert!((later - 23.0 - 100.0 * 50.29 / 3600.0).abs() < 1e-9);
        let bad = AyanamshaChoice::Custom {
            epoch_jd_tt: f64::NAN,
            value_deg: 23.0,
            rate_deg_per_year: 0.0,
        };
        assert_eq!(
            mean_deg(&bad, J2000, MODEL, DELTA_T).unwrap_err().status,
            Status::InvalidArg
        );
        let star = mean_deg(&Ayanamsha::TrueChitra.into(), J2000, MODEL, DELTA_T).unwrap_err();
        assert_eq!(star.status, Status::Unsupported);
        assert!(star.to_string().contains("Spica"), "{star}");
        assert!(!is_computable(Ayanamsha::TrueChitra));
        assert!(is_computable(Ayanamsha::Lahiri));
    }

    #[test]
    fn the_models_agree_closely_for_modern_epochs() {
        for id in [
            Ayanamsha::Lahiri,
            Ayanamsha::Raman,
            Ayanamsha::FaganBradley,
            Ayanamsha::Krishnamurti,
        ] {
            let a = mean_deg(&id.into(), J2000, PrecessionModel::Vondrak2011, DELTA_T).unwrap();
            let b = mean_deg(&id.into(), J2000, PrecessionModel::Iau2006, DELTA_T).unwrap();
            assert!(
                ((a - b) * 3600.0).abs() < 0.01,
                "{id:?}: {}\"",
                (a - b) * 3600.0
            );
        }
    }
}
