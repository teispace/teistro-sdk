//! Where two reckonings of the same days disagree, and why
//! (`03-design/setup-disagreements.md`): the official day an app shows
//! beside the modern one, with the instants that explain each difference.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use teistro_calendar::CalendarDate;
use teistro_core::catalogue::{
    Calendar, Karana, Kind, Masa, Nakshatra, Tithi, Yoga, write_in_full,
};
use teistro_core::envelope::{Envelope, Hash};
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_core::time::UtcOffset;
use teistro_panchanga::almanac::Panchanga;
use teistro_panchanga::festival::{EkadashiFast, Observance, Observances};
use teistro_panchanga::span::Span;

use super::AlmanacArea;
use crate::asked::{DayAsked, offset_of, place_of};
use crate::context::Context;
use crate::festival_request::FestivalRequest;

/// Minutes in a day.
const MINUTES: f64 = 1440.0;

/// One limb of one day as one reckoning names it at its sunrise.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "Reckoned{T}"))]
#[serde(rename_all = "camelCase")]
pub struct ReckonedLimb<T> {
    /// The member standing at this reckoning's sunrise.
    pub member: T,
    /// That member's own span under this reckoning's sky.
    pub span: Interval,
    /// This reckoning's sunrise.
    pub sunrise: JulianDay<Utc>,
    /// Where the two reckonings' members meet under this sky, when one
    /// follows the other: the reason, as an instant against the sunrise.
    pub change: Option<LimbChange>,
}

/// The instant one reckoning's member gives way to the other's, under one
/// sky.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct LimbChange {
    /// The instant.
    pub at: JulianDay<Utc>,
    /// Minutes from that sky's sunrise to it, negative before sunrise.
    pub minutes_from_sunrise: f64,
}

/// One limb both reckonings name differently at sunrise.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "{T}Disagreement"))]
#[serde(rename_all = "camelCase")]
pub struct LimbDisagreement<T> {
    /// As the first context reckons it: the day the app shows.
    pub official: ReckonedLimb<T>,
    /// As the context it is compared with reckons it.
    pub other: ReckonedLimb<T>,
}

/// A day on which some limb's member at sunrise differs.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct DayDisagreement {
    /// The civil day.
    pub date: CalendarDate,
    /// The tithi, when it differs.
    pub tithi: Option<LimbDisagreement<Tithi>>,
    /// The nakshatra, when it differs.
    pub nakshatra: Option<LimbDisagreement<Nakshatra>>,
    /// The nitya yoga, when it differs.
    pub yoga: Option<LimbDisagreement<Yoga>>,
    /// The karana, when it differs.
    pub karana: Option<LimbDisagreement<Karana>>,
}

/// The days two reckonings disagree on, and what the other one was.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct DayDisagreements {
    /// The profile the other context resolved.
    pub other_profile: String,
    /// The other context's settings hash, so a stored comparison is known
    /// stale when either side's settings change.
    pub other_settings_hash: Hash,
    /// How many days were compared.
    pub compared: usize,
    /// The days that differ, in order.
    pub days: Vec<DayDisagreement>,
}

/// One rule's occurrence both reckonings give a different day or file
/// under a different month, or one gives and the other does not.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct FestivalDisagreement {
    /// The rule's key.
    pub rule: String,
    /// The occurrence's amanta month as the first context names it, or as
    /// the other does when only it gives one. Each side's own is in its
    /// record: the two differ where the skies place an adhika month apart.
    pub month: Masa,
    /// Whether that month is adhika, the same way.
    pub adhika: bool,
    /// The first context's day and why, with each day's extent of the
    /// rite's window, which is the reason; absent when it gives none.
    pub official: Option<Observance>,
    /// The other context's.
    pub other: Option<Observance>,
}

/// One Ekadashi both reckonings fast on another day or name for another
/// month, or one fasts and the other does not.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct EkadashiDisagreement {
    /// The rule's key.
    pub rule: String,
    /// The Ekadashi.
    pub tithi: Tithi,
    /// Its amanta month as the first context names it, or as the other
    /// does when only it fasts; each side's own is in its record.
    pub month: Masa,
    /// Whether that month is adhika, the same way.
    pub adhika: bool,
    /// The first context's fast, with the vedha and excess that decided.
    pub official: Option<EkadashiFast>,
    /// The other context's.
    pub other: Option<EkadashiFast>,
}

/// The festivals two reckonings disagree on.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct FestivalDisagreements {
    /// The profile the other context resolved.
    pub other_profile: String,
    /// The other context's settings hash.
    pub other_settings_hash: Hash,
    /// How many observances and fasts the first context gave.
    pub compared: usize,
    /// The observances that differ, in the first context's order, then
    /// the ones only the other gives.
    pub observances: Vec<FestivalDisagreement>,
    /// The Ekadashi fasts that differ, the same way.
    pub ekadashis: Vec<EkadashiDisagreement>,
}

/// One limb's disagreement on a day, or none when both name one member.
fn limb<T: Copy + PartialEq>(
    ours: &[Span<T>],
    ours_sunrise: JulianDay<Utc>,
    theirs: &[Span<T>],
    theirs_sunrise: JulianDay<Utc>,
) -> Option<LimbDisagreement<T>> {
    let (mine, other) = (ours.first()?, theirs.first()?);
    if mine.member == other.member {
        return None;
    }
    let change = |at: JulianDay<Utc>, sunrise: JulianDay<Utc>| LimbChange {
        at,
        minutes_from_sunrise: (at.get() - sunrise.get()) * MINUTES,
    };
    // Which side runs ahead: the other names the member after ours (ours
    // still stands at our sunrise, and had ended by theirs), or the one
    // before. A pair that is neither -- a member one sky skips -- has no
    // single instant to blame.
    let (official, theirs_change) = if ours.get(1).is_some_and(|s| s.member == other.member) {
        (
            Some(change(mine.whole.to, ours_sunrise)),
            Some(change(other.whole.from, theirs_sunrise)),
        )
    } else if theirs.get(1).is_some_and(|s| s.member == mine.member) {
        (
            Some(change(mine.whole.from, ours_sunrise)),
            Some(change(other.whole.to, theirs_sunrise)),
        )
    } else {
        (None, None)
    };
    Some(LimbDisagreement {
        official: ReckonedLimb {
            member: mine.member,
            span: mine.whole,
            sunrise: ours_sunrise,
            change: official,
        },
        other: ReckonedLimb {
            member: other.member,
            span: other.whole,
            sunrise: theirs_sunrise,
            change: theirs_change,
        },
    })
}

/// The disagreement of one day, or none.
pub(crate) fn day(ours: &Panchanga, theirs: &Panchanga) -> Option<DayDisagreement> {
    let (a, b) = (ours.day.sunrise, theirs.day.sunrise);
    let found = DayDisagreement {
        date: ours.day.date.clone(),
        tithi: limb(&ours.limbs.tithi, a, &theirs.limbs.tithi, b),
        nakshatra: limb(&ours.limbs.nakshatra, a, &theirs.limbs.nakshatra, b),
        yoga: limb(&ours.limbs.yoga, a, &theirs.limbs.yoga, b),
        karana: limb(&ours.limbs.karana, a, &theirs.limbs.karana, b),
    };
    (found.tithi.is_some()
        || found.nakshatra.is_some()
        || found.yoga.is_some()
        || found.karana.is_some())
    .then_some(found)
}

/// Whether two spans of one occurrence, one under each sky, overlap: the
/// two reckonings move a tithi by minutes or hours, never by a day, so the
/// same occurrence overlaps itself and the next one does not.
fn overlap(a: Interval, b: Interval) -> bool {
    a.from.get() < b.to.get() && b.from.get() < a.to.get()
}

/// Pairs each of `ours` with the occurrence of `theirs` it is (`same`),
/// and keeps the pairs that differ (`differs`) and the unpaired on either
/// side, built into a record by `pair` from the side that is there and
/// both sides.
fn paired<A: Clone, R>(
    ours: &[A],
    theirs: &[A],
    same: impl Fn(&A, &A) -> bool,
    differs: impl Fn(&A, &A) -> bool,
    pair: impl Fn(&A, Option<A>, Option<A>) -> R,
) -> Vec<R> {
    let mut taken = vec![false; theirs.len()];
    let mut found = Vec::new();
    for mine in ours {
        let partner = theirs
            .iter()
            .zip(taken.iter_mut())
            .find(|(other, taken)| !**taken && same(mine, other));
        match partner {
            Some((other, taken)) => {
                *taken = true;
                if differs(mine, other) {
                    found.push(pair(mine, Some(mine.clone()), Some(other.clone())));
                }
            }
            None => found.push(pair(mine, Some(mine.clone()), None)),
        }
    }
    for (other, _) in theirs.iter().zip(&taken).filter(|(_, taken)| !**taken) {
        found.push(pair(other, None, Some(other.clone())));
    }
    found
}

/// The observances and fasts two answers give differently.
pub(crate) fn festivals(
    ours: &Observances,
    theirs: &Observances,
) -> (Vec<FestivalDisagreement>, Vec<EkadashiDisagreement>) {
    let observances = paired(
        &ours.observances,
        &theirs.observances,
        // One occurrence is one rule over one tithi, whatever month each sky
        // files it under: in 2028 the text has an adhika Kartika the modern
        // sky lacks, and every month to January 2029 is named one apart.
        // Paired by label, each fast there read as two one-sided ones.
        |a, b| a.rule == b.rule && overlap(a.tithi, b.tithi),
        |a, b| a.day != b.day || a.month != b.month || a.adhika != b.adhika,
        |known: &Observance, official, other| FestivalDisagreement {
            rule: known.rule.clone(),
            month: known.month,
            adhika: known.adhika,
            official,
            other,
        },
    );
    let ekadashis = paired(
        &ours.ekadashis,
        &theirs.ekadashis,
        |a, b| a.rule == b.rule && a.tithi == b.tithi && overlap(a.tithis[1], b.tithis[1]),
        |a, b| a.day != b.day || a.month != b.month || a.adhika != b.adhika,
        |known: &EkadashiFast, official, other| EkadashiDisagreement {
            rule: known.rule.clone(),
            tithi: known.tithi,
            month: known.month,
            adhika: known.adhika,
            official,
            other,
        },
    );
    (observances, ekadashis)
}

impl AlmanacArea<'_> {
    /// **Where this reckoning and another disagree**, day by day: the days
    /// between two dates, inclusive, on which the two name another member
    /// of some limb at sunrise, each limb with the member each names, its
    /// span, the sunrise, and the instant the two members meet under each
    /// sky (`03-design/setup-disagreements.md`).
    ///
    /// The app's reckoning is this context's, `official` in the record;
    /// `other` is the one it explains. Both are reckoned at one place on
    /// one clock, each in one crossing.
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::of`], for either context: a range of more than
    /// 366 days among them.
    pub fn disagreements(
        self,
        other: &Context,
        from: &CalendarDate,
        to: &CalendarDate,
        place: &Place,
        offset: UtcOffset,
    ) -> Result<Envelope<DayDisagreements>, Error> {
        let ours = self.of(from, to, place, offset)?;
        let theirs = other.almanac().of(from, to, place, offset)?;
        let days = ours
            .value
            .iter()
            .zip(&theirs.value)
            .filter_map(|(a, b)| day(a, b))
            .collect();
        Ok(Envelope::sealing(
            DayDisagreements {
                other_profile: other.profile().to_owned(),
                other_settings_hash: other.settings_hash(),
                compared: ours.value.len(),
                days,
            },
            ours.provenance,
        ))
    }

    /// **Where this reckoning and another put a festival on another day**:
    /// one festival request under both contexts, and each occurrence whose
    /// day differs or which only one of them gives, with each side's
    /// observance. An observance carries the extent of the rite's window
    /// the tithi held on each of its two days, which is the reason.
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::festivals`], for either context.
    pub fn festival_disagreements(
        self,
        other: &Context,
        from: &CalendarDate,
        to: &CalendarDate,
        place: &Place,
        offset: UtcOffset,
        request: &FestivalRequest,
    ) -> Result<Envelope<FestivalDisagreements>, Error> {
        let ours = self.festivals(from, to, place, offset, request)?;
        let theirs = other
            .almanac()
            .festivals(from, to, place, offset, request)?;
        let (observances, ekadashis) = festivals(&ours.value, &theirs.value);
        Ok(Envelope::sealing(
            FestivalDisagreements {
                other_profile: other.profile().to_owned(),
                other_settings_hash: other.settings_hash(),
                compared: ours.value.observances.len() + ours.value.ekadashis.len(),
                observances,
                ekadashis,
            },
            ours.provenance,
        ))
    }
}

/// The record's name where a binding sends it, which a refusal is named
/// under.
const DISAGREEMENTS: &str = "disagreements";

/// Where each catalogue member of a [`DayDisagreements`] sits, written in
/// full at the boundary as every other section's are.
const DAY_MEMBERS: [(&str, &str, Kind); 10] = [
    ("days", "date.calendar", Kind::Calendar),
    ("days", "date.era.era", Kind::Era),
    ("days", "tithi.official.member", Kind::Tithi),
    ("days", "tithi.other.member", Kind::Tithi),
    ("days", "nakshatra.official.member", Kind::Nakshatra),
    ("days", "nakshatra.other.member", Kind::Nakshatra),
    ("days", "yoga.official.member", Kind::Yoga),
    ("days", "yoga.other.member", Kind::Yoga),
    ("days", "karana.official.member", Kind::Karana),
    ("days", "karana.other.member", Kind::Karana),
];

impl DayDisagreements {
    /// The record as JSON, every catalogue member written as its full key:
    /// what the boundary carries and seals.
    ///
    /// # Errors
    ///
    /// `INTERNAL` if the record does not serialise, which one this crate
    /// built cannot do.
    pub fn in_full(&self) -> Result<Value, Error> {
        let mut value = serde_json::to_value(self).map_err(|error| {
            Error::internal(format!("the day disagreements did not serialise: {error}"))
        })?;
        for (list, path, kind) in DAY_MEMBERS {
            if let Some(items) = value.get_mut(list) {
                write_in_full(items, path, kind);
            }
        }
        Ok(value)
    }
}

impl FestivalDisagreements {
    /// The record as JSON, every catalogue member written as its full key,
    /// each side's observance or fast as the festival answer writes it:
    /// what the boundary carries and seals.
    ///
    /// # Errors
    ///
    /// `INTERNAL` if the record does not serialise, which one this crate
    /// built cannot do.
    pub fn in_full(&self) -> Result<Value, Error> {
        let mut value = serde_json::to_value(self).map_err(|error| {
            Error::internal(format!(
                "the festival disagreements did not serialise: {error}"
            ))
        })?;
        for list in ["observances", "ekadashis"] {
            let Some(items) = value.get_mut(list) else {
                continue;
            };
            write_in_full(items, "month", Kind::Masa);
            if list == "ekadashis" {
                write_in_full(items, "tithi", Kind::Tithi);
            }
            for (within, path, kind) in Observances::MEMBERS {
                if within == list {
                    write_in_full(items, &format!("official.{path}"), kind);
                    write_in_full(items, &format!("other.{path}"), kind);
                }
            }
        }
        Ok(value)
    }
}

/// Both comparisons of two reckonings a binding asks at once
/// (`03-design/setup-disagreements.md` §5): the days, the festivals, or
/// both, over one range at one place on one clock.
#[derive(Clone, Debug, PartialEq)]
pub struct DisagreementRequest {
    /// The first day.
    pub from: CalendarDate,
    /// The last day, inclusive.
    pub to: CalendarDate,
    /// The place both reckonings are at.
    pub place: Place,
    /// The offset their civil days run at.
    pub offset: UtcOffset,
    /// Whether the days are compared.
    pub days: bool,
    /// The festivals compared, when asked.
    pub festivals: Option<FestivalRequest>,
}

/// The request as a binding writes it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "DisagreementRequest"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct DisagreementAsked {
    #[serde(default)]
    calendar: Option<Calendar>,
    first: DayAsked,
    #[serde(default)]
    last: Option<DayAsked>,
    latitude_deg: f64,
    longitude_deg: f64,
    #[serde(default)]
    altitude_m: f64,
    utc_offset_seconds: i32,
    #[serde(default = "compared")]
    days: bool,
    #[serde(default)]
    festivals: Option<Value>,
}

/// The days are compared unless a request says not.
const fn compared() -> bool {
    true
}

impl DisagreementRequest {
    /// The record a binding sends, as JSON: `first` (and `last`, the first
    /// when left out) as `{"year", "month", "day"}` in `calendar`
    /// (Gregorian when left out), `latitudeDeg`, `longitudeDeg`,
    /// `altitudeM` and `utcOffsetSeconds`; `days`, true when left out; and
    /// `festivals`, a festival request, optional.
    ///
    /// ```
    /// use teistro::DisagreementRequest;
    ///
    /// let asked = DisagreementRequest::from_json(
    ///     r#"{"first": {"year": 2025, "month": 10, "day": 1},
    ///         "last": {"year": 2025, "month": 10, "day": 31},
    ///         "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700,
    ///         "days": false, "festivals": {"rules": "NEPAL"}}"#,
    /// )?;
    /// assert!(!asked.days && asked.festivals.is_some());
    /// let nothing = DisagreementRequest::from_json(
    ///     r#"{"first": {"year": 2025, "month": 10, "day": 1},
    ///         "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700,
    ///         "days": false}"#,
    /// )
    /// .unwrap_err();
    /// assert_eq!(nothing.field(), Some("disagreements.days"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, a place or offset out of range, a festival request its own
    /// reader refuses, or a request that compares nothing, each named
    /// under `disagreements`.
    pub fn from_json(text: &str) -> Result<DisagreementRequest, Error> {
        let asked: DisagreementAsked = teistro_core::strict::read(text, DISAGREEMENTS)?;
        asked.request().map_err(|why| why.under(DISAGREEMENTS))
    }
}

impl DisagreementAsked {
    fn request(self) -> Result<DisagreementRequest, Error> {
        let calendar = self.calendar.unwrap_or(Calendar::Gregorian);
        let festivals = self
            .festivals
            .map(|asked| FestivalRequest::from_json(&asked.to_string()))
            .transpose()?;
        if !self.days && festivals.is_none() {
            return Err(Error::invalid_arg(
                "a comparison asks for the days, the festivals or both",
            )
            .with_field("days"));
        }
        Ok(DisagreementRequest {
            from: self.first.in_calendar(calendar),
            to: self.last.unwrap_or(self.first).in_calendar(calendar),
            place: place_of(self.latitude_deg, self.longitude_deg, self.altitude_m)?,
            offset: offset_of(self.utc_offset_seconds)?,
            days: self.days,
            festivals,
        })
    }
}

/// What a [`DisagreementRequest`] answers: each comparison it asked,
/// sealed by the first context's provenance, as the boundary writes it.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Disagreements {
    /// The days, full-keyed, when asked.
    pub days: Option<Envelope<Value>>,
    /// The festivals, full-keyed, when asked.
    pub festivals: Option<Envelope<Value>>,
}

impl AlmanacArea<'_> {
    /// As [`AlmanacArea::disagreements`] and
    /// [`AlmanacArea::festival_disagreements`], for a request a binding
    /// sent: each record written in full and sealed as the boundary
    /// carries it.
    ///
    /// # Errors
    ///
    /// As the two comparisons, for either context.
    pub fn disagreements_request(
        self,
        other: &Context,
        asked: &DisagreementRequest,
    ) -> Result<Disagreements, Error> {
        let days = if asked.days {
            let Envelope { value, provenance } =
                self.disagreements(other, &asked.from, &asked.to, &asked.place, asked.offset)?;
            Some(Envelope::sealing(value.in_full()?, provenance))
        } else {
            None
        };
        let festivals = match &asked.festivals {
            Some(request) => {
                let Envelope { value, provenance } = self.festival_disagreements(
                    other,
                    &asked.from,
                    &asked.to,
                    &asked.place,
                    asked.offset,
                    request,
                )?;
                Some(Envelope::sealing(value.in_full()?, provenance))
            }
            None => None,
        };
        Ok(Disagreements { days, festivals })
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;

    fn jd(at: f64) -> JulianDay<Utc> {
        JulianDay::try_new(at).unwrap()
    }

    fn span<T>(member: T, from: f64, to: f64) -> Span<T> {
        let whole = Interval {
            from: jd(from),
            to: jd(to),
        };
        Span {
            member,
            whole,
            inside: whole,
        }
    }

    #[test]
    fn one_member_on_both_sides_is_no_disagreement() {
        let ours = [span(Tithi::ShuklaDashami, 0.0, 1.0)];
        assert_eq!(limb(&ours, jd(0.5), &ours, jd(0.5)), None);
    }

    #[test]
    fn a_reckoning_running_ahead_is_blamed_on_its_own_start_and_the_others_end() {
        // Ours already has the 11th at our sunrise (began 0.2 before it);
        // theirs still has the 10th, ending 0.1 after theirs.
        let ours = [span(Tithi::ShuklaEkadashi, 10.3, 11.3)];
        let theirs = [
            span(Tithi::ShuklaDashami, 9.6, 10.6),
            span(Tithi::ShuklaEkadashi, 10.6, 11.6),
        ];
        let found = limb(&ours, jd(10.5), &theirs, jd(10.5)).unwrap();
        assert_eq!(found.official.member, Tithi::ShuklaEkadashi);
        assert_eq!(found.other.member, Tithi::ShuklaDashami);
        let (a, b) = (found.official.change.unwrap(), found.other.change.unwrap());
        assert_eq!(a.at, jd(10.3));
        assert_eq!(b.at, jd(10.6));
        assert!((a.minutes_from_sunrise + 0.2 * MINUTES).abs() < 1e-6);
        assert!((b.minutes_from_sunrise - 0.1 * MINUTES).abs() < 1e-6);
    }

    #[test]
    fn a_reckoning_running_behind_is_blamed_on_its_own_end_and_the_others_start() {
        let ours = [
            span(Tithi::ShuklaDashami, 9.6, 10.6),
            span(Tithi::ShuklaEkadashi, 10.6, 11.6),
        ];
        let theirs = [span(Tithi::ShuklaEkadashi, 10.3, 11.3)];
        let found = limb(&ours, jd(10.5), &theirs, jd(10.5)).unwrap();
        assert_eq!(found.official.change.unwrap().at, jd(10.6));
        assert_eq!(found.other.change.unwrap().at, jd(10.3));
    }

    #[test]
    fn members_that_do_not_follow_each_other_name_no_instant() {
        let ours = [span(Tithi::ShuklaDashami, 9.6, 10.6)];
        let theirs = [span(Tithi::ShuklaDwadashi, 10.3, 11.3)];
        let found = limb(&ours, jd(10.5), &theirs, jd(10.5)).unwrap();
        assert_eq!((found.official.change, found.other.change), (None, None));
    }

    #[test]
    fn pairing_keeps_moved_and_unpaired_occurrences_and_drops_agreeing_ones() {
        // (rule, day, start): the same occurrence overlaps itself.
        type Row = (&'static str, u8, f64);
        let ours: [Row; 3] = [("a", 1, 0.0), ("b", 5, 4.0), ("c", 9, 8.0)];
        let theirs: [Row; 3] = [("a", 1, 0.1), ("b", 6, 4.2), ("d", 12, 11.0)];
        let found = paired(
            &ours,
            &theirs,
            |x, y| x.0 == y.0 && (x.2 - y.2).abs() < 0.5,
            |x, y| x.1 != y.1,
            |known, a, b| (known.0, a.map(|r| r.1), b.map(|r| r.1)),
        );
        assert_eq!(
            found,
            [
                ("b", Some(5), Some(6)),
                ("c", Some(9), None),
                ("d", None, Some(12))
            ]
        );
    }
}
