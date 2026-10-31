//! A range's change points: every instant at which something on the page
//! changes, in one sorted list, read from the spans a range already found
//! (`docs/03-design/change-points.md`, SR-07).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Catalogued, Rashi, Vara};
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};

use crate::limb::Limbs;
use crate::span::Span;

/// What a change point changes. The order is the order two points on one
/// instant stand in.
///
/// ```
/// use teistro_panchanga::change::ChangeKind;
///
/// assert_eq!(serde_json::to_string(&ChangeKind::MoonSign).unwrap(), r#""MOON_SIGN""#);
/// assert!(ChangeKind::Tithi < ChangeKind::Karana);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChangeKind {
    /// The tithi.
    Tithi,
    /// The Moon's nakshatra.
    Nakshatra,
    /// The nitya yoga.
    Yoga,
    /// The karana.
    Karana,
    /// The Moon's sign.
    MoonSign,
    /// The Sun's sign: the sankranti.
    SunSign,
    /// The vara, at sunrise.
    Vara,
}

/// One instant at which one thing changes.
///
/// ```
/// use teistro_core::quantity::JulianDay;
/// use teistro_panchanga::change::{ChangeKind, ChangePoint};
///
/// let point = ChangePoint {
///     at: JulianDay::literal(2_461_345.5),
///     kind: ChangeKind::Tithi,
///     from: "SHUKLA_PRATIPADA".to_string(),
///     to: "SHUKLA_DWITIYA".to_string(),
/// };
/// assert_eq!(serde_json::to_value(&point).unwrap()["kind"], "TITHI");
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ChangePoint {
    /// The instant.
    pub at: JulianDay<Utc>,
    /// What changes.
    pub kind: ChangeKind,
    /// The catalogue key of the member that ended.
    pub from: String,
    /// The catalogue key of the member that began.
    pub to: String,
}

/// A range's change points.
///
/// ```
/// use teistro_core::interval::Interval;
/// use teistro_panchanga::change::ChangePoints;
///
/// let quiet = ChangePoints { points: Vec::new(), window: Interval::literal(100.25, 101.25) };
/// assert!(serde_json::to_string(&quiet).unwrap().contains(r#""points":[]"#));
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ChangePoints {
    /// Every point at or after the window's start and before its end, by
    /// instant; on one instant, by [`ChangeKind`].
    pub points: Vec<ChangePoint>,
    /// The range as instants: the first day's opening to the last's close.
    pub window: Interval,
}

/// What one day gives the list.
#[derive(Clone, Debug)]
pub(crate) struct ChangeDay {
    pub(crate) window: Interval,
    pub(crate) sunrise: JulianDay<Utc>,
    pub(crate) vara: Vara,
    pub(crate) limbs: Limbs,
    pub(crate) moon_signs: Vec<Span<Rashi>>,
    pub(crate) sun_signs: Vec<Span<Rashi>>,
}

/// The changes along one member list, day by day: a span crossing a day's
/// edge is met twice and is one span, so a change is a member differing
/// from the last one met. A member recurs only after others, so the
/// neighbour is enough.
fn walk<'a, T: Catalogued + 'a>(
    kind: ChangeKind,
    spans: impl Iterator<Item = &'a Span<T>>,
    within: Interval,
    points: &mut Vec<ChangePoint>,
) {
    let mut last: Option<T> = None;
    for span in spans {
        if let Some(before) = last.filter(|before| *before != span.member) {
            push(kind, before, span.member, span.whole.from, within, points);
        }
        last = Some(span.member);
    }
}

/// A point, when it lies in the range.
fn push<T: Catalogued>(
    kind: ChangeKind,
    from: T,
    to: T,
    at: JulianDay<Utc>,
    within: Interval,
    points: &mut Vec<ChangePoint>,
) {
    if within.from.get() <= at.get() && at.get() < within.to.get() {
        points.push(ChangePoint {
            at,
            kind,
            from: from.key().to_string(),
            to: to.key().to_string(),
        });
    }
}

impl ChangePoints {
    /// The list a range's days give.
    pub(crate) fn of(days: &[ChangeDay]) -> Result<ChangePoints, Error> {
        let (Some(first), Some(last)) = (days.first(), days.last()) else {
            return Err(Error::internal("a range of days answered no day"));
        };
        let window = Interval::new(first.window.from, last.window.to)?;
        let mut points = Vec::new();
        let limbs = || days.iter().map(|day| &day.limbs);
        walk(
            ChangeKind::Tithi,
            limbs().flat_map(|limbs| &limbs.tithi),
            window,
            &mut points,
        );
        walk(
            ChangeKind::Nakshatra,
            limbs().flat_map(|limbs| &limbs.nakshatra),
            window,
            &mut points,
        );
        walk(
            ChangeKind::Yoga,
            limbs().flat_map(|limbs| &limbs.yoga),
            window,
            &mut points,
        );
        walk(
            ChangeKind::Karana,
            limbs().flat_map(|limbs| &limbs.karana),
            window,
            &mut points,
        );
        walk(
            ChangeKind::MoonSign,
            days.iter().flat_map(|day| &day.moon_signs),
            window,
            &mut points,
        );
        walk(
            ChangeKind::SunSign,
            days.iter().flat_map(|day| &day.sun_signs),
            window,
            &mut points,
        );
        for pair in days.windows(2) {
            if let [before, after] = pair {
                if before.vara != after.vara {
                    push(
                        ChangeKind::Vara,
                        before.vara,
                        after.vara,
                        after.sunrise,
                        window,
                        &mut points,
                    );
                }
            }
        }
        points.sort_by(|a, b| {
            a.at.get()
                .total_cmp(&b.at.get())
                .then_with(|| a.kind.cmp(&b.kind))
        });
        Ok(ChangePoints { points, window })
    }
}
