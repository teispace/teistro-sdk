//! The annual charts a birth is asked for, and what they answer
//! (`03-design/annual-chart.md`).
//!
//! A [`VarshaRequest`] is the record every binding writes as `varsha`: the
//! years wanted, where each year's chart is cast, and which of Tajika's
//! readings to take of it — the Muntha, the office-bearers and the year
//! lord, the sixteen yogas by matter, the sahams with their strength, the
//! Harsha bala and the annual dashas. [`ChartArea::varsha`] answers it
//! for one birth as a [`Varsha`], composed here once so Rust and every
//! binding read the same year the same way; the C boundary only encodes
//! what it answered.
//!
//! [`ChartArea::varsha`]: crate::ChartArea::varsha

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::DashaSystem;
use teistro_core::error::Error;
use teistro_core::house::House;
use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
use teistro_core::time::UtcOffset;
use teistro_tajika::{
    AnnualDasha, AnnualDashaRules, AnnualStates, Bala, Between, Friendship, Harsha, HarshaRules,
    Muntha, MunthaDegree, OfficeBearers, Pravesha, Reading, Saham, SahamNatures, SahamRules,
    SahamStrength, SahamStrengthRules, Varshesha, VarsheshaRules, YearYogas, YogaRules,
};

/// The name every refusal of a [`VarshaRequest`] is rooted at, which is
/// what every binding calls the record: `varsha.through`, `varsha.place`.
pub(crate) const VARSHA: &str = "varsha";

/// The annual charts a birth is asked for.
///
/// A record rather than a handful of arguments, because the annual chart
/// grows: the month and sixty-hour charts are the same search with a step,
/// and a field added to a record is not a change to every call.
///
/// ```
/// use teistro::{Asked, House, Saham, VarshaRequest};
///
/// let request = VarshaRequest::from_json(
///     r#"{"through": 40, "place": "birth", "matters": [7, 10], "sahams": ["PUNYA"]}"#,
/// )?;
/// assert_eq!(request.through, 40);
/// assert_eq!(request.matters, Some(Asked::These(vec![House::ALL[6], House::ALL[9]])));
/// assert_eq!(request.sahams, Some(Asked::These(vec![Saham::Punya])));
///
/// // A refusal names the field the caller wrote, under `varsha`.
/// let wrong = VarshaRequest::from_json(r#"{"through": 0}"#).unwrap_err();
/// assert_eq!(wrong.field(), Some("varsha.through"));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct VarshaRequest {
    /// Which longitude the Sun returns to; the tradition's by default.
    #[serde(default)]
    pub reading: Reading,
    /// The last year of life wanted, 1 to 200.
    pub through: u16,
    /// Where the Muntha stands inside the sign it has reached; the
    /// source's own reading by default (crux C107).
    #[serde(default)]
    pub muntha: MunthaDegree,
    /// Where each year's own chart is cast, when the caller wants the
    /// charts and not only their instants; absent, none is founded and
    /// the answer is the instants and the Muntha.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub place: Option<AnnualPlace>,
    /// The readings the year lord's chain parts on, where authorities
    /// differ; the source's own by default (crux C106).
    #[serde(default)]
    pub varshesha: VarsheshaRules,
    /// The matters each year's sixteen Tajika yogas are judged for;
    /// absent, none is (`03-design/tajika-yogas.md`). Needs `place`,
    /// since the yogas are read from the year's own chart.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matters: Option<Asked<House>>,
    /// The readings the sixteen part on, where the source leaves a
    /// choice; its own by default.
    #[serde(default)]
    pub yogas: YogaRules,
    /// The sahams each chart is read for; absent, none is
    /// (`03-design/tajika-sahams.md`). Without `place` the birth's own are
    /// read and no year's, since a saham is read from a chart.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sahams: Option<Asked<Saham>>,
    /// The readings the sahams part on — when a sign is added, where a
    /// house stands, Roga's formula; the source's own by default.
    #[serde(default)]
    pub saham_rules: SahamRules,
    /// The readings a saham's strength parts on; the chapter's by default.
    #[serde(default)]
    pub saham_strength: SahamStrengthReadings,
    /// The Harsha bala's reading of Venus's house of joy.
    #[serde(default)]
    pub harsha_rules: HarshaRules,
    /// The annual dashas each year is divided by; absent, none is
    /// (`03-design/annual-dashas.md`). Needs `place`, since a year's
    /// dasha opens at its own chart and the Patyayini is read from it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dashas: Option<Asked<DashaSystem>>,
    /// The readings the annual dashas part on — the clock, the balance,
    /// the birth period, the depth; the sources' own by default.
    #[serde(default)]
    pub dasha_rules: AnnualDashaRules,
}

impl VarshaRequest {
    /// The years one to `through`, their instants and Muntha and nothing
    /// else, every reading the source's own.
    #[must_use]
    pub fn through(through: u16) -> VarshaRequest {
        VarshaRequest {
            reading: Reading::default(),
            through,
            muntha: MunthaDegree::default(),
            place: None,
            varshesha: VarsheshaRules::default(),
            matters: None,
            yogas: YogaRules::default(),
            sahams: None,
            saham_rules: SahamRules::default(),
            saham_strength: SahamStrengthReadings::default(),
            harsha_rules: HarshaRules::default(),
            dashas: None,
            dasha_rules: AnnualDashaRules::default(),
        }
    }

    /// The same request, with each year's own chart cast at `place`.
    #[must_use]
    pub fn at(mut self, place: AnnualPlace) -> VarshaRequest {
        self.place = Some(place);
        self
    }

    /// The same request, judging the sixteen yogas for these matters.
    #[must_use]
    pub fn with_matters(mut self, matters: Asked<House>) -> VarshaRequest {
        self.matters = Some(matters);
        self
    }

    /// The same request, reading these sahams.
    #[must_use]
    pub fn with_sahams(mut self, sahams: Asked<Saham>) -> VarshaRequest {
        self.sahams = Some(sahams);
        self
    }

    /// The same request, dividing each year by these annual dashas.
    #[must_use]
    pub fn with_dashas(mut self, dashas: Asked<DashaSystem>) -> VarshaRequest {
        self.dashas = Some(dashas);
        self
    }

    /// The request a binding writes as `varsha`, read and checked; a
    /// refusal names the field the caller wrote, `varsha.through`.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, or a request [`VarshaRequest::check`] refuses.
    pub fn from_json(text: &str) -> Result<VarshaRequest, Error> {
        let mut given: serde_json::Value = teistro_core::strict::read(text, VARSHA)?;
        // The fields whose readers say more than serde's own message are
        // read on their own, under their own path, so a refusal names
        // `varsha.<field>` — the field the caller wrote — where the strict
        // reader, handed the whole record, could only name the record.
        let place = take_field::<AnnualPlace>(&mut given, "place")?;
        let matters = take_field::<Asked<House>>(&mut given, "matters")?;
        let sahams = take_field::<Asked<Saham>>(&mut given, "sahams")?;
        let saham_rules = take_field::<SahamRules>(&mut given, "sahamRules")?;
        let dashas = take_field::<Asked<DashaSystem>>(&mut given, "dashas")?;
        let dasha_rules = take_field::<AnnualDashaRules>(&mut given, "dashaRules")?;
        let mut asked: VarshaRequest = teistro_core::strict::read_value(&given, VARSHA)?;
        asked.place = place;
        asked.matters = matters;
        asked.sahams = sahams;
        asked.saham_rules = saham_rules.unwrap_or_default();
        asked.dashas = dashas;
        asked.dasha_rules = dasha_rules.unwrap_or_default();
        asked.check()?;
        Ok(asked)
    }

    /// The request checked whole, before any year is searched for; a
    /// refusal names the field the caller wrote, `varsha.dashas`.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on a `through` outside one to two hundred; on matters
    /// or annual dashas asked for without a `place`, since both are read
    /// from each year's own chart; on a year clock or yoga reading that
    /// cannot be one.
    pub fn check(&self) -> Result<(), Error> {
        // The matters and the annual dashas are read from each year's own
        // chart, so they need a place, and a request for either without one
        // is refused by the field that asked. The sahams do not, since a
        // birth chart holds sahams of its own and without a place those are
        // what is answered.
        if self.place.is_none() {
            let needs_a_chart = [
                ("matters", self.matters.is_some(), "the sixteen yogas are"),
                ("dashas", self.dashas.is_some(), "an annual dasha is"),
            ];
            if let Some((field, _, what)) = needs_a_chart.iter().find(|(_, asked, _)| *asked) {
                return Err(Error::invalid_arg(format!(
                    "{what} read from each year's own chart, and no chart is founded without a place"
                ))
                .with_field(format!("{VARSHA}.{field}"))
                .with_hint(format!("add {VARSHA}.place: \"birth\", or a residence")));
            }
        }
        self.dasha_rules
            .clock
            .check()
            .map_err(|error| error.with_field(format!("{VARSHA}.dashaRules.clock")))?;
        // Named in the caller's own casing rather than the Rust field
        // behind it.
        self.yogas.check().map_err(|error| {
            let field = if error.field() == Some("strong_from") {
                "yogas.strongFrom"
            } else {
                "yogas"
            };
            error.with_field(format!("{VARSHA}.{field}"))
        })?;
        teistro_tajika::years(self.through).map_err(|error| error.under(VARSHA))?;
        Ok(())
    }

    /// Every reading a saham's strength is judged under, assembled from
    /// the request's three records.
    #[must_use]
    pub fn strength_rules(&self) -> SahamStrengthRules {
        SahamStrengthRules {
            sahams: self.saham_rules,
            harsha: self.harsha_rules,
            natures: self.saham_strength.natures,
            friendship: self.saham_strength.friendship,
            weak_below: self.saham_strength.weak_below,
        }
    }
}

/// One field of the varsha record read on its own, under its own path, and
/// removed from `given` so the record's own read does not see it.
fn take_field<T: Serialize + serde::de::DeserializeOwned>(
    given: &mut serde_json::Value,
    field: &str,
) -> Result<Option<T>, Error> {
    given
        .as_object_mut()
        .and_then(|fields| fields.remove(field))
        .map(|value| teistro_core::strict::read_value::<T>(&value, &format!("{VARSHA}.{field}")))
        .transpose()
}

/// `varsha.sahamStrength`: the readings a saham's strength parts on that
/// are its own, the sahams' and the Harsha bala's having records of their
/// own beside it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct SahamStrengthReadings {
    /// Which planets are benefic and malefic.
    pub natures: SahamNatures,
    /// Whose friendship "friend" and "inimical" read.
    pub friendship: Friendship,
    /// The Vishwa bala below which a saham's lord is weak, sub-sub units.
    pub weak_below: Bala,
}

impl Default for SahamStrengthReadings {
    fn default() -> SahamStrengthReadings {
        let rules = SahamStrengthRules::default();
        SahamStrengthReadings {
            natures: rules.natures,
            friendship: rules.friendship,
            weak_below: rules.weak_below,
        }
    }
}

/// What a request asks about: `"all"`, or these by name in the caller's
/// own order — the matters the sixteen yogas are judged for, the sahams a
/// chart is read for, the annual dashas a year is divided by.
///
/// `"all"` is a word the caller writes and not a default, because every
/// member a year is a cost a caller asking about one did not ask to pay.
/// A member named twice is refused, because it is a mistake and never a
/// request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Asked<T> {
    /// Every member, in the catalogue's order.
    All,
    /// These, in this order.
    These(Vec<T>),
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for teistro_core::house::House {}
    impl Sealed for teistro_tajika::Saham {}
    impl Sealed for teistro_core::catalogue::DashaSystem {}
}

/// A member a request may ask about by name, and how the wire names it:
/// a house, a saham, an annual dasha.
pub trait Askable: sealed::Sealed + Copy + PartialEq + 'static {
    /// How the wire writes one: a house's number, a saham's key.
    type Wire: serde::de::DeserializeOwned + Serialize;
    /// Every member, in the order `"all"` answers them.
    const ALL: &'static [Self];
    /// The field's two shapes, as a refusal says them.
    const SHAPES: &'static str;
    /// The field's name, as a refusal says it.
    const FIELD: &'static str;
    /// What a refusal calls one before the wire's own spelling of it.
    const NOUN: &'static str;
    /// The member the wire named, or why it names none.
    ///
    /// # Errors
    ///
    /// Why the wire's value names no member.
    fn read(wire: Self::Wire) -> Result<Self, String>;
    /// The member as the wire writes it.
    fn wire(self) -> Self::Wire;
}

impl Askable for House {
    type Wire = u8;
    const ALL: &'static [Self] = &House::ALL;
    const SHAPES: &'static str = "\"all\", or a list of house numbers 1 to 12";
    const FIELD: &'static str = "matters";
    const NOUN: &'static str = "house ";

    fn read(number: u8) -> Result<Self, String> {
        House::try_new(number).map_err(|why| why.message)
    }

    fn wire(self) -> u8 {
        self.get()
    }
}

impl Askable for Saham {
    type Wire = Saham;
    const ALL: &'static [Self] = &Saham::ALL;
    const SHAPES: &'static str = "\"all\", or a list of saham keys such as \"PUNYA\"";
    const FIELD: &'static str = "sahams";
    const NOUN: &'static str = "saham ";

    fn read(saham: Saham) -> Result<Self, String> {
        Ok(saham)
    }

    fn wire(self) -> Saham {
        self
    }
}

/// An annual dasha is named by its catalogue key, bare (`"MUDDA"`) or
/// full (`"dasha_system.MUDDA"`): the full key is what every binding reads
/// a system back as, so a caller can hand back what it was given, and the
/// bare one is what the Rust key and the natal settings spell.
impl Askable for DashaSystem {
    type Wire = String;
    const ALL: &'static [Self] = &teistro_tajika::ANNUAL_DASHAS;
    const SHAPES: &'static str = "\"all\", or a list of annual dasha keys: \"dasha_system.PATYAYINI\", \"dasha_system.MUDDA\", \"dasha_system.VARSHA_YOGINI\", with or without the kind";
    const FIELD: &'static str = "dashas";
    const NOUN: &'static str = "dasha ";

    fn read(key: String) -> Result<Self, String> {
        let bare = key
            .strip_prefix(Self::KIND.name())
            .and_then(|rest| rest.strip_prefix('.'))
            .unwrap_or(&key);
        let system = Self::from_key(bare).ok_or_else(|| {
            teistro_core::catalogue::UnknownKey::in_kind::<Self>(bare).to_string()
        })?;
        if teistro_tajika::ANNUAL_DASHAS.contains(&system) {
            Ok(system)
        } else {
            Err(format!(
                "{key} is not an annual dasha; the annual dashas are {}",
                Self::SHAPES
            ))
        }
    }

    fn wire(self) -> String {
        self.full_key().to_owned()
    }
}

impl<T: Askable> Asked<T> {
    /// The members asked about, in the order they are answered.
    #[must_use]
    pub fn members(&self) -> &[T] {
        match self {
            Asked::All => T::ALL,
            Asked::These(these) => these,
        }
    }
}

impl<T: Askable> Serialize for Asked<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Asked::All => serializer.serialize_str("all"),
            Asked::These(these) => serializer.collect_seq(these.iter().map(|one| one.wire())),
        }
    }
}

impl<'de, T: Askable> Deserialize<'de> for Asked<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Shapes<T>(std::marker::PhantomData<T>);
        impl<'de, T: Askable> serde::de::Visitor<'de> for Shapes<T> {
            type Value = Asked<T>;

            fn expecting(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                out.write_str(T::SHAPES)
            }

            fn visit_str<E: serde::de::Error>(self, word: &str) -> Result<Asked<T>, E> {
                if word == "all" {
                    Ok(Asked::All)
                } else {
                    Err(E::custom(format!(
                        "{} are {}, not \"{word}\"",
                        T::FIELD,
                        T::SHAPES
                    )))
                }
            }

            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> Result<Asked<T>, A::Error> {
                use serde::de::Error as _;
                let mut these: Vec<T> = Vec::new();
                while let Some(wire) = seq.next_element::<T::Wire>()? {
                    let named = format!(
                        "{}{}",
                        T::NOUN,
                        serde_json::to_string(&wire).unwrap_or_default()
                    );
                    let one = T::read(wire).map_err(A::Error::custom)?;
                    if these.contains(&one) {
                        return Err(A::Error::custom(format!("{named} is asked twice")));
                    }
                    these.push(one);
                }
                Ok(Asked::These(these))
            }
        }
        deserializer.deserialize_any(Shapes(std::marker::PhantomData))
    }
}

/// Where a year's chart is cast (`03-design/muntha.md`, "Where a year's
/// chart is cast").
///
/// **The birthplace or a residence**, and the SDK decides neither for
/// anyone: the schools differ, and the one Tajika text read casts every
/// chart it works "for Bombay (the place of birth of the native)" without
/// stating a rule. So `"birth"` is a word a caller writes, not a default
/// a caller receives.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnnualPlace {
    /// The birth chart's own place and clock.
    Birth,
    /// Somewhere else, under its own clock.
    At {
        /// Where.
        place: Place,
        /// The clock kept there.
        offset: UtcOffset,
    },
}

/// The words `varsha.place` is written in, other than `"birth"`.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Residence {
    latitude_deg: f64,
    longitude_deg: f64,
    #[serde(default)]
    altitude_m: f64,
    utc_offset_seconds: i32,
}

/// What a caller is told when `place` is neither of its two shapes.
const PLACE_SHAPES: &str = "\"birth\", or {latitudeDeg, longitudeDeg, altitudeM, utcOffsetSeconds}";

impl Serialize for AnnualPlace {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            AnnualPlace::Birth => serializer.serialize_str("birth"),
            AnnualPlace::At { place, offset } => Residence {
                latitude_deg: place.latitude.get(),
                longitude_deg: place.longitude.get(),
                altitude_m: place.altitude.get(),
                utc_offset_seconds: offset.seconds(),
            }
            .serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for AnnualPlace {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Shapes;
        impl<'de> serde::de::Visitor<'de> for Shapes {
            type Value = AnnualPlace;

            fn expecting(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(out, "{PLACE_SHAPES}")
            }

            fn visit_str<E: serde::de::Error>(self, word: &str) -> Result<AnnualPlace, E> {
                if word == "birth" {
                    Ok(AnnualPlace::Birth)
                } else {
                    Err(E::custom(format!(
                        "a place is {PLACE_SHAPES}, not \"{word}\""
                    )))
                }
            }

            fn visit_map<M: serde::de::MapAccess<'de>>(
                self,
                map: M,
            ) -> Result<AnnualPlace, M::Error> {
                use serde::de::Error as _;
                let at: Residence =
                    Deserialize::deserialize(serde::de::value::MapAccessDeserializer::new(map))?;
                let refused = |why: String| M::Error::custom(why);
                let place = Place::new(
                    Latitude::try_new(at.latitude_deg)
                        .map_err(|why| refused(format!("latitudeDeg: {why}")))?,
                    Longitude::try_new(at.longitude_deg)
                        .map_err(|why| refused(format!("longitudeDeg: {why}")))?,
                    Altitude::try_new(at.altitude_m)
                        .map_err(|why| refused(format!("altitudeM: {why}")))?,
                );
                let offset = UtcOffset::try_from_seconds(at.utc_offset_seconds)
                    .map_err(|why| refused(format!("utcOffsetSeconds: {why}")))?;
                Ok(AnnualPlace::At { place, offset })
            }
        }
        deserializer.deserialize_any(Shapes)
    }
}

/// One birth's answer to a [`VarshaRequest`]: its years, and its own
/// sahams.
#[derive(Clone, Debug, Default)]
pub struct Varsha {
    /// The years, each with its chart when a place was asked for.
    pub years: Vec<VarshaYear>,
    /// The birth chart's own sahams, with their strength, when
    /// [`VarshaRequest::sahams`] asked; the source reads a year's beside
    /// them.
    pub natal_sahams: Vec<SahamStrength>,
}

/// One annual chart's instant and the Muntha standing at it, with the
/// year's own chart when a place was asked for.
///
/// They travel together because they are answered together: the Muntha
/// is the return's own year count progressed over the birth's lagna, and
/// the office-bearers are read from the birth and the chart founded at
/// that instant, so a second pass to fetch either would be a second chance
/// to disagree about which year it is (`03-design/muntha.md`).
#[derive(Clone, Debug)]
pub struct VarshaYear {
    /// The instant, and which year of the birth it opens.
    pub pravesha: Pravesha,
    /// The Muntha standing at it, progressed by that year's own count.
    pub muntha: Muntha,
    /// The year's own chart, read down to what Tajika reads from it; none
    /// unless [`VarshaRequest::place`] asked for the charts.
    pub annual: Option<AnnualChart>,
}

/// What a year's own chart says, for the office-bearers and whoever reads
/// the chart after them.
#[derive(Clone, Debug)]
pub struct AnnualChart {
    /// The annual chart's lagna, sidereal degrees.
    pub lagna_deg: f64,
    /// The five office-bearers, and whether the year opened by day.
    pub bearers: OfficeBearers,
    /// The lord of the year, with every claim it was chosen over.
    pub year_lord: Varshesha,
    /// The pairs of that chart that make a yoga: an Ithasala or an
    /// Ishrafa. The pairs that make none are the rest of the twenty-one,
    /// which [`ChartArea::drishtis`](crate::ChartArea::drishtis) answers.
    pub yogas: Vec<Between>,
    /// Which of the seven are retrograde and which combust: what the
    /// matters were judged on, reported so an answer can be read without
    /// the call that made it.
    pub states: AnnualStates,
    /// The sixteen yogas for each matter [`VarshaRequest::matters`] asked
    /// about, in its order; empty when it asked about none.
    pub matters: Vec<YearYogas>,
    /// Each saham [`VarshaRequest::sahams`] asked for, in its order, with
    /// its strength under the year's own lord; empty when it asked for
    /// none.
    pub sahams: Vec<SahamStrength>,
    /// The seven's Harsha bala in this year's chart.
    pub harsha: [Harsha; 7],
    /// Each annual dasha [`VarshaRequest::dashas`] asked for, in its
    /// order, under [`VarshaRequest::dasha_rules`]; empty when it asked
    /// for none.
    pub dashas: Vec<AnnualDasha>,
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        reason = "tests unwrap what they build and fail by panicking"
    )]

    use super::*;

    #[test]
    fn a_request_reads_back_what_it_writes() {
        let asked = VarshaRequest::through(12)
            .at(AnnualPlace::Birth)
            .with_matters(Asked::These(vec![House::ALL[6]]))
            .with_sahams(Asked::All)
            .with_dashas(Asked::These(vec![DashaSystem::Mudda]));
        let text = serde_json::to_string(&asked).unwrap();
        assert_eq!(VarshaRequest::from_json(&text).unwrap(), asked);
    }

    #[test]
    fn a_refusal_names_the_field_the_caller_wrote() {
        let refused = |text: &str| VarshaRequest::from_json(text).unwrap_err();
        let field = |text: &str| refused(text).field().map(str::to_owned);
        assert_eq!(
            field(r#"{"through": 201}"#).as_deref(),
            Some("varsha.through")
        );
        assert_eq!(
            field(r#"{"through": 3, "matters": [7]}"#).as_deref(),
            Some("varsha.matters")
        );
        assert_eq!(
            field(r#"{"through": 3, "dashas": "all"}"#).as_deref(),
            Some("varsha.dashas")
        );
        assert_eq!(
            field(r#"{"through": 3, "place": "birth", "sahams": ["PUNYA", "PUNYA"]}"#).as_deref(),
            Some("varsha.sahams")
        );
        assert_eq!(
            field(r#"{"through": 3, "place": "home"}"#).as_deref(),
            Some("varsha.place")
        );
        assert_eq!(
            field(r#"{"through": 3, "extra": 1}"#).as_deref(),
            Some("varsha.extra")
        );
    }

    #[test]
    fn an_annual_dasha_is_named_by_its_key_bare_or_full() {
        let read = |text: &str| {
            VarshaRequest::from_json(text)
                .unwrap()
                .dashas
                .unwrap()
                .members()
                .to_vec()
        };
        assert_eq!(
            read(
                r#"{"through": 1, "place": "birth", "dashas": ["MUDDA", "dasha_system.PATYAYINI"]}"#
            ),
            [DashaSystem::Mudda, DashaSystem::Patyayini]
        );
        let natal = VarshaRequest::from_json(
            r#"{"through": 1, "place": "birth", "dashas": ["VIMSHOTTARI"]}"#,
        )
        .unwrap_err();
        assert!(
            natal.to_string().contains("is not an annual dasha"),
            "{natal}"
        );
    }
}
