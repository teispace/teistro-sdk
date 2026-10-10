//! KP through the façade: a chart read as the KP Readers read it
//! (`03-design/kp.md`).

use serde::{Deserialize, Serialize};
use teistro_chart::ChartZodiac;
use teistro_core::catalogue::Ayanamsha;
use teistro_core::error::Error;
use teistro_core::settings::AyanamshaChoice;
use teistro_core::time::UtcOffset;
use teistro_kp::KpNumber;

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
    number: Option<KpNumber>,
}

impl KpRequest {
    /// A reading under the KP ayanamshas alone.
    #[must_use]
    pub const fn new() -> KpRequest {
        KpRequest {
            any_ayanamsha: false,
            clock: None,
            number: None,
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

    /// The same reading as a **horary** chart for the querent's number
    /// (crux C156): the lagna at the number's start, the other cusps
    /// those that ascendant has at the chart's place. The ruling planets
    /// stay the moment's own.
    #[must_use]
    pub const fn for_number(mut self, number: KpNumber) -> KpRequest {
        self.number = Some(number);
        self
    }

    /// The horary number, when one was named.
    #[must_use]
    pub const fn number(&self) -> Option<KpNumber> {
        self.number
    }

    /// The request as the bindings write it: `{"number": 74, "clock":
    /// 19800, "anyAyanamsha": true}`, every member optional, the clock in
    /// seconds east of UT.
    ///
    /// ```
    /// use teistro::KpRequest;
    ///
    /// let request = KpRequest::from_json(r#"{"number": 74, "clock": 19800}"#)?;
    /// assert_eq!(request.number().map(|n| n.get()), Some(74));
    /// assert_eq!(request.clock().map(|c| c.seconds()), Some(19_800));
    /// assert_eq!(KpRequest::from_json("{}")?, KpRequest::new());
    /// let refused = KpRequest::from_json(r#"{"number": 250}"#).unwrap_err();
    /// assert_eq!(refused.field(), Some("kp.number"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Text that is not the record, a member it does not know, a number
    /// outside 1 to 249 or a clock outside a day, each named under `kp`.
    pub fn from_json(text: &str) -> Result<KpRequest, Error> {
        let asked: Asked = teistro_core::strict::read(text, KP)?;
        let mut request = KpRequest::new();
        if asked.any_ayanamsha {
            request = request.under_any_ayanamsha();
        }
        if let Some(seconds) = asked.clock {
            let clock = UtcOffset::try_from_seconds(seconds)
                .map_err(|why| Error::from(why).with_field(format!("{KP}.clock")))?;
            request = request.on_clock(clock);
        }
        if let Some(number) = asked.number {
            request = request.for_number(KpNumber::new(number).map_err(|why| why.under(KP))?);
        }
        Ok(request)
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
            "found the chart under the kp-default profile, or read it as it is: \
             `\"anyAyanamsha\": true` in the `kp` record, \
             `KpRequest::new().under_any_ayanamsha()` in Rust",
        ))
    }
}

/// The record every binding writes the request as.
const KP: &str = "kp";

/// [`KpRequest`] as the bindings write it, camel-cased as every request
/// record is; every member optional.
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub(crate) struct Asked {
    #[serde(default)]
    number: Option<u16>,
    #[serde(default)]
    clock: Option<i32>,
    #[serde(default)]
    any_ayanamsha: bool,
}
