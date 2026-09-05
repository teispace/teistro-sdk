//! The calendars of the Teistro SDK: the fixed day every calendar
//! converts through, the four arithmetic calendars (proleptic Gregorian,
//! proleptic Julian, the mixed civil calendar with its transition, the
//! ISO week date), and Bikram Sambat over its table
//! (`docs/03-design/calendar-gregorian-julian.md`,
//! `calendar-bikram-sambat.md`).
//!
//! ```
//! use teistro_calendar::{CalendarSystem, FixedDay, Gregorian, BikramSambat};
//!
//! let day = Gregorian.to_fixed_ymd(2015, 4, 14).expect("a real date");
//! let bs = BikramSambat::shipped().date_of(day).expect("in the table");
//! assert_eq!((bs.year, bs.month, bs.day), (2072, 1, 1));
//! assert_eq!(day.weekday().to_string(), "Tuesday");
//! ```

pub mod bikram_sambat;
pub mod date;
pub mod fixed;
pub mod gregorian;
pub mod iso_week;
pub mod julian;
pub mod mixed;
pub mod solar;

pub use bikram_sambat::BikramSambat;
pub use date::{CalendarCapabilities, CalendarDate, CalendarSystem, EraNumber, shipped};
pub use fixed::{FixedDay, Weekday};
pub use gregorian::Gregorian;
pub use iso_week::{IsoWeek, IsoWeekDate};
pub use julian::Julian;
pub use mixed::{Mixed, Transition};
pub use solar::{DayLight, MonthStartRule, SolarModel};
