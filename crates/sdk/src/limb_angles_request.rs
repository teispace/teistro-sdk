//! The panchanga's own sidereal longitudes at instants, as a binding asks
//! them (SR-137, `03-design/panchanga-reach.md` §3): the Sun and the Moon
//! where the day's limbs read them, with the elongation and the yoga's
//! sum.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Calendar;
use teistro_core::envelope::Envelope;
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_core::time::UtcOffset;
use teistro_panchanga::limb::LimbAngles;

use crate::AlmanacArea;
use crate::asked::{instant, place_of};

/// The record's name where a binding sends it, which a refusal is named
/// under.
const LIMB_ANGLES: &str = "limbAngles";

/// Instants and a place, as a binding asks the limbs' angles.
///
/// ```
/// use teistro::LimbAnglesRequest;
///
/// let asked = LimbAnglesRequest::from_json(
///     r#"{"instants": [2461345.25, 2461345.5], "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400}"#,
/// )?;
/// assert_eq!(asked.instants.len(), 2);
/// let none = LimbAnglesRequest::from_json(r#"{"instants": [], "latitudeDeg": 27.7, "longitudeDeg": 85.3}"#)
///     .unwrap_err();
/// assert_eq!(none.field(), Some("limbAngles.instants"));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct LimbAnglesRequest {
    /// The instants, UTC.
    pub instants: Vec<JulianDay<Utc>>,
    /// The place: a topocentric panchanga reads the Moon from it.
    pub place: Place,
}

/// The request as a binding writes it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "LimbAnglesRequest"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct LimbAnglesAsked {
    /// The instants, UTC Julian days: one to 4096.
    instants: Vec<f64>,
    /// The place's latitude, degrees north positive.
    latitude_deg: f64,
    /// The place's longitude, degrees east positive.
    longitude_deg: f64,
    /// The place's height above sea level in metres, 0 when left out.
    #[serde(default)]
    altitude_m: f64,
}

impl LimbAnglesRequest {
    /// The record a binding sends, as JSON: `instants`, UTC Julian days;
    /// and `latitudeDeg`, `longitudeDeg` and `altitudeM` (0 when left out).
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, no instants or more than
    /// [`teistro_panchanga::almanac::MOST_ANGLE_INSTANTS`], an instant
    /// that is not finite, or a place out of range, each named under
    /// `limbAngles`.
    pub fn from_json(text: &str) -> Result<LimbAnglesRequest, Error> {
        let asked: LimbAnglesAsked = teistro_core::strict::read(text, LIMB_ANGLES)?;
        asked.request().map_err(|why| why.under(LIMB_ANGLES))
    }
}

impl LimbAnglesAsked {
    fn request(self) -> Result<LimbAnglesRequest, Error> {
        let most = teistro_panchanga::almanac::MOST_ANGLE_INSTANTS;
        if self.instants.is_empty() || self.instants.len() > most {
            return Err(Error::invalid_arg(format!(
                "the angles are asked at one to {most} instants, not {}",
                self.instants.len()
            ))
            .with_field("instants"));
        }
        Ok(LimbAnglesRequest {
            instants: self
                .instants
                .iter()
                .map(|at| instant(*at, "instants"))
                .collect::<Result<_, _>>()?,
            place: place_of(self.latitude_deg, self.longitude_deg, self.altitude_m)?,
        })
    }
}

impl AlmanacArea<'_> {
    /// **The Sun's and the Moon's sidereal longitudes at each instant as
    /// the panchanga reads them**, with the elongation (the tithi's and the
    /// karana's angle) and the yoga's sum (SR-137,
    /// `03-design/panchanga-reach.md` §3).
    ///
    /// Read in the one sky every almanac day's limbs are found in, so
    /// every knob of the profile is honoured as the limbs honour it: the
    /// ayanamsha and its basis, a custom ayanamsha, the panchanga's centre
    /// (the place matters when it is topocentric), apparent or true
    /// positions, and a provider that defines its own zodiac, the Surya
    /// Siddhanta's. A wheel drawn from these angles and a day's limbs
    /// never disagree: at a limb's end the search found, the angle sits on
    /// the limb's grid to the search's tolerance.
    ///
    /// ```no_run
    /// use teistro::Context;
    /// use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    ///
    /// let sdk = Context::builder().profile("nepali-drik").build()?;
    /// let kathmandu = Place::new(Latitude::try_new(27.7172)?, Longitude::try_new(85.324)?, Altitude::try_new(1400.0)?);
    /// let at = JulianDay::try_new(2_461_345.25)?;
    /// let angles = sdk.almanac().limb_angles(&[at], &kathmandu)?;
    /// let now = angles.value[0];
    /// println!("Sun {:.4}°, Moon {:.4}°, {:?}", now.sun_deg, now.moon_deg, now.tithi());
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A context with no ephemeris, no instants or more than
    /// [`teistro_panchanga::almanac::MOST_ANGLE_INSTANTS`] (`INVALID_ARG`
    /// on `instants`), and an instant the provider refuses.
    pub fn limb_angles(
        self,
        instants: &[JulianDay<Utc>],
        place: &Place,
    ) -> Result<Envelope<Vec<LimbAngles>>, Error> {
        // The instants are UTC and no civil day is read, so the clock the
        // provenance states is UTC's.
        let found = self.with_almanac(Calendar::Gregorian, UtcOffset::UTC, |almanac| {
            almanac.limb_angles(instants, place)
        })?;
        Ok(Envelope::sealing(found.value, found.provenance))
    }

    /// As [`AlmanacArea::limb_angles`] for a request a binding sent.
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::limb_angles`].
    pub fn limb_angles_request(
        self,
        asked: &LimbAnglesRequest,
    ) -> Result<Envelope<Vec<LimbAngles>>, Error> {
        self.limb_angles(&asked.instants, &asked.place)
    }
}
