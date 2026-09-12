//! A Bikram Sambat calendar page, and why the conversions are not
//! arithmetic.
//!
//! The Nepali calendar is not a formula. Its month lengths are decided
//! by where the Sun stands at the moment a month begins, so they vary
//! year to year — Baisakh is 30 or 31 or 32 days depending on the year —
//! and the authoritative table only covers BS 1970 to 2095. Outside that
//! span the SDK computes the months from the Surya Siddhanta as the text
//! prints it.
//!
//! Every date the SDK **returns** therefore says how it was decided:
//! `Tabular` from the official table, `Computed` from the engine, or
//! `Divergent` where the two disagree and the table wins. A date a
//! caller *states* is `Defined`: it is what was asked for. So the
//! resolution to read is always the one on the answer.
//!
//! In Rust that distinction is in the **type**: `CalendarResolution` is
//! an enum a `match` must cover, where the other three bindings hand
//! over a string.
//!
//! ```sh
//! cargo run --release -p teistro --example calendar
//! ```

#![expect(clippy::print_stdout, reason = "an example is a program that prints")]

use teistro::CalendarResolution;
use teistro::catalogue::Calendar;
use teistro::{CalendarDate, Context, Error, messages};

/// The days of the week, from the ISO numbering `weekday_of` answers in.
const WEEK: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

/// How a date says it was decided — a `match` rather than a string, so a
/// resolution the SDK gains is a compile error here rather than a
/// surprise at run time.
fn decided(how: &CalendarResolution) -> &'static str {
    match how {
        CalendarResolution::Defined => "defined",
        CalendarResolution::Tabular { .. } => "tabular",
        CalendarResolution::Computed { .. } => "computed",
        CalendarResolution::Divergent { .. } => "divergent",
    }
}

/// A date with its era and how it was decided.
fn described(day: &CalendarDate) -> String {
    let era = day.era.map_or_else(String::new, |era| {
        format!(" {} {}", era.era.key(), era.year)
    });
    format!(
        "{}-{:02}-{:02}{era} [{}]",
        day.year,
        day.month,
        day.day,
        decided(&day.resolution)
    )
}

/// One BS month as a calendar grid.
///
/// Every cell is a real date the SDK converted, not a number counted up:
/// a month that gains or loses a day at either end is then right by
/// construction.
fn month_page(sdk: &Context, year: i32, month: u8) -> Result<String, Error> {
    let length = sdk
        .calendar()
        .month_length(Calendar::BikramSambat, year, month)?;
    let first = CalendarDate::defined(Calendar::BikramSambat, year, month, 1);
    // ISO weekday 1..=7; a page starts on Monday, so the first of the
    // month sits at column `weekday - 1`.
    let lead = usize::from(sdk.calendar().weekday_of(&first)? as u8) - 1;

    let mut cells: Vec<String> = vec![String::new(); lead];
    cells.extend((1..=length).map(|day| day.to_string()));
    let mut lines = vec![
        WEEK.iter()
            .map(|name| format!("{name:>3}"))
            .collect::<Vec<_>>()
            .join("  "),
    ];
    for week in cells.chunks(7) {
        lines.push(
            week.iter()
                .map(|cell| format!("{cell:>3}"))
                .collect::<Vec<_>>()
                .join("  "),
        );
    }
    Ok(lines.join("\n"))
}

fn main() -> Result<(), Error> {
    // No ephemeris: a calendar needs none, and naming `Ephemeris::None`
    // says so rather than leaving a reader to wonder. Everything below
    // computes; only `positions`, `chart` and `almanac` would refuse.
    let sdk = Context::builder()
        .profile("nepali-default")
        .locale("ne-Deva-NP")
        .ephemeris([teistro::Ephemeris::None])
        .build()?;
    let year = 2082;
    let month_name = |month: i64| -> String {
        sdk.intl()
            .render_typed(&messages::sdk::calendar::bikram_sambat::MonthName { month })
            .text
    };

    // ── A whole year, with its Gregorian spans ─────────────────────────
    println!("BS {year}");
    let mut total = 0_u32;
    for month in 1..=12_u8 {
        let length = sdk
            .calendar()
            .month_length(Calendar::BikramSambat, year, month)?;
        total += u32::from(length);
        let opens = CalendarDate::defined(Calendar::BikramSambat, year, month, 1);
        let closes = CalendarDate::defined(Calendar::BikramSambat, year, month, length);
        let starts = sdk.calendar().convert(&opens, Calendar::Gregorian)?;
        let ends = sdk.calendar().convert(&closes, Calendar::Gregorian)?;
        println!(
            "  {month:>2}  {:<10} {length:>2} days   {}-{:02}-{:02} to {}-{:02}-{:02}",
            month_name(i64::from(month)),
            starts.year,
            starts.month,
            starts.day,
            ends.year,
            ends.month,
            ends.day,
        );
    }
    println!("      {:<10} {total} days in the year", "");
    // A BS year is 365 or 366 days like any solar year, but its months
    // are not: the shortest here is 29 days and the longest 32, which is
    // why a month length is asked for and never assumed.

    // ── One month as a page ────────────────────────────────────────────
    println!("\nBaisakh {year}");
    println!("{}", month_page(&sdk, year, 1)?);

    // ── The round trip, and what each date says about itself ───────────
    println!();
    let new_year = CalendarDate::defined(Calendar::BikramSambat, year, 1, 1);
    let gregorian = sdk.calendar().convert(&new_year, Calendar::Gregorian)?;
    let back = sdk.calendar().convert(&gregorian, Calendar::BikramSambat)?;
    println!("  BS   {}", described(&new_year));
    println!("  ->   {}", described(&gregorian));
    println!("  ->   {}", described(&back));
    let fixed = sdk.calendar().fixed_of(&new_year)?;
    println!(
        "  fixed day {}, weekday {}, Julian day {:.1}",
        fixed.get(),
        sdk.calendar().weekday_of(&new_year)? as u8,
        // A free function, not an area's operation: a fixed day and a
        // Julian day are two spellings of one integer, and no profile or
        // locale changes the arithmetic.
        teistro::jd_of_fixed(fixed),
    );

    // ── Inside the table, and outside it ───────────────────────────────
    println!();
    for asked in [2082, 2200, 1960] {
        let greg = sdk.calendar().convert(
            &CalendarDate::defined(Calendar::BikramSambat, asked, 1, 1),
            Calendar::Gregorian,
        )?;
        let answer = sdk.calendar().convert(&greg, Calendar::BikramSambat)?;
        println!(
            "  BS {asked} began {}-{:02}-{:02}, and the answer is [{}]",
            greg.year,
            greg.month,
            greg.day,
            decided(&answer.resolution),
        );
    }
    println!(
        "       the official table runs BS 1970 to 2095; on either side the SDK's\n\
         \x20      own engine answers, and says so"
    );

    // ── The typed message accessors ────────────────────────────────────
    // A date rendered for a reader goes through the locale, not through
    // string concatenation: the key is spelled once, in the generator,
    // and the parameters are a struct the compiler checks.
    println!();
    println!(
        "  rendered  {}",
        sdk.intl()
            .render_typed(&messages::sdk::calendar::bikram_sambat::date::Long {
                day: 1,
                month_name: month_name(1),
                year: i64::from(year),
            })
            .text
    );
    Ok(())
}
