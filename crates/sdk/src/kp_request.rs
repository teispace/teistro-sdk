//! KP through the façade: a chart read as the KP Readers read it
//! (`03-design/kp.md`).

use teistro_chart::ChartZodiac;
use teistro_core::catalogue::Ayanamsha;
use teistro_core::error::Error;
use teistro_core::settings::AyanamshaChoice;
use teistro_core::time::UtcOffset;

/// The ayanamshas a KP reading takes without being told otherwise:
/// Krishnamurti's own, and the VP291 variant of it.
pub const KP_AYANAMSHAS: [Ayanamsha; 2] = [Ayanamsha::Krishnamurti, Ayanamsha::KrishnamurtiVp291];

/// How to read a chart as KP.
///
/// A KP reading refuses a chart founded under any ayanamsha but
/// [`KP_AYANAMSHAS`] (crux C157): every sub lord moves with the zodiac,
/// and reading KP under Lahiri's is a common mistake made without
/// knowing. A caller who means it says so.
///
/// ```
/// use teistro::KpRequest;
///
/// assert!(!KpRequest::new().takes_any_ayanamsha());
/// assert!(KpRequest::new().under_any_ayanamsha().takes_any_ayanamsha());
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KpRequest {
    any_ayanamsha: bool,
    clock: Option<UtcOffset>,
}

impl KpRequest {
    /// A reading under the KP ayanamshas alone.
    #[must_use]
    pub const fn new() -> KpRequest {
        KpRequest {
            any_ayanamsha: false,
            clock: None,
        }
    }

    /// The same reading, taking a chart founded under any zodiac.
    #[must_use]
    pub const fn under_any_ayanamsha(mut self) -> KpRequest {
        self.any_ayanamsha = true;
        self
    }

    /// The same reading, with the clock the moment was judged on: what a
    /// civil day lord (`kp.day_lord_day = CIVIL`) takes its weekday from.
    #[must_use]
    pub const fn on_clock(mut self, clock: UtcOffset) -> KpRequest {
        self.clock = Some(clock);
        self
    }

    /// The clock the moment was judged on, when named.
    #[must_use]
    pub const fn clock(&self) -> Option<UtcOffset> {
        self.clock
    }

    /// Whether a chart under any zodiac is read.
    #[must_use]
    pub const fn takes_any_ayanamsha(&self) -> bool {
        self.any_ayanamsha
    }

    /// Refuses a chart in a zodiac the reading does not take.
    pub(crate) fn check(self, zodiac: &ChartZodiac) -> Result<(), Error> {
        if self.any_ayanamsha {
            return Ok(());
        }
        let founded = match zodiac.ayanamsha {
            Some(AyanamshaChoice::Catalogued { id }) if KP_AYANAMSHAS.contains(&id) => {
                return Ok(());
            }
            Some(AyanamshaChoice::Catalogued { id }) => id.key().to_owned(),
            Some(_) => String::from("a custom ayanamsha"),
            None => String::from("the tropical zodiac"),
        };
        Err(Error::invalid_arg(format!(
            "a KP reading takes the KRISHNAMURTI or KRISHNAMURTI_VP291 ayanamsha, and this \
             chart was founded under {founded}"
        ))
        .with_field("frame.ayanamsha")
        .with_hint(
            "found the chart under the kp-default profile, or ask \
             KpRequest::new().under_any_ayanamsha() to read it as it is",
        ))
    }
}
