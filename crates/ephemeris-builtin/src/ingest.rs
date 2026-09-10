//! Reading the published VSOP87 files.
//!
//! This is the **generator's** side and is behind the `ingest` feature.
//! A consumer receives truncated tables and has no use for a parser of a
//! 5.7 MB text file; paying for one in a wasm binary would be the
//! opposite of the point of a `compact` tier.
//!
//! The source is CDS catalogue VI/81 (Bretagnon and Francou 1988). The
//! files are deliberately not in the repository: what is checked in is
//! what the ingester emits.

use std::collections::BTreeMap;
use std::path::Path;

use crate::series::Term;

/// The three coordinates a VSOP87 file carries for one body, each a flat
/// list of terms that already carry their own power of `t`.
///
/// Which three depends on the variant: `X, Y, Z` for the rectangular
/// variants (A, C, E) and longitude, latitude and radius for the
/// spherical ones (B, D). The SDK ingests variant A, so these are
/// heliocentric rectangular coordinates in the ecliptic and equinox of
/// J2000.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BodySeries {
    /// The three coordinates, in the file's own order.
    pub coordinates: [Vec<Term>; 3],
}

impl BodySeries {
    /// How many terms the whole body carries.
    #[must_use]
    pub fn terms(&self) -> usize {
        self.coordinates.iter().map(Vec::len).sum()
    }

    /// The body's three coordinates at `t`, keeping only terms at or
    /// above `threshold` in absolute amplitude.
    #[must_use]
    pub fn at(&self, t: f64, threshold: f64) -> [f64; 3] {
        let mut out = [0.0; 3];
        for (slot, terms) in out.iter_mut().zip(&self.coordinates) {
            *slot = terms
                .iter()
                .filter(|term| term.amplitude.abs() >= threshold)
                .map(|term| term.at(t))
                .sum();
        }
        out
    }
}

/// The eight bodies VSOP87 indexes, with the suffix its files use.
///
/// The Earth is one of them and is not optional: it is subtracted from
/// the other seven to make a geocentric direction, and it is what the
/// Sun's direction is computed from.
pub const BODIES: [(&str, &str); 8] = [
    ("Mercury", "mer"),
    ("Venus", "ven"),
    ("Earth", "ear"),
    ("Mars", "mar"),
    ("Jupiter", "jup"),
    ("Saturn", "sat"),
    ("Uranus", "ura"),
    ("Neptune", "nep"),
];

/// What went wrong reading a file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IngestError {
    /// A line was shorter than the fixed record the format specifies.
    Short {
        /// The one-based line number.
        line: usize,
        /// How many bytes it held.
        length: usize,
    },
    /// A field did not parse as the format says it must.
    Field {
        /// The one-based line number.
        line: usize,
        /// Which field.
        field: &'static str,
    },
    /// A coordinate index outside the three a body has.
    Coordinate {
        /// The one-based line number.
        line: usize,
        /// What the file said.
        found: usize,
    },
    /// A file could not be read at all.
    Unreadable {
        /// The path that failed.
        path: String,
        /// What the operating system said.
        detail: String,
    },
}

impl core::fmt::Display for IngestError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            IngestError::Short { line, length } => write!(
                f,
                "line {line} is {length} bytes; the record format needs at least {MINIMUM_RECORD}"
            ),
            IngestError::Field { line, field } => {
                write!(
                    f,
                    "line {line}: the {field} field did not parse as a number"
                )
            }
            IngestError::Coordinate { line, found } => write!(
                f,
                "line {line}: coordinate {found} is not 1, 2 or 3, so the file is not a VSOP87 body file"
            ),
            IngestError::Unreadable { path, detail } => write!(f, "cannot read {path}: {detail}"),
        }
    }
}

impl std::error::Error for IngestError {}

/// The shortest a term record can be: the format is
/// `1x,4i1,i5,12i3,f15.11,2f18.11,f14.11,f20.11`, whose last field ends
/// at column 131.
const MINIMUM_RECORD: usize = 131;

/// Where each field the SDK needs sits, by the columns the format fixes.
///
/// The columns are read by position rather than by splitting on
/// whitespace because the twelve mean-longitude multipliers are `i3`
/// fields, which touch as soon as one of them reaches `-10`.
const AMPLITUDE: core::ops::Range<usize> = 79..97;
const PHASE: core::ops::Range<usize> = 97..111;
const FREQUENCY: core::ops::Range<usize> = 111..131;

/// Parses one VSOP87 body file.
///
/// Header lines are skipped by their own marker; every other line is a
/// term record. The amplitude, phase and frequency are taken and the
/// equivalent sine and cosine pair the file also carries is not, because
/// the two forms are the same series and the evaluator uses one of them.
///
/// # Errors
///
/// [`IngestError`] naming the line and the field, because a file that
/// does not parse is a file the SDK misread rather than a file that is
/// wrong, and the line number is what tells the two apart.
pub fn parse(text: &str) -> Result<BodySeries, IngestError> {
    let mut series = BodySeries::default();
    for (index, line) in text.lines().enumerate() {
        let number = index + 1;
        if line.trim_start().starts_with("VSOP87") {
            continue;
        }
        if line.len() < MINIMUM_RECORD {
            return Err(IngestError::Short {
                line: number,
                length: line.len(),
            });
        }
        let field = |range: core::ops::Range<usize>, what: &'static str| {
            line.get(range)
                .and_then(|text| text.trim().parse::<f64>().ok())
                .ok_or(IngestError::Field {
                    line: number,
                    field: what,
                })
        };
        let coordinate = line
            .get(3..4)
            .and_then(|text| text.parse::<usize>().ok())
            .ok_or(IngestError::Field {
                line: number,
                field: "coordinate",
            })?;
        let power = line
            .get(4..5)
            .and_then(|text| text.parse::<u8>().ok())
            .ok_or(IngestError::Field {
                line: number,
                field: "power",
            })?;
        // `get_mut` rather than a bound check and then an index: the
        // check and the access become one step, so there is no `expect`
        // left for a later edit to make reachable.
        let slot = coordinate
            .checked_sub(1)
            .and_then(|slot| series.coordinates.get_mut(slot))
            .ok_or(IngestError::Coordinate {
                line: number,
                found: coordinate,
            })?;
        slot.push(Term::new(
            field(AMPLITUDE, "amplitude")?,
            field(PHASE, "phase")?,
            field(FREQUENCY, "frequency")?,
            power,
        ));
    }
    Ok(series)
}

/// Reads every body's VSOP87A series from a directory of the published
/// files.
///
/// # Errors
///
/// [`IngestError::Unreadable`] for a file that is not there, and
/// whatever [`parse`] reports for one that is.
pub fn load(dir: &Path) -> Result<BTreeMap<&'static str, BodySeries>, IngestError> {
    let mut out = BTreeMap::new();
    for (name, suffix) in BODIES {
        let path = dir.join(format!("VSOP87A.{suffix}"));
        let text = std::fs::read_to_string(&path).map_err(|error| IngestError::Unreadable {
            path: path.display().to_string(),
            detail: error.to_string(),
        })?;
        out.insert(name, parse(&text)?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "a test fails by panicking and reads its own fixture by index"
    )]

    use super::*;

    /// One real record, with the header line above it as the files have.
    const SAMPLE: &str = " VSOP87 VERSION A1    EARTH     VARIABLE 1 (XYZ)       *T**0    843 TERMS
 1310    1  0  0  1  0  0  0  0  0  0  0  0  0 -0.00001522262     0.99982928833     0.99982928844 1.75348568475    6283.07584999140 ";

    /// The file's own redundancy is the check: it carries `S` and `K`
    /// beside `A`, and `A` is `hypot(S, K)`. A column read off by one is
    /// then caught by arithmetic rather than by eye.
    #[test]
    fn the_parser_reads_the_columns_the_format_fixes() {
        let series = parse(SAMPLE).expect("a well-formed record");
        assert_eq!(series.coordinates[0].len(), 1, "one X term");
        assert_eq!(series.terms(), 1);
        let term = series.coordinates[0][0];
        let sine = -0.000_015_222_62_f64;
        let cosine = 0.999_829_288_33_f64;
        assert!(
            (term.amplitude - sine.hypot(cosine)).abs() < 1e-11,
            "the amplitude column is A = hypot(S, K): {} against {}",
            term.amplitude,
            sine.hypot(cosine)
        );
        assert!((term.phase - 1.753_485_684_75).abs() < 1e-11);
        assert!((term.frequency - 6_283.075_849_991_40).abs() < 1e-9);
        assert_eq!(term.power, 0);
    }

    #[test]
    fn a_header_is_skipped_and_not_read_as_a_term() {
        let series =
            parse(" VSOP87 VERSION A1    EARTH     VARIABLE 1 (XYZ)").expect("only a header");
        assert_eq!(series.terms(), 0);
    }

    #[test]
    fn a_short_record_names_its_line_rather_than_being_skipped() {
        let text = format!("{SAMPLE}\n 1310    2  0  0");
        assert_eq!(
            parse(&text),
            Err(IngestError::Short {
                line: 3,
                length: 16
            }),
            "a truncated record is refused, and says which line"
        );
    }

    #[test]
    fn a_threshold_keeps_the_terms_at_or_above_it() {
        let mut series = BodySeries::default();
        series.coordinates[0] = vec![Term::new(1.0, 0.0, 0.0, 0), Term::new(0.25, 0.0, 0.0, 0)];
        assert!((series.at(0.0, 0.0)[0] - 1.25).abs() < 1e-15, "everything");
        assert!((series.at(0.0, 0.25)[0] - 1.25).abs() < 1e-15, "at is kept");
        assert!(
            (series.at(0.0, 0.5)[0] - 1.0).abs() < 1e-15,
            "below is dropped"
        );
    }
}
