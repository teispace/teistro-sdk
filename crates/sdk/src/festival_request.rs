//! What a festival reckoning is asked (`03-design/festival-rules.md` §7):
//! the rules, a pack the SDK ships amended rule by rule.
//!
//! The range, the place and the clock are the almanac's, which
//! [`AlmanacArea::festivals`](crate::AlmanacArea::festivals) takes beside
//! this; the record a binding writes as `festivals` carries none of them,
//! so the days and their observances cannot be asked of two places.

use serde::Serialize;
use serde_json::Value;
use teistro_core::error::Error;
use teistro_panchanga::festival::FestivalRule;

/// A pack of rules the SDK ships, which a request may name rather than
/// spell out.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FestivalPack {
    /// *Dharmasindhu*'s rules ([`FestivalRule::dharmasindhu`]).
    Dharmasindhu,
}

impl FestivalPack {
    /// Every pack the SDK ships.
    pub const ALL: [FestivalPack; 1] = [FestivalPack::Dharmasindhu];

    /// Its key, as a request names it.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            FestivalPack::Dharmasindhu => "DHARMASINDHU",
        }
    }

    /// Its rules.
    #[must_use]
    pub fn rules(self) -> Vec<FestivalRule> {
        match self {
            FestivalPack::Dharmasindhu => FestivalRule::dharmasindhu(),
        }
    }
}

/// What a festival reckoning is asked: rules, each key once.
///
/// ```
/// use teistro::{FestivalPack, FestivalRequest};
/// use teistro::festival::{Choice, FestivalRule};
///
/// // The shipped pack, with Lakshmi puja always on the earlier day.
/// let mut earlier = FestivalRule::dharmasindhu().remove(3);
/// earlier.decide.clear();
/// earlier.otherwise = Choice::Earlier;
/// let asked = FestivalRequest::from(FestivalPack::Dharmasindhu).with_rule(earlier.clone());
/// assert_eq!(asked.rules().len(), 4);
/// assert_eq!(asked.rules()[3], earlier);
///
/// // The same, as a binding writes it.
/// let text = format!(r#"{{"rules": ["DHARMASINDHU", {}]}}"#, serde_json::to_string(&earlier)?);
/// assert_eq!(FestivalRequest::from_json(&text)?, asked);
///
/// let wrong = FestivalRequest::from_json(r#"{"rules": "DHARMASINDU"}"#).unwrap_err();
/// assert_eq!(wrong.field(), Some("festivals.rules"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FestivalRequest {
    rules: Vec<FestivalRule>,
}

impl From<FestivalPack> for FestivalRequest {
    fn from(pack: FestivalPack) -> FestivalRequest {
        FestivalRequest::new(pack.rules())
    }
}

impl FestivalRequest {
    /// A request for these rules, as given; [`FestivalRequest::check`]
    /// refuses a key given twice.
    #[must_use]
    pub fn new(rules: Vec<FestivalRule>) -> FestivalRequest {
        FestivalRequest { rules }
    }

    /// The same request with a rule added, or put in place of the one
    /// with its key.
    #[must_use]
    pub fn with_rule(mut self, rule: FestivalRule) -> FestivalRequest {
        match self.rules.iter_mut().find(|held| held.key == rule.key) {
            Some(held) => *held = rule,
            None => self.rules.push(rule),
        }
        self
    }

    /// The rules, in order.
    #[must_use]
    pub fn rules(&self) -> &[FestivalRule] {
        &self.rules
    }

    /// Refuses what no reckoning could answer, naming the field under
    /// `rules`: a rule [`FestivalRule::check`] refuses, or a key given
    /// twice, which would answer two observances a reader cannot tell
    /// apart.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on `rules[i]` and the rule's own field.
    pub fn check(&self) -> Result<(), Error> {
        for (index, rule) in self.rules.iter().enumerate() {
            let at = format!("rules[{index}]");
            rule.check().map_err(|why| why.under(&at))?;
            if self
                .rules
                .iter()
                .take(index)
                .any(|held| held.key == rule.key)
            {
                return Err(Error::invalid_arg(format!(
                    "the key `{}` is given twice; an observance is named by its rule's key, so a pack holds each once",
                    rule.key
                ))
                .with_field(format!("{at}.key")));
            }
        }
        Ok(())
    }

    /// The request a binding writes as `festivals`, read and checked.
    ///
    /// `rules` names a [`FestivalPack`] (`"DHARMASINDHU"`), or is a list
    /// whose items each name a pack or spell a [`FestivalRule`] out, in
    /// order; a later rule replaces an earlier one with its key. A
    /// catalogue member anywhere in it may be written bare or in full
    /// (`masa.ASHWINA`), as every binding reads one back.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming the key under `festivals` that is not JSON of
    /// its kind, is not a field, or fails [`FestivalRequest::check`].
    pub fn from_json(text: &str) -> Result<FestivalRequest, Error> {
        let asked: Asked = teistro_core::strict::read(text, FESTIVALS)?;
        let items = match &asked.rules {
            Value::Array(items) => items.iter().enumerate().collect(),
            one => vec![(0, one)],
        };
        let listed = asked.rules.is_array();
        let mut request = FestivalRequest::new(Vec::new());
        for (index, item) in items {
            let field = if listed {
                format!("{FESTIVALS}.rules[{index}]")
            } else {
                format!("{FESTIVALS}.rules")
            };
            match item {
                Value::String(name) => {
                    for rule in pack(name, &field)?.rules() {
                        request = request.with_rule(rule);
                    }
                }
                other if listed => {
                    let rule: FestivalRule = teistro_core::strict::read_value(other, &field)?;
                    rule.check().map_err(|why| why.under(&field))?;
                    request = request.with_rule(rule);
                }
                _ => {
                    return Err(Error::invalid_arg(
                        "`rules` is a pack's name or a list of names and rules",
                    )
                    .with_field(field));
                }
            }
        }
        request.check().map_err(|why| why.under(FESTIVALS))?;
        Ok(request)
    }
}

/// The record every binding writes the request as.
const FESTIVALS: &str = "festivals";

/// The pack a name names, or the refusal listing them.
fn pack(name: &str, field: &str) -> Result<FestivalPack, Error> {
    FestivalPack::ALL
        .into_iter()
        .find(|pack| pack.key() == name)
        .ok_or_else(|| {
            let known: Vec<&str> = FestivalPack::ALL.iter().map(|pack| pack.key()).collect();
            Error::invalid_arg(format!(
                "`{name}` is not a pack the SDK ships; it ships {}, or a rule may be spelt out",
                known.join(", ")
            ))
            .with_field(field)
        })
}

/// [`FestivalRequest`] as the bindings write it. `rules` is read in a
/// second step, because each item is a name or a rule and the refusal
/// should say which item was wrong rather than that no shape matched.
#[derive(Serialize, serde::Deserialize)]
struct Asked {
    rules: Value,
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "a test fails by panicking and indexes what it built"
    )]

    use super::*;
    use teistro_panchanga::festival::Choice;

    fn refused(text: &str) -> Option<String> {
        FestivalRequest::from_json(text)
            .unwrap_err()
            .field()
            .map(str::to_owned)
    }

    #[test]
    fn a_pack_by_name_alone_or_in_a_list_is_the_pack() {
        let pack = FestivalRequest::from(FestivalPack::Dharmasindhu);
        for text in [
            r#"{"rules": "DHARMASINDHU"}"#,
            r#"{"rules": ["DHARMASINDHU"]}"#,
            r#"{"rules": ["DHARMASINDHU", "DHARMASINDHU"]}"#,
        ] {
            assert_eq!(FestivalRequest::from_json(text).unwrap(), pack, "{text}");
        }
    }

    #[test]
    fn a_later_rule_replaces_the_one_with_its_key_in_place() {
        let mut moved = FestivalRule::dharmasindhu().remove(0);
        moved.otherwise = Choice::Earlier;
        let rule = serde_json::to_string(&moved).unwrap();
        let asked =
            FestivalRequest::from_json(&format!(r#"{{"rules": ["DHARMASINDHU", {rule}]}}"#))
                .unwrap();
        assert_eq!(asked.rules()[0], moved);
        assert_eq!(asked.rules().len(), 4);
        // And a pack after the rule puts the shipped one back.
        let asked =
            FestivalRequest::from_json(&format!(r#"{{"rules": [{rule}, "DHARMASINDHU"]}}"#))
                .unwrap();
        assert_eq!(asked, FestivalRequest::from(FestivalPack::Dharmasindhu));
    }

    #[test]
    fn a_catalogue_member_reads_in_full() {
        let rule = serde_json::to_string(&FestivalRule::dharmasindhu()[0])
            .unwrap()
            .replace(r#""CHAITRA""#, r#""masa.CHAITRA""#)
            .replace(r#""SHUKLA_NAVAMI""#, r#""tithi.SHUKLA_NAVAMI""#);
        assert!(rule.contains("masa.CHAITRA"), "{rule}");
        let asked = FestivalRequest::from_json(&format!(r#"{{"rules": [{rule}]}}"#)).unwrap();
        assert_eq!(asked.rules()[0], FestivalRule::dharmasindhu()[0]);
    }

    #[test]
    fn a_refusal_names_the_item_and_its_field() {
        assert_eq!(
            refused(r#"{"rules": "DHARMA"}"#).as_deref(),
            Some("festivals.rules")
        );
        assert_eq!(
            refused(r#"{"rules": ["DHARMASINDHU", "DHARMA"]}"#).as_deref(),
            Some("festivals.rules[1]")
        );
        assert_eq!(
            refused(r#"{"rules": 3}"#).as_deref(),
            Some("festivals.rules")
        );
        assert_eq!(
            refused(r#"{"rule": "DHARMASINDHU"}"#).as_deref(),
            Some("festivals")
        );
        let mut keyless = FestivalRule::dharmasindhu()[0].clone();
        keyless.key.clear();
        let rule = serde_json::to_string(&keyless).unwrap();
        assert_eq!(
            refused(&format!(r#"{{"rules": ["DHARMASINDHU", {rule}]}}"#)).as_deref(),
            Some("festivals.rules[1].key")
        );
        let unknown = serde_json::to_string(&FestivalRule::dharmasindhu()[0])
            .unwrap()
            .replacen('{', r#"{"colour": "red", "#, 1);
        assert_eq!(
            refused(&format!(r#"{{"rules": [{unknown}]}}"#)).as_deref(),
            Some("festivals.rules[0].colour")
        );
    }

    #[test]
    fn a_key_given_twice_is_refused_by_the_second() {
        let rules = FestivalRule::dharmasindhu();
        let twice =
            FestivalRequest::new(vec![rules[1].clone(), rules[0].clone(), rules[1].clone()]);
        assert_eq!(twice.check().unwrap_err().field(), Some("rules[2].key"));
    }
}
