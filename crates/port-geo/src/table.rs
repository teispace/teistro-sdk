//! A [`GeoTable`]'s places: the small catalogue a table carries, which
//! answers the place catalogue's questions without a dataset
//! (`docs/03-design/geo-port.md` §12.8).

use std::collections::BTreeMap;

use teistro_core::error::Error;
use teistro_core::math::{asin, atan2, cos, sin};

use crate::catalogue::{
    ChosenName, FoundPlace, LangTag, NamedPlace, NearPlace, NearbyPlaces, ParentPlace, PlaceKind,
    PlaceListing, PlaceLookup, PlaceName, PlaceNames, PlaceNamesRequest, PlaceReverse, PlaceSearch,
    Population, label,
};
use crate::{GeoPoint, GeoTable, country_code, field_of, refused, subdivision_code};

/// The Earth's mean radius in metres, for the table's great-circle
/// distances.
const EARTH_RADIUS_M: f64 = 6_371_008.8;

/// What an exact match of a name scores in a table's search, a prefix
/// match being one less; a named parent adds [`PARENT_SCORE`].
const EXACT_SCORE: i32 = 2;

/// What a place whose parent the query names after a comma gains.
const PARENT_SCORE: i32 = 10;

/// One name of a table's place.
///
/// ```
/// use teistro_port_geo::TableName;
///
/// let name: TableName = serde_json::from_str(r#"{"text": "काठमाडौं", "lang": "ne", "script": "Deva"}"#)?;
/// assert_eq!(name.lang.as_deref(), Some("ne"));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TableName {
    /// The name.
    pub text: String,
    /// Its language, or `null`.
    #[serde(default)]
    pub lang: Option<String>,
    /// Its script, or `null`.
    #[serde(default)]
    pub script: Option<String>,
}

/// One place of a table's catalogue.
///
/// ```
/// use teistro_port_geo::{PlaceKind, TablePlace};
///
/// let place: TablePlace = serde_json::from_str(r#"{"id": "np", "names": [{"text": "Nepal"}],
///     "kind": "COUNTRY", "level": 0, "code": "NP", "latitudeDeg": 28.0, "longitudeDeg": 84.0}"#)?;
/// assert_eq!((place.kind, place.parent), (PlaceKind::Country, None));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TablePlace {
    /// Its id, unique in the table.
    pub id: String,
    /// Its names, the canonical one first; at least one.
    pub names: Vec<TableName>,
    /// What it is.
    pub kind: PlaceKind,
    /// Its admin level for a country or unit, else `null`.
    #[serde(default)]
    pub level: Option<u8>,
    /// Its ISO 3166 code, or `null`.
    #[serde(default)]
    pub code: Option<String>,
    /// Its latitude, in degrees.
    pub latitude_deg: f64,
    /// Its longitude, in degrees.
    pub longitude_deg: f64,
    /// The id of the unit it lies in, or `null` for a root.
    #[serde(default)]
    pub parent: Option<String>,
    /// Its number of people, or `null`.
    #[serde(default)]
    pub population: Option<Population>,
    /// Its IANA time zone, or `null`.
    #[serde(default)]
    pub zone: Option<String>,
    /// What a landmark is (`temple`), or `null`.
    #[serde(default)]
    pub class: Option<String>,
}

/// Lower case, with runs of white space as one space and none at the ends:
/// how a table's search compares a query with a name.
fn folded(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// The great-circle distance from `a` to `b` in metres, and the initial
/// bearing from `a` to `b` in degrees, in `[0, 360)`.
fn distance_and_bearing(from: (f64, f64), to: (f64, f64)) -> (f64, f64) {
    let (lat1, lon1) = (from.0.to_radians(), from.1.to_radians());
    let (lat2, lon2) = (to.0.to_radians(), to.1.to_radians());
    let (dlat, dlon) = (lat2 - lat1, lon2 - lon1);
    let (half_dlat, half_dlon) = (sin(dlat / 2.0), sin(dlon / 2.0));
    let haversine = half_dlat * half_dlat + cos(lat1) * cos(lat2) * half_dlon * half_dlon;
    let distance = 2.0 * EARTH_RADIUS_M * asin(haversine.sqrt().min(1.0));
    let east = sin(dlon) * cos(lat2);
    let north = cos(lat1) * sin(lat2) - sin(lat1) * cos(lat2) * cos(dlon);
    let bearing = atan2(east, north).to_degrees().rem_euclid(360.0);
    // `rem_euclid` can round a tiny negative angle up to exactly 360.
    (distance, if bearing >= 360.0 { 0.0 } else { bearing })
}

impl TablePlace {
    /// Refuses a place that is not well formed, under `root`.
    pub(crate) fn check(&self, root: &str) -> Result<(), Error> {
        if self.id.trim().is_empty() {
            return Err(refused(root, "id", String::from("a place has an id")));
        }
        if self.names.is_empty() || self.names.iter().any(|name| name.text.trim().is_empty()) {
            return Err(refused(
                root,
                "names",
                String::from("a place has at least one name, and no name is empty"),
            ));
        }
        for (at, name) in self.names.iter().enumerate() {
            if let Some(lang) = name.lang.as_deref()
                && LangTag::parse(lang).is_none()
            {
                return Err(refused(
                    root,
                    &format!("names[{at}].lang"),
                    format!("`{lang}` is not a language tag"),
                ));
            }
        }
        GeoPoint::try_from_degrees(self.latitude_deg, self.longitude_deg).map_err(|why| {
            let field = field_of(root, why.field().unwrap_or("latitudeDeg"));
            why.with_field(field)
        })?;
        if self.kind.is_unit() != self.level.is_some() {
            return Err(refused(
                root,
                "level",
                String::from("a country or unit has an admin level, and no other place has one"),
            ));
        }
        if let Some(code) = self.code.as_deref() {
            let shaped = if self.kind == PlaceKind::Country {
                country_code(code)
            } else {
                subdivision_code(code)
            };
            if !shaped {
                return Err(refused(
                    root,
                    "code",
                    format!("`{code}` is not an ISO 3166 code of this kind of place"),
                ));
            }
        }
        Ok(())
    }

    /// The name chosen for `lang` (§12.4): a name tagged with the language
    /// (and its script, when asked), else one in the language's script with
    /// no language, else none.
    fn name_in(&self, lang: &LangTag) -> Option<&TableName> {
        let script = lang.script_or_usual();
        let tagged = self.names.iter().find(|name| {
            name.lang
                .as_deref()
                .and_then(LangTag::parse)
                .is_some_and(|tag| tag.language == lang.language)
                && (lang.script.is_none() || name.script.as_deref() == lang.script.as_deref())
        });
        tagged.or_else(|| {
            self.names.iter().find(|name| {
                name.lang.is_none() && script.is_some() && name.script.as_deref() == script
            })
        })
    }

    /// The canonical name.
    fn canonical(&self) -> &TableName {
        // `check` refused a place with no name, and a table that read is checked.
        self.names.first().unwrap_or(&FALLBACK)
    }

    /// Its name in `lang`, else its canonical name.
    fn shown(&self, lang: Option<&LangTag>) -> &TableName {
        lang.and_then(|lang| self.name_in(lang))
            .unwrap_or_else(|| self.canonical())
    }
}

/// Never shown: a checked place has a first name.
static FALLBACK: TableName = TableName {
    text: String::new(),
    lang: None,
    script: None,
};

/// A table's places by id, and the reading of each answer.
struct Catalogue<'a> {
    places: &'a [TablePlace],
    by_id: BTreeMap<&'a str, &'a TablePlace>,
}

impl<'a> Catalogue<'a> {
    fn of(places: &'a [TablePlace]) -> Catalogue<'a> {
        Catalogue {
            places,
            by_id: places
                .iter()
                .map(|place| (place.id.as_str(), place))
                .collect(),
        }
    }

    /// The units above `place`, coarse to fine; a parent that is not a
    /// unit is passed through, and a cycle ends the walk.
    fn chain(&self, place: &TablePlace) -> Vec<&'a TablePlace> {
        let mut chain = Vec::new();
        let mut at = place.parent.as_deref();
        while let Some(id) = at {
            let Some(parent) = self.by_id.get(id).copied() else {
                break;
            };
            if parent.id == place.id || chain.iter().any(|seen: &&TablePlace| seen.id == parent.id)
            {
                break;
            }
            chain.push(parent);
            at = parent.parent.as_deref();
        }
        chain.retain(|parent| parent.level.is_some());
        chain.reverse();
        chain
    }

    fn named(&self, place: &TablePlace, lang: Option<&LangTag>) -> NamedPlace {
        let name = place.shown(lang);
        let parents: Vec<ParentPlace> = self
            .chain(place)
            .into_iter()
            .map(|parent| ParentPlace {
                id: parent.id.clone(),
                name: parent.shown(lang).text.clone(),
                level: parent.level.unwrap_or(0),
                code: parent.code.clone(),
            })
            .collect();
        NamedPlace {
            id: place.id.clone(),
            name: name.text.clone(),
            name_lang: name.lang.clone(),
            label: label(&name.text, &parents),
            kind: place.kind,
            level: place.level,
            code: place.code.clone(),
            latitude_deg: place.latitude_deg,
            longitude_deg: place.longitude_deg,
            population: place.population,
            parents,
            line: None,
            zone: place.zone.clone(),
            class: place.class.clone(),
        }
    }

    /// The place `within` names: an id, else an ISO code in either case.
    fn unit(&self, within: &str) -> Option<&'a TablePlace> {
        self.by_id.get(within).copied().or_else(|| {
            self.places.iter().find(|place| {
                place
                    .code
                    .as_deref()
                    .is_some_and(|code| code.eq_ignore_ascii_case(within))
            })
        })
    }
}

/// The people of a place for ranking, most first, a place with none last.
fn people(place: &TablePlace) -> Option<u64> {
    place.population.map(|population| population.people)
}

/// A table's places checked: each well formed, ids unique, each parent a
/// place of the table.
pub(crate) fn check_places(places: &[TablePlace]) -> Result<(), Error> {
    let mut seen = BTreeMap::new();
    for (at, place) in places.iter().enumerate() {
        let root = format!("geo.places[{at}]");
        place.check(&root)?;
        if seen.insert(place.id.as_str(), at).is_some() {
            return Err(refused(
                &root,
                "id",
                format!("the id `{}` is given twice", place.id),
            ));
        }
    }
    for (at, place) in places.iter().enumerate() {
        if let Some(parent) = place.parent.as_deref()
            && !seen.contains_key(parent)
        {
            return Err(refused(
                &format!("geo.places[{at}]"),
                "parent",
                format!("`{parent}` is not a place of the table"),
            ));
        }
    }
    Ok(())
}

impl GeoTable {
    /// The table's search (§12.8).
    pub(crate) fn search_places(&self, asked: &PlaceSearch) -> Vec<FoundPlace> {
        let catalogue = Catalogue::of(&self.places);
        let lang = asked.lang.as_deref().and_then(LangTag::parse);
        let (name, parent) = match asked.query.split_once(',') {
            Some((name, parent)) => (folded(name), Some(folded(parent))),
            None => (folded(&asked.query), None),
        };
        let mut found: Vec<(i32, Option<u64>, &TablePlace)> = self
            .places
            .iter()
            .filter_map(|place| {
                let best = place
                    .names
                    .iter()
                    .map(|written| folded(&written.text))
                    .filter_map(|written| {
                        if written == name {
                            Some(EXACT_SCORE)
                        } else if !name.is_empty() && written.starts_with(&name) {
                            Some(EXACT_SCORE - 1)
                        } else {
                            None
                        }
                    })
                    .max()?;
                let named_parent = parent.as_ref().is_some_and(|parent| {
                    catalogue.chain(place).iter().any(|unit| {
                        unit.names
                            .iter()
                            .any(|written| folded(&written.text).starts_with(parent.as_str()))
                    })
                });
                let score = best + if named_parent { PARENT_SCORE } else { 0 };
                Some((score, people(place), place))
            })
            .collect();
        found.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)).then(a.2.id.cmp(&b.2.id)));
        found
            .into_iter()
            .map(|(score, _, place)| FoundPlace {
                place: catalogue.named(place, lang.as_ref()),
                score,
            })
            .collect()
    }

    /// The table's reverse (§12.8): the nearest places, and the nearest
    /// place's chain of units.
    pub(crate) fn reverse_places(&self, asked: &PlaceReverse) -> NearbyPlaces {
        let catalogue = Catalogue::of(&self.places);
        let lang = asked.lang.as_deref().and_then(LangTag::parse);
        let point = (asked.latitude_deg, asked.longitude_deg);
        let mut near: Vec<(f64, f64, &TablePlace)> = self
            .places
            .iter()
            .map(|place| {
                let (distance, bearing) =
                    distance_and_bearing((place.latitude_deg, place.longitude_deg), point);
                (distance, bearing, place)
            })
            .collect();
        near.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.2.id.cmp(&b.2.id)));
        let admin = near.first().map_or_else(Vec::new, |(_, _, nearest)| {
            let mut units = catalogue.chain(nearest);
            if nearest.level.is_some() {
                units.push(nearest);
            }
            units
                .into_iter()
                .map(|unit| catalogue.named(unit, lang.as_ref()))
                .collect()
        });
        NearbyPlaces {
            near: near
                .into_iter()
                .take(asked.limit as usize)
                .map(|(distance_m, bearing_deg, place)| NearPlace {
                    place: catalogue.named(place, lang.as_ref()),
                    distance_m,
                    bearing_deg,
                    inside: None,
                })
                .collect(),
            admin,
        }
    }

    /// The place an id names in the table, which retires none.
    pub(crate) fn place_of(&self, asked: &PlaceLookup) -> Option<NamedPlace> {
        let catalogue = Catalogue::of(&self.places);
        let place = catalogue.by_id.get(asked.id.as_str()).copied()?;
        Some(catalogue.named(
            place,
            asked.lang.as_deref().and_then(LangTag::parse).as_ref(),
        ))
    }

    /// The table's names of a place, as written.
    pub(crate) fn place_names(&self, asked: &PlaceNamesRequest) -> PlaceNames {
        let Some(place) = self.places.iter().find(|place| place.id == asked.id) else {
            return PlaceNames::none(asked);
        };
        let written = |name: &TableName| PlaceName {
            text: name.text.clone(),
            lang: name.lang.clone(),
            script: name.script.clone(),
            kind: None,
            origin: String::from("TABLE"),
            machine: false,
        };
        PlaceNames {
            id: asked.id.clone(),
            best: asked
                .langs
                .iter()
                .map(|lang| ChosenName {
                    asked: lang.clone(),
                    name: LangTag::parse(lang)
                        .and_then(|tag| place.name_in(&tag))
                        .map(written),
                })
                .collect(),
            all: place.names.iter().map(written).collect(),
        }
    }

    /// The table's listing (§12.6).
    pub(crate) fn list_places(&self, asked: &PlaceListing) -> Vec<NamedPlace> {
        let catalogue = Catalogue::of(&self.places);
        let lang = asked.lang.as_deref().and_then(LangTag::parse);
        let Some(unit) = catalogue.unit(&asked.within) else {
            return Vec::new();
        };
        let mut listed: Vec<&TablePlace> = self
            .places
            .iter()
            .filter(|place| {
                let mut depth = 0_usize;
                let mut at = place.parent.as_deref();
                while let Some(id) = at {
                    depth += 1;
                    if depth > self.places.len() {
                        return false;
                    }
                    if id == unit.id {
                        return asked.depth.is_none_or(|most| depth <= usize::from(most));
                    }
                    at = catalogue
                        .by_id
                        .get(id)
                        .and_then(|parent| parent.parent.as_deref());
                }
                false
            })
            .filter(|place| asked.level.is_none_or(|level| place.level == Some(level)))
            .filter(|place| asked.kinds.is_empty() || asked.kinds.contains(&place.kind))
            .collect();
        listed.sort_by(|a, b| {
            people(b)
                .cmp(&people(a))
                .then_with(|| a.canonical().text.cmp(&b.canonical().text))
                .then_with(|| a.id.cmp(&b.id))
        });
        listed
            .into_iter()
            .map(|place| catalogue.named(place, lang.as_ref()))
            .collect()
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;
    use crate::{GeoProvider, PopulationKind, list, names, reverse, search};

    const TABLE: &str = r#"{
        "source": {"name": "test", "version": "1", "dataVersion": "2026.11.0", "tzdb": null},
        "areas": [],
        "places": [
            {"id": "np", "names": [{"text": "Nepal"}, {"text": "नेपाल", "lang": "ne", "script": "Deva"}],
             "kind": "COUNTRY", "level": 0, "code": "NP", "latitudeDeg": 28.0, "longitudeDeg": 84.0,
             "population": {"people": 29164578, "kind": "COUNTED"}},
            {"id": "np-p3", "names": [{"text": "Bagmati"}, {"text": "बागमती", "script": "Deva"}],
             "kind": "ADMIN", "level": 1, "code": "NP-P3", "latitudeDeg": 27.65, "longitudeDeg": 85.39, "parent": "np"},
            {"id": "ktm-district", "names": [{"text": "Kathmandu"}], "kind": "ADMIN", "level": 2,
             "latitudeDeg": 27.69, "longitudeDeg": 85.28, "parent": "np-p3",
             "population": {"people": 2007527, "kind": "MODELLED"}},
            {"id": "ktm", "names": [{"text": "Kathmandu"}, {"text": "काठमाडौं", "script": "Deva"}],
             "kind": "SETTLEMENT", "latitudeDeg": 27.7083, "longitudeDeg": 85.3206, "parent": "ktm-district",
             "population": {"people": 975000, "kind": "COUNTED"}},
            {"id": "pkr", "names": [{"text": "Pokhara"}, {"text": "पोखरा", "lang": "ne"}],
             "kind": "SETTLEMENT", "latitudeDeg": 28.2107, "longitudeDeg": 83.9863, "parent": "np"}
        ]
    }"#;

    fn table() -> GeoTable {
        GeoTable::from_json(TABLE).unwrap()
    }

    fn searched(query: &str, lang: Option<&str>) -> Vec<FoundPlace> {
        let asked = PlaceSearch {
            query: query.into(),
            focus: None,
            limit: 10,
            lang: lang.map(str::to_owned),
        };
        search(&table(), &asked).unwrap()
    }

    #[test]
    fn a_tables_search_finds_a_name_in_either_script_and_ranks_by_people() {
        let latin = searched("kathmandu", None);
        let ids: Vec<&str> = latin.iter().map(|hit| hit.place.id.as_str()).collect();
        // Both exact; the district's two million first.
        assert_eq!(ids, ["ktm-district", "ktm"]);
        let deva = searched("काठमाडौं", Some("ne"));
        assert_eq!(deva[0].place.id, "ktm");
        // A Devanagari name with no language serves `ne`, and says so.
        assert_eq!(
            (
                deva[0].place.name.as_str(),
                deva[0].place.name_lang.as_deref()
            ),
            ("काठमाडौं", None)
        );
        assert_eq!(deva[0].place.label, "काठमाडौं, Kathmandu, नेपाल");
        // A prefix is found below an exact match.
        let prefix = searched("Pok", None);
        assert_eq!((prefix[0].place.id.as_str(), prefix[0].score), ("pkr", 1));
        // A named parent outranks.
        let named = searched("Kathmandu, Bagm", None);
        assert!(named.iter().all(|hit| hit.score > PARENT_SCORE));
        assert_eq!(searched("Lalitpur", None), Vec::new());
        // The parents, coarse to fine, with their codes.
        let parents: Vec<_> = latin[1]
            .place
            .parents
            .iter()
            .map(|parent| (parent.level, parent.code.as_deref()))
            .collect();
        assert_eq!(parents, [(0, Some("NP")), (1, Some("NP-P3")), (2, None)]);
    }

    #[test]
    fn a_tables_reverse_names_the_nearest_and_its_units() {
        let asked = PlaceReverse {
            latitude_deg: 27.7172,
            longitude_deg: 85.324,
            limit: 2,
            lang: Some("ne-Deva-NP".into()),
        };
        let nearby = reverse(&table(), &asked).unwrap();
        assert_eq!(nearby.near.len(), 2);
        assert_eq!(nearby.near[0].place.id, "ktm");
        assert!((900.0..1_100.0).contains(&nearby.near[0].distance_m));
        // North-north-east of the city's point.
        assert!((10.0..30.0).contains(&nearby.near[0].bearing_deg));
        let admin: Vec<&str> = nearby.admin.iter().map(|unit| unit.id.as_str()).collect();
        assert_eq!(admin, ["np", "np-p3", "ktm-district"]);
        assert_eq!(nearby.admin[0].name, "नेपाल");
    }

    #[test]
    fn a_tables_names_and_listings() {
        let asked = PlaceNamesRequest {
            id: "pkr".into(),
            langs: vec!["ne".into(), "sa".into(), "en".into()],
        };
        let pokhara = names(&table(), &asked).unwrap();
        assert_eq!(pokhara.all.len(), 2);
        assert_eq!(pokhara.best[0].name.as_ref().unwrap().text, "पोखरा");
        // Tagged `ne`, so no name of Sanskrit's; none tagged English.
        assert!(pokhara.best[1].name.is_none() && pokhara.best[2].name.is_none());
        let unknown = PlaceNamesRequest {
            id: "nowhere".into(),
            langs: vec!["ne".into()],
        };
        assert_eq!(names(&table(), &unknown).unwrap().all, Vec::new());

        // A place by its id, named in the language asked (SR-164).
        let lookup = |id: &str| PlaceLookup {
            id: id.into(),
            lang: Some("ne".into()),
        };
        let found = crate::place(&table(), &lookup("pkr")).unwrap();
        assert!(!found.moved);
        let found = found.place.unwrap();
        assert_eq!((found.id.as_str(), found.name.as_str()), ("pkr", "पोखरा"));
        assert_eq!(
            crate::place(&table(), &lookup("nowhere")).unwrap().place,
            None
        );

        let listing = |json: &str| {
            let asked: PlaceListing = serde_json::from_str(json).unwrap();
            list(&table(), &asked)
                .unwrap()
                .into_iter()
                .map(|place| place.id)
                .collect::<Vec<_>>()
        };
        assert_eq!(listing(r#"{"within": "NP"}"#), ["np-p3", "pkr"]);
        assert_eq!(
            listing(r#"{"within": "np", "depth": null}"#),
            ["ktm-district", "ktm", "np-p3", "pkr"]
        );
        assert_eq!(
            listing(r#"{"within": "np", "depth": null, "level": 2}"#),
            ["ktm-district"]
        );
        assert_eq!(
            listing(r#"{"within": "np", "depth": null, "kinds": ["SETTLEMENT"]}"#),
            ["ktm", "pkr"]
        );
        assert_eq!(listing(r#"{"within": "IN"}"#), Vec::<String>::new());
        let ranked = list(
            &table(),
            &serde_json::from_str(r#"{"within": "np-p3", "depth": 2}"#).unwrap(),
        )
        .unwrap();
        assert_eq!(
            ranked[0].population.unwrap().kind,
            Some(PopulationKind::Modelled)
        );
    }

    #[test]
    fn a_request_is_refused_by_its_field_before_any_provider_is_asked() {
        let field = |why: Error| why.field().unwrap().to_owned();
        let mut asked = PlaceSearch {
            query: " ".into(),
            focus: None,
            limit: 10,
            lang: None,
        };
        assert_eq!(field(search(&table(), &asked).unwrap_err()), "query");
        asked.query = "x".into();
        asked.limit = 0;
        assert_eq!(field(search(&table(), &asked).unwrap_err()), "limit");
        asked.limit = 101;
        assert_eq!(field(search(&table(), &asked).unwrap_err()), "limit");
        asked.limit = 5;
        asked.lang = Some("नेपाली".into());
        assert_eq!(field(search(&table(), &asked).unwrap_err()), "lang");
        asked.lang = None;
        asked.focus = Some(crate::SearchFocus {
            latitude_deg: 95.0,
            longitude_deg: 0.0,
            scale_m: 1.0,
        });
        assert_eq!(
            field(search(&table(), &asked).unwrap_err()),
            "focus.latitudeDeg"
        );
        asked.focus = Some(crate::SearchFocus {
            latitude_deg: 5.0,
            longitude_deg: 0.0,
            scale_m: 0.0,
        });
        assert_eq!(field(search(&table(), &asked).unwrap_err()), "focus.scaleM");
        let far = PlaceReverse {
            latitude_deg: 0.0,
            longitude_deg: 181.0,
            limit: 1,
            lang: None,
        };
        assert_eq!(field(reverse(&table(), &far).unwrap_err()), "longitudeDeg");
        let blank = PlaceNamesRequest {
            id: String::new(),
            langs: vec![],
        };
        assert_eq!(field(names(&table(), &blank).unwrap_err()), "id");
        let bad_lang = PlaceNamesRequest {
            id: "np".into(),
            langs: vec!["ne".into(), "1".into()],
        };
        assert_eq!(field(names(&table(), &bad_lang).unwrap_err()), "langs[1]");
        let deep: PlaceListing = serde_json::from_str(r#"{"within": "np", "depth": 0}"#).unwrap();
        assert_eq!(field(list(&table(), &deep).unwrap_err()), "depth");
        let many: PlaceListing =
            serde_json::from_str(r#"{"within": "np", "limit": 1001}"#).unwrap();
        assert_eq!(field(list(&table(), &many).unwrap_err()), "limit");
    }

    #[test]
    fn a_tables_places_are_refused_by_the_part_that_is_wrong() {
        let refused = |places: &str| {
            GeoTable::from_json(&format!(
                r#"{{"source": {{"name": "t", "version": "1", "dataVersion": "1", "tzdb": null}}, "areas": [], "places": [{places}]}}"#
            ))
            .unwrap_err()
            .field()
            .unwrap()
            .to_owned()
        };
        let place = |extra: &str| {
            format!(
                r#"{{"id": "a", "names": [{{"text": "A"}}], "latitudeDeg": 1, "longitudeDeg": 1, {extra}}}"#
            )
        };
        assert_eq!(refused(&place(r#""kind": "ADMIN""#)), "geo.places[0].level");
        assert_eq!(
            refused(&place(r#""kind": "SETTLEMENT", "level": 1"#)),
            "geo.places[0].level"
        );
        assert_eq!(
            refused(&place(r#""kind": "COUNTRY", "level": 0, "code": "NP-P3""#)),
            "geo.places[0].code"
        );
        assert_eq!(
            refused(&place(r#""kind": "SETTLEMENT", "parent": "b""#)),
            "geo.places[0].parent"
        );
        assert_eq!(
            refused(&format!(
                "{}, {}",
                place(r#""kind": "NATURAL""#),
                place(r#""kind": "NATURAL""#)
            )),
            "geo.places[1].id"
        );
        assert_eq!(
            refused(
                r#"{"id": "a", "names": [], "kind": "NATURAL", "latitudeDeg": 1, "longitudeDeg": 1}"#
            ),
            "geo.places[0].names"
        );
        assert_eq!(
            refused(
                r#"{"id": "a", "names": [{"text": "A", "lang": "x"}], "kind": "NATURAL", "latitudeDeg": 1, "longitudeDeg": 1}"#
            ),
            "geo.places[0].names[0].lang"
        );
        assert_eq!(
            refused(
                r#"{"id": "a", "names": [{"text": "A"}], "kind": "NATURAL", "latitudeDeg": 91, "longitudeDeg": 1}"#
            ),
            "geo.places[0].latitudeDeg"
        );
        // A blank name, and a blank id.
        assert_eq!(
            refused(
                r#"{"id": "a", "names": [{"text": " "}], "kind": "NATURAL", "latitudeDeg": 1, "longitudeDeg": 1}"#
            ),
            "geo.places[0].names"
        );
        assert_eq!(
            refused(
                r#"{"id": " ", "names": [{"text": "A"}], "kind": "NATURAL", "latitudeDeg": 1, "longitudeDeg": 1}"#
            ),
            "geo.places[0].id"
        );
        // A table without places writes none.
        let bare = GeoTable::from_json(
            r#"{"source": {"name": "t", "version": "1", "dataVersion": "1", "tzdb": null}, "areas": []}"#,
        )
        .unwrap();
        assert!(!serde_json::to_string(&bare).unwrap().contains("places"));
    }

    /// A provider whose catalogue answers whatever a test sets.
    struct Wrong(NamedPlace);

    impl GeoProvider for Wrong {
        fn source(&self) -> crate::GeoSource {
            table().source
        }
        fn zone_at(&self, _: GeoPoint) -> Result<Option<String>, Error> {
            Ok(None)
        }
        fn height_at(&self, _: GeoPoint) -> Result<Option<crate::GroundHeight>, Error> {
            Ok(None)
        }
        fn admin_at(&self, _: GeoPoint) -> Result<Vec<crate::AdminUnit>, Error> {
            Ok(Vec::new())
        }
        fn search(&self, _: &PlaceSearch) -> Result<Vec<FoundPlace>, Error> {
            Ok(vec![FoundPlace {
                place: self.0.clone(),
                score: 1,
            }])
        }
        fn reverse(&self, _: &PlaceReverse) -> Result<NearbyPlaces, Error> {
            Ok(NearbyPlaces {
                near: vec![NearPlace {
                    place: self.0.clone(),
                    distance_m: -1.0,
                    bearing_deg: 0.0,
                    inside: None,
                }],
                admin: Vec::new(),
            })
        }
        fn names(&self, _: &PlaceNamesRequest) -> Result<PlaceNames, Error> {
            Ok(PlaceNames {
                id: String::from("another"),
                best: Vec::new(),
                all: Vec::new(),
            })
        }
    }

    #[test]
    fn a_providers_malformed_catalogue_answer_is_its_failure() {
        let good = searched("Pokhara", None).remove(0).place;
        let failed = |place: NamedPlace| {
            let asked = PlaceSearch {
                query: "x".into(),
                focus: None,
                limit: 1,
                lang: None,
            };
            let why = search(&Wrong(place), &asked).unwrap_err();
            (why.status, why.field().unwrap().to_owned())
        };
        let provider = |field: &str| (teistro_core::error::Status::Provider, field.to_owned());
        assert_eq!(
            failed(NamedPlace {
                code: Some("np".into()),
                ..good.clone()
            }),
            provider("[0].place.code")
        );
        assert_eq!(
            failed(NamedPlace {
                level: Some(1),
                ..good.clone()
            }),
            provider("[0].place.level")
        );
        assert_eq!(
            failed(NamedPlace {
                longitude_deg: 200.0,
                ..good.clone()
            }),
            provider("[0].place.longitudeDeg")
        );
        assert_eq!(
            failed(NamedPlace {
                name: String::new(),
                ..good.clone()
            }),
            provider("[0].place.name")
        );
        let mut backwards = good.clone();
        backwards.parents = vec![
            ParentPlace {
                id: "b".into(),
                name: "B".into(),
                level: 2,
                code: None,
            },
            ParentPlace {
                id: "a".into(),
                name: "A".into(),
                level: 1,
                code: None,
            },
        ];
        assert_eq!(failed(backwards), provider("[0].place.parents[1].level"));
        let far = PlaceReverse {
            latitude_deg: 0.0,
            longitude_deg: 0.0,
            limit: 1,
            lang: None,
        };
        let why = reverse(&Wrong(good.clone()), &far).unwrap_err();
        assert_eq!(why.field(), Some("near[0].distanceM"));
        let other = PlaceNamesRequest {
            id: "pkr".into(),
            langs: vec![],
        };
        assert_eq!(names(&Wrong(good), &other).unwrap_err().field(), Some("id"));
    }

    #[test]
    fn a_borrowed_or_shared_provider_answers_the_catalogue_as_the_provider() {
        fn answers<P: GeoProvider>(provider: &P) -> String {
            let asked = PlaceSearch {
                query: "Kathmandu".into(),
                focus: None,
                limit: 3,
                lang: None,
            };
            let near = PlaceReverse {
                latitude_deg: 28.0,
                longitude_deg: 84.0,
                limit: 1,
                lang: None,
            };
            let named = PlaceNamesRequest {
                id: "np".into(),
                langs: vec!["ne".into()],
            };
            let listing: PlaceListing = serde_json::from_str(r#"{"within": "np"}"#).unwrap();
            format!(
                "{:?} {:?} {:?} {:?}",
                provider.search(&asked),
                provider.reverse(&near),
                provider.names(&named),
                provider.list(&listing)
            )
        }
        let table = table();
        assert_eq!(answers(&&table), answers(&table));
        assert_eq!(
            answers(&std::sync::Arc::new(table.clone())),
            answers(&table)
        );
        // A provider with no catalogue of its own answers the defaults.
        let wrong = Wrong(searched("Pokhara", None).remove(0).place);
        let listing: PlaceListing = serde_json::from_str(r#"{"within": "np"}"#).unwrap();
        assert_eq!(wrong.list(&listing).unwrap(), Vec::new());
    }

    #[test]
    fn a_cycle_of_parents_ends_and_a_tie_ranks_by_name_then_id() {
        let cyclic = GeoTable::from_json(
            r#"{"source": {"name": "t", "version": "1", "dataVersion": "1", "tzdb": null}, "areas": [], "places": [
                {"id": "a", "names": [{"text": "Alpha"}], "kind": "ADMIN", "level": 1, "latitudeDeg": 1, "longitudeDeg": 1, "parent": "b"},
                {"id": "b", "names": [{"text": "Beta"}], "kind": "ADMIN", "level": 1, "latitudeDeg": 1, "longitudeDeg": 1, "parent": "a"},
                {"id": "d", "names": [{"text": "Same"}], "kind": "NATURAL", "latitudeDeg": 1, "longitudeDeg": 1, "parent": "a"},
                {"id": "c", "names": [{"text": "Same"}], "kind": "NATURAL", "latitudeDeg": 1, "longitudeDeg": 1, "parent": "a"}
            ]}"#,
        )
        .unwrap();
        // A place's parents stop where the chain comes back to it.
        let alpha = search(
            &cyclic,
            &PlaceSearch {
                query: "Alpha".into(),
                focus: None,
                limit: 1,
                lang: None,
            },
        )
        .unwrap();
        let parents: Vec<&str> = alpha[0]
            .place
            .parents
            .iter()
            .map(|p| p.id.as_str())
            .collect();
        assert_eq!(parents, ["b"]);
        // A chain that loops without reaching the unit is not below it.
        let none: PlaceListing = serde_json::from_str(r#"{"within": "c", "depth": null}"#).unwrap();
        assert_eq!(list(&cyclic, &none).unwrap(), Vec::new());
        // No people on either: the same name, then the id, ranks them.
        let kept: PlaceListing =
            serde_json::from_str(r#"{"within": "a", "kinds": ["NATURAL"]}"#).unwrap();
        let ids: Vec<String> = list(&cyclic, &kept)
            .unwrap()
            .into_iter()
            .map(|p| p.id)
            .collect();
        assert_eq!(ids, ["c", "d"]);
    }
}
