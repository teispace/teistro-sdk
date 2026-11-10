//! A guna milan total graded into bands (`03-design/matching.md` §8,
//! SR-39): a table of rows a consumer can replace, and three tables as
//! printed.
//!
//! The rank 1 text is *Muhurta Martanda* (Narayana, 1571), Vivaha-prakarana
//! v. 1, whose *ahīndūrdhvaguṇaikyaṃ śubham* sets a floor: a total above 18
//! is auspicious, more the better. It names no band above the floor. The
//! bands Nepal's almanacs print (middling, good, excellent) are their own
//! convention, and they lower the floor to 16 when the two signs stand
//! well (*satkuta*). Each is a [`GunaBands`] value, and a consumer's own
//! table is one too.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Koota;
use teistro_core::error::Error;

use crate::ashta::{ASHTA_KOOTA_POINTS, AshtaKoota, KootaReading};

/// The field a table's refusals are named under.
const BANDS: &str = "bands";

/// A band a total falls in. The words for each are the `sdk.matching.band`
/// message's, in every locale that carries it.
///
/// ```
/// use teistro_matching::GunaBand;
///
/// assert!(GunaBand::Unfit < GunaBand::Fit);
/// assert!(GunaBand::Good < GunaBand::Excellent);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GunaBand {
    /// Not fit for marriage: below the floor (*aśubha*, *nindya*, "does
    /// not join").
    Unfit,
    /// Above the floor, with no band named above it: *Muhurta Martanda*'s
    /// *śubha*.
    Fit,
    /// *Madhyama*: above the floor, up to 20.
    Middling,
    /// *Uttama* (*śreṣṭha* in one almanac): above 20, up to 30.
    Good,
    /// *Ati-uttama* (*uttama* in one almanac): above 30.
    Excellent,
}

impl GunaBand {
    /// Every band, lowest first.
    pub const ALL: [GunaBand; 5] = [
        GunaBand::Unfit,
        GunaBand::Fit,
        GunaBand::Middling,
        GunaBand::Good,
        GunaBand::Excellent,
    ];
}

/// How a total clears a row's threshold.
///
/// ```
/// use teistro_matching::Clears;
///
/// assert!(Clears::AtLeast.cleared(18.0, 18.0));
/// assert!(!Clears::Above.cleared(18.0, 18.0));
/// assert!(Clears::Above.cleared(18.5, 18.0));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Clears {
    /// The total is the threshold or more ("18 or more").
    #[default]
    AtLeast,
    /// The total is more than the threshold ("above 18").
    Above,
}

impl Clears {
    /// Whether `total` clears `threshold` this way.
    ///
    /// ```
    /// use teistro_matching::Clears;
    ///
    /// assert!(Clears::AtLeast.cleared(16.0, 16.0));
    /// assert!(!Clears::Above.cleared(16.0, 16.0));
    /// ```
    #[must_use]
    pub fn cleared(self, total: f64, threshold: f64) -> bool {
        match self {
            Clears::AtLeast => total >= threshold,
            Clears::Above => total > threshold,
        }
    }
}

/// How the two signs must stand for a row to apply: the almanacs' *satkuta*
/// and *dushkuta*, read from the signs alone (2/12, 6/8 and 9/5 are the
/// dushkuta), whether or not a lift restores the Bhakoot.
///
/// ```
/// use teistro_matching::BhakootStanding;
///
/// assert_eq!(BhakootStanding::default(), BhakootStanding::Any);
/// assert!(BhakootStanding::Satkuta.holds(false));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BhakootStanding {
    /// Either way.
    #[default]
    Any,
    /// The signs stand well: no 2/12, 6/8 or 9/5.
    Satkuta,
    /// The signs stand 2/12, 6/8 or 9/5.
    Dushkuta,
}

impl BhakootStanding {
    /// Whether a match whose signs stand as `dushkuta` says meets this.
    ///
    /// ```
    /// use teistro_matching::BhakootStanding;
    ///
    /// assert!(BhakootStanding::Any.holds(true));
    /// assert!(!BhakootStanding::Satkuta.holds(true));
    /// assert!(BhakootStanding::Dushkuta.holds(true));
    /// ```
    #[must_use]
    pub const fn holds(self, dushkuta: bool) -> bool {
        match self {
            BhakootStanding::Any => true,
            BhakootStanding::Satkuta => !dushkuta,
            BhakootStanding::Dushkuta => dushkuta,
        }
    }
}

/// One row of a table: the band a total takes when it clears `from` and
/// the signs stand as `bhakoot` says.
///
/// ```
/// use teistro_matching::{BandRow, BhakootStanding, Clears, GunaBand};
///
/// let row: BandRow = serde_json::from_str(r#"{"band": "GOOD", "from": 20, "clears": "ABOVE"}"#)?;
/// assert_eq!((row.band, row.clears, row.bhakoot), (GunaBand::Good, Clears::Above, BhakootStanding::Any));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct BandRow {
    /// The band.
    pub band: GunaBand,
    /// The threshold, out of 36.
    pub from: f64,
    /// How the total clears it; `AT_LEAST` when left out.
    #[serde(default)]
    pub clears: Clears,
    /// How the signs must stand; `ANY` when left out.
    #[serde(default)]
    pub bhakoot: BhakootStanding,
}

impl BandRow {
    const fn new(band: GunaBand, from: f64, clears: Clears, bhakoot: BhakootStanding) -> BandRow {
        BandRow {
            band,
            from,
            clears,
            bhakoot,
        }
    }
}

/// A table of bands: rows in order, the last that applies giving the band.
/// The first row must take every total (`from` 0, `AT_LEAST`, `ANY`), so
/// no total is left without one.
///
/// ```
/// use teistro_matching::{GunaBand, GunaBands};
///
/// // The rank 1 floor: above 18 is auspicious.
/// let martanda = GunaBands::default();
/// assert_eq!(martanda.name, "MUHURTA_MARTANDA");
/// assert_eq!(martanda.grade_total(18.0, false).band, GunaBand::Unfit);
/// assert_eq!(martanda.grade_total(18.5, false).band, GunaBand::Fit);
///
/// // Nepal's almanacs: 18 joins, and 16 is passed when the signs stand well.
/// let almanac = GunaBands::nepali_almanac();
/// assert_eq!(almanac.grade_total(18.0, true).band, GunaBand::Middling);
/// assert_eq!(almanac.grade_total(17.0, false).band, GunaBand::Middling);
/// assert_eq!(almanac.grade_total(17.0, true).band, GunaBand::Unfit);
/// assert_eq!(almanac.grade_total(30.0, false).band, GunaBand::Good);
/// assert_eq!(almanac.grade_total(30.5, false).band, GunaBand::Excellent);
///
/// // A consumer's own table is a value like these.
/// let own = GunaBands::from_json(
///     r#"{"name": "OURS", "source": "our panel", "rows": [
///         {"band": "UNFIT", "from": 0},
///         {"band": "GOOD", "from": 24}]}"#,
/// )?;
/// assert_eq!(own.grade_total(24.0, false).band, GunaBand::Good);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GunaBands {
    /// The table's name, which the grade carries.
    pub name: String,
    /// Where its rows come from.
    pub source: String,
    /// The rows, in order: the last that applies gives the band.
    pub rows: Vec<BandRow>,
}

impl Default for GunaBands {
    /// The rank 1 text's: [`GunaBands::muhurta_martanda`].
    fn default() -> GunaBands {
        GunaBands::muhurta_martanda()
    }
}

/// A total graded by a table.
///
/// ```
/// use teistro_matching::{GunaBand, GunaBands, KootaRules, Native, ashta_koota};
///
/// // Two Moons in Rohini: 28 of 36, and one sign, so no Bhakoot dosha.
/// let moon = Native::of_moon(45.0)?;
/// let koota = ashta_koota(moon, moon, KootaRules::default());
/// let graded = GunaBands::nepali_almanac().grade(&koota);
/// assert_eq!((graded.total, graded.band, graded.dushkuta), (28.0, GunaBand::Good, false));
/// assert_eq!(graded.bands, "NEPALI_ALMANAC");
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Graded {
    /// The total, out of 36.
    pub total: f64,
    /// Whether the signs stand 2/12, 6/8 or 9/5.
    pub dushkuta: bool,
    /// The band.
    pub band: GunaBand,
    /// The table's name.
    pub bands: String,
    /// The row that gave the band, counted from 0.
    pub row: usize,
}

impl GunaBands {
    /// The names of the tables shipped, the default first.
    pub const SHIPPED: [&'static str; 3] = ["MUHURTA_MARTANDA", "NEPALI_ALMANAC", "ARYAVARTA"];

    /// *Muhurta Martanda* (Narayana, 1571), Vivaha-prakarana v. 1:
    /// *ahīndūrdhvaguṇaikyaṃ śubham*, the agreement of gunas above 18
    /// (*ahi* 8, *indu* 1, read right to left) is auspicious, and the more
    /// the better (the author's gloss: *aṣṭādaśādhikaṃ guṇaikyaṃ
    /// yathottaraṃ śubham*). Read in the Khemraj edition, Bombay 1904,
    /// pp. 32–33. No band above the floor is named, so there are two.
    ///
    /// ```
    /// use teistro_matching::{GunaBand, GunaBands};
    ///
    /// let bands = GunaBands::muhurta_martanda();
    /// assert_eq!(bands.grade_total(36.0, true).band, GunaBand::Fit);
    /// ```
    #[must_use]
    pub fn muhurta_martanda() -> GunaBands {
        GunaBands {
            name: String::from("MUHURTA_MARTANDA"),
            source: String::from(
                "Muhurta Martanda, Vivaha-prakarana v. 1 and the author's gloss (Khemraj, Bombay 1904, pp. 32-33): above 18 is auspicious, the more the better",
            ),
            rows: vec![
                BandRow::new(GunaBand::Unfit, 0.0, Clears::AtLeast, BhakootStanding::Any),
                BandRow::new(GunaBand::Fit, 18.0, Clears::Above, BhakootStanding::Any),
            ],
        }
    }

    /// The bands Nepal's almanacs print over their guna grids, a
    /// convention no verse gives: below 18 the marriage does not join, up
    /// to 20 middling, 20 to 30 good, 30 to 36 excellent; with the signs
    /// standing well (no 2/12, 6/8 or 9/5), above 16 joins. *Surya
    /// Panchanga* VS 2082, printed p. 10, and *Kashyapa Panchanga* VS 2082,
    /// pp. 12 and 15. A boundary both bands name (20, 30) is the lower
    /// band's, as Kashyapa's "up to 30 good, above 30 excellent" reads it.
    ///
    /// ```
    /// use teistro_matching::{GunaBand, GunaBands};
    ///
    /// let bands = GunaBands::nepali_almanac();
    /// assert_eq!(bands.grade_total(20.0, false).band, GunaBand::Middling);
    /// assert_eq!(bands.grade_total(16.5, false).band, GunaBand::Middling);
    /// assert_eq!(bands.grade_total(16.0, false).band, GunaBand::Unfit);
    /// ```
    #[must_use]
    pub fn nepali_almanac() -> GunaBands {
        GunaBands {
            name: String::from("NEPALI_ALMANAC"),
            source: String::from(
                "Surya Panchanga VS 2082 p. 10 and Kashyapa Panchanga VS 2082 pp. 12, 15: below 18 does not join, up to 20 middling, 20 to 30 good, above 30 excellent; with no 2/12, 6/8 or 9/5, above 16 joins",
            ),
            rows: vec![
                BandRow::new(GunaBand::Unfit, 0.0, Clears::AtLeast, BhakootStanding::Any),
                BandRow::new(
                    GunaBand::Middling,
                    16.0,
                    Clears::Above,
                    BhakootStanding::Satkuta,
                ),
                BandRow::new(
                    GunaBand::Middling,
                    18.0,
                    Clears::AtLeast,
                    BhakootStanding::Any,
                ),
                BandRow::new(GunaBand::Good, 20.0, Clears::Above, BhakootStanding::Any),
                BandRow::new(
                    GunaBand::Excellent,
                    30.0,
                    Clears::Above,
                    BhakootStanding::Any,
                ),
            ],
        }
    }

    /// *Aryavarta Panchanga* VS 2082 (PDF p. 7, over its guna grid): up to
    /// 16 blamed (*nindya*), 17 to 20 middling, 20 to 30 best
    /// (*śreṣṭha*), above 30 excellent (*uttama*); the signs are not asked.
    ///
    /// ```
    /// use teistro_matching::{GunaBand, GunaBands};
    ///
    /// let bands = GunaBands::aryavarta();
    /// assert_eq!(bands.grade_total(16.0, false).band, GunaBand::Unfit);
    /// assert_eq!(bands.grade_total(17.0, true).band, GunaBand::Middling);
    /// ```
    #[must_use]
    pub fn aryavarta() -> GunaBands {
        GunaBands {
            name: String::from("ARYAVARTA"),
            source: String::from(
                "Aryavarta Panchanga VS 2082, over its guna grid: up to 16 nindya, 17 to 20 madhyama, 20 to 30 shreshtha, above 30 uttama",
            ),
            rows: vec![
                BandRow::new(GunaBand::Unfit, 0.0, Clears::AtLeast, BhakootStanding::Any),
                BandRow::new(
                    GunaBand::Middling,
                    16.0,
                    Clears::Above,
                    BhakootStanding::Any,
                ),
                BandRow::new(GunaBand::Good, 20.0, Clears::Above, BhakootStanding::Any),
                BandRow::new(
                    GunaBand::Excellent,
                    30.0,
                    Clears::Above,
                    BhakootStanding::Any,
                ),
            ],
        }
    }

    /// A shipped table by its name, one of [`GunaBands::SHIPPED`].
    ///
    /// ```
    /// use teistro_matching::GunaBands;
    ///
    /// assert_eq!(GunaBands::shipped("ARYAVARTA"), Some(GunaBands::aryavarta()));
    /// assert_eq!(GunaBands::shipped("aryavarta"), None);
    /// ```
    #[must_use]
    pub fn shipped(name: &str) -> Option<GunaBands> {
        match name {
            "MUHURTA_MARTANDA" => Some(GunaBands::muhurta_martanda()),
            "NEPALI_ALMANAC" => Some(GunaBands::nepali_almanac()),
            "ARYAVARTA" => Some(GunaBands::aryavarta()),
            _ => None,
        }
    }

    /// A table as JSON: `name`, `source` and `rows`, each row `band`,
    /// `from`, and optionally `clears` and `bhakoot`; checked as
    /// [`GunaBands::check`] does.
    ///
    /// ```
    /// use teistro_matching::GunaBands;
    ///
    /// // The first row must take every total.
    /// let gapped = GunaBands::from_json(
    ///     r#"{"name": "X", "source": "", "rows": [{"band": "FIT", "from": 18}]}"#,
    /// )
    /// .unwrap_err();
    /// assert_eq!(gapped.field(), Some("bands.rows[0]"));
    /// let typo = GunaBands::from_json(
    ///     r#"{"name": "X", "source": "", "rows": [{"band": "FIT", "form": 18}]}"#,
    /// )
    /// .unwrap_err();
    /// assert!(typo.field().is_some_and(|field| field.starts_with("bands.rows[0]")));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, or a table [`GunaBands::check`] refuses, named under `bands`.
    pub fn from_json(text: &str) -> Result<GunaBands, Error> {
        let bands: GunaBands = teistro_core::strict::read(text, BANDS)?;
        bands.check()?;
        Ok(bands)
    }

    /// Whether the table grades every total: a name, at least one row, the
    /// first taking every total, and each threshold within 0 to 36.
    ///
    /// ```
    /// use teistro_matching::GunaBands;
    ///
    /// for name in GunaBands::SHIPPED {
    ///     assert!(GunaBands::shipped(name).is_some_and(|bands| bands.check().is_ok()));
    /// }
    /// let mut nameless = GunaBands::default();
    /// nameless.name.clear();
    /// assert_eq!(nameless.check().unwrap_err().field(), Some("bands.name"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG`, named by the field at fault under `bands`.
    pub fn check(&self) -> Result<(), Error> {
        if self.name.trim().is_empty() {
            return Err(
                Error::invalid_arg("a table of bands needs a name").with_field("bands.name")
            );
        }
        let Some(first) = self.rows.first() else {
            return Err(Error::invalid_arg("a table of bands needs a row").with_field("bands.rows"));
        };
        let takes_all = first.from.abs() < f64::EPSILON
            && first.clears == Clears::AtLeast
            && first.bhakoot == BhakootStanding::Any;
        if !takes_all {
            return Err(Error::invalid_arg(
                "the first row must take every total: `from` 0, `clears` AT_LEAST, `bhakoot` ANY",
            )
            .with_field("bands.rows[0]"));
        }
        for (at, row) in self.rows.iter().enumerate() {
            if !(row.from.is_finite() && (0.0..=ASHTA_KOOTA_POINTS).contains(&row.from)) {
                return Err(Error::invalid_arg(format!(
                    "a threshold is within 0 to 36, not {}",
                    row.from
                ))
                .with_field(format!("bands.rows[{at}].from")));
            }
        }
        Ok(())
    }

    /// The band of an Ashta Koota: its total, and whether its signs stand
    /// 2/12, 6/8 or 9/5, read from its Bhakoot row.
    ///
    /// ```
    /// use teistro_matching::{GunaBand, GunaBands, KootaRules, Native, ashta_koota};
    ///
    /// let moon = Native::of_moon(45.0)?;
    /// let koota = ashta_koota(moon, moon, KootaRules::default());
    /// assert_eq!(GunaBands::default().grade(&koota).band, GunaBand::Fit);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn grade(&self, koota: &AshtaKoota) -> Graded {
        let dushkuta = koota
            .row(Koota::Bhakoot)
            .is_some_and(|row| matches!(row.reading, KootaReading::Bhakoot { dosha: Some(_), .. }));
        self.grade_total(koota.total, dushkuta)
    }

    /// The band of a total, the signs standing as `dushkuta` says: the last
    /// row that applies. A table [`GunaBands::check`] refuses still
    /// answers, `UNFIT` from its first row when no row applies.
    ///
    /// ```
    /// use teistro_matching::{GunaBand, GunaBands};
    ///
    /// let bands = GunaBands::nepali_almanac();
    /// // 17 is middling when the signs stand well, unfit when they do not.
    /// assert_eq!(bands.grade_total(17.0, false).band, GunaBand::Middling);
    /// assert_eq!(bands.grade_total(17.0, true).band, GunaBand::Unfit);
    /// assert_eq!(bands.grade_total(18.0, true).band, GunaBand::Middling);
    /// ```
    #[must_use]
    pub fn grade_total(&self, total: f64, dushkuta: bool) -> Graded {
        let (row, band) = self
            .rows
            .iter()
            .enumerate()
            .filter(|(_, row)| row.clears.cleared(total, row.from) && row.bhakoot.holds(dushkuta))
            .map(|(at, row)| (at, row.band))
            .next_back()
            .unwrap_or((0, GunaBand::Unfit));
        Graded {
            total,
            dushkuta,
            band,
            bands: self.name.clone(),
            row,
        }
    }
}
