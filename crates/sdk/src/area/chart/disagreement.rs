//! Where two charts disagree, and on what (`03-design/chart-disagreements.md`):
//! one birth under two settings, two births under one, or both. The chart
//! counterpart of the almanac's `disagreements`: the same two contexts,
//! `otherProfile` and `otherSettingsHash` beside the facts, and only the
//! facts that differ, each with both sides.

use serde::Serialize;
use serde_json::Value;
use teistro_core::catalogue::{DashaSystem, Graha, Kind, Nakshatra, Rashi, write_in_full};
use teistro_core::envelope::{Envelope, Hash, content_hash};
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_rules::NetStatus;
use teistro_serial::Document;

use super::ChartArea;
use crate::context::Context;
use crate::{FoundRequest, Present, RulesReading};

/// One placement as one side reads it: the sign, and where in the circle
/// and in the sign it stands, which is the reason when the two signs
/// differ (a lagna at 29.9° of one sign beside 0.2° of the next).
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SignPlacement {
    /// The sign.
    pub sign: Rashi,
    /// The longitude in that side's zodiac, degrees.
    pub longitude_deg: f64,
    /// Degrees into the sign, 0 to 30.
    pub degrees_in_sign: f64,
}

impl SignPlacement {
    fn of(longitude_deg: f64) -> SignPlacement {
        let longitude = longitude_deg.rem_euclid(360.0);
        SignPlacement {
            sign: Rashi::of_longitude(longitude),
            longitude_deg: longitude,
            degrees_in_sign: longitude.rem_euclid(30.0),
        }
    }
}

/// The lagna in two signs.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct LagnaDisagreement {
    /// As the first chart has it.
    pub first: SignPlacement,
    /// As the second has it.
    pub second: SignPlacement,
}

/// A graha in two signs.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GrahaDisagreement {
    /// Which graha.
    pub graha: Graha,
    /// As the first chart places it.
    pub first: SignPlacement,
    /// As the second does.
    pub second: SignPlacement,
}

/// The Moon's star and pada as one side reads it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct MoonStar {
    /// The janma nakshatra.
    pub nakshatra: Nakshatra,
    /// Its pada, 1 to 4.
    pub pada: u8,
    /// The Moon's longitude in that side's zodiac, degrees.
    pub longitude_deg: f64,
}

/// The janma nakshatra or its pada, differing.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct NakshatraDisagreement {
    /// As the first chart has it.
    pub first: MoonStar,
    /// As the second has it.
    pub second: MoonStar,
}

/// The period running at the instant compared, at one level.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct RunningLord {
    /// Its lord.
    pub lord: Graha,
    /// The sign it is the period of, in a sign-based dasha.
    pub sign: Option<Rashi>,
    /// When it runs.
    pub interval: Interval,
}

/// One level of one dasha system whose running period differs.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DashaDisagreement {
    /// The system, by its bare key (`VIMSHOTTARI`, or a registered one's).
    pub system: String,
    /// The level, 1 for the mahadasha.
    pub level: u8,
    /// The first chart's period running then; none when its periods do
    /// not reach the instant.
    pub first: Option<RunningLord>,
    /// The second chart's.
    pub second: Option<RunningLord>,
}

/// One rule, a yoga or a dosha, that holds on one chart and not the
/// other, or holds on both with another net status.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct YogaDisagreement {
    /// The rule's key.
    pub rule: String,
    /// Its category, as its pack names it (`raja`, `arishta`, …).
    pub category: String,
    /// Its status on the first chart after its cancellations; none where
    /// it does not hold.
    pub first: Option<NetStatus>,
    /// The same on the second.
    pub second: Option<NetStatus>,
}

/// The facts two charts disagree on, and what the other context was.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ChartDisagreements {
    /// The profile the other context resolved.
    pub other_profile: String,
    /// The other context's settings hash, so a stored comparison is known
    /// stale when either side's settings change.
    pub other_settings_hash: Hash,
    /// The instant the dasha lords were compared at; absent, each chart's
    /// own birth.
    pub at: Option<JulianDay<Utc>>,
    /// How many facts were compared: the lagna, each graha both place, the
    /// nakshatra, each dasha level both carry and each rule either holds.
    pub compared: usize,
    /// The lagna, when its sign differs.
    pub lagna: Option<LagnaDisagreement>,
    /// Each graha whose sign differs, in the catalogue's order.
    pub grahas: Vec<GrahaDisagreement>,
    /// The Moon's nakshatra, when it or its pada differs.
    pub nakshatra: Option<NakshatraDisagreement>,
    /// Each dasha level whose running lord differs, by system then level.
    pub dashas: Vec<DashaDisagreement>,
    /// Each rule that differs, in the rule set's order, then the ones only
    /// the second chart holds.
    pub yogas: Vec<YogaDisagreement>,
}

/// Where each catalogue member of a [`ChartDisagreements`] sits, written
/// in full at the boundary as every other section's are.
const MEMBERS: [(&str, &str, Kind); 12] = [
    ("lagna", "first.sign", Kind::Rashi),
    ("lagna", "second.sign", Kind::Rashi),
    ("grahas", "graha", Kind::Graha),
    ("grahas", "first.sign", Kind::Rashi),
    ("grahas", "second.sign", Kind::Rashi),
    ("nakshatra", "first.nakshatra", Kind::Nakshatra),
    ("nakshatra", "second.nakshatra", Kind::Nakshatra),
    ("dashas", "first.lord", Kind::Graha),
    ("dashas", "second.lord", Kind::Graha),
    ("dashas", "first.sign", Kind::Rashi),
    ("dashas", "second.sign", Kind::Rashi),
    ("dashas", "system", Kind::DashaSystem),
];

impl ChartDisagreements {
    /// The record as JSON, every catalogue member written as its full key:
    /// what the boundary carries and seals. A registered dasha system's
    /// key is written under its kind as a catalogued one's is.
    ///
    /// # Errors
    ///
    /// `INTERNAL` if the record does not serialise, which one this crate
    /// built cannot do.
    pub fn in_full(&self) -> Result<Value, Error> {
        let mut value = serde_json::to_value(self).map_err(|error| {
            Error::internal(format!(
                "the chart disagreements did not serialise: {error}"
            ))
        })?;
        for (field, path, kind) in MEMBERS {
            if let Some(items) = value.get_mut(field) {
                write_in_full(items, path, kind);
            }
        }
        Ok(value)
    }
}

/// The record's name where a binding sends it, which a refusal is named
/// under.
const DISAGREEMENTS: &str = "disagreements";

/// Two charts compared (`03-design/chart-disagreements.md`): the first
/// under this context, the second under the other, at the second birth or
/// the first's again.
#[derive(Clone, Debug)]
pub struct ChartDisagreementRequest {
    /// The first chart's request.
    pub first: FoundRequest,
    /// The second's; the first's again when left out.
    pub second: Option<FoundRequest>,
    /// The instant the running dasha lords are compared at; each chart's
    /// own birth when left out.
    pub at: Option<JulianDay<Utc>>,
    /// The hash of the record as sent, canonical: the comparison's input.
    input: Hash,
}

impl ChartDisagreementRequest {
    /// The record a binding sends, as JSON: `first`, a chart request of one
    /// birth (`instant`, the place and the clock, and any `dashas`,
    /// `rules` and settings it reads); `second`, another, optional; and
    /// `at`, a UTC Julian day, optional.
    ///
    /// ```
    /// use teistro::ChartDisagreementRequest;
    ///
    /// let asked = ChartDisagreementRequest::from_json(
    ///     r#"{"first": {"instant": 2447000.25, "latitudeDeg": 27.7172, "longitudeDeg": 85.324,
    ///                   "utcOffsetSeconds": 20700, "rules": {"shipped": ["YOGAS"]}},
    ///         "at": 2460000.5}"#,
    /// )?;
    /// assert!(asked.second.is_none() && asked.at.is_some());
    /// let batch = ChartDisagreementRequest::from_json(
    ///     r#"{"first": {"instants": [2447000.25, 2447001.25], "latitudeDeg": 27.7,
    ///                   "longitudeDeg": 85.3, "utcOffsetSeconds": 20700}}"#,
    /// )
    /// .unwrap_err();
    /// assert_eq!(batch.field(), Some("disagreements.first.instants"));
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, a chart request its own reader refuses or one naming more than
    /// one birth, and an `at` that is no instant, each named under
    /// `disagreements`.
    pub fn from_json(text: &str) -> Result<ChartDisagreementRequest, Error> {
        let Value::Object(mut fields) = teistro_core::strict::parse(text, DISAGREEMENTS)? else {
            return Err(Error::invalid_arg("a chart comparison is a JSON object")
                .with_field(DISAGREEMENTS)
                .with_hint(EXAMPLE_HINT));
        };
        let input = content_hash(&Value::Object(fields.clone()));
        let chart = |name: &str, value: Value| -> Result<FoundRequest, Error> {
            let found = FoundRequest::from_json(&value.to_string())
                .map_err(|why| why.under(name).under(DISAGREEMENTS))?;
            if found.instants.len() != 1 {
                return Err(Error::invalid_arg("a comparison reads one birth a side")
                    .with_field(format!("{DISAGREEMENTS}.{name}.instants"))
                    .with_hint("write `instant`, and the other birth as `second`"));
            }
            Ok(found)
        };
        let first = match fields.remove("first") {
            Some(value) => chart("first", value)?,
            None => {
                return Err(Error::invalid_arg("a comparison names its first chart")
                    .with_field(format!("{DISAGREEMENTS}.first"))
                    .with_hint(EXAMPLE_HINT));
            }
        };
        let second = fields
            .remove("second")
            .map(|value| chart("second", value))
            .transpose()?;
        let at = match fields.remove("at") {
            None | Some(Value::Null) => None,
            Some(Value::Number(jd)) => Some(
                jd.as_f64()
                    .ok_or_else(|| Error::invalid_arg("`at` is a UTC Julian day"))
                    .and_then(|jd| JulianDay::<Utc>::try_new(jd).map_err(Error::from))
                    .map_err(|why| why.with_field(format!("{DISAGREEMENTS}.at")))?,
            ),
            Some(_) => {
                return Err(Error::invalid_arg("`at` is a UTC Julian day")
                    .with_field(format!("{DISAGREEMENTS}.at")));
            }
        };
        if let Some(key) = fields.keys().next() {
            return Err(Error::invalid_arg(format!("a comparison has no `{key}`"))
                .with_field(format!("{DISAGREEMENTS}.{key}"))
                .with_hint("it reads `first`, `second` and `at`"));
        }
        Ok(ChartDisagreementRequest {
            first,
            second,
            at,
            input,
        })
    }
}

/// What a refusal of the record's shape suggests.
const EXAMPLE_HINT: &str = "e.g. {\"first\": {\"instant\": 2447000.25, \"latitudeDeg\": 27.7, \"longitudeDeg\": 85.3, \"utcOffsetSeconds\": 20700}}";

/// One side, founded and read: the document, and its rules reading when
/// its request named rules.
struct Side<'a, 'r> {
    document: &'a Document,
    reading: Option<&'a RulesReading<'r>>,
}

impl ChartArea<'_> {
    /// **Where two charts disagree**: the first chart of `asked` founded
    /// under this context, the second (or the first's birth again) under
    /// `other`, and every fact of the two that differs: the lagna's sign,
    /// each graha's sign, the Moon's nakshatra and pada, each dasha
    /// level's running lord at `at` (each birth's own when left out), and
    /// each rule one holds and the other does not
    /// (`03-design/chart-disagreements.md`).
    ///
    /// Pass this context again as `other` to compare two births under one
    /// setting. A side whose request names no dasha system is read under
    /// Vimshottari; the rules are compared when either side's request
    /// names a `rules` record. The answer is sealed by the first chart's
    /// provenance, its input the whole record.
    ///
    /// ```
    /// # #[cfg(feature = "builtin-ephemeris")] {
    /// use teistro::{ChartDisagreementRequest, Context, Ephemeris};
    ///
    /// let classical = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
    /// let kp = Context::builder().profile("kp-default").ephemeris([Ephemeris::Builtin]).build()?;
    /// let asked = ChartDisagreementRequest::from_json(
    ///     r#"{"first": {"instant": 2447000.25, "latitudeDeg": 27.7172, "longitudeDeg": 85.324,
    ///                   "utcOffsetSeconds": 20700}}"#,
    /// )?;
    /// let differ = classical.chart().disagreements(&kp, &asked)?;
    /// // The lagna, nine grahas, the nakshatra and Vimshottari's levels.
    /// assert!(differ.value.compared > 11);
    /// assert_eq!(differ.value.other_profile, "kp-default");
    /// // One context against itself disagrees on nothing.
    /// let same = classical.chart().disagreements(&classical, &asked)?;
    /// assert!(same.value.lagna.is_none() && same.value.grahas.is_empty() && same.value.dashas.is_empty());
    /// # }
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Whatever founding either chart refuses under its context, named
    /// under `disagreements.first` or `disagreements.second`.
    pub fn disagreements(
        self,
        other: &Context,
        asked: &ChartDisagreementRequest,
    ) -> Result<Envelope<ChartDisagreements>, Error> {
        let first = with_a_dasha(asked.first.clone().resolved(self.context.keys()))
            .map_err(under("first"))?;
        let second = with_a_dasha(
            asked
                .second
                .clone()
                .unwrap_or_else(|| asked.first.clone())
                .resolved(other.keys()),
        )
        .map_err(under("second"))?;
        let ours = self
            .compose(&first.instants, &first.request, &first.records)
            .map_err(under("first"))?;
        let theirs = other
            .chart()
            .compose(&second.instants, &second.request, &second.records)
            .map_err(under("second"))?;
        let (Some(a), Some(b)) = (ours.founded.value.first(), theirs.founded.value.first()) else {
            return Err(Error::internal(
                "a comparison founded no chart, which cannot happen",
            ));
        };
        let value = compared(
            &Side {
                document: a,
                reading: ours.readings.first(),
            },
            &Side {
                document: b,
                reading: theirs.readings.first(),
            },
            asked.at,
            other,
        );
        let mut provenance = ours.founded.provenance.clone();
        provenance.input_hash = asked.input;
        Ok(Envelope::sealing(value, provenance))
    }

    /// As [`ChartArea::disagreements`], written in full and sealed as the
    /// boundary carries it.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::disagreements`].
    pub fn disagreements_request(
        self,
        other: &Context,
        asked: &ChartDisagreementRequest,
    ) -> Result<Envelope<Value>, Error> {
        let Envelope { value, provenance } = self.disagreements(other, asked)?;
        Ok(Envelope::sealing(value.in_full()?, provenance))
    }
}

/// A refusal of one side, named under it and the record.
fn under(side: &'static str) -> impl Fn(Error) -> Error {
    move |why| why.under(side).under(DISAGREEMENTS)
}

/// A request with Vimshottari where it named no dasha, so the dasha
/// lords are always among the facts compared.
fn with_a_dasha(found: Result<FoundRequest, Error>) -> Result<FoundRequest, Error> {
    let mut found = found?;
    if found.request.dashas().is_empty() {
        found.request = found.request.with_dashas([DashaSystem::Vimshottari]);
    }
    Ok(found)
}

/// Every fact of two founded charts compared.
fn compared(
    first: &Side<'_, '_>,
    second: &Side<'_, '_>,
    at: Option<JulianDay<Utc>>,
    other: &Context,
) -> ChartDisagreements {
    let (a, b) = (&first.document.foundation, &second.document.foundation);
    let mut compared = 1;
    let lagna = {
        let (x, y) = (
            SignPlacement::of(a.lagna_deg),
            SignPlacement::of(b.lagna_deg),
        );
        (x.sign != y.sign).then_some(LagnaDisagreement {
            first: x,
            second: y,
        })
    };
    let mut grahas = Vec::new();
    for graha in Graha::ALL {
        let (Some(x), Some(y)) = (a.graha(graha), b.graha(graha)) else {
            continue;
        };
        compared += 1;
        let (x, y) = (
            SignPlacement::of(x.longitude_deg),
            SignPlacement::of(y.longitude_deg),
        );
        if x.sign != y.sign {
            grahas.push(GrahaDisagreement {
                graha,
                first: x,
                second: y,
            });
        }
    }
    let star = |foundation: &teistro_chart::foundation::ChartFoundation| {
        let longitude = foundation.graha(Graha::Moon)?.longitude_deg;
        let moon = teistro_matching::Native::of_moon(longitude).ok()?;
        Some(MoonStar {
            nakshatra: moon.nakshatra,
            pada: moon.pada,
            longitude_deg: longitude.rem_euclid(360.0),
        })
    };
    let nakshatra = match (star(a), star(b)) {
        (Some(x), Some(y)) => {
            compared += 1;
            (x.nakshatra != y.nakshatra || x.pada != y.pada).then_some(NakshatraDisagreement {
                first: x,
                second: y,
            })
        }
        _ => None,
    };
    let (dashas, levels) = dasha_lords(first.document, second.document, at);
    compared += levels;
    let (yogas, rules) = yogas(first.reading, second.reading);
    compared += rules;
    ChartDisagreements {
        other_profile: other.profile().to_owned(),
        other_settings_hash: other.settings_hash(),
        at,
        compared,
        lagna,
        grahas,
        nakshatra,
        dashas,
        yogas,
    }
}

/// The period of `level` running at `at` in a document's dasha, if its
/// rows reach it.
fn running(rows: &[teistro_dasha::PeriodRow], level: usize, at: f64) -> Option<RunningLord> {
    rows.iter()
        .find(|row| {
            row.level() == level && row.interval.from.get() <= at && at < row.interval.to.get()
        })
        .map(|row| RunningLord {
            lord: row.lord,
            sign: row.sign,
            interval: row.interval,
        })
}

/// Each dasha level both documents carry whose running period differs,
/// and how many levels were compared.
fn dasha_lords(
    first: &Document,
    second: &Document,
    at: Option<JulianDay<Utc>>,
) -> (Vec<DashaDisagreement>, usize) {
    let mut found = Vec::new();
    let mut levels = 0;
    for ours in &first.dashas {
        let Some(theirs) = second
            .dashas
            .iter()
            .find(|theirs| theirs.system.key() == ours.system.key())
        else {
            continue;
        };
        let deepest = ours.depth.get().min(theirs.depth.get());
        let (x_at, y_at) = match at {
            Some(at) => (at.get(), at.get()),
            None => (
                first.foundation.instant.get(),
                second.foundation.instant.get(),
            ),
        };
        for level in 1..=deepest {
            levels += 1;
            let x = running(&ours.periods, usize::from(level), x_at);
            let y = running(&theirs.periods, usize::from(level), y_at);
            let same = match (&x, &y) {
                (Some(x), Some(y)) => x.lord == y.lord && x.sign == y.sign,
                (None, None) => true,
                _ => false,
            };
            if !same {
                found.push(DashaDisagreement {
                    system: ours.system.key().to_owned(),
                    level,
                    first: x,
                    second: y,
                });
            }
        }
    }
    (found, levels)
}

/// The net status a held rule answers, active where none of its
/// cancellations is stated.
fn status(present: &Present<'_>) -> NetStatus {
    present.result.status.unwrap_or(NetStatus::Active)
}

/// Each rule that holds on one reading and not the other, or with another
/// status, and how many rules either holds.
fn yogas(
    first: Option<&RulesReading<'_>>,
    second: Option<&RulesReading<'_>>,
) -> (Vec<YogaDisagreement>, usize) {
    let none = Vec::new();
    let ours = first.map_or(&none, |reading| &reading.present);
    let theirs = second.map_or(&none, |reading| &reading.present);
    let mut found = Vec::new();
    let mut compared = 0;
    for held in ours {
        compared += 1;
        let other = theirs.iter().find(|one| one.rule.key == held.rule.key);
        let (x, y) = (Some(status(held)), other.map(status));
        if x != y {
            found.push(YogaDisagreement {
                rule: held.rule.key.clone(),
                category: held.rule.category.clone(),
                first: x,
                second: y,
            });
        }
    }
    for held in theirs {
        if ours.iter().all(|one| one.rule.key != held.rule.key) {
            compared += 1;
            found.push(YogaDisagreement {
                rule: held.rule.key.clone(),
                category: held.rule.category.clone(),
                first: None,
                second: Some(status(held)),
            });
        }
    }
    (found, compared)
}

/// The record's schema as a binding writes it: `first` and `second` chart
/// requests, each the record `chart.found` reads, and `at`, with three
/// examples a model can copy.
#[cfg(feature = "schema")]
pub(crate) fn schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    // Named, not inlined: `chart.found`'s request carries every chart
    // record's schema, which twice over would swell a tool list.
    let chart = serde_json::json!({
        "type": "object",
        "description": "A chart request of one birth, as `chart.found`'s `request` reads it (`schema.describe` with {\"tool\": \"chart.found\"} answers its schema): `instant`, `latitudeDeg`, `longitudeDeg`, `altitudeM`, `utcOffsetSeconds`, and any `dashas` and `rules` record compared.",
    });
    schemars::json_schema!({
        "type": "object",
        "description": "Two charts compared: `first` founded under the call's context, `second` (the first's birth again when left out) under the other's, and `at` the UTC Julian day the running dasha lords are compared at (each birth's own when left out).",
        "properties": {
            "first": chart,
            "second": chart,
            "at": { "type": "number", "description": "A UTC Julian day." },
        },
        "required": ["first"],
        "additionalProperties": false,
        "examples": [
            { "first": { "instant": 2_447_000.25, "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20_700 } },
            { "first": { "instant": 2_447_000.25, "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20_700, "rules": { "shipped": ["YOGAS"] } },
              "second": { "instant": 2_447_000.27, "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20_700, "rules": { "shipped": ["YOGAS"] } } },
            { "first": { "instant": 2_447_000.25, "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20_700, "dashas": ["VIMSHOTTARI", "YOGINI"] },
              "at": 2_461_000.5 },
        ],
    })
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    fn jd(at: f64) -> JulianDay<Utc> {
        JulianDay::try_new(at).unwrap()
    }

    fn row(path: &str, lord: Graha, from: f64, to: f64) -> teistro_dasha::PeriodRow {
        teistro_dasha::PeriodRow {
            path: path.to_owned(),
            sign: None,
            lord,
            interval: Interval {
                from: jd(from),
                to: jd(to),
            },
        }
    }

    #[test]
    fn a_placement_reads_its_sign_from_the_circle() {
        let late = SignPlacement::of(29.9);
        let early = SignPlacement::of(390.2);
        assert_eq!((late.sign, early.sign), (Rashi::Aries, Rashi::Taurus));
        assert!((early.longitude_deg - 30.2).abs() < 1e-9);
        assert!((early.degrees_in_sign - 0.2).abs() < 1e-9);
    }

    #[test]
    fn the_running_period_is_the_one_holding_the_instant_at_its_level() {
        let rows = [
            row("0", Graha::Saturn, 0.0, 100.0),
            row("0/0", Graha::Saturn, 0.0, 40.0),
            row("0/1", Graha::Mercury, 40.0, 100.0),
            row("1", Graha::Mercury, 100.0, 200.0),
        ];
        assert_eq!(running(&rows, 1, 50.0).unwrap().lord, Graha::Saturn);
        assert_eq!(running(&rows, 2, 50.0).unwrap().lord, Graha::Mercury);
        // A period ends where the next begins.
        assert_eq!(running(&rows, 1, 100.0).unwrap().lord, Graha::Mercury);
        assert!(running(&rows, 2, 150.0).is_none());
        assert!(running(&rows, 1, 250.0).is_none());
    }

    #[test]
    fn every_member_path_names_a_field_the_record_writes() {
        let fields = ["lagna", "grahas", "nakshatra", "dashas", "yogas"];
        for (field, _, _) in MEMBERS {
            assert!(fields.contains(&field), "{field}");
        }
    }
}
