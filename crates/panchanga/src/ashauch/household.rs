//! A household's periods (SR-28, `03-design/ashauch.md` §11): several
//! events and several observers in one request, each observer's overlaps
//! (*saṃpāta*) resolved in the order the news reached that observer,
//! since the overlap is made by knowing, not by which came first
//! (*Dharmasindhu* p. 305).

use serde::{Deserialize, Serialize};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};

use super::{Ashauch, AshauchEvent, AshauchRequest, Nearness, Relation};

/// The most ties one request names, so a request is bounded.
pub const MOST_TIES: usize = 64;

/// One event of a household, shared by the observers its ties name.
///
/// ```
/// use teistro_core::quantity::JulianDay;
/// use teistro_panchanga::ashauch::{AshauchEvent, Deceased, HouseholdEvent};
///
/// let event = HouseholdEvent {
///     key: String::from("grandfather"),
///     event: AshauchEvent::Death { at: JulianDay::literal(2_461_051.5), cremated_at: None, ahitagni: false, deceased: Deceased::default() },
/// };
/// assert_eq!(event.key, "grandfather");
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HouseholdEvent {
    /// The caller's name for it, which ties and answers repeat.
    pub key: String,
    /// The event.
    pub event: AshauchEvent,
}

/// How one observer stands to one event.
///
/// ```
/// use teistro_panchanga::ashauch::{HouseholdTie, Nearness, Relation};
///
/// let tie = HouseholdTie {
///     event: String::from("grandfather"),
///     relation: Relation::Child,
///     heard_at: None,
///     abroad: false,
///     nearness: Nearness::SameVillage,
///     full_days: None,
///     fire_giver: true,
/// };
/// assert!(tie.fire_giver);
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HouseholdTie {
    /// The event's key.
    pub event: String,
    /// The observer's relation to it.
    pub relation: Relation,
    /// When the news reached this observer; at the event when left out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heard_at: Option<JulianDay<Utc>>,
    /// Whether it happened abroad, to this observer.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub abroad: bool,
    /// Where it was, to this observer.
    #[serde(default)]
    pub nearness: Nearness,
    /// The full period, ten days when left out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_days: Option<u8>,
    /// Whether this observer lights the pyre.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub fire_giver: bool,
}

/// One observer of a household, and the events that reach them.
///
/// ```
/// use teistro_panchanga::ashauch::HouseholdObserver;
///
/// let observer = HouseholdObserver { key: String::from("son"), ties: Vec::new() };
/// assert!(observer.ties.is_empty());
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HouseholdObserver {
    /// The caller's name for the observer.
    pub key: String,
    /// The events, in any order.
    pub ties: Vec<HouseholdTie>,
}

/// Several events and observers in one request.
///
/// ```
/// use teistro_panchanga::ashauch::Household;
///
/// let household = Household { events: Vec::new(), observers: Vec::new() };
/// assert!(household.check().is_err());
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Household {
    /// The events.
    pub events: Vec<HouseholdEvent>,
    /// The observers.
    pub observers: Vec<HouseholdObserver>,
}

/// One observer's periods, in the order the news reached them.
///
/// ```
/// use teistro_core::quantity::JulianDay;
/// use teistro_panchanga::ashauch::HouseholdPeriods;
///
/// let periods = HouseholdPeriods { key: String::from("son"), periods: Vec::new(), pure_at: JulianDay::literal(2_461_062.5) };
/// assert!(periods.periods.is_empty());
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HouseholdPeriods {
    /// The observer's key.
    pub key: String,
    /// Each tie's period, its `key` the event's.
    pub periods: Vec<Ashauch>,
    /// The instant the last of them ends.
    pub pure_at: JulianDay<Utc>,
}

/// The answer, an entry for each observer in the order asked.
///
/// ```
/// use teistro_panchanga::ashauch::HouseholdAnswer;
///
/// let answer = HouseholdAnswer { observers: Vec::new() };
/// assert!(answer.in_full().is_ok());
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HouseholdAnswer {
    /// The observers.
    pub observers: Vec<HouseholdPeriods>,
}

impl HouseholdAnswer {
    /// As JSON, every catalogue member written as its full key.
    ///
    /// ```
    /// use teistro_panchanga::ashauch::HouseholdAnswer;
    ///
    /// let answer = HouseholdAnswer { observers: Vec::new() };
    /// assert_eq!(answer.in_full()?["observers"], serde_json::json!([]));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INTERNAL` if the answer does not serialise.
    pub fn in_full(&self) -> Result<serde_json::Value, Error> {
        let observers = self
            .observers
            .iter()
            .map(|observer| {
                let periods = observer
                    .periods
                    .iter()
                    .map(Ashauch::in_full)
                    .collect::<Result<Vec<_>, Error>>()?;
                let pure_at = serde_json::to_value(observer.pure_at).map_err(|error| {
                    Error::internal(format!("the instant did not serialise: {error}"))
                })?;
                Ok(serde_json::json!({
                    "key": observer.key,
                    "periods": periods,
                    "pureAt": pure_at,
                }))
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(serde_json::json!({ "observers": observers }))
    }
}

impl Household {
    /// Whether every tie names one event, once for its observer, and the
    /// request is bounded.
    ///
    /// ```
    /// use teistro_panchanga::ashauch::{Household, HouseholdObserver, HouseholdTie, Nearness, Relation};
    ///
    /// let tie = HouseholdTie {
    ///     event: String::from("nobody"),
    ///     relation: Relation::Child,
    ///     heard_at: None,
    ///     abroad: false,
    ///     nearness: Nearness::SameVillage,
    ///     full_days: None,
    ///     fire_giver: false,
    /// };
    /// let household = Household {
    ///     events: Vec::new(),
    ///     observers: vec![HouseholdObserver { key: String::from("son"), ties: vec![tie] }],
    /// };
    /// assert_eq!(household.check().unwrap_err().field(), Some("observers[0].ties[0].event"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` when no observer is named, a key repeats, an observer
    /// has no tie, a tie names an unknown event or repeats one, or the
    /// request names more than [`MOST_TIES`] ties.
    pub fn check(&self) -> Result<(), Error> {
        if self.observers.is_empty() {
            return Err(Error::invalid_arg("a household names an observer").with_field("observers"));
        }
        for (at, event) in self.events.iter().enumerate() {
            if self
                .events
                .iter()
                .take(at)
                .any(|seen| seen.key == event.key)
            {
                return Err(
                    Error::invalid_arg(format!("the event `{}` repeats", event.key))
                        .with_field(format!("events[{at}].key")),
                );
            }
        }
        let ties: usize = self.observers.iter().map(|one| one.ties.len()).sum();
        if ties > MOST_TIES {
            return Err(Error::invalid_arg(format!(
                "a household names at most {MOST_TIES} ties, not {ties}"
            ))
            .with_field("observers"));
        }
        for (at, observer) in self.observers.iter().enumerate() {
            let here = format!("observers[{at}]");
            if self
                .observers
                .iter()
                .take(at)
                .any(|seen| seen.key == observer.key)
            {
                return Err(
                    Error::invalid_arg(format!("the observer `{}` repeats", observer.key))
                        .with_field(format!("{here}.key")),
                );
            }
            if observer.ties.is_empty() {
                return Err(Error::invalid_arg("an observer names a tie")
                    .with_field(format!("{here}.ties")));
            }
            for (row, tie) in observer.ties.iter().enumerate() {
                let field = format!("{here}.ties[{row}].event");
                if !self.events.iter().any(|event| event.key == tie.event) {
                    return Err(Error::invalid_arg(format!(
                        "no event has the key `{}`",
                        tie.event
                    ))
                    .with_field(field));
                }
                if observer
                    .ties
                    .iter()
                    .take(row)
                    .any(|seen| seen.event == tie.event)
                {
                    return Err(Error::invalid_arg(format!(
                        "the event `{}` is tied twice",
                        tie.event
                    ))
                    .with_field(field));
                }
            }
        }
        Ok(())
    }

    /// One observer's requests, in the order the news reached them and
    /// with nothing yet running: [`crate::Almanac::household`] fills
    /// `during` from each answer in turn. Two pieces of news at one
    /// instant keep the order asked.
    ///
    /// ```
    /// use teistro_panchanga::ashauch::{AshauchEvent, Deceased, Household, HouseholdEvent, HouseholdObserver, HouseholdTie, Nearness, Relation};
    /// use teistro_core::quantity::JulianDay;
    ///
    /// let death = |key: &str, at: f64| HouseholdEvent {
    ///     key: key.to_owned(),
    ///     event: AshauchEvent::Death { at: JulianDay::literal(at), cremated_at: None, ahitagni: false, deceased: Deceased::default() },
    /// };
    /// let tie = |event: &str| HouseholdTie {
    ///     event: event.to_owned(),
    ///     relation: Relation::Sapinda,
    ///     heard_at: None,
    ///     abroad: false,
    ///     nearness: Nearness::SameVillage,
    ///     full_days: None,
    ///     fire_giver: false,
    /// };
    /// let household = Household {
    ///     events: vec![death("later", 20.0), death("earlier", 10.0)],
    ///     observers: vec![HouseholdObserver { key: String::from("son"), ties: vec![tie("later"), tie("earlier")] }],
    /// };
    /// let asked = household.requests(&household.observers[0]);
    /// assert_eq!(asked[0].key, "earlier");
    /// ```
    #[must_use]
    pub fn requests(&self, observer: &HouseholdObserver) -> Vec<AshauchRequest> {
        let mut requests: Vec<AshauchRequest> = observer
            .ties
            .iter()
            .filter_map(|tie| {
                let found = self.events.iter().find(|event| event.key == tie.event)?;
                Some(AshauchRequest {
                    key: found.key.clone(),
                    event: found.event,
                    relation: tie.relation,
                    heard_at: tie.heard_at,
                    abroad: tie.abroad,
                    nearness: tie.nearness,
                    full_days: tie.full_days,
                    during: Vec::new(),
                    fire_giver: tie.fire_giver,
                })
            })
            .collect();
        requests.sort_by(|one, other| one.known().get().total_cmp(&other.known().get()));
        requests
    }
}
