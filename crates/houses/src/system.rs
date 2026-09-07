//! Which house system a module uses here — the settings question, in
//! one place.
//!
//! `houses.placement_system` answers "which house is it in" for most of
//! a chart, `houses.chalit_system` answers it for the chalit, and
//! `houses.module_overrides` overrides either for a named module, which
//! is how the KP reading takes Placidus while the rest of the chart is
//! whole-sign.
//!
//! **Nothing in the SDK read that last knob** until this module. It is
//! the same shape of gap that made a chart founded on the SDK's own
//! default profile fail when `state` was built — a knob shipped, cited
//! and resolved by the settings layer with no module on the other end of
//! it (registry entry 23) — and it is why this is one function rather
//! than every module reading the settings for itself and two of them
//! disagreeing about which system a chart is in
//! (`03-design/houses-measured.md` §5).
//!
//! ```
//! use teistro_core::catalogue::HouseSystem;
//! use teistro_core::settings::{Profile, SettingsPatch};
//! use teistro_houses::system::{Purpose, system_for};
//!
//! let settings = Profile::shipped(teistro_core::settings::DEFAULT_PROFILE)
//!     .expect("the default profile")
//!     .resolve(&SettingsPatch::default())
//!     .expect("it resolves")
//!     .settings;
//! // The placement system, and the chalit, which the default profile
//! // patches to Sripati.
//! assert_eq!(system_for(&settings, Purpose::Chalit, None), HouseSystem::Sripati);
//! // A module with no override takes the same answer.
//! assert_eq!(
//!     system_for(&settings, Purpose::Chalit, Some("jaimini")),
//!     system_for(&settings, Purpose::Chalit, None)
//! );
//! ```

use serde::Serialize;
use teistro_core::catalogue::HouseSystem;
use teistro_core::settings::Settings;

/// What a house system is being asked for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Purpose {
    /// "Which house is this body in", which most of a chart asks.
    Placement,
    /// The bhava chalit, which is a different question and often a
    /// different answer.
    Chalit,
}

/// The house system a module uses for a purpose.
///
/// A module's override wins over the purpose's own system; a module with
/// no override, or no module at all, takes the purpose's.
#[must_use]
pub fn system_for(settings: &Settings, purpose: Purpose, module: Option<&str>) -> HouseSystem {
    if let Some(override_for) = module.and_then(|module| override_of(settings, module)) {
        return override_for;
    }
    match purpose {
        Purpose::Placement => settings.houses.placement_system,
        Purpose::Chalit => settings.houses.chalit_system,
    }
}

/// The system a module's own override names, if it has one.
///
/// The name is matched without regard to case, because a settings
/// document is written by hand and `KP` and `kp` are the same module.
#[must_use]
pub fn override_of(settings: &Settings, module: &str) -> Option<HouseSystem> {
    settings
        .houses
        .module_overrides
        .iter()
        .find(|(named, _)| named.eq_ignore_ascii_case(module))
        .map(|(_, system)| *system)
}

/// Every module the settings name an override for, in the settings'
/// own order.
pub fn overridden(settings: &Settings) -> impl Iterator<Item = (&str, HouseSystem)> {
    settings
        .houses
        .module_overrides
        .iter()
        .map(|(module, system)| (module.as_str(), *system))
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "tests fail by panicking"
    )]

    use super::{Purpose, overridden, override_of, system_for};
    use teistro_core::catalogue::HouseSystem;
    use teistro_core::settings::{Profile, SettingsPatch};

    fn settings() -> teistro_core::settings::Settings {
        Profile::shipped(teistro_core::settings::DEFAULT_PROFILE)
            .expect("the default profile")
            .resolve(&SettingsPatch::default())
            .expect("it resolves")
            .settings
    }

    #[test]
    fn a_purpose_takes_its_own_system() {
        let settings = settings();
        assert_eq!(
            system_for(&settings, Purpose::Placement, None),
            settings.houses.placement_system
        );
        assert_eq!(
            system_for(&settings, Purpose::Chalit, None),
            settings.houses.chalit_system
        );
    }

    #[test]
    fn a_module_override_wins_over_the_purpose() {
        let mut settings = settings();
        settings
            .houses
            .module_overrides
            .insert(String::from("kp"), HouseSystem::Placidus);
        // Both purposes yield to the override, because the override is
        // about the module and not about the question.
        for purpose in [Purpose::Placement, Purpose::Chalit] {
            assert_eq!(
                system_for(&settings, purpose, Some("kp")),
                HouseSystem::Placidus,
                "{purpose:?}"
            );
            // And a module without one is unaffected.
            assert_eq!(
                system_for(&settings, purpose, Some("jaimini")),
                system_for(&settings, purpose, None),
                "{purpose:?}"
            );
        }
    }

    #[test]
    fn a_module_name_is_matched_whatever_its_case() {
        let mut settings = settings();
        settings
            .houses
            .module_overrides
            .insert(String::from("kp"), HouseSystem::Placidus);
        for spelling in ["kp", "KP", "Kp"] {
            assert_eq!(
                override_of(&settings, spelling),
                Some(HouseSystem::Placidus),
                "{spelling}"
            );
        }
        assert_eq!(override_of(&settings, "tajika"), None);
        assert_eq!(overridden(&settings).count(), 1);
        assert_eq!(
            overridden(&settings).next(),
            Some(("kp", HouseSystem::Placidus))
        );
    }

    #[test]
    fn the_shipped_profiles_all_answer() {
        // Every profile the SDK ships resolves to a system for both
        // purposes: the failure `state` met, tested for here.
        for profile in teistro_core::settings::SHIPPED_PROFILES {
            let settings = Profile::shipped(profile)
                .expect("a shipped profile")
                .resolve(&SettingsPatch::default())
                .expect("it resolves")
                .settings;
            for purpose in [Purpose::Placement, Purpose::Chalit] {
                let system = system_for(&settings, purpose, None);
                assert!(
                    HouseSystem::ALL.contains(&system),
                    "{profile} {purpose:?}: {system:?}"
                );
            }
        }
    }
}
