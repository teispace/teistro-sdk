//! The relation resolver and the rite's exception held to the texts'
//! cases (`03-design/ashauch.md` §§10, 12).

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the answers they read"
)]

use teistro_core::error::Status;
use teistro_panchanga::ashauch::KinStep::{Daughter, Father, Husband, Mother, Son, Wife};
use teistro_panchanga::ashauch::{
    AshauchKind, AshauchTable, Kin, KinAsked, KinKeeps, KinObserver, KinStep, KinTable, Relation,
    Rite, RiteAsked, RiteBeginning, RiteTable, RiteVerdict, Sex, Upanayana, resolve, rite,
};

const MAN: KinObserver = KinObserver {
    sex: Sex::Male,
    married: None,
    upanayana: None,
};
const WIFE: KinObserver = KinObserver {
    sex: Sex::Female,
    married: Some(true),
    upanayana: None,
};
const GIRL: KinObserver = KinObserver {
    sex: Sex::Female,
    married: Some(false),
    upanayana: None,
};

fn kin(path: &[KinStep], observer: KinObserver) -> KinAsked {
    KinAsked {
        path: path.to_vec(),
        observer,
        person_married: None,
        living_forebears: 0,
    }
}

fn read(asked: &KinAsked) -> Kin {
    resolve(&KinTable::dharmasindhu(), asked).unwrap()
}

fn relation(path: &[KinStep], observer: KinObserver) -> Option<Relation> {
    read(&kin(path, observer)).relation
}

/// A line up `up` persons and down `down`.
fn line(up: usize, down: usize) -> Vec<KinStep> {
    let mut path = vec![Father; up];
    path.extend(vec![Son; down]);
    path
}

#[test]
fn ds_299_the_observers_own_number_decides_at_the_seventh() {
    // Standing 7th, for an 8th: ten days.
    let found = read(&kin(&line(6, 7), MAN));
    assert_eq!(found.relation, Some(Relation::Sapinda));
    let degree = found.degree.unwrap();
    assert_eq!((degree.observer, degree.person, degree.reach), (7, 8, 7));
    // Standing 8th, for a 7th: three.
    assert_eq!(relation(&line(7, 6), MAN), Some(Relation::Samanodaka));
}

#[test]
fn ds_300_ns_257_each_living_forebear_adds_a_person() {
    let mut asked = kin(&line(7, 6), MAN);
    asked.living_forebears = 1;
    assert_eq!(read(&asked).relation, Some(Relation::Sapinda));
    asked.path = line(8, 6);
    assert_eq!(read(&asked).relation, Some(Relation::Samanodaka));
    asked.living_forebears = 2;
    assert_eq!(read(&asked).relation, Some(Relation::Sapinda));
    asked.path = line(9, 6);
    asked.living_forebears = 3;
    assert_eq!(read(&asked).relation, Some(Relation::Sapinda));
    asked.path = line(10, 6);
    assert_eq!(read(&asked).relation, Some(Relation::Samanodaka));
    // A table that keeps the first view does not count them.
    let table = KinTable {
        living_extend: false,
        ..KinTable::dharmasindhu()
    };
    asked.path = line(7, 6);
    assert_eq!(
        resolve(&table, &asked).unwrap().relation,
        Some(Relation::Samanodaka)
    );
}

#[test]
fn ds_299_the_lines_bounds() {
    assert_eq!(relation(&line(13, 1), MAN), Some(Relation::Samanodaka));
    assert_eq!(relation(&line(14, 1), MAN), Some(Relation::Sagotra));
    assert_eq!(relation(&line(20, 1), MAN), Some(Relation::Sagotra));
    let past = read(&kin(&line(21, 1), MAN));
    assert_eq!((past.relation, past.keeps), (None, KinKeeps::Bath));
    assert!(past.clause.contains("p. 299"));
    assert_eq!(past.degree.unwrap().observer, 22);
}

#[test]
fn the_named_kin_of_a_man() {
    use Relation as R;
    for (path, want) in [
        (&[Father][..], R::Child),
        (&[Mother], R::Child),
        (&[Son], R::Father),
        (&[Daughter], R::Father),
        (&[Wife], R::Spouse),
        (&[Father, Wife], R::StepChild),
        (&[Mother, Father], R::MaternalGrandfather),
        (&[Mother, Mother], R::MaternalGrandmother),
        (&[Mother, Father, Son], R::MaternalUncle),
        (&[Mother, Mother, Son], R::MaternalUncle),
        (&[Mother, Mother, Daughter], R::MaternalAunt),
        (&[Mother, Father, Son, Wife], R::MaternalUnclesWife),
        (&[Father, Daughter, Son], R::SistersSon),
        (&[Daughter, Son], R::DaughtersSon),
        (&[Wife, Father], R::SonInLaw),
        (&[Wife, Mother], R::SonInLaw),
        (&[Wife, Mother, Son], R::WifesBrother),
        (&[Daughter, Husband], R::ParentInLaw),
        (&[Father, Father, Daughter, Son], R::Bandhu),
        (&[Mother, Father, Daughter, Son], R::Bandhu),
        (&[Mother, Father, Son, Son], R::Bandhu),
    ] {
        assert_eq!(relation(path, MAN), Some(want), "{path:?}");
    }
    // The father's and the mother's three bandhus.
    assert_eq!(
        relation(&[Father, Mother, Father, Son, Son], MAN),
        Some(R::Bandhu)
    );
    assert_eq!(
        relation(&[Mother, Mother, Father, Daughter, Son], MAN),
        Some(R::Bandhu)
    );
    // Every relation the rows name has a row in the table it is asked of.
    for row in KinTable::dharmasindhu().rows {
        if let Some(relation) = row.relation {
            assert!(
                AshauchTable::dharmasindhu()
                    .relations
                    .iter()
                    .any(|found| found.relation == relation),
                "{relation:?}"
            );
        }
    }
}

#[test]
fn the_named_kin_of_a_woman() {
    use Relation as R;
    assert_eq!(relation(&[Father], WIFE), Some(R::MarriedDaughter));
    assert_eq!(relation(&[Father], GIRL), Some(R::Child));
    assert_eq!(relation(&[Son], WIFE), Some(R::Mother));
    assert_eq!(relation(&[Husband], WIFE), Some(R::Spouse));
    assert_eq!(relation(&[Husband, Wife], WIFE), Some(R::CoWife));
    assert_eq!(relation(&[Father, Son], WIFE), Some(R::MarriedSibling));
    // Her line is her husband's, at his number.
    let found = read(&kin(&[Husband, Father], WIFE));
    assert_eq!(found.relation, Some(R::Sapinda));
    assert_eq!(found.degree.unwrap().observer, 2);
    // DS p. 301: her natal grandfather and uncle, a bath.
    for path in [&[Father, Father][..], &[Father, Father, Son]] {
        let found = read(&kin(path, WIFE));
        assert_eq!((found.relation, found.keeps), (None, KinKeeps::Bath));
    }
    assert_eq!(
        read(&kin(&[Father, Father, Father], WIFE)).keeps,
        KinKeeps::Bath
    );
    assert!(
        read(&kin(&[Father, Father, Father], WIFE))
            .clause
            .contains("p. 301")
    );
}

#[test]
fn ds_301_an_unbound_observer_keeps_a_parents_period_alone() {
    let boy = KinObserver {
        upanayana: Some(Upanayana::NotDone),
        ..MAN
    };
    assert_eq!(relation(&[Father], boy), Some(Relation::Child));
    assert_eq!(read(&kin(&[Father, Son], boy)).keeps, KinKeeps::Nothing);
    assert_eq!(read(&kin(&[Father, Father], GIRL)).keeps, KinKeeps::Nothing);
    // Where none is performed, marriage stands in its place.
    let unwed = KinObserver {
        upanayana: Some(Upanayana::NotPerformed),
        married: Some(false),
        ..MAN
    };
    assert_eq!(read(&kin(&[Father, Son], unwed)).keeps, KinKeeps::Nothing);
    let wed = KinObserver {
        married: Some(true),
        ..unwed
    };
    assert_eq!(relation(&[Father, Son], wed), Some(Relation::Sapinda));
}

#[test]
fn a_daughter_of_the_line_turns_on_her_marriage() {
    let mut asked = kin(&[Father, Daughter], MAN);
    let refused = resolve(&KinTable::dharmasindhu(), &asked).unwrap_err();
    assert_eq!(refused.field(), Some("personMarried"));
    asked.person_married = Some(true);
    assert_eq!(read(&asked).relation, Some(Relation::MarriedSibling));
    asked.person_married = Some(false);
    let found = read(&asked);
    assert_eq!(found.relation, Some(Relation::Sapinda));
    assert_eq!(found.within_girls_reach, Some(true));
    // Married past the sister's row, she has left the line.
    let mut far = kin(&[Father, Father, Father, Father, Son, Daughter], MAN);
    far.person_married = Some(true);
    assert_eq!(read(&far).keeps, KinKeeps::Bath);
    far.person_married = Some(false);
    assert_eq!(read(&far).within_girls_reach, Some(false));
    // A father's sister, married and not.
    let mut aunt = kin(&[Father, Father, Daughter], MAN);
    aunt.person_married = Some(true);
    assert_eq!(read(&aunt).relation, Some(Relation::PaternalAunt));
    // The woman asked of her marriage is asked by name.
    let unknown = KinObserver {
        married: None,
        ..WIFE
    };
    let refused = resolve(&KinTable::dharmasindhu(), &kin(&[Father], unknown)).unwrap_err();
    assert_eq!(refused.field(), Some("observer.married"));
}

#[test]
fn ds_301_302_the_seeming_rows() {
    let niece = read(&kin(&[Father, Daughter, Daughter], MAN));
    assert_eq!((niece.relation, niece.keeps), (None, KinKeeps::Bath));
    let granddaughter = read(&kin(&[Daughter, Daughter], MAN));
    assert_eq!(granddaughter.keeps, KinKeeps::Nothing);
}

#[test]
fn a_path_no_row_or_line_holds_is_a_bath_when_near() {
    let found = read(&kin(&[Mother, Father, Father], MAN));
    assert_eq!((found.relation, found.keeps), (None, KinKeeps::Bath));
    assert!(found.clause.contains("p. 309"));
    assert_eq!(read(&kin(&[Son, Husband], MAN)).keeps, KinKeeps::Bath);
    assert_eq!(
        read(&kin(&[Father, Husband, Son], MAN)).keeps,
        KinKeeps::Bath
    );
    // A grandmother and a grandson's wife stand in the line.
    assert_eq!(relation(&[Father, Mother], MAN), Some(Relation::Sapinda));
    assert_eq!(relation(&[Son, Son, Wife], MAN), Some(Relation::Sapinda));
}

#[test]
fn a_path_or_table_that_cannot_be_read_is_refused() {
    let table = KinTable::dharmasindhu();
    let field = |asked: &KinAsked| {
        resolve(&table, asked)
            .unwrap_err()
            .field()
            .map(str::to_owned)
    };
    assert_eq!(field(&kin(&[], MAN)).as_deref(), Some("path"));
    assert_eq!(field(&kin(&[Father; 50], MAN)).as_deref(), Some("path"));
    assert_eq!(field(&kin(&[Son, Father], MAN)).as_deref(), Some("path[1]"));
    assert_eq!(field(&kin(&[Son, Mother], MAN)).as_deref(), Some("path[1]"));
    assert_eq!(
        field(&kin(&[Wife, Husband], MAN)).as_deref(),
        Some("path[1]")
    );
    let mut many = kin(&[Father, Son], MAN);
    many.living_forebears = 4;
    assert_eq!(field(&many).as_deref(), Some("livingForebears"));
    let bad = KinTable {
        samanodaka_to: 3,
        ..KinTable::dharmasindhu()
    };
    assert!(bad.check().is_err());
    let mut rows = KinTable::dharmasindhu();
    rows.rows[0].relation = None;
    assert_eq!(rows.check().unwrap_err().field(), Some("rows[0]"));
    rows.rows[0].path.clear();
    assert!(rows.check().is_err());
}

#[test]
fn a_familys_own_row_answers_first() {
    let mut table = KinTable::dharmasindhu();
    table.rows.insert(0, table.rows[0].clone());
    table.rows[0].path = vec![Mother, Father, Father];
    table.rows[0].relation = Some(Relation::Bandhu);
    table.rows[0].clause = String::from("our custom");
    let found = resolve(&table, &kin(&[Mother, Father, Father], MAN)).unwrap();
    assert_eq!(found.relation, Some(Relation::Bandhu));
    assert_eq!(found.clause, "our custom");
}

fn asked(rite: Rite) -> RiteAsked {
    RiteAsked {
        rite,
        begun: false,
        no_other_way: false,
        calamity: false,
        kind: None,
    }
}

fn verdict(asked: RiteAsked) -> RiteVerdict {
    rite(&RiteTable::dharmasindhu(), &asked).unwrap().verdict
}

#[test]
fn ds_309_310_a_rite_past_its_nandi_goes_on_where_no_other_way_serves() {
    for one in [
        Rite::Marriage,
        Rite::Upanayana,
        Rite::Chudakarana,
        Rite::Pratishtha,
        Rite::TankDedication,
        Rite::KotiHoma,
        Rite::TulapurushaGift,
    ] {
        let begun = RiteAsked {
            begun: true,
            no_other_way: true,
            ..asked(one)
        };
        let found = rite(&RiteTable::dharmasindhu(), &begun).unwrap();
        assert_eq!(found.verdict, RiteVerdict::GoesOn);
        assert_eq!(found.beginning, Some(RiteBeginning::NandiShraddha));
        assert_eq!(
            verdict(RiteAsked {
                no_other_way: false,
                ..begun
            }),
            RiteVerdict::Barred
        );
        assert_eq!(verdict(asked(one)), RiteVerdict::Barred);
    }
}

#[test]
fn ds_310_a_girls_marriage_may_open_in_a_birth_period_alone() {
    let pressed = RiteAsked {
        no_other_way: true,
        kind: Some(AshauchKind::Birth),
        ..asked(Rite::Marriage)
    };
    assert_eq!(verdict(pressed), RiteVerdict::GoesOn);
    let death = RiteAsked {
        kind: Some(AshauchKind::Death),
        ..pressed
    };
    assert_eq!(verdict(death), RiteVerdict::Barred);
}

#[test]
fn ds_309_a_vow_begun_goes_on_in_part_and_the_rest_by_their_rows() {
    let begun = |one| RiteAsked {
        begun: true,
        ..asked(one)
    };
    assert_eq!(verdict(begun(Rite::Vow)), RiteVerdict::InPart);
    assert_eq!(verdict(asked(Rite::Vow)), RiteVerdict::Barred);
    for one in [
        Rite::Purashcharana,
        Rite::Penance,
        Rite::Sacrifice,
        Rite::PriestsOffice,
        Rite::Shraddha,
    ] {
        assert_eq!(verdict(begun(one)), RiteVerdict::GoesOn, "{one:?}");
        assert_eq!(verdict(asked(one)), RiteVerdict::Barred, "{one:?}");
    }
    for one in [
        Rite::Shanti,
        Rite::FoodCharity,
        Rite::EclipseRites,
        Rite::SankrantiBath,
        Rite::ParthivaWorship,
        Rite::DailyRules,
    ] {
        assert_eq!(verdict(asked(one)), RiteVerdict::GoesOn, "{one:?}");
    }
    assert_eq!(verdict(asked(Rite::Worship)), RiteVerdict::Barred);
}

#[test]
fn ds_309_a_calamity_lifts_every_bar() {
    let pressed = RiteAsked {
        calamity: true,
        ..asked(Rite::Worship)
    };
    let found = rite(&RiteTable::dharmasindhu(), &pressed).unwrap();
    assert_eq!((found.verdict, found.row), (RiteVerdict::GoesOn, 0));
}

#[test]
fn a_rite_no_row_names_is_refused_and_a_row_is_data() {
    let empty = RiteTable {
        name: String::from("OURS"),
        rows: Vec::new(),
    };
    let refused = rite(&empty, &asked(Rite::Worship)).unwrap_err();
    assert_eq!(refused.status, Status::Unsupported);
    let mut table = RiteTable::dharmasindhu();
    table.rows.retain(|row| row.no_other_way != Some(true));
    // The others' reading: past the nāndī, no plea of no other way.
    table
        .rows
        .insert(0, RiteTable::dharmasindhu().rows[2].clone());
    table.rows[0].no_other_way = None;
    let begun = RiteAsked {
        begun: true,
        ..asked(Rite::Marriage)
    };
    assert_eq!(rite(&table, &begun).unwrap().verdict, RiteVerdict::GoesOn);
}
