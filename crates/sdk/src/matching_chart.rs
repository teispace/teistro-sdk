//! Two founded charts matched through the Moon of each
//! (`03-design/matching.md`).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;
use teistro_matching::{AshtaKoota, KootaRules, Native, ashta_koota};
use teistro_serial::Document;

use crate::area::ChartArea;
use crate::reading::ChartRequest;
use crate::western_aspects::Partner;

/// The record's name where a binding sends it, which a refusal is named
/// under.
const MATCHING: &str = "matching";

/// Which side of a match a birth stands on. Varna and Gana read
/// differently when the two swap, so a match names them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MatchRole {
    /// The bride's birth.
    Bride,
    /// The groom's birth.
    Groom,
}

impl MatchRole {
    /// The other side.
    #[must_use]
    pub const fn other(self) -> MatchRole {
        match self {
            MatchRole::Bride => MatchRole::Groom,
            MatchRole::Groom => MatchRole::Bride,
        }
    }

    /// The role's name, which a refusal is named by.
    const fn field(self) -> &'static str {
        match self {
            MatchRole::Bride => "bride",
            MatchRole::Groom => "groom",
        }
    }
}

/// A match against a partner's birth, as a binding asks it: the partner,
/// the side the partner stands on, every chart of the batch on the other,
/// and the rules.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartnerMatching {
    /// Whose birth every chart is matched with.
    pub partner: Partner,
    /// The side the partner stands on; every chart stands on the other.
    pub partner_role: MatchRole,
    /// The readings the kootas are computed under; the sources' own when
    /// left out.
    #[serde(default)]
    pub rules: KootaRules,
}

impl PartnerMatching {
    /// The record a binding sends, as JSON: `partner`, `{"instant": jd,
    /// "place": {"latitude", "longitude", "altitude"}, "utcOffsetSeconds"}`,
    /// `partnerRole`, `"BRIDE"` or `"GROOM"`, and `rules`, the
    /// [`KootaRules`] with every field optional.
    ///
    /// ```
    /// use teistro::{MatchRole, PartnerMatching};
    /// use teistro::matching::NadiDosha;
    ///
    /// let asked = PartnerMatching::from_json(
    ///     r#"{"partner": {"instant": 2447892.5, "place": {"latitude": 27.7, "longitude": 85.3, "altitude": 0}}, "partnerRole": "BRIDE", "rules": {"nadiDosha": "MIDDLE_ONLY"}}"#,
    /// )?;
    /// assert_eq!(asked.partner_role, MatchRole::Bride);
    /// assert_eq!(asked.rules.nadi_dosha, NadiDosha::MiddleOnly);
    /// // The side is never assumed.
    /// let unsided = PartnerMatching::from_json(
    ///     r#"{"partner": {"instant": 2447892.5, "place": {"latitude": 27.7, "longitude": 85.3, "altitude": 0}}}"#,
    /// )
    /// .unwrap_err();
    /// assert_eq!(unsided.field(), Some("matching"));
    /// let typo = PartnerMatching::from_json(
    ///     r#"{"partner": {"instant": 2447892.5, "place": {"latitude": 27.7, "longitude": 85.3, "altitude": 0}}, "partnerRole": "GROOM", "rules": {"nadi": "ANY"}}"#,
    /// )
    /// .unwrap_err();
    /// assert_eq!(typo.field(), Some("matching.rules.nadi"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read and a value out of its bounds, each named under `matching`.
    pub fn from_json(text: &str) -> Result<PartnerMatching, Error> {
        teistro_core::strict::read(text, MATCHING)
    }
}

impl ChartArea<'_> {
    /// The **Ashta Koota** of a bride's chart and a groom's (*Muhurta
    /// Chintamani* VI.21–34): each koota's points out of 36 and what it
    /// read, Bhakoot's dosha with its five exceptions as clauses, and
    /// Nadi's dosha. Never a verdict: the texts leave the judgement to the
    /// reader (`03-design/matching.md`).
    ///
    /// Each native is the chart's Moon, in its own sidereal zodiac. The
    /// bride and the groom are named because Varna and Gana read
    /// differently when they swap.
    ///
    /// ```no_run
    /// # use teistro::{Context, Document, Ephemeris, KootaRules};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let (bride, groom): (Document, Document) = todo!();
    /// let koota = sdk.chart().matching(&bride, &groom, KootaRules::default())?;
    /// for row in &koota.kootas {
    ///     println!("{:?}: {} of {}", row.reading, row.points, row.max_points);
    /// }
    /// println!("{} of 36", koota.total);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// A chart founded in the tropical zodiac, where a nakshatra means
    /// nothing, or one that does not place the Moon, named `bride` or
    /// `groom`.
    pub fn matching(
        self,
        bride: &Document,
        groom: &Document,
        rules: KootaRules,
    ) -> Result<AshtaKoota, Error> {
        Ok(ashta_koota(
            native(bride, MatchRole::Bride.field())?,
            native(groom, MatchRole::Groom.field())?,
            rules,
        ))
    }

    /// Each chart matched with a partner's birth ([`ChartArea::matching`]),
    /// one [`AshtaKoota`] a chart in the order given: the partner on
    /// [`PartnerMatching::partner_role`]'s side, every chart on the other.
    /// The partner is founded once, under this context's profile, which
    /// must be sidereal.
    ///
    /// # Errors
    ///
    /// A partner that cannot be founded or whose chart is tropical, named
    /// `partner`, and what [`ChartArea::matching`] refuses of a chart,
    /// hinted with its place in the list.
    pub fn matching_with(
        self,
        charts: &[Document],
        asked: &PartnerMatching,
    ) -> Result<Vec<AshtaKoota>, Error> {
        let Partner {
            instant,
            place,
            utc_offset,
        } = asked.partner;
        let partner = self
            .reading(instant, &ChartRequest::at(place, utc_offset))
            .map_err(|why| why.with_field("partner"))?
            .value;
        let theirs = native(&partner, "partner")?;
        let role = asked.partner_role.other();
        charts
            .iter()
            .enumerate()
            .map(|(at, chart)| {
                let ours = native(chart, role.field())
                    .map_err(|why| why.with_hint(format!("chart {at}")))?;
                Ok(match role {
                    MatchRole::Bride => ashta_koota(ours, theirs, asked.rules),
                    MatchRole::Groom => ashta_koota(theirs, ours, asked.rules),
                })
            })
            .collect()
    }
}

/// A chart's Moon as matching reads it, refused by the role it was given.
fn native(chart: &Document, role: &str) -> Result<Native, Error> {
    let foundation = &chart.foundation;
    if !foundation.zodiac.is_sidereal() {
        return Err(Error::invalid_arg(format!(
            "the {role}'s chart is founded in the tropical zodiac, where a nakshatra means nothing"
        ))
        .with_field(role)
        .with_hint("found it under a sidereal profile, such as the default"));
    }
    let moon = foundation.graha(Graha::Moon).ok_or_else(|| {
        Error::invalid_arg(format!("the {role}'s chart does not place the Moon")).with_field(role)
    })?;
    Native::of_moon(moon.longitude_deg).map_err(|error| error.with_field(role))
}
