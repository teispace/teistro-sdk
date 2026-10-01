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
use teistro_panchanga::festival::{EkadashiRule, FestivalRule, FollowingRule};

/// A pack of rules the SDK ships, which a request may name rather than
/// spell out.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FestivalPack {
    /// *Dharmasindhu*'s rules ([`FestivalRule::dharmasindhu`]) and its
    /// three Ekadashi observers ([`EkadashiRule::dharmasindhu`]).
    Dharmasindhu,
    /// *Dharmasindhu*'s rules as Nepal's national panchanga keeps them
    /// ([`FestivalRule::nepal`]: a rite of the daylight on the day whose
    /// sunrise holds its tithi, C197), with the days it counts from
    /// another's ([`FollowingRule::nepal`]): Holi in the hills and in the
    /// Terai.
    Nepal,
}

impl FestivalPack {
    /// Every pack the SDK ships.
    pub const ALL: [FestivalPack; 2] = [FestivalPack::Dharmasindhu, FestivalPack::Nepal];

    /// Its key, as a request names it.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            FestivalPack::Dharmasindhu => "DHARMASINDHU",
            FestivalPack::Nepal => "NEPAL",
        }
    }

    /// Its karmakala rules.
    #[must_use]
    pub fn rules(self) -> Vec<FestivalRule> {
        match self {
            FestivalPack::Dharmasindhu => FestivalRule::dharmasindhu(),
            FestivalPack::Nepal => FestivalRule::nepal(),
        }
    }

    /// Its Ekadashi rules.
    #[must_use]
    pub fn ekadashis(self) -> Vec<EkadashiRule> {
        match self {
            FestivalPack::Dharmasindhu | FestivalPack::Nepal => EkadashiRule::dharmasindhu(),
        }
    }

    /// Its rules counted from another's day.
    #[must_use]
    pub fn following(self) -> Vec<FollowingRule> {
        match self {
            FestivalPack::Dharmasindhu => Vec::new(),
            FestivalPack::Nepal => FollowingRule::nepal(),
        }
    }
}

/// What a festival reckoning is asked: karmakala rules, Ekadashi rules
/// and rules counted from another's day, each key once across all three,
/// because an answer is named by it.
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
/// assert_eq!(asked.rules().len(), FestivalRule::dharmasindhu().len());
/// assert_eq!(asked.rules()[3], earlier);
/// assert_eq!(asked.ekadashis().len(), 3);
///
/// // The same, as a binding writes it.
/// let text = format!(r#"{{"rules": ["DHARMASINDHU", {}]}}"#, serde_json::to_string(&earlier)?);
/// assert_eq!(FestivalRequest::from_json(&text)?, asked);
///
/// // Nepal's pack counts the Terai's Holi a day from the Holika fire.
/// let nepal = FestivalRequest::from(FestivalPack::Nepal);
/// assert_eq!(nepal.following()[1].key, "HOLI_TERAI");
/// assert_eq!(nepal.reach(), 1);
///
/// let wrong = FestivalRequest::from_json(r#"{"rules": "DHARMASINDU"}"#).unwrap_err();
/// assert_eq!(wrong.field(), Some("festivals.rules"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FestivalRequest {
    rules: Vec<FestivalRule>,
    ekadashis: Vec<EkadashiRule>,
    following: Vec<FollowingRule>,
}

impl From<FestivalPack> for FestivalRequest {
    fn from(pack: FestivalPack) -> FestivalRequest {
        FestivalRequest::new(Vec::new()).with_pack(pack)
    }
}

/// A rule of any of a request's three kinds, by its key.
trait Keyed {
    fn key(&self) -> &str;
}

impl Keyed for FestivalRule {
    fn key(&self) -> &str {
        &self.key
    }
}

impl Keyed for EkadashiRule {
    fn key(&self) -> &str {
        &self.key
    }
}

impl Keyed for FollowingRule {
    fn key(&self) -> &str {
        &self.key
    }
}

/// Puts `rule` in place of the one in `list` with its key, or hands it
/// back when none has it.
fn replaced<T: Keyed>(list: &mut [T], rule: T) -> Result<(), T> {
    match list.iter_mut().find(|held| held.key() == rule.key()) {
        Some(held) => {
            *held = rule;
            Ok(())
        }
        None => Err(rule),
    }
}

impl FestivalRequest {
    /// A request for these rules, as given; [`FestivalRequest::check`]
    /// refuses a key given twice.
    #[must_use]
    pub fn new(rules: Vec<FestivalRule>) -> FestivalRequest {
        FestivalRequest {
            rules,
            ekadashis: Vec::new(),
            following: Vec::new(),
        }
    }

    /// Drops the rule with this key, of whichever kind.
    fn forget(&mut self, key: &str) {
        self.rules.retain(|held| held.key != key);
        self.ekadashis.retain(|held| held.key != key);
        self.following.retain(|held| held.key != key);
    }

    /// The same request with a rule added, or put in place of the one
    /// with its key; a rule of another kind with that key goes.
    #[must_use]
    pub fn with_rule(mut self, rule: FestivalRule) -> FestivalRequest {
        if let Err(rule) = replaced(&mut self.rules, rule) {
            self.forget(&rule.key);
            self.rules.push(rule);
        }
        self
    }

    /// The same request with an Ekadashi rule added, or put in place of
    /// the one with its key; a rule of another kind with that key goes.
    #[must_use]
    pub fn with_ekadashi(mut self, rule: EkadashiRule) -> FestivalRequest {
        if let Err(rule) = replaced(&mut self.ekadashis, rule) {
            self.forget(&rule.key);
            self.ekadashis.push(rule);
        }
        self
    }

    /// The same request with a rule counted from another's day added, or
    /// put in place of the one with its key; a rule of another kind with
    /// that key goes.
    #[must_use]
    pub fn with_following(mut self, rule: FollowingRule) -> FestivalRequest {
        if let Err(rule) = replaced(&mut self.following, rule) {
            self.forget(&rule.key);
            self.following.push(rule);
        }
        self
    }

    /// The same request with every rule of a pack, each as its kind's
    /// `with_` puts it.
    #[must_use]
    pub fn with_pack(self, pack: FestivalPack) -> FestivalRequest {
        let request = pack
            .rules()
            .into_iter()
            .fold(self, FestivalRequest::with_rule)
            .with_ekadashis(pack.ekadashis());
        pack.following()
            .into_iter()
            .fold(request, FestivalRequest::with_following)
    }

    /// The same request with each of these Ekadashi rules, as
    /// [`FestivalRequest::with_ekadashi`].
    #[must_use]
    pub fn with_ekadashis(self, rules: impl IntoIterator<Item = EkadashiRule>) -> FestivalRequest {
        rules.into_iter().fold(self, FestivalRequest::with_ekadashi)
    }

    /// The karmakala rules, in order.
    #[must_use]
    pub fn rules(&self) -> &[FestivalRule] {
        &self.rules
    }

    /// The Ekadashi rules, in order.
    #[must_use]
    pub fn ekadashis(&self) -> &[EkadashiRule] {
        &self.ekadashis
    }

    /// The rules counted from another's day, in order.
    #[must_use]
    pub fn following(&self) -> &[FollowingRule] {
        &self.following
    }

    /// How many days the request reaches past an observance's own:
    /// the most any rule counts from another's.
    #[must_use]
    pub fn reach(&self) -> u8 {
        self.following
            .iter()
            .map(|rule| rule.days)
            .max()
            .unwrap_or(0)
    }

    /// Refuses what no reckoning could answer, naming the field under
    /// `rules`, `ekadashis` or `following`: a rule its own `check`
    /// refuses, a key given twice across all three, which would answer two
    /// observances a reader cannot tell apart, or a rule counted from one
    /// that is not a karmakala rule of the request.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on `rules[i]`, `ekadashis[i]` or `following[i]` and
    /// the rule's own field.
    pub fn check(&self) -> Result<(), Error> {
        let rules = self.rules.iter().map(|rule| (&rule.key, rule.check()));
        let ekadashis = self.ekadashis.iter().map(|rule| (&rule.key, rule.check()));
        let following = self.following.iter().map(|rule| (&rule.key, rule.check()));
        let mut seen = std::collections::BTreeSet::new();
        for (list, items) in [
            ("rules", rules.collect::<Vec<_>>()),
            ("ekadashis", ekadashis.collect()),
            ("following", following.collect()),
        ] {
            for (index, (key, checked)) in items.into_iter().enumerate() {
                let at = format!("{list}[{index}]");
                checked.map_err(|why| why.under(&at))?;
                if !seen.insert(key) {
                    return Err(Error::invalid_arg(format!(
                        "the key `{key}` is given twice; an observance is named by its rule's key, so a pack holds each once"
                    ))
                    .with_field(format!("{at}.key")));
                }
            }
        }
        for (index, rule) in self.following.iter().enumerate() {
            if !self.rules.iter().any(|held| held.key == rule.after) {
                return Err(Error::invalid_arg(format!(
                    "{}: `after` is `{}`, which no karmakala rule of the request has as its key; a rule counts from an observance the request finds",
                    rule.key, rule.after
                ))
                .with_field(format!("following[{index}].after")));
            }
        }
        Ok(())
    }

    /// The request a binding writes as `festivals`, read and checked.
    ///
    /// `rules` names a [`FestivalPack`] (`"DHARMASINDHU"`), or is a list
    /// whose items each name a pack or spell a rule out, in order: an
    /// [`EkadashiRule`] when it has a `vedha`, a [`FollowingRule`] when it
    /// has an `after`, else a [`FestivalRule`]. A
    /// later rule replaces an earlier one with its key. A
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
                    request = request.with_pack(pack(name, &field)?);
                }
                Value::Object(spelt) if listed && spelt.contains_key("vedha") => {
                    let rule: EkadashiRule = teistro_core::strict::read_value(item, &field)?;
                    rule.check().map_err(|why| why.under(&field))?;
                    request = request.with_ekadashi(rule);
                }
                Value::Object(spelt) if listed && spelt.contains_key("after") => {
                    let rule: FollowingRule = teistro_core::strict::read_value(item, &field)?;
                    rule.check().map_err(|why| why.under(&field))?;
                    request = request.with_following(rule);
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
        assert_eq!(asked.rules().len(), FestivalRule::dharmasindhu().len());
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
    fn an_ekadashi_rule_is_read_by_its_vedha_and_replaces_by_key() {
        let mut madhava = EkadashiRule::dharmasindhu().remove(1);
        madhava.table.pure.twelfth = teistro_panchanga::festival::Which::Earlier;
        let rule = serde_json::to_string(&madhava).unwrap();
        let asked =
            FestivalRequest::from_json(&format!(r#"{{"rules": ["DHARMASINDHU", {rule}]}}"#))
                .unwrap();
        assert_eq!(asked.ekadashis().len(), 3);
        assert_eq!(asked.ekadashis()[1], madhava);
        assert_eq!(asked.rules().len(), FestivalRule::dharmasindhu().len());
        // A field the Ekadashi rule does not have is refused by its path.
        let wrong = rule.replacen('{', r#"{"month": "KARTIKA", "#, 1);
        assert_eq!(
            refused(&format!(r#"{{"rules": [{wrong}]}}"#)).as_deref(),
            Some("festivals.rules[0].month")
        );
    }

    #[test]
    fn a_key_is_one_rule_s_across_both_kinds() {
        let mut ekadashi = EkadashiRule::dharmasindhu().remove(0);
        ekadashi.key = "JANMASHTAMI".to_owned();
        let asked = FestivalRequest::from(FestivalPack::Dharmasindhu).with_ekadashi(ekadashi);
        assert!(asked.rules().iter().all(|rule| rule.key != "JANMASHTAMI"));
        let twice = FestivalRequest::new(FestivalRule::dharmasindhu()).with_ekadashis([]);
        let mut clash = EkadashiRule::dharmasindhu().remove(0);
        clash.key = "RAMA_NAVAMI".to_owned();
        let twice = FestivalRequest {
            ekadashis: vec![clash],
            ..twice
        };
        assert_eq!(twice.check().unwrap_err().field(), Some("ekadashis[0].key"));
    }

    #[test]
    fn the_nepal_pack_is_dharmasindhu_s_with_its_counted_days() {
        let nepal = FestivalRequest::from_json(r#"{"rules": "NEPAL"}"#).unwrap();
        assert_eq!(nepal, FestivalRequest::from(FestivalPack::Nepal));
        assert_eq!(nepal.rules(), FestivalRule::nepal().as_slice());
        assert_ne!(nepal.rules(), FestivalRule::dharmasindhu().as_slice());
        assert_eq!(nepal.following(), FollowingRule::nepal().as_slice());
        assert_eq!(nepal.reach(), 1);
        let shipped = FestivalRequest::from(FestivalPack::Dharmasindhu);
        assert_eq!(shipped.following(), []);
        assert_eq!(shipped.reach(), 0);
    }

    #[test]
    fn a_following_rule_is_read_by_its_after_and_replaces_by_key() {
        let rule = r#"{"key": "HOLI_TERAI", "source": "mine", "after": "HOLIKA", "days": 2}"#;
        let asked =
            FestivalRequest::from_json(&format!(r#"{{"rules": ["NEPAL", {rule}]}}"#)).unwrap();
        assert_eq!(asked.following().len(), 2);
        assert_eq!(asked.following()[1].days, 2);
        assert_eq!(asked.reach(), 2);
        // A rule of another kind with its key goes.
        let mut hills = FestivalRule::dharmasindhu()
            .into_iter()
            .find(|rule| rule.key == "HOLIKA")
            .unwrap();
        hills.key = "HOLI_HILLS".to_owned();
        let asked = asked.with_rule(hills);
        assert!(
            asked
                .following()
                .iter()
                .all(|rule| rule.key != "HOLI_HILLS")
        );
    }

    #[test]
    fn a_following_rule_counts_from_a_karmakala_rule_of_the_request() {
        let rule = |after: &str, days: u8| {
            format!(
                r#"{{"rules": ["NEPAL", {{"key": "MINE", "source": "mine", "after": "{after}", "days": {days}}}]}}"#
            )
        };
        assert!(FestivalRequest::from_json(&rule("HOLIKA", 15)).is_ok());
        assert_eq!(
            refused(&rule("HOLIKA", 16)).as_deref(),
            Some("festivals.rules[1].days")
        );
        assert_eq!(
            refused(&rule("MINE", 1)).as_deref(),
            Some("festivals.rules[1].after")
        );
        // Not from an Ekadashi rule, another counted day, or no rule.
        for after in ["EKADASHI_SMARTA", "HOLI_TERAI", "NOBODY"] {
            assert_eq!(
                refused(&rule(after, 1)).as_deref(),
                Some("festivals.following[2].after"),
                "{after}"
            );
        }
    }

    #[test]
    fn a_key_given_twice_is_refused_by_the_second() {
        let rules = FestivalRule::dharmasindhu();
        let twice =
            FestivalRequest::new(vec![rules[1].clone(), rules[0].clone(), rules[1].clone()]);
        assert_eq!(twice.check().unwrap_err().field(), Some("rules[2].key"));
    }
}
