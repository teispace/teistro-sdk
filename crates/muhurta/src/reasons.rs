//! Why a window is good, in words (`03-design/muhurta-reasons.md`, SR-154).
//!
//! A window's clauses say what held over it. The reasons say it again as
//! intents a voice pack words (`sdk.voice.muhurta.*`):
//!
//! - a short **line**: the limbs that qualify it, and how many doshas it is
//!   clear of;
//! - a longer **list** in three runs. `FOR` is what favours the window.
//!   `DESPITE` is what holds against it without barring it. `AVOIDED` is
//!   each dosha the search looked for and did not find.
//!
//! The favourable half of a dosha is an absence, so `AVOIDED` names only a
//! dosha the search judges for every window ([`watched`]), and the rite's
//! own bars. It never names a check that was not made.
//!
//! The slots come from the clause itself, written in full keys by the one
//! table [`ClauseKey::members`] (`spelling.rs`): a member becomes an entity
//! slot, a list of members a list slot, and a nested record's fields are
//! lifted beside the rest. So no clause is respelled here, and a clause
//! added later is said as soon as its members are listed.

use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use teistro_core::catalogue::{Kind, write_in_full};
use teistro_core::error::{Error, Status};
use teistro_intl::{Params, Value};

use crate::activity::{ActivityRules, Bar};
use crate::clause::{Clause, ClauseKey, ClauseKind};
use crate::judge::Judgement;
use crate::search::Answer;

/// The namespace every intent is under.
pub const NS: &str = "sdk.voice.muhurta";

/// Which run of the list an item is in.
///
/// ```
/// use teistro_muhurta::reasons::Side;
///
/// assert_eq!(serde_json::to_string(&Side::Avoided)?, r#""AVOIDED""#);
/// assert_eq!(Side::Despite.segment(), "despite");
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "MuhurtaReasonSide"))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Side {
    /// A clause that favours the window.
    For,
    /// A clause against the time that held without barring it: a caution.
    Despite,
    /// A dosha the search looked for and did not find.
    Avoided,
}

impl Side {
    /// The intent's segment for the run: `favours`, `despite` or `avoided`.
    #[must_use]
    pub const fn segment(self) -> &'static str {
        match self {
            Side::For => "favours",
            Side::Despite => "despite",
            Side::Avoided => "avoided",
        }
    }
}

/// One reason: an intent, its slots, and what it says.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "MuhurtaReason"))]
#[serde(rename_all = "camelCase")]
pub struct Reason {
    /// The intent, a message key of the voice pack
    /// (`sdk.voice.muhurta.avoided.MARS_IN_EIGHTH`).
    pub intent: String,
    /// Its slots, the clause's members and a `variant`.
    #[cfg_attr(
        feature = "schema",
        schemars(with = "std::collections::BTreeMap<String, serde_json::Value>")
    )]
    pub params: Params,
    /// The run it is in.
    pub side: Side,
    /// The clause it says, as its key.
    pub key: ClauseKey,
    /// For `FOR` and `DESPITE`, the clause's index in the window's
    /// `clauses`; nothing for `AVOIDED`, which says an absence.
    #[serde(default)]
    pub clause: Option<usize>,
}

/// The "why this date" line: an intent and its slots, the qualifying
/// limbs and how much the list says.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "MuhurtaReasonLine"))]
#[serde(rename_all = "camelCase")]
pub struct Line {
    /// The intent, `sdk.voice.muhurta.line`.
    pub intent: String,
    /// Its slots: `limbs`, `count`, `avoided`, `despite` and `variant`.
    #[cfg_attr(
        feature = "schema",
        schemars(with = "std::collections::BTreeMap<String, serde_json::Value>")
    )]
    pub params: Params,
}

/// Why one window is good: a short line and the longer list.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "MuhurtaReasons"))]
#[serde(rename_all = "camelCase")]
pub struct WindowReasons {
    /// The "why this date" line.
    pub line: Line,
    /// The list: `FOR`, then `DESPITE`, then `AVOIDED`.
    pub reasons: Vec<Reason>,
}

/// The doshas the search judges for every window, in the order the list
/// says them, and whether each needs a native (`03-design/muhurta-reasons.md`
/// §1). A test plants each one and finds it reported.
const WATCHED: [(ClauseKey, bool); 13] = [
    (ClauseKey::Kaala, false),
    (ClauseKey::AshubhaYoga, false),
    (ClauseKey::Varjyam, false),
    (ClauseKey::Kartari, false),
    (ClauseKey::MoonInDusthana, false),
    (ClauseKey::MoonJoined, false),
    (ClauseKey::VenusInSixth, false),
    (ClauseKey::MarsInEighth, false),
    (ClauseKey::Kunavamsa, false),
    (ClauseKey::PanchakaRemainder, false),
    (ClauseKey::MaleficInLagna, false),
    (ClauseKey::AshtamaLagna, true),
    (ClauseKey::Ghata, true),
];

/// The doshas a search under `rules` judges for every window: the ones
/// every search judges, those that need a native when it has one, and the
/// rite's own when its rules ask for them.
///
/// ```
/// use teistro_muhurta::ActivityRules;
/// use teistro_muhurta::clause::ClauseKey;
/// use teistro_muhurta::reasons::watched;
///
/// let rules = ActivityRules::raman_marriage();
/// let without = watched(&rules, false);
/// assert!(without.contains(&ClauseKey::MarsInEighth));
/// assert!(!without.contains(&ClauseKey::Ghata), "a ghata needs a native");
/// assert!(watched(&rules, true).contains(&ClauseKey::Ghata));
/// ```
#[must_use]
pub fn watched(rules: &ActivityRules, native: bool) -> Vec<ClauseKey> {
    let own = [
        (ClauseKey::Pada, !rules.padas.is_empty()),
        (ClauseKey::UnwantedPlacement, !rules.unwanted.is_empty()),
        (ClauseKey::Pradosha, rules.pradosha),
    ];
    WATCHED
        .iter()
        .filter(|(_, needs)| native || !needs)
        .map(|(key, _)| *key)
        .chain(own.iter().filter(|(_, asked)| *asked).map(|(key, _)| *key))
        .collect()
}

/// Whether a clause of this kind can favour a window.
#[must_use]
pub const fn can_favour(key: ClauseKey) -> bool {
    matches!(
        key,
        ClauseKey::Tithi
            | ClauseKey::Nakshatra
            | ClauseKey::Yoga
            | ClauseKey::Karana
            | ClauseKey::Vara
            | ClauseKey::Month
            | ClauseKey::SolarMonth
            | ClauseKey::Lagna
            | ClauseKey::Choghadiya
            | ClauseKey::Abhijit
            | ClauseKey::MuhurtaYoga
            | ClauseKey::Tarabala
            | ClauseKey::Chandrabala
            | ClauseKey::BeneficInLagna
            | ClauseKey::ExaltedInLagna
            | ClauseKey::LuminaryInEleventh
            | ClauseKey::KendraBenefics
            | ClauseKey::VrishaVastu
    )
}

/// Whether a clause of this kind can hold against a window.
#[must_use]
pub const fn can_go_against(key: ClauseKey) -> bool {
    !matches!(
        key,
        ClauseKey::Abhijit
            | ClauseKey::BeneficInLagna
            | ClauseKey::ExaltedInLagna
            | ClauseKey::LuminaryInEleventh
            | ClauseKey::KendraBenefics
    )
}

/// Where a favourable clause stands in the `FOR` run: the limbs in the
/// order a panchanga prints them, then the rest.
const fn rank(key: ClauseKey) -> u8 {
    match key {
        ClauseKey::Nakshatra => 0,
        ClauseKey::Tithi => 1,
        ClauseKey::Vara => 2,
        ClauseKey::Yoga => 3,
        ClauseKey::Karana => 4,
        ClauseKey::Month | ClauseKey::SolarMonth => 5,
        ClauseKey::Lagna => 6,
        ClauseKey::MuhurtaYoga => 7,
        ClauseKey::Abhijit => 8,
        ClauseKey::Tarabala | ClauseKey::Chandrabala => 9,
        ClauseKey::Choghadiya => 10,
        _ => 11,
    }
}

/// The limbs the line names, in its order.
const LINE_LIMBS: [ClauseKey; 4] = [
    ClauseKey::Nakshatra,
    ClauseKey::Tithi,
    ClauseKey::Vara,
    ClauseKey::Yoga,
];

/// Every intent the composer can say: the line, then each side's keys.
/// A pack that words the voice words every one.
///
/// ```
/// use teistro_muhurta::reasons::intents;
///
/// let all = intents();
/// assert_eq!(all[0], "sdk.voice.muhurta.line");
/// assert!(all.contains(&"sdk.voice.muhurta.avoided.MARS_IN_EIGHTH".to_string()));
/// assert!(!all.contains(&"sdk.voice.muhurta.favours.MARS_IN_EIGHTH".to_string()));
/// ```
#[must_use]
pub fn intents() -> Vec<String> {
    let mut all = vec![format!("{NS}.line")];
    for side in [Side::For, Side::Despite, Side::Avoided] {
        for key in ClauseKey::ALL {
            let said = match side {
                Side::For => can_favour(key),
                Side::Despite | Side::Avoided => can_go_against(key),
            };
            if said {
                all.push(intent(side, key));
            }
        }
    }
    all
}

/// An intent's key.
fn intent(side: Side, key: ClauseKey) -> String {
    format!("{NS}.{}.{}", side.segment(), key_name(key))
}

/// A clause key as serde writes it (`MARS_IN_EIGHTH`).
fn key_name(key: ClauseKey) -> String {
    serde_json::to_value(key)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_default()
}

/// The reasons for every window of an answer, in its order.
///
/// # Errors
///
/// `INTERNAL` if a clause does not serialise, which one this crate built
/// cannot do.
pub fn of_answer(
    answer: &Answer,
    rules: &ActivityRules,
    native: bool,
) -> Result<Vec<WindowReasons>, Error> {
    let watched = watched(rules, native);
    answer
        .windows
        .iter()
        .enumerate()
        .map(|(index, window)| of_window(window, index, rules, &watched))
        .collect()
}

/// The reasons for one window, the `index`-th of its answer, which seeds
/// the wording's variants so neighbouring windows read differently.
///
/// # Errors
///
/// As [`of_answer`].
pub fn of_window(
    window: &Judgement,
    index: usize,
    rules: &ActivityRules,
    watched: &[ClauseKey],
) -> Result<WindowReasons, Error> {
    let mut favour: Vec<(usize, &Clause)> = Vec::new();
    let mut against: Vec<(usize, &Clause)> = Vec::new();
    for (at, clause) in window.clauses.iter().enumerate() {
        if clause.favourable() {
            favour.push((at, clause));
        } else {
            against.push((at, clause));
        }
    }
    favour.sort_by_key(|(at, clause)| (rank(clause.kind.key()), *at));

    let mut reasons = Vec::new();
    for (at, clause) in favour.iter().chain(&against) {
        let side = if clause.favourable() {
            Side::For
        } else {
            Side::Despite
        };
        reasons.push(said(side, &clause.kind, Some(*at))?);
    }
    let held = |key: ClauseKey| window.clauses.iter().any(|c| c.kind.key() == key);
    for key in watched {
        if !held(*key) {
            reasons.push(Reason {
                intent: intent(Side::Avoided, *key),
                params: Params::new(),
                side: Side::Avoided,
                key: *key,
                clause: None,
            });
        }
    }
    for bar in &rules.bars {
        let escaped = !window.clauses.iter().any(|c| bar.names(&c.kind));
        let listed = matches!(bar, Bar::Key(key) if watched.contains(key));
        if escaped && !listed {
            reasons.push(match bar {
                Bar::Key(key) => Reason {
                    intent: intent(Side::Avoided, *key),
                    params: Params::new(),
                    side: Side::Avoided,
                    key: *key,
                    clause: None,
                },
                Bar::Clause(kind) => said(Side::Avoided, kind, None)?,
            });
        }
    }
    for (place, reason) in reasons.iter_mut().enumerate() {
        reason
            .params
            .insert(String::from("variant"), variant(index, place));
    }

    let limbs: Vec<Value> = LINE_LIMBS
        .iter()
        .filter_map(|key| {
            favour
                .iter()
                .find(|(_, c)| c.kind.key() == *key)
                .and_then(|(_, c)| limb(&c.kind))
        })
        .collect();
    let count = |side: Side| {
        let n = reasons.iter().filter(|r| r.side == side).count();
        Value::Int(i64::try_from(n).unwrap_or(i64::MAX))
    };
    let mut line = Params::new();
    line.insert(
        String::from("count"),
        Value::Int(i64::try_from(limbs.len()).unwrap_or(i64::MAX)),
    );
    line.insert(String::from("limbs"), Value::List(limbs));
    line.insert(String::from("avoided"), count(Side::Avoided));
    line.insert(String::from("despite"), count(Side::Despite));
    line.insert(String::from("variant"), variant(index, 0));
    Ok(WindowReasons {
        line: Line {
            intent: format!("{NS}.line"),
            params: line,
        },
        reasons,
    })
}

/// The wording's variant for the `place`-th item of the `index`-th window.
fn variant(index: usize, place: usize) -> Value {
    Value::Int(i64::try_from(index.wrapping_add(place)).unwrap_or(0))
}

/// The member a line names for a limb clause.
fn limb(kind: &ClauseKind) -> Option<Value> {
    Some(match kind {
        ClauseKind::Nakshatra { nakshatra, .. } => Value::catalogued(*nakshatra),
        ClauseKind::Tithi { tithi, .. } => Value::catalogued(*tithi),
        ClauseKind::Vara { vara, .. } => Value::catalogued(*vara),
        ClauseKind::Yoga { yoga, .. } => Value::catalogued(*yoga),
        _ => return None,
    })
}

/// A clause said on one side, its members as slots.
fn said(side: Side, kind: &ClauseKind, clause: Option<usize>) -> Result<Reason, Error> {
    let key = kind.key();
    Ok(Reason {
        intent: intent(side, key),
        params: slots(kind)?,
        side,
        key,
        clause,
    })
}

/// A clause's fields as slots: each catalogue member an entity, a list of
/// them a list, a number an integer, any other text as itself, and a
/// nested record's fields lifted beside the rest (`pada.nakshatra` is
/// `nakshatra`).
fn slots(kind: &ClauseKind) -> Result<Params, Error> {
    let mut json = serde_json::to_value(kind).map_err(|error| {
        Error::new(
            Status::Internal,
            format!("a muhurta clause did not serialise: {error}"),
        )
    })?;
    let members = kind.key().members();
    for (path, member) in members {
        write_in_full(&mut json, path, *member);
    }
    let mut params = Params::new();
    if let Json::Object(fields) = json {
        for (name, value) in fields {
            if name == "clause" {
                continue;
            }
            lift(&mut params, &name, &name, value, members);
        }
    }
    Ok(params)
}

/// One field into the slots, by its path among the clause's members.
fn lift(params: &mut Params, path: &str, name: &str, value: Json, members: &[(&str, Kind)]) {
    let member = members.iter().any(|(p, _)| *p == path);
    let slot = match value {
        Json::Object(fields) => {
            for (inner, value) in fields {
                lift(params, &format!("{path}.{inner}"), &inner, value, members);
            }
            return;
        }
        Json::String(text) if member => Value::Entity(text),
        Json::String(text) => Value::Str(text),
        Json::Number(n) => match n.as_i64() {
            Some(i) => Value::Int(i),
            None => Value::Num(n.as_f64().unwrap_or_default()),
        },
        Json::Bool(b) => Value::Str(String::from(if b { "true" } else { "false" })),
        Json::Array(items) => Value::List(
            items
                .into_iter()
                .filter_map(|item| match item {
                    Json::String(text) if member => Some(Value::Entity(text)),
                    Json::String(text) => Some(Value::Str(text)),
                    _ => None,
                })
                .collect(),
        ),
        Json::Null => return,
    };
    params.insert(String::from(name), slot);
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
#[path = "reasons/tests.rs"]
mod tests;
