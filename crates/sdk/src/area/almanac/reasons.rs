//! Why a published day differs from the rule's
//! (`03-design/published-overrides.md` §4): the request asked again under
//! one change at a time, the publisher's place, the publisher's sky and
//! each shipped pack's rule of the same key, and the first that gives the
//! published day named as the reason.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use teistro_calendar::CalendarDate;
use teistro_core::catalogue::{Calendar, Kind, write_in_full};
use teistro_core::envelope::{Envelope, Hash};
use teistro_core::error::Error;
use teistro_core::quantity::Place;
use teistro_core::time::UtcOffset;
use teistro_panchanga::festival::{Observances, Published, PublishedStatus};

use super::AlmanacArea;
use crate::asked::{DayAsked, offset_of, place_of};
use crate::context::Context;
use crate::festival_request::{FestivalPack, FestivalRequest};

/// A place a publisher may have reckoned its days at, and its clock.
#[derive(Clone, Debug, PartialEq)]
pub struct ReckonedPlace {
    /// What the answer calls it (`Kathmandu`).
    pub name: String,
    /// The place.
    pub place: Place,
    /// The clock its civil days run on.
    pub offset: UtcOffset,
}

/// One change a published day was tried under, or the reason it differs.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "by", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PublishedCause {
    /// The rule reckoned at another place.
    #[serde(rename_all = "camelCase")]
    Place {
        /// The place's name, as asked.
        name: String,
    },
    /// The rule reckoned under another context's settings and ephemeris.
    #[serde(rename_all = "camelCase")]
    Sky {
        /// That context's profile.
        profile: String,
        /// Its settings' hash, which tells two contexts of one profile
        /// apart.
        settings_hash: Hash,
    },
    /// Another shipped pack's rule of the same key.
    #[serde(rename_all = "camelCase")]
    Rule {
        /// The pack, by its key.
        pack: &'static str,
    },
    /// No change tried gives the published day: the publisher's own
    /// choice. Never a change tried.
    Publisher,
}

/// One change tried, and the rule's day under it.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ChangeTried {
    /// The change.
    pub change: PublishedCause,
    /// The rule's day under it; absent when it gives no occurrence within
    /// seven days of the published one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub day: Option<CalendarDate>,
    /// Whether that is the published day.
    pub reproduces: bool,
}

/// Why one published day differs from the rule's here.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PublishedReason {
    /// The rule's key.
    pub rule: String,
    /// The set that published it.
    pub set: String,
    /// The published day.
    pub day: CalendarDate,
    /// The rule's own day, here and under this context.
    pub computed: CalendarDate,
    /// The first change that gives the published day, in the order
    /// tried; [`PublishedCause::Publisher`] when none does.
    pub reason: PublishedCause,
    /// Every change tried, in order, with the day each gave, so a
    /// "publisher's own" is shown and not asserted.
    pub tried: Vec<ChangeTried>,
}

/// Why each published day of an answer differs from the rule's.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PublishedReasons {
    /// One per occurrence a `SET` moved, in the answer's order: the
    /// observances, the Ekadashi fasts, then the dated observances.
    pub reasons: Vec<PublishedReason>,
}

/// A moved occurrence: the rule, the set, the published day and the rule's.
struct Moved {
    rule: String,
    set: String,
    day: CalendarDate,
    computed: CalendarDate,
}

/// Every occurrence of `answer` a `SET` moved, in its order.
fn moved(answer: &Observances) -> Vec<Moved> {
    let observances = answer
        .observances
        .iter()
        .map(|o| (&o.rule, &o.day, &o.published));
    let fasts = answer
        .ekadashis
        .iter()
        .map(|f| (&f.rule, &f.day, &f.published));
    let dated = answer.dated.iter().map(|d| (&d.rule, &d.day, &d.published));
    observances
        .chain(fasts)
        .chain(dated)
        .filter_map(|(rule, day, published)| match published {
            Some(Published {
                status: PublishedStatus::Published,
                set,
                computed: Some(computed),
                ..
            }) => Some(Moved {
                rule: rule.clone(),
                set: set.clone(),
                day: day.clone(),
                computed: computed.clone(),
            }),
            _ => None,
        })
        .collect()
}

/// The rule's day under one change: the occurrence the same entry placed,
/// found by its rule, set and published day, whose `computed` is the
/// rule's day when it differs. `None` when the entry placed nothing.
fn under(answer: &Observances, of: &Moved) -> Option<CalendarDate> {
    let observances = answer
        .observances
        .iter()
        .map(|o| (&o.rule, &o.day, &o.published));
    let fasts = answer
        .ekadashis
        .iter()
        .map(|f| (&f.rule, &f.day, &f.published));
    let dated = answer.dated.iter().map(|d| (&d.rule, &d.day, &d.published));
    observances
        .chain(fasts)
        .chain(dated)
        .find_map(|(rule, day, published)| match published {
            Some(p) if *rule == of.rule && p.set == of.set && *day == of.day => {
                Some(p.computed.clone().unwrap_or_else(|| day.clone()))
            }
            _ => None,
        })
}

/// The request with `pack`'s rule of `key` in place of its own, or `None`
/// when the pack has no rule of that key or the same one.
fn with_packs_rule(
    request: &FestivalRequest,
    pack: FestivalPack,
    key: &str,
) -> Option<FestivalRequest> {
    if let Some(rule) = pack.rules().into_iter().find(|r| r.key == key) {
        let own = request.rules().iter().find(|r| r.key == key);
        return (own != Some(&rule)).then(|| request.clone().with_rule(rule));
    }
    if let Some(rule) = pack.ekadashis().into_iter().find(|r| r.key == key) {
        let own = request.ekadashis().iter().find(|r| r.key == key);
        return (own != Some(&rule)).then(|| request.clone().with_ekadashi(rule));
    }
    if let Some(rule) = pack.following().into_iter().find(|r| r.key == key) {
        let own = request.following().iter().find(|r| r.key == key);
        return (own != Some(&rule)).then(|| request.clone().with_following(rule));
    }
    let rule = pack.dated().into_iter().find(|r| r.key == key)?;
    let own = request.dated().iter().find(|r| r.key == key);
    (own != Some(&rule)).then(|| request.clone().with_dated(rule))
}

/// The record's name where a binding sends it, which a refusal is named
/// under.
pub(crate) const PUBLISHED_REASONS: &str = "publishedReasons";

impl AlmanacArea<'_> {
    /// **Why each published day differs from the rule's**
    /// (`03-design/published-overrides.md` §4). The request, with its
    /// published sets, is asked again under one change at a time: each of
    /// `places` the publisher may have reckoned at, each of `skies` (another
    /// context: the committee's Surya Siddhanta beside a modern sky), then
    /// each shipped pack's rule of the same key. The first change whose
    /// rule's day is the published one is the reason; when none is, the
    /// reason is the publisher's own choice. Every change tried is listed
    /// with the day it gave.
    ///
    /// Only occurrences a `SET` moved are explained: a `WITHDRAW` keeps
    /// the rule's day, and an `ADD` has no rule.
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::festivals`], for this context, each place and
    /// each sky.
    pub fn published_reasons(
        self,
        (from, to): (&CalendarDate, &CalendarDate),
        (place, offset): (&Place, UtcOffset),
        request: &FestivalRequest,
        places: &[ReckonedPlace],
        skies: &[&Context],
    ) -> Result<Envelope<PublishedReasons>, Error> {
        let asked = self.festivals(from, to, place, offset, request)?;
        let moved = moved(&asked.value);
        if moved.is_empty() {
            return Ok(Envelope::sealing(
                PublishedReasons {
                    reasons: Vec::new(),
                },
                asked.provenance,
            ));
        }
        // Each change asked once, however many days it explains.
        let mut changes: Vec<(PublishedCause, Observances)> = Vec::new();
        for at in places {
            let answer = self.festivals(from, to, &at.place, at.offset, request)?;
            changes.push((
                PublishedCause::Place {
                    name: at.name.clone(),
                },
                answer.value,
            ));
        }
        for sky in skies {
            let answer = sky.almanac().festivals(from, to, place, offset, request)?;
            changes.push((
                PublishedCause::Sky {
                    profile: sky.profile().to_owned(),
                    settings_hash: sky.settings_hash(),
                },
                answer.value,
            ));
        }
        // Each pack asked once with its rule in place of the request's for
        // every moved key it has a different rule for.
        let mut keys: Vec<String> = moved.iter().map(|m| m.rule.clone()).collect();
        keys.sort_unstable();
        keys.dedup();
        let mut by_pack: Vec<(Vec<String>, PublishedCause, Observances)> = Vec::new();
        for pack in FestivalPack::ALL {
            let mut other = request.clone();
            let mut replaced = Vec::new();
            for key in &keys {
                if let Some(with) = with_packs_rule(&other, pack, key) {
                    other = with;
                    replaced.push(key.clone());
                }
            }
            if !replaced.is_empty() {
                let answer = self.festivals(from, to, place, offset, &other)?;
                by_pack.push((
                    replaced,
                    PublishedCause::Rule { pack: pack.key() },
                    answer.value,
                ));
            }
        }
        let reasons = moved
            .into_iter()
            .map(|of| {
                let tried: Vec<ChangeTried> = changes
                    .iter()
                    .map(|(change, answer)| (change, answer))
                    .chain(
                        by_pack
                            .iter()
                            .filter(|(keys, ..)| keys.contains(&of.rule))
                            .map(|(_, change, answer)| (change, answer)),
                    )
                    .map(|(change, answer)| {
                        let day = under(answer, &of);
                        ChangeTried {
                            change: change.clone(),
                            reproduces: day.as_ref() == Some(&of.day),
                            day,
                        }
                    })
                    .collect();
                let reason = tried
                    .iter()
                    .find(|t| t.reproduces)
                    .map_or(PublishedCause::Publisher, |t| t.change.clone());
                PublishedReason {
                    rule: of.rule,
                    set: of.set,
                    day: of.day,
                    computed: of.computed,
                    reason,
                    tried,
                }
            })
            .collect();
        Ok(Envelope::sealing(
            PublishedReasons { reasons },
            asked.provenance,
        ))
    }
}

/// Where each catalogue member of a [`PublishedReasons`] sits, written in
/// full at the boundary as every other section's are.
const MEMBERS: [(&str, &str, Kind); 6] = [
    ("reasons", "day.calendar", Kind::Calendar),
    ("reasons", "day.era.era", Kind::Era),
    ("reasons", "computed.calendar", Kind::Calendar),
    ("reasons", "computed.era.era", Kind::Era),
    ("reasons", "tried.day.calendar", Kind::Calendar),
    ("reasons", "tried.day.era.era", Kind::Era),
];

impl PublishedReasons {
    /// The record as JSON, every catalogue member written as its full key:
    /// what the boundary carries and seals.
    ///
    /// # Errors
    ///
    /// `INTERNAL` if the record does not serialise, which one this crate
    /// built cannot do.
    pub fn in_full(&self) -> Result<Value, Error> {
        let mut value = serde_json::to_value(self).map_err(|error| {
            Error::internal(format!("the published reasons did not serialise: {error}"))
        })?;
        for (list, path, kind) in MEMBERS {
            if let Some(items) = value.get_mut(list) {
                write_in_full(items, path, kind);
            }
        }
        Ok(value)
    }
}

/// A [`AlmanacArea::published_reasons`] request as a binding sends it,
/// read: the range, the place, the festival request with its sets, and
/// the places the publisher may have reckoned at.
#[derive(Clone, Debug, PartialEq)]
pub struct PublishedReasonsRequest {
    /// The first day.
    pub from: CalendarDate,
    /// The last day, inclusive.
    pub to: CalendarDate,
    /// The place.
    pub place: Place,
    /// The offset its civil days run at.
    pub offset: UtcOffset,
    /// The festival request, with the published sets to explain.
    pub festivals: FestivalRequest,
    /// The places tried, in order.
    pub places: Vec<ReckonedPlace>,
}

/// A place as a binding writes one.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "ReckonedPlace"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct PlaceAsked {
    name: String,
    latitude_deg: f64,
    longitude_deg: f64,
    #[serde(default)]
    altitude_m: f64,
    utc_offset_seconds: i32,
}

/// The request as a binding writes it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "PublishedReasonsRequest"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ReasonsAsked {
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
    festivals: Value,
    #[serde(default)]
    places: Vec<PlaceAsked>,
}

impl PublishedReasonsRequest {
    /// The record a binding sends, as JSON: `first` (and `last`, the first
    /// when left out) as `{"year", "month", "day"}` in `calendar`
    /// (Gregorian when left out), `latitudeDeg`, `longitudeDeg`,
    /// `altitudeM` and `utcOffsetSeconds`; `festivals`, a festival request
    /// with its `published` sets; and `places`, each `{"name",
    /// "latitudeDeg", "longitudeDeg", "altitudeM", "utcOffsetSeconds"}`.
    ///
    /// ```
    /// use teistro::PublishedReasonsRequest;
    ///
    /// let asked = PublishedReasonsRequest::from_json(
    ///     r#"{"first": {"year": 2026, "month": 10, "day": 1},
    ///         "last": {"year": 2026, "month": 10, "day": 31},
    ///         "latitudeDeg": 28.2096, "longitudeDeg": 83.9856, "utcOffsetSeconds": 20700,
    ///         "festivals": {"rules": "NEPAL"},
    ///         "places": [{"name": "Kathmandu", "latitudeDeg": 27.7172, "longitudeDeg": 85.324,
    ///                     "altitudeM": 1400, "utcOffsetSeconds": 20700}]}"#,
    /// )?;
    /// assert_eq!(asked.places[0].name, "Kathmandu");
    /// let stray = PublishedReasonsRequest::from_json(
    ///     r#"{"first": {"year": 2026, "month": 10, "day": 1}, "latitudeDeg": 95,
    ///         "longitudeDeg": 0, "utcOffsetSeconds": 0, "festivals": {"rules": "NEPAL"}}"#,
    /// )
    /// .unwrap_err();
    /// assert_eq!(stray.field(), Some("publishedReasons.latitudeDeg"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, a place or offset out of range, or a festival request its
    /// own reader refuses, each named under `publishedReasons`.
    pub fn from_json(text: &str) -> Result<PublishedReasonsRequest, Error> {
        let asked: ReasonsAsked = teistro_core::strict::read(text, PUBLISHED_REASONS)?;
        asked.request().map_err(|why| why.under(PUBLISHED_REASONS))
    }
}

impl ReasonsAsked {
    fn request(self) -> Result<PublishedReasonsRequest, Error> {
        let calendar = self.calendar.unwrap_or(Calendar::Gregorian);
        let places = self
            .places
            .into_iter()
            .enumerate()
            .map(|(index, at)| {
                let read = || -> Result<ReckonedPlace, Error> {
                    Ok(ReckonedPlace {
                        place: place_of(at.latitude_deg, at.longitude_deg, at.altitude_m)?,
                        offset: offset_of(at.utc_offset_seconds)?,
                        name: at.name.clone(),
                    })
                };
                read().map_err(|why| why.under(&format!("places[{index}]")))
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(PublishedReasonsRequest {
            from: self.first.in_calendar(calendar),
            to: self.last.unwrap_or(self.first).in_calendar(calendar),
            place: place_of(self.latitude_deg, self.longitude_deg, self.altitude_m)?,
            offset: offset_of(self.utc_offset_seconds)?,
            festivals: FestivalRequest::from_json(&self.festivals.to_string())?,
            places,
        })
    }
}

impl AlmanacArea<'_> {
    /// As [`AlmanacArea::published_reasons`], for a request a binding
    /// sent: the record written in full and sealed as the boundary carries
    /// it.
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::published_reasons`].
    pub fn published_reasons_request(
        self,
        skies: &[&Context],
        asked: &PublishedReasonsRequest,
    ) -> Result<Envelope<Value>, Error> {
        let Envelope { value, provenance } = self.published_reasons(
            (&asked.from, &asked.to),
            (&asked.place, asked.offset),
            &asked.festivals,
            &asked.places,
            skies,
        )?;
        Ok(Envelope::sealing(value.in_full()?, provenance))
    }
}
