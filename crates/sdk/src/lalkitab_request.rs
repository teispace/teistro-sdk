//! Lal Kitab through the façade (`03-design/lalkitab.md`): a chart's teva
//! read off its lagna and grahas, and the life the kernel reads from it.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Rashi;
use teistro_core::error::Error;
use teistro_lalkitab::tables::PLANETS;
use teistro_lalkitab::{CycleStart, Life, LifeRules, Teva, VarshphalTable, life};
use teistro_serial::Document;

use crate::ChartArea;

/// The record's name where a binding sends it, which a refusal is named
/// under.
const LALKITAB: &str = "lalkitab";

/// A varshphal list as a reader writes it: row `y − 1` is year of life
/// `y`, and its entry `h − 1` the house natal house `h` reaches that year
/// (`03-design/lalkitab.md` §3.6). The SDK does not ship the book's list.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VarshphalRows {
    /// The 120 rows, one per year of life.
    pub rows: Vec<[u8; 12]>,
}

/// What a chart's Lal Kitab is read under: where the 35-year cycle starts,
/// a year of life to read, and the varshphal list its annual teva comes
/// from.
///
/// ```
/// use teistro::LalKitabRequest;
/// use teistro::catalogue::Graha;
///
/// let asked = LalKitabRequest::from_json(
///     r#"{"cycle": {"planet": "VENUS", "year": 17}, "year": 30}"#,
/// )?;
/// assert_eq!(asked.cycle.planet, Graha::Venus);
/// assert_eq!(asked.year, Some(30));
/// let typo = LalKitabRequest::from_json(r#"{"years": 30}"#).unwrap_err();
/// assert_eq!(typo.field(), Some("lalkitab.years"));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct LalKitabRequest {
    /// Where the cycle starts; the book's general table (Saturn from the
    /// first year) when left out.
    pub cycle: CycleStart,
    /// The year of life to read, from 1 (birth to the first birthday);
    /// none reads no year.
    pub year: Option<u16>,
    /// The list the year's annual teva is read from; none reads none.
    pub varshphal: Option<VarshphalRows>,
}

impl LalKitabRequest {
    /// The record a binding sends, as JSON: `cycle` (`planet`, `year`),
    /// `year` and `varshphal` (`rows`), every member optional.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, or a key it does not
    /// read, named under `lalkitab`.
    pub fn from_json(text: &str) -> Result<LalKitabRequest, Error> {
        teistro_core::strict::read(text, LALKITAB)
    }
}

impl ChartArea<'_> {
    /// A chart's Lal Kitab (`03-design/lalkitab.md`): its teva, each
    /// graha's whole-sign house from the lagna's sign in the chart's own
    /// zodiac (crux LK-C1), and what the teva says over a life — the
    /// reading, the 35-year cycle, and the year asked for.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on a cycle start, a year or a varshphal list the
    /// kernel refuses, named under `lalkitab` (`lalkitab.cycle.year`,
    /// `lalkitab.year`, `lalkitab.varshphal.rows[5]`).
    pub fn lalkitab(self, document: &Document, asked: &LalKitabRequest) -> Result<Life, Error> {
        let foundation = &document.foundation;
        let lagna = Rashi::of_longitude(foundation.lagna_deg);
        let mut signs = Vec::with_capacity(PLANETS.len());
        for graha in PLANETS {
            let at = foundation.graha(graha).ok_or_else(|| {
                Error::internal(format!("a founded chart places {}", graha.key()))
            })?;
            signs.push((graha, Rashi::of_longitude(at.longitude_deg)));
        }
        let teva = Teva::from_signs(lagna, signs)?;
        let cycle = CycleStart::new(asked.cycle.planet, asked.cycle.year)
            .map_err(|error| error.under("lalkitab.cycle"))?;
        let varshphal = asked
            .varshphal
            .as_ref()
            .map(|list| VarshphalTable::from_rows(list.rows.clone()))
            .transpose()
            .map_err(|error| error.under("lalkitab.varshphal"))?;
        let rules = LifeRules {
            cycle,
            year: asked.year,
            varshphal,
        };
        life(&teva, &rules).map_err(|error| error.under(LALKITAB))
    }
}
