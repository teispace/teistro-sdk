//! The catalogue: one enum per kind with stable ids, cited attributes and
//! confidence marks, generated from `catalogue/*.yaml` by
//! `cargo xtask gen catalogue` and held to its sources by
//! `cargo xtask check-catalogue`. The hand-written part is the vocabulary
//! the generated code shares: the trait every kind implements, marks,
//! sources, and the error for a key nobody has.
//!
//! ```
//! use teistro_core::catalogue::{Catalogued, Nakshatra};
//!
//! let rohini: Nakshatra = "ROHINI".parse().expect("catalogued");
//! assert_eq!(rohini.id(), 3);
//! assert_eq!(rohini.full_key(), "nakshatra.ROHINI");
//! assert_eq!(rohini.attributes().vimshottari_lord.key(), "MOON");
//! assert_eq!(Nakshatra::ALL.len(), 27);
//! ```

use core::fmt;

use crate::key::KeyId;

#[rustfmt::skip]
mod generated;

pub use generated::*;

/// How well a catalogue row is sourced (ADR-0018).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Mark {
    /// Verified against a text or the baseline engine's validated data.
    Verified,
    /// Traditional: in use, awaiting a citation.
    Traditional,
    /// Shape only: the identifier exists, the content does not.
    Shape,
}

impl Mark {
    /// The one-letter form used in the sources: V, T or S.
    #[must_use]
    pub const fn letter(self) -> char {
        match self {
            Mark::Verified => 'V',
            Mark::Traditional => 'T',
            Mark::Shape => 'S',
        }
    }
}

impl fmt::Display for Mark {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Mark::Verified => "verified",
            Mark::Traditional => "traditional",
            Mark::Shape => "shape",
        })
    }
}

/// A citation: a text and the place in it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Source {
    /// The text: `BPHS`, `baseline-engine`, `Muhurta Chintamani`.
    pub text: &'static str,
    /// Where in it: a chapter and verse, a file, a page.
    pub reference: &'static str,
}

impl Source {
    /// A citation.
    #[must_use]
    pub const fn new(text: &'static str, reference: &'static str) -> Source {
        Source { text, reference }
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.text, self.reference)
    }
}

/// What every catalogued kind offers, for generic code; the generated
/// enums add typed attributes on top.
pub trait Catalogued:
    Copy + Eq + Ord + core::hash::Hash + fmt::Debug + fmt::Display + core::str::FromStr + 'static
{
    /// The kind.
    const KIND: Kind;

    /// Every member, in id order.
    fn all() -> &'static [Self];

    /// The key inside the kind (`SUN`).
    fn key(self) -> &'static str;

    /// The catalogue id.
    fn id(self) -> u16;

    /// The confidence mark.
    fn mark(self) -> Mark;

    /// The member with a key or a former key.
    fn from_key(key: &str) -> Option<Self>;

    /// The member with an id.
    fn from_id(id: u16) -> Option<Self>;

    /// The full key (`graha.SUN`).
    fn full_key(self) -> String {
        format!("{}.{}", Self::KIND.name(), self.key())
    }

    /// The packed id for the C boundary.
    fn key_id(self) -> KeyId {
        KeyId::new(Self::KIND, self.id())
    }

    /// The nearest key to `key` among the members, within two edits, for
    /// the message a wrong key gets.
    #[must_use]
    fn suggest(key: &str) -> Option<&'static str> {
        let upper = key.to_ascii_uppercase();
        Self::all()
            .iter()
            .map(|m| (distance(&upper, m.key()), m.key()))
            .filter(|(d, _)| *d <= 2)
            .min_by_key(|(d, _)| *d)
            .map(|(_, k)| k)
    }
}

/// A key or id nobody has.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnknownKey {
    /// The kind searched, when known.
    pub kind: Option<Kind>,
    /// What was asked for.
    pub key: String,
    /// The nearest known key, when one is within two edits.
    pub suggestion: Option<&'static str>,
}

impl UnknownKey {
    /// A key that is not a member of `T`.
    #[must_use]
    pub fn in_kind<T: Catalogued>(key: &str) -> UnknownKey {
        UnknownKey {
            kind: Some(T::KIND),
            key: key.to_string(),
            suggestion: T::suggest(key),
        }
    }

    /// An id that resolves to nothing.
    #[must_use]
    pub fn id(id: KeyId) -> UnknownKey {
        UnknownKey {
            kind: id.kind(),
            key: format!("{}#{}", id.kind_number(), id.id()),
            suggestion: None,
        }
    }

    /// A kind name nobody has.
    #[must_use]
    pub fn kind_name(name: &str) -> UnknownKey {
        let lower = name.to_ascii_lowercase();
        UnknownKey {
            kind: None,
            key: name.to_string(),
            suggestion: Kind::ALL
                .iter()
                .map(|k| (distance(&lower, k.name()), k.name()))
                .filter(|(d, _)| *d <= 2)
                .min_by_key(|(d, _)| *d)
                .map(|(_, k)| k),
        }
    }
}

impl fmt::Display for UnknownKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            Some(kind) => write!(f, "unknown {kind} key `{}`", self.key)?,
            None => write!(f, "unknown key `{}`", self.key)?,
        }
        if let Some(suggestion) = self.suggestion {
            write!(f, "; did you mean `{suggestion}`?")?;
        }
        Ok(())
    }
}

impl std::error::Error for UnknownKey {}

/// Binary search over a table sorted by key.
pub(crate) fn lookup<T: Copy>(table: &[(&str, T)], key: &str) -> Option<T> {
    table
        .binary_search_by(|(k, _)| (*k).cmp(key))
        .ok()
        .and_then(|i| table.get(i))
        .map(|(_, v)| *v)
}

/// The Levenshtein distance, for suggestions; both inputs are short, and
/// a length difference beyond the suggestion threshold is answered at
/// once.
pub(crate) fn distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.len().abs_diff(b.len()) > 2 {
        return a.len().abs_diff(b.len());
    }
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut current = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let substitute = previous.get(j).copied().unwrap_or(0) + usize::from(ca != cb);
            let insert = current.get(j).copied().unwrap_or(0) + 1;
            let delete = previous.get(j + 1).copied().unwrap_or(0) + 1;
            current.push(substitute.min(insert).min(delete));
        }
        previous = current;
    }
    previous.last().copied().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;

    #[test]
    fn every_kind_has_dense_ids_and_unique_keys() {
        fn check<T: Catalogued>() {
            for (index, member) in T::all().iter().enumerate() {
                assert_eq!(usize::from(member.id()), index, "{}", member.full_key());
                assert_eq!(T::from_id(member.id()), Some(*member));
                assert_eq!(T::from_key(member.key()), Some(*member));
                assert_eq!(member.key_id().kind(), Some(T::KIND));
                assert_eq!(key_of(member.key_id()), Some(member.key()));
                assert!(crate::key::is_key_name(member.key()), "{}", member.key());
            }
            assert_eq!(T::KIND.count(), T::all().len());
        }
        check::<Graha>();
        check::<Rashi>();
        check::<Nakshatra>();
        check::<Tithi>();
        check::<Karana>();
        check::<Yoga>();
        check::<Vara>();
        check::<Masa>();
        check::<Ayanamsha>();
        check::<HouseSystem>();
        check::<Varga>();
        check::<DashaSystem>();
        check::<Point>();
        check::<Deity>();
    }

    #[test]
    fn kinds_are_numbered_once_and_named() {
        let mut numbers: Vec<u8> = Kind::ALL.iter().map(|k| k.number()).collect();
        numbers.sort_unstable();
        numbers.dedup();
        assert_eq!(numbers.len(), Kind::ALL.len());
        for kind in Kind::ALL {
            assert_eq!(Kind::from_number(kind.number()), Some(kind));
            assert_eq!(Kind::from_name(kind.name()), Some(kind));
        }
        assert!(Kind::Rule.is_open());
        assert!(!Kind::Graha.is_open());
    }

    #[test]
    fn the_classical_facts_hold_together() {
        for graha in Graha::ALL {
            let a = graha.attributes();
            if let (Some(ex), Some(de)) = (a.exaltation, a.debilitation) {
                assert_eq!((ex.sign.id() + 6) % 12, de.sign.id(), "{graha}");
            }
            for own in a.own {
                assert!(
                    own.attributes().lord == graha || own.attributes().co_lord == Some(graha),
                    "{graha} owns {own} whose lord is {}",
                    own.attributes().lord
                );
            }
            if graha.attributes().body != BodyClass::Outer {
                assert!(a.descriptors.is_some(), "{graha} has descriptors");
            }
        }
        for rashi in Rashi::ALL {
            assert!(
                rashi.attributes().lord.attributes().own.contains(&rashi),
                "{rashi}"
            );
        }
        for nakshatra in Nakshatra::ALL {
            assert_eq!(nakshatra.attributes().padas.len(), 4);
        }
        assert_eq!(
            Vara::ALL
                .iter()
                .filter(|v| v.attributes().lord == Graha::Sun)
                .count(),
            1
        );
        for tithi in Tithi::ALL {
            let a = tithi.attributes();
            assert_eq!(
                a.paksha,
                if tithi.id() < 15 {
                    Paksha::Shukla
                } else {
                    Paksha::Krishna
                }
            );
            assert_eq!(u16::from(a.number), tithi.id() % 15 + 1);
        }
    }

    #[test]
    fn unknown_keys_report_kind_and_suggestion() {
        let error = Graha::from_str_error("SUNN");
        assert_eq!(error.kind, Some(Kind::Graha));
        assert_eq!(error.suggestion, Some("SUN"));
        assert_eq!(
            error.to_string(),
            "unknown graha key `SUNN`; did you mean `SUN`?"
        );
        assert_eq!(distance("kitten", "sitting"), 3);
        assert_eq!(UnknownKey::kind_name("grahas").suggestion, Some("graha"));
    }

    impl Graha {
        fn from_str_error(key: &str) -> UnknownKey {
            match key.parse::<Graha>() {
                Ok(_) => panic!("{key} parsed"),
                Err(e) => e,
            }
        }
    }

    #[test]
    fn aliases_resolve_and_marks_are_read() {
        assert_eq!(Yoga::from_key("VISHKAMBA"), Some(Yoga::Vishkambha));
        assert_eq!(Graha::Sun.mark(), Mark::Verified);
        assert_eq!(Graha::Rahu.mark(), Mark::Traditional);
        assert!(Graha::Sun.sources().iter().any(|s| s.text == "BPHS"));
        assert_eq!(resolve(Kind::Rashi, "LEO"), Some(Rashi::Leo.key_id()));
        assert_eq!(Rashi::try_from(Rashi::Leo.key_id()), Ok(Rashi::Leo));
        assert!(Rashi::try_from(Graha::Sun.key_id()).is_err());
    }
}
