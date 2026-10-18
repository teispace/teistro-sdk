//! A chart request's `masaPravesha` record: one year's months, or one
//! month's days, of each chart (`03-design/annual-chart.md`, "The month's
//! and the day's chart").
//!
//! A month's chart is its own first day's, as the *Tajika Nilakanthi*'s
//! commentary says, so both answers are rows of one shape,
//! [`DinaPravesha`]: the twelve months as their first days, or the thirty
//! days of the month asked for. Given a `place`, each month's chart is
//! founded there and its lord, the [`Masesha`], and its results, the
//! [`MasaPhala`], read beside it; or each day's, and its lord, the
//! [`Dinesha`], and its results, the [`DinaPhala`].

use serde::{Deserialize, Serialize};
use teistro_core::error::Error;
use teistro_core::time::UtcOffset;
use teistro_serial::Document;
use teistro_tajika::{
    DinaPhala, DinaPravesha, Dinesha, MasaPhala, MasaPhalaRules, MasaPravesha, Masesha,
    MaseshaRules, Reading,
};

use crate::area::ChartArea;
use crate::reading::ChartRequest;
use crate::varsha::AnnualPlace;

/// One year's months, or one month's days, of a birth.
///
/// ```
/// use teistro::MasaPraveshaRequest;
/// use teistro::tajika::Reading;
///
/// let months = MasaPraveshaRequest::from_json(r#"{"year": 30}"#)?;
/// assert_eq!(months, MasaPraveshaRequest::new(30));
/// assert_eq!(months.reading, Reading::Sidereal);
/// let days = MasaPraveshaRequest::new(30).with_month(4);
/// assert_eq!(days.month, Some(4));
/// let refused = MasaPraveshaRequest::from_json(r#"{"year": 30, "month": 13}"#).unwrap_err();
/// assert_eq!(refused.field(), Some("masaPravesha.month"));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MasaPraveshaRequest {
    /// Which longitude the year's Sun returns to, and so which zodiac the
    /// year divides in; the tradition's, sidereal, by default. The mean
    /// year places no Sun and is refused.
    #[serde(default)]
    pub reading: Reading,
    /// The year divided, one to two hundred, counted as
    /// [`Pravesha::year`](crate::Pravesha::year) counts it.
    pub year: u16,
    /// The month whose thirty days are asked for, one to twelve; absent,
    /// the year's twelve months are, each as its first day.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub month: Option<u8>,
    /// Where each month's chart is founded, `"birth"` or a residence, as
    /// [`VarshaRequest::place`](crate::VarshaRequest::place) is written;
    /// given, each month's row carries its lord and its results, and each
    /// day's row its lord and its results. Absent, the rows are the instants only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub place: Option<AnnualPlace>,
    /// The readings the month's lord, and the day's, part on; the verse's
    /// own by default.
    #[serde(default)]
    pub masesha: MaseshaRules,
    /// The floors the month's and the day's results judge strong and weak
    /// by, and the waning Moon they count a malefic; the year yogas' own
    /// by default.
    #[serde(default)]
    pub results: MasaPhalaRules,
}

/// One row of a `masaPravesha` answer: a month or a day, and, for a month
/// whose chart was founded, its lord and its results, or for a day, its
/// lord and its results.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct MasaRow {
    /// The month or the day.
    #[serde(flatten)]
    pub entry: DinaPravesha,
    /// The month's lord, when the request named a `place`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub masesha: Option<Masesha>,
    /// The month's results, when the request named a `place`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results: Option<MasaPhala>,
    /// The day's lord, when the request named a `month` and a `place`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dinesha: Option<Dinesha>,
    /// The day's results, when the request named a `month` and a `place`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day_results: Option<DinaPhala>,
}

impl MasaPraveshaRequest {
    /// The twelve months of `year`, in the sidereal year.
    #[must_use]
    pub fn new(year: u16) -> MasaPraveshaRequest {
        MasaPraveshaRequest {
            reading: Reading::Sidereal,
            year,
            month: None,
            place: None,
            masesha: MaseshaRules::default(),
            results: MasaPhalaRules::default(),
        }
    }

    /// The same months, each founded at `place` with its lord read.
    #[must_use]
    pub const fn at(mut self, place: AnnualPlace) -> MasaPraveshaRequest {
        self.place = Some(place);
        self
    }

    /// The same request, the month's lord read under `rules`.
    #[must_use]
    pub const fn with_masesha(mut self, rules: MaseshaRules) -> MasaPraveshaRequest {
        self.masesha = rules;
        self
    }

    /// The same request, the month's results judged under `rules`.
    #[must_use]
    pub const fn with_results(mut self, rules: MasaPhalaRules) -> MasaPraveshaRequest {
        self.results = rules;
        self
    }

    /// The same year's `month`'s thirty days instead.
    #[must_use]
    pub const fn with_month(mut self, month: u8) -> MasaPraveshaRequest {
        self.month = Some(month);
        self
    }

    /// The same request, the year read by `reading`.
    #[must_use]
    pub const fn with_reading(mut self, reading: Reading) -> MasaPraveshaRequest {
        self.reading = reading;
        self
    }

    /// The request a binding writes as `masaPravesha`, read and checked;
    /// a refusal names the field the caller wrote, `masaPravesha.month`.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record, a key it does not
    /// read, or a request [`MasaPraveshaRequest::check`] refuses.
    pub fn from_json(text: &str) -> Result<MasaPraveshaRequest, Error> {
        let asked: MasaPraveshaRequest = teistro_core::strict::read(text, MASA_PRAVESHA)?;
        asked.check().map_err(|why| why.under(MASA_PRAVESHA))?;
        Ok(asked)
    }

    /// Whether the request can be answered, before any Sun is searched
    /// for.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming `year` outside one to two hundred, `month`
    /// outside one to twelve, `results.strongFrom` below
    /// `results.weakBelow`, or `reading` for the mean year.
    pub fn check(&self) -> Result<(), Error> {
        year_of(self.year)?;
        if let Some(month) = self.month {
            month_of(month)?;
        }
        self.results.check().map_err(|why| why.under("results"))?;
        teistro_tajika::divided(self.reading).map(|_| ())
    }
}

/// `year` where a life has it.
///
/// # Errors
///
/// `INVALID_ARG` naming `year` outside one to two hundred.
pub(crate) fn year_of(year: u16) -> Result<u16, Error> {
    teistro_tajika::years(year).map_err(|_| {
        Error::invalid_arg(format!(
            "a year is asked for by the returns it completes, 1 to {}, not {year}",
            teistro_tajika::MOST_YEARS
        ))
        .with_field("year")
    })
}

/// `month` where a year has it.
///
/// # Errors
///
/// `INVALID_ARG` naming `month` outside one to twelve.
pub(crate) fn month_of(month: u8) -> Result<u8, Error> {
    if (1..=teistro_tajika::MONTHS).contains(&month) {
        Ok(month)
    } else {
        Err(Error::invalid_arg(format!(
            "a year divides into months 1 to {}, not {month}",
            teistro_tajika::MONTHS
        ))
        .with_field("month"))
    }
}

/// The record every binding writes the request as.
const MASA_PRAVESHA: &str = "masaPravesha";

impl ChartArea<'_> {
    /// What `asked` asks of one birth: the year's twelve months, each as
    /// its first day, or the month's thirty days
    /// ([`ChartArea::masa_praveshas`], [`ChartArea::dina_praveshas`]).
    /// With a `place`, the year's chart and each month's are founded there
    /// (the birth's own place on `clock` for `"birth"`), and each month
    /// carries its [`ChartArea::masesha`] and its [`ChartArea::masa_phala`],
    /// beside the year's lord read on the year's chart. With a `month` as
    /// well, the year's, the month's and each day's charts are founded, and
    /// each day carries its [`ChartArea::dinesha`] and its
    /// [`ChartArea::dina_phala`], beside the year's and the month's lords.
    ///
    /// # Errors
    ///
    /// As [`ChartArea::dina_praveshas`], [`ChartArea::masesha`],
    /// [`ChartArea::varshesha`], [`ChartArea::masa_phala`],
    /// [`ChartArea::dinesha`] and [`ChartArea::dina_phala`].
    pub fn masa_pravesha(
        self,
        birth: &Document,
        clock: UtcOffset,
        asked: &MasaPraveshaRequest,
    ) -> Result<Vec<MasaRow>, Error> {
        let bare = |entry: DinaPravesha| MasaRow {
            entry,
            masesha: None,
            results: None,
            dinesha: None,
            day_results: None,
        };
        let request = asked.place.map(|place| match place {
            AnnualPlace::Birth => ChartRequest::at(birth.foundation.place, clock),
            AnnualPlace::At { place, offset } => ChartRequest::at(place, offset),
        });
        if let Some(month) = asked.month {
            let days = self.dina_praveshas(birth, asked.reading, asked.year, month)?;
            return match request {
                Some(request) => self.days_with_lords(birth, asked, &request, days),
                None => Ok(days.into_iter().map(bare).collect()),
            };
        }
        let months = self.masa_praveshas(birth, asked.reading, asked.year)?;
        let Some(request) = request else {
            return Ok(months
                .into_iter()
                .map(DinaPravesha::from)
                .map(bare)
                .collect());
        };
        let found = |entry: &MasaPravesha| Ok::<_, Error>(self.reading(entry.at, &request)?.value);
        // The first month's entry is the year's own, so its chart is the
        // year's chart.
        let Some(first) = months.first() else {
            return Ok(Vec::new());
        };
        let annual = found(first)?;
        let year_lord = self
            .varshesha(birth, &annual, first.year, asked.masesha.chain)?
            .graha;
        months
            .iter()
            .map(|entry| {
                let month = if entry.month == 1 {
                    annual.clone()
                } else {
                    found(entry)?
                };
                let lord = self.masesha(birth, &annual, &month, entry, asked.masesha)?;
                let results = self.masa_phala(&month, &lord, year_lord, asked.results)?;
                Ok(MasaRow {
                    entry: DinaPravesha::from(*entry),
                    masesha: Some(lord),
                    results: Some(results),
                    dinesha: None,
                    day_results: None,
                })
            })
            .collect()
    }

    /// One month's `days`, each founded under `request` with its lord read
    /// beside the year's and the month's charts.
    fn days_with_lords(
        self,
        birth: &Document,
        asked: &MasaPraveshaRequest,
        request: &ChartRequest,
        days: Vec<DinaPravesha>,
    ) -> Result<Vec<MasaRow>, Error> {
        let Some(first) = days.first() else {
            return Ok(Vec::new());
        };
        let found = |at| Ok::<_, Error>(self.reading(at, request)?.value);
        let months = self.masa_praveshas(birth, asked.reading, asked.year)?;
        let (Some(year), Some(this_month)) = (
            months.first(),
            months.iter().find(|one| one.month == first.month),
        ) else {
            return Ok(Vec::new());
        };
        let annual = found(year.at)?;
        let year_lord = self
            .varshesha(birth, &annual, year.year, asked.masesha.chain)?
            .graha;
        // The first day's entry is the month's own, so its chart is the
        // month's chart.
        let month = found(first.at)?;
        let month_lord = self
            .masesha(birth, &annual, &month, this_month, asked.masesha)?
            .graha;
        days.into_iter()
            .map(|entry| {
                let day = if entry.day == 1 {
                    month.clone()
                } else {
                    found(entry.at)?
                };
                let lord = self.dinesha(birth, &annual, &month, &day, &entry, asked.masesha)?;
                let results = self.dina_phala(&day, &lord, year_lord, month_lord, asked.results)?;
                Ok(MasaRow {
                    entry,
                    masesha: None,
                    results: None,
                    dinesha: Some(lord),
                    day_results: Some(results),
                })
            })
            .collect()
    }
}
