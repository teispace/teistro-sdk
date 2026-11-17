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
