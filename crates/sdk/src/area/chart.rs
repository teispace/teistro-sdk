//! `sdk.chart`: a chart founded from a birth record, and a batch of them
//! founded in one crossing.

use teistro_astro::precession::PrecessionModel;
use teistro_calendar::solar::drik::DrikSun;
use teistro_chart::foundation::{ChartFoundation, Founder};
use teistro_core::catalogue::{Ayanamsha, ChartKind};
use teistro_core::envelope::{Envelope, content_hash};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_core::settings::AyanamshaChoice;
use teistro_core::time::UtcOffset;

use crate::area::system_of;
use crate::context::Context;
use crate::ephemeris::no_ephemeris;

/// `sdk.chart`: the foundation every reading is built on — the lagna,
/// the day's lagna, the ayanamsha applied, the day part, the grahas
/// placed.
///
/// **A batch is the shape, and one chart is the batch unwrapped.** A
/// rectification pass wants a hundred charts at one place, and founding
/// them together resolves the settings once, builds the solar model
/// once, and reckons the day each instant belongs to against the same
/// sunrise. That is the "batch in the signature, not bolted on" rule the
/// maintainer's brief asks for.
#[derive(Clone, Copy, Debug)]
pub struct ChartArea<'a> {
    context: &'a Context,
}

impl<'a> ChartArea<'a> {
    pub(crate) fn of(context: &'a Context) -> ChartArea<'a> {
        ChartArea { context }
    }

    /// The context this area was read off.
    #[must_use]
    pub fn context(self) -> &'a Context {
        self.context
    }

    /// Many charts at one place, founded in **one crossing**.
    ///
    /// The envelope's provenance is one stamp over the batch: the
    /// settings, the provider and the steps are the same for every chart
    /// in it, and the input hash is of the whole request.
    ///
    /// # Errors
    ///
    /// A context with no ephemeris, a civil calendar the SDK does not
    /// ship, or the first instant the founder refuses.
    pub fn found_many(
        self,
        instants: &[JulianDay<Utc>],
        place: &Place,
        offset: UtcOffset,
        kind: ChartKind,
    ) -> Result<Envelope<Vec<ChartFoundation>>, Error> {
        let provider = self.context.ephemeris().ok_or_else(no_ephemeris)?;
        let resolved = self.context.resolved();
        let settings = &resolved.settings;
        let calendar = system_of(settings.calendars.civil_calendar)?;
        // A custom ayanamsha is a value rather than a catalogue member,
        // and the solar model wants a member; Lahiri is what the
        // boundary substitutes, so this substitutes the same.
        let ayanamsha = match settings.frame.ayanamsha {
            AyanamshaChoice::Catalogued { id } => id,
            AyanamshaChoice::Custom { .. } => Ayanamsha::Lahiri,
        };
        let model = DrikSun::new(
            provider,
            ayanamsha,
            settings.day.sunrise,
            settings.provider.overrides,
            self.context.delta_t(),
        );
        let mut founded = Founder::new(
            provider,
            resolved,
            &model,
            calendar,
            &offset,
            PrecessionModel::default(),
            self.context.delta_t(),
        )
        .found(instants, place, kind)?;
        // The founder leaves the content hash a placeholder and the C
        // boundary fills it, so a Rust consumer's envelope carried one
        // where a binding's blob carried a hash. Filled here for the
        // same reason the boundary fills it, which also answers the
        // asymmetry `serial-and-the-envelope.md` §8 noticed.
        founded.provenance.content_hash = content_hash(&founded.value);
        Ok(founded)
    }

    /// One chart: the batch of one, unwrapped.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::found_many`].
    pub fn found(
        self,
        instant: JulianDay<Utc>,
        place: &Place,
        offset: UtcOffset,
        kind: ChartKind,
    ) -> Result<Envelope<ChartFoundation>, Error> {
        let many = self.found_many(&[instant], place, offset, kind)?;
        let Envelope { value, provenance } = many;
        let Some(one) = value.into_iter().next() else {
            return Err(Error::internal(
                "a batch of one instant founded no chart, which cannot happen",
            ));
        };
        Ok(Envelope::new(one, provenance))
    }
}
