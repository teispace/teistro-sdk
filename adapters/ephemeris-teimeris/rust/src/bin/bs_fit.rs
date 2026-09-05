//! The source memo's classical-against-modern comparison from the SDK's
//! own code (`docs/calendars/bikram-sambat.md`): the Bikram Sambat engine
//! run over the official span with the drik Sun through this adapter
//! (Teimeris positions, its Lahiri ayanamsha, the SDK's rise and set
//! solver), under every named month-start rule, printed as the fit table
//! beside the shipped classical frame.

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "a tooling binary"
)]

use std::process::ExitCode;

use teistro_astro::DeltaTModel;
use teistro_calendar::BikramSambat;
use teistro_calendar::bikram_sambat::{Engine, FitReport, KATHMANDU, fit};
use teistro_calendar::solar::{DrikSun, MonthStartRule, SolarModel};
use teistro_core::catalogue::Ayanamsha;
use teistro_core::settings::{OverridePolicy, Sunrise};
use teistro_core::time::LocalClock;
use teistro_ephemeris_teimeris::{TeimerisProvider, data_dir_from_env};
use teistro_siddhanta::SuryaSiddhanta;
use teistro_time::zones;

fn main() -> ExitCode {
    let data_dir = data_dir_from_env();
    let provider = match TeimerisProvider::open(&data_dir) {
        Ok(provider) => provider,
        Err(error) => {
            eprintln!("cannot open Teimeris over {}: {error}", data_dir.display());
            return ExitCode::FAILURE;
        }
    };
    let calendar = BikramSambat::shipped();
    let official = calendar.official_rows();
    let (first_year, last_year) = calendar.official_years();
    let Some(first_start) = calendar.year_start(first_year) else {
        eprintln!("the shipped table has no official span");
        return ExitCode::FAILURE;
    };
    let clock = zones::nepal();
    let classical = SuryaSiddhanta::text();
    let drik = DrikSun::new(
        &provider,
        Ayanamsha::Lahiri,
        Sunrise::CentreNoRefraction.into(),
        OverridePolicy::PreferNative,
        DeltaTModel::TableThenModel,
    );
    let models: [(&str, &dyn SolarModel); 2] =
        [("Surya Siddhanta, the text", &classical), ("drik", &drik)];
    println!(
        "official span BS {first_year} to {last_year} ({} years); place {KATHMANDU}; clock {}",
        official.len(),
        clock.describe()
    );
    println!();
    println!("{}", FitReport::MARKDOWN_HEADER);
    for (label, model) in models {
        let probe = Engine::new(model, clock, KATHMANDU, MonthStartRule::SankrantiDay);
        let instants: Vec<(
            i32,
            [teistro_core::quantity::JulianDay<teistro_core::quantity::Utc>; 13],
        )> = match (first_year..=last_year)
            .map(|year| probe.sankrantis(year).map(|found| (year, found)))
            .collect()
        {
            Ok(instants) => instants,
            Err(error) => {
                eprintln!("{label}: {error}");
                return ExitCode::FAILURE;
            }
        };
        for rule in MonthStartRule::NAMED {
            let engine = Engine::new(model, clock, KATHMANDU, rule);
            let rows: Result<Vec<_>, _> = instants
                .iter()
                .map(|(year, found)| engine.year_from(*year, found))
                .collect();
            match rows {
                Ok(rows) => {
                    let report = fit(&format!("{label}; {rule}"), &rows, &official, first_start);
                    println!("{}", report.markdown_row());
                }
                Err(error) => println!("| {label}; {rule} | refused: {error} | | | | |"),
            }
        }
    }
    println!();
    println!("drik model: {}", SolarModel::describe(&drik));
    ExitCode::SUCCESS
}
