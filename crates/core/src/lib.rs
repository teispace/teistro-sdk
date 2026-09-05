//! The core of the Teistro SDK: the vocabulary every other crate speaks.
//!
//! - [`catalogue`]: every kind of entity (grahas, signs, nakshatras,
//!   tithis and the rest) as an enum whose discriminants are stable ids,
//!   with cited attributes, generated from `catalogue/*.yaml`;
//! - [`key`]: keys (`graha.SUN`), packed ids for the C boundary, and
//!   resolution with suggestions;
//! - [`quantity`]: validated newtypes for every domain quantity, so a
//!   latitude cannot be passed where a longitude is expected;
//! - [`angle`]: the canonical nanoarcsecond angle and exact
//!   classification (ADR-0016);
//! - [`ratio`]: exact rationals for period arithmetic;
//! - [`error`] and [`envelope`]: the status codes, the error, and the
//!   provenance every result carries (ADR-0020);
//! - [`settings`]: the knobs, the profiles, their resolution, coherence
//!   and the hash every result carries;
//! - [`time`]: clock offsets and the local-clock abstraction the calendars
//!   and the time layer share;
//! - [`registry`] and [`limits`]: what a context registers and bounds.
//!
//! ```
//! use teistro_core::angle::Nas;
//! use teistro_core::catalogue::{Graha, Rashi};
//! use teistro_core::quantity::Degrees;
//!
//! let mars = Graha::from_key("MARS").expect("a catalogued key");
//! assert_eq!(mars.attributes().exaltation.map(|e| e.sign), Some(Rashi::Capricorn));
//!
//! let longitude = Nas::from_degrees(Degrees::try_new(222.5763).expect("finite"));
//! assert_eq!(longitude.sign(), Rashi::Scorpio);
//! assert_eq!(longitude.nakshatra().key(), "ANURADHA");
//! ```

pub mod angle;
pub mod catalogue;
pub mod envelope;
pub mod error;
pub mod key;
pub mod limits;
pub mod quantity;
pub mod ratio;
pub mod registry;
pub mod settings;
pub mod time;

pub use angle::Nas;
pub use catalogue::{Catalogued, Kind, Mark, Source, UnknownKey};
pub use envelope::{Envelope, Provenance};
pub use error::{Error, Result, Status};
pub use key::KeyId;
pub use quantity::{Altitude, Degrees, InvalidValue, JulianDay, Latitude, Longitude, Place};
pub use ratio::Ratio;
pub use settings::{Profile, Settings, SettingsPatch};
pub use time::{LocalClock, UtcOffset};
