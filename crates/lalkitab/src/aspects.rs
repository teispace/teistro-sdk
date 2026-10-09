//! Aspects (1952 pp. 102–113).
//!
//! An aspect runs forward only, from the earlier house to the later, and
//! the house it falls on does not look back (p. 102). The printed arrow
//! table gives which houses look and how hard (crux LK-C7 takes it over
//! the verse's looser pairing); the *yog drishti* relations of p. 112 are
//! a rule over every house, checked against all twelve printed rows.

use serde::Serialize;

/// How strongly one house looks at another.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Strength {
    /// A quarter: 2 to 6, 6 to 12, 8 back to 2.
    Quarter,
    /// A half: 3 to 9 and 11, 5 to 9.
    Half,
    /// The whole: 1 to 7, 4 to 10.
    Full,
}

impl Strength {
    /// The share of the looking house's effect that reaches, in percent.
    #[must_use]
    pub const fn percent(self) -> u8 {
        match self {
            Strength::Quarter => 25,
            Strength::Half => 50,
            Strength::Full => 100,
        }
    }
}

/// The houses a house looks at, with how hard (1952 pp. 102–104): the
/// printed arrow table, start houses 1, 2, 3, 4, 5, 6 and 8.
#[must_use]
pub const fn looks_at(house: u8) -> &'static [(u8, Strength)] {
    match house {
        1 => &[(7, Strength::Full)],
        2 => &[(6, Strength::Quarter)],
        3 => &[(9, Strength::Half), (11, Strength::Half)],
        4 => &[(10, Strength::Full)],
        5 => &[(9, Strength::Half)],
        6 => &[(12, Strength::Quarter)],
        8 => &[(2, Strength::Quarter)],
        _ => &[],
    }
}

/// The houses that look at `house`, with how hard.
pub fn looked_at_by(house: u8) -> impl Iterator<Item = (u8, Strength)> {
    (1..=12_u8).flat_map(move |from| {
        looks_at(from)
            .iter()
            .filter(move |(to, _)| *to == house)
            .map(move |(_, strength)| (from, *strength))
    })
}

/// One *yog drishti* relation of a house (1952 p. 112): the house it sees
/// under the relation, and the house it is seen from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Relation {
    /// The house this one can see.
    pub sees: u8,
    /// The house this one is seen from.
    pub seen_from: u8,
}

/// Every *yog drishti* relation of one house (1952 p. 112).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct YogDrishti {
    /// Mutual help (*bahami madad*).
    pub help: Relation,
    /// The general condition (*aam halat*).
    pub general: Relation,
    /// Collision (*takrao*).
    pub collision: Relation,
    /// Foundation (*buniyadi*).
    pub foundation: Relation,
    /// Deceit (*dhokha*).
    pub deceit: Relation,
    /// A joint wall (*mushtarka diwar*).
    pub wall: Relation,
}

/// The house `steps` on from `house`, counting round the twelve.
#[must_use]
pub const fn ahead(house: u8, steps: u8) -> u8 {
    (house + steps + 11) % 12 + 1
}

/// A house's *yog drishti* relations: the rule p. 112 prints house by
/// house, the house `a` on as seen and the house `b` on as seeing.
///
/// ```
/// use teistro_lalkitab::aspects::yog_drishti;
///
/// let fourth = yog_drishti(4);
/// assert_eq!((fourth.help.sees, fourth.help.seen_from), (8, 12));
/// assert_eq!((fourth.wall.sees, fourth.wall.seen_from), (5, 3));
/// ```
#[must_use]
pub fn yog_drishti(house: u8) -> YogDrishti {
    let relation = |sees: u8, seen_from: u8| Relation {
        sees: ahead(house, sees),
        seen_from: ahead(house, seen_from),
    };
    YogDrishti {
        help: relation(4, 8),
        general: relation(6, 6),
        collision: relation(7, 5),
        foundation: relation(8, 4),
        deceit: relation(9, 3),
        wall: relation(1, 11),
    }
}
