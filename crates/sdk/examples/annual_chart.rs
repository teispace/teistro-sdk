//! The annual chart: the one instant every Tajika judgement is made from.
//!
//! A birth chart is cast for a birth. An **annual** chart is cast for the
//! moment the Sun comes back to the longitude it held then — once a year,
//! about twenty minutes earlier than the clock would say, and never on the
//! birthday itself (`docs/03-design/annual-chart.md`).
//!
//! What it teaches:
//!
//! 1. **The instant, then the chart.** Whether the annual chart is cast
//!    for the birthplace or for where you live now is a question the
//!    schools answer differently, so the SDK answers the instant and casts
//!    the year's chart only where you name a place — `AnnualPlace::Birth`
//!    or a residence — in the one call every binding's `varsha` makes.
//! 2. **Which longitude is a choice with a name.** `SIDEREAL` is the
//!    tradition's; `TROPICAL` is the Western solar return and is most of a
//!    circle of lagna away by the fortieth year; `MEAN` is the older
//!    arithmetic and needs no ephemeris at all. None of them is a fallback
//!    for another.
//! 3. **Fewer than you asked for is the answer**, not a refusal: an
//!    ephemeris that ends before your hundredth year says so by giving you
//!    the years it has.
//!
//! The record is `birth_chart`'s own city, so the two can be read side by
//! side.
//!
//! ```sh
//! cargo run --release -p teistro --example annual_chart
//! ```

#![expect(clippy::print_stdout, reason = "an example is a program that prints")]

use teistro::catalogue::{Calendar, DashaSystem, Graha, Rashi};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::tajika::Reading;
use teistro::{
    AnnualChart, AnnualPlace, Asked, CalendarDate, ChartRequest, CivilDateTime, CivilTime, Context,
    Document, Ephemeris, Error, House, Resolved, Saham, UtcOffset, VarshaRequest, VarshaYear,
    ZoneSpec,
};

/// A birth, with what every request of it needs.
struct Birth<'a> {
    sdk: &'a Context,
    chart: Document,
    /// The clock the birth was cast under, which `AnnualPlace::Birth`
    /// casts each year's chart under too.
    clock: UtcOffset,
    at_home: ChartRequest,
}

impl Birth<'_> {
    /// The years `asked` opens.
    fn years(&self, asked: &VarshaRequest) -> Result<Vec<VarshaYear>, Error> {
        Ok(self
            .sdk
            .chart()
            .varsha(&self.chart, self.clock, asked)?
            .years)
    }

    /// The thirtieth year, its chart cast at the birthplace, under a
    /// request `asked` completes.
    fn thirtieth(
        &self,
        asked: impl FnOnce(VarshaRequest) -> VarshaRequest,
    ) -> Result<VarshaYear, Error> {
        self.years(&asked(VarshaRequest::through(30).at(AnnualPlace::Birth)))?
            .pop()
            .ok_or_else(|| Error::internal("thirty years asked for"))
    }
}

/// A year's own chart, which a request naming a place always has.
fn chart_of(year: &VarshaYear) -> Result<&AnnualChart, Error> {
    year.annual
        .as_ref()
        .ok_or_else(|| Error::internal("a place asks for each year's chart"))
}

/// Keys joined, or `none` for none, as every binding prints them.
fn listed<'k>(keys: impl Iterator<Item = &'k str>) -> String {
    let joined = keys.collect::<Vec<_>>().join(", ");
    if joined.is_empty() {
        String::from("none")
    } else {
        joined
    }
}

/// The birth record resolved to an instant, with the clock kept there.
fn when(sdk: &Context) -> Result<Resolved, Error> {
    let born = CalendarDate::defined(Calendar::Gregorian, 1990, 4, 14);
    sdk.time().resolve(
        &CivilDateTime::at(born, CivilTime::new(5, 30, 0)?),
        &ZoneSpec::Iana {
            zone: String::from("Asia/Kathmandu"),
        },
    )
}

/// The years a birth opens, and the chart of one of them cast where you
/// choose; answers the forty for the readings to be measured against.
fn the_years(birth: &Birth<'_>) -> Result<Vec<VarshaYear>, Error> {
    let years = birth.years(&VarshaRequest::through(40))?;
    println!("returns computed: {}", years.len());
    let thirtieth = years
        .iter()
        .find(|one| one.pravesha.year == 30)
        .ok_or_else(|| Error::internal("the built-in reaches a 1990 birth's thirtieth year"))?;
    println!(
        "the thirtieth year opens at jd {:.6}",
        thirtieth.pravesha.at.get()
    );

    // A return is about a sidereal year after the last, never a calendar one.
    let gaps: Vec<f64> = years
        .iter()
        .zip(years.iter().skip(1))
        .map(|(one, next)| next.pravesha.at.get() - one.pravesha.at.get())
        .collect();
    let shortest = gaps.iter().copied().fold(f64::INFINITY, f64::min);
    let longest = gaps.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    println!("between returns: {shortest:.4} to {longest:.4} days");

    // ── The chart of that year, cast where you choose ──────────────────
    let annual = birth
        .sdk
        .chart()
        .reading(thirtieth.pravesha.at, &birth.at_home)?
        .value;
    println!(
        "natal lagna {:.3}°, annual lagna {:.3}°",
        birth.chart.foundation.lagna_deg, annual.foundation.lagna_deg
    );
    Ok(years)
}

/// The year's own chart and its five office-bearers, cast where you say:
/// here the birthplace, the one Tajika text read casts every chart for.
fn the_office_bearers(birth: &Birth<'_>) -> Result<(), Error> {
    let cast = birth.thirtieth(|asked| asked)?;
    let chart = chart_of(&cast)?;
    let b = &chart.bearers;
    let five = [
        b.muntha,
        b.janma_lagna,
        b.varsha_lagna,
        b.tri_rashi,
        b.dina_ratri,
    ]
    .map(Graha::key)
    .join(" ");
    println!(
        "muntha in {}; office-bearers {five}, {}",
        cast.muntha.sign.key(),
        if b.by_day { "by day" } else { "by night" }
    );

    // And the lord of that year, with the reason it holds it: the answer
    // carries every claim it was chosen over.
    let lord = &chart.year_lord;
    println!(
        "year lord {} at {}, chosen {}",
        lord.graha.key(),
        lord.vishwa,
        lord.chosen.key()
    );
    for claim in &lord.claims {
        println!(
            "  {:<8} {}  {} portfolio(s)  {} the lagna",
            claim.graha.key(),
            claim.vishwa,
            claim.portfolios,
            if claim.aspects_lagna {
                "aspects"
            } else {
                "does not aspect"
            }
        );
    }
    Ok(())
}

/// The sixteen Tajika yogas answer a matter, not a chart: fourteen of them
/// judge the lagnesha against the lord of the house you ask about, so you
/// name the houses — marriage (7) and career (10) here.
fn the_matters(birth: &Birth<'_>) -> Result<(), Error> {
    let houses = vec![House::try_new(7)?, House::try_new(10)?];
    let judged = birth.thirtieth(|asked| asked.with_matters(Asked::These(houses)))?;
    for matter in &chart_of(&judged)?.matters {
        println!(
            "house {}: {} with {}, held {}; not answered {}",
            matter.house.get(),
            matter.lagnesha.key(),
            matter.karyesha.key(),
            listed(matter.held.iter().map(|one| one.yoga.key())),
            matter
                .unanswered
                .iter()
                .map(|yoga| yoga.key())
                .collect::<Vec<_>>()
                .join(", "),
        );
    }
    Ok(())
}

/// The sahams: forty-one sensitive points, each a − b + c. Name the ones
/// you want, or `Asked::All`; each comes back with its sign, that sign's
/// lord and the house it fell in, as the source reads them.
fn the_sahams(birth: &Birth<'_>) -> Result<(), Error> {
    let asked = vec![Saham::Punya, Saham::Vivaha, Saham::KaryaSiddhi];
    let year = birth.thirtieth(|request| request.with_sahams(Asked::These(asked)))?;
    let chart = chart_of(&year)?;
    for one in &chart.sahams {
        let at = &one.place;
        println!(
            "{:<12} {:>6.2}°  {}, lord {}, house {}{}",
            one.saham.key(),
            at.longitude_deg,
            at.sign.key(),
            at.lord.key(),
            at.house.get(),
            if at.added_sign { " (a sign added)" } else { "" }
        );
        // Its strength is the source's clauses, reported and never scored.
        let strong = one.strong().into_iter().filter(|(_, holds)| *holds);
        let weak = one.weak().into_iter().filter(|(_, holds)| *holds);
        println!(
            "  strong: {}; weak: {}",
            listed(strong.map(|(clause, _)| clause.key())),
            listed(weak.map(|(clause, _)| clause.key())),
        );
    }
    // And the seven's Harsha bala that year: four places each is happy in.
    let harsha: Vec<String> = chart
        .harsha
        .iter()
        .map(|one| format!("{} {}", one.graha.key(), one.total.units()))
        .collect();
    println!("{}", harsha.join(", "));
    Ok(())
}

/// The annual dashas: the year divided among its lords. The Mudda runs
/// round the nine from the birth nakshatra's lord, one lord further each
/// year; the Patyayini is read from the year's own chart, and its lagna's
/// share is a sign's. The Sun is read over the year once for both, and
/// each year closes on the next return.
fn the_dashas(birth: &Birth<'_>) -> Result<(), Error> {
    let asked = vec![DashaSystem::Mudda, DashaSystem::Patyayini];
    let year = birth.thirtieth(|request| request.with_dashas(Asked::These(asked)))?;
    for dasha in &chart_of(&year)?.dashas {
        let mahas: Vec<String> = dasha
            .periods
            .iter()
            .filter(|period| period.level() == 1)
            .map(|period| {
                let whose = period.sign.map_or_else(|| period.lord.key(), Rashi::key);
                format!("{whose} {:.1}", period.interval.days())
            })
            .collect();
        println!("{}: {}", dasha.system.key(), mahas.join(", "));
    }
    Ok(())
}

/// The readings are named, and they are not each other.
fn the_readings(birth: &Birth<'_>, years: &[VarshaYear]) -> Result<(), Error> {
    for reading in Reading::ALL {
        let one = birth.years(&VarshaRequest {
            reading,
            ..VarshaRequest::through(30)
        })?;
        let (Some(this), Some(sidereal)) = (one.get(29), years.get(29)) else {
            return Err(Error::internal("thirty years asked for"));
        };
        let apart = (this.pravesha.at.get() - sidereal.pravesha.at.get()) * 24.0;
        println!(
            "{:<9} thirtieth year, {apart:.2} hours from the sidereal one",
            reading.key()
        );
    }
    Ok(())
}

fn main() -> Result<(), Error> {
    let sdk = Context::builder()
        .profile("nepali-default")
        .ephemeris([Ephemeris::Builtin])
        .build()?;
    let place = Place::new(
        Latitude::try_new(27.7172)?,
        Longitude::try_new(85.324)?,
        Altitude::try_new(1400.0)?,
    );
    let resolved = when(&sdk)?;
    let at_home = ChartRequest::at(place, resolved.zone.offset);
    let birth = Birth {
        sdk: &sdk,
        chart: sdk.chart().reading(resolved.instant, &at_home)?.value,
        clock: resolved.zone.offset,
        at_home,
    };

    let years = the_years(&birth)?;
    the_office_bearers(&birth)?;
    the_matters(&birth)?;
    the_sahams(&birth)?;
    the_dashas(&birth)?;
    the_readings(&birth, &years)?;

    // ── What it refuses, and by which field ────────────────────────────
    match birth.years(&VarshaRequest::through(0)) {
        Ok(_) => Err(Error::internal("year 0 opens no return")),
        Err(refusal) => {
            println!(
                "refused  {}: {}",
                refusal.field().unwrap_or_default(),
                refusal.message
            );
            Ok(())
        }
    }
}
