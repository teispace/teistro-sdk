//! What a condition can be about: a body, or a sign reached from one
//! (`03-design/rules-engine.md`, "Subjects are references").
//!
//! The texts' rules name more than the nine grahas and the lagna. BPHS names
//! the lord of a house (ch. 34), the dispositor of a karaka and the karaka
//! itself (ch. 40), the pada of a house and the houses counted from it
//! (ch. 29), the upapada and the lord of the seventh from it (ch. 30), and the
//! Karakamsha (ch. 33). Each is one of two kinds, and the kinds are types:
//!
//! - a [`BodyRef`] resolves to a body, which has a longitude, a dignity and a
//!   motion: a graha or the lagna named, the lord of a sign, or the holder of
//!   a chara karaka;
//! - a [`SignRef`] resolves to a sign, which has a house and occupants and
//!   nothing else: any body's sign, a house from the lagna, a pada, a body's
//!   navamsha, the upapada, or a house counted from any of these.
//!
//! So a predicate that needs a body — a dignity, a conjunction by degrees, who
//! aspects — takes a `BodyRef`, and a rule asking for the dignity of a pada is
//! refused when it is read, with the reason, rather than answering false.
//!
//! In a rule a reference is written the way the recording engine writes a
//! body, and extends it:
//!
//! | written | reference |
//! |---|---|
//! | `"JUPITER"`, `"LAGNA"` | that body |
//! | `{"lordOf": 10}` | the lord of the tenth house's sign |
//! | `{"karaka": "AmK"}`, `{"karaka": "AK", "scheme": 8}` | the graha holding it, seven karakas unless the scheme says eight |
//! | `7` | the seventh house's sign, whole signs from the lagna |
//! | `{"arudha": 1}` | the pada of the sign named, here the arudha lagna |
//! | `"UPAPADA"` | the upapada, under [`Upapada`](crate::Upapada) |
//! | `{"navamsha": {"karaka": "AK"}}` | a body's navamsha sign, here the Karakamsha |
//! | `{"badhakaOf": 1}` | the badhaka sthana of a sign, here the lagna's |
//! | `{"exaltationOf": "MOON"}`, `{"debilitationOf": "SELF"}` | where a body is exalted or debilitated |
//! | `{"exaltedIn": {"debilitationOf": "SELF"}}` | the body exalted in a sign |
//! | `"SELF"` | the body a `for-any` bound |
//! | `{"from": {"arudha": 1}, "house": 11}` | the eleventh sign from another |
//!
//! ```
//! use teistro_rules::{BodyRef, House, SignRef};
//!
//! // "the Lord of the 7th, counted from Upa Pad" (BPHS ch. 30 vv. 42 to 43).
//! let written: BodyRef = serde_json::from_str(r#"{"lordOf": {"from": "UPAPADA", "house": 7}}"#)?;
//! let built = BodyRef::lord_of(SignRef::counted(SignRef::Upapada, House::try_new(7)?));
//! assert_eq!(written, built);
//! assert_eq!(serde_json::to_string(&built)?, r#"{"lordOf":{"from":"UPAPADA","house":7}}"#);
//!
//! // A pada has a sign and no dignity, so a body cannot be one.
//! let refused = serde_json::from_str::<BodyRef>(r#"{"arudha": 1}"#).unwrap_err();
//! assert!(refused.to_string().contains("names a sign"));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::ser::{SerializeMap, Serializer};
use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{CharaKaraka, Graha};

use crate::language::{Body, House, Karaka, KarakaScheme, SELF};

/// A reference that resolves to a body.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BodyRef {
    /// That graha, or the lagna.
    Body(Body),
    /// The lord of a sign, the catalogue's (Mars for Scorpio, Saturn for
    /// Aquarius).
    LordOf(Box<SignRef>),
    /// The body a `for-any` bound, written `SELF`; none outside one.
    Bound,
    /// The body exalted in a sign, when one is.
    ExaltedIn(Box<SignRef>),
    /// The graha holding a chara karaka, when one holds it.
    Karaka {
        /// Which karaka.
        karaka: Karaka,
        /// Among seven grahas or eight.
        scheme: KarakaScheme,
    },
}

/// A reference that resolves to a sign.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SignRef {
    /// The sign a body stands in.
    Of(BodyRef),
    /// A house's sign, whole signs from the lagna.
    House(House),
    /// The arudha pada of the house standing in a sign (BPHS ch. 29 vv. 1 to
    /// 5), the catalogue's lords counted.
    Arudha(Box<SignRef>),
    /// The upapada, whichever house [`Upapada`](crate::Upapada) reads (BPHS
    /// ch. 30 vv. 1 to 6).
    Upapada,
    /// A body's navamsha sign.
    Navamsha(BodyRef),
    /// The sign a body is exalted in, when it has one.
    Exaltation(BodyRef),
    /// The sign a body is debilitated in, when it has one.
    Debilitation(BodyRef),
    /// The badhaka sthana of a sign: the eleventh from a movable sign (BPHS
    /// ch. 50 vv. 20 to 21), and by later tradition the ninth from a fixed one
    /// and the seventh from a dual one (crux C86).
    Badhaka(Box<SignRef>),
    /// The sign so many houses on from another, whole signs.
    Counted {
        /// From where.
        from: Box<SignRef>,
        /// Which house from it, the first being itself.
        house: House,
    },
}

impl BodyRef {
    /// The lord of a sign.
    #[must_use]
    pub fn lord_of(sign: impl Into<SignRef>) -> BodyRef {
        BodyRef::LordOf(Box::new(sign.into()))
    }

    /// The holder of a karaka among seven grahas.
    #[must_use]
    pub const fn karaka(karaka: CharaKaraka) -> BodyRef {
        BodyRef::Karaka {
            karaka: Karaka(karaka),
            scheme: KarakaScheme::Seven,
        }
    }

    /// The body, when the reference names one outright.
    #[must_use]
    pub const fn named(&self) -> Option<Body> {
        match self {
            BodyRef::Body(body) => Some(*body),
            _ => None,
        }
    }
}

impl SignRef {
    /// The pada of the house standing in a sign: `SignRef::arudha(1)` is the
    /// arudha lagna.
    #[must_use]
    pub fn arudha(sign: impl Into<SignRef>) -> SignRef {
        SignRef::Arudha(Box::new(sign.into()))
    }

    /// The badhaka sthana of a sign: `SignRef::badhaka(1)` is the lagna's.
    #[must_use]
    pub fn badhaka(sign: impl Into<SignRef>) -> SignRef {
        SignRef::Badhaka(Box::new(sign.into()))
    }

    /// The sign `house` houses on from `from`.
    #[must_use]
    pub fn counted(from: impl Into<SignRef>, house: House) -> SignRef {
        SignRef::Counted {
            from: Box::new(from.into()),
            house,
        }
    }

    /// The Karakamsha: the atmakaraka's navamsha, among seven karakas (BPHS
    /// ch. 33 v. 1).
    #[must_use]
    pub const fn karakamsha() -> SignRef {
        SignRef::Navamsha(BodyRef::karaka(CharaKaraka::Atmakaraka))
    }

    /// The body, when the reference names one outright.
    #[must_use]
    pub const fn named(&self) -> Option<Body> {
        match self {
            SignRef::Of(body) => body.named(),
            _ => None,
        }
    }
}

impl From<Body> for BodyRef {
    fn from(body: Body) -> BodyRef {
        BodyRef::Body(body)
    }
}

impl From<Graha> for BodyRef {
    fn from(graha: Graha) -> BodyRef {
        BodyRef::Body(Body::Graha(graha))
    }
}

impl From<BodyRef> for SignRef {
    fn from(body: BodyRef) -> SignRef {
        SignRef::Of(body)
    }
}

impl From<Body> for SignRef {
    fn from(body: Body) -> SignRef {
        SignRef::Of(BodyRef::Body(body))
    }
}

impl From<Graha> for SignRef {
    fn from(graha: Graha) -> SignRef {
        SignRef::Of(graha.into())
    }
}

impl From<House> for SignRef {
    fn from(house: House) -> SignRef {
        SignRef::House(house)
    }
}

// A house is written as its number, and a number converts through `House`.
impl TryFrom<u8> for SignRef {
    type Error = String;

    fn try_from(house: u8) -> Result<SignRef, String> {
        House::try_new(house).map(SignRef::House)
    }
}

/// Who a placement condition asks about: a sign reached from a reference, or
/// whichever benefic or malefic first meets the condition.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Subject {
    /// That reference.
    Ref(SignRef),
    /// The first benefic, in [`Body::ALL`]'s order, that meets the condition.
    AnyBenefic,
    /// The first malefic that meets it.
    AnyMalefic,
}

impl From<SignRef> for Subject {
    fn from(reference: SignRef) -> Subject {
        Subject::Ref(reference)
    }
}

impl From<Body> for Subject {
    fn from(body: Body) -> Subject {
        Subject::Ref(body.into())
    }
}

const ANY_BENEFIC: &str = "any-benefic";
const ANY_MALEFIC: &str = "any-malefic";
const UPAPADA: &str = "UPAPADA";

// ---- Prose ----------------------------------------------------------------

impl core::fmt::Display for BodyRef {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            BodyRef::Body(body) => f.write_str(body.key()),
            BodyRef::Bound => f.write_str("the body found"),
            BodyRef::ExaltedIn(sign) => write!(f, "the body exalted in {sign}"),
            BodyRef::LordOf(sign) => write!(f, "the lord of {sign}"),
            BodyRef::Karaka { karaka, scheme } => {
                let abbreviation = karaka.abbreviation();
                match scheme {
                    KarakaScheme::Seven => write!(f, "the {abbreviation}"),
                    KarakaScheme::Eight => write!(f, "the {abbreviation} among eight"),
                }
            }
        }
    }
}

impl core::fmt::Display for SignRef {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SignRef::Of(body) => write!(f, "{body}"),
            SignRef::House(house) => write!(f, "house {}", house.get()),
            SignRef::Arudha(sign) => write!(f, "the pada of {sign}"),
            SignRef::Badhaka(sign) => write!(f, "the badhaka sthana of {sign}"),
            SignRef::Exaltation(body) => write!(f, "the exaltation of {body}"),
            SignRef::Debilitation(body) => write!(f, "the debilitation of {body}"),
            SignRef::Upapada => f.write_str("the upapada"),
            SignRef::Navamsha(body) => write!(f, "the navamsha of {body}"),
            SignRef::Counted { from, house } => write!(f, "house {} from {from}", house.get()),
        }
    }
}

// ---- Writing ---------------------------------------------------------------

impl Serialize for BodyRef {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            BodyRef::Body(body) => body.serialize(serializer),
            BodyRef::LordOf(sign) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("lordOf", sign)?;
                map.end()
            }
            BodyRef::Bound => serializer.serialize_str(SELF),
            BodyRef::ExaltedIn(sign) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("exaltedIn", sign)?;
                map.end()
            }
            BodyRef::Karaka { karaka, scheme } => {
                let eight = *scheme == KarakaScheme::Eight;
                let mut map = serializer.serialize_map(Some(1 + usize::from(eight)))?;
                map.serialize_entry("karaka", karaka)?;
                if eight {
                    map.serialize_entry("scheme", scheme)?;
                }
                map.end()
            }
        }
    }
}

impl Serialize for SignRef {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            SignRef::Of(body) => body.serialize(serializer),
            SignRef::House(house) => house.serialize(serializer),
            SignRef::Upapada => serializer.serialize_str(UPAPADA),
            SignRef::Arudha(sign) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("arudha", sign)?;
                map.end()
            }
            SignRef::Navamsha(body) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("navamsha", body)?;
                map.end()
            }
            SignRef::Badhaka(sign) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("badhakaOf", sign)?;
                map.end()
            }
            SignRef::Exaltation(body) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("exaltationOf", body)?;
                map.end()
            }
            SignRef::Debilitation(body) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("debilitationOf", body)?;
                map.end()
            }
            SignRef::Counted { from, house } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("from", from)?;
                map.serialize_entry("house", house)?;
                map.end()
            }
        }
    }
}

impl Serialize for Subject {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Subject::Ref(reference) => reference.serialize(serializer),
            Subject::AnyBenefic => serializer.serialize_str(ANY_BENEFIC),
            Subject::AnyMalefic => serializer.serialize_str(ANY_MALEFIC),
        }
    }
}

// ---- Reading ---------------------------------------------------------------

/// A reference as written, before its kind is known: a key, a house number,
/// or an object of named parts.
enum Written {
    Key(String),
    Number(u64),
    Object(Vec<(String, Written)>),
}

impl<'de> Deserialize<'de> for Written {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Written, D::Error> {
        struct Shape;

        impl<'de> Visitor<'de> for Shape {
            type Value = Written;

            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("a body's key, a house number or a reference object")
            }

            fn visit_str<E: de::Error>(self, key: &str) -> Result<Written, E> {
                Ok(Written::Key(key.to_owned()))
            }

            fn visit_u64<E: de::Error>(self, number: u64) -> Result<Written, E> {
                Ok(Written::Number(number))
            }

            fn visit_i64<E: de::Error>(self, number: i64) -> Result<Written, E> {
                u64::try_from(number)
                    .map(Written::Number)
                    .map_err(|_| E::custom(format!("house {number} is not 1 to 12")))
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Written, A::Error> {
                let mut parts = Vec::new();
                while let Some((name, part)) = map.next_entry::<String, Written>()? {
                    if parts.iter().any(|(seen, _)| *seen == name) {
                        return Err(de::Error::custom(format!("`{name}` is written twice")));
                    }
                    parts.push((name, part));
                }
                Ok(Written::Object(parts))
            }
        }

        deserializer.deserialize_any(Shape)
    }
}

/// The body forms, for the message that refuses something else.
const BODY_FORMS: &str =
    "a graha's key, LAGNA, SELF, {\"lordOf\": …}, {\"exaltedIn\": …} or {\"karaka\": …}";
/// The parts only a sign reference has.
const SIGN_PARTS: [&str; 7] = [
    "arudha",
    "badhakaOf",
    "navamsha",
    "exaltationOf",
    "debilitationOf",
    "from",
    "house",
];
/// The sign forms.
const SIGN_FORMS: &str = "a body, a house number, UPAPADA, {\"arudha\": …}, {\"badhakaOf\": …}, {\"navamsha\": …} or {\"from\": …, \"house\": …}";

impl Written {
    /// The names of an object's parts.
    fn names(parts: &[(String, Written)]) -> impl Iterator<Item = &str> {
        parts.iter().map(|(name, _)| name.as_str())
    }

    /// The parts of an object, none unknown, each where `names` lists it.
    fn parts<const N: usize>(
        parts: Vec<(String, Written)>,
        names: [&str; N],
    ) -> Result<[Option<Written>; N], String> {
        let mut found: [Option<Written>; N] = [const { None }; N];
        for (name, part) in parts {
            let slot = names
                .iter()
                .position(|known| *known == name)
                .and_then(|at| found.get_mut(at))
                .ok_or_else(|| {
                    format!(
                        "`{name}` is not part of this reference, which takes {}",
                        names.join(" and ")
                    )
                })?;
            *slot = Some(part);
        }
        Ok(found)
    }

    /// The one part an object of one named part holds.
    fn only(parts: Vec<(String, Written)>, name: &str) -> Result<Written, String> {
        let [part] = Written::parts(parts, [name])?;
        part.ok_or_else(|| format!("`{name}` has no value"))
    }

    fn house(self) -> Result<House, String> {
        match self {
            Written::Number(n) => u8::try_from(n)
                .map_err(|_| format!("house {n} is not 1 to 12"))
                .and_then(House::try_new),
            _ => Err(String::from("a house is written as its number, 1 to 12")),
        }
    }

    fn body(self) -> Result<BodyRef, String> {
        match self {
            Written::Key(key) if key == SELF => Ok(BodyRef::Bound),
            Written::Key(key) => Body::from_key(&key).map(BodyRef::Body),
            Written::Number(n) => Err(format!(
                "house {n} names a sign, and this names a body: {BODY_FORMS}"
            )),
            Written::Object(parts) => {
                let has = |wanted: &str| Written::names(&parts).any(|name| name == wanted);
                if has("lordOf") {
                    Ok(BodyRef::lord_of(Written::only(parts, "lordOf")?.sign()?))
                } else if has("exaltedIn") {
                    Ok(BodyRef::ExaltedIn(Box::new(
                        Written::only(parts, "exaltedIn")?.sign()?,
                    )))
                } else if has("karaka") {
                    let [karaka, scheme] = Written::parts(parts, ["karaka", "scheme"])?;
                    let karaka = match karaka {
                        Some(Written::Key(key)) => Karaka::from_abbreviation(&key)?,
                        _ => return Err(String::from("`karaka` is an abbreviation, AK to PiK")),
                    };
                    let scheme = match scheme {
                        None => KarakaScheme::Seven,
                        Some(Written::Number(n)) => u8::try_from(n)
                            .map_err(|_| format!("karaka scheme {n} is not 7 or 8"))
                            .and_then(KarakaScheme::try_from)?,
                        Some(_) => return Err(String::from("`scheme` is 7 or 8")),
                    };
                    Ok(BodyRef::Karaka { karaka, scheme })
                } else if let Some(sign) =
                    Written::names(&parts).find(|name| SIGN_PARTS.contains(name))
                {
                    Err(format!(
                        "`{sign}` names a sign, and this names a body: {BODY_FORMS}"
                    ))
                } else {
                    let names: Vec<&str> = Written::names(&parts).collect();
                    Err(format!(
                        "`{}` is not a reference: {BODY_FORMS}",
                        names.join(", ")
                    ))
                }
            }
        }
    }

    fn sign(self) -> Result<SignRef, String> {
        match self {
            Written::Key(key) if key == UPAPADA => Ok(SignRef::Upapada),
            Written::Number(_) => self.house().map(SignRef::House),
            Written::Object(parts)
                if Written::names(&parts).any(|name| SIGN_PARTS.contains(&name)) =>
            {
                let has = |wanted: &str| Written::names(&parts).any(|name| name == wanted);
                if has("arudha") {
                    Ok(SignRef::arudha(Written::only(parts, "arudha")?.sign()?))
                } else if has("badhakaOf") {
                    Ok(SignRef::badhaka(Written::only(parts, "badhakaOf")?.sign()?))
                } else if has("navamsha") {
                    Ok(SignRef::Navamsha(Written::only(parts, "navamsha")?.body()?))
                } else if has("exaltationOf") {
                    Ok(SignRef::Exaltation(
                        Written::only(parts, "exaltationOf")?.body()?,
                    ))
                } else if has("debilitationOf") {
                    Ok(SignRef::Debilitation(
                        Written::only(parts, "debilitationOf")?.body()?,
                    ))
                } else {
                    match Written::parts(parts, ["from", "house"])? {
                        [Some(from), Some(house)] => {
                            Ok(SignRef::counted(from.sign()?, house.house()?))
                        }
                        _ => Err(String::from("a counted sign takes both `from` and `house`")),
                    }
                }
            }
            body => body.body().map(SignRef::Of).map_err(|reason| {
                if reason.contains("names a sign") {
                    reason
                } else {
                    format!("{reason}; a sign is also {SIGN_FORMS}")
                }
            }),
        }
    }
}

impl<'de> Deserialize<'de> for BodyRef {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<BodyRef, D::Error> {
        Written::deserialize(deserializer)?
            .body()
            .map_err(de::Error::custom)
    }
}

impl<'de> Deserialize<'de> for SignRef {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<SignRef, D::Error> {
        Written::deserialize(deserializer)?
            .sign()
            .map_err(de::Error::custom)
    }
}

impl<'de> Deserialize<'de> for Subject {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Subject, D::Error> {
        match Written::deserialize(deserializer)? {
            Written::Key(key) if key == ANY_BENEFIC => Ok(Subject::AnyBenefic),
            Written::Key(key) if key == ANY_MALEFIC => Ok(Subject::AnyMalefic),
            written => written.sign().map(Subject::Ref).map_err(de::Error::custom),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests unwrap what they read")]

    use super::*;

    fn house(n: u8) -> House {
        House::try_new(n).unwrap()
    }

    #[test]
    fn every_form_reads_as_written_and_writes_back_the_same() {
        let body_forms: [(&str, BodyRef); 4] = [
            (r#""JUPITER""#, Graha::Jupiter.into()),
            (r#""LAGNA""#, Body::Lagna.into()),
            (r#"{"lordOf":10}"#, BodyRef::lord_of(house(10))),
            (
                r#"{"karaka":"AK","scheme":8}"#,
                BodyRef::Karaka {
                    karaka: Karaka(CharaKaraka::Atmakaraka),
                    scheme: KarakaScheme::Eight,
                },
            ),
        ];
        for (json, built) in body_forms {
            assert_eq!(
                serde_json::from_str::<BodyRef>(json).unwrap(),
                built,
                "{json}"
            );
            assert_eq!(serde_json::to_string(&built).unwrap(), json);
            // Every body is a sign too.
            assert_eq!(
                serde_json::from_str::<SignRef>(json).unwrap(),
                SignRef::Of(built)
            );
        }
        let sign_forms: [(&str, SignRef); 5] = [
            ("7", house(7).into()),
            (r#""UPAPADA""#, SignRef::Upapada),
            (
                r#"{"arudha":{"lordOf":"UPAPADA"}}"#,
                SignRef::arudha(BodyRef::lord_of(SignRef::Upapada)),
            ),
            (r#"{"navamsha":{"karaka":"AK"}}"#, SignRef::karakamsha()),
            (
                r#"{"from":{"arudha":1},"house":11}"#,
                SignRef::counted(SignRef::arudha(house(1)), house(11)),
            ),
        ];
        for (json, built) in sign_forms {
            assert_eq!(
                serde_json::from_str::<SignRef>(json).unwrap(),
                built,
                "{json}"
            );
            assert_eq!(serde_json::to_string(&built).unwrap(), json);
        }
        // The karaka's scheme is written only when it is not the default.
        assert_eq!(
            serde_json::from_str::<BodyRef>(r#"{"karaka":"AmK","scheme":7}"#).unwrap(),
            BodyRef::karaka(CharaKaraka::Amatyakaraka)
        );
        assert_eq!(
            serde_json::from_str::<Subject>(r#""any-malefic""#).unwrap(),
            Subject::AnyMalefic
        );
    }

    #[test]
    fn a_reference_of_the_wrong_kind_or_shape_is_refused_with_its_reason() {
        let bodies = [
            (r#"{"arudha": 1}"#, "`arudha` names a sign"),
            (r#"{"from": 1, "house": 2}"#, "names a sign"),
            ("4", "house 4 names a sign"),
            (r#""UPAPADA""#, "`UPAPADA` is not a body"),
            (
                r#"{"lordOf": 1, "extra": 2}"#,
                "`extra` is not part of this reference",
            ),
            (r#"{"karaka": "XK"}"#, "`XK` is not a chara karaka"),
            (r#"{"karaka": "AK", "scheme": 9}"#, "scheme 9 is not 7 or 8"),
            (r#"{"lordOf": 13}"#, "house 13 is not 1 to 12"),
            (r#"{"lordOf": -1}"#, "house -1 is not 1 to 12"),
            (r#"{"lordOf": 1, "lordOf": 2}"#, "written twice"),
            (r#"{"planet": "SUN"}"#, "`planet` is not a reference"),
            (r#""URANUS""#, "`URANUS` is not a body"),
        ];
        for (json, reason) in bodies {
            let error = serde_json::from_str::<BodyRef>(json)
                .unwrap_err()
                .to_string();
            assert!(error.contains(reason), "{json}: {error}");
        }
        let signs = [
            (r#"{"from": "UPAPADA"}"#, "takes both `from` and `house`"),
            (r#"{"navamsha": 3}"#, "house 3 names a sign"),
            (
                r#"{"arudha": 1, "house": 2}"#,
                "`house` is not part of this reference",
            ),
            (r#""NEPTUNE""#, "a sign is also"),
            ("true", "a body's key, a house number or a reference object"),
        ];
        for (json, reason) in signs {
            let error = serde_json::from_str::<SignRef>(json)
                .unwrap_err()
                .to_string();
            assert!(error.contains(reason), "{json}: {error}");
        }
    }

    #[test]
    fn a_predicate_needing_a_body_refuses_a_sign_when_the_rule_is_read() {
        let refused = serde_json::from_str::<crate::Condition>(
            r#"{"type": "planet-dignity", "planet": {"arudha": 1}, "dignities": ["EXALTED"]}"#,
        )
        .unwrap_err()
        .to_string();
        assert!(refused.contains("`arudha` names a sign"), "{refused}");
        assert!(
            serde_json::from_str::<crate::Condition>(
                r#"{"type": "planet-in-kendra", "planet": {"arudha": 1}}"#
            )
            .is_ok()
        );
    }
}
