//! Festival metadata and the central holidays (`03-design/festival-metadata.md`,
//! `03-design/india-central-holidays.md`), without an ephemeris where none is
//! needed.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::assert_is_empty,
    missing_docs,
    reason = "a test fails by panicking and indexes what it built"
)]

use std::collections::BTreeMap;

use teistro::catalogue::{Calendar, FestivalKind, Tradition};
use teistro::festival::{
    FestivalMeta, MetaFilter, Observances, PublishedEntry, PublishedSet, SubjectTo,
};
use teistro::{FestivalPack, FestivalRequest, PackForm, PackRequest};

/// The classification page's community column by key.
fn communities() -> BTreeMap<String, String> {
    let page = include_str!("../../../docs/03-design/festival-classification.md");
    page.lines()
        .filter_map(|line| {
            let cells: Vec<&str> = line.split('|').map(str::trim).collect();
            let key = cells.get(1)?.strip_prefix('`')?.strip_suffix('`')?;
            Some((key.to_owned(), (*cells.get(4)?).to_owned()))
        })
        .collect()
}

/// A page's words for a community as the values' keys.
fn keys_of(column: &str) -> Option<(Vec<&'static str>, Option<Tradition>)> {
    Some(match column {
        "Newar" => (vec!["NEWAR"], None),
        "Kirat" => (vec!["KIRAT"], None),
        "Tharu" => (vec!["THARU"], None),
        "Thakali" => (vec!["THAKALI"], None),
        "Gurung" => (vec!["GURUNG"], None),
        "Sherpa" => (vec!["SHERPA"], None),
        "Tamang" => (vec!["TAMANG"], None),
        "Maithil and Bhojpuri" => (vec!["MAITHIL", "BHOJPURI"], None),
        "Tharu and Maithil" => (vec!["THARU", "MAITHIL"], None),
        "Buddhist" => (Vec::new(), Some(Tradition::Buddhist)),
        "Sikh" => (Vec::new(), Some(Tradition::Sikh)),
        "Jain" => (Vec::new(), Some(Tradition::Jain)),
        "Muslim" => (Vec::new(), Some(Tradition::Islamic)),
        _ => return None,
    })
}

#[test]
fn every_rule_of_nepals_pack_holds_the_classifications_community() {
    let page = communities();
    let listed = FestivalPack::NepalPanchanga.listing();
    let mut held = 0;
    for row in &listed.rules {
        assert!(
            row.meta.regions.iter().any(|region| region == "NP"),
            "{}",
            row.key
        );
        let Some((keys, tradition)) = page.get(&row.key).and_then(|column| keys_of(column)) else {
            continue;
        };
        held += 1;
        for key in keys {
            assert!(
                row.meta.communities.iter().any(|held| held == key),
                "{}",
                row.key
            );
        }
        if tradition.is_some() {
            assert_eq!(row.meta.tradition, tradition, "{}", row.key);
        }
    }
    assert_eq!(held, 78, "rows the page names a community or a faith for");
}

#[test]
fn every_rule_of_every_pack_says_what_no_source_gives() {
    for pack in FestivalPack::ALL {
        for row in pack.listing().rules {
            assert_eq!(row.unsourced, row.meta.unsourced(), "{}", row.key);
            assert!(!row.source.is_empty(), "{}", row.key);
        }
    }
    // Every key the Nepal packs ship is in the table: a key it lacks would
    // be blank in every field, and even the most unsourced has its country.
    for row in FestivalPack::Nepal.listing().rules {
        assert!(!row.meta.is_empty(), "{}", row.key);
    }
}

#[test]
fn a_pack_listing_is_read_from_its_name() -> Result<(), teistro::Error> {
    let asked = PackRequest::from_json(r#"{"pack": "INDIA_CENTRAL"}"#)?;
    let listed = asked.pack.listing();
    assert_eq!(listed.pack, "INDIA_CENTRAL");
    let form = |key: &str| {
        listed
            .rules
            .iter()
            .find(|row| row.key == key)
            .map(|row| row.form)
    };
    assert_eq!(form("HOLI"), Some(PackForm::Following));
    assert_eq!(form("CHRISTMAS"), Some(PackForm::Dated));
    assert_eq!(form("LAKSHMI_PUJA"), Some(PackForm::Lunar));
    assert_eq!(form("MUHARRAM"), Some(PackForm::Published));
    assert_eq!(
        FestivalPack::NepalPanchanga
            .listing()
            .rules
            .iter()
            .find(|row| row.form == PackForm::Ekadashi)
            .map(|row| row.meta.kind.is_some()),
        Some(true)
    );
    assert_eq!(FestivalPack::Gaudiya.listing().rules.len(), 6);
    Ok(())
}

#[test]
fn the_state_pack_lists_the_central_rules_it_shares_in_the_state() -> Result<(), teistro::Error> {
    let asked = PackRequest::from_json(r#"{"pack": "INDIA_MAHARASHTRA"}"#)?;
    let listed = asked.pack.listing();
    let central = FestivalPack::IndiaCentral.listing();
    let row =
        |rows: &[teistro::PackRow], key: &str| rows.iter().find(|row| row.key == key).cloned();
    let holi = row(&listed.rules, "HOLI").unwrap();
    assert_eq!(holi.form, PackForm::Following);
    assert!(!holi.meta.optional && row(&central.rules, "HOLI").unwrap().meta.optional);
    assert_eq!(holi.source, row(&central.rules, "HOLI").unwrap().source);
    assert_eq!(
        row(&listed.rules, "MAHARASHTRA_DAY").unwrap().form,
        PackForm::Dated
    );
    let parsi = row(&listed.rules, "PARSI_NEW_YEAR").unwrap();
    assert_eq!(parsi.form, PackForm::Published);
    assert_eq!(parsi.meta.tradition, Some(Tradition::Zoroastrian));
    assert!(row(&listed.rules, "JANMASHTAMI").is_none());
    let state = MetaFilter {
        regions: vec!["MAHARASHTRA".to_owned()],
        ..MetaFilter::default()
    };
    assert!(
        listed
            .rules
            .iter()
            .filter(|row| row.key != "HOLIKA")
            .all(|row| state.admits(&row.meta)),
        "every day of the state's list is kept in the state"
    );
    assert!(!central.rules.iter().any(|row| state.admits(&row.meta)));
    Ok(())
}

#[test]
fn west_bengal_counts_its_puja_days_and_keeps_its_new_year_by_the_sun() -> Result<(), teistro::Error>
{
    let asked = PackRequest::from_json(r#"{"pack": "INDIA_WEST_BENGAL"}"#)?;
    let listed = asked.pack.listing();
    let row =
        |rows: &[teistro::PackRow], key: &str| rows.iter().find(|row| row.key == key).cloned();
    for key in ["HOLI", "DURGA_SHASHTHI", "VIJAYA_DASHAMI"] {
        assert_eq!(
            row(&listed.rules, key).unwrap().form,
            PackForm::Following,
            "{key}"
        );
    }
    assert_eq!(
        row(&listed.rules, "KALI_PUJA").unwrap().form,
        PackForm::Lunar
    );
    assert_eq!(
        row(&listed.rules, "POHELA_BOISHAKH").unwrap().form,
        PackForm::Dated
    );
    let shab = row(&listed.rules, "SHAB_E_BARAT").unwrap();
    assert_eq!(shab.form, PackForm::Published);
    assert_eq!(shab.meta.tradition, Some(Tradition::Islamic));
    // Diwali is Kali Puja here, and the central Holika is on no list.
    assert!(row(&listed.rules, "LAKSHMI_PUJA").is_none());
    assert!(row(&listed.rules, "HOLIKA").is_none());
    let state = MetaFilter {
        regions: vec!["WEST_BENGAL".to_owned()],
        ..MetaFilter::default()
    };
    assert!(
        listed.rules.iter().all(|row| state.admits(&row.meta)),
        "every day of the state's list is kept in the state"
    );
    Ok(())
}

#[test]
fn kerala_keeps_its_own_days_by_the_malayalam_month() -> Result<(), teistro::Error> {
    let asked = PackRequest::from_json(r#"{"pack": "INDIA_KERALA"}"#)?;
    let listed = asked.pack.listing();
    let row =
        |rows: &[teistro::PackRow], key: &str| rows.iter().find(|row| row.key == key).cloned();
    for key in [
        "VISHU",
        "FIRST_ONAM",
        "THIRUVONAM",
        "FOURTH_ONAM",
        "SREE_NARAYANA_GURU_JAYANTHI",
        "SREE_NARAYANA_GURU_SAMADHI",
        "MAUNDY_THURSDAY",
    ] {
        assert_eq!(
            row(&listed.rules, key).unwrap().form,
            PackForm::Dated,
            "{key}"
        );
    }
    for key in [
        "KARKIDAKA_VAVU",
        "JANMASHTAMI",
        "AVANI_AVITTOM",
        "LAKSHMI_PUJA",
    ] {
        assert_eq!(
            row(&listed.rules, key).unwrap().form,
            PackForm::Lunar,
            "{key}"
        );
    }
    let avittom = row(&listed.rules, "AVANI_AVITTOM").unwrap();
    assert!(avittom.meta.optional);
    assert_eq!(avittom.meta.communities, ["BRAHMIN"]);
    assert_eq!(
        row(&listed.rules, "EID_AL_FITR").unwrap().form,
        PackForm::Published
    );
    let state = MetaFilter {
        regions: vec!["KERALA".to_owned()],
        ..MetaFilter::default()
    };
    assert!(
        listed.rules.iter().all(|row| state.admits(&row.meta)),
        "every day of the state's list is kept in the state"
    );
    Ok(())
}

#[test]
fn tamil_nadu_keeps_its_own_days_by_the_vakya_month() -> Result<(), teistro::Error> {
    let asked = PackRequest::from_json(r#"{"pack": "INDIA_TAMIL_NADU"}"#)?;
    let listed = asked.pack.listing();
    let row =
        |rows: &[teistro::PackRow], key: &str| rows.iter().find(|row| row.key == key).cloned();
    for key in [
        "THAI_PONGAL",
        "THIRUVALLUVAR_DAY",
        "UZHAVAR_THIRUNAL",
        "PUTHANDU",
        "THAI_POOSAM",
        "BANK_ACCOUNTS_CLOSING",
    ] {
        assert_eq!(
            row(&listed.rules, key).unwrap().form,
            PackForm::Dated,
            "{key}"
        );
    }
    for key in [
        "JANMASHTAMI",
        "GANESHA_CHATURTHI",
        "VIJAYA_DASHAMI",
        "NARAK_CHATURDASHI",
    ] {
        assert_eq!(
            row(&listed.rules, key).unwrap().form,
            PackForm::Lunar,
            "{key}"
        );
    }
    assert_eq!(
        row(&listed.rules, "AYUDHA_PUJA").unwrap().form,
        PackForm::Following
    );
    assert_eq!(
        row(&listed.rules, "EID_AL_ADHA").unwrap().form,
        PackForm::Published
    );
    // Deepavali is the Naraka Chaturdashi here, not the central Diwali.
    assert!(row(&listed.rules, "LAKSHMI_PUJA").is_none());
    let state = MetaFilter {
        regions: vec!["TAMIL_NADU".to_owned()],
        ..MetaFilter::default()
    };
    assert!(
        listed.rules.iter().all(|row| state.admits(&row.meta)),
        "every day of the state's list is kept in the state"
    );
    Ok(())
}

#[test]
fn assam_keeps_its_own_days_by_the_assamese_month() -> Result<(), teistro::Error> {
    let asked = PackRequest::from_json(r#"{"pack": "INDIA_ASSAM"}"#)?;
    let listed = asked.pack.listing();
    let row =
        |rows: &[teistro::PackRow], key: &str| rows.iter().find(|row| row.key == key).cloned();
    for key in [
        "MAGH_BIHU",
        "BOHAG_BIHU_SECOND",
        "KATI_BIHU",
        "GWTHAR_BATHOU_SAN",
        "SANKARDEVA_JANMOTSAV",
        "MADHABDEVA_TIROBHAV",
        "JANMASHTAMI",
        "ASOM_DIVAS",
    ] {
        assert_eq!(
            row(&listed.rules, key).unwrap().form,
            PackForm::Dated,
            "{key}"
        );
    }
    for key in [
        "DOLA_YATRA",
        "KALI_PUJA",
        "BIR_CHILARAI_DIVAS",
        "KARAM_PUJA",
    ] {
        assert_eq!(
            row(&listed.rules, key).unwrap().form,
            PackForm::Lunar,
            "{key}"
        );
    }
    assert_eq!(
        row(&listed.rules, "VIJAYA_DASHAMI").unwrap().form,
        PackForm::Following
    );
    assert_eq!(
        row(&listed.rules, "MUHARRAM").unwrap().form,
        PackForm::Published
    );
    let state = MetaFilter {
        regions: vec!["ASSAM".to_owned()],
        ..MetaFilter::default()
    };
    assert!(
        listed.rules.iter().all(|row| state.admits(&row.meta)),
        "every day of the state's list is kept in the state"
    );
    Ok(())
}

#[test]
fn a_request_reads_its_filter_and_a_rule_keeps_its_values_on_the_wire() -> Result<(), teistro::Error>
{
    let meta = FestivalMeta::nepal("GAI_JATRA");
    let text = serde_json::to_string(&meta).map_err(|e| teistro::Error::internal(e.to_string()))?;
    assert!(
        text.contains("\"kind\":\"GAZETTED\"") || text.contains("GAZETTED"),
        "{text}"
    );
    let bad =
        FestivalRequest::from_json(r#"{"rules": "NEPAL", "only": {"kind": ["X"]}}"#).unwrap_err();
    assert!(
        bad.field()
            .is_some_and(|field| field.starts_with("festivals"))
    );
    let filter = MetaFilter {
        kinds: vec![FestivalKind::Gazetted],
        ..MetaFilter::default()
    };
    let asked = FestivalRequest::from(FestivalPack::IndiaCentral).keeping(filter);
    assert!(asked.keeps("EID_AL_FITR"));
    assert!(asked.keeps("HOLI"));
    assert!(!asked.keeps("HOLIKA"));
    assert!(!asked.keeps("NO_SUCH_KEY"));
    assert!(FestivalRequest::from(FestivalPack::IndiaCentral).keeps("NO_SUCH_KEY"));
    Ok(())
}

#[cfg(feature = "builtin-ephemeris")]
mod reckoned {
    use teistro::quantity::{Altitude, Latitude, Longitude, Place};
    use teistro::{CalendarDate, Context, Ephemeris, UtcOffset};

    use super::*;

    fn delhi() -> Place {
        Place::new(
            Latitude::literal(28.6139),
            Longitude::literal(77.209),
            Altitude::literal(216.0),
        )
    }

    fn year(asked: &FestivalRequest, year: i32) -> Result<Observances, teistro::Error> {
        let sdk = Context::builder().ephemeris([Ephemeris::Builtin]).build()?;
        let day = |month, day| CalendarDate::defined(Calendar::Gregorian, year, month, day);
        Ok(sdk
            .almanac()
            .festivals(
                &day(1, 1),
                &day(12, 31),
                &delhi(),
                UtcOffset::literal(5, 30, 0),
                asked,
            )?
            .value)
    }

    #[test]
    fn the_moons_days_stand_as_printed_and_marked_and_only_in_a_noticed_year()
    -> Result<(), teistro::Error> {
        let asked = FestivalRequest::from(FestivalPack::IndiaCentral);
        let found = year(&asked, 2026)?;
        let eid = found
            .added
            .iter()
            .find(|day| day.key == "EID_AL_FITR")
            .ok_or_else(|| teistro::Error::internal("no Eid"))?;
        assert_eq!((eid.day.month, eid.day.day), (3, 21));
        assert_eq!(eid.published.subject_to, Some(SubjectTo::MoonSighting));
        let bakrid = found
            .added
            .iter()
            .find(|day| day.key == "EID_AL_ADHA")
            .ok_or_else(|| teistro::Error::internal("no Bakrid"))?;
        assert_eq!((bakrid.day.month, bakrid.day.day), (5, 28));
        assert_eq!(bakrid.published.subject_to, None);
        assert!(year(&asked, 2031)?.added.is_empty());
        Ok(())
    }

    #[test]
    fn a_filter_keeps_a_day_counted_from_a_rule_it_drops() -> Result<(), teistro::Error> {
        let asked = FestivalRequest::from(FestivalPack::IndiaCentral).keeping(MetaFilter {
            kinds: vec![FestivalKind::Gazetted],
            ..MetaFilter::default()
        });
        let found = year(&asked, 2025)?;
        assert!(found.observances.iter().any(|day| day.rule == "HOLI"));
        assert!(!found.observances.iter().any(|day| day.rule == "HOLIKA"));
        assert!(found.dated.iter().any(|day| day.rule == "GOOD_FRIDAY"));
        assert_eq!(found.added.len(), 4);
        Ok(())
    }

    #[test]
    fn a_consumers_set_replaces_a_day_set_by_the_moon() -> Result<(), teistro::Error> {
        let correction = PublishedSet::from_json(
            r#"{"format": "teistro.published/1", "publisher": "a state", "id": "s", "version": 1,
                "entries": [{"action": "ADD", "key": "EID_AL_FITR",
                             "day": {"calendar": "GREGORIAN", "year": 2026, "month": 3, "day": 22},
                             "source": "an order"}]}"#,
        )?;
        let asked = FestivalRequest::from(FestivalPack::IndiaCentral).with_published(correction);
        let found = year(&asked, 2026)?;
        let eid: Vec<_> = found
            .added
            .iter()
            .filter(|day| day.key == "EID_AL_FITR")
            .collect();
        assert_eq!(eid.len(), 1);
        assert_eq!(eid[0].day.day, 22);
        assert_eq!(eid[0].published.subject_to, None);
        Ok(())
    }

    #[test]
    fn an_added_entry_in_code_carries_its_mark() {
        let set = PublishedSet::india_central();
        assert!(set.entries.iter().all(|entry| matches!(entry, PublishedEntry::Add { meta, .. } if meta.tradition == Some(Tradition::Islamic))));
    }
}
