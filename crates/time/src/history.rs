//! A zone's offsets as a history: rows in force from an instant, the
//! first row before its own instant too (tzdb's local-mean-time stub).

use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::time::{LocalClock, UtcOffset};

/// One row of a history: the offset in force from an instant.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OffsetRow {
    /// The instant the offset takes effect, in UTC.
    pub from: JulianDay<Utc>,
    /// The offset.
    pub offset: UtcOffset,
    /// tzdb's abbreviation for the row (`LMT`, `+0545`).
    pub abbreviation: &'static str,
}

/// A zone's offset history: rows in ascending order of their instants.
#[derive(Clone, Debug, PartialEq)]
pub struct OffsetHistory {
    /// The zone's name (`Asia/Kathmandu`).
    pub zone: &'static str,
    /// The source the rows were taken from, for the stamp.
    pub source: &'static str,
    /// The rows, ascending.
    pub rows: &'static [OffsetRow],
}

impl OffsetHistory {
    /// The row in force at an instant: the last row at or before it, or
    /// the first row before any of them.
    #[must_use]
    pub fn row_at(&self, instant: JulianDay<Utc>) -> Option<&OffsetRow> {
        let index = self
            .rows
            .partition_point(|row| row.from.get() <= instant.get());
        self.rows.get(index.saturating_sub(1))
    }

    /// The instants at which the offset changes, for tests and reports.
    pub fn transitions(&self) -> impl Iterator<Item = JulianDay<Utc>> + '_ {
        self.rows.iter().skip(1).map(|row| row.from)
    }
}

impl LocalClock for OffsetHistory {
    fn offset_at(&self, instant: JulianDay<Utc>) -> UtcOffset {
        self.row_at(instant)
            .map_or(UtcOffset::UTC, |row| row.offset)
    }

    fn describe(&self) -> String {
        format!("{} ({})", self.zone, self.source)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    static ROWS: [OffsetRow; 2] = [
        OffsetRow {
            from: JulianDay::literal(2_400_000.5),
            offset: UtcOffset::literal(1, 0, 0),
            abbreviation: "LMT",
        },
        OffsetRow {
            from: JulianDay::literal(2_450_000.5),
            offset: UtcOffset::literal(2, 0, 0),
            abbreviation: "+02",
        },
    ];

    #[test]
    fn rows_answer_before_between_and_after() {
        let history = OffsetHistory {
            zone: "Test/Zone",
            source: "a test",
            rows: &ROWS,
        };
        let at = |jd: f64| history.offset_at(JulianDay::try_new(jd).unwrap());
        assert_eq!(at(2_000_000.0).seconds(), 3600);
        assert_eq!(at(2_400_000.5).seconds(), 3600);
        assert_eq!(at(2_449_999.0).seconds(), 3600);
        assert_eq!(at(2_450_000.5).seconds(), 7200);
        assert_eq!(at(2_460_000.0).seconds(), 7200);
        assert_eq!(history.transitions().count(), 1);
        assert_eq!(history.describe(), "Test/Zone (a test)");
        assert_eq!(
            history
                .row_at(JulianDay::try_new(2_460_000.0).unwrap())
                .unwrap()
                .abbreviation,
            "+02"
        );
        let empty = OffsetHistory {
            zone: "Empty",
            source: "none",
            rows: &[],
        };
        assert_eq!(
            empty.offset_at(JulianDay::try_new(2_460_000.0).unwrap()),
            UtcOffset::UTC
        );
    }
}
