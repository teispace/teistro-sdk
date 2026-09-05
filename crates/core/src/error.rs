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
#[repr(i32)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
    #[must_use]
    pub const fn code(self) -> i32 {
        self as i32
    }

    /// The status with a code.
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
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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

/// A reference to a localisable message: a key and its slots.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MessageRef {
    /// The message key (`sdk.error.dstGap`).
    pub key: String,
    /// The slots, as name and value.
    pub slots: Vec<(String, String)>,
}

/// An error. The status, the detail and the message are always present;
/// the field, the hint and the localisable form live behind one pointer so
/// that a `Result` stays small on the success path.
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
struct Extra {
    field: Option<String>,
    hint: Option<String>,
    key: Option<MessageRef>,
}

#[derive(serde::Serialize, serde::Deserialize)]
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
    #[must_use]
    pub fn invalid_arg(message: impl Into<String>) -> Error {
        Error::new(Status::InvalidArg, message)
    }

    /// `UNSUPPORTED`.
    #[must_use]
    pub fn unsupported(message: impl Into<String>) -> Error {
        Error::new(Status::Unsupported, message)
    }

    /// `LIMIT`.
    #[must_use]
    pub fn limit(message: impl Into<String>) -> Error {
        Error::new(Status::Limit, message)
    }

    /// `INTERNAL`.
    #[must_use]
    pub fn internal(message: impl Into<String>) -> Error {
        Error::new(Status::Internal, message)
    }

    fn extra_mut(&mut self) -> &mut Extra {
        self.extra.get_or_insert_with(Box::default)
    }

    /// Adds the detail.
    #[must_use]
    pub fn with_detail(mut self, detail: Detail) -> Error {
        self.detail = Some(detail);
        self
    }

    /// Names the field.
    #[must_use]
    pub fn with_field(mut self, field: impl Into<String>) -> Error {
        self.extra_mut().field = Some(field.into());
        self
    }

    /// Adds a hint.
    #[must_use]
    pub fn with_hint(mut self, hint: impl Into<String>) -> Error {
        self.extra_mut().hint = Some(hint.into());
        self
    }

    /// Adds the localisable form.
    #[must_use]
    pub fn with_key(mut self, key: impl Into<String>, slots: Vec<(String, String)>) -> Error {
        self.extra_mut().key = Some(MessageRef {
            key: key.into(),
            slots,
        });
        self
    }

    /// The field involved, when known.
    #[must_use]
    pub fn field(&self) -> Option<&str> {
        self.extra.as_ref().and_then(|e| e.field.as_deref())
    }

    /// The hint, when there is one.
    #[must_use]
    pub fn hint(&self) -> Option<&str> {
        self.extra.as_ref().and_then(|e| e.hint.as_deref())
    }

    /// The localisable form, when there is one.
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
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;
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
}
