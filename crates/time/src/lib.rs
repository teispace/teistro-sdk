//! The time layer of the Teistro SDK (`docs/03-design/time-and-timezone.md`).
//!
//! This seed holds what the calendars already need: [`OffsetHistory`], a
//! zone's offsets as rows in force from an instant, answering as a
//! [`LocalClock`]; and [`zones`], the shipped histories, each row cited to
//! tzdb. Time scales and Delta T, zone resolution with its replay-safe
//! metadata, the sunrise-anchored local day and ghati-pala follow.
//!
//! ```
//! use teistro_core::quantity::{JulianDay, Utc};
//! use teistro_core::time::LocalClock;
//! use teistro_time::zones;
//!
//! // Nepal moved from +05:30 to +05:45 at the start of 1986.
//! let nepal = zones::nepal();
//! let before = JulianDay::<Utc>::try_new(2_446_000.0).expect("finite");
//! let after = JulianDay::<Utc>::try_new(2_447_000.0).expect("finite");
//! assert_eq!(nepal.offset_at(before).to_string(), "+05:30");
//! assert_eq!(nepal.offset_at(after).to_string(), "+05:45");
//! ```

pub mod history;
pub mod zones;

pub use history::{OffsetHistory, OffsetRow};
pub use teistro_core::time::{LocalClock, LocalMeanTime, UtcOffset};
