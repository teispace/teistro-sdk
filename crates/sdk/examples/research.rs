//! A two-group study, as `03-design/research.md` asks one to be run: the
//! predicates named, the labels and the test fixed, and the input hash
//! published before any data are read. `cargo xtask check-rust` runs this
//! file, and every binding's `research` prints these lines.
//!
//! ```sh
//! cargo run --release -p teistro --example research
//! ```

#![expect(clippy::print_stdout, reason = "an example is a program that prints")]

use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::research::{Birth, Contrast, Design, GroupTest, Study};
use teistro::{ChartRequest, Context, Ephemeris, Error, RuleRequest, ShippedRules, UtcOffset};

fn main() -> Result<(), Error> {
    let sdk = Context::builder()
        .profile("nepali-default")
        .ephemeris([Ephemeris::Builtin])
        .build()?;

    // Forty-eight births at Kathmandu, two months and an hour apart, and
    // the first sixteen called the cases. The labels mean nothing, so a
    // study that reads them honestly finds nothing: that is what the
    // corrections are for.
    let place = Place::new(
        Latitude::try_new(27.7172)?,
        Longitude::try_new(85.324)?,
        Altitude::try_new(1400.0)?,
    );
    let offset = UtcOffset::try_from_seconds(20_700)?;
    let births: Vec<Birth> = (0..48_u32)
        .map(|i| {
            let instant = 2_444_240.5 + 61.37 * f64::from(i) + f64::from(i % 24) / 24.0;
            Birth::new(JulianDay::<Utc>::literal(instant), place, offset)
        })
        .collect();
    let rules = RuleRequest::shipped([ShippedRules::Yogas]).rule_set()?;
    let request = ChartRequest::at(place, offset);
    let study = Study::new(&births, &request, &rules);
    let design = Design::new((0..48).map(|i| u16::from(i < 16)).collect());

    let table = sdk.research().counts(study, &design)?;
    println!(
        "{} rules over {} births",
        table.value.rows.len(),
        births.len()
    );

    let test = GroupTest {
        alpha: Some(0.05),
        ..GroupTest::new(2026, 999, Contrast::CaseVsRest { case: 1 })
    };
    let tested = sdk.research().compare(study, &design, &test)?;
    // The registration: the births, the rules, the labels and the test.
    println!("registered as {}", tested.provenance.input_hash);
    let value = &tested.value;
    println!(
        "{} permutations, none can say less than p = {:.4}",
        value.permutations, value.resolution
    );

    let under = |method: fn(&teistro::research::UnderAlpha) -> bool| {
        value
            .rows
            .iter()
            .filter(|row| row.under_alpha.as_ref().is_some_and(method))
            .count()
    };
    println!(
        "under 0.05: {} raw, {} after max-T, {} after Holm",
        under(|u| u.raw),
        under(|u| u.max_t),
        under(|u| u.holm)
    );

    // The smallest raw p, the first such row on a tie, and what the family
    // makes of it.
    let Some(best) = value.rows.iter().reduce(|kept, row| {
        if row.p.value < kept.p.value {
            row
        } else {
            kept
        }
    }) else {
        return Ok(());
    };
    if let [rest, cases] = best.counts.as_slice() {
        println!(
            "{}: {} of {} cases, {} of {} others, p {:.3}, max-T {:.3}",
            best.predicate,
            cases.present,
            cases.present + cases.absent,
            rest.present,
            rest.present + rest.absent,
            best.p.value,
            best.adjusted.max_t
        );
    }
    if let Some(effect) = best.effect {
        let d = effect.risk_difference;
        println!(
            "difference {:.3} ({:.3} to {:.3})",
            d.estimate, d.low, d.high
        );
    }
    Ok(())
}
