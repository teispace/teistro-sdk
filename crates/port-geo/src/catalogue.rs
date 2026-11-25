//! The place catalogue's half of the geo port (`docs/03-design/geo-port.md`
//! §12, SR-141 and SR-129): a name to places, a point to the places near it
//! and the units holding it, a place's names by language and script, and
//! the places of a unit, ranked by population.
//!
//! Each question is a request record read strictly, as every request is,
//! and checked here once for every provider ([`search`], [`reverse`],
//! [`names`], [`list`]), and each answer is checked as a resolution is: a
//! malformed answer is the provider's failure, `PROVIDER`.

use teistro_core::error::{Error, Status};

use crate::{GeoPoint, GeoProvider, country_code, field_of, restated, subdivision_code};

/// The most places a search or a reverse answers.
pub const MOST_FOUND: u32 = 100;

/// The most places a listing answers.
pub const MOST_LISTED: u32 = 1_000;

/// The distance, in metres, at which a search focus's gain halves when the
/// request leaves it out: teigeodata's and Pelias's default, a city and its
/// surroundings.
pub const FOCUS_SCALE_M: f64 = 50_000.0;

/// What a place of the catalogue is.
///
/// ```
/// use teistro_port_geo::PlaceKind;
///
/// assert_eq!(serde_json::to_value(PlaceKind::PostalArea)?, "POSTAL_AREA");
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlaceKind {
    /// A country, at admin level 0.
    Country,
    /// An administrative unit below a country: a province, a district, a
    /// municipality, a ward.
    Admin,
    /// A populated place: a city, a town, a village.
    Settlement,
    /// A named part of a settlement, or a locality with no people: a tole,
    /// a neighbourhood.
    Locality,
    /// A natural feature: a peak, a pass, a lake, a river.
    Natural,
    /// A postal area.
    PostalArea,
}

impl PlaceKind {
    /// Whether a place of this kind is a country or a unit, which has an
    /// admin level.
    #[must_use]
    pub const fn is_unit(self) -> bool {
        matches!(self, PlaceKind::Country | PlaceKind::Admin)
    }
}

/// Whether a place's number of people was counted or modelled.
///
/// ```
/// use teistro_port_geo::PopulationKind;
///
/// assert_eq!(serde_json::to_value(PopulationKind::Modelled)?, "MODELLED");
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PopulationKind {
    /// A source stated it: a census, or an estimate as the source gives it.
    Counted,
    /// A population grid summed over the place (a unit's area).
    Modelled,
}

/// A place's number of people, and whether it was counted or modelled.
///
/// ```
/// use teistro_port_geo::{Population, PopulationKind};
///
/// let kathmandu = Population { people: 863_151, kind: Some(PopulationKind::Modelled) };
/// assert_eq!(serde_json::to_value(kathmandu)?["kind"], "MODELLED");
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Population {
    /// The number of people.
    pub people: u64,
    /// Counted or modelled, `null` where the data does not say.
    pub kind: Option<PopulationKind>,
}

/// One admin unit above a place, as a place's `parents` list it.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ParentPlace {
    /// The unit's id, as a listing's `within` takes it.
    pub id: String,
    /// Its name, in the language asked where it has one.
    pub name: String,
    /// Its admin level: 0 for a country, then coarse to fine.
    pub level: u8,
    /// Its ISO 3166 code, or `null`.
    pub code: Option<String>,
}

/// A place of the catalogue, as a search, a reverse and a listing answer it.
///
/// ```
/// use teistro_port_geo::{NamedPlace, PlaceKind};
///
/// let place: NamedPlace = serde_json::from_str(r#"{"id": "np/1", "name": "Pokhara", "nameLang": null,
///     "label": "Pokhara, Nepal", "kind": "SETTLEMENT", "level": null, "code": null,
///     "latitudeDeg": 28.21, "longitudeDeg": 83.99, "population": null, "parents": []}"#)?;
/// assert_eq!(place.kind, PlaceKind::Settlement);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NamedPlace {
    /// The provider's stable id for the place (`tgd:np/326`).
    pub id: String,
    /// Its name in the language asked, else its canonical name.
    pub name: String,
    /// The language the data tags `name` with (`ne`), or `null` where it
    /// leaves the language undetermined.
    pub name_lang: Option<String>,
    /// `Name, Parent, Country`, for a picker: the name, the finest parent
    /// whose name differs from it, and the country when not already said.
    pub label: String,
    /// What the place is.
    pub kind: PlaceKind,
    /// Its admin level for a country or unit (0 a country), `null` for any
    /// other place.
    pub level: Option<u8>,
    /// Its ISO 3166 code, or `null`.
    pub code: Option<String>,
    /// Its latitude, in degrees.
    pub latitude_deg: f64,
    /// Its longitude, in degrees.
    pub longitude_deg: f64,
    /// Its number of people, or `null` where the data has none.
    pub population: Option<Population>,
    /// Its admin units, coarse to fine; a level the data does not hold is
    /// absent.
    pub parents: Vec<ParentPlace>,
    /// The id line the place's id belongs to, where a provider numbers
    /// more than one (teigeodata's `odbl` main and `cc-by-4.0` lite): an id
    /// is looked up again only in data of its line; `null` for a provider
    /// with one.
    #[serde(default)]
    pub line: Option<String>,
    /// Its IANA time zone (`Asia/Kathmandu`), or `null` where the data
    /// gives none.
    #[serde(default)]
    pub zone: Option<String>,
    /// What a landmark is (`temple`, `palace`, `airport`), the provider's
    /// own word, or `null`.
    #[serde(default)]
    pub class: Option<String>,
}

/// A place a search found, and its score.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FoundPlace {
    /// The place.
    pub place: NamedPlace,
    /// The provider's score, higher better; it compares within one query
    /// only.
    pub score: i32,
}

/// A place near a point a reverse was asked about.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NearPlace {
    /// The place.
    pub place: NamedPlace,
    /// The great-circle distance from the place to the point, in metres.
    pub distance_m: f64,
    /// The initial bearing from the place to the point, in degrees
    /// clockwise from true north, in `[0, 360)`.
    pub bearing_deg: f64,
    /// Whether the point lies inside the place, `null` where the provider
    /// cannot say.
    pub inside: Option<bool>,
}

/// A point's nearest named places and the units holding it.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NearbyPlaces {
    /// The nearest named places, nearest first.
    pub near: Vec<NearPlace>,
    /// The admin units holding the point, coarse to fine; a level the data
    /// holds no outline for is absent, and abroad of a country pack the
    /// list may be short or empty.
    pub admin: Vec<NamedPlace>,
}

/// One name of a place.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlaceName {
    /// The name.
    pub text: String,
    /// Its language (`ne`), or `null` where undetermined.
    pub lang: Option<String>,
    /// Its script (`Deva`), or `null` where unknown.
    pub script: Option<String>,
    /// Its type, as the provider names it (`OFFICIAL`, `ALT`), or `null`.
    #[serde(rename = "type")]
    pub kind: Option<String>,
    /// How it was made, as the provider names it (`REVIEWED`, a person's
    /// correction against a cited source; `OFFICIAL`; `WIKIDATA`), or
    /// `UNKNOWN`.
    pub origin: String,
    /// Whether a program wrote it (a romanisation, a transliteration)
    /// rather than a person.
    pub machine: bool,
}

/// The name chosen for one language asked.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChosenName {
    /// The language as asked.
    pub asked: String,
    /// The name chosen, or `null` where the place has none in it.
    pub name: Option<PlaceName>,
}

/// A place's names.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlaceNames {
    /// The place's id, as asked.
    pub id: String,
    /// One entry per language asked, in the order asked.
    pub best: Vec<ChosenName>,
    /// Every name the place has, its canonical name first; empty for a
    /// place the provider does not know.
    pub all: Vec<PlaceName>,
}

impl PlaceNames {
    /// The answer for a place with no names: each language asked, with
    /// none.
    ///
    /// ```
    /// use teistro_port_geo::{PlaceNames, PlaceNamesRequest};
    ///
    /// let none = PlaceNames::none(&PlaceNamesRequest { id: "x".into(), langs: vec!["ne".into()] });
    /// assert!(none.all.is_empty() && none.best[0].name.is_none());
    /// ```
    #[must_use]
    pub fn none(asked: &PlaceNamesRequest) -> PlaceNames {
        PlaceNames {
            id: asked.id.clone(),
            best: asked
                .langs
                .iter()
                .map(|lang| ChosenName {
                    asked: lang.clone(),
                    name: None,
                })
                .collect(),
            all: Vec::new(),
        }
    }
}

/// A search's focus: prefer places near this point.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SearchFocus {
    /// The latitude, in degrees.
    pub latitude_deg: f64,
    /// The longitude, in degrees.
    pub longitude_deg: f64,
    /// The distance, in metres, at which the focus's gain halves; 50 km
    /// when left out.
    #[serde(default = "focus_scale")]
    pub scale_m: f64,
}

/// A name to places: what `geo.search` reads.
///
/// ```
/// use teistro_port_geo::PlaceSearch;
///
/// let asked: PlaceSearch = serde_json::from_str(r#"{"query": "Kathmandu, Bagmati"}"#)?;
/// assert_eq!((asked.limit, asked.focus, asked.lang), (10, None, None));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlaceSearch {
    /// The text as typed, in any script; after a comma it names a parent
    /// (`Kathmandu, Bagmati`).
    pub query: String,
    /// Prefer places near this point, or `null`.
    #[serde(default)]
    pub focus: Option<SearchFocus>,
    /// The most places to answer, 1 to 100; 10 when left out.
    #[serde(default = "ten")]
    pub limit: u32,
    /// The language to name each place in (`ne`, `en-Latn`), or `null`
    /// for its canonical name.
    #[serde(default)]
    pub lang: Option<String>,
}

/// A point to its nearest places and its units: what `geo.reverse` reads.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlaceReverse {
    /// The latitude, in degrees.
    pub latitude_deg: f64,
    /// The longitude, in degrees.
    pub longitude_deg: f64,
    /// The most nearby places to answer, 1 to 100; 5 when left out.
    #[serde(default = "five")]
    pub limit: u32,
    /// The language to name each place in, or `null`.
    #[serde(default)]
    pub lang: Option<String>,
}

/// A place's names: what `geo.names` reads.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlaceNamesRequest {
    /// The place's id, as an answer gave it.
    pub id: String,
    /// The languages to choose a name in (`ne`, `en`, `hi`, `sa`); empty
    /// for none.
    #[serde(default)]
    pub langs: Vec<String>,
}

/// A place by its id: what `geo.place` reads (SR-164). A provider follows
/// a retired id to the place that replaced it, so the answer's `id` is the
/// survivor's: a caller that saved `id` stores the answer's instead.
///
/// ```
/// use teistro_port_geo::PlaceLookup;
///
/// let asked: PlaceLookup = serde_json::from_str(r#"{"id": "tgd:np/54s", "lang": "ne"}"#)?;
/// assert!(asked.checked().is_ok());
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlaceLookup {
    /// The place's id, as an answer gave it.
    pub id: String,
    /// The language to name it in, or `null`.
    #[serde(default)]
    pub lang: Option<String>,
}

/// What `geo.place` answers: the place an id names, and whether the id
/// was retired for it.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LookedUpPlace {
    /// The id asked.
    pub asked: String,
    /// The place, under its current id; `null` for an id the provider does
    /// not know or holds as removed.
    pub place: Option<NamedPlace>,
    /// Whether the id asked was retired and `place` is the one that
    /// replaced it: a caller that kept the id stores `place.id` instead.
    pub moved: bool,
}

impl PlaceLookup {
    /// The request checked: an id, and the language a tag.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` named by the field.
    pub fn checked(&self) -> Result<(), Error> {
        written(&self.id, "id", "a place's id")?;
        tagged(self.lang.as_deref(), "lang")
    }
}

/// The places of a unit: what `geo.list` reads.
///
/// ```
/// use teistro_port_geo::PlaceListing;
///
/// let asked: PlaceListing = serde_json::from_str(r#"{"within": "NP", "level": 1}"#)?;
/// assert_eq!((asked.depth, asked.limit), (Some(1), 100));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlaceListing {
    /// The unit to list within: a place's id, or an ISO 3166 code (`NP`,
    /// `NP-P3`).
    pub within: String,
    /// Only units of this admin level, or `null` for any place.
    #[serde(default)]
    pub level: Option<u8>,
    /// Only places of these kinds; empty for every kind.
    #[serde(default)]
    pub kinds: Vec<PlaceKind>,
    /// How many levels below `within` to look: 1 its children, `null` any
    /// depth; 1 when left out.
    #[serde(default = "one")]
    pub depth: Option<u8>,
    /// The most places to answer, 1 to 1,000; 100 when left out.
    #[serde(default = "hundred")]
    pub limit: u32,
    /// The language to name each place in, or `null`.
    #[serde(default)]
    pub lang: Option<String>,
}

fn focus_scale() -> f64 {
    FOCUS_SCALE_M
}

fn ten() -> u32 {
    10
}

fn five() -> u32 {
    5
}

#[expect(
    clippy::unnecessary_wraps,
    reason = "serde's default for an optional depth whose null means any"
)]
fn one() -> Option<u8> {
    Some(1)
}

fn hundred() -> u32 {
    100
}

/// A language tag's language and script, as a provider reads a `lang`.
///
/// ```
/// use teistro_port_geo::LangTag;
///
/// let ne = LangTag::parse("ne-Deva-NP").unwrap();
/// assert_eq!((ne.language.as_str(), ne.script.as_deref()), ("ne", Some("Deva")));
/// // The script a language is usually written in, where none is asked.
/// assert_eq!(LangTag::parse("sa").unwrap().script_or_usual(), Some("Deva"));
/// assert!(LangTag::parse("not a tag").is_none());
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LangTag {
    /// The language subtag, lower case (`ne`).
    pub language: String,
    /// The script subtag, title case (`Deva`), where the tag names one.
    pub script: Option<String>,
}

impl LangTag {
    /// The tag's language (two or three letters) and script (four letters,
    /// where its second subtag is one); a region or any later subtag is
    /// ignored. `None` for text that is not such a tag.
    #[must_use]
    pub fn parse(tag: &str) -> Option<LangTag> {
        let mut parts = tag.split(['-', '_']);
        let language = parts.next()?;
        if !(2..=3).contains(&language.len()) || !language.bytes().all(|b| b.is_ascii_alphabetic())
        {
            return None;
        }
        let script = parts
            .next()
            .filter(|part| part.len() == 4 && part.bytes().all(|b| b.is_ascii_alphabetic()))
            .map(|part| {
                let mut chars = part.chars();
                let mut out = String::new();
                if let Some(first) = chars.next() {
                    out.push(first.to_ascii_uppercase());
                }
                out.extend(chars.map(|c| c.to_ascii_lowercase()));
                out
            });
        Some(LangTag {
            language: language.to_ascii_lowercase(),
            script,
        })
    }

    /// The script asked, else the script the language is usually written
    /// in where the port knows it: Devanagari for `ne`, `hi`, `sa`, `mr`,
    /// `mai`, `bho` and `new`, Latin for `en`.
    #[must_use]
    pub fn script_or_usual(&self) -> Option<&str> {
        if let Some(script) = self.script.as_deref() {
            return Some(script);
        }
        match self.language.as_str() {
            "ne" | "hi" | "sa" | "mr" | "mai" | "bho" | "new" => Some("Deva"),
            "en" => Some("Latn"),
            _ => None,
        }
    }
}

/// `limit` refused unless it is 1 to `most`.
fn limited(limit: u32, most: u32) -> Result<usize, Error> {
    if (1..=most).contains(&limit) {
        return Ok(limit as usize);
    }
    Err(Error::invalid_arg(format!("a limit of {limit} is not 1 to {most}")).with_field("limit"))
}

/// `lang` refused unless it is a language tag.
fn tagged(lang: Option<&str>, field: &str) -> Result<(), Error> {
    match lang {
        Some(tag) if LangTag::parse(tag).is_none() => {
            Err(Error::invalid_arg(format!("`{tag}` is not a language tag"))
                .with_field(field.to_owned())
                .with_hint("a BCP 47 tag such as `ne`, `ne-Deva` or `en`"))
        }
        _ => Ok(()),
    }
}

/// `text` refused when it is empty or blank.
fn written(text: &str, field: &str, what: &str) -> Result<(), Error> {
    if text.trim().is_empty() {
        return Err(Error::invalid_arg(format!("{what} is empty")).with_field(field.to_owned()));
    }
    Ok(())
}

impl PlaceSearch {
    /// The request checked: a query, a limit of 1 to 100, a focus on the
    /// globe with a positive scale, a language tag.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` named by the field.
    pub fn checked(&self) -> Result<usize, Error> {
        written(&self.query, "query", "a search's query")?;
        if let Some(focus) = self.focus {
            GeoPoint::try_from_degrees(focus.latitude_deg, focus.longitude_deg).map_err(|why| {
                let field = why.field().map(|field| format!("focus.{field}"));
                match field {
                    Some(field) => why.with_field(field),
                    None => why,
                }
            })?;
            if !(focus.scale_m.is_finite() && focus.scale_m > 0.0) {
                return Err(Error::invalid_arg(format!(
                    "a focus scale of {} m is not a positive number",
                    focus.scale_m
                ))
                .with_field("focus.scaleM"));
            }
        }
        tagged(self.lang.as_deref(), "lang")?;
        limited(self.limit, MOST_FOUND)
    }
}

impl PlaceReverse {
    /// The point asked, checked with the limit and the language.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` named by the field.
    pub fn checked(&self) -> Result<(GeoPoint, usize), Error> {
        let at = GeoPoint::try_from_degrees(self.latitude_deg, self.longitude_deg)?;
        tagged(self.lang.as_deref(), "lang")?;
        Ok((at, limited(self.limit, MOST_FOUND)?))
    }
}

impl PlaceNamesRequest {
    /// The request checked: an id, and each language a tag.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` named by the field.
    pub fn checked(&self) -> Result<(), Error> {
        written(&self.id, "id", "a place's id")?;
        for (at, lang) in self.langs.iter().enumerate() {
            tagged(Some(lang), &format!("langs[{at}]"))?;
        }
        Ok(())
    }
}

impl PlaceListing {
    /// The children of `unit` (an id or an ISO 3166 code), every kind, at
    /// most 100, named canonically: the request's defaults.
    ///
    /// ```
    /// use teistro_port_geo::{PlaceKind, PlaceListing};
    ///
    /// let asked = PlaceListing::within("NP").at_any_depth().of_level(3).of_kinds([PlaceKind::Admin]);
    /// assert_eq!((asked.depth, asked.level, asked.limit), (None, Some(3), 100));
    /// ```
    #[must_use]
    pub fn within(unit: impl Into<String>) -> PlaceListing {
        PlaceListing {
            within: unit.into(),
            level: None,
            kinds: Vec::new(),
            depth: one(),
            limit: hundred(),
            lang: None,
        }
    }

    /// This listing looking at any depth below its unit.
    #[must_use]
    pub fn at_any_depth(mut self) -> PlaceListing {
        self.depth = None;
        self
    }

    /// This listing kept to units of admin level `level`.
    #[must_use]
    pub fn of_level(mut self, level: u8) -> PlaceListing {
        self.level = Some(level);
        self
    }

    /// This listing kept to places of `kinds`.
    #[must_use]
    pub fn of_kinds(mut self, kinds: impl IntoIterator<Item = PlaceKind>) -> PlaceListing {
        self.kinds = kinds.into_iter().collect();
        self
    }

    /// The request checked: a unit, a depth of at least 1, a limit of 1 to
    /// 1,000, a language tag.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` named by the field.
    pub fn checked(&self) -> Result<usize, Error> {
        written(&self.within, "within", "a listing's unit")?;
        if self.depth == Some(0) {
            return Err(
                Error::invalid_arg("a listing looks at least one level down")
                    .with_field("depth")
                    .with_hint("1 for the unit's children, null for any depth"),
            );
        }
        tagged(self.lang.as_deref(), "lang")?;
        limited(self.limit, MOST_LISTED)
    }
}

impl NamedPlace {
    /// Refuses a place that is not well formed, under `root`.
    fn check(&self, root: &str) -> Result<(), Error> {
        let at = |field: &str| field_of(root, field);
        if self.id.trim().is_empty() {
            return Err(Error::invalid_arg("a place has an id").with_field(at("id")));
        }
        if self.name.trim().is_empty() {
            return Err(Error::invalid_arg("a place has a name").with_field(at("name")));
        }
        GeoPoint::try_from_degrees(self.latitude_deg, self.longitude_deg).map_err(|why| {
            let field = at(why.field().unwrap_or("latitudeDeg"));
            why.with_field(field)
        })?;
        if self.kind.is_unit() != self.level.is_some() {
            return Err(Error::invalid_arg(
                "a country or unit has an admin level, and no other place has one",
            )
            .with_field(at("level")));
        }
        if let Some(code) = self.code.as_deref() {
            let shaped = match self.kind {
                PlaceKind::Country => country_code(code),
                _ => subdivision_code(code) || country_code(code),
            };
            if !shaped {
                return Err(
                    Error::invalid_arg(format!("`{code}` is not an ISO 3166 code"))
                        .with_field(at("code")),
                );
            }
        }
        let mut last = 0_u8;
        for (index, parent) in self.parents.iter().enumerate() {
            let root = format!("{}[{index}]", at("parents"));
            if parent.level < last {
                return Err(
                    Error::invalid_arg("a place's parents are listed coarse to fine")
                        .with_field(field_of(&root, "level")),
                );
            }
            last = parent.level;
            if parent.id.trim().is_empty() {
                return Err(
                    Error::invalid_arg("a parent has an id").with_field(field_of(&root, "id"))
                );
            }
        }
        Ok(())
    }
}

/// A finite, non-negative distance and a bearing in `[0, 360)`.
fn check_near(near: &NearPlace, root: &str) -> Result<(), Error> {
    near.place.check(&field_of(root, "place"))?;
    if !(near.distance_m.is_finite() && near.distance_m >= 0.0) {
        return Err(Error::invalid_arg(format!(
            "a distance of {} m is not a distance",
            near.distance_m
        ))
        .with_field(field_of(root, "distanceM")));
    }
    if !(near.bearing_deg.is_finite() && (0.0..360.0).contains(&near.bearing_deg)) {
        return Err(Error::invalid_arg(format!(
            "a bearing of {}° is not in [0, 360)",
            near.bearing_deg
        ))
        .with_field(field_of(root, "bearingDeg")));
    }
    Ok(())
}

/// The places matching `asked.query`, best first, as `provider` ranks them:
/// the request checked, at most `limit`, every answer checked.
///
/// ```
/// use teistro_port_geo::{GeoTable, PlaceSearch, search};
///
/// let table = GeoTable::from_json(r#"{"source": {"name": "t", "version": "1", "dataVersion": "1", "tzdb": null}, "areas": []}"#)?;
/// // A provider with no catalogue finds nothing.
/// let asked = PlaceSearch { query: "Kathmandu".into(), focus: None, limit: 5, lang: None };
/// assert!(search(&table, &asked)?.is_empty());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// `INVALID_ARG` on the request's field; the provider's own refusal;
/// `PROVIDER` naming an answer that is not well formed (`[2].place.code`).
pub fn search(provider: &dyn GeoProvider, asked: &PlaceSearch) -> Result<Vec<FoundPlace>, Error> {
    let limit = asked.checked()?;
    let mut found = provider.search(asked)?;
    found.truncate(limit);
    for (at, hit) in found.iter().enumerate() {
        hit.place
            .check(&format!("[{at}].place"))
            .map_err(|why| restated(why, Status::Provider))?;
    }
    Ok(found)
}

/// The places nearest a point and the units holding it, as `provider`
/// answers them: the request checked, every answer checked.
///
/// # Errors
///
/// As [`search`], the point refused on `latitudeDeg` or `longitudeDeg`.
pub fn reverse(provider: &dyn GeoProvider, asked: &PlaceReverse) -> Result<NearbyPlaces, Error> {
    let (_, limit) = asked.checked()?;
    let mut nearby = provider.reverse(asked)?;
    nearby.near.truncate(limit);
    for (at, near) in nearby.near.iter().enumerate() {
        check_near(near, &format!("near[{at}]")).map_err(|why| restated(why, Status::Provider))?;
    }
    for (at, unit) in nearby.admin.iter().enumerate() {
        unit.check(&format!("admin[{at}]"))
            .map_err(|why| restated(why, Status::Provider))?;
    }
    Ok(nearby)
}

/// A place's names as `provider` holds them, with a name chosen for each
/// language asked.
///
/// # Errors
///
/// `INVALID_ARG` on the request's field; the provider's own refusal;
/// `PROVIDER` for an answer naming another place, or one whose chosen names
/// do not follow the languages asked.
pub fn names(provider: &dyn GeoProvider, asked: &PlaceNamesRequest) -> Result<PlaceNames, Error> {
    asked.checked()?;
    let answer = provider.names(asked)?;
    if answer.id != asked.id {
        return Err(Error::new(
            Status::Provider,
            format!(
                "asked the names of `{}`, the provider answered `{}`'s",
                asked.id, answer.id
            ),
        )
        .with_field("id"));
    }
    let follows = answer.best.len() == asked.langs.len()
        && answer
            .best
            .iter()
            .zip(&asked.langs)
            .all(|(chosen, lang)| &chosen.asked == lang);
    if !follows {
        return Err(Error::new(
            Status::Provider,
            "the chosen names do not follow the languages asked, one each in order",
        )
        .with_field("best"));
    }
    for (at, name) in answer.all.iter().enumerate() {
        if name.text.is_empty() {
            return Err(Error::new(Status::Provider, "a name is empty")
                .with_field(format!("all[{at}].text")));
        }
    }
    Ok(answer)
}

/// The place an id names, as `provider` holds it, a retired id followed to
/// its survivor: the request checked, the answer checked, and `moved` when
/// the place answered is under another id.
///
/// # Errors
///
/// As [`search`].
pub fn place(provider: &dyn GeoProvider, asked: &PlaceLookup) -> Result<LookedUpPlace, Error> {
    asked.checked()?;
    let found = provider.place(asked)?;
    if let Some(found) = &found {
        found
            .check("place")
            .map_err(|why| restated(why, Status::Provider))?;
    }
    Ok(LookedUpPlace {
        asked: asked.id.clone(),
        moved: found.as_ref().is_some_and(|found| found.id != asked.id),
        place: found,
    })
}

/// The places of a unit, as `provider` lists and ranks them: the request
/// checked, at most `limit`, every answer checked.
///
/// # Errors
///
/// As [`search`].
pub fn list(provider: &dyn GeoProvider, asked: &PlaceListing) -> Result<Vec<NamedPlace>, Error> {
    let limit = asked.checked()?;
    let mut places = provider.list(asked)?;
    places.truncate(limit);
    for (at, place) in places.iter().enumerate() {
        place
            .check(&format!("[{at}]"))
            .map_err(|why| restated(why, Status::Provider))?;
    }
    Ok(places)
}

/// The label of a place called `name` with `parents` coarse to fine: the
/// name, the finest parent whose name differs from it, and the country
/// (the coarsest, at level 0) when not already said.
///
/// ```
/// use teistro_port_geo::{ParentPlace, label};
///
/// let unit = |id: &str, name: &str, level| ParentPlace { id: id.into(), name: name.into(), level, code: None };
/// let parents = [unit("1", "Nepal", 0), unit("2", "Bagmati", 1), unit("3", "Kathmandu", 2)];
/// assert_eq!(label("Kathmandu", &parents), "Kathmandu, Bagmati, Nepal");
/// assert_eq!(label("Nepal", &parents[..0]), "Nepal");
/// ```
#[must_use]
pub fn label(name: &str, parents: &[ParentPlace]) -> String {
    let mut parts = vec![name];
    if let Some(finest) = parents.iter().rev().find(|parent| parent.name != name) {
        parts.push(&finest.name);
    }
    if let Some(country) = parents.first().filter(|parent| parent.level == 0)
        && !parts.contains(&country.name.as_str())
    {
        parts.push(&country.name);
    }
    parts.join(", ")
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use std::sync::Arc;

    use super::*;
    use crate::{AdminUnit, GeoSource, GroundHeight};

    /// A provider answering every catalogue question with what it holds.
    #[derive(Default)]
    struct Canned {
        found: Vec<FoundPlace>,
        nearby: NearbyPlaces,
        names: Option<PlaceNames>,
        listed: Vec<NamedPlace>,
        looked_up: Option<NamedPlace>,
    }

    impl GeoProvider for Canned {
        fn source(&self) -> GeoSource {
            GeoSource {
                name: "canned".into(),
                version: "1".into(),
                data_version: "1".into(),
                tzdb: None,
            }
        }

        fn zone_at(&self, _: GeoPoint) -> Result<Option<String>, Error> {
            Ok(None)
        }

        fn height_at(&self, _: GeoPoint) -> Result<Option<GroundHeight>, Error> {
            Ok(None)
        }

        fn admin_at(&self, _: GeoPoint) -> Result<Vec<AdminUnit>, Error> {
            Ok(Vec::new())
        }

        fn search(&self, _: &PlaceSearch) -> Result<Vec<FoundPlace>, Error> {
            Ok(self.found.clone())
        }

        fn reverse(&self, _: &PlaceReverse) -> Result<NearbyPlaces, Error> {
            Ok(self.nearby.clone())
        }

        fn names(&self, asked: &PlaceNamesRequest) -> Result<PlaceNames, Error> {
            Ok(self
                .names
                .clone()
                .unwrap_or_else(|| PlaceNames::none(asked)))
        }

        fn list(&self, _: &PlaceListing) -> Result<Vec<NamedPlace>, Error> {
            Ok(self.listed.clone())
        }

        fn place(&self, _: &PlaceLookup) -> Result<Option<NamedPlace>, Error> {
            Ok(self.looked_up.clone())
        }
    }

    fn parent(id: &str, name: &str, level: u8) -> ParentPlace {
        ParentPlace {
            id: id.into(),
            name: name.into(),
            level,
            code: None,
        }
    }

    fn pokhara() -> NamedPlace {
        NamedPlace {
            id: "p1".into(),
            name: "Pokhara".into(),
            name_lang: None,
            label: "Pokhara, Kaski, Nepal".into(),
            kind: PlaceKind::Settlement,
            level: None,
            code: None,
            latitude_deg: 28.21,
            longitude_deg: 83.99,
            population: Some(Population {
                people: 518_452,
                kind: Some(PopulationKind::Counted),
            }),
            parents: vec![parent("np", "Nepal", 0), parent("kaski", "Kaski", 2)],
            line: None,
            zone: Some("Asia/Kathmandu".into()),
            class: None,
        }
    }

    fn nepal() -> NamedPlace {
        NamedPlace {
            id: "np".into(),
            name: "Nepal".into(),
            label: "Nepal".into(),
            kind: PlaceKind::Country,
            level: Some(0),
            code: Some("NP".into()),
            parents: Vec::new(),
            population: None,
            ..pokhara()
        }
    }

    fn near(distance_m: f64, bearing_deg: f64) -> NearPlace {
        NearPlace {
            place: pokhara(),
            distance_m,
            bearing_deg,
            inside: None,
        }
    }

    fn search_for(query: &str) -> PlaceSearch {
        PlaceSearch {
            query: query.into(),
            focus: None,
            limit: 10,
            lang: None,
        }
    }

    fn reverse_at() -> PlaceReverse {
        PlaceReverse {
            latitude_deg: 28.2,
            longitude_deg: 84.0,
            limit: 5,
            lang: None,
        }
    }

    /// The field and status a refusal names.
    fn refusal(why: &Error) -> (Status, Option<&str>) {
        (why.status, why.field())
    }

    fn malformed(place: NamedPlace) -> Error {
        let canned = Canned {
            listed: vec![pokhara(), place],
            ..Canned::default()
        };
        list(&canned, &PlaceListing::within("NP")).unwrap_err()
    }

    /// A provider answering only the questions every provider must.
    struct Bare;

    impl GeoProvider for Bare {
        fn source(&self) -> GeoSource {
            Canned::default().source()
        }

        fn zone_at(&self, _: GeoPoint) -> Result<Option<String>, Error> {
            Ok(None)
        }

        fn height_at(&self, _: GeoPoint) -> Result<Option<GroundHeight>, Error> {
            Ok(None)
        }

        fn admin_at(&self, _: GeoPoint) -> Result<Vec<AdminUnit>, Error> {
            Ok(Vec::new())
        }
    }

    #[test]
    fn a_provider_without_a_catalogue_finds_nothing_borrowed_or_shared() {
        fn answers<P: GeoProvider>(provider: &P) -> (usize, NearbyPlaces, PlaceNames, usize, bool) {
            let asked = PlaceNamesRequest {
                id: "p1".into(),
                langs: vec!["ne".into()],
            };
            let at = GeoPoint::new(
                teistro_core::quantity::Latitude::literal(28.2),
                teistro_core::quantity::Longitude::literal(84.0),
            );
            (
                provider.search(&search_for("Pokhara")).unwrap().len(),
                provider.reverse(&reverse_at()).unwrap(),
                provider.names(&asked).unwrap(),
                provider.list(&PlaceListing::within("NP")).unwrap().len(),
                provider.at_sea(at).unwrap(),
            )
        }
        let bare = answers(&Bare);
        assert_eq!(bare.0 + bare.3, 0);
        assert_eq!(bare.1, NearbyPlaces::default());
        assert_eq!(
            bare.2,
            PlaceNames::none(&PlaceNamesRequest {
                id: "p1".into(),
                langs: vec!["ne".into()]
            })
        );
        assert!(!bare.4);
        assert_eq!(answers(&&Bare), bare);
        assert_eq!(answers(&Arc::new(Bare)), bare);
    }

    #[test]
    fn a_language_tag_is_its_language_and_script() {
        let tag = LangTag::parse("NE_deva_np").unwrap();
        assert_eq!(
            (tag.language.as_str(), tag.script.as_deref()),
            ("ne", Some("Deva"))
        );
        // A region in the script's place is no script.
        assert_eq!(LangTag::parse("en-GB").unwrap().script, None);
        assert_eq!(
            LangTag::parse("en").unwrap().script_or_usual(),
            Some("Latn")
        );
        assert_eq!(
            LangTag::parse("new").unwrap().script_or_usual(),
            Some("Deva")
        );
        assert_eq!(LangTag::parse("ja").unwrap().script_or_usual(), None);
        assert_eq!(
            LangTag::parse("sa-Latn").unwrap().script_or_usual(),
            Some("Latn")
        );
        for bad in ["", "n", "nepa", "n3", "ne pal"] {
            assert!(LangTag::parse(bad).is_none(), "{bad:?}");
        }
    }

    #[test]
    fn a_search_is_refused_on_the_field_at_fault() {
        let canned = Canned::default();
        let field = |asked: PlaceSearch| {
            search(&canned, &asked)
                .unwrap_err()
                .field()
                .map(str::to_owned)
        };
        assert_eq!(field(search_for("  ")).as_deref(), Some("query"));
        let far = SearchFocus {
            latitude_deg: 91.0,
            longitude_deg: 0.0,
            scale_m: 1.0,
        };
        assert_eq!(
            field(PlaceSearch {
                focus: Some(far),
                ..search_for("x")
            })
            .as_deref(),
            Some("focus.latitudeDeg")
        );
        for scale_m in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            let focus = SearchFocus {
                latitude_deg: 28.0,
                longitude_deg: 84.0,
                scale_m,
            };
            assert_eq!(
                field(PlaceSearch {
                    focus: Some(focus),
                    ..search_for("x")
                })
                .as_deref(),
                Some("focus.scaleM")
            );
        }
        assert_eq!(
            field(PlaceSearch {
                lang: Some("not a tag".into()),
                ..search_for("x")
            })
            .as_deref(),
            Some("lang")
        );
        for limit in [0, MOST_FOUND + 1] {
            assert_eq!(
                field(PlaceSearch {
                    limit,
                    ..search_for("x")
                })
                .as_deref(),
                Some("limit")
            );
        }
        let focus = SearchFocus {
            latitude_deg: 28.0,
            longitude_deg: 84.0,
            scale_m: FOCUS_SCALE_M,
        };
        let asked = PlaceSearch {
            focus: Some(focus),
            lang: Some("ne".into()),
            limit: MOST_FOUND,
            ..search_for("x")
        };
        assert_eq!(asked.checked().unwrap(), 100);
    }

    #[test]
    fn a_search_keeps_the_limit_and_checks_every_answer() {
        let hit = |score| FoundPlace {
            place: pokhara(),
            score,
        };
        let canned = Canned {
            found: vec![hit(3), hit(2), hit(1)],
            ..Canned::default()
        };
        let asked = PlaceSearch {
            limit: 2,
            ..search_for("Pokhara")
        };
        assert_eq!(
            search(&canned, &asked)
                .unwrap()
                .iter()
                .map(|f| f.score)
                .collect::<Vec<_>>(),
            [3, 2]
        );
        // Through a reference and a shared provider, the same answer.
        assert_eq!(search(&&canned, &asked).unwrap().len(), 2);
        assert_eq!(
            search(
                &Arc::new(Canned {
                    found: vec![hit(1)],
                    ..Canned::default()
                }),
                &asked
            )
            .unwrap()
            .len(),
            1
        );
        let bad = Canned {
            found: vec![
                hit(1),
                FoundPlace {
                    place: NamedPlace {
                        name: " ".into(),
                        ..pokhara()
                    },
                    score: 0,
                },
            ],
            ..Canned::default()
        };
        let why = search(&bad, &asked).unwrap_err();
        assert_eq!(refusal(&why), (Status::Provider, Some("[1].place.name")));
    }

    #[test]
    fn a_malformed_place_is_the_providers_failure_named_by_its_field() {
        let cases: [(NamedPlace, &str); 8] = [
            (
                NamedPlace {
                    id: String::new(),
                    ..pokhara()
                },
                "[1].id",
            ),
            (
                NamedPlace {
                    latitude_deg: 100.0,
                    ..pokhara()
                },
                "[1].latitudeDeg",
            ),
            (
                NamedPlace {
                    longitude_deg: f64::NAN,
                    ..pokhara()
                },
                "[1].longitudeDeg",
            ),
            (
                NamedPlace {
                    level: Some(2),
                    ..pokhara()
                },
                "[1].level",
            ),
            (
                NamedPlace {
                    level: None,
                    ..nepal()
                },
                "[1].level",
            ),
            (
                NamedPlace {
                    code: Some("NP-P3".into()),
                    ..nepal()
                },
                "[1].code",
            ),
            (
                NamedPlace {
                    code: Some("np".into()),
                    ..pokhara()
                },
                "[1].code",
            ),
            (
                NamedPlace {
                    parents: vec![parent("kaski", "Kaski", 2), parent("np", "Nepal", 0)],
                    ..pokhara()
                },
                "[1].parents[1].level",
            ),
        ];
        for (place, field) in cases {
            assert_eq!(refusal(&malformed(place)), (Status::Provider, Some(field)));
        }
        let unnamed_parent = NamedPlace {
            parents: vec![parent("np", "Nepal", 0), parent(" ", "Kaski", 2)],
            ..pokhara()
        };
        assert_eq!(
            refusal(&malformed(unnamed_parent)),
            (Status::Provider, Some("[1].parents[1].id"))
        );
        // A unit's subdivision code, and a place's country code, are well formed.
        let bagmati = NamedPlace {
            kind: PlaceKind::Admin,
            level: Some(1),
            code: Some("NP-P3".into()),
            ..pokhara()
        };
        let in_nepal = NamedPlace {
            code: Some("NP".into()),
            ..pokhara()
        };
        let canned = Canned {
            listed: vec![nepal(), bagmati, in_nepal],
            ..Canned::default()
        };
        assert_eq!(list(&canned, &PlaceListing::within("NP")).unwrap().len(), 3);
    }

    #[test]
    fn a_listing_is_refused_on_its_field_and_keeps_its_limit() {
        let canned = Canned {
            listed: vec![nepal(), pokhara(), pokhara()],
            ..Canned::default()
        };
        let field = |asked: PlaceListing| {
            list(&canned, &asked)
                .unwrap_err()
                .field()
                .map(str::to_owned)
        };
        assert_eq!(field(PlaceListing::within("")).as_deref(), Some("within"));
        let flat = PlaceListing {
            depth: Some(0),
            ..PlaceListing::within("NP")
        };
        let why = list(&canned, &flat).unwrap_err();
        assert_eq!((why.field(), why.hint().is_some()), (Some("depth"), true));
        assert_eq!(
            field(PlaceListing {
                lang: Some("?".into()),
                ..PlaceListing::within("NP")
            })
            .as_deref(),
            Some("lang")
        );
        assert_eq!(
            field(PlaceListing {
                limit: MOST_LISTED + 1,
                ..PlaceListing::within("NP")
            })
            .as_deref(),
            Some("limit")
        );
        let two = PlaceListing {
            limit: 2,
            ..PlaceListing::within("NP")
        }
        .at_any_depth();
        assert_eq!(list(&Arc::new(canned), &two).unwrap().len(), 2);
        let kinds = PlaceListing::within("NP")
            .of_level(1)
            .of_kinds([PlaceKind::Admin, PlaceKind::Country]);
        assert_eq!(
            (kinds.level, kinds.kinds.len(), kinds.depth),
            (Some(1), 2, Some(1))
        );
    }

    #[test]
    fn a_reverse_checks_its_point_and_every_answer() {
        let canned = Canned::default();
        let why = reverse(
            &canned,
            &PlaceReverse {
                latitude_deg: -91.0,
                ..reverse_at()
            },
        )
        .unwrap_err();
        assert_eq!(why.field(), Some("latitudeDeg"));
        let why = reverse(
            &canned,
            &PlaceReverse {
                lang: Some("x y".into()),
                ..reverse_at()
            },
        )
        .unwrap_err();
        assert_eq!(why.field(), Some("lang"));
        let why = reverse(
            &canned,
            &PlaceReverse {
                limit: 0,
                ..reverse_at()
            },
        )
        .unwrap_err();
        assert_eq!(why.field(), Some("limit"));

        let good = Canned {
            nearby: NearbyPlaces {
                near: vec![near(10.0, 0.0), near(20.0, 359.9), near(0.0, 90.0)],
                admin: vec![nepal()],
            },
            ..Canned::default()
        };
        let two = PlaceReverse {
            limit: 2,
            ..reverse_at()
        };
        let nearby = reverse(&&good, &two).unwrap();
        assert_eq!((nearby.near.len(), nearby.admin.len()), (2, 1));
        assert_eq!(reverse(&Arc::new(good), &two).unwrap().near.len(), 2);
    }

    #[test]
    fn a_malformed_nearby_answer_is_the_providers_failure() {
        let failing = |nearby: NearbyPlaces| {
            refusal(
                &reverse(
                    &Canned {
                        nearby,
                        ..Canned::default()
                    },
                    &reverse_at(),
                )
                .unwrap_err(),
            )
            .1
            .map(str::to_owned)
        };
        for distance_m in [-1.0, f64::NAN, f64::INFINITY] {
            let nearby = NearbyPlaces {
                near: vec![near(distance_m, 0.0)],
                admin: Vec::new(),
            };
            assert_eq!(failing(nearby).as_deref(), Some("near[0].distanceM"));
        }
        for bearing_deg in [-0.1, 360.0, f64::NAN] {
            let nearby = NearbyPlaces {
                near: vec![near(1.0, bearing_deg)],
                admin: Vec::new(),
            };
            assert_eq!(failing(nearby).as_deref(), Some("near[0].bearingDeg"));
        }
        let unnamed = NearPlace {
            place: NamedPlace {
                id: String::new(),
                ..pokhara()
            },
            ..near(1.0, 1.0)
        };
        assert_eq!(
            failing(NearbyPlaces {
                near: vec![unnamed],
                admin: Vec::new()
            })
            .as_deref(),
            Some("near[0].place.id")
        );
        let unit = NamedPlace {
            level: None,
            ..nepal()
        };
        let why = reverse(
            &Canned {
                nearby: NearbyPlaces {
                    near: Vec::new(),
                    admin: vec![unit],
                },
                ..Canned::default()
            },
            &reverse_at(),
        )
        .unwrap_err();
        assert_eq!(refusal(&why), (Status::Provider, Some("admin[0].level")));
    }

    #[test]
    fn a_places_names_follow_the_languages_asked() {
        let asked = PlaceNamesRequest {
            id: "p1".into(),
            langs: vec!["ne".into(), "en".into()],
        };
        // A provider holding no names answers none for each language.
        let none = names(&Canned::default(), &asked).unwrap();
        assert_eq!(
            (none.id.as_str(), none.best.len(), none.all.len()),
            ("p1", 2, 0)
        );
        assert!(none.best.iter().all(|chosen| chosen.name.is_none()));

        let name = |text: &str| PlaceName {
            text: text.into(),
            lang: Some("ne".into()),
            script: Some("Deva".into()),
            kind: None,
            origin: "test".into(),
            machine: false,
        };
        let chosen = |asked: &str| ChosenName {
            asked: asked.into(),
            name: Some(name("पोखरा")),
        };
        let answer = |id: &str, best: Vec<ChosenName>, all: Vec<PlaceName>| Canned {
            names: Some(PlaceNames {
                id: id.into(),
                best,
                all,
            }),
            ..Canned::default()
        };
        let good = answer("p1", vec![chosen("ne"), chosen("en")], vec![name("पोखरा")]);
        assert_eq!(names(&&good, &asked).unwrap().all.len(), 1);
        assert_eq!(names(&Arc::new(good), &asked).unwrap().best.len(), 2);

        let failed = |canned: Canned| {
            refusal(&names(&canned, &asked).unwrap_err())
                .1
                .map(str::to_owned)
        };
        assert_eq!(
            failed(answer("p2", vec![chosen("ne"), chosen("en")], Vec::new())).as_deref(),
            Some("id")
        );
        assert_eq!(
            failed(answer("p1", vec![chosen("ne")], Vec::new())).as_deref(),
            Some("best")
        );
        assert_eq!(
            failed(answer("p1", vec![chosen("en"), chosen("ne")], Vec::new())).as_deref(),
            Some("best")
        );
        assert_eq!(
            failed(answer(
                "p1",
                vec![chosen("ne"), chosen("en")],
                vec![name("x"), name("")]
            ))
            .as_deref(),
            Some("all[1].text")
        );

        let why = names(
            &Canned::default(),
            &PlaceNamesRequest {
                id: " ".into(),
                langs: Vec::new(),
            },
        )
        .unwrap_err();
        assert_eq!(why.field(), Some("id"));
        let why = names(
            &Canned::default(),
            &PlaceNamesRequest {
                id: "p1".into(),
                langs: vec!["ne".into(), "??".into()],
            },
        )
        .unwrap_err();
        assert_eq!(why.field(), Some("langs[1]"));
    }

    /// A place by its id (SR-164): the request refused on its field, the
    /// survivor answered under its own id, and a malformed answer the
    /// provider's failure.
    #[test]
    fn a_place_by_its_id_is_the_survivor_checked() {
        let none = Canned::default();
        for (asked, field) in [
            (
                PlaceLookup {
                    id: " ".into(),
                    lang: None,
                },
                "id",
            ),
            (
                PlaceLookup {
                    id: "p0".into(),
                    lang: Some("1".into()),
                },
                "lang",
            ),
        ] {
            let why = place(&none, &asked).unwrap_err();
            assert_eq!((why.status, why.field()), (Status::InvalidArg, Some(field)));
        }
        let asked = PlaceLookup {
            id: "p0".into(),
            lang: None,
        };
        let unknown = place(&none, &asked).unwrap();
        assert_eq!((unknown.place, unknown.moved), (None, false));
        let survivor = Canned {
            looked_up: Some(pokhara()),
            ..Canned::default()
        };
        let found = place(&survivor, &asked).unwrap();
        assert!(found.moved);
        let found = found.place.unwrap();
        assert_eq!(
            (found.id.as_str(), found.zone.as_deref()),
            ("p1", Some("Asia/Kathmandu"))
        );
        let kept = PlaceLookup {
            id: "p1".into(),
            lang: None,
        };
        assert!(!place(&survivor, &kept).unwrap().moved);
        let malformed = Canned {
            looked_up: Some(NamedPlace {
                name: String::new(),
                ..pokhara()
            }),
            ..Canned::default()
        };
        let why = place(&malformed, &asked).unwrap_err();
        assert_eq!(
            (why.status, why.field()),
            (Status::Provider, Some("place.name"))
        );
    }

    #[test]
    fn a_label_says_the_country_once() {
        let parents = [parent("np", "Nepal", 0), parent("kaski", "Kaski", 2)];
        assert_eq!(label("Pokhara", &parents), "Pokhara, Kaski, Nepal");
        // The finest parent sharing the place's name is passed over.
        let city = [parent("np", "Nepal", 0), parent("ktm", "Kathmandu", 2)];
        assert_eq!(label("Kathmandu", &city), "Kathmandu, Nepal");
        // A country's own parent, itself, is not said twice.
        assert_eq!(label("Nepal", &[parent("np", "Nepal", 0)]), "Nepal");
        // Parents without a country end at the finest.
        assert_eq!(
            label("Pokhara", &[parent("kaski", "Kaski", 2)]),
            "Pokhara, Kaski"
        );
    }

    #[test]
    fn a_requests_defaults_are_its_serde_defaults() {
        let search: PlaceSearch = serde_json::from_str(
            r#"{"query": "x", "focus": {"latitudeDeg": 1, "longitudeDeg": 2}}"#,
        )
        .unwrap();
        assert_eq!(
            (search.limit, search.focus.unwrap().scale_m),
            (10, FOCUS_SCALE_M)
        );
        let reverse: PlaceReverse =
            serde_json::from_str(r#"{"latitudeDeg": 1, "longitudeDeg": 2}"#).unwrap();
        assert_eq!(reverse.limit, 5);
        let listing: PlaceListing = serde_json::from_str(r#"{"within": "NP"}"#).unwrap();
        assert_eq!(listing, PlaceListing::within("NP"));
        let any: PlaceListing = serde_json::from_str(r#"{"within": "NP", "depth": null}"#).unwrap();
        assert_eq!(any.depth, None);
    }
}
