//! The progressions a batch asks of every birth, as the bindings write them
//! (`03-design/western-progressions.md`, step 5).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_serial::Document;
use teistro_western::{AngleMethod, ArcMeasure, Progression, Rate, YearMeasure};

use crate::area::ChartArea;
use crate::hit_request::PointAsked;
use crate::progressed::{
    ContactRequest, Directed, DirectionArc, Progressed, ProgressedContact, ProgressionRequest,
};
use crate::reading::ChartRequest;

/// The record every binding writes the request as.
const PROGRESSIONS: &str = "progressions";

/// What a batch asks of each birth's progressions: the progressed chart and
/// the direction at one instant of life, the contacts over a window, or
/// both.
///
/// ```
/// use teistro::ProgressionsRequest;
/// use teistro::western::YearMeasure;
///
/// let asked = ProgressionsRequest::from_json(
///     r#"{"at": 2417505.5, "year": "NOON_SIDEREAL_TIME", "direction": "NAIBOD",
///         "contacts": {"from": 2417484.5, "to": 2417941.5, "grahas": ["MOON"], "aspects": [135]}}"#,
/// )?;
/// assert_eq!(asked.progressed.progression.year, YearMeasure::NoonSiderealTime);
/// assert!(asked.at.is_some() && asked.contacts.is_some());
/// // A refusal names the field the caller wrote.
/// let backwards = ProgressionsRequest::from_json(r#"{"contacts": {"from": 2, "to": 1}}"#).unwrap_err();
/// assert_eq!(backwards.field(), Some("progressions.contacts.to"));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ProgressionsRequest {
    /// The instant of life the progressed chart and the direction are read
    /// for; none for neither.
    pub at: Option<JulianDay<Utc>>,
    /// The measure and the angle method.
    pub progressed: ProgressionRequest,
    /// The direction's arc: the solar arc under `progressed`'s measure
    /// unless asked.
    pub direction: DirectionArc,
    /// The window of life and the contacts searched in it; none for none.
    pub contacts: Option<ContactWindow>,
}

/// A window of life and the contacts to search in it.
#[derive(Clone, Debug, PartialEq)]
pub struct ContactWindow {
    /// The window's start, an instant of life.
    pub from: JulianDay<Utc>,
    /// Its end, after the start.
    pub to: JulianDay<Utc>,
    /// The contacts searched; its progression is the request's.
    pub asked: ContactRequest,
}

/// One birth's progressions, each `None` where the request asked for none.
#[derive(Clone, Debug, PartialEq)]
pub struct Progressions {
    /// The progressed chart at the request's `at`.
    pub progressed: Option<Progressed>,
    /// The direction to the request's `at`.
    pub directed: Option<Directed>,
    /// The contacts in the request's window.
    pub contacts: Option<Vec<ProgressedContact>>,
}

impl ProgressionsRequest {
    /// Reads the request from JSON: `{"at", "rate", "year", "angles",
    /// "direction", "contacts"}`, every field optional but one of `at` and
    /// `contacts`. The measures are spelled as their members
    /// (`"NOON_SIDEREAL_TIME"`, `{"sky": "DAY", "life": "YEAR"}`), the
    /// direction `"SOLAR"`, `"NAIBOD"`, `"PTOLEMY"` or `{"PER_YEAR": 1}`,
    /// and the contacts `{"from", "to", "grahas", "points", "aspects"}` as
    /// the hit list spells them.
    ///
    /// # Errors
    ///
    /// Text that is not the record, a key it does not read, a request that
    /// asks for nothing, and whatever the measure, the arc or the contacts'
    /// request refuses, each named under `progressions`.
    pub fn from_json(text: &str) -> Result<ProgressionsRequest, Error> {
        let asked: Asked = teistro_core::strict::read(text, PROGRESSIONS)?;
        asked.request().map_err(|why| why.under(PROGRESSIONS))
    }
}

impl ChartArea<'_> {
    /// Everything a [`ProgressionsRequest`] asks of one birth: the
    /// progressed chart and the direction at its instant, and the contacts
    /// in its window, the progressed chart founded by `request`.
    ///
    /// # Errors
    ///
    /// Whatever [`ChartArea::progressed`], [`ChartArea::directed`] or
    /// [`ChartArea::progressed_contacts`] refuses.
    pub fn progressions(
        self,
        birth: &Document,
        asked: &ProgressionsRequest,
        request: &ChartRequest,
    ) -> Result<Progressions, Error> {
        let (progressed, directed) = match asked.at {
            Some(at) => (
                Some(self.progressed(birth, at, &asked.progressed, request)?),
                Some(self.directed(birth, at, &asked.direction, request)?),
            ),
            None => (None, None),
        };
        let contacts = match &asked.contacts {
            Some(window) => {
                Some(self.progressed_contacts(birth, window.from, window.to, &window.asked)?)
            }
            None => None,
        };
        Ok(Progressions {
            progressed,
            directed,
            contacts,
        })
    }
}

/// [`ProgressionsRequest`] as the bindings write it.
#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct Asked {
    #[serde(skip_serializing_if = "Option::is_none")]
    at: Option<f64>,
    rate: Rate,
    year: YearMeasure,
    angles: AngleMethod,
    direction: DirectionAsked,
    #[serde(skip_serializing_if = "Option::is_none")]
    contacts: Option<ContactsAsked>,
}

/// The direction's arc as a request spells it.
#[derive(Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum DirectionAsked {
    /// The Sun's arc under the request's measure.
    #[default]
    Solar,
    /// Naibod's measure.
    Naibod,
    /// Ptolemy's measure.
    Ptolemy,
    /// Any degrees a year.
    PerYear(f64),
}

/// The contacts as a request spells them.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContactsAsked {
    from: f64,
    to: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    grahas: Option<Vec<Graha>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    points: Option<Vec<PointAsked>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    aspects: Option<Vec<u16>>,
}

impl Asked {
    /// The request, checked; refusals named from the record's root.
    fn request(self) -> Result<ProgressionsRequest, Error> {
        if self.at.is_none() && self.contacts.is_none() {
            return Err(
                Error::invalid_arg("a progressions request asks for nothing")
                    .with_field("at")
                    .with_hint("name an instant of life as `at`, a window as `contacts`, or both"),
            );
        }
        let progression = Progression {
            rate: self.rate,
            year: self.year,
        };
        progression.check()?;
        let direction = match self.direction {
            DirectionAsked::Solar => DirectionArc::Solar(progression),
            DirectionAsked::Naibod => DirectionArc::Measure(ArcMeasure::Naibod),
            DirectionAsked::Ptolemy => DirectionArc::Measure(ArcMeasure::Ptolemy),
            DirectionAsked::PerYear(degrees) => {
                let measure = ArcMeasure::PerYear(degrees);
                measure
                    .degrees_per_year()
                    .map_err(|why| why.with_field("direction"))?;
                DirectionArc::Measure(measure)
            }
        };
        let at = self.at.map(|jd| instant(jd, "at")).transpose()?;
        let contacts = match self.contacts {
            Some(contacts) => Some(
                contacts
                    .window(progression)
                    .map_err(|why| why.under("contacts"))?,
            ),
            None => None,
        };
        Ok(ProgressionsRequest {
            at,
            progressed: ProgressionRequest {
                progression,
                angles: self.angles,
            },
            direction,
            contacts,
        })
    }
}

impl ContactsAsked {
    /// The window and its contacts' request, the window running forward.
    fn window(self, progression: Progression) -> Result<ContactWindow, Error> {
        let (from, to) = (instant(self.from, "from")?, instant(self.to, "to")?);
        if to.get() <= from.get() {
            return Err(Error::invalid_arg(format!(
                "a contact window must run forward, and {} is not after {}",
                to.get(),
                from.get()
            ))
            .with_field("to"));
        }
        let mut asked = ContactRequest::default().with_progression(progression);
        if let Some(grahas) = self.grahas {
            asked = asked.with_grahas(grahas);
        }
        if let Some(points) = self.points {
            asked = asked.with_points(points.into_iter().map(Into::into));
        }
        if let Some(aspects) = self.aspects {
            asked = asked.with_aspects(aspects);
        }
        // The planets, points and aspects are the hit list's, refused as
        // it refuses them; the window of sky it searches comes later.
        asked.hits_between(from, to).check()?;
        Ok(ContactWindow { from, to, asked })
    }
}

/// A UTC Julian day a request names, refused by its field.
fn instant(jd: f64, field: &str) -> Result<JulianDay<Utc>, Error> {
    JulianDay::<Utc>::try_new(jd).map_err(|why| Error::from(why).with_field(field))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    fn refused(text: &str) -> Error {
        ProgressionsRequest::from_json(text).unwrap_err()
    }

    #[test]
    fn an_absent_field_is_leos_default() {
        let asked = ProgressionsRequest::from_json(r#"{"at": 2417505.5}"#).unwrap();
        assert_eq!(asked.progressed, ProgressionRequest::default());
        assert_eq!(asked.direction, DirectionArc::default());
        assert_eq!(asked.contacts, None);
        let asked =
            ProgressionsRequest::from_json(r#"{"contacts": {"from": 1e6, "to": 2e6}}"#).unwrap();
        let window = asked.contacts.unwrap();
        assert_eq!(window.asked, ContactRequest::default());
    }

    #[test]
    fn every_measure_is_read_as_spelled() {
        let asked = ProgressionsRequest::from_json(
            r#"{"at": 2417505.5, "rate": {"sky": "DAY", "life": {"DAYS": 30}},
                "angles": "QUOTIDIAN", "direction": {"PER_YEAR": 1.5},
                "contacts": {"from": 1e6, "to": 2e6, "points": ["LAGNA", "graha.SUN"], "aspects": [45]}}"#,
        )
        .unwrap();
        assert_eq!(
            asked.progressed.progression.rate.life,
            teistro_western::Span::Days(30.0)
        );
        assert_eq!(asked.progressed.angles, AngleMethod::Quotidian);
        assert_eq!(
            asked.direction,
            DirectionArc::Measure(ArcMeasure::PerYear(1.5))
        );
        let window = asked.contacts.unwrap();
        assert_eq!(window.asked.aspects, [45]);
        assert_eq!(window.asked.points.len(), 2);
        assert_eq!(window.asked.progression, asked.progressed.progression);
    }

    #[test]
    fn a_refusal_names_the_field_under_progressions() {
        for (text, field) in [
            ("{}", "progressions.at"),
            (r#"{"at": 2417505.5, "colour": 1}"#, "progressions.colour"),
            (
                r#"{"at": 2417505.5, "year": "NOON_SIDEREAL_TIME", "rate": {"sky": "SYNODIC_MONTH", "life": "YEAR"}}"#,
                "progressions.year",
            ),
            (
                r#"{"at": 2417505.5, "rate": {"sky": {"DAYS": 0}, "life": "YEAR"}}"#,
                "progressions.rate.sky",
            ),
            (
                r#"{"at": 2417505.5, "direction": {"PER_YEAR": -1}}"#,
                "progressions.direction",
            ),
            (
                r#"{"contacts": {"from": 2e6, "to": 1e6}}"#,
                "progressions.contacts.to",
            ),
            (
                r#"{"contacts": {"from": 1e6, "to": 2e6, "points": ["NOWHERE"]}}"#,
                "progressions.contacts.points[0]",
            ),
            (
                r#"{"contacts": {"from": 1e6, "to": 2e6, "aspects": [200]}}"#,
                "progressions.contacts.aspects",
            ),
            (
                r#"{"contacts": {"from": 1e6, "to": 2e6, "grahas": ["URANUS"]}}"#,
                "progressions.contacts.grahas",
            ),
        ] {
            assert_eq!(refused(text).field(), Some(field), "{text}");
        }
    }
}
