//! Anniversaries over synthetic days (`03-design/shraddha.md`): the clauses
//! a real sky reaches too rarely to test, the kshaya year's joined lunation
//! above all (C669), and the refusals.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

mod common;

use common::{days_of, ghati};
use teistro_calendar::lunisolar::MonthKind;
use teistro_core::catalogue::{Masa, Tithi};
use teistro_panchanga::festival::{
    Anniversary, AnniversaryKind, FirstYearAdhika, Half, JanmaOptions, MonthClause, Origin,
    OriginFound, Window, anniversaries,
};

fn asked(month: Masa, tithi: Tithi, kind: AnniversaryKind) -> (Anniversary, OriginFound) {
    let origin = Origin::Lunar {
        month,
        tithi,
        adhika: false,
        half: Half::First,
        at: None,
    };
    (
        Anniversary {
            key: "ONE".to_owned(),
            origin,
            kind,
            window: None,
            first_year_adhika: FirstYearAdhika::Adhika,
            janma: JanmaOptions::default(),
        },
        OriginFound::lunar(month, tithi, false, Half::First, None),
    )
}

/// The bright 5th from late on day 0 to near day 1's end: day 1 alone
/// holds its kutapa.
fn fifth() -> [(Tithi, f64); 3] {
    [
        (Tithi::ShuklaChaturthi, ghati(0, 54.0)),
        (Tithi::ShuklaPanchami, ghati(1, 57.0)),
        (Tithi::ShuklaShashthi, ghati(4, 0.0)),
    ]
}

/// C669: in a kshaya year the joined lunation answers to both its names,
/// so an anniversary of the skipped month is kept in it on its tithi, and
/// one of the lunation's own first name is kept there as ever.
#[test]
fn a_kshaya_lunation_keeps_both_its_months_anniversaries() {
    let kshaya = days_of(&fifth(), &[], Masa::Margashirsha, MonthKind::Kshaya);
    for month in [Masa::Pausha, Masa::Margashirsha] {
        let found = anniversaries(
            &[asked(
                month,
                Tithi::ShuklaPanchami,
                AnniversaryKind::Ekoddishta,
            )],
            &kshaya,
        )
        .unwrap();
        assert_eq!(found.anniversaries.len(), 1, "{month:?}");
        let kept = &found.anniversaries[0];
        assert_eq!(kept.month_by, MonthClause::KshayaJoined, "{month:?}");
        assert_eq!(kept.observance.day.day, 2, "{month:?}");
    }
    // An ordinary Margashirsha keeps Pausha's in no lunation of its own.
    let ordinary = days_of(&fifth(), &[], Masa::Margashirsha, MonthKind::Nija);
    let found = anniversaries(
        &[asked(
            Masa::Pausha,
            Tithi::ShuklaPanchami,
            AnniversaryKind::Ekoddishta,
        )],
        &ordinary,
    )
    .unwrap();
    assert_eq!(found.anniversaries, Vec::new());
}

/// DS p. 266 (C663): a tithi holding both days' kutapa wholly gives the
/// earlier day; a shraddha of a death in Bhadrapada's dark half is a
/// parvana, at aparahna, and any other an ekoddishta (C665).
#[test]
fn the_choice_and_the_kind_follow_the_texts() {
    let both = [
        (Tithi::ShuklaChaturthi, ghati(0, 12.0)),
        (Tithi::ShuklaPanchami, ghati(1, 24.0)),
        (Tithi::ShuklaShashthi, ghati(4, 0.0)),
    ];
    let days = days_of(&both, &[], Masa::Kartika, MonthKind::Nija);
    let found = anniversaries(
        &[asked(
            Masa::Kartika,
            Tithi::ShuklaPanchami,
            AnniversaryKind::Shraddha,
        )],
        &days,
    )
    .unwrap();
    let kept = &found.anniversaries[0];
    assert_eq!(kept.observance.day.day, 1);
    assert_eq!(kept.kind, AnniversaryKind::Ekoddishta);
    assert_eq!(
        kept.observance.extents[0].window.days(),
        kept.observance.extents[1].window.days()
    );
    let dark = OriginFound::lunar(
        Masa::Bhadrapada,
        Tithi::KrishnaPanchami,
        false,
        Half::First,
        None,
    );
    assert!(dark.is_parvana());
    assert!(
        OriginFound::lunar(Masa::Kartika, Tithi::Amavasya, false, Half::First, None).is_parvana()
    );
    assert!(
        !OriginFound::lunar(
            Masa::Ashwina,
            Tithi::KrishnaPanchami,
            false,
            Half::First,
            None
        )
        .is_parvana()
    );
    assert_eq!(
        AnniversaryKind::Parvana.window(),
        Window::Part {
            part: teistro_panchanga::festival::DayPart::Aparahna
        }
    );
}

/// What cannot be kept is refused by its field.
#[test]
fn a_refusal_names_the_field() {
    let (mut janma, origin) = asked(
        Masa::Kartika,
        Tithi::ShuklaPanchami,
        AnniversaryKind::JanmaTithi,
    );
    janma.window = Some(Window::Sunset);
    let days = days_of(&fifth(), &[], Masa::Kartika, MonthKind::Nija);
    let refused = anniversaries(&[(janma, origin.clone())], &days).unwrap_err();
    assert_eq!(refused.field(), Some("window"));
    let (mut shraddha, _) = asked(
        Masa::Kartika,
        Tithi::ShuklaPanchami,
        AnniversaryKind::Shraddha,
    );
    shraddha.janma.two_muhurta_floor = true;
    let refused = anniversaries(&[(shraddha, origin.clone())], &days).unwrap_err();
    assert_eq!(refused.field(), Some("janma"));
    let (mut blank, _) = asked(
        Masa::Kartika,
        Tithi::ShuklaPanchami,
        AnniversaryKind::Shraddha,
    );
    blank.key = " ".to_owned();
    assert_eq!(
        anniversaries(&[(blank, origin)], &days)
            .unwrap_err()
            .field(),
        Some("key")
    );
}
