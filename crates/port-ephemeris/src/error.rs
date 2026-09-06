//! Why a provider could not answer: a whole-batch failure with a stable
//! code at the C boundary, and its form as the SDK's error.

use core::fmt;

use serde::Serialize;
use teistro_core::error::{Error, Status};

use crate::vtable::ProviderCode;

/// Why a provider could not answer. A per-cell failure is a
/// [`crate::CellStatus`], not an error.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProviderError {
    /// The operation or option is not implemented by this provider.
    Unsupported {
        /// What was asked for.
        what: String,
    },
    /// The instant is outside coverage.
    OutOfRange {
        /// The instant.
        jd: f64,
    },
    /// A data file is missing.
    DataMissing {
        /// Which.
        detail: String,
    },
    /// The provider refused rather than answer silently with something
    /// else (a fallback model, a stale setting).
    Refused {
        /// Why.
        detail: String,
    },
    /// The request is malformed.
    Invalid {
        /// What is wrong.
        detail: String,
    },
    /// The provider's own error, with a code outside the reserved range.
    Provider {
        /// Its code.
        code: i32,
        /// Its message.
        detail: String,
    },
}

impl ProviderError {
    /// The codes the port reserves at the C boundary: a provider's own
    /// codes stay outside `-6` to `-1` and `0`.
    pub const RESERVED_CODES: core::ops::RangeInclusive<i32> = -6..=0;

    /// The stable code at the C boundary, which every binding reads from
    /// [`ProviderCode`] rather than writing as a number.
    #[must_use]
    pub fn code(&self) -> i32 {
        let named = match self {
            ProviderError::Unsupported { .. } => ProviderCode::Unsupported,
            ProviderError::OutOfRange { .. } => ProviderCode::OutOfRange,
            ProviderError::DataMissing { .. } => ProviderCode::DataMissing,
            ProviderError::Refused { .. } => ProviderCode::Refused,
            ProviderError::Invalid { .. } => ProviderCode::Invalid,
            ProviderError::Provider { code, .. } => return *code,
        };
        named as i32
    }

    /// The error a C code stands for, with a context string as detail.
    #[must_use]
    pub fn from_code(code: i32, context: &str) -> ProviderError {
        if code == ProviderCode::Unsupported as i32 {
            ProviderError::unsupported(context)
        } else if code == ProviderCode::OutOfRange as i32 {
            ProviderError::OutOfRange { jd: f64::NAN }
        } else if code == ProviderCode::DataMissing as i32 {
            ProviderError::DataMissing {
                detail: context.to_string(),
            }
        } else if code == ProviderCode::Refused as i32 {
            ProviderError::Refused {
                detail: context.to_string(),
            }
        } else if code == ProviderCode::Invalid as i32 {
            ProviderError::invalid(context)
        } else {
            ProviderError::Provider {
                code,
                detail: context.to_string(),
            }
        }
    }

    /// An unsupported-operation error.
    #[must_use]
    pub fn unsupported(what: impl Into<String>) -> ProviderError {
        ProviderError::Unsupported { what: what.into() }
    }

    /// A malformed-request error.
    #[must_use]
    pub fn invalid(detail: impl Into<String>) -> ProviderError {
        ProviderError::Invalid {
            detail: detail.into(),
        }
    }

    /// The SDK status the error maps to.
    #[must_use]
    pub const fn status(&self) -> Status {
        match self {
            ProviderError::Unsupported { .. } => Status::Unsupported,
            ProviderError::OutOfRange { .. } => Status::OutOfRange,
            ProviderError::Invalid { .. } => Status::InvalidArg,
            ProviderError::DataMissing { .. }
            | ProviderError::Refused { .. }
            | ProviderError::Provider { .. } => Status::Provider,
        }
    }
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProviderError::Unsupported { what } => {
                write!(f, "the provider does not support {what}")
            }
            ProviderError::OutOfRange { jd } => {
                write!(f, "JD {jd} is outside the provider's coverage")
            }
            ProviderError::DataMissing { detail } => write!(f, "a data file is missing: {detail}"),
            ProviderError::Refused { detail } => write!(f, "the provider refused: {detail}"),
            ProviderError::Invalid { detail } => write!(f, "invalid request: {detail}"),
            ProviderError::Provider { code, detail } => {
                write!(f, "provider error {code}: {detail}")
            }
        }
    }
}

impl std::error::Error for ProviderError {}

impl From<ProviderError> for Error {
    fn from(error: ProviderError) -> Error {
        let message = error.to_string();
        let sdk = Error::new(error.status(), message);
        match error {
            ProviderError::OutOfRange { .. } => sdk.with_field("jd"),
            ProviderError::Unsupported { .. } => sdk.with_hint(
                "choose a provider that declares the operation, or the SDK's own implementation through the override policy",
            ),
            _ => sdk,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    #[test]
    fn codes_round_trip_and_map_to_statuses() {
        let errors = [
            ProviderError::unsupported("houses"),
            ProviderError::DataMissing {
                detail: String::from("sepl_18.se1"),
            },
            ProviderError::Refused {
                detail: String::from("a second path"),
            },
            ProviderError::invalid("no observer"),
            ProviderError::Provider {
                code: -102,
                detail: String::from("engine"),
            },
        ];
        for error in errors {
            let back = ProviderError::from_code(error.code(), "context");
            assert_eq!(back.code(), error.code());
            assert!(!ProviderError::RESERVED_CODES.contains(&-102));
            let sdk: Error = error.clone().into();
            assert_eq!(sdk.status, error.status());
        }
        assert!(matches!(
            ProviderError::from_code(-2, ""),
            ProviderError::OutOfRange { .. }
        ));
        let sdk: Error = ProviderError::OutOfRange { jd: 1.0 }.into();
        assert_eq!(sdk.field(), Some("jd"));
        assert_eq!(sdk.status, Status::OutOfRange);
        assert_eq!(
            ProviderError::unsupported("houses").to_string(),
            "the provider does not support houses"
        );
    }
}
