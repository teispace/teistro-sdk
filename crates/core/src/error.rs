//! The error model: one closed status with a stable code, a detail, a
//! message that names the field and the range, a hint when there is one,
//! and an optional message key for localisation. A success never carries
//! a message; a degenerate astronomical outcome is a typed state on the
//! result, never an error (`docs/02-architecture/06-api-conventions.md`).
//!
//! ```
//! use teistro_core::error::{Error, Status};
//! use teistro_core::quantity::Latitude;
//!
//! let error: Error = Latitude::try_new(95.0).unwrap_err().into();
//! assert_eq!(error.status, Status::InvalidArg);
//! assert_eq!(error.status.code(), -1);
//! assert_eq!(error.to_string(), "invalid argument: latitude 95 is outside -90 to 90 degrees");
//! ```

use core::fmt;

use crate::catalogue::UnknownKey;
use crate::quantity::InvalidValue;
use crate::ratio::RatioError;

/// The status of a call, with the code it has at the C boundary.
///
/// ```
/// use teistro_core::error::{Error, Status};
///
/// // Every status crosses JSON by its C name.
/// assert_eq!(serde_json::to_string(&Status::OutOfRange).unwrap(), r#""OUT_OF_RANGE""#);
/// assert_eq!(Status::Limit.to_string(), "LIMIT");
/// ```
#[repr(i32)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Status {
    /// Success.
    Ok = 0,
    /// A value refused at construction, or a request that contradicts itself.
    InvalidArg = -1,
    /// An instant or place outside the provider's or the calendar's coverage.
    OutOfRange = -2,
    /// The settings need something the provider does not declare.
    Capability = -3,
    /// The provider failed; its own code and message are carried.
    Provider = -4,
    /// A search hit its iteration cap.
    NotConverged = -5,
    /// A registered but unimplemented variant, or an unknown key.
    Unsupported = -6,
    /// A pack failed validation or targets another catalogue version.
    Pack = -7,
    /// A batch, range or cache limit was exceeded.
    Limit = -8,
    /// A struct or blob from an incompatible version.
    SchemaVersion = -9,
    /// A panic caught at the boundary; never expected.
    Internal = -10,
}

impl Status {
    /// The stable numeric code.
    ///
    /// ```
    /// use teistro_core::error::Status;
    ///
    /// assert_eq!((Status::Ok.code(), Status::Internal.code()), (0, -10));
    /// ```
    #[must_use]
    pub const fn code(self) -> i32 {
        self as i32
    }

    /// The status with a code.
    ///
    /// ```
    /// use teistro_core::error::Status;
    ///
    /// assert_eq!(Status::from_code(-6), Some(Status::Unsupported));
    /// assert_eq!(Status::from_code(-11), None);
    /// ```
    #[must_use]
    pub const fn from_code(code: i32) -> Option<Status> {
        Some(match code {
            0 => Status::Ok,
            -1 => Status::InvalidArg,
            -2 => Status::OutOfRange,
            -3 => Status::Capability,
            -4 => Status::Provider,
            -5 => Status::NotConverged,
            -6 => Status::Unsupported,
            -7 => Status::Pack,
            -8 => Status::Limit,
            -9 => Status::SchemaVersion,
            -10 => Status::Internal,
            _ => return None,
        })
    }

    /// The name at the C boundary.
    ///
    /// ```
    /// use teistro_core::error::Status;
    ///
    /// assert_eq!(Status::NotConverged.name(), "NOT_CONVERGED");
    /// ```
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Status::Ok => "OK",
            Status::InvalidArg => "INVALID_ARG",
            Status::OutOfRange => "OUT_OF_RANGE",
            Status::Capability => "CAPABILITY",
            Status::Provider => "PROVIDER",
            Status::NotConverged => "NOT_CONVERGED",
            Status::Unsupported => "UNSUPPORTED",
            Status::Pack => "PACK",
            Status::Limit => "LIMIT",
            Status::SchemaVersion => "SCHEMA_VERSION",
            Status::Internal => "INTERNAL",
        }
    }

    /// The phrase an error message opens with.
    ///
    /// ```
    /// use teistro_core::error::{Error, Status};
    ///
    /// assert_eq!(Status::Pack.phrase(), "pack rejected");
    /// assert_eq!(Error::new(Status::Pack, "no rules").to_string(), "pack rejected: no rules");
    /// ```
    #[must_use]
    pub const fn phrase(self) -> &'static str {
        match self {
            Status::Ok => "ok",
            Status::InvalidArg => "invalid argument",
            Status::OutOfRange => "out of range",
            Status::Capability => "capability missing",
            Status::Provider => "provider failed",
            Status::NotConverged => "did not converge",
            Status::Unsupported => "unsupported",
            Status::Pack => "pack rejected",
            Status::Limit => "limit exceeded",
            Status::SchemaVersion => "schema version mismatch",
            Status::Internal => "internal error",
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// What, more precisely, went wrong; appended as the modules need.
///
/// ```
/// use teistro_core::error::Detail;
///
/// assert_eq!(serde_json::to_string(&Detail::DstGap).unwrap(), r#""DST_GAP""#);
/// ```
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Detail {
    /// A registered variant with no implementation (ADR-0018).
    Unsourced,
    /// A key nobody has.
    UnknownKey,
    /// A former key, still resolving.
    DeprecatedKey,
    /// A civil time inside a daylight-saving gap.
    DstGap,
    /// A birth time absent under a policy that refuses fallbacks.
    TimeUnknown,
    /// A calendar date that does not exist (a transition gap, a day beyond
    /// the month).
    NonexistentDate,
    /// A registry that no longer accepts definitions.
    Sealed,
    /// An overflow of the exact arithmetic.
    Overflow,
    /// A batch larger than the context allows.
    BatchTooLarge,
}

impl Detail {
    /// The key it is serialised as, and the one every binding reads:
    /// `UNKNOWN_KEY`, `DST_GAP`.
    ///
    /// ```
    /// use teistro_core::error::Detail;
    ///
    /// assert_eq!(Detail::BatchTooLarge.key(), "BATCH_TOO_LARGE");
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Detail::Unsourced => "UNSOURCED",
            Detail::UnknownKey => "UNKNOWN_KEY",
            Detail::DeprecatedKey => "DEPRECATED_KEY",
            Detail::DstGap => "DST_GAP",
            Detail::TimeUnknown => "TIME_UNKNOWN",
            Detail::NonexistentDate => "NONEXISTENT_DATE",
            Detail::Sealed => "SEALED",
            Detail::Overflow => "OVERFLOW",
            Detail::BatchTooLarge => "BATCH_TOO_LARGE",
        }
    }
}

/// A reference to a localisable message: a key and its slots.
///
/// ```
/// use teistro_core::error::{Error, Status};
///
/// let error = Error::invalid_arg("no such day").with_key("sdk.error.dstGap", vec![(String::from("zone"), String::from("Asia/Kathmandu"))]);
/// let key = error.key().unwrap();
/// assert_eq!((key.key.as_str(), key.slots.len()), ("sdk.error.dstGap", 1));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MessageRef {
    /// The message key (`sdk.error.dstGap`).
    pub key: String,
    /// The slots, as name and value.
    pub slots: Vec<(String, String)>,
}

/// An error. The status, the detail and the message are always present;
/// the field, the hint and the localisable form live behind one pointer so
/// that a `Result` stays small on the success path.
///
/// ```
/// use teistro_core::error::{Detail, Error, Status};
///
/// let error = Error::new(Status::OutOfRange, "JD 1e7 is past the data").with_field("instant").with_hint("load a longer ephemeris");
/// assert_eq!(error.to_string(), "out of range: JD 1e7 is past the data (field `instant`); load a longer ephemeris");
/// // It crosses JSON whole, and reads back the same.
/// let json = serde_json::to_string(&error).unwrap();
/// assert_eq!(serde_json::from_str::<Error>(&json).unwrap(), error);
/// assert_eq!(error.with_detail(Detail::Overflow).detail, Some(Detail::Overflow));
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    /// The status.
    pub status: Status,
    /// The detail, when there is one.
    pub detail: Option<Detail>,
    /// An English message naming the field and the accepted range.
    pub message: String,
    extra: Option<Box<Extra>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct Extra {
    field: Option<String>,
    hint: Option<String>,
    key: Option<MessageRef>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct ErrorRepr {
    status: Status,
    detail: Option<Detail>,
    message: String,
    field: Option<String>,
    hint: Option<String>,
    key: Option<MessageRef>,
}

impl Error {
    /// An error with a status and a message.
    ///
    /// ```
    /// use teistro_core::error::{Error, Status};
    ///
    /// let error = Error::new(Status::NotConverged, "the search stopped at 64 steps");
    /// assert_eq!((error.status, error.detail, error.field()), (Status::NotConverged, None, None));
    /// ```
    #[must_use]
    pub fn new(status: Status, message: impl Into<String>) -> Error {
        Error {
            status,
            detail: None,
            message: message.into(),
            extra: None,
        }
    }

    /// `INVALID_ARG`.
    ///
    /// ```
    /// use teistro_core::error::{Error, Status};
    ///
    /// assert_eq!(Error::invalid_arg("x").status, Status::InvalidArg);
    /// ```
    #[must_use]
    pub fn invalid_arg(message: impl Into<String>) -> Error {
        Error::new(Status::InvalidArg, message)
    }

    /// `UNSUPPORTED`.
    ///
    /// ```
    /// use teistro_core::error::{Error, Status};
    ///
    /// assert_eq!(Error::unsupported("x").status, Status::Unsupported);
    /// ```
    #[must_use]
    pub fn unsupported(message: impl Into<String>) -> Error {
        Error::new(Status::Unsupported, message)
    }

    /// `CAPABILITY`: a module family this build leaves out (`kp`,
    /// `western`, …), with the build that has it as the hint.
    ///
    /// ```
    /// use teistro_core::error::{Error, Status};
    ///
    /// let error = Error::left_out("kp");
    /// assert_eq!(error.status, Status::Capability);
    /// assert_eq!(error.message, "this build leaves out the `kp` module family");
    /// assert!(error.hint().unwrap().contains("`full`"));
    /// ```
    #[must_use]
    pub fn left_out(family: &str) -> Error {
        Error::new(
            Status::Capability,
            format!("this build leaves out the `{family}` module family"),
        )
        .with_hint(format!(
            "use a build with `{family}`: the default `full` build has every family, and the wasm module's is `@teistro/sdk-wasm` rather than a profile's subpath"
        ))
    }

    /// `LIMIT`.
    ///
    /// ```
    /// use teistro_core::error::{Error, Status};
    ///
    /// assert_eq!(Error::limit("x").status.code(), -8);
    /// ```
    #[must_use]
    pub fn limit(message: impl Into<String>) -> Error {
        Error::new(Status::Limit, message)
    }

    /// `INTERNAL`.
    ///
    /// ```
    /// use teistro_core::error::{Error, Status};
    ///
    /// assert_eq!(Error::internal("x").status.code(), -10);
    /// ```
    #[must_use]
    pub fn internal(message: impl Into<String>) -> Error {
        Error::new(Status::Internal, message)
    }

    fn extra_mut(&mut self) -> &mut Extra {
        self.extra.get_or_insert_with(Box::default)
    }

    /// Adds the detail.
    ///
    /// ```
    /// use teistro_core::error::{Detail, Error};
    ///
    /// let error = Error::invalid_arg("02:30 did not happen").with_detail(Detail::DstGap);
    /// assert_eq!(error.detail, Some(Detail::DstGap));
    /// ```
    #[must_use]
    pub fn with_detail(mut self, detail: Detail) -> Error {
        self.detail = Some(detail);
        self
    }

    /// Names the field.
    ///
    /// ```
    /// use teistro_core::error::Error;
    ///
    /// assert_eq!(Error::invalid_arg("x").with_field("place.latitude").field(), Some("place.latitude"));
    /// ```
    #[must_use]
    pub fn with_field(mut self, field: impl Into<String>) -> Error {
        self.extra_mut().field = Some(field.into());
        self
    }

    /// Names the field from an outer record: `root.field`, or `root` itself
    /// when the error named none. What a reader of one element of a list
    /// calls to say which element it was (`layouts_json[2].shape.rings`).
    ///
    /// A field that begins with an index is joined without a dot:
    /// `drawings` under `request` is `request.drawings`, `[1]` is
    /// `request[1]`.
    ///
    /// ```
    /// use teistro_core::error::Error;
    ///
    /// let inner = Error::invalid_arg("x").with_field("shape.rings");
    /// assert_eq!(inner.under("layouts_json[2]").field(), Some("layouts_json[2].shape.rings"));
    /// // An index joins without a dot; no field at all takes the root.
    /// assert_eq!(Error::invalid_arg("x").with_field("[1]").under("request").field(), Some("request[1]"));
    /// assert_eq!(Error::invalid_arg("x").under("request").field(), Some("request"));
    /// ```
    #[must_use]
    pub fn under(self, root: &str) -> Error {
        let field = match self.field() {
            None | Some("") => root.to_owned(),
            Some(inner) if inner.starts_with('[') => format!("{root}{inner}"),
            Some(inner) => format!("{root}.{inner}"),
        };
        self.with_field(field)
    }

    /// Adds a hint.
    ///
    /// ```
    /// use teistro_core::error::Error;
    ///
    /// let error = Error::invalid_arg("no zone `Asia/Kathmandoo`").with_hint("did you mean `Asia/Kathmandu`?");
    /// assert_eq!(error.hint(), Some("did you mean `Asia/Kathmandu`?"));
    /// ```
    #[must_use]
    pub fn with_hint(mut self, hint: impl Into<String>) -> Error {
        self.extra_mut().hint = Some(hint.into());
        self
    }

    /// Adds the localisable form.
    ///
    /// ```
    /// use teistro_core::error::Error;
    ///
    /// let error = Error::invalid_arg("x").with_key("sdk.error.timeUnknown", Vec::new());
    /// assert_eq!(error.key().map(|key| key.key.as_str()), Some("sdk.error.timeUnknown"));
    /// ```
    #[must_use]
    pub fn with_key(mut self, key: impl Into<String>, slots: Vec<(String, String)>) -> Error {
        self.extra_mut().key = Some(MessageRef {
            key: key.into(),
            slots,
        });
        self
    }

    /// The field involved, when known.
    ///
    /// ```
    /// use teistro_core::error::Error;
    /// use teistro_core::quantity::Place;
    ///
    /// // A refused value names its field when it becomes an error.
    /// let error: Error = Place::try_from_degrees(91.0, 0.0, 0.0).unwrap_err().into();
    /// assert_eq!(error.field(), Some("lat_deg"));
    /// ```
    #[must_use]
    pub fn field(&self) -> Option<&str> {
        self.extra.as_ref().and_then(|e| e.field.as_deref())
    }

    /// The hint, when there is one.
    ///
    /// ```
    /// use teistro_core::error::Error;
    ///
    /// assert_eq!(Error::invalid_arg("x").hint(), None);
    /// ```
    #[must_use]
    pub fn hint(&self) -> Option<&str> {
        self.extra.as_ref().and_then(|e| e.hint.as_deref())
    }

    /// The localisable form, when there is one.
    ///
    /// ```
    /// use teistro_core::error::Error;
    ///
    /// assert!(Error::invalid_arg("x").key().is_none());
    /// ```
    #[must_use]
    pub fn key(&self) -> Option<&MessageRef> {
        self.extra.as_ref().and_then(|e| e.key.as_ref())
    }
}

impl serde::Serialize for Error {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        ErrorRepr {
            status: self.status,
            detail: self.detail,
            message: self.message.clone(),
            field: self.field().map(str::to_string),
            hint: self.hint().map(str::to_string),
            key: self.key().cloned(),
        }
        .serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Error {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Error, D::Error> {
        let repr = ErrorRepr::deserialize(deserializer)?;
        let mut error = Error::new(repr.status, repr.message);
        error.detail = repr.detail;
        if repr.field.is_some() || repr.hint.is_some() || repr.key.is_some() {
            error.extra = Some(Box::new(Extra {
                field: repr.field,
                hint: repr.hint,
                key: repr.key,
            }));
        }
        Ok(error)
    }
}

/// The shape an error is written in, which is its private representation's.
#[cfg(feature = "schema")]
impl schemars::JsonSchema for Error {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("Error")
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        ErrorRepr::json_schema(generator)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.status.phrase(), self.message)?;
        if let Some(field) = self.field() {
            write!(f, " (field `{field}`)")?;
        }
        if let Some(hint) = self.hint() {
            write!(f, "; {hint}")?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {}

impl From<InvalidValue> for Error {
    fn from(invalid: InvalidValue) -> Error {
        let field = invalid.field.clone();
        let error = Error::invalid_arg(
            InvalidValue {
                field: None,
                ..invalid
            }
            .to_string(),
        );
        match field {
            Some(field) => error.with_field(field),
            None => error,
        }
    }
}

impl From<UnknownKey> for Error {
    fn from(unknown: UnknownKey) -> Error {
        let hint = unknown.suggestion.map(|s| format!("did you mean `{s}`?"));
        let error = Error::unsupported(
            UnknownKey {
                suggestion: None,
                ..unknown
            }
            .to_string(),
        )
        .with_detail(Detail::UnknownKey);
        match hint {
            Some(hint) => error.with_hint(hint),
            None => error,
        }
    }
}

impl From<RatioError> for Error {
    fn from(error: RatioError) -> Error {
        Error::internal(error.to_string()).with_detail(Detail::Overflow)
    }
}

/// A result carrying the SDK's error.
pub type Result<T, E = Error> = core::result::Result<T, E>;

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;

    /// A detail's key is its serialised spelling, so a program matching
    /// on either reads the same word.
    #[test]
    fn a_detail_is_keyed_as_it_serialises() {
        for detail in [
            Detail::Unsourced,
            Detail::UnknownKey,
            Detail::DeprecatedKey,
            Detail::DstGap,
            Detail::TimeUnknown,
            Detail::NonexistentDate,
            Detail::Sealed,
            Detail::Overflow,
            Detail::BatchTooLarge,
        ] {
            assert_eq!(
                serde_json::to_string(&detail).unwrap(),
                format!("\"{}\"", detail.key())
            );
        }
    }

    #[test]
    fn a_field_is_named_from_an_outer_record() {
        let inner = Error::invalid_arg("x").with_field("shape.rings");
        assert_eq!(
            inner.under("layouts_json[2]").field(),
            Some("layouts_json[2].shape.rings")
        );
        assert_eq!(
            Error::invalid_arg("x").under("theme_json").field(),
            Some("theme_json")
        );
        let indexed = Error::invalid_arg("x").with_field("[1].varga");
        assert_eq!(indexed.under("drawings").field(), Some("drawings[1].varga"));
    }
    use crate::catalogue::Graha;

    #[test]
    fn codes_round_trip_and_messages_read() {
        for status in [
            Status::Ok,
            Status::InvalidArg,
            Status::OutOfRange,
            Status::Capability,
            Status::Provider,
            Status::NotConverged,
            Status::Unsupported,
            Status::Pack,
            Status::Limit,
            Status::SchemaVersion,
            Status::Internal,
        ] {
            assert_eq!(Status::from_code(status.code()), Some(status));
        }
        assert_eq!(Status::from_code(7), None);
        let error: Error = "SUNN".parse::<Graha>().unwrap_err().into();
        assert_eq!(error.status, Status::Unsupported);
        assert_eq!(error.detail, Some(Detail::UnknownKey));
        assert_eq!(
            error.to_string(),
            "unsupported: unknown graha key `SUNN`; did you mean `SUN`?"
        );
        let error = Error::invalid_arg("a topocentric frame needs an observer")
            .with_field("observer")
            .with_key("sdk.error.observerMissing", vec![]);
        assert_eq!(
            error.to_string(),
            "invalid argument: a topocentric frame needs an observer (field `observer`)"
        );
        let json = serde_json::to_string(&error).unwrap_or_default();
        assert!(
            json.contains("\"status\":\"INVALID_ARG\"") && json.contains("\"field\":\"observer\"")
        );
        let back: Error = serde_json::from_str(&json).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(back, error);
        assert!(
            core::mem::size_of::<Error>() <= 48,
            "{}",
            core::mem::size_of::<Error>()
        );
    }

    #[test]
    fn every_status_names_itself_and_opens_its_message() {
        let all = [
            (Status::Ok, "OK", "ok"),
            (Status::InvalidArg, "INVALID_ARG", "invalid argument"),
            (Status::OutOfRange, "OUT_OF_RANGE", "out of range"),
            (Status::Capability, "CAPABILITY", "capability missing"),
            (Status::Provider, "PROVIDER", "provider failed"),
            (Status::NotConverged, "NOT_CONVERGED", "did not converge"),
            (Status::Unsupported, "UNSUPPORTED", "unsupported"),
            (Status::Pack, "PACK", "pack rejected"),
            (Status::Limit, "LIMIT", "limit exceeded"),
            (
                Status::SchemaVersion,
                "SCHEMA_VERSION",
                "schema version mismatch",
            ),
            (Status::Internal, "INTERNAL", "internal error"),
        ];
        for (status, name, phrase) in all {
            assert_eq!((status.name(), status.to_string()), (name, name.to_owned()));
            assert_eq!(status.phrase(), phrase);
            let error = Error::new(status, "m");
            assert_eq!(error.to_string(), format!("{phrase}: m"));
        }
    }

    #[test]
    fn every_conversion_keeps_its_status_field_and_hint() {
        let left = Error::left_out("kp");
        assert_eq!(left.status, Status::Capability);
        assert!(
            left.to_string()
                .contains("`kp` module family; use a build with `kp`")
        );
        assert_eq!(Error::limit("m").status, Status::Limit);
        assert_eq!(Error::internal("m").status, Status::Internal);

        let invalid = |field: Option<&str>| InvalidValue {
            quantity: "latitude",
            value: String::from("91"),
            accepted: "-90 to 90 degrees",
            field: field.map(String::from),
        };
        let named = Error::from(invalid(Some("place.latitude")));
        assert_eq!(
            (named.status, named.field()),
            (Status::InvalidArg, Some("place.latitude"))
        );
        assert_eq!(Error::from(invalid(None)).field(), None);

        let unknown = Error::from(UnknownKey {
            kind: None,
            key: String::from("zzz"),
            suggestion: None,
        });
        assert_eq!(
            (unknown.status, unknown.detail, unknown.hint()),
            (Status::Unsupported, Some(Detail::UnknownKey), None)
        );

        let overflow = Error::from(RatioError::Overflow);
        assert_eq!(
            (overflow.status, overflow.detail),
            (Status::Internal, Some(Detail::Overflow))
        );
        let boxed: Box<dyn std::error::Error> = Box::new(overflow);
        assert_eq!(
            boxed.to_string(),
            "internal error: the rational exceeds the exact range"
        );
    }
}
