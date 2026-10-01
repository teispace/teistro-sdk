//! Observances counted from another's day (`03-design/festival-rules.md`
//! §9.4).
//!
//! Some observances fall a number of civil days after another, whatever
//! tithi runs then: the ash of the Holika fire is honoured "the morning
//! after, on the pratipada" (*Dharmasindhu* p. 95), and Nepal's committee
//! prints the Terai's Holi on the day after the hills' (VS 2082 and 2083),
//! though the full moon still holds that day's sunrise. A rule of the tithi
//! would miss it by a day; this one counts days.

use serde::{Deserialize, Serialize};
use teistro_core::error::Error;

use super::{Decided, FestivalDay, Observance, Observances, Unjudged};

/// An observance on the day a number of civil days after another rule's.
///
/// ```
/// use teistro_panchanga::festival::FollowingRule;
///
/// let terai = FollowingRule::nepal().into_iter().find(|rule| rule.key == "HOLI_TERAI").ok_or("shipped")?;
/// assert_eq!((terai.after.as_str(), terai.days), ("HOLIKA", 1));
/// # Ok::<(), &str>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct FollowingRule {
    /// The rule's key in its pack, `"HOLI_TERAI"`.
    pub key: String,
    /// Where the rule is stated.
    pub source: String,
    /// The key of the karmakala rule whose day it counts from.
    pub after: String,
    /// How many civil days after that day, 0 for the same day, at most
    /// [`FollowingRule::MOST_DAYS`].
    pub days: u8,
}

impl FollowingRule {
    /// The most days a rule may count: a fortnight, past which the next
    /// fortnight's tithis would name the day better than a count.
    pub const MOST_DAYS: u8 = 15;

    /// Refuses a rule that could not be judged: an empty key or `after`,
    /// a rule following itself, or a count past [`FollowingRule::MOST_DAYS`].
    ///
    /// # Errors
    ///
    /// `INVALID_ARG`, naming the field that is wrong.
    pub fn check(&self) -> Result<(), Error> {
        if self.key.trim().is_empty() {
            return Err(Error::invalid_arg("a festival rule needs a key").with_field("key"));
        }
        if self.after.trim().is_empty() || self.after == self.key {
            return Err(Error::invalid_arg(format!(
                "{}: `after` names the karmakala rule whose day this one counts from, not `{}`",
                self.key, self.after
            ))
            .with_field("after"));
        }
        if self.days > FollowingRule::MOST_DAYS {
            return Err(Error::invalid_arg(format!(
                "{}: a rule counts at most {} days from another's, not {}; past a fortnight, name the day by its tithi",
                self.key,
                FollowingRule::MOST_DAYS,
                self.days
            ))
            .with_field("days"));
        }
        Ok(())
    }

    /// The days Nepal's national panchanga prints as counted from another:
    /// Holi in the hill districts on the Holika day, and in the Terai on the
    /// day after (VS 2082 p. 23, VS 2083 p. 25).
    #[must_use]
    pub fn nepal() -> Vec<FollowingRule> {
        vec![
            FollowingRule {
                key: "HOLI_HILLS".to_owned(),
                source: "Nepal's national panchanga, VS 2082 p. 23 and VS 2083 p. 25: \"पहाडी जिल्लामा होली\" on the day of the Holika fire".to_owned(),
                after: "HOLIKA".to_owned(),
                days: 0,
            },
            FollowingRule {
                key: "HOLI_TERAI".to_owned(),
                source: "Nepal's national panchanga, VS 2082 p. 23 and VS 2083 p. 25: \"तराईमा होली\" the day after the fire; Dharmasindhu p. 95: the ash is honoured the morning after".to_owned(),
                after: "HOLIKA".to_owned(),
                days: 1,
            },
        ]
    }
}

/// The observances each following rule gives, counted from the
/// observances `found` over the same `days`.
///
/// Each carries the facts of the observance it counts from, its tithi,
/// case and extents, with `decided_by` naming that rule and the count. An
/// observance whose counted day the days do not reach is unjudged.
///
/// # Errors
///
/// A rule's own refusal ([`FollowingRule::check`]).
pub fn following(
    rules: &[FollowingRule],
    found: &[Observance],
    days: &[FestivalDay],
) -> Result<Observances, Error> {
    for rule in rules {
        rule.check()?;
    }
    let mut answer = Observances::default();
    for observance in found {
        for rule in rules.iter().filter(|rule| rule.after == observance.rule) {
            let counted = days
                .iter()
                .position(|day| day.date == observance.day)
                .and_then(|at| days.get(at + usize::from(rule.days)));
            match counted {
                Some(day) => answer.observances.push(Observance {
                    rule: rule.key.clone(),
                    day: day.date.clone(),
                    decided_by: Decided::After {
                        rule: observance.rule.clone(),
                        days: rule.days,
                    },
                    ..observance.clone()
                }),
                None => answer.unjudged.push(Unjudged {
                    rule: rule.key.clone(),
                    tithi: observance.tithi,
                    why: format!(
                        "the days do not reach {} days after {}'s day",
                        rule.days, observance.rule
                    ),
                }),
            }
        }
    }
    Ok(answer)
}
