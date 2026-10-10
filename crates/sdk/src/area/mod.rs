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

mod almanac;
mod calendar;
#[cfg(feature = "chart")]
mod chart;
mod engine;
mod frame;
#[cfg(feature = "chart")]
mod interpret;
mod intl;
mod keys;
#[cfg(feature = "chart")]
mod matching;
#[cfg(feature = "numerology")]
mod numerology;
#[cfg(feature = "research")]
pub(crate) mod research;
mod time;

use teistro_calendar::solar::drik::DrikSun;
use teistro_calendar::{CalendarSystem, shipped};
use teistro_core::catalogue::{Ayanamsha, Calendar};
use teistro_core::error::Error;
use teistro_core::settings::AyanamshaChoice;
use teistro_port_ephemeris::EphemerisProvider;

use crate::context::Context;

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

/// The solar model a context reckons its days with: its ayanamsha, sunrise
/// convention, override policy and Delta T model.
///
/// One construction, because the chart founder, the almanac and the
/// muhurta search each need it and must agree on it. A custom ayanamsha is
/// a value rather than a catalogue member, and the model wants a member;
/// Lahiri is what the boundary substitutes, so this substitutes the same.
pub(crate) fn drik_sun<'p>(
    context: &Context,
    provider: &'p (dyn EphemerisProvider + 'p),
) -> DrikSun<'p, dyn EphemerisProvider + 'p> {
    let settings = context.settings();
    let ayanamsha = match settings.frame.ayanamsha {
        AyanamshaChoice::Catalogued { id } => id,
        AyanamshaChoice::Custom { .. } => Ayanamsha::Lahiri,
    };
    DrikSun::new(
        provider,
        ayanamsha,
        settings.day.sunrise,
        settings.provider.overrides,
        context.delta_t(),
    )
}

#[cfg(feature = "muhurta")]
pub use almanac::MuhurtaDays;
pub use almanac::{AlmanacAnswer, AlmanacArea, AlmanacRequest, AlmanacSections, FestivalDays};
pub use calendar::CalendarArea;
#[cfg(feature = "chart")]
pub use chart::{ChartArea, Interpreted};
pub use engine::EngineArea;
pub use frame::FrameArea;
#[cfg(feature = "chart")]
pub use interpret::{Answers, InterpretArea, Plans};
pub use intl::IntlArea;
pub use keys::KeysArea;
#[cfg(feature = "chart")]
pub use matching::MatchingArea;
#[cfg(feature = "numerology")]
pub use numerology::NumerologyArea;
#[cfg(feature = "research")]
pub use research::ResearchArea;
pub use time::TimeArea;
