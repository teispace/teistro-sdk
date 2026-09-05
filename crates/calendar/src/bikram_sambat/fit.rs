//! The measurement: how a computed span reproduces the official table,
//! month by month and year by year, with the running drift and the set of
//! divergences. The numbers a frame is chosen by, and the ones the source
//! memo publishes.

use core::fmt;

use crate::bikram_sambat::engine::YearRow;

/// A month whose computed length differs from the table's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Divergence {
    /// The Bikram Sambat year.
    pub year: i32,
    /// The month, 1 for Baisakh.
    pub month: u8,
    /// The table's length.
    pub tabular: u8,
    /// The computed length.
    pub computed: u8,
}

/// How a computed span reproduces a table.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct FitReport {
    /// The label of the frame measured.
    pub frame: String,
    /// Years compared.
    pub years: u32,
    /// Years whose twelve lengths all match.
    pub years_exact: u32,
    /// Years whose total matches.
    pub totals_matched: u32,
    /// Months compared.
    pub months: u32,
    /// Months whose length matches.
    pub months_matched: u32,
    /// The running difference of the year totals at the end of the span.
    pub drift_end: i32,
    /// The largest excursion of the running difference, signed.
    pub drift_max: i32,
    /// The largest difference between a computed 1 Baisakh and the
    /// table's, in days, signed.
    pub start_offset_max: i64,
    /// The months that differ.
    pub divergences: Vec<Divergence>,
}

impl FitReport {
    /// The share of months reproduced, in percent.
    #[must_use]
    pub fn month_agreement(&self) -> f64 {
        if self.months == 0 {
            0.0
        } else {
            f64::from(self.months_matched) * 100.0 / f64::from(self.months)
        }
    }

    /// Whether the running day count stays within a day, the condition
    /// for appending computed years to the table.
    #[must_use]
    pub const fn drift_within_a_day(&self) -> bool {
        self.drift_end.abs() <= 1 && self.drift_max.abs() <= 1
    }

    /// One row of a Markdown table: frame, months, years, drift, offset.
    #[must_use]
    pub fn markdown_row(&self) -> String {
        format!(
            "| {} | {}/{} ({:.1} %) | {}/{} | {}/{} | {} (max {}) | {} |",
            self.frame,
            self.months_matched,
            self.months,
            self.month_agreement(),
            self.years_exact,
            self.years,
            self.totals_matched,
            self.years,
            self.drift_end,
            self.drift_max,
            self.start_offset_max
        )
    }

    /// The header the rows go under.
    pub const MARKDOWN_HEADER: &'static str = "| frame | months | years exact | year totals | drift end (max) | start offset max |\n|---|---:|---:|---:|---:|---:|";
}

impl fmt::Display for FitReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.markdown_row())
    }
}

/// Measures computed rows against official rows keyed by year; years
/// present in both are compared, the table's first day anchoring the
/// official year starts.
#[must_use]
pub fn fit(
    frame: &str,
    computed: &[YearRow],
    official: &[(i32, [u8; 12])],
    official_first_start: crate::fixed::FixedDay,
) -> FitReport {
    let mut report = FitReport {
        frame: frame.to_string(),
        years: 0,
        years_exact: 0,
        totals_matched: 0,
        months: 0,
        months_matched: 0,
        drift_end: 0,
        drift_max: 0,
        start_offset_max: 0,
        divergences: Vec::new(),
    };
    let mut official_start = official_first_start;
    let mut drift = 0i32;
    for (year, lengths) in official {
        let total: i64 = lengths.iter().map(|d| i64::from(*d)).sum();
        if let Some(row) = computed.iter().find(|row| row.year == *year) {
            report.years += 1;
            let mut exact = true;
            for (month, (tabular, computed)) in lengths.iter().zip(row.months.iter()).enumerate() {
                report.months += 1;
                if tabular == computed {
                    report.months_matched += 1;
                } else {
                    exact = false;
                    report.divergences.push(Divergence {
                        year: *year,
                        month: u8::try_from(month + 1).unwrap_or(0),
                        tabular: *tabular,
                        computed: *computed,
                    });
                }
            }
            if exact {
                report.years_exact += 1;
            }
            let computed_total = i64::from(row.days());
            if computed_total == total {
                report.totals_matched += 1;
            }
            drift += i32::try_from(computed_total - total).unwrap_or(0);
            if drift.abs() > report.drift_max.abs() {
                report.drift_max = drift;
            }
            let offset = official_start.days_until(row.start);
            if offset.abs() > report.start_offset_max.abs() {
                report.start_offset_max = offset;
            }
        }
        official_start = official_start.plus_days(total);
    }
    report.drift_end = drift;
    report
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_core::quantity::JulianDay;

    use super::*;
    use crate::fixed::FixedDay;

    fn row(year: i32, start: FixedDay, months: [u8; 12]) -> YearRow {
        YearRow {
            year,
            start,
            months,
            sankrantis: [JulianDay::J2000; 12],
        }
    }

    #[test]
    fn a_fit_counts_months_years_drift_and_offsets() {
        let a = [31, 31, 32, 31, 31, 30, 30, 30, 29, 30, 29, 31];
        let mut b = a;
        b[1] = 32;
        b[2] = 31;
        let mut c = a;
        c[11] = 30;
        let start = FixedDay::new(1000);
        let official = [(2000, a), (2001, a), (2002, a)];
        let computed = [
            row(2000, start, a),
            row(2001, start.plus_days(365), b),
            row(2002, start.plus_days(731), c),
            row(2003, start.plus_days(1095), a),
        ];
        let report = fit("test", &computed, &official, start);
        assert_eq!(
            (report.years, report.years_exact, report.totals_matched),
            (3, 1, 2)
        );
        assert_eq!((report.months, report.months_matched), (36, 33));
        assert_eq!(report.divergences.len(), 3);
        assert_eq!(
            *report.divergences.first().unwrap(),
            Divergence {
                year: 2001,
                month: 2,
                tabular: 31,
                computed: 32
            }
        );
        assert_eq!((report.drift_end, report.drift_max), (-1, -1));
        assert_eq!(report.start_offset_max, 1);
        assert!(report.drift_within_a_day());
        assert!((report.month_agreement() - 91.666).abs() < 0.01);
        assert!(
            report
                .markdown_row()
                .starts_with("| test | 33/36 (91.7 %) | 1/3 | 2/3 | -1 (max -1) | 1 |")
        );
        assert!(FitReport::MARKDOWN_HEADER.starts_with("| frame |"));
        let empty = fit("none", &[], &official, start);
        assert!((empty.month_agreement()).abs() < 1e-12);
    }
}
