//! **Why each knob has its value, and where two values part** (SR-110;
//! `03-design/settings-and-profiles.md` §3.5).
//!
//! Resolution lays patches over the root, and [`Origins`] is the
//! accounting it keeps as it lays them: each knob's last writer, noted by
//! the one merge ([`Settings::patched`]) at the moment it sets the knob,
//! so the layer an explanation names is what the merge did and never a
//! second reading of the patches. [`Resolved::explained`] reads it with
//! the knob's value, its citation and its effect note;
//! [`Resolved::diff`] compares two values over the same enumeration of
//! knobs ([`Settings::knobs`]), so the two can never list different knobs.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::catalogue::Source;
use crate::envelope::Hash;
use crate::error::Error;

use super::{Diagnostics, Profile, ProfileId, Resolved, Settings, SettingsPatch};

/// The layer of resolution that set a knob, in the order the layers are
/// laid: the root, a profile (a shipped profile's base, then the profile),
/// the context's patch, then a request's (`03-design/settings-and-profiles.md`
/// §3.2). A settings value handed in whole, a snapshot or a document, is
/// its own layer.
///
/// ```
/// use teistro_core::settings::Layer;
///
/// assert_eq!(serde_json::to_value(Layer::Context).unwrap(), "CONTEXT");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Layer {
    /// The root, every knob set and cited ([`super::root`]).
    Root,
    /// A profile: the one the context names or the profile it extends,
    /// which the row's `profile` names.
    Profile,
    /// The patch a context was built with.
    Context,
    /// A patch laid over a context for one call or a derived context
    /// (`ctx.with_patch`, `ts_context_derive`, a binding's `withSettings`).
    Request,
    /// A settings value handed in whole, a snapshot or a document: every
    /// knob it did not have patched over it.
    Document,
}

/// What a difference in a knob moves, declared beside each knob: what
/// SR-94's comparison of two almanacs reads a differing knob as
/// (`SKY` a model reason, `SUNRISE` the sunrise's, `RULE` a rule's).
///
/// ```
/// use teistro_core::settings::{Frame, Moves};
///
/// // The ayanamsha moves every sidereal longitude.
/// assert_eq!(Frame::MOVES[1], Moves::Sky);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Moves {
    /// The sky: where the grahas, the lagna or the cusps stand.
    Sky,
    /// The moment of sunrise, and with it every limb read at sunrise.
    Sunrise,
    /// How the same sky is read: a table, a division, a rule.
    Rule,
    /// Which instant a civil time names.
    Time,
    /// How an answer is written or computed, never what it is.
    Form,
}

/// One knob of the settings document: its group, its name and what a
/// difference in it moves.
///
/// ```
/// use teistro_core::settings::Settings;
///
/// let knob = Settings::knobs()[0];
/// assert_eq!((knob.path(), knob.effect_key()), (String::from("frame.zodiac"), String::from("sdk.settings.effect.frame.zodiac")));
/// let basis = Settings::knobs().into_iter().find(|knob| knob.name == "ayanamsha_basis").unwrap();
/// assert_eq!(basis.effect_key(), "sdk.settings.effect.frame.ayanamshaBasis");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Knob {
    /// The group, as the document names it: `frame`.
    pub group: &'static str,
    /// The knob, as the document names it: `ayanamsha`.
    pub name: &'static str,
    /// What a difference in it moves.
    pub moves: Moves,
}

impl Knob {
    /// `group.knob`, as a refusal and a citation name it.
    #[must_use]
    pub fn path(&self) -> String {
        format!("{}.{}", self.group, self.name)
    }

    /// The key of its effect note in the locale packs:
    /// `sdk.settings.effect.<group>.<knob>`, the knob in camelCase as a
    /// pack's keys are (`frame.ayanamshaBasis`).
    #[must_use]
    pub fn effect_key(&self) -> String {
        let mut key = format!("sdk.settings.effect.{}.", self.group);
        let mut upper = false;
        for c in self.name.chars() {
            if c == '_' {
                upper = true;
            } else if upper {
                key.push(c.to_ascii_uppercase());
                upper = false;
            } else {
                key.push(c);
            }
        }
        key
    }
}

/// Who set one knob: the layer, the profile where a profile did, and the
/// citation the layer gives for it (none for a patch or a document,
/// which are the consumer's choice).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Origin {
    /// The layer.
    pub layer: Layer,
    /// The profile that set it, for [`Layer::Profile`].
    pub profile: Option<ProfileId>,
    /// The citation the layer gives for its value.
    pub source: Option<Source>,
}

/// Where every knob of a resolved value came from: the accounting
/// resolution keeps as it lays each patch, the last writer of each knob.
///
/// ```
/// use teistro_core::settings::{Layer, Profile, SettingsPatch};
///
/// let committee = Profile::shipped("nepali-committee").unwrap().resolve(&SettingsPatch::default()).unwrap();
/// // Its own patch sets the siddhanta, its base the civil calendar, the root the node.
/// let of = |group, knob| committee.origins.of(group, knob);
/// assert_eq!(of("frame", "siddhanta").profile.map(|p| p.to_string()).as_deref(), Some("nepali-committee"));
/// assert_eq!(of("calendars", "civil_calendar").profile.map(|p| p.to_string()).as_deref(), Some("nepali-drik"));
/// assert_eq!(of("frame", "node").layer, Layer::Root);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Origins {
    /// The layer of every knob no patch set.
    under: Layer,
    /// The knobs a patch set, each with its last writer.
    set: BTreeMap<(&'static str, &'static str), Origin>,
}

impl Origins {
    /// Every knob from one layer: the root's, or a document's.
    pub(crate) fn under(layer: Layer) -> Origins {
        Origins {
            under: layer,
            set: BTreeMap::new(),
        }
    }

    /// Who set `group.knob`; the layer underneath every patch when none did.
    #[must_use]
    pub fn of(&self, group: &str, knob: &str) -> Origin {
        self.set
            .iter()
            .find(|((g, k), _)| *g == group && *k == knob)
            .map_or_else(
                || Origin {
                    layer: self.under,
                    profile: None,
                    source: match self.under {
                        Layer::Root => super::profiles::root_source(group, knob),
                        _ => None,
                    },
                },
                |(_, origin)| origin.clone(),
            )
    }
}

impl Settings {
    /// Every knob of every group, in document order, with what a
    /// difference in it moves: the one enumeration explain, diff and the
    /// knob lints all walk.
    ///
    /// ```
    /// use teistro_core::settings::Settings;
    ///
    /// assert_eq!(Settings::knobs().len(), Settings::knob_paths().len());
    /// ```
    #[must_use]
    pub fn knobs() -> Vec<Knob> {
        super::groups()
            .into_iter()
            .flat_map(|(group, names, moves)| {
                names
                    .iter()
                    .zip(moves.iter())
                    .map(move |(name, moves)| Knob {
                        group,
                        name,
                        moves: *moves,
                    })
            })
            .collect()
    }

    /// `patch` laid over these settings as the `layer` that sets what it
    /// names, each knob it sets noted in `origins` with `profile`'s
    /// citation for it: the merge resolution makes, with its accounting.
    pub(crate) fn laid(
        self,
        patch: &SettingsPatch,
        origins: &mut Origins,
        layer: Layer,
        profile: Option<&Profile>,
    ) -> Settings {
        self.patched_noting(patch, &mut |group, knob| {
            let origin = Origin {
                layer,
                profile: profile.map(|p| p.id.clone()),
                source: profile.and_then(|p| p.cited(group, knob)),
            };
            origins.set.insert((group, knob), origin);
        })
    }
}

/// The knobs an explanation or a comparison is asked about: each a group
/// (`frame`) or a knob (`frame.ayanamsha`); every knob when left out.
///
/// ```
/// use teistro_core::settings::SettingsQuery;
///
/// let asked = SettingsQuery::from_json(r#"{"knobs": ["frame", "day.sunrise"]}"#)?;
/// assert_eq!(asked.selected()?.len(), 12);
/// let refused = SettingsQuery::from_json(r#"{"knobs": ["frame.nod"]}"#)?.selected().unwrap_err();
/// assert_eq!(refused.field(), Some("knobs[0]"));
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SettingsQuery {
    /// Groups (`frame`) or knobs (`frame.ayanamsha`), in any order; the
    /// answer keeps document order. Every knob when left out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub knobs: Option<Vec<String>>,
}

impl SettingsQuery {
    /// The query read from its JSON, as a record reads its request:
    /// `{}` for every knob.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for text that is not a query, named at the field.
    pub fn from_json(json: &str) -> Result<SettingsQuery, Error> {
        crate::strict::deserialize_str(json, "")
    }

    /// The knobs asked, in document order.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` at `knobs[i]` for an entry that is neither a group
    /// nor a knob.
    pub fn selected(&self) -> Result<Vec<Knob>, Error> {
        let every = Settings::knobs();
        let Some(asked) = &self.knobs else {
            return Ok(every);
        };
        for (at, name) in asked.iter().enumerate() {
            if !every.iter().any(|knob| names(knob, name)) {
                return Err(unknown_knob(at, name));
            }
        }
        Ok(every
            .into_iter()
            .filter(|knob| asked.iter().any(|name| names(knob, name)))
            .collect())
    }
}

/// What `settings.diff` reads: the knobs to compare, as
/// [`SettingsQuery`] names them; a record of its own so that each record
/// has its own request type (a comparison may come to read more).
///
/// ```
/// use teistro_core::settings::SettingsDiffQuery;
///
/// let asked = SettingsDiffQuery::from_json(r#"{"knobs": ["frame"]}"#)?;
/// assert_eq!(asked.query().selected()?.len(), 11);
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SettingsDiffQuery {
    /// Groups (`frame`) or knobs (`frame.ayanamsha`) to compare, in any
    /// order; the answer keeps document order. Every knob when left out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub knobs: Option<Vec<String>>,
}

impl SettingsDiffQuery {
    /// The query read from its JSON, as a record reads its request.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` for text that is not a query, named at the field.
    pub fn from_json(json: &str) -> Result<SettingsDiffQuery, Error> {
        crate::strict::deserialize_str(json, "")
    }

    /// The selection it names.
    #[must_use]
    pub fn query(self) -> SettingsQuery {
        SettingsQuery { knobs: self.knobs }
    }
}

/// Whether `name` is the knob's group or its path.
fn names(knob: &Knob, name: &str) -> bool {
    name == knob.group
        || name
            .split_once('.')
            .is_some_and(|(group, knob_name)| group == knob.group && knob_name == knob.name)
}

#[cold]
#[inline(never)]
fn unknown_knob(at: usize, name: &str) -> Error {
    let mut groups: Vec<&str> = Settings::knobs().iter().map(|knob| knob.group).collect();
    groups.dedup();
    Error::invalid_arg(format!("no settings group or knob `{name}`"))
        .with_field(format!("knobs[{at}]"))
        .with_hint(format!(
            "a group ({}) or a knob, `frame.ayanamsha`",
            groups.join(", ")
        ))
}

/// A citation as an answer writes it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct KnobSource {
    /// The text, the record or the body: `BPHS`, `Nepal Panchanga Nirnayak Vikas Samiti`.
    pub text: String,
    /// Where in it: a chapter and verse, a page, a measurement.
    pub reference: String,
}

impl From<Source> for KnobSource {
    fn from(source: Source) -> KnobSource {
        KnobSource {
            text: source.text.to_owned(),
            reference: source.reference.to_owned(),
        }
    }
}

/// One knob explained.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct KnobExplained {
    /// `group.knob`.
    pub knob: String,
    /// Its value, as the canonical settings document writes it.
    pub value: serde_json::Value,
    /// The layer that set it.
    pub layer: Layer,
    /// The profile that set it, where a profile did.
    pub profile: Option<ProfileId>,
    /// Why it has this value: the root's or the profile's citation; none
    /// where a patch or a document set it, which is the caller's choice.
    pub citation: Option<KnobSource>,
    /// What a difference in it moves.
    pub moves: Moves,
    /// What it changes, in one line: the `sdk.settings.effect.<group>.<knob>`
    /// message in the context's locale.
    pub effect: String,
}

/// Every knob asked of a resolved value, each with the layer that set it,
/// why and what it changes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SettingsExplained {
    /// The profile the value was resolved from.
    pub profile: ProfileId,
    /// The value's hash, as every result computed under it carries.
    pub settings_hash: Hash,
    /// The knobs asked, in document order.
    pub knobs: Vec<KnobExplained>,
}

/// One knob on which two values differ.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct KnobDifference {
    /// `group.knob`.
    pub knob: String,
    /// The value under the first settings.
    pub value: serde_json::Value,
    /// The value under the other.
    pub other_value: serde_json::Value,
    /// What the difference moves.
    pub moves: Moves,
    /// What the knob changes, in one line, in the first context's locale.
    pub effect: String,
}

/// The knobs on which two resolved values differ.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SettingsDiff {
    /// The first value's profile.
    pub profile: ProfileId,
    /// The other's.
    pub other_profile: ProfileId,
    /// The first value's hash.
    pub settings_hash: Hash,
    /// The other's.
    pub other_settings_hash: Hash,
    /// Each knob asked whose values differ, in document order; none when
    /// the two hash alike.
    pub knobs: Vec<KnobDifference>,
}

/// The settings document as JSON, one object per group.
fn document(settings: &Settings) -> Result<serde_json::Value, Error> {
    serde_json::to_value(settings)
        .map_err(|why| Error::internal(format!("serde cannot write the settings: {why}")))
}

/// `knob`'s value in the document.
fn value_of(document: &serde_json::Value, knob: &Knob) -> Result<serde_json::Value, Error> {
    document
        .get(knob.group)
        .and_then(|group| group.get(knob.name))
        .cloned()
        .ok_or_else(|| Error::internal(format!("the settings document has no `{}`", knob.path())))
}

impl Resolved {
    /// **Why each knob has its value**: the knobs `asked` names, each with
    /// its value, the layer that set it, its citation and its effect note,
    /// which `effect` renders from the note's key.
    ///
    /// ```
    /// use teistro_core::settings::{Layer, Profile, SettingsQuery, SettingsPatch};
    ///
    /// let drik = Profile::shipped("nepali-drik").unwrap().resolve(&SettingsPatch::default()).unwrap();
    /// let explained = drik.explained(&SettingsQuery::default(), &|key| key.to_owned())?;
    /// // A shipped profile cites every knob.
    /// assert!(explained.knobs.iter().all(|knob| knob.citation.is_some()));
    /// let centre = explained.knobs.iter().find(|knob| knob.knob == "frame.centre").unwrap();
    /// assert_eq!((centre.layer, centre.value.as_str()), (Layer::Profile, Some("TOPOCENTRIC")));
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` at `knobs[i]` for an entry that names no knob.
    pub fn explained(
        &self,
        asked: &SettingsQuery,
        effect: &dyn Fn(&str) -> String,
    ) -> Result<SettingsExplained, Error> {
        let written = document(&self.settings)?;
        let knobs = asked
            .selected()?
            .iter()
            .map(|knob| {
                let origin = self.origins.of(knob.group, knob.name);
                Ok(KnobExplained {
                    knob: knob.path(),
                    value: value_of(&written, knob)?,
                    layer: origin.layer,
                    profile: origin.profile,
                    citation: origin.source.map(KnobSource::from),
                    moves: knob.moves,
                    effect: effect(&knob.effect_key()),
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(SettingsExplained {
            profile: self.profile.clone(),
            settings_hash: self.settings.hash(),
            knobs,
        })
    }

    /// **Where two values part**: each knob `asked` names on which these
    /// settings and `other` differ, both values beside its effect note.
    ///
    /// ```
    /// use teistro_core::settings::{Profile, SettingsQuery, SettingsPatch};
    ///
    /// let resolve = |id| Profile::shipped(id).unwrap().resolve(&SettingsPatch::default()).unwrap();
    /// let diff = resolve("nepali-committee").diff(&resolve("nepali-drik"), &SettingsQuery::default(), &|key| key.to_owned())?;
    /// let knobs: Vec<&str> = diff.knobs.iter().map(|knob| knob.knob.as_str()).collect();
    /// assert!(knobs.contains(&"frame.siddhanta") && knobs.contains(&"frame.ayanamsha"));
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` at `knobs[i]` for an entry that names no knob.
    pub fn diff(
        &self,
        other: &Resolved,
        asked: &SettingsQuery,
        effect: &dyn Fn(&str) -> String,
    ) -> Result<SettingsDiff, Error> {
        let mine = document(&self.settings)?;
        let theirs = document(&other.settings)?;
        let mut knobs = Vec::new();
        for knob in asked.selected()? {
            let value = value_of(&mine, &knob)?;
            let other_value = value_of(&theirs, &knob)?;
            if value != other_value {
                knobs.push(KnobDifference {
                    knob: knob.path(),
                    value,
                    other_value,
                    moves: knob.moves,
                    effect: effect(&knob.effect_key()),
                });
            }
        }
        Ok(SettingsDiff {
            profile: self.profile.clone(),
            other_profile: other.profile.clone(),
            settings_hash: self.settings.hash(),
            other_settings_hash: other.settings.hash(),
            knobs,
        })
    }

    /// **The one merge for a call**: `patch` laid over these settings as a
    /// request's patch, under the same profile, and the result checked
    /// once; what a derived context resolves with, so its explanation
    /// names the request for every knob the patch set and keeps the rest's
    /// layers.
    ///
    /// ```
    /// use teistro_core::settings::{Layer, Node, Profile, SettingsPatch};
    ///
    /// let drik = Profile::shipped("nepali-drik").unwrap().resolve(&SettingsPatch::default()).unwrap();
    /// let mut patch = SettingsPatch::default();
    /// patch.frame.node = Some(Node::True);
    /// let derived = drik.derived(&patch)?;
    /// assert_eq!(derived.origins.of("frame", "node").layer, Layer::Request);
    /// assert_eq!(derived.origins.of("frame", "centre").layer, Layer::Profile);
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming the first knob a coherence error names, under
    /// `settings`, as [`Settings::derived`].
    pub fn derived(&self, patch: &SettingsPatch) -> Result<Resolved, Error> {
        let mut origins = self.origins.clone();
        self.settings
            .clone()
            .laid(patch, &mut origins, Layer::Request, None)
            .validated_with(self.profile.clone(), origins)
            .map_err(|diagnostics: Diagnostics| Error::from(diagnostics).under("settings"))
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use std::collections::BTreeSet;

    use super::super::profiles::ROOT_SOURCES;
    use super::super::{Node, SHIPPED_PROFILES};
    use super::*;

    fn shipped(id: &str) -> Profile {
        Profile::shipped(id).unwrap_or_else(|| panic!("{id}"))
    }

    /// The knobs a patch names, read from its JSON rather than from the
    /// merge: the independent reading the merge's accounting is held to.
    fn named(patch: &SettingsPatch) -> BTreeSet<String> {
        let written = serde_json::to_value(patch).unwrap();
        let mut out = BTreeSet::new();
        for (group, knobs) in written.as_object().unwrap() {
            for (knob, value) in knobs.as_object().unwrap() {
                if !value.is_null() {
                    out.insert(format!("{group}.{knob}"));
                }
            }
        }
        out
    }

    #[test]
    fn every_knob_of_the_root_is_cited() {
        let knobs: BTreeSet<String> = Settings::knobs().iter().map(Knob::path).collect();
        let cited: Vec<&str> = ROOT_SOURCES.iter().map(|cited| cited.knob).collect();
        let distinct: BTreeSet<String> = cited.iter().map(|knob| (*knob).to_owned()).collect();
        assert_eq!(cited.len(), distinct.len(), "a knob cited twice");
        assert_eq!(distinct, knobs, "the root cites exactly the knobs");
        assert!(
            ROOT_SOURCES
                .iter()
                .all(|cited| !cited.source.text.is_empty() && !cited.source.reference.is_empty())
        );
    }

    #[test]
    fn every_knob_a_shipped_profile_sets_is_cited_by_it() {
        let mut missing = Vec::new();
        for id in SHIPPED_PROFILES {
            let profile = shipped(id);
            for knob in named(&profile.patch) {
                let (group, name) = knob.split_once('.').unwrap();
                if profile.cited(group, name).is_none() {
                    missing.push(format!("{id}: {knob}"));
                }
            }
        }
        assert!(missing.is_empty(), "uncited: {missing:#?}");
    }

    /// The layer explain names is the last of root, base and profile
    /// whose patch names the knob, then the context's patch: for every
    /// knob of every shipped profile.
    #[test]
    fn the_layer_agrees_with_resolution_for_every_knob_of_every_shipped_profile() {
        let mut context = SettingsPatch::default();
        context.frame.node = Some(Node::True);
        for id in SHIPPED_PROFILES {
            let profile = shipped(id);
            let base = profile.base.as_ref().map(|b| shipped(b.as_str()));
            let own = named(&profile.patch);
            let inherited = base.as_ref().map(|b| named(&b.patch)).unwrap_or_default();
            for (patch, by_context) in [(SettingsPatch::default(), false), (context.clone(), true)]
            {
                let resolved = profile.resolve(&patch).unwrap();
                let explained = resolved
                    .explained(&SettingsQuery::default(), &|key| key.to_owned())
                    .unwrap();
                assert_eq!(explained.knobs.len(), Settings::knobs().len());
                for row in &explained.knobs {
                    let (expected, by) = if by_context && row.knob == "frame.node" {
                        (Layer::Context, None)
                    } else if own.contains(&row.knob) {
                        (Layer::Profile, Some(profile.id.clone()))
                    } else if inherited.contains(&row.knob) {
                        (Layer::Profile, base.as_ref().map(|b| b.id.clone()))
                    } else {
                        (Layer::Root, None)
                    };
                    assert_eq!(
                        (row.layer, &row.profile),
                        (expected, &by),
                        "{id} {}",
                        row.knob
                    );
                    // Every knob of a shipped profile is cited; the context's is not.
                    assert_eq!(
                        row.citation.is_some(),
                        expected != Layer::Context,
                        "{id} {}",
                        row.knob
                    );
                    // The note's key: the path with each `_x` written `X`.
                    let camel: String = row
                        .knob
                        .split('_')
                        .enumerate()
                        .map(|(at, part)| {
                            let mut chars = part.chars();
                            match (at, chars.next()) {
                                (0, _) | (_, None) => part.to_owned(),
                                (_, Some(first)) => {
                                    first.to_ascii_uppercase().to_string() + chars.as_str()
                                }
                            }
                        })
                        .collect();
                    assert_eq!(row.effect, format!("sdk.settings.effect.{camel}"));
                }
            }
        }
    }

    #[test]
    fn a_request_patch_is_its_own_layer_over_the_context() {
        let mut context = SettingsPatch::default();
        context.frame.node = Some(Node::True);
        let resolved = shipped("kp-default").resolve(&context).unwrap();
        let mut request = SettingsPatch::default();
        request.frame.node = Some(Node::Mean);
        let derived = resolved.derived(&request).unwrap();
        assert_eq!(derived.settings.frame.node, Node::Mean);
        assert_eq!(derived.origins.of("frame", "node").layer, Layer::Request);
        assert_eq!(
            derived.origins.of("frame", "ayanamsha").layer,
            Layer::Profile
        );
        // The same merge as a bare value's, so the same settings.
        assert_eq!(
            derived.settings,
            resolved
                .settings
                .clone()
                .derived(&request, resolved.profile.clone())
                .unwrap()
                .settings
        );
        // A document's knobs are its own layer.
        let document = Settings::from_json(&resolved.settings.canonical_json()).unwrap();
        assert_eq!(document.origins.of("frame", "node").layer, Layer::Document);
        assert!(document.origins.of("frame", "node").source.is_none());
    }

    #[test]
    fn diffing_the_committee_and_drik_names_every_knob_they_part_on() {
        let committee = shipped("nepali-committee")
            .resolve(&SettingsPatch::default())
            .unwrap();
        let drik = shipped("nepali-drik")
            .resolve(&SettingsPatch::default())
            .unwrap();
        let diff = committee
            .diff(&drik, &SettingsQuery::default(), &|k| k.to_owned())
            .unwrap();
        let knobs: Vec<&str> = diff.knobs.iter().map(|k| k.knob.as_str()).collect();
        assert_eq!(
            knobs,
            [
                "frame.ayanamsha",
                "frame.centre",
                "frame.siddhanta",
                "panchanga.lagna_table"
            ]
        );
        assert!(diff.knobs.iter().take(3).all(|k| k.moves == Moves::Sky));
        // A value is never different from itself, and the diff is symmetric.
        assert_eq!(
            committee
                .diff(&committee, &SettingsQuery::default(), &|k| k.to_owned())
                .unwrap()
                .knobs
                .len(),
            0
        );
        let back = drik
            .diff(&committee, &SettingsQuery::default(), &|k| k.to_owned())
            .unwrap();
        assert_eq!(back.knobs.len(), diff.knobs.len());
        assert_eq!(back.knobs[0].value, diff.knobs[0].other_value);
        // A selection narrows it.
        let frame = SettingsQuery {
            knobs: Some(vec![String::from("frame.siddhanta")]),
        };
        assert_eq!(
            committee
                .diff(&drik, &frame, &|k| k.to_owned())
                .unwrap()
                .knobs
                .len(),
            1
        );
    }

    #[test]
    fn every_knob_declares_what_it_moves() {
        for (group, names, moves) in super::super::groups() {
            assert_eq!(names.len(), moves.len(), "{group}");
        }
    }
}
