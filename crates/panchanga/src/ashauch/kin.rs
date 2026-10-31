//! The relation resolver (SR-27, `03-design/ashauch.md` §10): a path in
//! a family graph gives the [`Relation`] an ashauch asks for and the
//! degree the texts count.
//!
//! The persons are counted from the *kūṭastha*, the common forebear, who
//! is the first (*Dharmasindhu* p. 299). Whose number decides is the
//! observer's: one standing 8th keeps three days for a 7th in the other
//! line, one standing 7th keeps ten for an 8th. The sapiṇḍa's reach
//! grows by one for each of the observer's forebears living, to the
//! tenth (Śūlapāṇi on Sumantu in *Nirṇayasindhu* p. 257, which DS p.
//! 300 prefers).
//!
//! Every named path is a row of a [`KinTable`], with its clause, so a
//! family's own reckoning is a row, never code.

use serde::{Deserialize, Serialize};
use teistro_core::error::{Error, Status};

use super::{Relation, Sex, Upanayana};

/// The longest path read: 21 persons up and as many down.
const MOST_STEPS: usize = 42;

/// The most forebears who can add to the sapiṇḍa's reach (*Nirṇayasindhu*
/// p. 257: three living, the tenth person).
const MOST_LIVING: u8 = 3;

/// One step of a path from a person to the next.
///
/// ```
/// use teistro_panchanga::ashauch::KinStep;
///
/// // A brother is a father's son.
/// let brother = [KinStep::Father, KinStep::Son];
/// assert_eq!(serde_json::to_string(&brother)?, r#"["FATHER","SON"]"#);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KinStep {
    /// This person's father.
    Father,
    /// This person's mother.
    Mother,
    /// This person's son.
    Son,
    /// This person's daughter.
    Daughter,
    /// This person's husband.
    Husband,
    /// This person's wife.
    Wife,
}

/// The observer's facts the resolver reads.
///
/// ```
/// use teistro_panchanga::ashauch::{KinObserver, Sex};
///
/// let observer = KinObserver { sex: Sex::Male, married: None, upanayana: None };
/// assert_eq!(observer.sex, Sex::Male);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KinObserver {
    /// The observer's sex.
    pub sex: Sex,
    /// Whether the observer is married, when a row turns on it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub married: Option<bool>,
    /// The observer's upanayana. Left out, the observer has had it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upanayana: Option<Upanayana>,
}

/// What is asked: the path from the observer to the person the event is
/// of, the one who died or was born.
///
/// ```
/// use teistro_panchanga::ashauch::{KinAsked, KinObserver, KinStep, Sex};
///
/// let asked = KinAsked {
///     path: vec![KinStep::Father, KinStep::Son],
///     observer: KinObserver { sex: Sex::Male, married: None, upanayana: None },
///     person_married: None,
///     living_forebears: 0,
/// };
/// assert_eq!(asked.path.len(), 2);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KinAsked {
    /// The steps, the observer's first.
    pub path: Vec<KinStep>,
    /// The observer.
    pub observer: KinObserver,
    /// Whether the person at the path's end is married, when a row turns
    /// on it: a married daughter has left the line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub person_married: Option<bool>,
    /// The observer's forebears living, in order from the father: 1 the
    /// father, 2 the father and grandfather, 3 the great-grandfather too.
    #[serde(default)]
    pub living_forebears: u8,
}

/// What the observer keeps.
///
/// ```
/// use teistro_panchanga::ashauch::KinKeeps;
///
/// assert_ne!(KinKeeps::Period, KinKeeps::Bath);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KinKeeps {
    /// The period its [`Relation`] has in an ashauch table.
    Period,
    /// A bath, and no period.
    Bath,
    /// Nothing.
    Nothing,
}

/// One named path of a [`KinTable`], with its clause.
///
/// ```
/// use teistro_panchanga::ashauch::KinTable;
///
/// let table = KinTable::dharmasindhu();
/// assert!(table.rows.iter().all(|row| !row.path.is_empty()));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KinRow {
    /// The path, a step through a mother to her child written through
    /// the father.
    pub path: Vec<KinStep>,
    /// The observer's sex, when the row turns on it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observer: Option<Sex>,
    /// Whether the observer is married, when the row turns on it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observer_married: Option<bool>,
    /// Whether the person is married, when the row turns on it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub person_married: Option<bool>,
    /// The relation an ashauch asks for, when the row keeps a period.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relation: Option<Relation>,
    /// What is kept.
    pub keeps: KinKeeps,
    /// The text and page.
    pub clause: String,
}

/// A tradition's reckoning of kin, every row and bound data.
///
/// ```
/// use teistro_panchanga::ashauch::KinTable;
///
/// let table = KinTable::dharmasindhu();
/// assert_eq!((table.sapinda_to, table.samanodaka_to, table.sagotra_to), (7, 14, 21));
/// table.check()?;
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KinTable {
    /// The table's name, repeated in each answer.
    pub name: String,
    /// The last person who is a sapiṇḍa, the kūṭastha first.
    pub sapinda_to: u8,
    /// The last person who is a samānodaka.
    pub samanodaka_to: u8,
    /// The last person who is a sagotra.
    pub sagotra_to: u8,
    /// Whether each living forebear adds a person to the sapiṇḍa's reach.
    pub living_extend: bool,
    /// The last person who keeps a period for an unmarried girl.
    pub girl_to: u8,
    /// The named paths, the first that fits answering.
    pub rows: Vec<KinRow>,
    /// The clause for a line counted out.
    pub line: String,
    /// The clause for a person past the last sagotra.
    pub beyond: String,
    /// The clause for an observer who keeps only a parent's period.
    pub unbound: String,
    /// The clause for a married daughter's natal line.
    pub natal: String,
    /// The clause for a path no row names and no line holds.
    pub unnamed: String,
}

/// The degree of a line counted out.
///
/// ```
/// use teistro_panchanga::ashauch::KinDegree;
///
/// let degree = KinDegree { observer: 8, person: 7, reach: 7 };
/// assert!(degree.observer > degree.reach);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KinDegree {
    /// The observer's number from the kūṭastha, who is 1.
    pub observer: u8,
    /// The person's number from the kūṭastha.
    pub person: u8,
    /// The last person who is a sapiṇḍa here, the living forebears added.
    pub reach: u8,
}

/// The answer: the relation to ask an ashauch with, and why.
///
/// ```
/// use teistro_panchanga::ashauch::{KinAsked, KinKeeps, KinObserver, KinStep, KinTable, Relation, Sex, resolve};
///
/// let asked = KinAsked {
///     path: vec![KinStep::Father, KinStep::Father, KinStep::Son, KinStep::Son],
///     observer: KinObserver { sex: Sex::Male, married: None, upanayana: None },
///     person_married: None,
///     living_forebears: 0,
/// };
/// let found = resolve(&KinTable::dharmasindhu(), &asked)?;
/// assert_eq!((found.relation, found.keeps), (Some(Relation::Sapinda), KinKeeps::Period));
/// assert_eq!(found.degree.map(|degree| degree.observer), Some(3));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Kin {
    /// The relation, when a period is kept.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relation: Option<Relation>,
    /// What is kept.
    pub keeps: KinKeeps,
    /// The degree, when the path is one line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub degree: Option<KinDegree>,
    /// For an unmarried daughter of the line, whether the observer stands
    /// within the persons her bands name (`03-design/ashauch.md` C956).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub within_girls_reach: Option<bool>,
    /// The clause.
    pub clause: String,
    /// The table's name.
    pub table: String,
}

/// A line read off a path: the steps up to the kūṭastha, the steps down
/// from him, and whether it ends on a daughter of the line.
struct Line {
    up: u8,
    down: u8,
    daughter: bool,
}

/// A step through a mother to her child is a step through the father:
/// the line turns on the child's parent, not on which parent is named.
fn through_father(path: &[KinStep]) -> Vec<KinStep> {
    let next = path.iter().skip(1).map(Some).chain([None]);
    path.iter()
        .zip(next)
        .map(|(&step, next)| match (step, next) {
            (KinStep::Mother, Some(KinStep::Son | KinStep::Daughter)) => KinStep::Father,
            _ => step,
        })
        .collect()
}

/// The line `path` walks, when it is one: up through fathers, down
/// through sons, and at most a wife or a daughter at its end.
fn line_of(path: &[KinStep]) -> Option<Line> {
    let mut steps = path.iter().copied().peekable();
    let (mut up, mut down) = (0, 0);
    while steps.next_if_eq(&KinStep::Father).is_some() {
        up += 1;
    }
    while steps.next_if_eq(&KinStep::Son).is_some() {
        down += 1;
    }
    let last = steps.next();
    if steps.next().is_some() {
        return None;
    }
    let daughter = match last {
        None | Some(KinStep::Wife) => false,
        Some(KinStep::Mother) if down == 0 => false,
        Some(KinStep::Daughter) => {
            down += 1;
            true
        }
        Some(_) => return None,
    };
    Some(Line { up, down, daughter })
}

/// A fact a row turns on and the request leaves out.
fn asked_for(fact: Option<bool>, field: &str) -> Result<bool, Error> {
    fact.ok_or_else(|| {
        Error::invalid_arg(format!("this path turns on `{field}`, so say it")).with_field(field)
    })
}

/// The first row of `table` that fits. A row whose path and sex fit and
/// that turns on a fact left out asks for it.
fn named<'t>(
    table: &'t KinTable,
    path: &[KinStep],
    asked: &KinAsked,
) -> Result<Option<&'t KinRow>, Error> {
    for row in &table.rows {
        if row.path != path || row.observer.is_some_and(|sex| sex != asked.observer.sex) {
            continue;
        }
        let observer = row.observer_married.map_or(Ok(true), |wanted| {
            asked_for(asked.observer.married, "observer.married").map(|fact| fact == wanted)
        })?;
        if !observer {
            continue;
        }
        let person = row.person_married.map_or(Ok(true), |wanted| {
            asked_for(asked.person_married, "personMarried").map(|fact| fact == wanted)
        })?;
        if person {
            return Ok(Some(row));
        }
    }
    Ok(None)
}

/// Whether the observer keeps periods beyond a parent's: a man after his
/// upanayana, a woman once married (*Dharmasindhu* p. 301). Where no
/// upanayana is performed, marriage stands in its place (p. 301).
fn bound(observer: KinObserver) -> bool {
    match (observer.sex, observer.upanayana) {
        (Sex::Female, _) | (Sex::Male, Some(Upanayana::NotPerformed)) => {
            observer.married != Some(false)
        }
        (Sex::Male, Some(Upanayana::NotDone)) => false,
        (Sex::Male, _) => true,
    }
}

/// The relation and degree `asked` resolves to under `table`.
///
/// ```
/// use teistro_panchanga::ashauch::{KinAsked, KinObserver, KinStep, KinTable, Relation, Sex, resolve};
///
/// // Dharmasindhu p. 299: the observer stands 8th, the deceased 7th.
/// let mut path = vec![KinStep::Father; 7];
/// path.extend([KinStep::Son; 6]);
/// let mut asked = KinAsked {
///     path,
///     observer: KinObserver { sex: Sex::Male, married: None, upanayana: None },
///     person_married: None,
///     living_forebears: 0,
/// };
/// let table = KinTable::dharmasindhu();
/// assert_eq!(resolve(&table, &asked)?.relation, Some(Relation::Samanodaka));
/// // His father living, ten days (DS p. 300, on Śūlapāṇi's reading).
/// asked.living_forebears = 1;
/// assert_eq!(resolve(&table, &asked)?.relation, Some(Relation::Sapinda));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// `INVALID_ARG` on an empty or overlong path, a path that steps back on
/// itself, more forebears than count, a table whose bounds do not rise,
/// or a fact a row turns on and the request leaves out, named by that
/// field.
pub fn resolve(table: &KinTable, asked: &KinAsked) -> Result<Kin, Error> {
    table.check()?;
    check(asked)?;
    let answer = |relation, keeps, clause: &str| Kin {
        relation,
        keeps,
        degree: None,
        within_girls_reach: None,
        clause: clause.to_owned(),
        table: table.name.clone(),
    };
    let path = through_father(&asked.path);
    let parent = matches!(path.as_slice(), [KinStep::Father | KinStep::Mother]);
    if !parent && !bound(asked.observer) {
        return Ok(answer(None, KinKeeps::Nothing, &table.unbound));
    }
    if let Some(row) = named(table, &path, asked)? {
        return Ok(answer(row.relation, row.keeps, &row.clause));
    }
    let wife = asked.observer.sex == Sex::Female && path.first() == Some(&KinStep::Husband);
    let walked = if wife {
        path.get(1..).unwrap_or(&[])
    } else {
        &path
    };
    let Some(line) = line_of(walked) else {
        return Ok(answer(None, KinKeeps::Bath, &table.unnamed));
    };
    if line.daughter && asked_for(asked.person_married, "personMarried")? {
        return Ok(answer(None, KinKeeps::Bath, &table.unnamed));
    }
    if !wife
        && line.up > 0
        && asked.observer.sex == Sex::Female
        && asked.observer.married == Some(true)
    {
        return Ok(answer(None, KinKeeps::Bath, &table.natal));
    }
    let observer = line.up + 1;
    let reach = table.sapinda_to
        + if table.living_extend {
            asked.living_forebears
        } else {
            0
        };
    let (relation, clause) = if observer <= reach {
        (Relation::Sapinda, &table.line)
    } else if observer <= table.samanodaka_to {
        (Relation::Samanodaka, &table.line)
    } else if observer <= table.sagotra_to {
        (Relation::Sagotra, &table.line)
    } else {
        return Ok(Kin {
            degree: Some(KinDegree {
                observer,
                person: line.down + 1,
                reach,
            }),
            ..answer(None, KinKeeps::Bath, &table.beyond)
        });
    };
    Ok(Kin {
        degree: Some(KinDegree {
            observer,
            person: line.down + 1,
            reach,
        }),
        within_girls_reach: line.daughter.then_some(observer <= table.girl_to),
        ..answer(Some(relation), KinKeeps::Period, clause)
    })
}

fn check(asked: &KinAsked) -> Result<(), Error> {
    if asked.path.is_empty() || asked.path.len() > MOST_STEPS {
        return Err(Error::invalid_arg(format!(
            "a path is 1 to {MOST_STEPS} steps, not {}",
            asked.path.len()
        ))
        .with_field("path"));
    }
    if asked.living_forebears > MOST_LIVING {
        return Err(Error::new(
            Status::InvalidArg,
            format!(
                "up to {MOST_LIVING} forebears add to the reach, not {}",
                asked.living_forebears
            ),
        )
        .with_field("livingForebears"));
    }
    for (at, pair) in asked.path.windows(2).enumerate() {
        let back = matches!(
            pair,
            [
                KinStep::Son | KinStep::Daughter,
                KinStep::Father | KinStep::Mother
            ] | [KinStep::Wife, KinStep::Husband]
        );
        if back {
            return Err(
                Error::invalid_arg("a path steps forward: this step leads back")
                    .with_field(format!("path[{}]", at + 1)),
            );
        }
    }
    Ok(())
}

impl KinTable {
    /// The name [`KinTable::dharmasindhu`] answers under.
    ///
    /// ```
    /// use teistro_panchanga::ashauch::KinTable;
    ///
    /// assert_eq!(KinTable::dharmasindhu().name, KinTable::DHARMASINDHU);
    /// ```
    pub const DHARMASINDHU: &'static str = "DHARMASINDHU";

    /// *Dharmasindhu* pp. 299–303 and 309: persons counted from the
    /// kūṭastha, seven sapiṇḍas, seven samānodakas to the 14th, sagotras
    /// to the 21st, and the named kin.
    ///
    /// ```
    /// use teistro_panchanga::ashauch::KinTable;
    ///
    /// assert!(KinTable::dharmasindhu().living_extend);
    /// ```
    #[must_use]
    pub fn dharmasindhu() -> KinTable {
        KinTable {
            name: KinTable::DHARMASINDHU.to_owned(),
            sapinda_to: 7,
            samanodaka_to: 14,
            sagotra_to: 21,
            living_extend: true,
            girl_to: 3,
            rows: rows(),
            line: String::from(
                "Dharmasindhu p. 299: from the kūṭastha seven sapiṇḍas, seven samānodakas, sagotras to the 21st, the observer's own number deciding; p. 300 with Nirṇayasindhu p. 257: a person more for each forebear living",
            ),
            beyond: String::from(
                "Dharmasindhu p. 299: past the sagotras, a bath at a death while one clan is known",
            ),
            unbound: String::from(
                "Dharmasindhu p. 301: a man before his upanayana and a girl unmarried keep a parent's period and no other",
            ),
            natal: String::from(
                "Dharmasindhu p. 301: a married daughter, for her grandfather or paternal uncle, a bath",
            ),
            unnamed: String::from(
                "Dharmasindhu p. 309: kin of slight connection the texts give no period, a bath when near",
            ),
        }
    }

    /// Whether the table can be read: its bounds rise.
    ///
    /// ```
    /// use teistro_panchanga::ashauch::KinTable;
    ///
    /// let table = KinTable { sagotra_to: 5, ..KinTable::dharmasindhu() };
    /// assert!(table.check().is_err());
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` when a bound does not pass the one before it, or a
    /// row has no path or keeps a period with no relation.
    pub fn check(&self) -> Result<(), Error> {
        if self.sapinda_to == 0
            || self.samanodaka_to < self.sapinda_to
            || self.sagotra_to < self.samanodaka_to
        {
            return Err(Error::invalid_arg(
                "the sapiṇḍa, samānodaka and sagotra bounds rise from 1",
            )
            .with_field("sagotraTo"));
        }
        for (at, row) in self.rows.iter().enumerate() {
            if row.path.is_empty() || (row.keeps == KinKeeps::Period && row.relation.is_none()) {
                return Err(Error::invalid_arg(
                    "a row names a path, and a relation when it keeps a period",
                )
                .with_field(format!("rows[{at}]")));
            }
        }
        Ok(())
    }
}

/// A row keeping a relation's period.
fn period(path: &[KinStep], relation: Relation, clause: &str) -> KinRow {
    KinRow {
        path: path.to_vec(),
        observer: None,
        observer_married: None,
        person_married: None,
        relation: Some(relation),
        keeps: KinKeeps::Period,
        clause: clause.to_owned(),
    }
}

impl KinRow {
    fn of(self, sex: Sex) -> KinRow {
        KinRow {
            observer: Some(sex),
            ..self
        }
    }

    fn observer_married(self, married: bool) -> KinRow {
        KinRow {
            observer_married: Some(married),
            ..self
        }
    }

    fn person_married(self, married: bool) -> KinRow {
        KinRow {
            person_married: Some(married),
            ..self
        }
    }

    fn keeps(self, keeps: KinKeeps) -> KinRow {
        KinRow {
            relation: None,
            keeps,
            ..self
        }
    }
}

const BANDHU: &str = "Dharmasindhu p. 302: the nine bandhus, the sons of a father's sister, a mother's sister and a mother's brother, of oneself, one's father and one's mother";

/// The three cousins of a line, each from `from`.
fn bandhus(from: &[KinStep]) -> Vec<KinRow> {
    use KinStep::{Daughter, Father, Son};
    let cousins: [&[KinStep]; 3] = [
        &[Father, Father, Daughter, Son],
        &[KinStep::Mother, Father, Daughter, Son],
        &[KinStep::Mother, Father, Son, Son],
    ];
    cousins
        .iter()
        .map(|cousin| {
            let path = [from, cousin].concat();
            period(&through_father(&path), Relation::Bandhu, BANDHU)
        })
        .collect()
}

#[expect(
    clippy::too_many_lines,
    reason = "the Dharmasindhu's kin rows, one clause each, kept as one table"
)]
fn rows() -> Vec<KinRow> {
    use KinStep::{Daughter, Father, Husband, Mother, Son, Wife};
    use Relation as R;
    use Sex::{Female, Male};
    let mut rows = vec![
        period(
            &[Father],
            R::Child,
            "Dharmasindhu p. 301: a parent's death, the full period",
        )
        .of(Male),
        period(
            &[Mother],
            R::Child,
            "Dharmasindhu p. 301: a parent's death, the full period",
        )
        .of(Male),
        period(
            &[Father],
            R::Child,
            "Dharmasindhu p. 301: an unmarried daughter keeps her parents' period",
        )
        .of(Female)
        .observer_married(false),
        period(
            &[Mother],
            R::Child,
            "Dharmasindhu p. 301: an unmarried daughter keeps her parents' period",
        )
        .of(Female)
        .observer_married(false),
        period(
            &[Father],
            R::MarriedDaughter,
            "Dharmasindhu p. 301: a parent's death, a married daughter",
        )
        .of(Female),
        period(
            &[Mother],
            R::MarriedDaughter,
            "Dharmasindhu p. 301: a parent's death, a married daughter",
        )
        .of(Female),
        period(
            &[Son],
            R::Father,
            "Dharmasindhu p. 300: a child's death, the father",
        )
        .of(Male),
        period(
            &[Daughter],
            R::Father,
            "Dharmasindhu p. 300: a child's death, the father",
        )
        .of(Male),
        period(
            &[Son],
            R::Mother,
            "Dharmasindhu p. 300: a child's death, the mother",
        )
        .of(Female),
        period(
            &[Daughter],
            R::Mother,
            "Dharmasindhu p. 300: a child's death, the mother",
        )
        .of(Female),
        period(
            &[Husband],
            R::Spouse,
            "Dharmasindhu p. 304: husband and wife",
        ),
        period(&[Wife], R::Spouse, "Dharmasindhu p. 304: husband and wife"),
        period(&[Husband, Wife], R::CoWife, "Dharmasindhu p. 304: co-wives"),
        period(
            &[Father, Wife],
            R::StepChild,
            "Dharmasindhu p. 304: a step-mother",
        ),
        period(
            &[Father, Daughter],
            R::MarriedSibling,
            "Dharmasindhu p. 301: a brother and a married sister, to each other",
        )
        .person_married(true),
        period(
            &[Father, Son],
            R::MarriedSibling,
            "Dharmasindhu p. 301: a brother and a married sister, to each other",
        )
        .of(Female)
        .observer_married(true),
        period(
            &[Mother, Father],
            R::MaternalGrandfather,
            "Dharmasindhu p. 301: the mother's father",
        ),
        period(
            &[Mother, Mother],
            R::MaternalGrandmother,
            "Dharmasindhu p. 301: the mother's mother",
        ),
        period(
            &[Father, Father],
            R::Sapinda,
            "Dharmasindhu p. 301: a married daughter, for her grandfather, a bath",
        )
        .of(Female)
        .observer_married(true)
        .keeps(KinKeeps::Bath),
        period(
            &[Father, Father, Son],
            R::Sapinda,
            "Dharmasindhu p. 301: a married daughter, for her paternal uncle, a bath",
        )
        .of(Female)
        .observer_married(true)
        .keeps(KinKeeps::Bath),
        period(
            &[Father, Father, Daughter],
            R::PaternalAunt,
            "Dharmasindhu p. 302: a father's sister",
        )
        .person_married(true),
        period(
            &[Mother, Father, Son],
            R::MaternalUncle,
            "Dharmasindhu p. 301: the mother's brother",
        ),
        period(
            &[Mother, Father, Daughter],
            R::MaternalAunt,
            "Dharmasindhu p. 302: the mother's sister",
        ),
        period(
            &[Mother, Father, Son, Wife],
            R::MaternalUnclesWife,
            "Dharmasindhu p. 301: the maternal uncle's wife",
        ),
        period(
            &[Father, Daughter, Son],
            R::SistersSon,
            "Dharmasindhu p. 301: a sister's son",
        ),
        period(
            &[Father, Daughter, Daughter],
            R::SistersSon,
            "Dharmasindhu p. 301: a sister's daughter, a bath, as it seems",
        )
        .keeps(KinKeeps::Bath),
        period(
            &[Daughter, Son],
            R::DaughtersSon,
            "Dharmasindhu pp. 301–302: a daughter's son",
        ),
        period(
            &[Daughter, Daughter],
            R::DaughtersSon,
            "Dharmasindhu p. 302: a daughter's daughter, none, as it seems",
        )
        .keeps(KinKeeps::Nothing),
        period(
            &[Wife, Father],
            R::SonInLaw,
            "Dharmasindhu p. 302: a father-in-law",
        ),
        period(
            &[Wife, Mother],
            R::SonInLaw,
            "Dharmasindhu p. 302: a mother-in-law",
        ),
        period(
            &[Wife, Father, Son],
            R::WifesBrother,
            "Dharmasindhu p. 302: a wife's brother",
        ),
        period(
            &[Daughter, Husband],
            R::ParentInLaw,
            "Dharmasindhu p. 302: a son-in-law",
        ),
    ];
    rows.extend(bandhus(&[]));
    rows.extend(bandhus(&[Father]));
    rows.extend(bandhus(&[Mother]));
    rows
}
