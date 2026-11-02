//! A settings snapshot (`03-design/receipts-and-datasets.md` §3): the
//! profile, the whole canonical settings document and its hash, kept
//! beside something computed so it can be computed again under the very
//! settings it was. A receipt carries one, and so does every study of a
//! research dataset.

use serde::{Deserialize, Serialize};

use super::{ProfileId, Resolved, SCHEMA, Settings};
use crate::envelope::Hash;
use crate::error::{Error, Status};

/// The settings something was computed under, whole.
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
    /// writes it. Its schema is the settings document's (the MCP server's
    /// `settings.describe`), stated there rather than copied here.
    #[cfg_attr(
        feature = "schema",
        schemars(with = "serde_json::Map<String, serde_json::Value>")
    )]
    pub settings: Settings,
    /// The SHA-256 of the settings' canonical JSON: the `settings_hash`
    /// a provenance carries.
    pub hash: Hash,
}

impl SettingsSnapshot {
    /// The snapshot of resolved settings.
    #[must_use]
    pub fn of(resolved: &Resolved) -> SettingsSnapshot {
        SettingsSnapshot {
            profile: resolved.profile.as_str().to_owned(),
            settings: resolved.settings.clone(),
            hash: resolved.settings.hash(),
        }
    }

    /// Whether `hash` is the settings' own: `false` for a document edited
    /// after the snapshot was taken.
    ///
    /// ```
    /// use teistro_core::settings::{Node, SettingsSnapshot, root};
    /// use teistro_core::settings::ProfileId;
    ///
    /// let resolved = root().validated(ProfileId::new("mine")).unwrap();
    /// let mut snapshot = SettingsSnapshot::of(&resolved);
    /// assert!(snapshot.holds());
    /// snapshot.settings.frame.node = if snapshot.settings.frame.node == Node::True { Node::Mean } else { Node::True };
    /// assert!(!snapshot.holds());
    /// ```
    #[must_use]
    pub fn holds(&self) -> bool {
        self.settings.hash() == self.hash
    }

    /// The settings resolved again: held to their hash and their schema,
    /// and checked for coherence, under the snapshot's profile.
    ///
    /// ```
    /// use teistro_core::settings::{Node, ProfileId, SettingsSnapshot, root};
    ///
    /// let mut snapshot = SettingsSnapshot::of(&root().validated(ProfileId::new("mine")).unwrap());
    /// snapshot.settings.frame.node = if snapshot.settings.frame.node == Node::True { Node::Mean } else { Node::True };
    /// assert_eq!(snapshot.resolved().unwrap_err().field(), Some("hash"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` at `hash` when the hash is not the settings' own;
    /// `SCHEMA_VERSION` at `settings.schema` for a document of another
    /// schema; the coherence errors, named under `settings`.
    pub fn resolved(&self) -> Result<Resolved, Error> {
        if self.settings.schema != SCHEMA {
            return Err(Error::new(
                Status::SchemaVersion,
                format!(
                    "the snapshot's settings are schema {}, and this build reads {SCHEMA}",
                    self.settings.schema
                ),
            )
            .with_field("settings.schema"));
        }
        if !self.holds() {
            return Err(Error::invalid_arg(format!(
                "the snapshot's hash is {} and its settings hash to {}: the settings were \
                 changed after the snapshot was taken",
                self.hash,
                self.settings.hash()
            ))
            .with_field("hash"));
        }
        self.settings
            .clone()
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

    #[test]
    fn another_schema_is_refused_by_its_field() {
        let mut older = snapshot();
        older.settings.schema = SCHEMA + 1;
        let refused = older.resolved().unwrap_err();
        assert_eq!(refused.status, Status::SchemaVersion);
        assert_eq!(refused.field(), Some("settings.schema"));
    }

    #[test]
    fn settings_changed_under_the_hash_are_refused_at_it() {
        let mut changed = snapshot();
        changed.settings.time.delta_t =
            if changed.settings.time.delta_t == crate::settings::DeltaT::EspenakMeeus2006 {
                crate::settings::DeltaT::TableThenModel
            } else {
                crate::settings::DeltaT::EspenakMeeus2006
            };
        assert!(!changed.holds());
        let refused = changed.resolved().unwrap_err();
        assert_eq!(refused.status, Status::InvalidArg);
        assert_eq!(refused.field(), Some("hash"));
    }

    #[test]
    fn incoherent_settings_are_refused_under_settings() {
        let mut settings = root();
        settings.day.sunrise = SunriseConvention::Custom { altitude_deg: 91.0 };
        let incoherent = SettingsSnapshot {
            profile: String::from("mine"),
            hash: settings.hash(),
            settings,
        };
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
}
