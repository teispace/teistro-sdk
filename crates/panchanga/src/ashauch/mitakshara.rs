//! The *Mitākṣarā*'s table: Vijñāneśvara on *Yājñavalkya-smṛti*
//! III.18–25 (`03-design/ashauch.md` §2.10). Every row Mit settles is its
//! own, cited by verse; the rows it leaves to other texts are the
//! *Dharmasindhu*'s, each still citing DS. The commentary's e-text is not
//! copied: the clauses are this crate's own words.

use super::dharmasindhu::{band, days, late, relations as dharmasindhu_relations, row};
use super::{
    AshauchKind, AshauchTable, Band, Length, Mark, MotherRites, Nearness, NightRule, Relation,
    RelationRow, Sex,
};

use AshauchKind::Death;
use Length::{Bath, Full, Pakshini};
use Nearness::{ObserversHouse, OtherVillage};

/// The relations Mit gives its own length for, whose DS rows it replaces.
const OWN: [Relation; 8] = [
    Relation::Pupil,
    Relation::MaternalUncle,
    Relation::Bandhu,
    Relation::MaternalGrandfather,
    Relation::SonInLaw,
    Relation::ParentInLaw,
    Relation::MarriedSibling,
    Relation::Adoption,
];

impl AshauchTable {
    /// The name [`AshauchTable::mitakshara`] answers under.
    ///
    /// ```
    /// use teistro_panchanga::ashauch::AshauchTable;
    ///
    /// assert_eq!(AshauchTable::mitakshara().name, AshauchTable::MITAKSHARA);
    /// ```
    pub const MITAKSHARA: &'static str = "MITAKSHARA";

    /// The *Mitākṣarā*'s table (`03-design/ashauch.md` §2.10): the night
    /// by its first kalpa, the middle; the parents of a child under two
    /// keeping the full period (on III.18); a child's bands cut by the
    /// *cūḍākaraṇa* (III.23–24); its own lengths for the teacher, the
    /// maternal kin, the in-laws, a married sibling and an adopted son
    /// (III.24–25); late news by Vṛddha-Vasiṣṭha's months, a bath past the
    /// ninth, and a bath for news from another land (on III.21); and the
    /// mother's bar to the 20th or 30th day (Paiṭhīnasi, on III.19).
    /// Every other row is the *Dharmasindhu*'s.
    ///
    /// ```
    /// use teistro_panchanga::ashauch::{AshauchTable, MotherRites, NightRule};
    ///
    /// // The night halved, and the mother's bar to the 20th or 30th day.
    /// let table = AshauchTable::mitakshara();
    /// assert_eq!(table.night, NightRule::HalfNight);
    /// assert_eq!(table.mother_rites, Some(MotherRites { son: 20, daughter: 30 }));
    /// assert!(table.late_abroad.is_empty());
    /// table.check()?;
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn mitakshara() -> AshauchTable {
        let texts = AshauchTable::dharmasindhu();
        AshauchTable {
            name: AshauchTable::MITAKSHARA.to_owned(),
            night: NightRule::HalfNight,
            relations: relations(),
            bands: bands(),
            late: vec![late(90, days(3)), late(180, Pakshini), late(270, days(1))],
            late_abroad: Vec::new(),
            mother_rites: Some(MotherRites {
                son: 20,
                daughter: 30,
            }),
            ..texts
        }
    }
}

fn relations() -> Vec<RelationRow> {
    use Relation::{
        Adoption, Bandhu, Father, MarriedSibling, MaternalGrandfather, MaternalUncle, Mother,
        ParentInLaw, Pupil, SonInLaw,
    };
    let mut rows: Vec<RelationRow> = dharmasindhu_relations()
        .into_iter()
        .filter(|row| !OWN.contains(&row.relation))
        .collect();
    // A married daughter dying away from her father's house: none on her
    // father's side, after the rows for one dying in it.
    let at_home = rows
        .iter()
        .rposition(|row| row.married_daughter == Some(true))
        .map_or(0, |index| index + 1);
    rows.splice(
        at_home..at_home,
        [Mother, Father].map(|parent| {
            row(parent, Death, Bath, "Mitakshara on Yājñavalkya III.24 (Viṣṇu 22.33): a married daughter, none on her father's side unless she dies in his house")
                .married_daughter(true)
        }),
    );
    rows.extend([
        row(Pupil, Death, days(3), "Mitakshara on Yājñavalkya III.24 (Manu V.80): the ācārya, three nights"),
        row(MaternalUncle, Death, days(1), "Yājñavalkya III.24: the maternal uncle, a day; the Mitakshara keeps it for one not near")
            .at(OtherVillage),
        row(MaternalUncle, Death, Pakshini, "Mitakshara on Yājñavalkya III.24 (Manu V.81): the maternal uncle near, a pakṣiṇī"),
        row(Bandhu, Death, days(1), "Mitakshara on Yājñavalkya III.24: the kin by the mother or the father, as the maternal uncle, a day when not near")
            .at(OtherVillage),
        row(Bandhu, Death, Pakshini, "Mitakshara on Yājñavalkya III.24 (Manu V.81): the bāndhavas near, a pakṣiṇī"),
        row(MaternalGrandfather, Death, days(3), "Mitakshara on Yājñavalkya III.24 (Bṛhaspati): the maternal grandfather, three days"),
        row(SonInLaw, Death, Pakshini, "Mitakshara on Yājñavalkya III.24 (Vṛddha-Vasiṣṭha): a father- or mother-in-law, a pakṣiṇī"),
        row(ParentInLaw, Death, Bath, "Mitakshara on Yājñavalkya III.24: a son-in-law, a bath"),
        row(MarriedSibling, Death, days(1), "Mitakshara on Yājñavalkya III.24 (Viṣṇu 22.46): one not a sapiṇḍa dying in one's house, a night")
            .at(ObserversHouse),
        row(MarriedSibling, Death, Bath, "Mitakshara on Yājñavalkya III.24: a married sister or brother, a bath"),
        row(Adoption, Death, days(1), "Yājñavalkya III.25: a son not of one's body, a day; the Mitakshara keeps it for one not near")
            .at(OtherVillage),
        row(Adoption, Death, days(3), "Mitakshara on Yājñavalkya III.25 (Viṣṇu 22.43): a son not of one's body near, three nights"),
    ]);
    rows
}

/// A band holding only when the *cūḍākaraṇa* was or was not done.
fn chuda(done: bool, band: Band) -> Band {
    Band {
        chudakarana: Some(done),
        ..band
    }
}

/// III.18 on the Mitakshara's first reading: a child under two, the
/// parents the full period; the 3rd year begins at two.
const UNDER_TWO: Mark = Mark::Year { year: 3 };

fn bands() -> Vec<Band> {
    let mut bands = sons();
    bands.extend(daughters());
    bands
}

fn sons() -> Vec<Band> {
    vec![
        band(
            Sex::Male,
            Mark::Day { day: 12 },
            Full,
            Bath,
            "Mitakshara on Yājñavalkya III.18, III.23 (Śaṅkha): a son before the naming, the sapiṇḍas a bath; a child under two, the parents the full period",
        ),
        Band {
            buried: Some(false),
            ..band(
                Sex::Male,
                Mark::Month { month: 7 },
                Full,
                days(1),
                "Mitakshara on Yājñavalkya III.23–24: a son from the naming to teething, cremated, a day; the parents the full period",
            )
        },
        Band {
            buried: Some(true),
            ..band(
                Sex::Male,
                Mark::Month { month: 7 },
                Full,
                Bath,
                "Mitakshara on Yājñavalkya III.23: a son before teething, not cremated, a bath; the parents the full period",
            )
        },
        chuda(
            false,
            band(
                Sex::Male,
                UNDER_TWO,
                Full,
                days(1),
                "Mitakshara on Yājñavalkya III.23 (Viṣṇu 22.29): a son with teeth, before his cūḍā, a day; under two, the parents the full period",
            ),
        ),
        chuda(
            true,
            band(
                Sex::Male,
                UNDER_TWO,
                Full,
                days(3),
                "Mitakshara on Yājñavalkya III.23: a son after his cūḍā, three days; under two, the parents the full period",
            ),
        ),
        band(
            Sex::Male,
            Mark::Chudakarana,
            days(3),
            days(1),
            "Mitakshara on Yājñavalkya III.23: a son past two before his cūḍā, to the third year's end, a day; the parents three (Kaśyapa)",
        ),
        Band {
            samanodakas: Some(Bath),
            ..band(
                Sex::Male,
                Mark::Upanayana,
                days(3),
                days(3),
                "Mitakshara on Yājñavalkya III.23: a son after his cūḍā, or past three years, to the upanayana, three days",
            )
        },
    ]
}

fn daughters() -> Vec<Band> {
    vec![
        chuda(
            false,
            band(
                Sex::Female,
                UNDER_TWO,
                Full,
                Bath,
                "Mitakshara on Yājñavalkya III.24 (Āpastamba): a daughter before her cūḍā, a bath; under two, the parents the full period",
            ),
        ),
        chuda(
            true,
            band(
                Sex::Female,
                UNDER_TWO,
                Full,
                days(1),
                "Mitakshara on Yājñavalkya III.24: a daughter after her cūḍā, before her betrothal, a day; under two, the parents the full period",
            ),
        ),
        band(
            Sex::Female,
            Mark::Chudakarana,
            days(3),
            Bath,
            "Mitakshara on Yājñavalkya III.24 (Āpastamba): a daughter past two before her cūḍā, a bath; the parents three",
        ),
        band(
            Sex::Female,
            Mark::Betrothal,
            days(3),
            days(1),
            "Mitakshara on Yājñavalkya III.24: a daughter after her cūḍā, before her betrothal, the sapiṇḍas of three generations a day",
        ),
        band(
            Sex::Female,
            Mark::Marriage,
            days(3),
            days(3),
            "Mitakshara on Yājñavalkya III.24 (Manu V.72): a daughter betrothed, before her marriage, three days on either side",
        ),
    ]
}
