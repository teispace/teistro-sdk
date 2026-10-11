//! Two founded charts matched through the Moon of each, and Mars in
//! each (`03-design/matching.md`).

use serde::{Deserialize, Serialize};
use teistro_chart::foundation::ChartFoundation;
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;
use teistro_matching::{
    AshtaKoota, Avakahada, KootaRules, Kuja, KujaNative, KujaRules, MarriageDosha, MatchRole,
    Native, Porutham, PoruthamRules, ashta_koota, avakahada, kuja, marriage_doshas, porutham,
};
use teistro_serial::Document;

use crate::area::ChartArea;
use crate::partner::Partner;
use crate::reading::ChartRequest;

/// The record's name where a binding sends it, which a refusal is named
/// under.
const MATCHING: &str = "matching";

/// A match against a partner's birth, as a binding asks it: the partner,
/// the side the partner stands on, every chart of the batch on the other,
/// and the rules of each system.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
    /// The readings the ten considerations are computed under; the
    /// chapter's own when left out.
    #[serde(default)]
    pub porutham: PoruthamRules,
    /// The readings the Kuja dosha is computed under; the verse's own
    /// when left out.
    #[serde(default)]
    pub kuja: KujaRules,
}

/// One chart matched with a partner under both systems.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Matched {
    /// The North's eight kootas, out of 36.
    pub ashta_koota: AshtaKoota,
    /// The South's ten considerations.
    pub porutham: Porutham,
    /// Mars in each chart (*Manasagari*, jāyābhāva v. 4).
    pub kuja: Kuja,
}

impl Matched {
    /// Every marriage dosha the three readings report, as one list in
    /// their own order, each with whether it is lifted. Nothing is judged
    /// anew and no severity is given (`03-design/matching.md`, C289,
    /// C290).
    #[must_use]
    pub fn doshas(&self) -> Vec<MarriageDosha> {
        marriage_doshas(&self.ashta_koota, &self.porutham, &self.kuja)
    }
}

impl PartnerMatching {
    /// The record a binding sends, as JSON: `partner`, `{"instant": jd,
    /// "place": {"latitude", "longitude", "altitude"}, "utcOffsetSeconds"}`,
    /// `partnerRole`, `"BRIDE"` or `"GROOM"`, `rules`, the [`KootaRules`]
    /// with every field optional, `porutham`, the [`PoruthamRules`]
    /// likewise, and `kuja`, the [`KujaRules`] likewise.
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
            native(bride, MatchRole::Bride.name())?,
            native(groom, MatchRole::Groom.name())?,
            rules,
        ))
    }

    /// The **ten considerations** of a bride's chart and a groom's
    /// (*Kalaprakasika* XIII): whether each agrees and what it read, how
    /// many agree, how many of the chief five, and the p. 76 exception's
    /// clauses. Never a verdict: the chapter's "at least five" is the
    /// reader's to apply (`03-design/matching.md`).
    ///
    /// ```no_run
    /// # use teistro::{Context, Document, Ephemeris, PoruthamRules};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let (bride, groom): (Document, Document) = todo!();
    /// let ten = sdk.chart().porutham(&bride, &groom, PoruthamRules::default())?;
    /// for row in &ten.considerations {
    ///     println!("{:?}: agrees {}", row.reading, row.agrees);
    /// }
    /// println!("{} of 10 agree", ten.agreeing);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// What [`ChartArea::matching`] refuses.
    pub fn porutham(
        self,
        bride: &Document,
        groom: &Document,
        rules: PoruthamRules,
    ) -> Result<Porutham, Error> {
        Ok(porutham(
            native(bride, MatchRole::Bride.name())?,
            native(groom, MatchRole::Groom.name())?,
            rules,
        ))
    }

    /// The **Kuja dosha** of a bride's chart and a groom's (*Manasagari*,
    /// jāyābhāva v. 4): Mars's house from the lagna, the Moon and Venus in
    /// each, whether each carries the dosha under the rules and whether
    /// both do. Nothing is lifted: no verse read lifts it, so the popular
    /// cancellation is the reader's to apply (`03-design/matching.md`).
    ///
    /// ```no_run
    /// # use teistro::{Context, Document, Ephemeris, KujaRules};
    /// # fn main() -> Result<(), teistro::Error> {
    /// # let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// # let (bride, groom): (Document, Document) = todo!();
    /// let mars = sdk.chart().kuja(&bride, &groom, KujaRules::default())?;
    /// println!("bride {}, groom {}, both {}", mars.bride.dosha, mars.groom.dosha, mars.both);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// A chart founded in the tropical zodiac, or one that does not place
    /// the Moon, Venus or Mars, named `bride` or `groom`.
    pub fn kuja(self, bride: &Document, groom: &Document, rules: KujaRules) -> Result<Kuja, Error> {
        Ok(kuja(
            kuja_native(bride, MatchRole::Bride.name())?,
            kuja_native(groom, MatchRole::Groom.name())?,
            rules,
        ))
    }

    /// Each chart matched with a partner's birth under both systems and
    /// the Kuja dosha ([`ChartArea::matching`], [`ChartArea::porutham`]
    /// and [`ChartArea::kuja`]), one
    /// [`Matched`] a chart in the order given: the partner on
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
    ) -> Result<Vec<Matched>, Error> {
        let Partner {
            instant,
            place,
            utc_offset,
        } = asked.partner;
        let partner = self
            .reading(instant, &ChartRequest::at(place, utc_offset))
            .map_err(|why| why.with_field("partner"))?
            .value;
        let theirs = natives(&partner, "partner")?;
        let role = asked.partner_role.other();
        charts
            .iter()
            .enumerate()
            .map(|(at, chart)| {
                let ours = natives(chart, role.name())
                    .map_err(|why| why.with_hint(format!("chart {at}")))?;
                let (bride, groom) = match role {
                    MatchRole::Bride => (ours, theirs),
                    MatchRole::Groom => (theirs, ours),
                };
                Ok(Matched {
                    ashta_koota: ashta_koota(bride.0, groom.0, asked.rules),
                    porutham: porutham(bride.0, groom.0, asked.porutham),
                    kuja: kuja(bride.1, groom.1, asked.kuja),
                })
            })
            .collect()
    }
}

/// A chart's foundation, refused by the role it was given unless it is
/// sidereal.
fn sidereal<'a>(chart: &'a Document, role: &str) -> Result<&'a ChartFoundation, Error> {
    let foundation = &chart.foundation;
    if foundation.zodiac.is_sidereal() {
        Ok(foundation)
    } else {
        Err(Error::invalid_arg(format!(
            "the {role}'s chart is founded in the tropical zodiac, where a nakshatra means nothing"
        ))
        .with_field(role)
        .with_hint("found it under a sidereal profile, such as the default"))
    }
}

/// A graha's longitude in a chart, refused by the role it was given when
/// the chart does not place it.
fn longitude(foundation: &ChartFoundation, graha: Graha, role: &str) -> Result<f64, Error> {
    foundation
        .graha(graha)
        .map(|position| position.longitude_deg)
        .ok_or_else(|| {
            Error::invalid_arg(format!("the {role}'s chart does not place {graha:?}"))
                .with_field(role)
        })
}

/// A chart's avakahada: its Moon read as one native (C301), refused on a
/// tropical chart, where a nakshatra means nothing.
pub(crate) fn avakahada_of(foundation: &ChartFoundation) -> Result<Avakahada, Error> {
    const FIELD: &str = "avakahada";
    if !foundation.zodiac.is_sidereal() {
        return Err(Error::invalid_arg(
            "the chart is founded in the tropical zodiac, where a nakshatra means nothing",
        )
        .with_field(FIELD)
        .with_hint("found it under a sidereal profile, such as the default"));
    }
    let moon = longitude(foundation, Graha::Moon, FIELD)?;
    Native::of_moon(moon)
        .and_then(avakahada)
        .map_err(|error| error.under(FIELD))
}

/// A chart's Moon as matching reads it, refused by the role it was given.
fn native(chart: &Document, role: &str) -> Result<Native, Error> {
    let moon = longitude(sidereal(chart, role)?, Graha::Moon, role)?;
    Native::of_moon(moon).map_err(|error| error.with_field(role))
}

/// A chart's lagna, Moon, Venus and Mars as the Kuja dosha reads them.
fn kuja_native(chart: &Document, role: &str) -> Result<KujaNative, Error> {
    let foundation = sidereal(chart, role)?;
    KujaNative::of_longitudes(
        foundation.lagna_deg,
        longitude(foundation, Graha::Moon, role)?,
        longitude(foundation, Graha::Venus, role)?,
        longitude(foundation, Graha::Mars, role)?,
    )
    .map_err(|error| error.with_field(role))
}

/// Everything a match reads of one chart.
fn natives(chart: &Document, role: &str) -> Result<(Native, KujaNative), Error> {
    Ok((native(chart, role)?, kuja_native(chart, role)?))
}
