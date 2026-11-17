//! The season's blackouts over a range, as a binding asks them (SR-138,
//! `03-design/panchanga-reach.md` §2): the stretches the Sun and the Moon
//! close to a rite, by the context's profile, in every build.

use serde::{Deserialize, Serialize};
use teistro_calendar::CalendarDate;
use teistro_chart::zodiac::ChartZodiac;
use teistro_core::catalogue::{BlackoutKind, Calendar, Kind, write_in_full};
use teistro_core::envelope::{Convention, Envelope, content_hash};
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::time::UtcOffset;
use teistro_panchanga::limb::geocentric;
use teistro_panchanga::season::{Blackout, SKY_KINDS, sky_blackouts};
use teistro_time::ZoneSpec;
use teistro_time::local_day::local_midnight;

use crate::area::system_of;
use crate::asked::DayAsked;
use crate::ephemeris::no_ephemeris;
use crate::{AlmanacArea, DayClock};

/// The record's name where a binding sends it, which a refusal is named
/// under.
const BLACKOUTS: &str = "blackouts";

/// A range's blackouts, asked.
///
/// ```
/// use teistro::BlackoutsRequest;
/// use teistro::catalogue::BlackoutKind;
///
/// let asked = BlackoutsRequest::from_json(
///     r#"{"first": {"year": 2026, "month": 1, "day": 1}, "last": {"year": 2026, "month": 12, "day": 31},
///         "utcOffsetSeconds": 20700, "kinds": ["CHATURMAS", "KHARMAS"]}"#,
/// )?;
/// assert_eq!(asked.kinds, [BlackoutKind::Chaturmas, BlackoutKind::Kharmas]);
/// // Asta is the muhurta search's, by the criterion it names.
/// let asta = BlackoutsRequest::from_json(
///     r#"{"first": {"year": 2026, "month": 1, "day": 1}, "utcOffsetSeconds": 20700, "kinds": ["GURU_ASTA"]}"#,
/// )
/// .unwrap_err();
/// assert_eq!(asta.field(), Some("blackouts.kinds"));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct BlackoutsRequest {
    /// The range's first day.
    pub first: CalendarDate,
    /// Its last day, the first again for one day.
    pub last: CalendarDate,
    /// The clock the days' midnights are read on.
    pub clock: DayClock,
    /// The kinds wanted, each one the Sun and the Moon make; every such
    /// kind when empty.
    pub kinds: Vec<BlackoutKind>,
}

/// The request as a binding writes it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "BlackoutsRequest"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct BlackoutsAsked {
    /// The range's first day.
    first: DayAsked,
    /// The last day; the first when left out.
    #[serde(default)]
    last: Option<DayAsked>,
    /// The calendar the days are written in; Gregorian when left out.
    #[serde(default)]
    calendar: Option<Calendar>,
    /// The offset from UTC the days' midnights are read at, in seconds,
    /// east positive; or `zone`, not both.
    #[serde(default)]
    utc_offset_seconds: Option<i32>,
    /// The zone the days are read on, at its offset on the first day; or
    /// `utcOffsetSeconds`, not both.
    #[serde(default)]
    zone: Option<ZoneSpec>,
    /// The kinds wanted, each one the Sun and the Moon make: `CHATURMAS`,
    /// `ADHIKA_MASA`, `SAMSARPA`, `KSHAYA_MASA`, `KHARMAS`,
    /// `PITRU_PAKSHA`, `SANKRANTI` or `HOLASHTAKA`; every one when left
    /// out.
    #[serde(default)]
    kinds: Vec<BlackoutKind>,
}

impl BlackoutsRequest {
    /// The record a binding sends, as JSON: `first` and optional `last` as
    /// `{year, month, day}` in `calendar` (Gregorian when left out);
    /// `utcOffsetSeconds` or `zone`; and `kinds`, every kind the Sun and
    /// the Moon make when left out.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, an offset out of range, both or neither of the clock's
    /// fields, or a kind the Sun and the Moon do not make (asta and the
    /// eclipse's star are the muhurta search's; an eclipse's vedha is on
    /// `almanac.eclipses`), each named under `blackouts`.
    pub fn from_json(text: &str) -> Result<BlackoutsRequest, Error> {
        let asked: BlackoutsAsked = teistro_core::strict::read(text, BLACKOUTS)?;
        asked.request().map_err(|why| why.under(BLACKOUTS))
    }
}

impl BlackoutsAsked {
    fn request(self) -> Result<BlackoutsRequest, Error> {
        if let Some(kind) = self.kinds.iter().find(|kind| !SKY_KINDS.contains(kind)) {
            return Err(Error::invalid_arg(format!(
                "`{kind}` is not a blackout the Sun and the Moon make"
            ))
            .with_field("kinds")
            .with_hint(
                "asta and the eclipse's star are the muhurta search's season (almanac.elect); an eclipse's vedha is on almanac.eclipses",
            ));
        }
        let calendar = self.calendar.unwrap_or(Calendar::Gregorian);
        Ok(BlackoutsRequest {
            first: self.first.in_calendar(calendar),
            last: self.last.unwrap_or(self.first).in_calendar(calendar),
            clock: DayClock::asked(self.utc_offset_seconds, self.zone)?,
            kinds: self.kinds,
        })
    }
}

/// What a range's blackouts are a function of.
#[derive(Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
struct BlackoutsInput {
    first: String,
    last: String,
    utc_offset_seconds: i32,
    kinds: Vec<BlackoutKind>,
}

impl AlmanacArea<'_> {
    /// **The blackouts the Sun and the Moon make over a range of civil
    /// days** (SR-138, `03-design/panchanga-reach.md` §2): Chaturmas, an
    /// adhika month (Nepal's malmas), a samsarpa, a kshaya month, Kharmas,
    /// Pitru paksha, the sixteen ghatis about each sankranti and
    /// Holashtaka, each the instants it holds over, clipped to the range
    /// from the first day's local midnight to the midnight after the last,
    /// in order of their start.
    ///
    /// The rule is the muhurta search's own
    /// ([`teistro_panchanga::season::blackouts`]), read in the profile's
    /// chart zodiac taken at the range's middle, geocentric, as that
    /// search reads it: so it is in every build, and a ceremony page and an
    /// election agree. `kinds` keeps those asked; every kind when empty.
    ///
    /// ```no_run
    /// use teistro::{CalendarDate, Context, UtcOffset};
    /// use teistro::catalogue::{BlackoutKind, Calendar};
    ///
    /// let sdk = Context::builder().profile("nepali-drik").build()?;
    /// let year = sdk.almanac().blackouts(
    ///     &CalendarDate::defined(Calendar::Gregorian, 2026, 1, 1),
    ///     &CalendarDate::defined(Calendar::Gregorian, 2026, 12, 31),
    ///     UtcOffset::literal(5, 45, 0),
    ///     &[BlackoutKind::AdhikaMasa],
    /// )?;
    /// for malmas in &year.value {
    ///     println!("malmas from {} to {}", malmas.at.from, malmas.at.to);
    /// }
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A context with no ephemeris, a calendar the SDK does not ship, a
    /// range the wrong way round, a kind the Sun and the Moon do not make
    /// (`INVALID_ARG` on `kinds`), and an instant the provider refuses.
    pub fn blackouts(
        self,
        first: &CalendarDate,
        last: &CalendarDate,
        offset: UtcOffset,
        kinds: &[BlackoutKind],
    ) -> Result<Envelope<Vec<Blackout>>, Error> {
        if let Some(kind) = kinds.iter().find(|kind| !SKY_KINDS.contains(kind)) {
            return Err(Error::invalid_arg(format!(
                "`{kind}` is not a blackout the Sun and the Moon make"
            ))
            .with_field("kinds"));
        }
        let provider = self.context().ephemeris().ok_or_else(no_ephemeris)?;
        let calendar = system_of(first.calendar)?;
        let clock = self.clock_or(&offset);
        let range = Interval::new(
            local_midnight(clock, calendar.fixed_of(first)?)?,
            local_midnight(clock, calendar.fixed_of(last)?.plus_days(1))?,
        )?;
        let settings = self.context().settings();
        let models = (self.context().precession()?, self.context().delta_t());
        let completion = self.context().completion_over(provider);
        // One chart zodiac for the range, at its middle, as the muhurta
        // search takes it.
        let middle = JulianDay::<Utc>::try_new(f64::midpoint(range.from.get(), range.to.get()))?;
        let (tt, _) = teistro_astro::scale::tt_of(middle.relabel(), models.1)?;
        let searched = ChartZodiac::searched(&completion, settings, tt, models)?;
        let found: Vec<Blackout> =
            sky_blackouts(&completion, searched.frame, searched.zodiac, range)?
                .into_iter()
                .filter(|b| kinds.is_empty() || kinds.contains(&b.kind))
                .collect();
        let mut provenance = self.context().stamped(
            content_hash(&BlackoutsInput {
                first: first.to_string(),
                last: last.to_string(),
                utc_offset_seconds: offset.seconds(),
                kinds: kinds.to_vec(),
            }),
            geocentric(searched.frame),
            Vec::new(),
        );
        provenance.applied_conventions.extend([
            Convention {
                knob: String::from("blackouts.window"),
                value: format!("JD {} to {}", range.from.get(), range.to.get()),
                reason: String::from(
                    "a range of civil days runs from the first day's local midnight to the midnight after the last, and each blackout is clipped to it",
                ),
            },
            Convention {
                knob: String::from("blackouts.zodiacAt"),
                value: middle.get().to_string(),
                reason: String::from(
                    "one chart zodiac serves the range, taken at its middle and read geocentric, as the muhurta search's season reads it",
                ),
            },
        ]);
        Ok(Envelope::sealing(found, provenance))
    }

    /// As [`AlmanacArea::blackouts`] for a request a binding sent: on its
    /// offset, or on its zone at the first day's noon, each kind written
    /// as its full key.
    ///
    /// ```no_run
    /// use teistro::{BlackoutsRequest, Context};
    ///
    /// let sdk = Context::builder().build()?;
    /// let asked = BlackoutsRequest::from_json(
    ///     r#"{"first": {"year": 2026, "month": 12, "day": 1}, "last": {"year": 2026, "month": 12, "day": 31},
    ///         "utcOffsetSeconds": 19800, "kinds": ["KHARMAS"]}"#,
    /// )?;
    /// let written = sdk.almanac().blackouts_request(&asked)?;
    /// assert_eq!(written.value[0]["kind"], "blackout_kind.KHARMAS");
    /// # Ok::<(), teistro::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`AlmanacArea::blackouts`]; a zone the database does not have.
    pub fn blackouts_request(
        self,
        asked: &BlackoutsRequest,
    ) -> Result<Envelope<serde_json::Value>, Error> {
        let Envelope {
            value,
            mut provenance,
        } = self.clocked_from(&asked.clock, &asked.first, |area, offset| {
            area.blackouts(&asked.first, &asked.last, offset, &asked.kinds)
        })?;
        asked.clock.seal_answer(&mut provenance);
        let mut written = serde_json::to_value(&value).map_err(|error| {
            Error::internal(format!("the blackouts did not serialise: {error}"))
        })?;
        write_in_full(&mut written, "kind", Kind::BlackoutKind);
        Ok(Envelope::sealing(written, provenance))
    }
}
