//! The areas: what a consumer reads operations off.
//!
//! An area is a **borrowing view** of a context, which is the Rust
//! equivalent of what it is in every other binding — a frozen instance
//! in Node, a `late final` field in Dart, a `cached_property` in Python.
//! It allocates nothing, it cannot outlive the context it came from, and
//! `sdk.calendar()` costs a pointer copy, so a consumer who wants to
//! keep one writes `let cal = sdk.calendar();` exactly as a Node
//! consumer writes `const cal = ctx.calendar`.
//!
//! The operations in an area are the ones
//! `03-design/surface-areas.md` puts there and no others. That is not
//! tidiness: `check-areas` holds every binding's list to the same
//! canonical paths, so an operation invented here would be an operation
//! the other three lack.
//!
//! One module per area, because eight of them in one file would be one
//! file nobody reads.

mod calendar;
mod frame;
mod intl;
mod keys;
mod time;

use teistro_calendar::{CalendarSystem, shipped};
use teistro_core::catalogue::Calendar;
use teistro_core::error::Error;

/// The calendar the SDK ships for an id, or the refusal that says why it
/// does not.
///
/// One reading, because two areas need it -- the calendar's six
/// operations and the time area's `civil_of` -- and each would
/// otherwise spell the refusal itself.
pub(crate) fn system_of(id: Calendar) -> Result<&'static dyn CalendarSystem, Error> {
    shipped(id).ok_or_else(|| {
        Error::unsupported(format!("the SDK does not ship the `{}` calendar", id.key()))
            .with_field("calendar")
    })
}

pub use calendar::CalendarArea;
pub use frame::FrameArea;
pub use intl::IntlArea;
pub use keys::KeysArea;
pub use time::TimeArea;
