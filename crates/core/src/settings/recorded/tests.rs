#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking, and index the documents they built"
)]

use std::collections::BTreeMap;

use serde_json::{Value as Json, json};

use super::{
    AYANAMSHA_KEYS, Becomes, DASHA_SYSTEMS, KP_PROFILE, Opening, PAIRINGS, PROFILE, ROWS, Recorded,
    Refused, RequiredEphemeris, Row, Snapshot, Stored, TABLE, Value, house_system, recorded,
};
use crate::catalogue::{Ayanamsha, DashaSystem, HouseSystem};
use crate::error::{Error, Status};
use crate::quantity::Depth;
use crate::settings::knobs::{
    AyanamshaBasis, Balance, Centre, CharaKarakas, DayLordDay, Ekadhipatya, Node, PolarDayPolicy,
    PolarPolicy, RulingCount, Shodhana, Sunrise, Zodiac,
};
use crate::settings::{
    AyanamshaChoice, Profile, ProfileId, Settings, SettingsPatch, Siddhanta, SunriseConvention,
    SuryaBija, SuryaPlanets, SuryaSunrise,
};

/// The corpus's `default` profile, as its manifest's canonical form holds it.
const DEFAULT: &str = "ayanamsaId=1|dashaBalanceMethod=spatial|ekadhipatyaMethod=classical|engineVersion=1.8.0|houseSystem=whole-sign|nodeType=mean|siddhanta=drik|sunriseMode=apparent-refraction|swissephVersion=2.10.03|useTopocentric=true|zodiacMode=sidereal";

/// A snapshot holding one value at a path, nested where the path is.
fn holding(path: &str, value: Stored) -> Snapshot {
    let mut parts: Vec<&str> = path.split('.').collect();
    let mut value = value;
    while parts.len() > 1 {
        let key = parts.pop().unwrap();
        value = Stored::Fields(BTreeMap::from([(key.to_owned(), value)]));
    }
    Snapshot {
        fields: BTreeMap::from([(parts[0].to_owned(), value)]),
    }
}

fn text(value: &str) -> Stored {
    Stored::Text(value.to_owned())
}

/// The translation of one stored value.
fn of(path: &str, value: Stored) -> Result<Recorded, Refused> {
    recorded(&holding(path, value))
}

/// What a value-reading row is given to read, and where.
fn sample(row: &Row) -> (String, Stored) {
    let value = match row.value {
        Value::Text(value) => text(value),
        Value::Flag(value) => Stored::Flag(value),
        Value::Number(value) => Stored::Number(value),
        Value::Each(_) => match row.field {
            "ayanamsaId" => Stored::Number(3),
            "houseSystem" => text("koch"),
            "dashaMaxLevel" | "dashaDepthBySystem.*" => Stored::Number(4),
            "locale.script" => text("Deva"),
            _ => text("1.8.0"),
        },
    };
    (row.field.replace(".*", ".yogini"), value)
}

/// Every knob two settings disagree on, as `group.knob`.
fn differences(a: &Settings, b: &Settings) -> Vec<String> {
    let (a, b) = (
        serde_json::to_value(a).unwrap(),
        serde_json::to_value(b).unwrap(),
    );
    let mut differ = Vec::new();
    for (group, knobs) in a.as_object().unwrap() {
        let Some(knobs) = knobs.as_object() else {
            continue;
        };
        for (knob, value) in knobs {
            if b[group][knob] != *value {
                differ.push(format!("{group}.{knob}"));
            }
        }
    }
    differ
}

#[test]
fn every_row_takes_its_own_value_and_becomes_what_it_says() {
    for row in ROWS {
        let (path, value) = sample(row);
        let translated = of(&path, value.clone());
        match row.becomes {
            Becomes::Settings(write) => {
                let mut written = SettingsPatch::default();
                write(&mut written);
                assert_eq!(translated.unwrap().natal.settings, written, "{path}");
            }
            Becomes::Read(_) => {
                assert!(!translated.unwrap().natal.settings.is_empty(), "{path}");
            }
            Becomes::Ephemeris(write, needs) => {
                let translated = translated.unwrap();
                let mut written = SettingsPatch::default();
                write(&mut written);
                assert_eq!(
                    (translated.natal.settings, translated.ephemeris),
                    (written, Some(needs))
                );
            }
            Becomes::Locale(tag) => assert_eq!(translated.unwrap().locale, Some(tag)),
            Becomes::Script => {
                let translated = translated.unwrap();
                assert!(translated.locale.is_none() && translated.notes.is_empty());
            }
            Becomes::Note(why) => {
                let notes = translated.unwrap().notes;
                assert_eq!(notes.len(), 1, "{path}");
                assert_eq!(
                    (notes[0].field.as_str(), notes[0].why),
                    (path.as_str(), why)
                );
                assert_eq!(notes[0].value, value);
            }
            Becomes::Refused { anchor, why } => {
                let refused = translated.unwrap_err();
                assert_eq!(
                    (refused.field.as_str(), refused.anchor, refused.why),
                    (path.as_str(), anchor, why)
                );
                assert!(refused.decided);
            }
        }
    }
}

#[test]
fn each_translation_of_one_row_opens_both_contexts() {
    for row in ROWS {
        let (path, value) = sample(row);
        if let Ok(translated) = of(&path, value) {
            translated.natal.resolve().unwrap();
            translated.kp.resolve().unwrap();
        }
    }
}

#[test]
fn the_default_snapshot_is_the_profile_knob_for_knob() {
    let translated = recorded(&Snapshot::from_canonical(DEFAULT).unwrap()).unwrap();
    let profile = Profile::shipped(PROFILE).unwrap();
    let natal = translated.natal.resolve().unwrap();
    assert_eq!(
        differences(&natal.settings, &profile.settings()),
        Vec::<String>::new()
    );
    assert_eq!(natal.settings.hash(), profile.settings().hash());
    assert_eq!(translated.ephemeris, None);
    assert_eq!(translated.locale, None);
    let noted: Vec<&str> = translated
        .notes
        .iter()
        .map(|note| note.field.as_str())
        .collect();
    assert_eq!(noted, ["engineVersion", "swissephVersion"]);
}

#[test]
fn the_kp_context_is_kp_default_on_the_natal_frame() {
    let translated = of("useTopocentric", Stored::Flag(false)).unwrap();
    assert_eq!(translated.kp.profile.as_str(), KP_PROFILE);
    let kp = translated.kp.resolve().unwrap().settings;
    assert_eq!(
        (
            kp.frame.node,
            kp.frame.centre,
            kp.frame.ayanamsha_basis,
            kp.day.polar_day_policy
        ),
        (
            Node::Mean,
            Centre::Geocentric,
            AyanamshaBasis::True,
            PolarDayPolicy::NearestEvent
        )
    );
    // The ruling planets are the baseline's, as the natal context reads
    // them: seven, the sub lords too, by the civil weekday (C150, C151).
    assert_eq!(
        (kp.kp.ruling_count, kp.kp.day_lord_day),
        (RulingCount::WithSubs, DayLordDay::Civil)
    );
    // What KP Reader VI reads otherwise stays the profile's own.
    assert_eq!(
        kp.frame.ayanamsha,
        AyanamshaChoice::from(Ayanamsha::Krishnamurti)
    );
    assert_eq!(kp.houses.placement_system, HouseSystem::Placidus);
}

#[test]
fn the_sunrise_modes_translate_by_the_altitude_they_compute() {
    // C776.
    let sunrise = |mode: &str| {
        of("sunriseMode", text(mode))
            .unwrap()
            .natal
            .settings
            .day
            .sunrise
    };
    assert_eq!(
        sunrise("apparent-refraction"),
        Some(Sunrise::UpperLimbRefraction.into())
    );
    assert_eq!(
        sunrise("geometric"),
        Some(Sunrise::CentreNoRefraction.into())
    );
    assert_eq!(
        sunrise("upper-limb"),
        Some(SunriseConvention::Custom {
            altitude_deg: -0.5667
        })
    );
    assert_eq!(
        sunrise("lower-limb"),
        Some(SunriseConvention::Custom { altitude_deg: -1.1 })
    );
    let refused = of("sunriseMode", text("civil")).unwrap_err();
    assert_eq!((refused.anchor, refused.decided), (TABLE, false));
}

#[test]
fn the_surya_bija_is_refused_and_its_absence_is_the_text() {
    // C777.
    let refused = of("suryaBija", Stored::Flag(true)).unwrap_err();
    assert_eq!((refused.anchor, refused.decided), ("C777", true));
    let surya = recorded(
        &Snapshot::from_canonical(
            "ayanamsaId=1|siddhanta=surya|suryaBija=false|useTopocentric=true",
        )
        .unwrap(),
    )
    .unwrap();
    let frame = &surya.natal.settings.frame;
    assert_eq!(
        frame.siddhanta,
        Some(Siddhanta::Surya {
            bija: SuryaBija::None,
            sunrise: SuryaSunrise::Modern,
            planets: SuryaPlanets::Text,
        })
    );
    // The text's places are geocentric and in its own zodiac, whatever the
    // centre and the ayanamsha stored beside it say.
    assert_eq!(frame.centre, Some(Centre::Geocentric));
    assert_eq!(
        frame.ayanamsha,
        Some(AyanamshaChoice::from(Ayanamsha::Suryasiddhanta))
    );
    assert_eq!(surya.ephemeris, Some(RequiredEphemeris::SuryaSiddhanta));
    assert_eq!(surya.notes[0].field, "suryaBija");
    surya.natal.resolve().unwrap();
    // Nothing stored is the engine's default, read as nothing.
    let unset = of("suryaBija", Stored::Absent).unwrap();
    assert!(unset.notes.is_empty() && unset.natal.settings.is_empty());
}

#[test]
fn the_legacy_ekadhipatya_methods_are_refused() {
    // C60.
    for method in ["zero", "transfer"] {
        let refused = of("ekadhipatyaMethod", text(method)).unwrap_err();
        assert_eq!((refused.anchor, refused.decided), ("C60", true), "{method}");
    }
    let classical = of("ekadhipatyaMethod", text("classical")).unwrap();
    assert_eq!(
        (
            classical.natal.settings.strength.ekadhipatya,
            classical.natal.settings.strength.shodhana
        ),
        (Some(Ekadhipatya::EmptyToZero), Some(Shodhana::Sarva))
    );
}

#[test]
fn an_ayanamsha_number_is_the_member_whose_swiss_mode_it_is() {
    for member in Ayanamsha::ALL {
        let mode = i64::from(member.attributes().swiss_mode);
        let translated = of("ayanamsaId", Stored::Number(mode)).unwrap();
        assert_eq!(
            translated.natal.settings.frame.ayanamsha,
            Some(member.into()),
            "{mode}"
        );
    }
    let named = |mode| {
        of("ayanamsaId", Stored::Number(mode))
            .unwrap()
            .natal
            .settings
            .frame
            .ayanamsha
    };
    assert_eq!(named(0), Some(Ayanamsha::FaganBradley.into()));
    assert_eq!(named(3), Some(Ayanamsha::Raman.into()));
    assert_eq!(named(5), Some(Ayanamsha::Krishnamurti.into()));
    assert_eq!(named(27), Some(Ayanamsha::TrueChitra.into()));
    for unknown in [Stored::Number(47), Stored::Number(-1), text("1")] {
        let refused = of("ayanamsaId", unknown.clone()).unwrap_err();
        assert_eq!(
            (refused.anchor, refused.decided),
            (TABLE, false),
            "{unknown}"
        );
    }
}

#[test]
fn an_ayanamsha_key_is_the_member_its_number_names() {
    let zodiac = |stored| {
        of("ayanamsaId", stored)
            .unwrap()
            .natal
            .settings
            .frame
            .ayanamsha
    };
    // Every key translates as the number of the member it lists does, and
    // the keys reach every member: 47 members, one with a second spelling.
    for (key, member) in AYANAMSHA_KEYS {
        let mode = i64::from(member.attributes().swiss_mode);
        assert_eq!(zodiac(text(key)), Some(member.into()), "{key}");
        assert_eq!(zodiac(text(key)), zodiac(Stored::Number(mode)), "{key}");
    }
    let keys: std::collections::BTreeSet<&str> =
        AYANAMSHA_KEYS.iter().map(|(key, _)| *key).collect();
    assert_eq!(keys.len(), AYANAMSHA_KEYS.len());
    let reached: std::collections::BTreeSet<Ayanamsha> =
        AYANAMSHA_KEYS.iter().map(|(_, member)| *member).collect();
    assert_eq!(reached, Ayanamsha::ALL.into_iter().collect());
    // Four keys are not the member's key lower-cased with hyphens, which is
    // why the keys are listed rather than derived.
    let ruled: Vec<&str> = AYANAMSHA_KEYS
        .iter()
        .filter(|(key, member)| member.key().to_ascii_lowercase().replace('_', "-") != *key)
        .map(|(key, _)| *key)
        .collect();
    assert_eq!(
        ruled,
        [
            "surya-siddhanta",
            "surya-siddhanta-msun",
            "de-luce",
            "galcent-gil-brand"
        ]
    );
    // The legacy spelling and the hyphenated one are one member.
    assert_eq!(
        zodiac(text("suryasiddhanta")),
        zodiac(text("surya-siddhanta"))
    );
    // The application's default, and the three its documentation names first.
    assert_eq!(zodiac(text("lahiri")), Some(Ayanamsha::Lahiri.into()));
    assert_eq!(zodiac(text("raman")), Some(Ayanamsha::Raman.into()));
    assert_eq!(
        zodiac(text("krishnamurti")),
        Some(Ayanamsha::Krishnamurti.into())
    );
    // An unknown key, another case, a number written as text, a flag: each
    // refused on the field, with the table as its hint.
    for unknown in [
        text("chitrapaksha"),
        text("Lahiri"),
        text("LAHIRI"),
        text("1"),
        Stored::Flag(true),
    ] {
        let refused = of("ayanamsaId", unknown.clone()).unwrap_err();
        assert_eq!(
            (refused.field.as_str(), refused.anchor, refused.decided),
            ("ayanamsaId", TABLE, false),
            "{unknown}"
        );
        let error = Error::from(refused);
        assert_eq!(
            (error.status, error.field(), error.hint()),
            (Status::InvalidArg, Some("snapshot.ayanamsaId"), Some(TABLE)),
            "{unknown}"
        );
    }
    assert_eq!(
        of("ayanamsaId", text("chitrapaksha")).unwrap_err().why,
        "not one of the 48 keys the application stores an ayanamsha under (`AYANAMSHA_KEYS`)"
    );
    // A key under the Surya Siddhanta still yields to the text's own zodiac.
    let mut snapshot = holding("ayanamsaId", text("raman"));
    snapshot.fields.insert("siddhanta".into(), text("surya"));
    assert_eq!(
        recorded(&snapshot).unwrap().natal.settings.frame.ayanamsha,
        Some(Ayanamsha::Suryasiddhanta.into())
    );
}

#[test]
fn a_house_system_is_named_by_its_lower_cased_key() {
    let names = [
        "whole-sign",
        "placidus",
        "koch",
        "regiomontanus",
        "campanus",
        "equal",
        "meridian",
        "alcabitius",
        "porphyry",
        "topocentric",
        "morinus",
        "sripati",
        "equal-mc",
        "equal-aries",
        "vehlow",
        "carter",
        "horizon",
        "sunshine",
        "pullen-sd",
        "pullen-sr",
        "krusinski",
        "apc",
    ];
    for name in names {
        let system = house_system(name).unwrap();
        let translated = of("houseSystem", text(name)).unwrap();
        assert_eq!(
            translated.natal.settings.houses.placement_system,
            Some(system)
        );
        // The chalit stays the profile's, registry entry 14.
        assert_eq!(translated.natal.settings.houses.chalit_system, None);
        // A polar arc is divided as Porphyry does, but for the profile's
        // own whole-sign, which has none.
        let polar = (system != HouseSystem::WholeSign).then_some(PolarPolicy::FallbackPorphyry);
        assert_eq!(translated.natal.settings.houses.polar_policy, polar);
    }
    assert_eq!(names.len(), HouseSystem::ALL.len());
    for unknown in [text("whole_sign"), Stored::Number(1)] {
        assert!(of("houseSystem", unknown).is_err());
    }
}

#[test]
fn a_dasha_depth_is_each_system_s_and_a_system_s_own_wins() {
    let depth = |n| Depth::try_new(n).unwrap();
    let overall = of("dashaMaxLevel", Stored::Number(4)).unwrap();
    let map = overall.natal.settings.dasha.depth.unwrap();
    for (_, system) in DASHA_SYSTEMS {
        assert_eq!(map[&system], depth(4), "{system}");
    }
    // A system the baseline does not compute keeps the profile's depth.
    assert_eq!(map[&DashaSystem::Sudasa], depth(3));
    // A system's own depth stands over the overall one, whatever order
    // the snapshot lists them in.
    let both = Snapshot::from_json(
        r#"{"dashaDepthBySystem": {"vimshottari": 5, "padanadhamsa": 1}, "dashaMaxLevel": 2}"#,
    )
    .unwrap();
    let map = recorded(&both).unwrap().natal.settings.dasha.depth.unwrap();
    assert_eq!(
        (
            map[&DashaSystem::Vimshottari],
            map[&DashaSystem::Padanadhamsa],
            map[&DashaSystem::Yogini]
        ),
        (depth(5), depth(1), depth(2))
    );
    for (path, value) in [
        ("dashaMaxLevel", Stored::Number(0)),
        ("dashaMaxLevel", Stored::Number(6)),
        ("dashaMaxLevel", text("3")),
        ("dashaDepthBySystem.sudasa", Stored::Number(3)),
    ] {
        let refused = of(path, value).unwrap_err();
        assert_eq!((refused.field.as_str(), refused.decided), (path, false));
    }
}

#[test]
fn every_chara_karaka_count_and_balance_is_a_knob() {
    let karakas = |n| {
        of("charaKarakaSystem", Stored::Number(n))
            .unwrap()
            .natal
            .settings
            .jaimini
            .chara_karakas
    };
    assert_eq!(
        (karakas(7), karakas(8)),
        (Some(CharaKarakas::Seven), Some(CharaKarakas::Eight))
    );
    assert!(of("charaKarakaSystem", Stored::Number(9)).is_err());
    let balance = |method| {
        of("dashaBalanceMethod", text(method))
            .unwrap()
            .natal
            .settings
            .dasha
            .balance
    };
    assert_eq!(
        (balance("spatial"), balance("temporal")),
        (Some(Balance::Spatial), Some(Balance::Temporal))
    );
    let zodiac = |mode| {
        of("zodiacMode", text(mode))
            .unwrap()
            .natal
            .settings
            .frame
            .zodiac
    };
    // The recording engine's tropical mode moves nothing it records.
    assert_eq!(
        (zodiac("sidereal"), zodiac("tropical")),
        (Some(Zodiac::Sidereal), None)
    );
    let tropical = of("zodiacMode", text("tropical")).unwrap();
    assert_eq!(tropical.notes[0].field, "zodiacMode");
}

#[test]
fn a_language_is_its_locale_and_its_script_must_match() {
    let locale = |language: &str, script: &str| {
        recorded(
            &Snapshot::from_json(&format!(
                r#"{{"locale": {{"language": "{language}", "script": "{script}", "numeralSystem": "arabic", "calendarSystem": "bs", "timeFormat24h": false}}}}"#
            ))
            .unwrap(),
        )
    };
    assert_eq!(locale("ne", "Deva").unwrap().locale, Some("ne-Deva-NP"));
    assert_eq!(locale("en", "Latn").unwrap().locale, Some("en-Latn"));
    assert_eq!(locale("hi", "Deva").unwrap().locale, Some("hi-Deva-IN"));
    assert_eq!(locale("sa", "Deva").unwrap().locale, Some("sa-Deva"));
    // The presentation is noted, in table order.
    let noted: Vec<String> = locale("ne", "Deva")
        .unwrap()
        .notes
        .into_iter()
        .map(|note| note.field)
        .collect();
    assert_eq!(
        noted,
        [
            "locale.numeralSystem",
            "locale.calendarSystem",
            "locale.timeFormat24h"
        ]
    );
    let refused = locale("ne", "Latn").unwrap_err();
    assert_eq!(
        (refused.field.as_str(), refused.anchor, &refused.value),
        ("locale.script", PAIRINGS, &text("Latn"))
    );
    assert!(locale("ta", "Taml").is_err());
    assert!(of("locale.script", Stored::Flag(true)).is_err());
}

#[test]
fn a_field_or_value_the_table_does_not_know_is_refused_not_skipped() {
    for (path, value, why) in [
        (
            "ayanamsa",
            Stored::Number(1),
            "a field the table does not know",
        ),
        ("locale.font", text("x"), "a field the table does not know"),
        ("locale", text("ne"), "a field the table does not know"),
        (
            "nodeType",
            text("osculating"),
            "a value the table does not know",
        ),
        (
            "useTopocentric",
            text("true"),
            "a value the table does not know",
        ),
    ] {
        let refused = of(path, value).unwrap_err();
        assert_eq!(
            (
                refused.field.as_str(),
                refused.why,
                refused.anchor,
                refused.decided
            ),
            (path, why, TABLE, false)
        );
    }
}

#[test]
fn a_refusal_is_an_error_naming_the_field_and_its_anchor() {
    let unknown = Error::from(of("nodeType", text("osculating")).unwrap_err());
    assert_eq!(
        (unknown.status, unknown.field(), unknown.hint()),
        (Status::InvalidArg, Some("snapshot.nodeType"), Some(TABLE))
    );
    assert_eq!(
        unknown.message,
        "the stored `nodeType: \"osculating\"` is refused (03-design/compat-shim.md §4.2): a value the table does not know"
    );
    let decided = Error::from(of("ekadhipatyaMethod", text("zero")).unwrap_err());
    assert_eq!(
        (decided.status, decided.hint()),
        (Status::Unsupported, Some("C60"))
    );
}

#[test]
fn a_snapshot_reads_from_json_and_from_the_canonical_form() {
    let canonical = Snapshot::from_canonical(DEFAULT).unwrap();
    assert_eq!(canonical.fields["ayanamsaId"], Stored::Number(1));
    assert_eq!(canonical.fields["useTopocentric"], Stored::Flag(true));
    assert_eq!(canonical.fields["swissephVersion"], text("2.10.03"));
    assert_eq!(canonical.fields["useTopocentric"].to_string(), "true");
    // A value a row names as text stays text, though it reads as a flag.
    let true_node = Snapshot::from_canonical("nodeType=true|useTopocentric=false").unwrap();
    assert_eq!(true_node.fields["nodeType"], text("true"));
    assert_eq!(true_node.fields["useTopocentric"], Stored::Flag(false));
    let translated = recorded(&true_node).unwrap();
    assert_eq!(translated.natal.settings.frame.node, Some(Node::True));
    let refused = Snapshot::from_canonical("nodeType=mean|siddhanta").unwrap_err();
    assert_eq!(
        (refused.status, refused.field()),
        (Status::InvalidArg, Some("snapshot"))
    );
    // The same fields as JSON read the same.
    let json = serde_json::to_string(&canonical).unwrap();
    assert_eq!(Snapshot::from_json(&json).unwrap(), canonical);
    assert!(Snapshot::from_json(r#"{"nodeType": 1.5}"#).is_err());
}

#[test]
fn a_row_for_every_key_of_a_record_names_only_its_keys() {
    let row = ROWS
        .iter()
        .find(|row| row.field == "dashaDepthBySystem.*")
        .unwrap();
    assert!(row.names("dashaDepthBySystem.vimshottari"));
    assert!(!row.names("dashaDepthBySystem"));
    assert!(!row.names("dashaDepthBySystemx.vimshottari"));
    assert!(!Value::Text("mean").takes(&Stored::Number(1)));
    assert!(!Value::Flag(true).takes(&Stored::Flag(false)));
    assert!(!Value::Number(7).takes(&Stored::Number(8)));
}

#[test]
fn an_opening_resolves_only_a_shipped_profile() {
    let opening = Opening {
        profile: ProfileId::new("acme"),
        settings: SettingsPatch::default(),
    };
    let refused = opening.resolve().unwrap_err();
    assert_eq!(
        (refused.status, refused.field()),
        (Status::InvalidArg, Some("profile"))
    );
    // An incoherent patch is the profile's refusal, passed through.
    let mut settings = SettingsPatch::default();
    settings.day.sunrise = Some(SunriseConvention::Custom { altitude_deg: 95.0 });
    let incoherent = Opening {
        profile: ProfileId::new(PROFILE),
        settings,
    };
    assert_eq!(
        incoherent.resolve().unwrap_err().field(),
        Some("day.sunrise")
    );
}

#[test]
fn a_translation_writes_the_json_a_binding_reads() {
    let translated = recorded(
        &Snapshot::from_json(r#"{"nodeType": "true", "siddhanta": "surya", "locale": {"language": "en"}, "engineVersion": "1.8.0"}"#)
            .unwrap(),
    )
    .unwrap();
    let json = serde_json::to_value(&translated).unwrap();
    assert_eq!(json["natal"]["profile"], "conformance-baseline");
    assert_eq!(json["natal"]["settings"]["frame"]["node"], "TRUE");
    assert_eq!(json["kp"]["profile"], "kp-default");
    assert_eq!(json["kp"]["settings"]["frame"]["node"], "TRUE");
    assert_eq!(json["ephemeris"], "SURYA_SIDDHANTA");
    assert_eq!(json["locale"], "en-Latn");
    assert_eq!(
        json["notes"],
        json!([{"field": "engineVersion", "value": "1.8.0", "why": "kept for the stored chart's comparison (guide §8.3)"}])
    );
    let none = serde_json::to_value(recorded(&Snapshot::default()).unwrap()).unwrap();
    assert_eq!(
        (none["ephemeris"].clone(), none["locale"].clone()),
        (Json::Null, Json::Null)
    );
}
