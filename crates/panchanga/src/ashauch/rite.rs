//! May this rite go on? (SR-28, `03-design/ashauch.md` §12): the
//! *apavāda* of a rite during a period, answered with its clause
//! (*Dharmasindhu* pp. 309–310, *Nirṇayasindhu* p. 384). The answer
//! carries no day, and every row is data a tradition can change.

use serde::{Deserialize, Serialize};
use teistro_core::error::{Error, Status};

use super::AshauchKind;

/// A rite a table can answer for.
///
/// ```
/// use teistro_panchanga::ashauch::Rite;
///
/// assert_eq!(serde_json::to_string(&Rite::Marriage)?, r#""MARRIAGE""#);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "AshauchRite"))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Rite {
    /// A marriage.
    Marriage,
    /// An upanayana.
    Upanayana,
    /// A cūḍākaraṇa.
    Chudakarana,
    /// The installing of an image.
    Pratishtha,
    /// The dedication of a tank.
    TankDedication,
    /// A *koṭi-homa*.
    KotiHoma,
    /// A *tulāpuruṣa* gift.
    TulapurushaGift,
    /// A vow taken, such as Ananta's or an Ekadashi.
    Vow,
    /// A *puraścaraṇa* japa under its sankalpa.
    Purashcharana,
    /// A penance begun.
    Penance,
    /// A sacrifice, by the one consecrated for it.
    Sacrifice,
    /// A sacrifice, by a priest engaged for it.
    PriestsOffice,
    /// A rite to allay disease, danger or the king's anger.
    Shanti,
    /// Food given from a standing charity.
    FoodCharity,
    /// A shraddha.
    Shraddha,
    /// The bath, shraddha and gift of an eclipse.
    EclipseRites,
    /// A sankranti's bath.
    SankrantiBath,
    /// The worship of an earthen liṅga.
    ParthivaWorship,
    /// The body's daily rules: bathing, sipping, food, touch.
    DailyRules,
    /// Worship and every other rite, daily, occasional or desired.
    Worship,
}

/// What opens a rite, past which it is begun (Laghu-Viṣṇu in
/// *Nirṇayasindhu* p. 384).
///
/// ```
/// use teistro_panchanga::ashauch::RiteBeginning;
///
/// assert_ne!(RiteBeginning::NandiShraddha, RiteBeginning::Sankalpa);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RiteBeginning {
    /// The *nāndīśrāddha*, of a marriage and the like.
    NandiShraddha,
    /// The sankalpa, of a vow or a japa.
    Sankalpa,
    /// The choosing of the priests, of a sacrifice; a priest's own office
    /// from his *madhuparka*.
    Varana,
    /// The cooking, of a shraddha.
    Cooking,
}

/// The answer's verdict.
///
/// ```
/// use teistro_panchanga::ashauch::RiteVerdict;
///
/// assert_ne!(RiteVerdict::GoesOn, RiteVerdict::Barred);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RiteVerdict {
    /// It goes on.
    GoesOn,
    /// Its bodily part goes on; the rest is done by another or after the
    /// period, as the clause says.
    InPart,
    /// It waits for the period's end.
    Barred,
}

/// One row: the facts it needs, and what it answers.
///
/// ```
/// use teistro_panchanga::ashauch::RiteTable;
///
/// assert!(RiteTable::dharmasindhu().rows.iter().all(|row| !row.clause.is_empty()));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "AshauchRiteRow"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RiteRow {
    /// The rite; any when left out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rite: Option<Rite>,
    /// Whether the rite must be begun, when the row turns on it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub begun: Option<bool>,
    /// Whether there must be no other way, when the row turns on it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub no_other_way: Option<bool>,
    /// Whether the land must be in calamity, when the row turns on it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub calamity: Option<bool>,
    /// The kind of period, when the row turns on it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<AshauchKind>,
    /// What opens the rite.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub beginning: Option<RiteBeginning>,
    /// The verdict.
    pub verdict: RiteVerdict,
    /// The text and page.
    pub clause: String,
}

/// A tradition's *apavāda* rows, the first that fits answering.
///
/// ```
/// use teistro_panchanga::ashauch::RiteTable;
///
/// assert_eq!(RiteTable::dharmasindhu().name, RiteTable::DHARMASINDHU);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "AshauchRiteTable"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RiteTable {
    /// The table's name, repeated in each answer.
    pub name: String,
    /// The rows.
    pub rows: Vec<RiteRow>,
}

/// What is asked.
///
/// ```
/// use teistro_panchanga::ashauch::{Rite, RiteAsked};
///
/// let asked = RiteAsked { rite: Rite::Marriage, begun: true, no_other_way: true, calamity: false, kind: None };
/// assert!(asked.begun);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "AshauchRiteAsked"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RiteAsked {
    /// The rite.
    pub rite: Rite,
    /// Whether it has passed what opens it ([`RiteBeginning`]).
    #[serde(default)]
    pub begun: bool,
    /// Whether it cannot be moved: no later time will serve.
    #[serde(default)]
    pub no_other_way: bool,
    /// Whether the land is in calamity: war, plague or famine.
    #[serde(default)]
    pub calamity: bool,
    /// The kind of the period running.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<AshauchKind>,
}

/// The answer.
///
/// ```
/// use teistro_panchanga::ashauch::{Rite, RiteAsked, RiteTable, RiteVerdict, rite};
///
/// let asked = RiteAsked { rite: Rite::Worship, begun: false, no_other_way: false, calamity: false, kind: None };
/// assert_eq!(rite(&RiteTable::dharmasindhu(), &asked)?.verdict, RiteVerdict::Barred);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "AshauchRiteAnswer"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RiteAnswer {
    /// The rite.
    pub rite: Rite,
    /// The verdict.
    pub verdict: RiteVerdict,
    /// What opens this rite, when the texts name it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub beginning: Option<RiteBeginning>,
    /// The row's index in the table.
    pub row: usize,
    /// The clause.
    pub clause: String,
    /// The table's name.
    pub table: String,
}

/// Whether `rite` goes on during a period, under `table`.
///
/// ```
/// use teistro_panchanga::ashauch::{Rite, RiteAsked, RiteTable, RiteVerdict, rite};
///
/// // A marriage past its nāndīśrāddha, with no other date, goes on.
/// let asked = RiteAsked { rite: Rite::Marriage, begun: true, no_other_way: true, calamity: false, kind: None };
/// let table = RiteTable::dharmasindhu();
/// assert_eq!(rite(&table, &asked)?.verdict, RiteVerdict::GoesOn);
/// // Not begun, it waits.
/// let asked = RiteAsked { begun: false, ..asked };
/// assert_eq!(rite(&table, &asked)?.verdict, RiteVerdict::Barred);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// `UNSUPPORTED`, with a hint, when no row of the table fits.
pub fn rite(table: &RiteTable, asked: &RiteAsked) -> Result<RiteAnswer, Error> {
    let fits = |row: &RiteRow| {
        row.rite.is_none_or(|rite| rite == asked.rite)
            && row.begun.is_none_or(|begun| begun == asked.begun)
            && row
                .no_other_way
                .is_none_or(|wanted| wanted == asked.no_other_way)
            && row.calamity.is_none_or(|wanted| wanted == asked.calamity)
            && row.kind.is_none_or(|kind| Some(kind) == asked.kind)
    };
    let (row, found) = table
        .rows
        .iter()
        .enumerate()
        .find(|(_, row)| fits(row))
        .ok_or_else(|| {
            Error::new(
                Status::Unsupported,
                format!("the {} table has no row for {:?}", table.name, asked.rite),
            )
            .with_field("rite")
            .with_hint("add a row to the table for this rite")
        })?;
    Ok(RiteAnswer {
        rite: asked.rite,
        verdict: found.verdict,
        beginning: table
            .rows
            .iter()
            .filter(|row| row.rite == Some(asked.rite))
            .find_map(|row| row.beginning),
        row,
        clause: found.clause.clone(),
        table: table.name.clone(),
    })
}

fn row(rite: Rite, verdict: RiteVerdict, clause: &str) -> RiteRow {
    RiteRow {
        rite: Some(rite),
        begun: None,
        no_other_way: None,
        calamity: None,
        kind: None,
        beginning: None,
        verdict,
        clause: clause.to_owned(),
    }
}

impl RiteRow {
    fn opened(self, beginning: RiteBeginning, begun: bool) -> RiteRow {
        RiteRow {
            beginning: Some(beginning),
            begun: Some(begun),
            ..self
        }
    }

    fn no_other_way(self, wanted: bool) -> RiteRow {
        RiteRow {
            no_other_way: Some(wanted),
            ..self
        }
    }

    fn during(self, kind: AshauchKind) -> RiteRow {
        RiteRow {
            kind: Some(kind),
            ..self
        }
    }
}

/// The three rows of a rite opened by its nāndīśrāddha.
fn after_nandi(rite: Rite, named: &str) -> [RiteRow; 2] {
    use RiteBeginning::NandiShraddha;
    [
        row(rite, RiteVerdict::GoesOn, &format!("Dharmasindhu p. 309: {named}, past the nāndīśrāddha, no impurity; p. 310: only where no other way serves")).opened(NandiShraddha, true).no_other_way(true),
        row(rite, RiteVerdict::Barred, &format!("Dharmasindhu pp. 309–310: {named} waits: the exception holds past the nāndīśrāddha and where no other way serves")).opened(NandiShraddha, true),
    ]
}

impl RiteTable {
    /// The name [`RiteTable::dharmasindhu`] answers under.
    ///
    /// ```
    /// use teistro_panchanga::ashauch::RiteTable;
    ///
    /// assert_eq!(RiteTable::DHARMASINDHU, "DHARMASINDHU");
    /// ```
    pub const DHARMASINDHU: &'static str = "DHARMASINDHU";

    /// *Dharmasindhu* §§40–41, pp. 309–310, with what opens a rite from
    /// *Nirṇayasindhu* p. 384. The rule that an exception holds only
    /// where no other way serves is the default (the *Nirṇayasindhu* and
    /// Nāgoji in DS p. 310, C959); a table dropping the row answers the
    /// others' reading.
    ///
    /// ```
    /// use teistro_panchanga::ashauch::{Rite, RiteTable};
    ///
    /// let table = RiteTable::dharmasindhu();
    /// assert!(table.rows.iter().any(|row| row.rite == Some(Rite::Vow)));
    /// ```
    #[must_use]
    pub fn dharmasindhu() -> RiteTable {
        use AshauchKind::Birth;
        use Rite as R;
        use RiteBeginning::{Cooking, Sankalpa, Varana};
        use RiteVerdict::{Barred, GoesOn, InPart};
        let mut rows = vec![RiteRow {
            rite: None,
            calamity: Some(true),
            ..row(
                R::Worship,
                GoesOn,
                "Dharmasindhu p. 309: in a great calamity of the land, war, famine and the like, purity at once; the rest of the period is kept when it passes",
            )
        }];
        rows.push(row(R::Marriage, GoesOn, "Dharmasindhu p. 310: a girl's marriage, with no other muhurta and her season near, may even open in a birth period, with the kūṣmāṇḍa homa").opened(RiteBeginning::NandiShraddha, false).no_other_way(true).during(Birth));
        for (rite, named) in [
            (R::Marriage, "a marriage"),
            (R::Upanayana, "an upanayana"),
            (R::Chudakarana, "a cūḍākaraṇa"),
            (R::Pratishtha, "an image's installing"),
            (R::TankDedication, "a tank's dedication"),
            (R::KotiHoma, "a koṭi-homa"),
            (R::TulapurushaGift, "a tulāpuruṣa gift"),
        ] {
            rows.extend(after_nandi(rite, named));
        }
        rows.extend([
            row(R::Vow, InPart, "Dharmasindhu p. 309: a vow taken: its bodily rules kept oneself, its worship by another, the feeding of brahmins after the period").opened(Sankalpa, true),
            row(R::Purashcharana, GoesOn, "Dharmasindhu p. 309: a japa under its sankalpa, kept unbroken, past its beginning, no impurity").opened(Sankalpa, true),
            row(R::Penance, GoesOn, "Dharmasindhu p. 309: a penance begun, none while it is done; three nights' late impurity at its end").opened(Sankalpa, true),
            row(R::Sacrifice, GoesOn, "Dharmasindhu p. 309: the consecrated, from the dīkṣaṇīyā to the avabhṛtha bath, none in the sacrifice").opened(Varana, true),
            row(R::PriestsOffice, GoesOn, "Dharmasindhu p. 309: priests past the madhuparka, none in that rite").opened(Varana, true),
            row(R::Shraddha, GoesOn, "Nirṇayasindhu p. 384 (Laghu-Viṣṇu): a shraddha past its cooking, no impurity").opened(Cooking, true),
            row(R::Shanti, GoesOn, "Dharmasindhu p. 309: a rite to allay disease, danger or the king's anger, none"),
            row(R::FoodCharity, GoesOn, "Dharmasindhu p. 309: the giving of food by one who keeps a charity, none"),
            row(R::EclipseRites, GoesOn, "Dharmasindhu p. 310: the bath, shraddha and gifts an eclipse occasions, none; one says the bath alone"),
            row(R::SankrantiBath, GoesOn, "Dharmasindhu p. 310: a sankranti's bath and the like, none"),
            row(R::ParthivaWorship, GoesOn, "Dharmasindhu p. 310: the worship of an earthen liṅga, none"),
            row(R::DailyRules, GoesOn, "Dharmasindhu p. 310: the daily rules of bathing, sipping, food and touch, none"),
        ]);
        for rite in [
            R::Marriage,
            R::Upanayana,
            R::Chudakarana,
            R::Pratishtha,
            R::TankDedication,
            R::KotiHoma,
            R::TulapurushaGift,
            R::Vow,
            R::Purashcharana,
            R::Penance,
            R::Sacrifice,
            R::PriestsOffice,
            R::Shraddha,
            R::Worship,
        ] {
            rows.push(row(rite, Barred, "Dharmasindhu p. 310: every other rite, daily, occasional or desired, is not done in a period"));
        }
        RiteTable {
            name: RiteTable::DHARMASINDHU.to_owned(),
            rows,
        }
    }
}
