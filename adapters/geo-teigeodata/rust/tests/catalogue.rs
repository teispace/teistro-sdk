//! The place catalogue over real teigeodata packs (Nepal's country pack and
//! the world's baseline, named by `TEISTRO_TEIGEODATA_PACKS`): a search
//! across scripts, a point named to its units, a place's names by language,
//! a unit's places by population, and a pack that holds no outline for a
//! point answering fewer units rather than refusing.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::print_stderr,
    reason = "tests fail by panicking, and print what they measured where the runner shows it"
)]

mod common;

use std::sync::Arc;

use teigeodata_engine::{Engine, Layer, LayerOptions, Shared};
use teigeodata_format::component::Component;
use teistro_core::error::Status;
use teistro_geo_teigeodata::{NO_PACKS, TeigeodataGeo};
use teistro_port_geo::{
    FoundPlace, GeoPoint, GeoProvider, NamedPlace, PlaceKind, PlaceListing, PlaceLookup,
    PlaceNamesRequest, PlaceReverse, PlaceSearch, PopulationKind, SearchFocus, list, names, place,
    reverse, search,
};

fn searched(geo: &TeigeodataGeo, query: &str, lang: Option<&str>) -> Vec<FoundPlace> {
    let asked = PlaceSearch {
        query: query.into(),
        focus: None,
        limit: 5,
        lang: lang.map(str::to_owned),
    };
    search(geo, &asked).unwrap()
}

fn summary(found: &[FoundPlace]) -> Vec<(String, PlaceKind, i32)> {
    found
        .iter()
        .take(3)
        .map(|hit| (hit.place.id.clone(), hit.place.kind, hit.score))
        .collect()
}

fn reversed(
    geo: &TeigeodataGeo,
    lat: f64,
    lon: f64,
    lang: Option<&str>,
) -> teistro_port_geo::NearbyPlaces {
    let asked = PlaceReverse {
        latitude_deg: lat,
        longitude_deg: lon,
        limit: 5,
        lang: lang.map(str::to_owned),
    };
    reverse(geo, &asked).unwrap()
}

/// What a pack is, by its file name: the world's baseline, teigeodata's
/// app pack (the world and Nepal in one, `world-np-app-lite.tgd`), or a
/// country's pack.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Pack {
    Baseline,
    App,
    Country,
}

fn pack_of(path: &std::path::Path) -> Pack {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    if name.contains("baseline") {
        Pack::Baseline
    } else if name.contains("-app") {
        Pack::App
    } else {
        Pack::Country
    }
}

/// The packs `TEISTRO_TEIGEODATA_PACKS` names of one kind.
fn packs_of(test: &str, kind: Pack) -> Option<Vec<std::path::PathBuf>> {
    let packs: Vec<_> = common::packs(test)?
        .into_iter()
        .filter(|pack| pack_of(pack) == kind)
        .collect();
    if packs.is_empty() {
        eprintln!("SKIP  {test}: no pack of that kind is named");
        return None;
    }
    Some(packs)
}

#[test]
fn three_spellings_in_two_scripts_find_the_same_places_in_the_same_order() {
    let Some(geo) = common::provider("three spellings") else {
        return;
    };
    let deva = searched(&geo, "काठमाडौं", None);
    let latin = searched(&geo, "Kathmandu", None);
    let typo = searched(&geo, "Kathamandu", None);
    eprintln!("काठमाडौं   {:?}", summary(&deva));
    eprintln!("Kathmandu  {:?}", summary(&latin));
    eprintln!("Kathamandu {:?}", summary(&typo));
    let ids = |found: &[FoundPlace]| -> Vec<String> {
        found
            .iter()
            .take(3)
            .map(|hit| hit.place.id.clone())
            .collect()
    };
    assert_eq!(ids(&deva), ids(&latin));
    assert_eq!(ids(&typo), ids(&latin));
    // The first is Kathmandu, whatever its kind, in Nepal and Bagmati.
    for hit in latin.iter().take(3) {
        assert_eq!(hit.place.name, "Kathmandu", "{:?}", hit.place);
        let codes: Vec<_> = hit
            .place
            .parents
            .iter()
            .filter_map(|unit| unit.code.as_deref())
            .collect();
        assert!(
            codes.starts_with(&["NP", "NP-P3"]),
            "{:?}",
            hit.place.parents
        );
        assert!(hit.place.label.ends_with(", Nepal"), "{}", hit.place.label);
    }
    // A typo scores below the exact spelling, a named parent above it.
    assert!(typo[0].score < latin[0].score);
    let parented = searched(&geo, "Kathmandu, Bagmati", None);
    assert_eq!(ids(&parented), ids(&latin));
    assert!(parented[0].score >= latin[0].score + 1_000);
    // Asked in Nepali, the first is named in Devanagari.
    let nepali = searched(&geo, "Kathmandu", Some("ne"));
    assert!(
        nepali[0]
            .place
            .name
            .chars()
            .any(|c| ('\u{0900}'..='\u{097f}').contains(&c)),
        "{:?}",
        nepali[0].place
    );
    // No timezone area is a place of the catalogue.
    let london = searched(&geo, "Europe/London", None);
    assert!(
        london
            .iter()
            .all(|hit| hit.place.kind != PlaceKind::Admin || hit.place.level.is_some())
    );
}

#[test]
fn a_focus_ranks_the_pokhara_nearby_first() {
    let Some(geo) = common::provider("focus") else {
        return;
    };
    // Dang's Pokhara, a village near Ghorahi, against the city.
    let ghorahi = SearchFocus {
        latitude_deg: 28.04,
        longitude_deg: 82.49,
        scale_m: 20_000.0,
    };
    let plain = searched(&geo, "Pokhara", None);
    let asked = PlaceSearch {
        query: "Pokhara".into(),
        focus: Some(ghorahi),
        limit: 5,
        lang: None,
    };
    let focused = search(&geo, &asked).unwrap();
    eprintln!("Pokhara           {:?}", summary(&plain));
    eprintln!("Pokhara at Ghorahi {:?}", summary(&focused));
    // The focus adds up to 30 and never lifts a weaker match over the
    // city's importance (teigeodata's D-226): Dang's Pokhara gains, and
    // passes every other plain Pokhara, while the city stays first.
    let dang = |found: &[FoundPlace]| {
        found
            .iter()
            .find(|hit| hit.place.longitude_deg < 83.0)
            .map(|hit| hit.score)
            .unwrap()
    };
    assert!(
        dang(&focused) >= dang(&plain) + 15,
        "{} {}",
        dang(&focused),
        dang(&plain)
    );
    assert_eq!(focused[0].place.id, plain[0].place.id);
    assert!(
        focused[2].place.longitude_deg < 83.0,
        "{:?}",
        focused[2].place
    );
}

#[test]
fn a_point_in_kathmandu_is_named_to_its_finest_unit() {
    let Some(geo) = common::provider("reverse kathmandu") else {
        return;
    };
    let nearby = reversed(&geo, 27.7172, 85.324, Some("en"));
    for near in &nearby.near {
        eprintln!(
            "near {} {:?} {:.0} m inside {:?}",
            near.place.name, near.place.kind, near.distance_m, near.inside
        );
    }
    for unit in &nearby.admin {
        eprintln!("admin {} level {:?} {:?}", unit.name, unit.level, unit.code);
    }
    assert!(!nearby.near.is_empty() && nearby.near.len() <= 5);
    assert!(
        nearby
            .near
            .windows(2)
            .all(|pair| pair[0].distance_m <= pair[1].distance_m)
    );
    assert!(nearby.near[0].distance_m < 1_000.0);
    let levels: Vec<_> = nearby.admin.iter().map(|unit| unit.level).collect();
    assert_eq!(&levels[..2], &[Some(0), Some(1)]);
    assert_eq!(nearby.admin[0].code.as_deref(), Some("NP"));
    assert_eq!(nearby.admin[1].code.as_deref(), Some("NP-P3"));
    // The chain reaches below the province: the district, then the city
    // (the ward where the pack carries ward outlines).
    // Named in English, the country is `Nepal`, not another spelling.
    assert_eq!(nearby.admin[0].name, "Nepal");
    let finest = nearby.admin.last().unwrap();
    assert!(finest.level >= Some(3), "{:?}", nearby.admin);
    assert_eq!(finest.name, "Kathmandu");
    // Each unit's parents are the units before it.
    let ids: Vec<_> = nearby.admin.iter().map(|unit| unit.id.clone()).collect();
    let above: Vec<_> = finest
        .parents
        .iter()
        .map(|parent| parent.id.clone())
        .collect();
    assert_eq!(above, ids[..ids.len() - 1]);
}

#[test]
fn a_point_abroad_is_named_from_the_worlds_coarse_units() {
    let Some(geo) = common::provider("reverse london") else {
        return;
    };
    let nearby = reversed(&geo, 51.5074, -0.1278, None);
    let Some(first) = nearby.admin.first() else {
        eprintln!("SKIP  reverse london: no pack holds London's outlines (name the baseline)");
        return;
    };
    eprintln!(
        "London: near {:?}, admin {:?}",
        nearby
            .near
            .first()
            .map(|near| (&near.place.name, near.distance_m.round())),
        nearby
            .admin
            .iter()
            .map(|unit| (&unit.name, unit.code.as_deref()))
            .collect::<Vec<_>>()
    );
    assert_eq!(first.code.as_deref(), Some("GB"));
    assert!(nearby.admin.iter().all(|unit| unit.level <= Some(1)));
    assert_eq!(nearby.near[0].place.name, "London");
    assert_eq!(nearby.near[0].inside, Some(true));
}

/// teigeodata's app pack (TG-26) keeps Nepal's outlines to the ward and
/// none abroad: a point abroad has no unit there, or only the world's
/// levels 0 and 1, and that is fewer answers, never a refusal (C2209).
#[test]
fn a_pack_with_no_outline_for_a_point_answers_fewer_units_not_a_refusal() {
    let Some(country) = packs_of("no outline abroad", Pack::Country) else {
        return;
    };
    let nepal = TeigeodataGeo::open(&country).unwrap();
    // London against Nepal's pack alone: no unit, and no refusal.
    let london = GeoPoint::try_from_degrees(51.5074, -0.1278).unwrap();
    assert_eq!(nepal.admin_at(london).unwrap(), Vec::new());
    let place = teistro_port_geo::resolve(&nepal, london).unwrap();
    assert_eq!(place.admin, Vec::new());
    let nearby = reversed(&nepal, 51.5074, -0.1278, None);
    assert_eq!(nearby.admin, Vec::new());
    // Nepal's pack names no place in England: the nearest it has is far.
    assert!(nearby.near.iter().all(|near| near.distance_m > 1_000_000.0));
    let listing: PlaceListing = serde_json::from_str(r#"{"within": "GB"}"#).unwrap();
    assert_eq!(list(&nepal, &listing).unwrap(), Vec::new());

    // Kathmandu against the world's baseline alone: its coarse units only.
    let Some(world) = packs_of("coarse units only", Pack::Baseline) else {
        return;
    };
    let coarse = TeigeodataGeo::open(&world).unwrap();
    let units = reversed(&coarse, 27.7172, 85.324, None).admin;
    eprintln!(
        "Kathmandu on the baseline alone: {:?}",
        units
            .iter()
            .map(|unit| (&unit.name, unit.level))
            .collect::<Vec<_>>()
    );
    assert!(units.iter().all(|unit| unit.level <= Some(1)), "{units:?}");
    let kathmandu = GeoPoint::try_from_degrees(27.7172, 85.324).unwrap();
    assert!(
        coarse
            .admin_at(kathmandu)
            .unwrap()
            .iter()
            .all(|unit| unit.level <= 1)
    );
}

/// At sea a resolution answers the nautical zone, marked so, and never
/// fails (SR-144); a pack that knows only its own country cannot tell the
/// sea from land it lacks, so it answers no zone there (C2216).
#[test]
fn the_sea_answers_its_nautical_zone_and_a_country_pack_does_not_guess() {
    let Some(world) = packs_of("nautical zone", Pack::Baseline) else {
        return;
    };
    let coarse = TeigeodataGeo::open(&world).unwrap();
    let at = |lat, lon| GeoPoint::try_from_degrees(lat, lon).unwrap();
    for (lat, lon, zone) in [
        (0.0, -30.0, "Etc/GMT+2"),
        (15.0, 88.0, "Etc/GMT-6"),
        (-20.0, -140.0, "Etc/GMT+9"),
        (-10.0, 75.0, "Etc/GMT-5"),
    ] {
        assert!(coarse.at_sea(at(lat, lon)).unwrap(), "{lat}, {lon}");
        let place = teistro_port_geo::resolve(&coarse, at(lat, lon)).unwrap();
        assert_eq!((place.zone.as_deref(), place.nautical), (Some(zone), true));
        assert_eq!(place.admin, Vec::new());
    }
    // Land keeps its land zone, unmarked.
    for (lat, lon, zone) in [
        (51.5074, -0.1278, "Europe/London"),
        (27.7172, 85.324, "Asia/Kathmandu"),
    ] {
        assert!(!coarse.at_sea(at(lat, lon)).unwrap());
        let place = teistro_port_geo::resolve(&coarse, at(lat, lon)).unwrap();
        assert_eq!((place.zone.as_deref(), place.nautical), (Some(zone), false));
    }
    let Some(country) = packs_of("no world, no sea", Pack::Country) else {
        return;
    };
    let nepal = TeigeodataGeo::open(&country).unwrap();
    for (lat, lon) in [(0.0, -30.0), (51.5074, -0.1278)] {
        assert!(!nepal.at_sea(at(lat, lon)).unwrap());
        let place = teistro_port_geo::resolve(&nepal, at(lat, lon)).unwrap();
        assert_eq!((place.zone, place.nautical), (None, false));
    }
}

/// teigeodata's app pack, alone as the apps ship it first: Nepal outlined
/// to its local levels, the world's places without outlines. Abroad there
/// is no unit and no sea, and a point within a settlement takes that
/// settlement's zone (C2218, C2219).
#[test]
fn the_app_pack_outlines_nepal_alone_and_zones_a_town_abroad() {
    let Some(app) = packs_of("app pack", Pack::App) else {
        return;
    };
    let geo = TeigeodataGeo::open(&app).unwrap();
    let at = |lat, lon| GeoPoint::try_from_degrees(lat, lon).unwrap();
    // London: its own zone, from the settlement it lies within, no unit.
    let london = teistro_port_geo::resolve(&geo, at(51.5074, -0.1278)).unwrap();
    assert_eq!(
        (london.zone.as_deref(), london.nautical),
        (Some("Europe/London"), false)
    );
    assert_eq!(london.admin, Vec::new());
    let nearby = reversed(&geo, 51.5074, -0.1278, None);
    assert_eq!(nearby.admin, Vec::new());
    assert_eq!(nearby.near[0].place.name, "London");
    // Mid-Atlantic: the pack outlines no land there, so it cannot call
    // it sea; no zone, no refusal.
    for (lat, lon) in [(0.0, -30.0), (15.0, 88.0)] {
        assert!(!geo.at_sea(at(lat, lon)).unwrap());
        let place = teistro_port_geo::resolve(&geo, at(lat, lon)).unwrap();
        assert_eq!((place.zone, place.nautical), (None, false));
    }
    // Nepal to its local level, as the main pack outlines it.
    let kathmandu = teistro_port_geo::resolve(&geo, at(27.7172, 85.324)).unwrap();
    assert_eq!(kathmandu.zone.as_deref(), Some("Asia/Kathmandu"));
    let levels: Vec<_> = kathmandu.admin.iter().map(|unit| unit.level).collect();
    assert_eq!(levels, [0, 1, 2, 3]);
    // No DEM and no populations in this pack.
    assert_eq!(kathmandu.height, None);
    assert!(!common::holds(&geo, Component::Extra));
}

#[test]
fn a_places_names_are_chosen_per_language_with_a_fallback_to_its_script() {
    let Some(geo) = common::provider("names") else {
        return;
    };
    let city = searched(&geo, "Pokhara", None)
        .into_iter()
        .find(|hit| hit.place.kind == PlaceKind::Admin)
        .expect("Pokhara metropolitan city")
        .place;
    let asked = PlaceNamesRequest {
        id: city.id.clone(),
        langs: vec!["ne".into(), "en".into(), "hi".into(), "sa".into()],
    };
    let answer = names(&geo, &asked).unwrap();
    for chosen in &answer.best {
        eprintln!("{} {:?}", chosen.asked, chosen.name);
    }
    assert_ne!(answer.all, Vec::new());
    assert_eq!(answer.best.len(), 4);
    let nepali = answer.best[0].name.as_ref().unwrap();
    assert_eq!(nepali.text, "पोखरा");
    // Hindi and Sanskrit find a Devanagari name, by tag or by script.
    for chosen in &answer.best[2..] {
        let name = chosen.name.as_ref().unwrap();
        assert_eq!(name.script.as_deref(), Some("Deva"), "{chosen:?}");
    }
    // Every name says where it came from.
    assert!(answer.all.iter().all(|name| !name.origin.is_empty()));
    // An unknown id answers no names; a malformed one is the caller's.
    let unknown = PlaceNamesRequest {
        id: "tgd:np/zzzzzz".into(),
        langs: vec!["ne".into()],
    };
    assert_eq!(names(&geo, &unknown).unwrap().all, Vec::new());
    let malformed = PlaceNamesRequest {
        id: "Pokhara".into(),
        langs: vec![],
    };
    let why = names(&geo, &malformed).unwrap_err();
    assert_eq!((why.status, why.field()), (Status::InvalidArg, Some("id")));
}

/// A saved place read back by its id (SR-164): its record named as asked,
/// with its zone, not moved; a retired id answered with the place that
/// replaced it, where the packs carry the redirect (teigeodata D-382).
#[test]
fn a_place_is_read_back_by_its_id_and_a_retired_id_follows_its_survivor() {
    let Some(geo) = common::provider("place") else {
        return;
    };
    let city = searched(&geo, "Pokhara", None)
        .into_iter()
        .find(|hit| hit.place.kind == PlaceKind::Admin)
        .expect("Pokhara metropolitan city")
        .place;
    let asked = PlaceLookup {
        id: city.id.clone(),
        lang: Some("ne".into()),
    };
    let found = place(&geo, &asked).unwrap();
    assert!(!found.moved);
    let read = found.place.unwrap();
    assert_eq!(
        (read.id.as_str(), read.name.as_str()),
        (city.id.as_str(), "पोखरा")
    );
    assert_eq!(read.zone.as_deref(), Some("Asia/Kathmandu"));
    assert_eq!(
        (read.latitude_deg, read.longitude_deg),
        (city.latitude_deg, city.longitude_deg)
    );
    let unknown = PlaceLookup {
        id: "tgd:np/zzzzzz".into(),
        lang: None,
    };
    assert_eq!(place(&geo, &unknown).unwrap().place, None);
    let malformed = PlaceLookup {
        id: "Pokhara".into(),
        lang: None,
    };
    let why = place(&geo, &malformed).unwrap_err();
    assert_eq!((why.status, why.field()), (Status::InvalidArg, Some("id")));
    // Gundu gaun's id, retired into Gundu's in Nepal's 2026.11.2 build.
    let retired = PlaceLookup {
        id: "tgd:np/54s".into(),
        lang: None,
    };
    match place(&geo, &retired).unwrap() {
        found if found.moved => {
            assert_eq!(
                found.place.map(|place| place.id).as_deref(),
                Some("tgd:np/54t")
            );
        }
        found => eprintln!("SKIP  redirect: the packs carry none for tgd:np/54s ({found:?})"),
    }
}

#[test]
fn a_units_places_are_listed_by_population() {
    let Some(geo) = common::provider("listing") else {
        return;
    };
    let listed = |json: &str| -> Vec<NamedPlace> {
        list(&geo, &serde_json::from_str::<PlaceListing>(json).unwrap()).unwrap()
    };
    let provinces = listed(r#"{"within": "NP", "level": 1}"#);
    eprintln!(
        "Nepal's level 1: {:?}",
        provinces
            .iter()
            .map(|unit| (&unit.name, unit.population.map(|p| p.people)))
            .collect::<Vec<_>>()
    );
    assert_ne!(provinces, Vec::new());
    assert!(
        provinces
            .iter()
            .all(|unit| unit.level == Some(1) && unit.kind == PlaceKind::Admin)
    );
    let ranked = |places: &[NamedPlace]| {
        places.windows(2).all(|pair| {
            let people = |place: &NamedPlace| place.population.map(|found| found.people);
            people(&pair[0]) >= people(&pair[1])
        })
    };
    assert!(ranked(&provinces));
    let bagmati = provinces
        .iter()
        .find(|unit| unit.code.as_deref() == Some("NP-P3"))
        .unwrap();
    let local = listed(&format!(
        r#"{{"within": "{}", "level": 3, "depth": null, "limit": 1000}}"#,
        bagmati.id
    ));
    eprintln!(
        "Bagmati's local levels: {}, first {:?}",
        local.len(),
        local.first().map(|unit| (&unit.name, unit.population))
    );
    assert!(local.len() > 50, "{}", local.len());
    assert!(ranked(&local));
    // With populations Kathmandu leads; without (the app pack carries
    // none) every place ties and the canonical name orders them.
    let counted = common::holds(&geo, Component::Extra);
    let kathmandu = if counted {
        assert_eq!(local[0].name, "Kathmandu");
        &local[0]
    } else {
        assert!(local.iter().all(|unit| unit.population.is_none()));
        assert!(local.windows(2).all(|pair| pair[0].name <= pair[1].name));
        local.iter().find(|unit| unit.name == "Kathmandu").unwrap()
    };
    let modelled = local
        .iter()
        .filter(|unit| {
            unit.population
                .is_some_and(|found| found.kind == Some(PopulationKind::Modelled))
        })
        .count();
    eprintln!("of which modelled: {modelled}");
    // Kathmandu district's children: its municipalities, and no deeper.
    let district = &kathmandu.parents.last().unwrap().id;
    let children = listed(&format!(
        r#"{{"within": "{district}", "kinds": ["ADMIN"]}}"#
    ));
    eprintln!("Kathmandu district's units: {}", children.len());
    assert!(children.iter().all(|unit| unit.level == Some(3)));
    assert!(children.iter().any(|unit| unit.id == kathmandu.id));
    // Settlements of a city, at any depth, capped by the limit.
    let settlements = listed(&format!(
        r#"{{"within": "{}", "kinds": ["SETTLEMENT"], "depth": null, "limit": 3}}"#,
        kathmandu.id
    ));
    assert_eq!(settlements.len(), 3);
    assert!(
        settlements
            .iter()
            .all(|place| place.kind == PlaceKind::Settlement && place.level.is_none())
    );
    // A unit no pack holds lists nothing.
    assert_eq!(listed(r#"{"within": "tgd:np/zzzzzz"}"#), Vec::new());
    // No former unit is a unit now: the seven provinces, each counted
    // where the packs carry populations.
    assert!(
        !counted || provinces.iter().all(|unit| unit.population.is_some()),
        "{provinces:?}"
    );
    assert_eq!(provinces.len(), 7);
}

/// A provider over the store's `Shared`, built before any pack is
/// installed, answers from each pack the store adds, and a context built
/// over it sees them without being rebuilt (§13.2).
#[test]
fn packs_the_store_adds_later_answer_without_a_new_context() {
    let Some(packs) = common::packs("refill") else {
        return;
    };
    let store = Arc::new(Shared::default());
    let geo = Arc::new(TeigeodataGeo::from_shared(Arc::clone(&store)));
    let sdk = teistro::Context::builder()
        .geo(Arc::clone(&geo) as Arc<dyn GeoProvider>)
        .build()
        .unwrap();
    let kathmandu = teistro::PlaceRequest {
        latitude_deg: 27.7172,
        longitude_deg: 85.324,
    };
    let before = sdk.geo().resolve(&kathmandu).unwrap();
    assert_eq!(
        (before.zone, before.source.data_version.as_str()),
        (None, NO_PACKS)
    );
    assert_eq!(searched(&geo, "Kathmandu", None), Vec::new());

    // The store installs Nepal's pack: one publish, as `Shared::follow` does.
    store.update(|engine: &mut Engine| {
        for pack in &packs {
            engine.add(Layer::open(pack, LayerOptions::default()).unwrap());
        }
    });
    let after = sdk.geo().resolve(&kathmandu).unwrap();
    assert_eq!(after.zone.as_deref(), Some("Asia/Kathmandu"));
    assert_ne!(after.source.data_version, NO_PACKS);
    assert_eq!(after.source.data_version.split(" + ").count(), packs.len());
    assert_ne!(searched(&geo, "Kathmandu", None), Vec::new());

    // And removes them: the provider answers nothing again.
    store.update(|engine: &mut Engine| {
        let ids: Vec<_> = engine.layers().map(Layer::id).collect();
        for id in ids {
            engine.remove(id);
        }
    });
    assert_eq!(sdk.geo().resolve(&kathmandu).unwrap().zone, None);
}
