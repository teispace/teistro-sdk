//! What a context bounds: batch sizes, iteration caps, depth and cache
//! bytes. Limits are configuration, not settings: they do not move a
//! number, so they stay outside the settings hash and are stamped only
//! when they bite (`LIMIT`).

use crate::error::{Detail, Error};
use crate::quantity::Depth;

/// The bounds of one context.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Limits {
    /// The most instants, bodies or charts one call may carry.
    pub max_batch: usize,
    /// The iteration cap of every solver.
    pub max_iterations: u32,
    /// The deepest dasha level a request may ask for.
    pub max_depth: Depth,
    /// The memo cache's budget in bytes.
    pub cache_bytes: usize,
}

impl Limits {
    /// The defaults every profile starts from.
    pub const DEFAULT: Limits = Limits {
        max_batch: 100_000,
        max_iterations: 200,
        max_depth: Depth::MAX,
        cache_bytes: 64 << 20,
    };

    /// Checks a batch size against the limit.
    ///
    /// # Errors
    ///
    /// `LIMIT` naming the size and the bound.
    pub fn check_batch(&self, size: usize) -> Result<(), Error> {
        if size <= self.max_batch {
            Ok(())
        } else {
            Err(Error::limit(format!(
                "a batch of {size} exceeds the context's limit of {}",
                self.max_batch
            ))
            .with_detail(Detail::BatchTooLarge))
        }
    }

    /// Checks a requested depth against the limit.
    ///
    /// # Errors
    ///
    /// `LIMIT` naming the depth and the bound.
    pub fn check_depth(&self, depth: Depth) -> Result<(), Error> {
        if depth <= self.max_depth {
            Ok(())
        } else {
            Err(Error::limit(format!(
                "depth {depth} exceeds the context's limit of {}",
                self.max_depth
            )))
        }
    }
}

impl Default for Limits {
    fn default() -> Limits {
        Limits::DEFAULT
    }
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
    fn limits_bite_with_named_bounds() {
        let limits = Limits::default();
        assert!(limits.check_batch(100_000).is_ok());
        let error = limits.check_batch(100_001).unwrap_err();
        assert_eq!(error.detail, Some(Detail::BatchTooLarge));
        assert!(error.message.contains("100000"));
        assert!(limits.check_depth(Depth::MAX).is_ok());
    }
}
