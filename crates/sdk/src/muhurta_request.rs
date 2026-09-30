//! What a muhurta search is asked (`03-design/muhurta-at-the-boundary.md`
//! §3): the activity's rules, whose day it is read against, the order of
//! the answer, how much of it, and how asta is seen.
//!
//! The range, the place and the clock are the almanac's, which
//! [`AlmanacArea::muhurta`](crate::AlmanacArea::muhurta) takes beside this;
//! the record a binding writes as `muhurta` carries none of them, so the
//! days and their windows cannot be asked of two different places.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use teistro_astro::visibility::Criterion;
use teistro_calendar::CalendarDate;
use teistro_core::error::Error;
use teistro_muhurta::{ActivityRules, Native, Ranking, Request};

/// An activity the SDK ships the rules of, which a request may name
/// rather than spell out.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Activity {
    /// A marriage by Raman's *Muhurtha* ([`ActivityRules::raman_marriage`]).
    RamanMarriage,
    /// A marriage by the baseline engine's gates and weights
    /// ([`ActivityRules::baseline_marriage`]), which the `BASELINE`
    /// ranking reads.
    BaselineMarriage,
}

impl Activity {
    /// Every activity the SDK ships.
    pub const ALL: [Activity; 2] = [Activity::RamanMarriage, Activity::BaselineMarriage];

    /// Its key, as serde writes it and a request names it.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Activity::RamanMarriage => "RAMAN_MARRIAGE",
            Activity::BaselineMarriage => "BASELINE_MARRIAGE",
        }
    }

    /// Its rules.
    #[must_use]
    pub fn rules(self) -> ActivityRules {
        match self {
            Activity::RamanMarriage => ActivityRules::raman_marriage(),
            Activity::BaselineMarriage => ActivityRules::baseline_marriage(),
        }
    }
}

/// A visibility criterion a request may name rather than spell out.
const ASTA: [(&str, Criterion); 3] = [
    ("SURYA_SIDDHANTA", Criterion::SURYA_SIDDHANTA),
    ("COMBUSTION_ORB", Criterion::COMBUSTION_ORB),
    ("PTOLEMY", Criterion::PTOLEMY),
];

/// What a muhurta search is asked.
///
/// ```
/// use teistro::MuhurtaRequest;
/// use teistro::muhurta::{ActivityRules, Ranking};
///
/// // Raman's marriage rules, the texts' order, a week of windows.
/// let asked = MuhurtaRequest::new(ActivityRules::raman_marriage());
/// assert_eq!((asked.ranking(), asked.days_with_windows(), asked.most()), (Ranking::Texts, 7, 50));
///
/// // The baseline engine's, as a binding writes it.
/// let written = MuhurtaRequest::from_json(r#"{"rules": "BASELINE_MARRIAGE", "ranking": "BASELINE", "most": 10}"#)?;
/// assert_eq!(written, MuhurtaRequest::new(ActivityRules::baseline_marriage()).ranked(Ranking::Baseline).at_most(10));
///
/// let wrong = MuhurtaRequest::from_json(r#"{"rules": "RAMAN_MARIAGE"}"#).unwrap_err();
/// assert_eq!(wrong.field(), Some("muhurta.rules"));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MuhurtaRequest {
    rules: ActivityRules,
    native: Option<Native>,
    ranking: Ranking,
    days_with_windows: usize,
    most: usize,
    asta: Criterion,
}

impl MuhurtaRequest {
    /// How many of the best days are cut into windows when a request does
    /// not say: a week.
    pub const DAYS_WITH_WINDOWS: usize = 7;

    /// How many windows an answer holds when a request does not say.
    pub const MOST: usize = 50;

    /// A search under an activity's rules, read against no native, in the
    /// texts' order, cutting the best week of days into at most fifty
    /// windows, with asta seen by the Surya Siddhanta's degrees of time.
    #[must_use]
    pub const fn new(rules: ActivityRules) -> MuhurtaRequest {
        MuhurtaRequest {
            rules,
            native: None,
            ranking: Ranking::Texts,
            days_with_windows: Self::DAYS_WITH_WINDOWS,
            most: Self::MOST,
            asta: Criterion::SURYA_SIDDHANTA,
        }
    }

    /// Read against a native: Tarabala and Chandrabala from their star and
    /// Moon sign, the ashtama lagna from their lagna.
    #[must_use]
    pub const fn with_native(mut self, native: Native) -> MuhurtaRequest {
        self.native = Some(native);
        self
    }

    /// In another order.
    #[must_use]
    pub const fn ranked(mut self, ranking: Ranking) -> MuhurtaRequest {
        self.ranking = ranking;
        self
    }

    /// With more or fewer of the best days cut into windows.
    #[must_use]
    pub const fn with_windows_on(mut self, days: usize) -> MuhurtaRequest {
        self.days_with_windows = days;
        self
    }

    /// With at most `most` windows in the answer.
    #[must_use]
    pub const fn at_most(mut self, most: usize) -> MuhurtaRequest {
        self.most = most;
        self
    }

    /// With asta seen by another criterion (`muhurta-measured.md` §2
    /// measures how far the three shipped ones part).
    #[must_use]
    pub const fn with_asta(mut self, criterion: Criterion) -> MuhurtaRequest {
        self.asta = criterion;
        self
    }

    /// The activity's rules.
    #[must_use]
    pub const fn rules(&self) -> &ActivityRules {
        &self.rules
    }

    /// The native, if any.
    #[must_use]
    pub const fn native(&self) -> Option<&Native> {
        self.native.as_ref()
    }

    /// The order of the answer.
    #[must_use]
    pub const fn ranking(&self) -> Ranking {
        self.ranking
    }

    /// How many of the best days are cut into windows.
    #[must_use]
    pub const fn days_with_windows(&self) -> usize {
        self.days_with_windows
    }

    /// How many windows the answer holds at most.
    #[must_use]
    pub const fn most(&self) -> usize {
        self.most
    }

    /// How asta is seen.
    #[must_use]
    pub const fn asta(&self) -> &Criterion {
        &self.asta
    }

    /// Refuses a search for nothing — no day cut, or no window kept — and
    /// the `BASELINE` ranking over rules that carry no baseline event; a
    /// refusal names the field.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming `daysWithWindows`, `most` or `rules.baseline`.
    pub fn check(&self) -> Result<(), Error> {
        if self.days_with_windows == 0 {
            return Err(Error::invalid_arg(
                "a search that cuts no day into windows answers nothing; ask for one day or more",
            )
            .with_field("daysWithWindows"));
        }
        if self.most == 0 {
            return Err(Error::invalid_arg(
                "a search that keeps no window answers nothing; ask for one window or more",
            )
            .with_field("most"));
        }
        if self.ranking == Ranking::Baseline && self.rules.baseline.is_none() {
            return Err(Error::invalid_arg(
                "the BASELINE ranking reads the rules' baseline event, and these rules have none",
            )
            .with_field("rules.baseline")
            .with_hint("name BASELINE_MARRIAGE, or give the rules a baseline event"));
        }
        Ok(())
    }

    /// The search over a range, as the muhurta crate takes it.
    pub(crate) fn over(&self, from: &CalendarDate, to: &CalendarDate) -> Request {
        Request {
            rules: self.rules.clone(),
            from: from.clone(),
            to: to.clone(),
            native: self.native,
            ranking: self.ranking,
            days_with_windows: self.days_with_windows,
            most: self.most,
        }
    }

    /// The request a binding writes as `muhurta`, read and checked.
    ///
    /// `rules` names an [`Activity`] (`"RAMAN_MARRIAGE"`) or spells an
    /// [`ActivityRules`] out; `asta` names a criterion (`"SURYA_SIDDHANTA"`,
    /// `"COMBUSTION_ORB"`, `"PTOLEMY"`) or spells a [`Criterion`] out.
    /// Every other key is optional. A catalogue member anywhere in it may
    /// be written bare or in full (`nakshatra.ROHINI`), as every binding
    /// reads one back.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming the key under `muhurta` that is not JSON of
    /// its kind, is not a field, or fails [`MuhurtaRequest::check`].
    pub fn from_json(text: &str) -> Result<MuhurtaRequest, Error> {
        let asked: Asked = teistro_core::strict::read(text, MUHURTA)?;
        let rules = match &asked.rules {
            Value::String(name) => {
                named(name, &Activity::ALL.map(|a| (a.key(), a)), "rules")?.rules()
            }
            other => teistro_core::strict::read_value(other, &format!("{MUHURTA}.rules"))?,
        };
        let mut request = MuhurtaRequest::new(rules)
            .ranked(asked.ranking)
            .with_windows_on(asked.days_with_windows)
            .at_most(asked.most);
        if let Some(native) = asked.native {
            request = request.with_native(native);
        }
        if let Some(asta) = &asked.asta {
            request = request.with_asta(match asta {
                Value::String(name) => named(name, &ASTA, "asta")?,
                other => teistro_core::strict::read_value(other, &format!("{MUHURTA}.asta"))?,
            });
        }
        request.check().map_err(|why| why.under(MUHURTA))?;
        Ok(request)
    }
}

/// The record every binding writes the request as.
const MUHURTA: &str = "muhurta";

/// A named member of `names`, or the refusal listing them.
fn named<T: Copy>(name: &str, names: &[(&str, T)], field: &str) -> Result<T, Error> {
    names
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, member)| *member)
        .ok_or_else(|| {
            let known: Vec<&str> = names.iter().map(|(key, _)| *key).collect();
            Error::invalid_arg(format!(
                "`{name}` is not one the SDK names; it names {}, or the value may be spelt out",
                known.join(", ")
            ))
            .with_field(format!("{MUHURTA}.{field}"))
        })
}

/// [`MuhurtaRequest`] as the bindings write it, camel-cased as every
/// request record is. `rules` and `asta` are read in a second step,
/// because each is a name or a value and the refusal should say which
/// field was wrong rather than that no shape matched.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Asked {
    rules: Value,
    #[serde(default)]
    native: Option<Native>,
    #[serde(default)]
    ranking: Ranking,
    #[serde(default = "days_with_windows")]
    days_with_windows: usize,
    #[serde(default = "most")]
    most: usize,
    #[serde(default)]
    asta: Option<Value>,
}

const fn days_with_windows() -> usize {
    MuhurtaRequest::DAYS_WITH_WINDOWS
}

const fn most() -> usize {
    MuhurtaRequest::MOST
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "a test fails by panicking")]

    use super::*;
    use teistro_core::catalogue::{Nakshatra, Rashi};

    fn refused(text: &str) -> Error {
        MuhurtaRequest::from_json(text).unwrap_err()
    }

    #[test]
    fn a_record_reads_every_field_and_defaults_the_rest() {
        let text = r#"{"rules": "RAMAN_MARRIAGE", "native": {"star": "nakshatra.ROHINI", "moonSign": "TAURUS"},
            "ranking": "TEXTS", "daysWithWindows": 3, "most": 9, "asta": "PTOLEMY"}"#;
        let native = Native {
            star: Nakshatra::Rohini,
            moon_sign: Rashi::Taurus,
            lagna: None,
        };
        assert_eq!(
            MuhurtaRequest::from_json(text).unwrap(),
            MuhurtaRequest::new(ActivityRules::raman_marriage())
                .with_native(native)
                .with_windows_on(3)
                .at_most(9)
                .with_asta(Criterion::PTOLEMY)
        );
        assert_eq!(
            MuhurtaRequest::from_json(r#"{"rules": "RAMAN_MARRIAGE"}"#).unwrap(),
            MuhurtaRequest::new(ActivityRules::raman_marriage())
        );
    }

    #[test]
    fn rules_and_asta_spelt_out_read_as_their_values() {
        let own = serde_json::to_string(&ActivityRules::baseline_marriage()).unwrap();
        let asta = serde_json::to_string(&Criterion::COMBUSTION_ORB).unwrap();
        let text = format!(r#"{{"rules": {own}, "asta": {asta}, "ranking": "BASELINE"}}"#);
        assert_eq!(
            MuhurtaRequest::from_json(&text).unwrap(),
            MuhurtaRequest::new(ActivityRules::baseline_marriage())
                .ranked(Ranking::Baseline)
                .with_asta(Criterion::COMBUSTION_ORB)
        );
    }

    #[test]
    fn a_refusal_names_the_field_it_is_about() {
        for (text, field) in [
            (r#"{"rules": "RAMAN_MARIAGE"}"#, "muhurta.rules"),
            (
                r#"{"rules": "RAMAN_MARRIAGE", "asta": "PTOLMY"}"#,
                "muhurta.asta",
            ),
            (r#"{"rules": "RAMAN_MARRIAGE", "most": 0}"#, "muhurta.most"),
            (
                r#"{"rules": "RAMAN_MARRIAGE", "daysWithWindows": 0}"#,
                "muhurta.daysWithWindows",
            ),
            (
                r#"{"rules": "RAMAN_MARRIAGE", "ranking": "BASELINE"}"#,
                "muhurta.rules.baseline",
            ),
            (
                r#"{"rules": "RAMAN_MARRIAGE", "mostt": 3}"#,
                "muhurta.mostt",
            ),
            (
                r#"{"rules": "RAMAN_MARRIAGE", "native": {"star": "ROHINI", "moonSign": "TAURUS", "lagnaa": "LEO"}}"#,
                "muhurta.native.lagnaa",
            ),
            (r#"{"rules": {"day": 1}}"#, "muhurta.rules.day"),
            (r#"{"most": 3}"#, "muhurta"),
        ] {
            assert_eq!(refused(text).field(), Some(field), "{text}");
        }
        // A member of the wrong kind is refused even where its key is a
        // member of the right one: `JYESHTHA` is a masa and a star.
        let masa = refused(
            r#"{"rules": "RAMAN_MARRIAGE", "native": {"star": "masa.JYESHTHA", "moonSign": "TAURUS"}}"#,
        );
        assert_eq!(masa.field(), Some("muhurta.native.star"));
    }
}
