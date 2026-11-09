//! A second birth a chart is read against: the partner a match, a
//! synastry or a Davison chart takes (`03-design/matching.md`,
//! `03-design/western-synastry.md`).

use serde::{Deserialize, Serialize};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_core::time::{LocalClock as _, UtcOffset};
use teistro_time::ZoneSpec;

use crate::DayClock;
use crate::context::Context;

/// A second birth, which every chart of a batch is read against.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Partner {
    /// The birth's instant.
    pub instant: JulianDay<Utc>,
    /// Where it happened.
    pub place: Place,
    /// Its civil clock, which a chart's day is reckoned by; or `zone`, not
    /// both; UTC when neither is given.
    #[serde(
        rename = "utcOffsetSeconds",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub utc_offset: Option<UtcOffset>,
    /// The zone the birth is read on, at the offset it had then
    /// (`03-design/time-and-timezone.md` §3.4); or `utcOffsetSeconds`, not
    /// both.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone: Option<ZoneSpec>,
}

impl Partner {
    /// A birth on a stated offset.
    ///
    /// ```
    /// use teistro::{DayClock, Partner, UtcOffset};
    /// use teistro::quantity::{JulianDay, Place};
    ///
    /// let place = Place::try_from_degrees(27.7172, 85.324, 1400.0)?;
    /// let partner = Partner::at(JulianDay::literal(2_446_217.770_833), place, UtcOffset::literal(5, 30, 0));
    /// assert_eq!(partner.clock()?, DayClock::Offset(UtcOffset::literal(5, 30, 0)));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn at(instant: JulianDay<Utc>, place: Place, offset: UtcOffset) -> Partner {
        Partner {
            instant,
            place,
            utc_offset: Some(offset),
            zone: None,
        }
    }

    /// A birth on a zone's clock, read at the offset the zone had then.
    ///
    /// ```
    /// use teistro::{Partner, ZoneSpec};
    /// use teistro::quantity::{JulianDay, Place};
    ///
    /// let place = Place::try_from_degrees(27.7172, 85.324, 1400.0)?;
    /// let partner = Partner::in_zone(JulianDay::literal(2_446_217.770_833), place, ZoneSpec::iana("Asia/Kathmandu"));
    /// assert_eq!(partner.utc_offset, None);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn in_zone(instant: JulianDay<Utc>, place: Place, zone: ZoneSpec) -> Partner {
        Partner {
            instant,
            place,
            utc_offset: None,
            zone: Some(zone),
        }
    }

    /// The clock the birth is read on: the offset stated, the zone, or UTC
    /// when neither is given.
    ///
    /// ```
    /// use teistro::{DayClock, Partner, UtcOffset, ZoneSpec};
    /// use teistro::quantity::{JulianDay, Place};
    ///
    /// let place = Place::try_from_degrees(27.7172, 85.324, 1400.0)?;
    /// let mut partner = Partner::in_zone(JulianDay::literal(2_446_217.770_833), place, ZoneSpec::iana("Asia/Kathmandu"));
    /// assert_eq!(partner.clock()?, DayClock::Zone(ZoneSpec::iana("Asia/Kathmandu")));
    /// partner.utc_offset = Some(UtcOffset::literal(5, 30, 0));
    /// assert_eq!(partner.clock().unwrap_err().field(), Some("zone"));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on `zone` for both an offset and a zone.
    pub fn clock(&self) -> Result<DayClock, Error> {
        match (self.utc_offset, &self.zone) {
            (None, None) => Ok(DayClock::Offset(UtcOffset::UTC)),
            (offset, zone) => DayClock::asked(offset.map(UtcOffset::seconds), zone.clone()),
        }
    }

    /// The birth on the offset its clock had at the birth: itself when the
    /// offset was stated, and a zone's read through `context`'s database
    /// otherwise. What a Davison birth's mean of two clocks reads.
    ///
    /// ```
    /// use teistro::{Context, Ephemeris, Partner, UtcOffset, ZoneSpec};
    /// use teistro::quantity::{JulianDay, Place};
    ///
    /// let sdk = Context::builder().ephemeris([Ephemeris::None]).build()?;
    /// let place = Place::try_from_degrees(27.7172, 85.324, 1400.0)?;
    /// // 1985 in Kathmandu ran on +05:30.
    /// let partner = Partner::in_zone(JulianDay::literal(2_446_217.770_833), place, ZoneSpec::iana("Asia/Kathmandu"));
    /// let fixed = partner.fixed(&sdk)?;
    /// assert_eq!((fixed.utc_offset, fixed.zone), (Some(UtcOffset::literal(5, 30, 0)), None));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`Partner::clock`]; a zone the database does not have.
    pub fn fixed(&self, context: &Context) -> Result<Partner, Error> {
        Ok(match self.clock()? {
            DayClock::Offset(offset) => Partner {
                utc_offset: Some(offset),
                zone: None,
                ..self.clone()
            },
            DayClock::Zone(zone) => Partner {
                utc_offset: Some(zone.clock(context.zones())?.offset_at(self.instant)),
                zone: None,
                ..self.clone()
            },
        })
    }
}
