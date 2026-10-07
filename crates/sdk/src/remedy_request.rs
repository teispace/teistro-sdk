//! Remedies through the façade (`03-design/remedies.md`): every step read
//! off a chart, the running Vimśottarī daśā at an instant the caller names.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{DashaSystem, Graha, Rashi};
use teistro_core::error::Error;
use teistro_core::quantity::{Depth, JulianDay, Utc};
use teistro_dasha::Timeline;
use teistro_remedies::{
    NINE, Remedies, RemedyRules, RemedySky, Running, functional, shanti, subjects,
};
use teistro_serial::Document;
use teistro_state::state;

use crate::ChartArea;

/// The record's name where a binding sends it, which a refusal is named
/// under.
const REMEDIES: &str = "remedies";

/// What a chart's remedies are read under: the instant whose running daśā
/// names subjects, when one is asked about, and the readings.
///
/// ```
/// use teistro::RemedyRequest;
/// use teistro::remedies::{RikSource, SunWithKetu};
///
/// let asked = RemedyRequest::from_json(
///     r#"{"at": 2460676.5, "rules": {"shanti": {"rik": "YAJNAVALKYA"}, "devata": {"sunWithKetu": "SURYA"}}}"#,
/// )?;
/// assert_eq!(asked.at, Some(2_460_676.5));
/// assert_eq!(asked.rules.shanti.rik, RikSource::Yajnavalkya);
/// assert_eq!(asked.rules.devata.sun_with_ketu, SunWithKetu::Surya);
/// let typo = RemedyRequest::from_json(r#"{"rules": {"devatas": {}}}"#).unwrap_err();
/// assert_eq!(typo.field(), Some("remedies.rules.devatas"));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct RemedyRequest {
    /// The Julian day (UTC) whose running Vimśottarī mahādaśā and
    /// antardaśā name subjects and the antardaśā śānti (BPHS chs. 37–45);
    /// none reads no daśā.
    pub at: Option<f64>,
    /// The readings; the texts' own when left out.
    pub rules: RemedyRules,
}

impl RemedyRequest {
    /// The record a binding sends, as JSON: `at` and `rules`
    /// (`functional`, `shanti`, `devata`), every member optional.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, or a key it does not
    /// read, named under `remedies`.
    pub fn from_json(text: &str) -> Result<RemedyRequest, Error> {
        teistro_core::strict::read(text, REMEDIES)
    }
}

impl ChartArea<'_> {
    /// A chart's remedies (`03-design/remedies.md`): the lagna's functional
    /// natures, whom a remedy is for, each subject's graha-śānti, the
    /// running antardaśā's śānti when `at` is given, and the ishṭa-devatā
    /// in both charts.
    ///
    /// # Errors
    ///
    /// A chart whose states cannot be computed; with `at`, a document
    /// carrying no Vimśottarī daśā (`dashas`) or an instant that is not a
    /// finite number (`remedies.at`).
    pub fn remedies(self, document: &Document, asked: &RemedyRequest) -> Result<Remedies, Error> {
        let rules = asked.rules;
        let foundation = &document.foundation;
        let lagna = Rashi::of_longitude(foundation.lagna_deg);
        let longitude = |graha: Graha| {
            foundation
                .graha(graha)
                .map(|at| at.longitude_deg)
                .ok_or_else(|| Error::internal(format!("a founded chart places {}", graha.key())))
        };
        let mut signs = [lagna; 9];
        for (slot, graha) in signs.iter_mut().zip(NINE) {
            *slot = Rashi::of_longitude(longitude(graha)?);
        }
        let states = state(foundation, self.context().settings())?;
        let combust = NINE.map(|graha| {
            states
                .iter()
                .any(|one| one.graha == graha && one.combustion.is_combust())
        });
        let moon_waxing =
            (longitude(Graha::Moon)? - longitude(Graha::Sun)?).rem_euclid(360.0) < 180.0;
        let running = asked
            .at
            .map(|at| self.running_at(document, at))
            .transpose()?
            .flatten();
        let sky = RemedySky {
            lagna,
            signs,
            combust,
            moon_waxing,
            running,
        };
        let natures = functional(lagna, rules.functional);
        let subjects = subjects(&sky, &natures);
        let shantis = subjects
            .subjects
            .iter()
            .filter_map(|subject| shanti(subject.graha, rules.shanti))
            .collect();
        Ok(Remedies {
            rules,
            functional: natures,
            subjects,
            shantis,
            ishta_devata: self.ishta_devata(document, rules.devata)?,
        })
    }

    /// The Vimśottarī mahādaśā and antardaśā running at `at`; none before
    /// birth or past the cycle's end.
    fn running_at(self, document: &Document, at: f64) -> Result<Option<Running>, Error> {
        if !at.is_finite() {
            return Err(
                Error::invalid_arg(format!("the instant {at} is not a number"))
                    .with_field("remedies.at"),
            );
        }
        let vimshottari = teistro_dasha::DashaName::from(DashaSystem::Vimshottari);
        if !document
            .dashas
            .iter()
            .any(|reading| reading.system == vimshottari)
        {
            return Err(Error::invalid_arg(
                "the document carries no Vimshottari dasha, which the running antardasha is read \
                 from; ask for it with `ChartRequest::with_dashas`",
            )
            .with_field("dashas"));
        }
        let depth = Depth::try_new(2).map_err(|_| Error::internal("two levels are a depth"))?;
        let chain = self
            .dasha(document, DashaSystem::Vimshottari)?
            .at(JulianDay::<Utc>::literal(at), depth);
        let mut lords = chain.iter().map(|period| period.lord);
        Ok(match (lords.next(), lords.next()) {
            (Some(mahadasha), Some(antardasha)) => Some(Running {
                mahadasha,
                antardasha,
            }),
            _ => None,
        })
    }
}
