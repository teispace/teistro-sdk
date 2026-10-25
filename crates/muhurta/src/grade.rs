//! Grades: how a rule rates a member — a tithi, a vara, a lagna.
//!
//! The texts grade rather than only reject. Raman's marriage chapter
//! calls Monday, Wednesday, Thursday and Friday the best, Sunday and
//! Saturday middling and Tuesday to be rejected; its stars are a list of
//! the good with "not mentioned here are unsuitable". So a rule is three
//! lists and what an unlisted member is, and a clause is reported for the
//! best (for the time) and the rejected (against it). A middling member
//! is reported by nothing: it neither helps nor harms, and a third kind
//! of clause would make every "not favourable" read as "against" wrong.

use serde::{Deserialize, Serialize};

/// A member's grade under a rule.
///
/// ```
/// use teistro_core::catalogue::Vara;
/// use teistro_muhurta::{Grade, Graded};
///
/// // Raman's marriage chapter calls Saturday middling; his general shuddhi
/// // rejects Tuesday and Saturday.
/// let marriage = Graded {
///     best: vec![Vara::Somavara, Vara::Budhavara, Vara::Guruvara, Vara::Shukravara],
///     middling: vec![Vara::Ravivara, Vara::Shanivara],
///     rejected: vec![Vara::Mangalavara],
///     otherwise: Grade::Middling,
/// };
/// assert_eq!(marriage.grade(&Vara::Guruvara), Grade::Best);
/// assert_eq!(marriage.grade(&Vara::Mangalavara), Grade::Rejected);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Grade {
    /// Named good: a clause for the time.
    Best,
    /// Named ordinary, or unlisted where the rule leaves it so: no clause.
    Middling,
    /// Named bad, or unlisted where the rule says the rest are: a clause
    /// against the time.
    Rejected,
}

/// Three lists and what the rest are.
///
/// A member in more than one list takes the **worst** of its grades, so a
/// careless merge can only make a rule stricter.
///
/// ```
/// use teistro_core::catalogue::Vara;
/// use teistro_muhurta::{Grade, Graded};
///
/// // Raman's marriage chapter calls Saturday middling; his general shuddhi
/// // rejects Tuesday and Saturday.
/// let marriage = Graded {
///     best: vec![Vara::Somavara, Vara::Budhavara, Vara::Guruvara, Vara::Shukravara],
///     middling: vec![Vara::Ravivara, Vara::Shanivara],
///     rejected: vec![Vara::Mangalavara],
///     otherwise: Grade::Middling,
/// };
/// // A member in two lists takes the worse grade.
/// let mut strict = marriage.clone();
/// strict.rejected.push(Vara::Shukravara);
/// assert_eq!(strict.grade(&Vara::Shukravara), Grade::Rejected);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Graded<T> {
    /// The members named good.
    pub best: Vec<T>,
    /// The members named ordinary.
    pub middling: Vec<T>,
    /// The members named bad.
    pub rejected: Vec<T>,
    /// What a member in none of the lists is.
    pub otherwise: Grade,
}

impl<T: PartialEq + Clone> Graded<T> {
    /// A rule that only rejects: the rest are middling.
    ///
    /// ```
    /// use teistro_core::catalogue::Vara;
    /// use teistro_muhurta::{Grade, Graded};
    ///
    /// let general = Graded::rejecting(vec![Vara::Mangalavara, Vara::Shanivara]);
    /// assert_eq!(general.grade(&Vara::Shanivara), Grade::Rejected);
    /// assert_eq!(general.grade(&Vara::Ravivara), Grade::Middling);
    /// ```
    #[must_use]
    pub fn rejecting(rejected: Vec<T>) -> Graded<T> {
        Graded {
            best: Vec::new(),
            middling: Vec::new(),
            rejected,
            otherwise: Grade::Middling,
        }
    }

    /// A rule that grades nothing: every member middling.
    ///
    /// ```
    /// use teistro_core::catalogue::Vara;
    /// use teistro_muhurta::{Grade, Graded};
    ///
    /// let open: Graded<Vara> = Graded::none();
    /// assert!(Vara::ALL.iter().all(|v| open.grade(v) == Grade::Middling));
    /// ```
    #[must_use]
    pub const fn none() -> Graded<T> {
        Graded {
            best: Vec::new(),
            middling: Vec::new(),
            rejected: Vec::new(),
            otherwise: Grade::Middling,
        }
    }

    /// A rule that only admits: the members named middling, the rest
    /// rejected — a gate, which weighs nothing either way.
    ///
    /// ```
    /// use teistro_core::catalogue::Vara;
    /// use teistro_muhurta::{Grade, Graded};
    ///
    /// // A gate: Thursday and Friday pass, weighing nothing; the rest are shut.
    /// let gate = Graded::admitting(vec![Vara::Guruvara, Vara::Shukravara]);
    /// assert_eq!(gate.grade(&Vara::Guruvara), Grade::Middling);
    /// assert_eq!(gate.grade(&Vara::Somavara), Grade::Rejected);
    /// ```
    #[must_use]
    pub const fn admitting(admitted: Vec<T>) -> Graded<T> {
        Graded {
            best: Vec::new(),
            middling: admitted,
            rejected: Vec::new(),
            otherwise: Grade::Rejected,
        }
    }

    /// The grade of a member.
    ///
    /// ```
    /// use teistro_core::catalogue::Vara;
    /// use teistro_muhurta::{Grade, Graded};
    ///
    /// // Raman's marriage chapter calls Saturday middling; his general shuddhi
    /// // rejects Tuesday and Saturday.
    /// let marriage = Graded {
    ///     best: vec![Vara::Somavara, Vara::Budhavara, Vara::Guruvara, Vara::Shukravara],
    ///     middling: vec![Vara::Ravivara, Vara::Shanivara],
    ///     rejected: vec![Vara::Mangalavara],
    ///     otherwise: Grade::Middling,
    /// };
    /// assert_eq!(marriage.grade(&Vara::Shanivara), Grade::Middling);
    /// ```
    #[must_use]
    pub fn grade(&self, member: &T) -> Grade {
        if self.rejected.contains(member) {
            Grade::Rejected
        } else if self.middling.contains(member) {
            Grade::Middling
        } else if self.best.contains(member) {
            Grade::Best
        } else {
            self.otherwise
        }
    }

    /// Whether the rule names a member in any of its lists.
    ///
    /// ```
    /// use teistro_core::catalogue::Vara;
    /// use teistro_muhurta::{Grade, Graded};
    ///
    /// let general = Graded::rejecting(vec![Vara::Mangalavara]);
    /// assert!(general.names(&Vara::Mangalavara));
    /// assert!(!general.names(&Vara::Ravivara));
    /// ```
    #[must_use]
    pub fn names(&self, member: &T) -> bool {
        self.best.contains(member)
            || self.middling.contains(member)
            || self.rejected.contains(member)
    }

    /// This rule laid over a general one: a member this rule names takes
    /// this rule's grade, and one it does not takes the general rule's
    /// **named** grade before this rule's `otherwise`.
    ///
    /// The specific chapter over the general shuddhi, as the texts write
    /// it: Raman's marriage chapter calls Saturday middling though his
    /// general shuddhi rejects it, and leaves Vaidhriti to the general
    /// list, which rejects it.
    ///
    /// ```
    /// use teistro_core::catalogue::Vara;
    /// use teistro_muhurta::{Grade, Graded};
    ///
    /// // Raman's marriage chapter calls Saturday middling; his general shuddhi
    /// // rejects Tuesday and Saturday.
    /// let marriage = Graded {
    ///     best: vec![Vara::Somavara, Vara::Budhavara, Vara::Guruvara, Vara::Shukravara],
    ///     middling: vec![Vara::Ravivara, Vara::Shanivara],
    ///     rejected: vec![Vara::Mangalavara],
    ///     otherwise: Grade::Middling,
    /// };
    /// let general = Graded::rejecting(vec![Vara::Mangalavara, Vara::Shanivara]);
    /// let merged = marriage.over(&general);
    /// // The chapter speaks of Saturday, so its middling stands.
    /// assert_eq!(merged.grade(&Vara::Shanivara), Grade::Middling);
    /// assert_eq!(merged.grade(&Vara::Mangalavara), Grade::Rejected);
    /// ```
    #[must_use]
    pub fn over(mut self, general: &Graded<T>) -> Graded<T> {
        for (list, grade) in [
            (&general.best, Grade::Best),
            (&general.middling, Grade::Middling),
            (&general.rejected, Grade::Rejected),
        ] {
            for member in list {
                if !self.names(member) {
                    match grade {
                        Grade::Best => self.best.push(member.clone()),
                        Grade::Middling => self.middling.push(member.clone()),
                        Grade::Rejected => self.rejected.push(member.clone()),
                    }
                }
            }
        }
        self
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{Grade, Graded};

    fn marriage() -> Graded<u8> {
        Graded {
            best: vec![1, 3, 4, 5],
            middling: vec![0, 6],
            rejected: vec![2],
            otherwise: Grade::Middling,
        }
    }

    #[test]
    fn a_member_in_two_lists_takes_the_worse_grade() {
        let mut rule = marriage();
        rule.rejected.push(4);
        assert_eq!(rule.grade(&4), Grade::Rejected);
        rule.middling.push(5);
        assert_eq!(rule.grade(&5), Grade::Middling);
        assert_eq!(rule.grade(&9), Grade::Middling);
    }

    #[test]
    fn the_specific_rule_wins_where_it_speaks_and_the_general_where_it_does_not() {
        // General: reject 2 and 6 (Tuesday, Saturday) and 9.
        let general = Graded::rejecting(vec![2, 6, 9]);
        let merged = marriage().over(&general);
        // Saturday: the chapter says middling, and it wins.
        assert_eq!(merged.grade(&6), Grade::Middling);
        // 9: the chapter is silent, so the general rejection holds.
        assert_eq!(merged.grade(&9), Grade::Rejected);
        assert_eq!(merged.grade(&1), Grade::Best);
        assert_eq!(merged.grade(&2), Grade::Rejected);
        // What neither names takes the chapter's `otherwise`.
        assert_eq!(merged.grade(&7), Grade::Middling);
    }
}
