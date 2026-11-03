//! A settings snapshot (`03-design/receipts-and-datasets.md` §3): the
//! profile, the whole canonical settings document and its hash, kept
//! beside something computed so it can be computed again under the very
//! settings it was. A receipt carries one, and so does every study of a
//! research dataset.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::{ProfileId, Resolved, SCHEMA, Settings};
use crate::envelope::{Hash, content_hash};
use crate::error::{Error, Status};

/// The settings something was computed under, whole.
///
/// The document is kept as it was carried, not as this build would write
/// it: its hash is taken over the document's own canonical JSON, so a
/// snapshot written by a build with a knob this one lacks still holds to
/// its hash here, and is refused only where it is read as settings.
///
/// ```
/// use teistro_core::settings::{DEFAULT_PROFILE, Profile, SettingsPatch, SettingsSnapshot};
///
/// let resolved = Profile::shipped(DEFAULT_PROFILE).unwrap().resolve(&SettingsPatch::default())?;
/// let snapshot = SettingsSnapshot::of(&resolved);
/// assert_eq!(snapshot.hash, resolved.settings.hash());
/// // Read back, it resolves to the same settings under the same profile.
/// let again = snapshot.resolved()?;
/// assert_eq!((again.settings, again.profile), (resolved.settings, resolved.profile));
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct SettingsSnapshot {
    /// The profile the settings were resolved from.
    pub profile: String,
    /// The settings document, every knob, as `Context::settings_json`
    /// writes it, and kept as it was read. Its schema is the settings
    /// document's (the MCP server's `settings.describe`), stated there
    /// rather than copied here.
    pub settings: Map<String, Value>,
    /// The SHA-256 of the document's canonical JSON: the `settings_hash`
    /// a provenance carries.
    pub hash: Hash,
}

impl SettingsSnapshot {
    /// The snapshot of `settings` under `profile`.
    ///
    /// ```
    /// use teistro_core::settings::{SettingsSnapshot, root};
    ///
    /// let snapshot = SettingsSnapshot::new("mine", &root());
    /// assert_eq!(snapshot.hash, root().hash());
    /// assert_eq!(snapshot.settings()?, root());
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
    #[must_use]
    pub fn new(profile: &str, settings: &Settings) -> SettingsSnapshot {
        // The document read back from the canonical text the settings'
        // own hash is taken over, so the two hashes are one.
        let settings: Map<String, Value> =
            serde_json::from_str(&settings.canonical_json()).unwrap_or_default();
        SettingsSnapshot {
            profile: profile.to_owned(),
            hash: content_hash(&settings),
            settings,
        }
    }

    /// The snapshot of resolved settings.
    #[must_use]
    pub fn of(resolved: &Resolved) -> SettingsSnapshot {
        SettingsSnapshot::new(resolved.profile.as_str(), &resolved.settings)
    }

    /// Whether `hash` is the document's own: `false` for a document
    /// edited after the snapshot was taken. A key this build does not
    /// know is part of the document and holds to the hash.
    ///
    /// ```
    /// use teistro_core::envelope::content_hash;
    /// use teistro_core::settings::{SettingsSnapshot, root};
    ///
    /// let mut snapshot = SettingsSnapshot::new("mine", &root());
    /// assert!(snapshot.holds());
    /// snapshot.settings.insert("futureKnob".into(), true.into());
    /// assert!(!snapshot.holds());
    /// // Hashed again, as a later build that has the knob writes it.
    /// snapshot.hash = content_hash(&snapshot.settings);
    /// assert!(snapshot.holds());
    /// ```
    #[must_use]
    pub fn holds(&self) -> bool {
        content_hash(&self.settings) == self.hash
    }

    /// The document read as this build's settings, refused by the field
    /// under `settings`.
    ///
    /// ```
    /// use teistro_core::settings::{SettingsSnapshot, root};
    ///
    /// let mut snapshot = SettingsSnapshot::new("mine", &root());
    /// snapshot.settings.insert("futureKnob".into(), true.into());
    /// assert_eq!(snapshot.settings().unwrap_err().field(), Some("settings.futureKnob"));
    /// ```
    ///
    /// # Errors
    ///
    /// `SCHEMA_VERSION` at `settings.schema` for a document of another
    /// schema; `INVALID_ARG` at the field for a document this build does
    /// not read: a knob it does not have, or one it has and the document
    /// lacks.
    pub fn settings(&self) -> Result<Settings, Error> {
        // The schema first: a document of another schema is not read as
        // this one and refused for a knob it never had.
        if let Some(schema) = self
            .settings
            .get("schema")
            .and_then(Value::as_u64)
            .filter(|schema| *schema != u64::from(SCHEMA))
        {
            return Err(Error::new(
                Status::SchemaVersion,
                format!(
                    "the snapshot's settings are schema {schema}, and this build reads {SCHEMA}"
                ),
            )
            .with_field("settings.schema"));
        }
        crate::strict::read_value(&Value::Object(self.settings.clone()), "settings")
    }

    /// The settings resolved again: held to their hash, read by this
    /// build's schema, and checked for coherence, under the snapshot's
    /// profile.
    ///
    /// ```
    /// use teistro_core::settings::{SettingsSnapshot, root};
    ///
    /// let mut snapshot = SettingsSnapshot::new("mine", &root());
    /// snapshot.settings.insert("futureKnob".into(), true.into());
    /// assert_eq!(snapshot.resolved().unwrap_err().field(), Some("hash"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` at `hash` when the hash is not the document's own;
    /// what [`SettingsSnapshot::settings`] refuses; the coherence errors,
    /// named under `settings`.
    pub fn resolved(&self) -> Result<Resolved, Error> {
        if !self.holds() {
            return Err(Error::invalid_arg(format!(
                "the snapshot's hash is {} and its settings hash to {}: the settings were \
                 changed after the snapshot was taken",
                self.hash,
                content_hash(&self.settings)
            ))
            .with_field("hash"));
        }
        self.settings()?
            .validated(ProfileId::new(&self.profile))
            .map_err(|diagnostics| Error::from(diagnostics).under("settings"))
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;
    use crate::settings::{SunriseConvention, root};

    fn snapshot() -> SettingsSnapshot {
        SettingsSnapshot::of(&root().validated(ProfileId::new("mine")).unwrap())
    }

    /// `snapshot` with its document edited, and hashed again.
    fn rehashed(edit: impl FnOnce(&mut Map<String, Value>)) -> SettingsSnapshot {
        let mut snapshot = snapshot();
        edit(&mut snapshot.settings);
        snapshot.hash = content_hash(&snapshot.settings);
        snapshot
    }

    #[test]
    fn another_schema_is_refused_by_its_field() {
        let older = rehashed(|settings| {
            settings.insert("schema".into(), (SCHEMA + 1).into());
        });
        let refused = older.resolved().unwrap_err();
        assert_eq!(refused.status, Status::SchemaVersion);
        assert_eq!(refused.field(), Some("settings.schema"));
    }

    #[test]
    fn settings_changed_under_the_hash_are_refused_at_it() {
        let mut changed = snapshot();
        changed.settings["time"]["delta_t"] = "ESPENAK_MEEUS_2006_SENTINEL".into();
        assert!(!changed.holds());
        let refused = changed.resolved().unwrap_err();
        assert_eq!(refused.status, Status::InvalidArg);
        assert_eq!(refused.field(), Some("hash"));
    }

    #[test]
    fn incoherent_settings_are_refused_under_settings() {
        let mut settings = root();
        settings.day.sunrise = SunriseConvention::Custom { altitude_deg: 91.0 };
        let incoherent = SettingsSnapshot::new("mine", &settings);
        assert_eq!(
            incoherent.resolved().unwrap_err().field(),
            Some("settings.day.sunrise")
        );
    }

    #[test]
    fn a_snapshot_reads_back_from_its_canonical_json() {
        let written = crate::envelope::canonical_json(&snapshot());
        let read: SettingsSnapshot = crate::strict::read(&written, "snapshot").unwrap();
        assert_eq!(read, snapshot());
        assert!(read.resolved().is_ok());
    }

    /// A later build's knob, or one this build has and an earlier build's
    /// document lacks, is no broken hash: the document holds and is
    /// refused where it is read, by the knob.
    #[test]
    fn a_knob_this_build_does_not_know_is_refused_by_its_name() {
        let later = rehashed(|settings| {
            settings["frame"]["futureKnob"] = true.into();
        });
        assert!(later.holds());
        let refused = later.resolved().unwrap_err();
        assert_eq!(refused.status, Status::InvalidArg);
        assert_eq!(refused.field(), Some("settings.frame.futureKnob"));
        let earlier = rehashed(|settings| {
            settings.remove("time");
        });
        assert!(earlier.holds());
        // A missing knob is its record's fault, and the message names it.
        let missing = earlier.resolved().unwrap_err();
        assert_eq!(missing.field(), Some("settings"));
        assert!(missing.message.contains("`time`"), "{}", missing.message);
    }

    #[test]
    fn a_document_keeps_its_own_spelling_through_canonical_json() {
        let later = rehashed(|settings| {
            settings.insert(
                "futureKnob".into(),
                serde_json::json!({"z": 1.5, "a": [1, 2]}),
            );
        });
        let written = crate::envelope::canonical_json(&later);
        let read: SettingsSnapshot = crate::strict::read(&written, "snapshot").unwrap();
        assert_eq!(read, later);
        assert!(read.holds());
    }
}
