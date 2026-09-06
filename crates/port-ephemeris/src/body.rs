//! The bodies the port knows and the time scales a request is in.

use core::fmt;
use core::str::FromStr;

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::distance;
use teistro_core::error::Error;

/// The bodies the port knows: the ten classical bodies of modern
/// astrology and the four lunar points. Adapters map them to their own
/// numbering; a body an adapter cannot compute is reported per cell,
/// never guessed. The astrological grahas of the catalogue map onto these
/// under the node knob (Rahu is the mean or the true node).
///
/// ```
/// use teistro_port_ephemeris::Body;
///
/// assert_eq!(Body::MeanNode.key(), "MEAN_NODE");
/// assert_eq!("moon".parse::<Body>().ok(), Some(Body::Moon));
/// assert_eq!(Body::from_id(Body::Pluto.id()), Some(Body::Pluto));
/// assert!(!Body::TrueNode.has_distance());
/// ```
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Body {
    /// The Sun.
    Sun = 0,
    /// The Moon.
    Moon = 1,
    /// Mercury.
    Mercury = 2,
    /// Venus.
    Venus = 3,
    /// Mars.
    Mars = 4,
    /// Jupiter.
    Jupiter = 5,
    /// Saturn.
    Saturn = 6,
    /// Uranus.
    Uranus = 7,
    /// Neptune.
    Neptune = 8,
    /// Pluto.
    Pluto = 9,
    /// The mean ascending lunar node.
    MeanNode = 10,
    /// The true (osculating) ascending lunar node.
    TrueNode = 11,
    /// The mean lunar apogee.
    MeanApogee = 12,
    /// The osculating lunar apogee.
    OsculatingApogee = 13,
}

impl Body {
    /// Every body, in stable id order.
    pub const ALL: [Body; 14] = [
        Body::Sun,
        Body::Moon,
        Body::Mercury,
        Body::Venus,
        Body::Mars,
        Body::Jupiter,
        Body::Saturn,
        Body::Uranus,
        Body::Neptune,
        Body::Pluto,
        Body::MeanNode,
        Body::TrueNode,
        Body::MeanApogee,
        Body::OsculatingApogee,
    ];

    /// The ten classical bodies, the grid engines are measured on.
    pub const PLANETS: [Body; 10] = [
        Body::Sun,
        Body::Moon,
        Body::Mercury,
        Body::Venus,
        Body::Mars,
        Body::Jupiter,
        Body::Saturn,
        Body::Uranus,
        Body::Neptune,
        Body::Pluto,
    ];

    /// The stable numeric id used at the C boundary: the discriminant,
    /// which is the body's position in [`Body::ALL`].
    #[must_use]
    pub const fn id(self) -> u16 {
        self as u16
    }

    /// The body with a given id.
    #[must_use]
    pub fn from_id(id: u16) -> Option<Body> {
        Body::ALL.get(usize::from(id)).copied()
    }

    /// The upper-case key used in fixtures and bindings.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Body::Sun => "SUN",
            Body::Moon => "MOON",
            Body::Mercury => "MERCURY",
            Body::Venus => "VENUS",
            Body::Mars => "MARS",
            Body::Jupiter => "JUPITER",
            Body::Saturn => "SATURN",
            Body::Uranus => "URANUS",
            Body::Neptune => "NEPTUNE",
            Body::Pluto => "PLUTO",
            Body::MeanNode => "MEAN_NODE",
            Body::TrueNode => "TRUE_NODE",
            Body::MeanApogee => "MEAN_APOGEE",
            Body::OsculatingApogee => "OSCULATING_APOGEE",
        }
    }

    /// The body with a key, in any case.
    #[must_use]
    pub fn from_key(key: &str) -> Option<Body> {
        let upper = key.to_ascii_uppercase();
        Body::ALL.iter().copied().find(|b| b.key() == upper)
    }

    /// Whether the body has a physical distance; nodes and apogees are
    /// directions, and a provider may report their distance as zero.
    #[must_use]
    pub const fn has_distance(self) -> bool {
        !matches!(
            self,
            Body::MeanNode | Body::TrueNode | Body::MeanApogee | Body::OsculatingApogee
        )
    }
}

impl fmt::Display for Body {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.key())
    }
}

impl FromStr for Body {
    type Err = Error;

    fn from_str(key: &str) -> Result<Body, Error> {
        Body::from_key(key).ok_or_else(|| {
            let upper = key.to_ascii_uppercase();
            let suggestion = Body::ALL
                .iter()
                .map(|b| (distance(&upper, b.key()), b.key()))
                .filter(|(d, _)| *d <= 2)
                .min_by_key(|(d, _)| *d)
                .map(|(_, k)| k);
            let error = Error::invalid_arg(format!("`{key}` is not a body the port knows"))
                .with_field("body");
            match suggestion {
                Some(k) => error.with_hint(format!("did you mean `{k}`?")),
                None => error,
            }
        })
    }
}

/// The time scale of the instants in a request.
///
/// ```
/// use teistro_port_ephemeris::TimeScale;
///
/// assert_eq!(TimeScale::from_id(TimeScale::Tt.id()), Some(TimeScale::Tt));
/// assert_eq!(TimeScale::Ut1.to_string(), "UT1");
/// ```
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimeScale {
    /// Universal Time (UT1), the scale of civil time and of rise and set.
    Ut1 = 0,
    /// Terrestrial Time, the scale of the ephemerides.
    Tt = 1,
}

impl TimeScale {
    /// The stable id at the C boundary: the discriminant.
    #[must_use]
    pub const fn id(self) -> u32 {
        self as u32
    }

    /// The scale with a given id.
    #[must_use]
    pub const fn from_id(id: u32) -> Option<TimeScale> {
        match id {
            0 => Some(TimeScale::Ut1),
            1 => Some(TimeScale::Tt),
            _ => None,
        }
    }

    /// The scale's name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            TimeScale::Ut1 => "UT1",
            TimeScale::Tt => "TT",
        }
    }
}

impl fmt::Display for TimeScale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    #[test]
    fn bodies_round_trip_through_ids_and_keys() {
        for body in Body::ALL {
            assert_eq!(Body::from_id(body.id()), Some(body));
            assert_eq!(body.key().parse::<Body>().unwrap(), body);
            assert_eq!(body.to_string(), body.key());
        }
        assert_eq!(Body::from_id(200), None);
        assert!(Body::PLANETS.iter().copied().all(Body::has_distance));
        let error = "MOONN".parse::<Body>().unwrap_err();
        assert_eq!(error.field(), Some("body"));
        assert_eq!(error.hint(), Some("did you mean `MOON`?"));
        assert!("xyz".parse::<Body>().unwrap_err().hint().is_none());
    }

    #[test]
    fn scales_round_trip() {
        for scale in [TimeScale::Ut1, TimeScale::Tt] {
            assert_eq!(TimeScale::from_id(scale.id()), Some(scale));
        }
        assert_eq!(TimeScale::from_id(7), None);
        assert_eq!(
            serde_json::to_string(&Body::MeanApogee).unwrap(),
            "\"MEAN_APOGEE\""
        );
    }
}
