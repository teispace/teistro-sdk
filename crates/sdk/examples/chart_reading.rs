//! A chart reading: one call for the whole document a reader interprets.
//!
//! `birth_chart.rs` placed the grahas. A reading is what comes after: the
//! divisional charts, the houses, what each graha **is** rather than where
//! it is, which grahas look at which, and the derived points. Each is a
//! section a request asks for by name, and each is computed from the same
//! founded chart in the same call — so asking for all of them costs one
//! founding, and asking for none of them costs nothing
//! (`docs/03-design/chart-reading.md`).
//!
//! What it teaches:
//!
//! 1. **Sections are asked for.** `with_vargas`, `with_aspects`,
//!    `with_points`, `with_houses` and `with_state` are off by default, so
//!    a birth chart does not pay for twenty-one divisional charts it will
//!    not show.
//! 2. **Vargottama is a comparison, not a flag**: a graha whose navamsha
//!    sign is the sign it stands in. The document gives both signs.
//! 3. **A dignity and a house are different questions**, answered by
//!    different sections: `state` says how a graha fares in its sign,
//!    `houses` who rules a house.
//! 4. **The drishti are ragged**: how many there are depends on where the
//!    grahas stand, not on how many grahas there are.
//! 5. **A drawing is geometry, not pixels**: each cell's outline in a unit
//!    square, the sign and house it shows and the grahas in it, so any
//!    renderer draws the same chart.
//!
//! The record is `birth_chart.rs`'s own, and the output is the other three
//! bindings' `chart_reading` example's, line for line.
//!
//! ```sh
//! cargo run --release -p teistro --example chart_reading
//! ```

#![expect(clippy::print_stdout, reason = "an example is a program that prints")]

use teistro::catalogue::{Calendar, ChartLayout, Graha, Point, Rashi, Varga};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{
    CalendarDate, ChartRequest, CivilDateTime, CivilTime, Context, Document, Ephemeris, Error,
    VargaChart, ZoneSpec,
};

/// A catalogue member's name in the context's locale.
fn name(sdk: &Context, key: &str) -> Result<String, Error> {
    Ok(sdk.intl().entity(key)?.name().to_owned())
}

/// A document section the request asked for, or the refusal to pretend.
fn asked<T>(section: Option<T>, what: &str) -> Result<T, Error> {
    section.ok_or_else(|| {
        Error::internal(format!(
            "the reading has no {what}, though it was asked for"
        ))
    })
}

fn main() -> Result<(), Error> {
    let sdk = Context::builder()
        .profile("nepali-default")
        .locale("ne-Deva-NP")
        .ephemeris([Ephemeris::Builtin])
        .build()?;

    let born = CalendarDate::defined(Calendar::BikramSambat, 2042, 9, 17);
    let resolved = sdk.time().resolve(
        &CivilDateTime::at(born, CivilTime::new(0, 20, 0)?),
        &ZoneSpec::Iana {
            zone: String::from("Asia/Kathmandu"),
        },
    )?;

    // ── One call for every section ─────────────────────────────────────
    let place = Place::new(
        Latitude::try_new(27.7172)?,
        Longitude::try_new(85.324)?,
        Altitude::try_new(1400.0)?,
    );
    let request = ChartRequest::at(place, resolved.zone.offset)
        .with_vargas([Varga::D9, Varga::D10])
        .with_aspects()
        .with_points()
        .with_houses()
        .with_state()
        .with_drawings([(ChartLayout::NorthIndian, Varga::D9)]);
    let document = sdk.chart().reading(resolved.instant, &request)?.value;
    let [navamsha, dasamsha] = document.vargas.as_slice() else {
        return Err(Error::internal("two divisional charts were asked for"));
    };

    println!("reading  BS 2042-09-17  00:20  Kathmandu");
    lagna(&sdk, &document, navamsha, dasamsha)?;
    grahas(&sdk, &document, navamsha)?;
    houses(&sdk, &document)?;
    drishti(&sdk, &document)?;
    points(&sdk, &document)?;
    drawn(&sdk, &document)?;
    println!(
        "settings hash  {}…",
        sdk.settings_hash()
            .to_string()
            .get(..16)
            .unwrap_or_default()
    );
    Ok(())
}

/// The lagna's sign, and its sign in each divisional chart.
fn lagna(
    sdk: &Context,
    document: &Document,
    navamsha: &VargaChart,
    dasamsha: &VargaChart,
) -> Result<(), Error> {
    let lagna_deg = document.foundation.lagna_deg;
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a normalised longitude over thirty is 0 to 11"
    )]
    let lagna = Rashi::from_id((lagna_deg / 30.0).floor() as u16)
        .ok_or_else(|| Error::internal("a normalised lagna has a sign"))?;
    println!(
        "lagna    {} {:.4}°   D9 {}   D10 {}",
        name(sdk, lagna.full_key())?,
        lagna_deg % 30.0,
        name(sdk, navamsha.lagna.sign.full_key())?,
        name(sdk, dasamsha.lagna.sign.full_key())?,
    );
    Ok(())
}

/// What each graha is: its house, dignity, age, navamsha sign and burning.
fn grahas(sdk: &Context, document: &Document, navamsha: &VargaChart) -> Result<(), Error> {
    println!();
    println!(
        "graha        house  dignity          age            D9 sign      vargottama  combust"
    );
    println!("{}", "─".repeat(88));
    for (state, in_navamsha) in asked(document.state.as_ref(), "state")?
        .iter()
        .zip(&navamsha.grahas)
    {
        let at = in_navamsha.at;
        println!(
            "{:<12} {:>5}  {:<16} {:<14} {:<12} {:<11} {}",
            name(sdk, state.graha.full_key())?,
            state.house,
            name(sdk, state.dignity.full_key())?,
            name(sdk, state.age.full_key())?,
            name(sdk, at.sign.full_key())?,
            if at.sign == at.rashi { "yes" } else { "no" },
            state.combustion.burning.key(),
        );
    }
    Ok(())
}

/// The tenth house, by the houses service: the sign its middle falls in,
/// that sign's lord, and which kind of house it is.
fn houses(sdk: &Context, document: &Document) -> Result<(), Error> {
    let tenth = asked(
        asked(document.houses.as_ref(), "houses")?.bhava(10),
        "tenth bhava",
    )?;
    println!();
    println!(
        "bhava 10 {}, ruled by {} ({})",
        name(sdk, tenth.sign.full_key())?,
        name(sdk, tenth.lord.full_key())?,
        tenth.quadrant.key(),
    );
    Ok(())
}

/// Which grahas look at the Moon. `houses` counts inclusively from the
/// looking graha's sign, so the seventh is the house opposite it.
fn drishti(sdk: &Context, document: &Document) -> Result<(), Error> {
    let aspects = asked(document.aspects.as_ref(), "aspects")?;
    println!(
        "drishti  {} under {}; on the Moon:",
        aspects.all().len(),
        aspects.table()
    );
    for drishti in aspects.all().iter().filter(|one| one.to == Graha::Moon) {
        println!(
            "         {:<12} house {:>2} from it  {}",
            name(sdk, drishti.from.full_key())?,
            drishti.houses,
            drishti.strength.key(),
        );
    }
    Ok(())
}

/// The derived points. Gulika is Saturn's portion of the day's arc, which
/// is why a chart with no day to divide has none — the section is ragged
/// for that.
fn points(sdk: &Context, document: &Document) -> Result<(), Error> {
    let points = asked(document.points.as_ref(), "points")?.all();
    let gulika = points.iter().find(|found| found.point == Point::Gulika);
    let at = match gulika {
        Some(found) => format!(
            "{} {:.4}°",
            name(sdk, found.sign.full_key())?,
            found.longitude_deg % 30.0
        ),
        None => String::from("none"),
    };
    println!("gulika   {at}   {} points", points.len());
    Ok(())
}

/// The navamsha, drawn. A North Indian chart keeps its houses still and
/// moves the signs, so the lagna is always the top diamond; the cell says
/// which sign landed there.
fn drawn(sdk: &Context, document: &Document) -> Result<(), Error> {
    let drawing = asked(document.drawings.first(), "drawing")?;
    let placed = &drawing.placed;
    let risen = asked(placed.cells.iter().find(|cell| cell.lagna), "lagna cell")?;
    println!(
        "drawing  {} {}: {} cells, lagna in house {} ({}), grahas there: {}",
        placed.layout.to_lowercase(),
        drawing.varga.key().to_lowercase(),
        placed.cells.len(),
        risen.house,
        name(sdk, risen.sign.full_key())?,
        risen.bodies.len(),
    );
    Ok(())
}
