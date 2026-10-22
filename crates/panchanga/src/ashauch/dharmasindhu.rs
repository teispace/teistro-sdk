//! The *Dharmasindhu*'s table (third pariccheda, uttarārdha, §§35–38;
//! the 1888 Nirnaya-sagara edition's pp. 298–306), every row with its
//! page (`03-design/ashauch.md` §§2.2–2.7).

use super::{
    AshauchKind, AshauchTable, Band, LateRow, Length, LossRow, Mark, MotherRites, Nearness,
    NightRule, Overlap, PakshiniNight, Relation, RelationRow, Sex,
};

use AshauchKind::{Birth, Death};
use Length::{Bath, Full, Pakshini, Sajyotih};
use Nearness::{ObserversHouse, OtherVillage, SameVillage};

pub(super) const fn days(days: u8) -> Length {
    Length::Days { days }
}

/// One relation row, at any placement and for any deceased.
pub(super) fn row(
    relation: Relation,
    kind: AshauchKind,
    length: Length,
    clause: &str,
) -> RelationRow {
    RelationRow {
        relation,
        kind,
        nearness: None,
        married_daughter: None,
        length,
        heard_late: None,
        clause: clause.to_owned(),
    }
}

impl RelationRow {
    pub(super) fn at(self, nearness: Nearness) -> RelationRow {
        RelationRow {
            nearness: Some(nearness),
            ..self
        }
    }

    pub(super) fn married_daughter(self, married: bool) -> RelationRow {
        RelationRow {
            married_daughter: Some(married),
            ..self
        }
    }

    fn heard_late(self, length: Length) -> RelationRow {
        RelationRow {
            heard_late: Some(length),
            ..self
        }
    }
}

pub(super) fn band(
    sex: Sex,
    before: Mark,
    parents: Length,
    sapindas: Length,
    clause: &str,
) -> Band {
    Band {
        sex,
        before,
        buried: None,
        chudakarana: None,
        parents,
        sapindas,
        samanodakas: None,
        clause: clause.to_owned(),
    }
}

fn loss(to_month: u8, mother: Length, others: Length, clause: &str) -> LossRow {
    LossRow {
        to_month,
        mother,
        others,
        clause: clause.to_owned(),
    }
}

pub(super) const fn late(within_days: u16, length: Length) -> LateRow {
    LateRow {
        within_days,
        length,
    }
}

/// A sapiṇḍa's full period, which every row naming one cites.
const SAPINDA: &str =
    "Dharmasindhu p. 299 (birth), p. 301 (death): sapiṇḍas to the 7th generation, the full period";

impl AshauchTable {
    /// The name [`AshauchTable::dharmasindhu`] answers under.
    pub const DHARMASINDHU: &'static str = "DHARMASINDHU";

    /// The *Dharmasindhu*'s table, the default (`03-design/ashauch.md`
    /// F1): the night in thirds, its relation rows, the age bands of a son
    /// and a daughter, the losses, the late tables (its own at home,
    /// Mādhava's abroad, which it prefers), the last night's two days and
    /// its last quarter's three, and the mother's bar to the 30th or 40th
    /// day.
    #[must_use]
    pub fn dharmasindhu() -> AshauchTable {
        AshauchTable {
            name: AshauchTable::DHARMASINDHU.to_owned(),
            night: NightRule::Thirds,
            pakshini_at_night: PakshiniNight::SecondSunrise,
            relations: relations(),
            bands: bands(),
            losses: vec![
                loss(
                    3,
                    days(3),
                    Bath,
                    "Dharmasindhu p. 299: a loss in the first three months (srāva), the mother three nights, the father and the sapiṇḍas a bath",
                ),
                loss(
                    4,
                    days(4),
                    Bath,
                    "Dharmasindhu p. 299: a loss in the fourth month, the mother four nights, the others a bath",
                ),
                loss(
                    5,
                    days(5),
                    days(3),
                    "Dharmasindhu p. 299: a loss in the fifth month (pāta), the mother five days, the father and the sapiṇḍas three",
                ),
                loss(
                    6,
                    days(6),
                    days(3),
                    "Dharmasindhu p. 299: a loss in the sixth month (pāta), the mother six days, the father and the sapiṇḍas three",
                ),
            ],
            late: vec![
                late(90, days(3)),
                late(180, Pakshini),
                late(270, days(1)),
                late(360, Sajyotih),
            ],
            late_abroad: vec![late(45, days(3)), late(180, Pakshini), late(360, days(1))],
            overlap: Overlap {
                last_night: [2, 3],
                death_absorbs_longer_birth: false,
            },
            mother_rites: Some(MotherRites {
                son: 30,
                daughter: 40,
            }),
        }
    }
}

pub(super) fn relations() -> Vec<RelationRow> {
    use Relation::{
        AcharyasFamily, Adoption, Bandhu, Child, CoWife, Father, MarriedDaughter, MarriedSibling,
        MaternalGrandfather, MaternalGrandmother, MaternalUncle, Mother, OfAscetic, ParentInLaw,
        Pupil, Sagotra, Samanodaka, Sapinda, SonInLaw, Spouse, StepChild, Subject,
    };
    vec![
        row(Mother, Birth, Full, "Dharmasindhu p. 299: the mother of a birth from the 7th month, the full period, untouchable throughout"),
        row(Father, Birth, Full, "Dharmasindhu p. 299: the father keeps the full birth period, touchable after a bath"),
        row(Sapinda, Birth, Full, SAPINDA),
        row(Samanodaka, Birth, days(3), "Dharmasindhu p. 299: samānodakas three nights"),
        row(Sagotra, Birth, days(1), "Dharmasindhu p. 299: sagotras one night (the Nāgojībhaṭṭīya; others say none)"),
        row(Child, Death, Full, "Dharmasindhu p. 301: a parent's death, the full period; p. 304: heard however late, the full period from the hearing")
            .heard_late(Full),
        row(Spouse, Death, Full, "Dharmasindhu p. 304: husband and wife, the full period from the hearing however late")
            .heard_late(Full),
        row(CoWife, Death, Full, "Dharmasindhu p. 304: co-wives, the full period from the hearing however late")
            .heard_late(Full),
        row(StepChild, Death, Full, "Dharmasindhu p. 304: a step-mother's death, the full period; heard after it, three days")
            .heard_late(days(3)),
        row(Mother, Death, days(3), "Dharmasindhu p. 301: a married daughter dying at her father's house, her parents three days")
            .married_daughter(true)
            .at(ObserversHouse),
        row(Father, Death, days(3), "Dharmasindhu p. 301: a married daughter dying at her father's house, her parents three days")
            .married_daughter(true)
            .at(ObserversHouse),
        row(Mother, Death, Full, SAPINDA).married_daughter(false),
        row(Father, Death, Full, SAPINDA).married_daughter(false),
        row(Sapinda, Death, Full, SAPINDA),
        row(Samanodaka, Death, days(3), "Dharmasindhu p. 299: samānodakas three nights"),
        row(Sagotra, Death, days(1), "Dharmasindhu p. 299: sagotras one night (the Nāgojībhaṭṭīya; others say none)"),
        row(MarriedDaughter, Death, days(3), "Dharmasindhu p. 301: a parent's death, a married daughter three days inside the ten; p. 303: a pakṣiṇī after")
            .heard_late(Pakshini),
        row(MarriedSibling, Death, days(3), "Dharmasindhu p. 301: a married brother or sister dying in one's house, three days")
            .at(ObserversHouse),
        row(MarriedSibling, Death, days(1), "Dharmasindhu p. 301: a married brother or sister dying in another village, one day")
            .at(OtherVillage),
        row(MarriedSibling, Death, Pakshini, "Dharmasindhu p. 301: a married brother or sister, a pakṣiṇī")
            .at(SameVillage),
        row(MaternalGrandfather, Death, Pakshini, "Dharmasindhu p. 301: the maternal grandfather dying in another village, a pakṣiṇī")
            .at(OtherVillage),
        row(MaternalGrandfather, Death, days(3), "Dharmasindhu p. 301: the maternal grandfather, three days"),
        row(MaternalGrandmother, Death, Pakshini, "Dharmasindhu p. 301: the maternal grandmother, a pakṣiṇī"),
        row(MaternalUncle, Death, days(3), "Dharmasindhu p. 301: the maternal uncle dying in one's house, three days")
            .at(ObserversHouse),
        row(MaternalUncle, Death, Pakshini, "Dharmasindhu p. 301: the maternal uncle, a pakṣiṇī"),
        row(SonInLaw, Death, Pakshini, "Dharmasindhu p. 302: a father- or mother-in-law, to a son-in-law not present, a pakṣiṇī")
            .at(OtherVillage),
        row(SonInLaw, Death, days(3), "Dharmasindhu p. 302: a father- or mother-in-law, to a son-in-law present, three days"),
        row(ParentInLaw, Death, days(1), "Dharmasindhu p. 302: a son-in-law, to his parents-in-law, one day (or a bath)"),
        row(Pupil, Death, Pakshini, "Dharmasindhu p. 302: the ācārya dying in another village, a pakṣiṇī")
            .at(OtherVillage),
        row(Pupil, Death, days(3), "Dharmasindhu p. 302, Manu V.80: the ācārya, three days"),
        row(AcharyasFamily, Death, days(1), "Dharmasindhu p. 303, Manu V.80: the ācārya's wife or son, one day"),
        row(Adoption, Death, days(3), "Dharmasindhu p. 302: an adopted son and his natural or adoptive parents, three days"),
        row(Bandhu, Death, days(3), "Dharmasindhu p. 302: one of the three bandhus dying in one's house, three days")
            .at(ObserversHouse),
        row(Bandhu, Death, Pakshini, "Dharmasindhu p. 302: one of the three bandhus, a pakṣiṇī (one day for one not upanīta)"),
        row(Subject, Death, Sajyotih, "Dharmasindhu p. 303, Manu V.82: the ruler of the land, sajyotiḥ"),
        row(OfAscetic, Death, Bath, "Dharmasindhu p. 303: an ascetic's sapiṇḍas, a bath"),
    ]
}

fn bands() -> Vec<Band> {
    vec![
        band(
            Sex::Male,
            Mark::Day { day: 12 },
            days(3),
            Bath,
            "Dharmasindhu p. 300: a son before the naming (the 12th day), buried; the sapiṇḍas a bath, the parents three days",
        ),
        Band {
            buried: Some(false),
            ..band(
                Sex::Male,
                Mark::Month { month: 7 },
                days(3),
                days(1),
                "Dharmasindhu p. 300: a son from the naming to teething (the 7th month), cremated; the sapiṇḍas one day, the parents three",
            )
        },
        Band {
            buried: Some(true),
            ..band(
                Sex::Male,
                Mark::Month { month: 7 },
                days(3),
                Bath,
                "Dharmasindhu p. 300: a son from the naming to teething, buried; the sapiṇḍas a bath, the parents three",
            )
        },
        band(
            Sex::Male,
            Mark::Chudakarana,
            days(3),
            days(1),
            "Dharmasindhu p. 300: a son from teething to the cūḍā (or the third year's end); the sapiṇḍas one day, the parents three",
        ),
        Band {
            samanodakas: Some(Bath),
            ..band(
                Sex::Male,
                Mark::Upanayana,
                days(3),
                days(3),
                "Dharmasindhu p. 300: a son from the cūḍā to the upanayana (where it is not performed, to marriage or the 16th year, p. 301); every sapiṇḍa three days, the samānodakas a bath",
            )
        },
        band(
            Sex::Female,
            Mark::Month { month: 7 },
            days(1),
            Bath,
            "Dharmasindhu p. 300: a daughter to the 6th month; the sapiṇḍas a bath, the parents one day",
        ),
        band(
            Sex::Female,
            Mark::Year { year: 4 },
            days(3),
            Bath,
            "Dharmasindhu p. 300: a daughter from the 7th month to the third year's end; the sapiṇḍas a bath, the parents three days",
        ),
        band(
            Sex::Female,
            Mark::Betrothal,
            days(3),
            days(1),
            "Dharmasindhu p. 301: a daughter past three years, before her betrothal; the sapiṇḍas of three generations one day, the parents three",
        ),
        band(
            Sex::Female,
            Mark::Marriage,
            days(3),
            days(3),
            "Dharmasindhu p. 301: a daughter betrothed, before her marriage; the father's sapiṇḍas three days",
        ),
    ]
}
