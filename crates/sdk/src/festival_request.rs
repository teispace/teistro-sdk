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
use teistro_panchanga::festival::{
    Anniversary, Dated, DatedRule, EkadashiRule, FestivalMeta, FestivalRule, FollowingRule,
    KindRules, MetaFilter, PublishedEntry, PublishedSet, SaitRule,
};

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
    /// [`FestivalPack::Nepal`] and the further days the national panchanga
    /// prints ([`FestivalRule::nepal_panchanga`],
    /// [`FollowingRule::nepal_panchanga`]): rules fitted to the committee's
    /// VS 2082 and 2083 editions where no text in hand gives a table, each
    /// citing its page (`festival-rules.md` §10, C600).
    NepalPanchanga,
    /// *Dharmasindhu*'s fortnight of the ancestors
    /// ([`FestivalRule::pitri_paksha`], `03-design/shraddha.md` §3.2): a
    /// mahalaya on each death tithi of Bhadrapada's dark half and the
    /// fortnight's own days, each at aparahna.
    PitriPaksha,
    /// The Gaudiya Vaishnavas' Ekadashi after *Hari Bhakti Vilasa*
    /// ([`EkadashiRule::hari_bhakti_vilasa`], `festival-rules.md` §8.9):
    /// the Vaishnava's table and the eight mahadvadashis that name the
    /// fast or move it onto the 12th's day.
    Gaudiya,
    /// The Government of India's central holidays for its offices in Delhi
    /// (`03-design/india-central-holidays.md`): the list's days by rule,
    /// Holi counted from the Holika fire, and the four days the notices
    /// set by the moon as printed, each marked subject to the sighting.
    IndiaCentral,
}

/// Puts each rule's sourced values on it, from `of` its key.
fn stamped<T: Stamped>(rules: Vec<T>, of: impl Fn(&str) -> FestivalMeta) -> Vec<T> {
    rules
        .into_iter()
        .map(|mut rule| {
            *rule.meta_mut() = of(rule.key());
            rule
        })
        .collect()
}

/// A *Dharmasindhu* pack's values, whatever the key.
fn by_text(_: &str) -> FestivalMeta {
    FestivalMeta::hindu_text()
}

impl FestivalPack {
    /// Every pack the SDK ships.
    pub const ALL: [FestivalPack; 6] = [
        FestivalPack::Dharmasindhu,
        FestivalPack::Nepal,
        FestivalPack::NepalPanchanga,
        FestivalPack::PitriPaksha,
        FestivalPack::Gaudiya,
        FestivalPack::IndiaCentral,
    ];

    /// Its key, as a request names it.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            FestivalPack::Dharmasindhu => "DHARMASINDHU",
            FestivalPack::Nepal => "NEPAL",
            FestivalPack::NepalPanchanga => "NEPAL_PANCHANGA",
            FestivalPack::PitriPaksha => "PITRI_PAKSHA",
            FestivalPack::Gaudiya => "GAUDIYA",
            FestivalPack::IndiaCentral => "INDIA_CENTRAL",
        }
    }

    /// Its karmakala rules.
    #[must_use]
    pub fn rules(self) -> Vec<FestivalRule> {
        match self {
            FestivalPack::Dharmasindhu => stamped(FestivalRule::dharmasindhu(), by_text),
            FestivalPack::Nepal => stamped(FestivalRule::nepal(), FestivalMeta::nepal),
            FestivalPack::NepalPanchanga => {
                let mut rules = FestivalRule::nepal();
                rules.extend(FestivalRule::nepal_panchanga());
                stamped(rules, FestivalMeta::nepal)
            }
            FestivalPack::PitriPaksha => stamped(FestivalRule::pitri_paksha(), by_text),
            FestivalPack::Gaudiya => Vec::new(),
            FestivalPack::IndiaCentral => FestivalRule::india_central(),
        }
    }

    /// Its Ekadashi rules.
    #[must_use]
    pub fn ekadashis(self) -> Vec<EkadashiRule> {
        match self {
            FestivalPack::Dharmasindhu => stamped(EkadashiRule::dharmasindhu(), by_text),
            FestivalPack::Nepal | FestivalPack::NepalPanchanga => {
                stamped(EkadashiRule::dharmasindhu(), FestivalMeta::nepal)
            }
            FestivalPack::PitriPaksha | FestivalPack::IndiaCentral => Vec::new(),
            FestivalPack::Gaudiya => stamped(vec![EkadashiRule::hari_bhakti_vilasa()], by_text),
        }
    }

    /// Its rules kept on a date of a calendar.
    #[must_use]
    pub fn dated(self) -> Vec<DatedRule> {
        match self {
            FestivalPack::Dharmasindhu
            | FestivalPack::Nepal
            | FestivalPack::PitriPaksha
            | FestivalPack::Gaudiya => Vec::new(),
            FestivalPack::NepalPanchanga => {
                stamped(DatedRule::nepal_panchanga(), FestivalMeta::nepal)
            }
            FestivalPack::IndiaCentral => DatedRule::india_central(),
        }
    }

    /// The saits it elects on its rules' days
    /// (`03-design/festival-sait.md`).
    #[must_use]
    pub fn saits(self) -> Vec<SaitRule> {
        match self {
            FestivalPack::Dharmasindhu
            | FestivalPack::PitriPaksha
            | FestivalPack::Gaudiya
            | FestivalPack::IndiaCentral => Vec::new(),
            FestivalPack::Nepal => SaitRule::nepal(),
            FestivalPack::NepalPanchanga => {
                let mut saits = SaitRule::nepal();
                saits.extend(SaitRule::nepal_panchanga());
                saits
            }
        }
    }

    /// Its rules counted from another's day.
    #[must_use]
    pub fn following(self) -> Vec<FollowingRule> {
        match self {
            FestivalPack::Dharmasindhu | FestivalPack::PitriPaksha | FestivalPack::Gaudiya => {
                Vec::new()
            }
            FestivalPack::Nepal => stamped(FollowingRule::nepal(), FestivalMeta::nepal),
            FestivalPack::NepalPanchanga => {
                let mut rules = FollowingRule::nepal();
                rules.extend(FollowingRule::nepal_panchanga());
                stamped(rules, FestivalMeta::nepal)
            }
            FestivalPack::IndiaCentral => FollowingRule::india_central(),
        }
    }

    /// The published sets it lays over its rules' answer.
    #[must_use]
    pub fn published(self) -> Vec<PublishedSet> {
        match self {
            FestivalPack::IndiaCentral => vec![PublishedSet::india_central()],
            _ => Vec::new(),
        }
    }
}

/// What a festival reckoning is asked: karmakala rules, Ekadashi rules,
/// rules counted from another's day, rules kept on a date and a family's
/// anniversaries kept by the tithi, each key once across all five, because
/// an answer is named by it.
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
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FestivalRequest {
    rules: Vec<FestivalRule>,
    ekadashis: Vec<EkadashiRule>,
    following: Vec<FollowingRule>,
    /// Left out when none, so a request without them hashes as it did.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    dated: Vec<DatedRule>,
    /// Left out when none, as `dated` is.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    anniversaries: Vec<Anniversary>,
    /// The published sets laid over the answer, in order; left out when
    /// none, as `dated` is (`03-design/published-overrides.md`).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    published: Vec<PublishedSet>,
    /// The saits elected on the observances' days, by their rule's key;
    /// left out when none, as `dated` is (`03-design/festival-sait.md`).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    saits: Vec<SaitRule>,
    /// What the answer keeps, by the rules' meta; left out when it keeps
    /// all, as `dated` is (`03-design/festival-metadata.md` §4).
    #[serde(skip_serializing_if = "Option::is_none")]
    only: Option<MetaFilter>,
}

impl From<FestivalPack> for FestivalRequest {
    fn from(pack: FestivalPack) -> FestivalRequest {
        FestivalRequest::new(Vec::new()).with_pack(pack)
    }
}

/// A rule of any of a request's four kinds, by its key.
trait Keyed {
    fn key(&self) -> &str;
}

/// A rule of a kind a pack holds, which carries its values; an
/// anniversary, a family's own, does not.
trait Stamped: Keyed {
    fn meta_mut(&mut self) -> &mut FestivalMeta;
}

impl Keyed for Anniversary {
    fn key(&self) -> &str {
        &self.key
    }
}

macro_rules! keyed {
    ($($rule:ty),+) => {$(
        impl Keyed for $rule {
            fn key(&self) -> &str {
                &self.key
            }
        }

        impl Stamped for $rule {
            fn meta_mut(&mut self) -> &mut FestivalMeta {
                &mut self.meta
            }
        }
    )+};
}

keyed!(FestivalRule, EkadashiRule, FollowingRule, DatedRule);

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
            dated: Vec::new(),
            anniversaries: Vec::new(),
            published: Vec::new(),
            saits: Vec::new(),
            only: None,
        }
    }

    /// The same request keeping, of its answer, only the rules `only`
    /// admits (`03-design/festival-metadata.md` §4). Every rule is still
    /// reckoned, so a day counted from a rule filtered out stands. A
    /// family's anniversaries, which carry no values, stand only when no
    /// filter is set.
    ///
    /// ```
    /// use teistro::{FestivalPack, FestivalRequest};
    /// use teistro::catalogue::FestivalKind;
    /// use teistro::festival::MetaFilter;
    ///
    /// let asked = FestivalRequest::from(FestivalPack::NepalPanchanga)
    ///     .keeping(MetaFilter { kinds: vec![FestivalKind::Gazetted], ..MetaFilter::default() });
    /// assert!(asked.keeps("GAI_JATRA") && !asked.keeps("KAG_NO_SUCH"));
    /// let text = r#"{"rules": "NEPAL_PANCHANGA", "only": {"kinds": ["GAZETTED"]}}"#;
    /// assert_eq!(FestivalRequest::from_json(text)?, asked);
    /// # Ok::<(), teistro::Error>(())
    /// ```
    #[must_use]
    pub fn keeping(mut self, only: MetaFilter) -> FestivalRequest {
        self.only = Some(only);
        self
    }

    /// Whether the answer keeps the day of this key.
    #[must_use]
    pub fn keeps(&self, key: &str) -> bool {
        let Some(only) = &self.only else {
            return true;
        };
        self.meta_of(key).is_some_and(|meta| only.admits(meta))
    }

    /// What the rule of this key says of itself, of whichever list holds
    /// it: `None` for a key no rule has, and for an anniversary, which
    /// carries no values.
    #[must_use]
    pub fn meta_of(&self, key: &str) -> Option<&FestivalMeta> {
        self.rules
            .iter()
            .find(|rule| rule.key == key)
            .map(|rule| &rule.meta)
            .or_else(|| {
                self.ekadashis
                    .iter()
                    .find(|rule| rule.key == key)
                    .map(|rule| &rule.meta)
            })
            .or_else(|| {
                self.following
                    .iter()
                    .find(|rule| rule.key == key)
                    .map(|rule| &rule.meta)
            })
            .or_else(|| {
                self.dated
                    .iter()
                    .find(|rule| rule.key == key)
                    .map(|rule| &rule.meta)
            })
            .or_else(|| {
                self.published
                    .iter()
                    .flat_map(|set| &set.entries)
                    .find_map(|entry| match entry {
                        PublishedEntry::Add {
                            key: added, meta, ..
                        } if added == key => Some(meta),
                        _ => None,
                    })
            })
    }

    /// Drops the rule with this key, of whichever kind.
    fn forget(&mut self, key: &str) {
        self.rules.retain(|held| held.key != key);
        self.ekadashis.retain(|held| held.key != key);
        self.following.retain(|held| held.key != key);
        self.dated.retain(|held| held.key != key);
        self.anniversaries.retain(|held| held.key != key);
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

    /// The same request with a rule kept on a date added, or put in place
    /// of the one with its key; a rule of another kind with that key goes.
    #[must_use]
    pub fn with_dated(mut self, rule: DatedRule) -> FestivalRequest {
        if let Err(rule) = replaced(&mut self.dated, rule) {
            self.forget(&rule.key);
            self.dated.push(rule);
        }
        self
    }

    /// The same request with an anniversary added, or put in place of the
    /// one with its key; a rule of another kind with that key goes.
    ///
    /// ```
    /// use teistro::FestivalRequest;
    /// use teistro::catalogue::{Masa, Tithi};
    /// use teistro::festival::{Anniversary, AnniversaryKind, FirstYearAdhika, InSamsarpa, JanmaOptions, Origin};
    ///
    /// let barkhi = Anniversary {
    ///     key: "GRANDFATHER".to_owned(),
    ///     origin: Origin::Lunar { month: Masa::Kartika, tithi: Tithi::ShuklaDashami, adhika: false, half: Default::default(), at: None },
    ///     kind: AnniversaryKind::Shraddha,
    ///     window: None,
    ///     first_year_adhika: FirstYearAdhika::Adhika,
    ///     in_samsarpa: InSamsarpa::Also,
    ///     janma: JanmaOptions::default(),
    /// };
    /// let asked = FestivalRequest::new(Vec::new()).with_anniversary(barkhi);
    /// assert_eq!(asked.anniversaries()[0].key, "GRANDFATHER");
    /// ```
    #[must_use]
    pub fn with_anniversary(mut self, anniversary: Anniversary) -> FestivalRequest {
        if let Err(anniversary) = replaced(&mut self.anniversaries, anniversary) {
            self.forget(&anniversary.key);
            self.anniversaries.push(anniversary);
        }
        self
    }

    /// The same request with a publisher's days laid over its answer,
    /// after the sets it already has: a later set's entry for an
    /// occurrence replaces an earlier one's
    /// (`03-design/published-overrides.md`).
    ///
    /// ```
    /// use teistro::{FestivalPack, FestivalRequest};
    /// use teistro::festival::PublishedSet;
    ///
    /// let set = PublishedSet::from_json(r#"{"format": "teistro.published/1", "publisher": "a patro printed in Nepal",
    ///     "id": "np-2083", "version": 1, "entries": [{"action": "SET", "rule": "HOLIKA",
    ///     "day": {"calendar": "GREGORIAN", "year": 2027, "month": 3, "day": 21}, "source": "p. 12"}]}"#)?;
    /// let asked = FestivalRequest::from(FestivalPack::Nepal).with_published(set);
    /// assert_eq!(asked.published().len(), 1);
    /// asked.check()?;
    /// # Ok::<(), teistro::Error>(())
    /// ```
    #[must_use]
    pub fn with_published(mut self, set: PublishedSet) -> FestivalRequest {
        self.published.push(set);
        self
    }

    /// The same request with a sait elected on its rule's days, or put in
    /// place of the one for that rule.
    ///
    /// ```
    /// use teistro::{FestivalPack, FestivalRequest};
    /// use teistro::festival::{SaitChoice, SaitRule};
    ///
    /// let mut earliest = SaitRule::nepal().remove(0);
    /// earliest.choose = SaitChoice::Earliest;
    /// let asked = FestivalRequest::from(FestivalPack::Nepal).with_sait(earliest.clone());
    /// assert_eq!(asked.saits().iter().filter(|s| s.rule == earliest.rule).count(), 1);
    /// asked.check()?;
    /// # Ok::<(), teistro::Error>(())
    /// ```
    #[must_use]
    pub fn with_sait(mut self, sait: SaitRule) -> FestivalRequest {
        match self.saits.iter_mut().find(|held| held.rule == sait.rule) {
            Some(held) => *held = sait,
            None => self.saits.push(sait),
        }
        self
    }

    /// The saits, in order.
    #[must_use]
    pub fn saits(&self) -> &[SaitRule] {
        &self.saits
    }

    /// The published sets, in the order they are laid over the answer.
    #[must_use]
    pub fn published(&self) -> &[PublishedSet] {
        &self.published
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
        let request = pack
            .following()
            .into_iter()
            .fold(request, FestivalRequest::with_following);
        let request = pack
            .dated()
            .into_iter()
            .fold(request, FestivalRequest::with_dated);
        let request = pack
            .saits()
            .into_iter()
            .fold(request, FestivalRequest::with_sait);
        pack.published()
            .into_iter()
            .fold(request, FestivalRequest::with_published)
    }

    /// The same request with each of these Ekadashi rules, as
    /// [`FestivalRequest::with_ekadashi`].
    #[must_use]
    pub fn with_ekadashis(self, rules: impl IntoIterator<Item = EkadashiRule>) -> FestivalRequest {
        rules.into_iter().fold(self, FestivalRequest::with_ekadashi)
    }

    /// A request for a kind of day's own rules
    /// ([`NextKind::rules`](teistro_panchanga::festival::NextKind::rules)).
    ///
    /// ```
    /// use teistro::{FestivalRequest, NextKind};
    ///
    /// let asked = FestivalRequest::of_kind(NextKind::Sankranti.rules());
    /// assert_eq!((asked.rules().len(), asked.dated().len()), (0, 12));
    /// asked.check()?;
    /// # Ok::<(), teistro::Error>(())
    /// ```
    #[must_use]
    pub fn of_kind(read: KindRules) -> FestivalRequest {
        let request = FestivalRequest::new(read.rules).with_ekadashis(read.ekadashis);
        read.dated
            .into_iter()
            .fold(request, FestivalRequest::with_dated)
    }

    /// The request cut to what finding `key`'s days needs: the rule of
    /// that key, of whichever kind, and the karmakala rule it counts from;
    /// the saits elected on them; and each published set's entries for
    /// them, a set left with none dropped. `None` when no rule has the key
    /// and no set adds a day under it.
    ///
    /// ```
    /// use teistro::{FestivalPack, FestivalRequest};
    ///
    /// let nepal = FestivalRequest::from(FestivalPack::Nepal);
    /// // The Terai's Holi is counted from the Holika fire.
    /// let holi = nepal.narrowed_to("HOLI_TERAI").expect("in the pack");
    /// assert_eq!((holi.rules().len(), holi.following().len()), (1, 1));
    /// assert_eq!(holi.rules()[0].key, holi.following()[0].after);
    /// holi.check()?;
    /// assert!(nepal.narrowed_to("NO_SUCH_DAY").is_none());
    /// # Ok::<(), teistro::Error>(())
    /// ```
    #[must_use]
    pub fn narrowed_to(&self, key: &str) -> Option<FestivalRequest> {
        let after = self
            .following
            .iter()
            .find(|rule| rule.key == key)
            .map(|rule| rule.after.as_str());
        let kept = |held: &str| held == key || after == Some(held);
        let added = |set: &PublishedSet| {
            set.entries.iter().any(
                |entry| matches!(entry, PublishedEntry::Add { key: added, .. } if added == key),
            )
        };
        let carried = self.rules.iter().any(|r| r.key == key)
            || self.ekadashis.iter().any(|r| r.key == key)
            || self.following.iter().any(|r| r.key == key)
            || self.dated.iter().any(|r| r.key == key)
            || self.anniversaries.iter().any(|r| r.key == key)
            || self.published.iter().any(added);
        if !carried {
            return None;
        }
        let published = self
            .published
            .iter()
            .filter_map(|set| {
                let entries: Vec<PublishedEntry> = set
                    .entries
                    .iter()
                    .filter(|entry| match entry {
                        PublishedEntry::Set { rule, .. }
                        | PublishedEntry::Withdraw { rule, .. } => kept(rule),
                        PublishedEntry::Add { key: added, .. } => kept(added),
                    })
                    .cloned()
                    .collect();
                (!entries.is_empty()).then(|| PublishedSet {
                    entries,
                    ..set.clone()
                })
            })
            .collect();
        Some(FestivalRequest {
            only: self.only.clone(),
            rules: self
                .rules
                .iter()
                .filter(|r| kept(&r.key))
                .cloned()
                .collect(),
            ekadashis: self
                .ekadashis
                .iter()
                .filter(|r| kept(&r.key))
                .cloned()
                .collect(),
            following: self
                .following
                .iter()
                .filter(|r| kept(&r.key))
                .cloned()
                .collect(),
            dated: self
                .dated
                .iter()
                .filter(|r| kept(&r.key))
                .cloned()
                .collect(),
            anniversaries: self
                .anniversaries
                .iter()
                .filter(|r| kept(&r.key))
                .cloned()
                .collect(),
            published,
            saits: self
                .saits
                .iter()
                .filter(|s| kept(&s.rule))
                .cloned()
                .collect(),
        })
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

    /// The rules kept on a date, in order.
    #[must_use]
    pub fn dated(&self) -> &[DatedRule] {
        &self.dated
    }

    /// The anniversaries, in order.
    #[must_use]
    pub fn anniversaries(&self) -> &[Anniversary] {
        &self.anniversaries
    }

    /// How many days the request reaches past an observance's own:
    /// the most any rule counts from another's, or from the Sun's entry,
    /// or an Ekadashi's mahadvadashis read past the 11th's day.
    #[must_use]
    pub fn reach(&self) -> u8 {
        self.following
            .iter()
            .map(|rule| rule.days)
            .chain(self.dated.iter().map(DatedRule::reach))
            .chain(self.ekadashis.iter().map(EkadashiRule::reach))
            .max()
            .unwrap_or(0)
    }

    /// Refuses what no reckoning could answer, naming the field under
    /// `rules`, `ekadashis`, `following`, `dated` or `anniversaries`: a rule
    /// its own `check` refuses, a key given twice across all five, which would answer two
    /// observances a reader cannot tell apart, or a rule counted from one
    /// that is not a karmakala rule of the request.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on `rules[i]`, `ekadashis[i]`, `following[i]`,
    /// `dated[i]` or `anniversaries[i]` and the rule's own field.
    pub fn check(&self) -> Result<(), Error> {
        let rules = self.rules.iter().map(|rule| (&rule.key, rule.check()));
        let ekadashis = self.ekadashis.iter().map(|rule| (&rule.key, rule.check()));
        let following = self.following.iter().map(|rule| (&rule.key, rule.check()));
        let dated = self.dated.iter().map(|rule| (&rule.key, rule.check()));
        let anniversaries = self
            .anniversaries
            .iter()
            .map(|asked| (&asked.key, asked.check()));
        let mut seen = std::collections::BTreeSet::new();
        for (list, items) in [
            ("rules", rules.collect::<Vec<_>>()),
            ("ekadashis", ekadashis.collect()),
            ("following", following.collect()),
            ("dated", dated.collect()),
            ("anniversaries", anniversaries.collect()),
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
        self.check_published()?;
        for (index, sait) in self.saits.iter().enumerate() {
            let at = format!("saits[{index}]");
            sait.check().map_err(|why| why.under(&at))?;
            let observed = self.rules.iter().any(|r| r.key == sait.rule)
                || self.following.iter().any(|r| r.key == sait.rule);
            if !observed {
                return Err(Error::invalid_arg(format!(
                    "`{}` is no karmakala or following rule of the request; a sait is elected on the day of a rule the request carries",
                    sait.rule
                ))
                .with_field(format!("{at}.rule")));
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

    /// Each set read as [`PublishedSet::check`] reads it, each `SET` and
    /// `WITHDRAW` naming a rule the request carries and each `ADD` a key
    /// none of its rules uses.
    fn check_published(&self) -> Result<(), Error> {
        let carried = |key: &str| {
            self.rules.iter().any(|r| r.key == key)
                || self.ekadashis.iter().any(|r| r.key == key)
                || self.following.iter().any(|r| r.key == key)
                || self.dated.iter().any(|r| r.key == key)
                || self.anniversaries.iter().any(|r| r.key == key)
        };
        for (index, set) in self.published.iter().enumerate() {
            let at = format!("published[{index}]");
            set.check().map_err(|why| why.under(&at))?;
            for (entry_at, entry) in set.entries.iter().enumerate() {
                let field = format!("{at}.entries[{entry_at}]");
                match entry {
                    PublishedEntry::Set { rule, .. } | PublishedEntry::Withdraw { rule, .. }
                        if !carried(rule) =>
                    {
                        return Err(Error::invalid_arg(format!(
                            "`{rule}` is no rule of the request; a published day is laid over a rule's, so the request carries the rule it names"
                        ))
                        .with_field(format!("{field}.rule")));
                    }
                    PublishedEntry::Add { key, .. } if carried(key) => {
                        return Err(Error::invalid_arg(format!(
                            "`{key}` is a rule of the request; publish its day with `SET`, since `ADD` is for a day no rule computes"
                        ))
                        .with_field(format!("{field}.key")));
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    /// The request a binding writes as `festivals`, read and checked.
    ///
    /// `rules` names a [`FestivalPack`] (`"DHARMASINDHU"`), or is a list
    /// whose items each name a pack or spell a rule out, tried in order: a
    /// [`SaitRule`] when it has `windows`, an [`EkadashiRule`] when it has
    /// a `vedha`, an [`Anniversary`] when it has an `origin`, a
    /// [`FollowingRule`] when it has an `after`, a [`DatedRule`] when it
    /// has one of [`Dated::KEYS`], else a [`FestivalRule`]. A later rule
    /// replaces an earlier one with its key. A
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
                Value::Object(spelt) if listed => {
                    request = match told(spelt, &field)? {
                        Some(Spelt::Sait) => {
                            let sait: SaitRule = teistro_core::strict::read_value(item, &field)?;
                            sait.check().map_err(|why| why.under(&field))?;
                            request.with_sait(sait)
                        }
                        Some(Spelt::Ekadashi) => {
                            let rule: EkadashiRule =
                                teistro_core::strict::read_value(item, &field)?;
                            rule.check().map_err(|why| why.under(&field))?;
                            request.with_ekadashi(rule)
                        }
                        Some(Spelt::Anniversary) => {
                            let asked: Anniversary =
                                teistro_core::strict::read_value(item, &field)?;
                            asked.check().map_err(|why| why.under(&field))?;
                            request.with_anniversary(asked)
                        }
                        Some(Spelt::Following) => {
                            let rule: FollowingRule =
                                teistro_core::strict::read_value(item, &field)?;
                            rule.check().map_err(|why| why.under(&field))?;
                            request.with_following(rule)
                        }
                        Some(Spelt::Dated) => {
                            let rule: DatedRule = teistro_core::strict::read_value(item, &field)?;
                            rule.check().map_err(|why| why.under(&field))?;
                            request.with_dated(rule)
                        }
                        None => {
                            let rule: FestivalRule =
                                teistro_core::strict::read_value(item, &field)?;
                            rule.check().map_err(|why| why.under(&field))?;
                            request.with_rule(rule)
                        }
                    };
                }
                _ => {
                    return Err(Error::invalid_arg(
                        "`rules` is a pack's name or a list of names and rules",
                    )
                    .with_field(field));
                }
            }
        }
        for (index, set) in asked.published.iter().enumerate() {
            let field = format!("{FESTIVALS}.published[{index}]");
            let set: PublishedSet = teistro_core::strict::read_value(set, &field)?;
            request = request.with_published(set);
        }
        if let Some(only) = asked.only {
            request = request.keeping(only);
        }
        request.check().map_err(|why| why.under(FESTIVALS))?;
        Ok(request)
    }
}

/// The record every binding writes the request as.
const FESTIVALS: &str = "festivals";

/// A rule spelt out in a list of rules, other than a [`FestivalRule`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Spelt {
    Sait,
    Ekadashi,
    Anniversary,
    Following,
    Dated,
}

/// How a rule spelt out is told apart, in the order the reader tries:
/// the first kind whose keys the item carries any one of, and a
/// [`FestivalRule`] when it carries none. The reader and the schema
/// (`x-teistro-when`) both read this table.
const SPELT: [(Spelt, &[&str]); 5] = [
    (Spelt::Sait, &["windows"]),
    (Spelt::Ekadashi, &["vedha"]),
    (Spelt::Anniversary, &["origin"]),
    (Spelt::Following, &["after"]),
    (Spelt::Dated, &Dated::KEYS),
];

/// The kind of rule `item` spells out, `None` for a [`FestivalRule`].
///
/// # Errors
///
/// `INVALID_ARG` for an item giving two of one kind's keys, two dates,
/// which serde would read as whichever it met first.
fn told(item: &serde_json::Map<String, Value>, field: &str) -> Result<Option<Spelt>, Error> {
    let Some((kind, keys)) = SPELT
        .iter()
        .find(|(_, keys)| keys.iter().any(|key| item.contains_key(*key)))
    else {
        return Ok(None);
    };
    let given: Vec<&str> = keys
        .iter()
        .copied()
        .filter(|key| item.contains_key(*key))
        .collect();
    if let [_, _, ..] = given.as_slice() {
        return Err(Error::invalid_arg(format!(
            "a rule gives one of `{}`, and this gives `{}`",
            keys.join("`, `"),
            given.join("` and `")
        ))
        .with_field(field));
    }
    Ok(Some(*kind))
}

/// The pack a name names, or the refusal listing them.
pub(crate) fn pack(name: &str, field: &str) -> Result<FestivalPack, Error> {
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

// [`FestivalRequest`] as the bindings write it. `rules` is read in a
// second step, because each item is a name or a rule and the refusal
// should say which item was wrong rather than that no shape matched.
/// Festival rules to reckon over an almanac's days
/// (`03-design/festival-rules.md` §7), and the published days laid over
/// their answer.
#[derive(Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "FestivalRequest"))]
pub(crate) struct Asked {
    /// The rules: a pack's name, or a list of names and rules spelt out.
    #[cfg_attr(feature = "schema", schemars(with = "FestivalRules"))]
    rules: Value,
    /// The published sets laid over the answer, a later one's entry
    /// replacing an earlier one's for the same occurrence.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(feature = "schema", schemars(with = "Vec<PublishedSet>"))]
    published: Vec<Value>,
    /// What the answer keeps, by the rules' `meta`; every rule when left
    /// out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    only: Option<MetaFilter>,
}

/// The schema of [`Asked::rules`]: a pack's name, or a list of
/// [`FestivalRuleItem`]s. The reader takes it in a second step, so the
/// type stands only in the schema.
#[cfg(feature = "schema")]
struct FestivalRules;

/// The schema of one item of a list of rules: a pack's name, or a rule
/// spelt out, picked by [`SPELT`].
#[cfg(feature = "schema")]
struct FestivalRuleItem;

#[cfg(feature = "schema")]
fn pack_names() -> Value {
    serde_json::json!({
        "type": "string",
        "enum": FestivalPack::ALL.iter().map(|pack| pack.key()).collect::<Vec<_>>(),
    })
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for FestivalRules {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "FestivalRules".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "description": "The rules: a pack the SDK ships, by name, or a list whose items each name a pack or spell a rule out; a later rule replaces an earlier one with its key.",
            "anyOf": [
                pack_names(),
                { "type": "array", "items": generator.subschema_for::<FestivalRuleItem>() },
            ],
        })
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for FestivalRuleItem {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "FestivalRuleItem".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let mut options = vec![pack_names()];
        for (kind, keys) in SPELT {
            let mut option = match kind {
                Spelt::Sait => generator.subschema_for::<SaitRule>(),
                Spelt::Ekadashi => generator.subschema_for::<EkadashiRule>(),
                Spelt::Anniversary => generator.subschema_for::<Anniversary>(),
                Spelt::Following => generator.subschema_for::<FollowingRule>(),
                Spelt::Dated => generator.subschema_for::<DatedRule>(),
            };
            option.insert("x-teistro-when".into(), keys.to_vec().into());
            options.push(option.into());
        }
        options.push(generator.subschema_for::<FestivalRule>().into());
        schemars::json_schema!({
            "description": "A pack the SDK ships, by name, or a rule spelt out, told apart by its keys: the first kind whose keys it carries any one of, else a festival rule.",
            "anyOf": options,
        })
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
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
    fn each_kind_of_rule_is_told_by_the_keys_serde_writes_it_with() {
        let anniversary = Anniversary {
            key: "GRANDFATHER".to_owned(),
            origin: teistro_panchanga::festival::Origin::Lunar {
                month: teistro_core::catalogue::Masa::Kartika,
                tithi: teistro_core::catalogue::Tithi::ShuklaDashami,
                adhika: false,
                half: teistro_panchanga::festival::Half::default(),
                at: None,
            },
            kind: teistro_panchanga::festival::AnniversaryKind::Shraddha,
            window: None,
            first_year_adhika: teistro_panchanga::festival::FirstYearAdhika::Adhika,
            in_samsarpa: teistro_panchanga::festival::InSamsarpa::Also,
            janma: teistro_panchanga::festival::JanmaOptions::default(),
        };
        let written = |rule: Value| told(rule.as_object().unwrap(), "f").unwrap();
        assert_eq!(
            written(serde_json::to_value(&SaitRule::nepal()[0]).unwrap()),
            Some(Spelt::Sait)
        );
        assert_eq!(
            written(serde_json::to_value(&EkadashiRule::dharmasindhu()[0]).unwrap()),
            Some(Spelt::Ekadashi)
        );
        assert_eq!(
            written(serde_json::to_value(&anniversary).unwrap()),
            Some(Spelt::Anniversary)
        );
        assert_eq!(
            written(serde_json::to_value(&FollowingRule::nepal()[0]).unwrap()),
            Some(Spelt::Following)
        );
        for rule in DatedRule::nepal_panchanga() {
            assert_eq!(
                written(serde_json::to_value(&rule).unwrap()),
                Some(Spelt::Dated)
            );
        }
        for rule in FestivalRule::nepal_panchanga() {
            assert_eq!(
                written(serde_json::to_value(&rule).unwrap()),
                None,
                "{}",
                rule.key
            );
        }
    }

    #[test]
    fn a_rule_giving_two_dates_is_refused_not_read_as_the_first() {
        let text = r#"{"rules": [{"key": "X", "source": "a test",
            "civilDate": {"calendar": "BIKRAM_SAMBAT", "month": 1, "day": 1},
            "sunEnters": {"nakshatra": "ARDRA"}}]}"#;
        let refused = FestivalRequest::from_json(text).unwrap_err();
        assert_eq!(refused.field(), Some("festivals.rules[0]"));
        assert!(
            refused.message.contains("`civilDate` and `sunEnters`"),
            "{}",
            refused.message
        );
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

    /// The rules as they stand before a pack stamps its values on them.
    fn unstamped(rules: &[FestivalRule]) -> Vec<FestivalRule> {
        rules
            .iter()
            .cloned()
            .map(|mut rule| {
                rule.meta = FestivalMeta::default();
                rule
            })
            .collect()
    }

    #[test]
    fn the_nepal_pack_is_dharmasindhu_s_with_its_counted_days() {
        let nepal = FestivalRequest::from_json(r#"{"rules": "NEPAL"}"#).unwrap();
        assert_eq!(nepal, FestivalRequest::from(FestivalPack::Nepal));
        assert_eq!(unstamped(nepal.rules()), FestivalRule::nepal());
        assert_ne!(unstamped(nepal.rules()), FestivalRule::dharmasindhu());
        let following: Vec<&str> = nepal.following().iter().map(|r| r.key.as_str()).collect();
        let shipped_keys = FollowingRule::nepal();
        let keys: Vec<&str> = shipped_keys.iter().map(|r| r.key.as_str()).collect();
        assert_eq!(following, keys);
        assert_eq!(nepal.reach(), 1);
        let shipped = FestivalRequest::from(FestivalPack::Dharmasindhu);
        assert_eq!(shipped.following(), []);
        assert_eq!(shipped.reach(), 0);
    }

    #[test]
    fn the_nepal_panchanga_pack_is_nepal_s_and_its_fitted_days() {
        let asked = FestivalRequest::from_json(r#"{"rules": "NEPAL_PANCHANGA"}"#).unwrap();
        assert_eq!(asked, FestivalRequest::from(FestivalPack::NepalPanchanga));
        let nepal = FestivalRule::nepal();
        assert_eq!(unstamped(&asked.rules()[..nepal.len()]), nepal);
        assert_eq!(
            asked.rules().len(),
            nepal.len() + FestivalRule::nepal_panchanga().len()
        );
        let counted = FollowingRule::nepal().len() + FollowingRule::nepal_panchanga().len();
        assert_eq!(asked.following().len(), counted);
        // The Bhumi puja, three days after the Sun enters Ardra.
        assert_eq!(asked.reach(), 3);
        asked.check().unwrap();
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
