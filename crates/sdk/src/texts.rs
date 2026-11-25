//! The texts a build may leave out of its bundles (core#241,
//! `03-design/wasm-profiles.md`, "Text switches"): each is a feature of
//! this crate, and `build.rs` leaves its words out of every bundle when
//! the feature, or the family that says it, is off. A request for a text
//! the build leaves out is refused here, by the feature's name, rather
//! than answered with keys no bundle words.

use teistro_core::error::Error;

/// A text a build may leave out, one per
/// [`teistro_intl::pack::TEXT_SWITCHES`] entry, in its order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Text {
    /// A festival's description (SR-153): `about` in a pack request.
    FestivalAbout,
    /// Why a muhurta window is good (SR-154): `reasons` in a muhurta
    /// request.
    MuhurtaReasons,
    /// The rashifal's reading voice (SR-152): `readings` in a rashifal
    /// batch.
    RashifalReading,
}

impl Text {
    /// Every text, in the switches' order.
    #[cfg(test)]
    pub(crate) const ALL: [Text; 3] = [
        Text::FestivalAbout,
        Text::MuhurtaReasons,
        Text::RashifalReading,
    ];

    /// The feature that keeps it.
    pub(crate) const fn feature(self) -> &'static str {
        match self {
            Text::FestivalAbout => "festival-about",
            Text::MuhurtaReasons => "muhurta-reasons",
            Text::RashifalReading => "rashifal-reading",
        }
    }

    /// Whether this build keeps it.
    pub(crate) const fn kept(self) -> bool {
        match self {
            Text::FestivalAbout => cfg!(feature = "festival-about"),
            Text::MuhurtaReasons => cfg!(feature = "muhurta-reasons"),
            Text::RashifalReading => cfg!(feature = "rashifal-reading"),
        }
    }

    /// Refuses `field`, the request's ask for this text, when the build
    /// leaves the text out.
    ///
    /// # Errors
    ///
    /// `UNSUPPORTED` naming the feature, under `field`.
    pub(crate) fn require(self, field: &str) -> Result<(), Error> {
        if self.kept() {
            return Ok(());
        }
        Err(Error::unsupported(format!(
            "this build leaves out the `{feature}` text: build with the feature `{feature}`, or with `texts` or `full`",
            feature = self.feature()
        ))
        .with_field(field))
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::Text;
    use teistro_intl::pack::TEXT_SWITCHES;

    /// One text per switch, in its order, named as its feature is: the
    /// build script reads the switches, this module the features.
    #[test]
    fn every_switch_is_a_text_refused_by_its_name() {
        let named: Vec<&str> = Text::ALL.iter().map(|text| text.feature()).collect();
        let switched: Vec<&str> = TEXT_SWITCHES.iter().map(|switch| switch.feature).collect();
        assert_eq!(named, switched);
        let manifest = include_str!("../Cargo.toml");
        for text in Text::ALL {
            assert!(
                manifest.contains(&format!("\n{} = []", text.feature())),
                "{} is not a feature",
                text.feature()
            );
            assert_eq!(text.require("x").is_ok(), text.kept());
        }
    }
}
