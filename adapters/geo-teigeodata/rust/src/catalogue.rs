//! The place catalogue's questions answered from a teigeodata engine
//! (`docs/03-design/geo-port.md` §12): a search across scripts, the
//! nearest places and the units holding a point, a place's names by
//! language and script, and the places of a unit ranked by population.
//!
//! **Missing data is fewer answers.** A level a pack holds no outline for,
//! a place with no population, no ISO code or no parent, and a component a
//! pack was installed without (its outlines, its hierarchy, its
//! populations) leave that part of an answer out; they are never a
//! refusal (C2209). A refusal is a section that does not match its hash.

use std::collections::VecDeque;
use std::str::FromStr;

use teigeodata_core::{E6, E7, IsoCode, PlaceId, PlaceKind as Kind, Point};
use teigeodata_engine::{
    Engine, Error as EngineError, NameInfo, NameType, Origin, Place, PopulationSource, Scratch,
};
use teistro_core::error::{Error, Status};
use teistro_port_geo::{
    ChosenName, FoundPlace, GeoPoint, LangTag, NamedPlace, NearPlace, NearbyPlaces, ParentPlace,
    PlaceKind, PlaceListing, PlaceName, PlaceNames, PlaceNamesRequest, PlaceReverse, PlaceSearch,
    Population, PopulationKind, label,
};

use crate::{NAME_BYTES, failed};

/// How many more places a search or a reverse asks the engine for than it
/// answers, so that the timezone areas it leaves out do not shorten the
/// answer.
const SPARE: usize = 8;

/// The most places a listing visits below its unit.
const MOST_VISITED: usize = 200_000;

/// The port's kind of a teigeodata place, `None` for a place that is not
/// one of the catalogue's (a timezone area, a kind this reader does not
/// know).
fn kind_of(place: &Place) -> Option<PlaceKind> {
    match place.kind? {
        Kind::Country => Some(PlaceKind::Country),
        Kind::Admin => Some(PlaceKind::Admin),
        Kind::Settlement => Some(PlaceKind::Settlement),
        Kind::Locality => Some(PlaceKind::Locality),
        Kind::Natural => Some(PlaceKind::Natural),
        Kind::PostalArea => Some(PlaceKind::PostalArea),
        _ => None,
    }
}

/// `answer`, with a component that was not installed read as no answer.
fn optional<T>(answer: Result<Option<T>, EngineError>, field: &str) -> Result<Option<T>, Error> {
    match answer {
        Ok(found) => Ok(found),
        Err(EngineError::MissingComponent { .. }) => Ok(None),
        Err(why) => Err(failed(&why, field)),
    }
}

/// `filled` into a list, with a component that was not installed read as
/// none.
fn optional_list(
    filled: Result<(), EngineError>,
    out: &mut Vec<Place>,
    field: &str,
) -> Result<(), Error> {
    match filled {
        Ok(()) => Ok(()),
        Err(EngineError::MissingComponent { .. }) => {
            out.clear();
            Ok(())
        }
        Err(why) => Err(failed(&why, field)),
    }
}

/// One name of a place as teigeodata states it.
struct Written {
    text: String,
    info: NameInfo,
}

/// Every name of `place`, its canonical name first; none where its pack
/// has no names installed.
fn written(engine: &Engine, place: &Place, field: &str) -> Result<Vec<Written>, Error> {
    let mut buf = [0_u8; NAME_BYTES];
    let mut out = Vec::new();
    let read = engine.names(place, &mut buf, |text, info| {
        out.push(Written {
            text: text.to_owned(),
            info,
        });
    });
    match read {
        Ok(()) => Ok(out),
        Err(EngineError::MissingComponent { .. }) => Ok(Vec::new()),
        Err(why) => Err(failed(&why, field)),
    }
}

/// Which of `names` serves `lang` (§12.4), `None` where none does. A name
/// serves when the data tags it with the language (and the script, when
/// asked), or leaves its language undetermined and writes it in the
/// language's script; failing both, one in the language's script tagged
/// with another language. Among those, a reviewed name first, then a
/// person's before a program's, then by type (official, local, untyped,
/// another spelling, a historical name), then a tagged name before an
/// undetermined one, then the data's order.
fn chosen(names: &[Written], lang: &LangTag) -> Option<usize> {
    let script = lang.script_or_usual();
    names
        .iter()
        .enumerate()
        .filter_map(|(at, name)| {
            let name_script = name.info.script.map(|tag| tag.to_string());
            let in_script = script.is_some() && name_script.as_deref() == script;
            let (group, tier) = match name.info.lang {
                Some(tag) if tag.to_string() == lang.language => {
                    if lang.script.is_some() && name_script.as_deref() != lang.script.as_deref() {
                        return None;
                    }
                    (0, 0)
                }
                None if in_script => (0, 1),
                Some(_) if in_script => (1, 0),
                _ => return None,
            };
            let reviewed = u8::from(name.info.origin != Origin::Reviewed);
            let machine = u8::from(name.info.origin.is_machine());
            let kind = match name.info.kind {
                Some(NameType::Official) => 0,
                Some(NameType::Local) => 1,
                None => 2,
                Some(NameType::Alt | NameType::Colloquial) => 3,
                Some(_) => 4,
            };
            Some(((group, reviewed, machine, kind, tier, at), at))
        })
        .min_by_key(|(rank, _)| *rank)
        .map(|(_, at)| at)
}

/// The name `place` is shown by in `lang`, and the language the data tags
/// it with: the name chosen for `lang`, else the canonical one.
fn shown(
    engine: &Engine,
    place: &Place,
    lang: Option<&LangTag>,
    field: &str,
) -> Result<Option<(String, Option<String>)>, Error> {
    let names = written(engine, place, field)?;
    let at = lang.and_then(|lang| chosen(&names, lang)).unwrap_or(0);
    Ok(names
        .into_iter()
        .nth(at)
        .map(|name| (name.text, name.info.lang.map(|tag| tag.to_string()))))
}

/// The port's name of a teigeodata name type.
fn type_name(kind: NameType) -> &'static str {
    match kind {
        NameType::Official => "OFFICIAL",
        NameType::Local => "LOCAL",
        NameType::Alt => "ALT",
        NameType::Colloquial => "COLLOQUIAL",
        NameType::Historical => "HISTORICAL",
        NameType::FormerOfficial => "FORMER_OFFICIAL",
        _ => "OTHER",
    }
}

/// A teigeodata name as the port's.
fn place_name(name: &Written) -> PlaceName {
    PlaceName {
        text: name.text.clone(),
        lang: name.info.lang.map(|tag| tag.to_string()),
        script: name.info.script.map(|tag| tag.to_string()),
        kind: name.info.kind.map(|kind| String::from(type_name(kind))),
        origin: name.info.origin.name().to_ascii_uppercase(),
        machine: name.info.origin.is_machine(),
    }
}

/// The ISO 3166 code of `place`, as text.
fn code_of(engine: &Engine, place: &Place, field: &str) -> Result<Option<String>, Error> {
    Ok(optional(engine.iso_code(place), field)?.map(|code| code.as_str().to_owned()))
}

/// The admin units above `place`, coarse to fine, each named in `lang`.
fn parents(
    engine: &Engine,
    place: &Place,
    lang: Option<&LangTag>,
    field: &str,
) -> Result<Vec<ParentPlace>, Error> {
    let mut above = Vec::new();
    optional_list(engine.ancestors(place, &mut above), &mut above, field)?;
    let mut out = Vec::with_capacity(above.len());
    for parent in above.iter().rev() {
        if !kind_of(parent).is_some_and(PlaceKind::is_unit) {
            continue;
        }
        let Some((name, _)) = shown(engine, parent, lang, field)? else {
            continue;
        };
        out.push(ParentPlace {
            id: parent.id.to_string(),
            name,
            level: parent.admin_level,
            code: code_of(engine, parent, field)?,
        });
    }
    // The hierarchy lists units by parent, not by level: a level held out
    // of order across packs is dropped rather than answered out of order.
    let mut last = 0_u8;
    out.retain(|parent| {
        let keep = parent.level >= last;
        if keep {
            last = parent.level;
        }
        keep
    });
    Ok(out)
}

/// The number of people of `place`, and whether it was counted or
/// modelled.
fn population_of(engine: &Engine, place: &Place, field: &str) -> Result<Option<Population>, Error> {
    Ok(
        optional(engine.population(place), field)?.map(|population| Population {
            people: u64::from(population.people),
            kind: match population.source {
                Some(PopulationSource::Stated) => Some(PopulationKind::Counted),
                Some(PopulationSource::Modelled) => Some(PopulationKind::Modelled),
                _ => None,
            },
        }),
    )
}

/// `place` as a place of the catalogue, named in `lang`; `None` for a
/// place that is not one (a timezone area) or has no name.
fn named(
    engine: &Engine,
    place: &Place,
    lang: Option<&LangTag>,
    field: &str,
) -> Result<Option<NamedPlace>, Error> {
    let Some(kind) = kind_of(place) else {
        return Ok(None);
    };
    let Some((name, name_lang)) = shown(engine, place, lang, field)? else {
        return Ok(None);
    };
    let parents = parents(engine, place, lang, field)?;
    Ok(Some(NamedPlace {
        id: place.id.to_string(),
        label: label(&name, &parents),
        name,
        name_lang,
        kind,
        level: kind.is_unit().then_some(place.admin_level),
        code: code_of(engine, place, field)?,
        latitude_deg: place.point.lat_degrees(),
        longitude_deg: place.point.lon_degrees(),
        population: population_of(engine, place, field)?,
        parents,
    }))
}

/// The language a request names, read by the port's rule.
fn lang_of(lang: Option<&str>) -> Option<LangTag> {
    lang.and_then(LangTag::parse)
}

/// The engine's point for a request's degrees, on its 1e-6° grid.
fn point(latitude_deg: f64, longitude_deg: f64, field: &str) -> Result<Point<E6>, Error> {
    Point::from_degrees(latitude_deg, longitude_deg).map_err(|why| {
        Error::new(
            Status::Provider,
            format!("teigeodata does not place the point: {why}"),
        )
        .with_field(field.to_owned())
    })
}

/// A search, as teigeodata ranks it (§12.3).
#[cfg(feature = "search")]
pub(crate) fn search(
    engine: &Engine,
    scratch: &mut Scratch,
    asked: &PlaceSearch,
) -> Result<Vec<FoundPlace>, Error> {
    use teigeodata_engine::{Focus, SearchOptions};

    let mut options = SearchOptions::default();
    if let Some(focus) = asked.focus {
        options = options.with_focus(Focus {
            at: point(focus.latitude_deg, focus.longitude_deg, "focus")?,
            scale_m: focus.scale_m,
        });
    }
    let limit = asked.limit as usize;
    let mut hits = Vec::new();
    engine
        .search_with(&asked.query, limit + SPARE, &options, scratch, &mut hits)
        .map_err(|why| failed(&why, "search"))?;
    let lang = lang_of(asked.lang.as_deref());
    let mut found = Vec::with_capacity(limit);
    for hit in &hits {
        if found.len() == limit {
            break;
        }
        if let Some(place) = named(engine, &hit.place, lang.as_ref(), "search")? {
            found.push(FoundPlace {
                place,
                score: hit.score,
            });
        }
    }
    Ok(found)
}

/// No search: the adapter was built without its `search` feature.
#[cfg(not(feature = "search"))]
pub(crate) fn search(
    _: &Engine,
    _: &mut Scratch,
    _: &PlaceSearch,
) -> Result<Vec<FoundPlace>, Error> {
    Err(Error::new(
        Status::Capability,
        "this teigeodata adapter was built without its search",
    )
    .with_field("search")
    .with_hint("build `teistro-geo-teigeodata` with its `search` feature"))
}

/// The nearest named places, kept to the point's country, and the units
/// holding it (§12.5).
/// The admin units holding `at` into `out`, coarse to fine; `false` where
/// the packs hold no outline index to ask (a component not installed),
/// which is no answer rather than a refusal.
pub(crate) fn units_at(
    engine: &Engine,
    scratch: &mut Scratch,
    at: Point<E7>,
    out: &mut Vec<Place>,
) -> Result<bool, Error> {
    match engine.admin_at(at, scratch, out) {
        Ok(_) => Ok(true),
        Err(EngineError::MissingComponent { .. }) => {
            out.clear();
            Ok(false)
        }
        Err(why) => Err(failed(&why, "zone")),
    }
}

/// How many of the nearest places [`nearest_named`] looks through for a
/// settlement, a locality or a unit.
const NEAREST: usize = 8;

/// Where the settlement, locality or unit nearest `at` stands, in any
/// country; `None` where the packs name none.
pub(crate) fn nearest_named(
    engine: &Engine,
    scratch: &mut Scratch,
    at: GeoPoint,
) -> Result<Option<Point<E6>>, Error> {
    let from = point(at.latitude.get(), at.longitude.get(), "zone")?;
    let mut near = Vec::new();
    engine
        .reverse(from, NEAREST, scratch, &mut near)
        .map_err(|why| failed(&why, "zone"))?;
    Ok(near
        .iter()
        .find(|one| {
            matches!(
                one.place.kind,
                Some(Kind::Settlement | Kind::Locality | Kind::Admin | Kind::Country)
            )
        })
        .map(|one| one.place.point))
}

/// The timezone area of the settlement or locality nearest `at` that the
/// point lies within (`Engine::mark_inside`: 750 m of a settlement, 700 m
/// of a locality), from each place's own zone (`Engine::tz_of`); `None`
/// where the point lies within none, or the packs hold no place zones.
pub(crate) fn zone_within(
    engine: &Engine,
    scratch: &mut Scratch,
    at: GeoPoint,
) -> Result<Option<Place>, Error> {
    let from = point(at.latitude.get(), at.longitude.get(), "zone")?;
    let mut near = Vec::new();
    engine
        .reverse(from, NEAREST, scratch, &mut near)
        .map_err(|why| failed(&why, "zone"))?;
    near.retain(|one| matches!(one.place.kind, Some(Kind::Settlement | Kind::Locality)));
    match engine.mark_inside(from, scratch, &mut near) {
        Ok(()) => {}
        Err(EngineError::MissingComponent { .. }) => return Ok(None),
        Err(why) => return Err(failed(&why, "zone")),
    }
    let Some(within) = near.iter().find(|one| one.inside == Some(true)) else {
        return Ok(None);
    };
    match engine.tz_of(&within.place) {
        Ok(area) => Ok(area),
        Err(EngineError::MissingComponent { .. }) => Ok(None),
        Err(why) => Err(failed(&why, "zone")),
    }
}

pub(crate) fn reverse(
    engine: &Engine,
    scratch: &mut Scratch,
    asked: &PlaceReverse,
) -> Result<NearbyPlaces, Error> {
    let at = point(asked.latitude_deg, asked.longitude_deg, "latitudeDeg")?;
    let lang = lang_of(asked.lang.as_deref());
    let limit = asked.limit as usize;
    let mut near = Vec::new();
    engine
        .reverse_in_country(at, limit + SPARE, scratch, &mut near)
        .map_err(|why| failed(&why, "near"))?;
    match engine.mark_inside(at, scratch, &mut near) {
        Ok(()) | Err(EngineError::MissingComponent { .. }) => {}
        Err(why) => return Err(failed(&why, "near")),
    }
    let mut answer = NearbyPlaces::default();
    for one in &near {
        if answer.near.len() == limit {
            break;
        }
        if let Some(place) = named(engine, &one.place, lang.as_ref(), "near")? {
            answer.near.push(NearPlace {
                place,
                distance_m: one.meters,
                bearing_deg: one.bearing_deg,
                inside: one.inside,
            });
        }
    }
    let mut units = Vec::new();
    match engine.admin_at(at.to_e7(), scratch, &mut units) {
        Ok(_) => {}
        Err(EngineError::MissingComponent { .. }) => units.clear(),
        Err(why) => return Err(failed(&why, "admin")),
    }
    for unit in &units {
        if let Some(place) = named(engine, unit, lang.as_ref(), "admin")? {
            answer.admin.push(place);
        }
    }
    Ok(answer)
}

/// The place an id names, or `INVALID_ARG` on `field` for text that is
/// not a teigeodata id.
fn place_by_id(engine: &Engine, id: &str, field: &str) -> Result<Option<Place>, Error> {
    let id = PlaceId::from_str(id).map_err(|why| {
        Error::invalid_arg(format!("`{id}` is not a teigeodata place id: {why}"))
            .with_field(field.to_owned())
            .with_hint("an id an answer gave, such as `tgd:np/326`")
    })?;
    engine.get(id).map_err(|why| failed(&why, field))
}

/// A place's names, and one chosen for each language asked (§12.4).
pub(crate) fn names(engine: &Engine, asked: &PlaceNamesRequest) -> Result<PlaceNames, Error> {
    let Some(place) = place_by_id(engine, &asked.id, "id")? else {
        return Ok(PlaceNames::none(asked));
    };
    let all = written(engine, &place, "names")?;
    Ok(PlaceNames {
        id: asked.id.clone(),
        best: asked
            .langs
            .iter()
            .map(|lang| ChosenName {
                asked: lang.clone(),
                name: LangTag::parse(lang)
                    .and_then(|tag| chosen(&all, &tag))
                    .and_then(|at| all.get(at))
                    .map(place_name),
            })
            .collect(),
        all: all.iter().map(place_name).collect(),
    })
}

/// The unit a listing's `within` names: a teigeodata id, else an ISO 3166
/// code.
fn unit(engine: &Engine, within: &str) -> Result<Option<Place>, Error> {
    if let Ok(code) = IsoCode::from_str(within) {
        return optional(engine.by_iso(code), "within");
    }
    place_by_id(engine, within, "within")
}

/// The places below a unit matching the listing, ranked by population
/// (§12.6): the walk descends through countries and units, at most
/// `depth` levels and [`MOST_VISITED`] places.
pub(crate) fn list(engine: &Engine, asked: &PlaceListing) -> Result<Vec<NamedPlace>, Error> {
    let Some(within) = unit(engine, &asked.within)? else {
        return Ok(Vec::new());
    };
    let mut queue = VecDeque::from([(within, 0_usize)]);
    let mut visited = 0_usize;
    let mut children = Vec::new();
    let mut matched: Vec<(Option<u64>, String, Place)> = Vec::new();
    while let Some((at, depth)) = queue.pop_front() {
        if asked.depth.is_some_and(|most| depth >= usize::from(most)) {
            continue;
        }
        optional_list(engine.children(&at, &mut children), &mut children, "within")?;
        for child in &children {
            visited += 1;
            if visited > MOST_VISITED {
                break;
            }
            let Some(kind) = kind_of(child) else {
                continue;
            };
            // A former unit (a development region, a village development
            // committee) is history, not a place of the unit now.
            if child.flags & teigeodata_format::places::FLAG_FORMER != 0 {
                continue;
            }
            if kind.is_unit() {
                queue.push_back((*child, depth + 1));
            }
            let level_matches = asked
                .level
                .is_none_or(|level| kind.is_unit() && child.admin_level == level);
            let kind_matches = asked.kinds.is_empty() || asked.kinds.contains(&kind);
            if level_matches && kind_matches {
                let people = population_of(engine, child, "within")?.map(|found| found.people);
                let mut buf = [0_u8; NAME_BYTES];
                let canonical = optional(engine.name(child, &mut buf), "within")?
                    .unwrap_or_default()
                    .to_owned();
                matched.push((people, canonical, *child));
            }
        }
    }
    matched.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| a.1.cmp(&b.1))
            .then_with(|| a.2.id.cmp(&b.2.id))
    });
    let lang = lang_of(asked.lang.as_deref());
    let mut out = Vec::with_capacity(asked.limit as usize);
    for (_, _, place) in &matched {
        if out.len() == asked.limit as usize {
            break;
        }
        if let Some(place) = named(engine, place, lang.as_ref(), "within")? {
            out.push(place);
        }
    }
    Ok(out)
}
