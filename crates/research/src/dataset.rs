//! A research dataset (`03-design/receipts-and-datasets.md` §2, SR-57):
//! births, each rated for how reliable its time is, and studies, each with
//! the settings it is run under and the worst rating it admits.
//!
//! Loaded strictly and refused by the field. A study is the request
//! `research.study` reads, less what the dataset supplies: the births of
//! the records the study admits, their groups, and their events.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use teistro_core::error::{Error, Status};
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Utc};
use teistro_core::settings::SettingsSnapshot;
use teistro_core::time::UtcOffset;

/// The format a dataset names, and the only one this build reads.
///
/// ```
/// assert_eq!(teistro_research::dataset::FORMAT, "teistro.dataset/1");
/// ```
pub const FORMAT: &str = "teistro.dataset/1";

/// The identifier of the dataset's JSON Schema, generated from these
/// types into `crates/research/schema/dataset.schema.json` by `cargo xtask
/// document-schema`: a name, one per release, as the chart document's is.
pub const SCHEMA_ID: &str = concat!("urn:teistro:schema:dataset:", env!("CARGO_PKG_VERSION"));

/// What a dataset read alone is named under.
const DATASET: &str = "dataset";

/// The fields of `research.study`'s request a dataset supplies, which a
/// study's own request may not carry.
const SUPPLIED: [&str; 3] = ["births", "subjects", "design"];

/// How reliable a birth's data is, by its source: the Rodden Rating, as
/// Astro-Databank keeps it (`03-design/receipts-and-datasets.md` §2.1).
///
/// The rating grades the **source**, not the time's precision: an `AA`
/// time is still rounded where its clerk rounded it.
///
/// ```
/// use teistro_research::dataset::Rating;
///
/// assert!(Rating::A.admitted_by(Rating::B));
/// assert!(!Rating::C.admitted_by(Rating::B));
/// // A record with no time, or a questioned date, is admitted by no study.
/// assert!(!Rating::X.admitted_by(Rating::Dd));
/// assert_eq!(serde_json::to_string(&Rating::Dd)?, r#""DD""#);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Rating {
    /// A birth certificate, a civil register or a written family record.
    Aa,
    /// The native, kin or an associate, from memory.
    A,
    /// A biography or an autobiography.
    B,
    /// Caution: no source, an ambiguous one, or one not credible.
    C,
    /// Dirty data: two or more unsubstantiated, conflicting sources.
    Dd,
    /// The date, without a time.
    X,
    /// The date itself questioned.
    Xx,
}

impl Rating {
    /// Where the rating stands, best first; none for `X` and `XX`, which
    /// grade no time.
    const fn rank(self) -> Option<u8> {
        match self {
            Rating::Aa => Some(0),
            Rating::A => Some(1),
            Rating::B => Some(2),
            Rating::C => Some(3),
            Rating::Dd => Some(4),
            Rating::X | Rating::Xx => None,
        }
    }

    /// Whether a study whose worst admitted rating is `minimum` admits a
    /// record rated `self`: a time rated no worse. `X` and `XX` are
    /// admitted by none, and admit nothing as a minimum.
    ///
    /// ```
    /// use teistro_research::dataset::Rating;
    ///
    /// assert!(Rating::Aa.admitted_by(Rating::Aa));
    /// assert!(!Rating::Dd.admitted_by(Rating::C));
    /// assert!(!Rating::Xx.admitted_by(Rating::Dd));
    /// assert!(!Rating::Aa.admitted_by(Rating::X));
    /// ```
    #[must_use]
    pub const fn admitted_by(self, minimum: Rating) -> bool {
        match (self.rank(), minimum.rank()) {
            (Some(rank), Some(worst)) => rank <= worst,
            _ => false,
        }
    }
}

/// A birth as a dataset records it: the fields `research.study` reads.
///
/// ```
/// # let snapshot = teistro_core::settings::SettingsSnapshot::of(
/// #     &teistro_core::settings::root().validated(teistro_core::settings::ProfileId::new("mine")).unwrap());
/// # let birth = |jd: f64| serde_json::json!({"instant": jd, "latitudeDeg": 27.7,
/// #     "longitudeDeg": 85.3, "utcOffsetSeconds": 20700});
/// # let dataset = teistro_research::dataset::Dataset::from_value(&serde_json::json!({
/// #     "format": "teistro.dataset/1", "name": "a sample",
/// #     "records": [
/// #         {"id": "a", "birth": birth(2447000.25), "rating": "AA", "source": "a register"},
/// #         {"id": "b", "birth": birth(2448000.75), "rating": "X"},
/// #         {"id": "c", "birth": birth(2449000.5), "rating": "A"}],
/// #     "studies": [{"id": "s", "settings": snapshot, "minimumRating": "A",
/// #         "groups": {"a": 0, "c": 1},
/// #         "request": {"study": "COUNTS", "rules": {"shipped": ["YOGAS"]}}}]}), "dataset")?;
/// let birth = dataset.records[0].birth;
/// assert_eq!((birth.latitude_deg, birth.altitude_m), (27.7, None));
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "DatasetBirth"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DatasetBirth {
    /// The instant, a UTC Julian day.
    pub instant: f64,
    /// Degrees north positive.
    pub latitude_deg: f64,
    /// Degrees east positive.
    pub longitude_deg: f64,
    /// Metres above sea level; 0 when left out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub altitude_m: Option<f64>,
    /// The offset from UTC the birth's clock ran at, seconds east positive.
    pub utc_offset_seconds: i32,
    /// How far either side the time may be, in minutes; 0 when left out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uncertainty_minutes: Option<f64>,
}

/// One record: a birth, its rating and where it came from.
///
/// ```
/// use teistro_research::dataset::Rating;
/// # let snapshot = teistro_core::settings::SettingsSnapshot::of(
/// #     &teistro_core::settings::root().validated(teistro_core::settings::ProfileId::new("mine")).unwrap());
/// # let birth = |jd: f64| serde_json::json!({"instant": jd, "latitudeDeg": 27.7,
/// #     "longitudeDeg": 85.3, "utcOffsetSeconds": 20700});
/// # let dataset = teistro_research::dataset::Dataset::from_value(&serde_json::json!({
/// #     "format": "teistro.dataset/1", "name": "a sample",
/// #     "records": [
/// #         {"id": "a", "birth": birth(2447000.25), "rating": "AA", "source": "a register"},
/// #         {"id": "b", "birth": birth(2448000.75), "rating": "X"},
/// #         {"id": "c", "birth": birth(2449000.5), "rating": "A"}],
/// #     "studies": [{"id": "s", "settings": snapshot, "minimumRating": "A",
/// #         "groups": {"a": 0, "c": 1},
/// #         "request": {"study": "COUNTS", "rules": {"shipped": ["YOGAS"]}}}]}), "dataset")?;
/// let record = &dataset.records[0];
/// assert_eq!((record.rating, record.source.as_deref()), (Rating::Aa, Some("a register")));
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "DatasetRecord"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DatasetRecord {
    /// The record's id, unique in the dataset.
    pub id: String,
    /// The birth.
    pub birth: DatasetBirth,
    /// How reliable the birth's time is, by its source.
    pub rating: Rating,
    /// The source, in words: the register, the book and page, the person.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// The instant of the record's event, a UTC Julian day, for a timed
    /// study.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event: Option<f64>,
}

/// One study: the settings it runs under, the worst rating it admits,
/// its groups, and the rest of its request.
///
/// ```
/// use teistro_research::dataset::Rating;
/// # let snapshot = teistro_core::settings::SettingsSnapshot::of(
/// #     &teistro_core::settings::root().validated(teistro_core::settings::ProfileId::new("mine")).unwrap());
/// # let birth = |jd: f64| serde_json::json!({"instant": jd, "latitudeDeg": 27.7,
/// #     "longitudeDeg": 85.3, "utcOffsetSeconds": 20700});
/// # let dataset = teistro_research::dataset::Dataset::from_value(&serde_json::json!({
/// #     "format": "teistro.dataset/1", "name": "a sample",
/// #     "records": [
/// #         {"id": "a", "birth": birth(2447000.25), "rating": "AA", "source": "a register"},
/// #         {"id": "b", "birth": birth(2448000.75), "rating": "X"},
/// #         {"id": "c", "birth": birth(2449000.5), "rating": "A"}],
/// #     "studies": [{"id": "s", "settings": snapshot, "minimumRating": "A",
/// #         "groups": {"a": 0, "c": 1},
/// #         "request": {"study": "COUNTS", "rules": {"shipped": ["YOGAS"]}}}]}), "dataset")?;
/// let study = &dataset.studies[0];
/// assert_eq!(study.minimum_rating, Rating::A);
/// assert!(study.settings.holds());
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "DatasetStudy"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DatasetStudy {
    /// The study's id, unique in the dataset.
    pub id: String,
    /// The settings the study is run under, whole.
    pub settings: SettingsSnapshot,
    /// The worst rating the study admits: `AA`, `A`, `B`, `C` or `DD`.
    pub minimum_rating: Rating,
    /// Each admitted record's group, by its id, for a `COUNTS` or
    /// `COMPARE` study.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub groups: Option<BTreeMap<String, u32>>,
    /// `research.study`'s request less `births`, `subjects` and `design`,
    /// which the dataset supplies.
    pub request: Map<String, Value>,
}

/// A research dataset.
///
/// ```
/// use teistro_research::dataset::{Dataset, Rating};
///
/// let refused = Dataset::from_json(r#"{"format": "teistro.dataset/1", "name": "x", "records": [
///     {"id": "a", "birth": {"instant": 2447000.25, "latitudeDeg": 97, "longitudeDeg": 85.3,
///      "utcOffsetSeconds": 20700}, "rating": "AA"}], "studies": []}"#).unwrap_err();
/// assert_eq!(refused.field(), Some("dataset.records[0].birth.latitudeDeg"));
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Dataset {
    /// [`FORMAT`].
    pub format: String,
    /// The dataset's name.
    pub name: String,
    /// What it is, in words.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The records, in the order a study reads them.
    pub records: Vec<DatasetRecord>,
    /// The studies.
    pub studies: Vec<DatasetStudy>,
}

/// A record a study set aside, and its rating.
///
/// ```
/// use teistro_research::dataset::{Excluded, Rating};
/// # let snapshot = teistro_core::settings::SettingsSnapshot::of(
/// #     &teistro_core::settings::root().validated(teistro_core::settings::ProfileId::new("mine")).unwrap());
/// # let birth = |jd: f64| serde_json::json!({"instant": jd, "latitudeDeg": 27.7,
/// #     "longitudeDeg": 85.3, "utcOffsetSeconds": 20700});
/// # let dataset = teistro_research::dataset::Dataset::from_value(&serde_json::json!({
/// #     "format": "teistro.dataset/1", "name": "a sample",
/// #     "records": [
/// #         {"id": "a", "birth": birth(2447000.25), "rating": "AA", "source": "a register"},
/// #         {"id": "b", "birth": birth(2448000.75), "rating": "X"},
/// #         {"id": "c", "birth": birth(2449000.5), "rating": "A"}],
/// #     "studies": [{"id": "s", "settings": snapshot, "minimumRating": "A",
/// #         "groups": {"a": 0, "c": 1},
/// #         "request": {"study": "COUNTS", "rules": {"shipped": ["YOGAS"]}}}]}), "dataset")?;
/// let asked = dataset.study_request("s")?;
/// assert_eq!(asked.excluded, [Excluded { id: "b".to_owned(), rating: Rating::X }]);
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "DatasetExcluded"))]
pub struct Excluded {
    /// The record's id.
    pub id: String,
    /// Its rating, worse than the study admits.
    pub rating: Rating,
}

/// What a study reads of a dataset: `research.study`'s request, and the
/// records it admitted and set aside.
///
/// ```
/// # let snapshot = teistro_core::settings::SettingsSnapshot::of(
/// #     &teistro_core::settings::root().validated(teistro_core::settings::ProfileId::new("mine")).unwrap());
/// # let birth = |jd: f64| serde_json::json!({"instant": jd, "latitudeDeg": 27.7,
/// #     "longitudeDeg": 85.3, "utcOffsetSeconds": 20700});
/// # let dataset = teistro_research::dataset::Dataset::from_value(&serde_json::json!({
/// #     "format": "teistro.dataset/1", "name": "a sample",
/// #     "records": [
/// #         {"id": "a", "birth": birth(2447000.25), "rating": "AA", "source": "a register"},
/// #         {"id": "b", "birth": birth(2448000.75), "rating": "X"},
/// #         {"id": "c", "birth": birth(2449000.5), "rating": "A"}],
/// #     "studies": [{"id": "s", "settings": snapshot, "minimumRating": "A",
/// #         "groups": {"a": 0, "c": 1},
/// #         "request": {"study": "COUNTS", "rules": {"shipped": ["YOGAS"]}}}]}), "dataset")?;
/// let asked = dataset.study_request("s")?;
/// assert_eq!((asked.index, asked.study.id.as_str()), (0, "s"));
/// assert_eq!(asked.request["births"].as_array().map(Vec::len), Some(2));
/// # Ok::<(), teistro_core::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct StudyRequest<'d> {
    /// The study.
    pub study: &'d DatasetStudy,
    /// Its index in the dataset, which a refusal is named by.
    pub index: usize,
    /// The request `research.study` reads.
    pub request: Value,
    /// The ids of the records admitted, in the order of its births.
    pub admitted: Vec<String>,
    /// The records set aside by their rating.
    pub excluded: Vec<Excluded>,
}

impl Dataset {
    /// A dataset read strictly from its JSON and checked, every refusal
    /// named by its field under `dataset`
    /// (`03-design/receipts-and-datasets.md` §2.3).
    ///
    /// ```
    /// use teistro_research::dataset::Dataset;
    ///
    /// let refused = Dataset::from_json(r#"{"format": "teistro.dataset/1", "name": "x",
    ///     "records": [], "studies": []}"#).unwrap_err();
    /// assert_eq!(refused.field(), Some("dataset.records"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not a dataset, a key it does not
    /// have, no records, a record id empty or given twice, a birth out of
    /// range, a study id given twice, a `minimumRating` of `X` or `XX`, a
    /// group naming no record, and a request carrying `births`, `subjects`
    /// or `design`; `SCHEMA_VERSION` at `dataset.format` for another
    /// format; what [`SettingsSnapshot::resolved`] refuses of a study's
    /// settings, under `dataset.studies[i].settings`.
    pub fn from_json(text: &str) -> Result<Dataset, Error> {
        Dataset::from_value(&teistro_core::strict::parse(text, DATASET)?, DATASET)
    }

    /// As [`Dataset::from_json`], from a value, refused under `root`.
    ///
    /// ```
    /// use teistro_research::dataset::Dataset;
    ///
    /// let refused = Dataset::from_value(&serde_json::json!({"format": "teistro.dataset/2",
    ///     "name": "x", "records": [], "studies": []}), "studies.dataset").unwrap_err();
    /// assert_eq!(refused.field(), Some("studies.dataset.format"));
    /// ```
    ///
    /// # Errors
    ///
    /// As [`Dataset::from_json`].
    pub fn from_value(value: &Value, root: &str) -> Result<Dataset, Error> {
        let dataset: Dataset = teistro_core::strict::read_value(value, root)?;
        dataset.checked().map_err(|why| why.under(root))?;
        Ok(dataset)
    }

    /// Every rule the types do not hold, named under the dataset's root.
    fn checked(&self) -> Result<(), Error> {
        if self.format != FORMAT {
            return Err(Error::new(
                Status::SchemaVersion,
                format!(
                    "a dataset in `{}`; this build reads `{FORMAT}`",
                    self.format
                ),
            )
            .with_field("format"));
        }
        if self.records.is_empty() {
            return Err(
                Error::invalid_arg("a dataset has records, and this one has none")
                    .with_field("records"),
            );
        }
        let mut ids = BTreeSet::new();
        for (index, record) in self.records.iter().enumerate() {
            record
                .checked(&mut ids)
                .map_err(|why| why.under(&format!("records[{index}]")))?;
        }
        let mut studies = BTreeSet::new();
        for (index, study) in self.studies.iter().enumerate() {
            study
                .checked(&ids, &mut studies)
                .map_err(|why| why.under(&format!("studies[{index}]")))?;
        }
        Ok(())
    }

    /// The study named `id`, and its index.
    ///
    /// ```
    /// # let snapshot = teistro_core::settings::SettingsSnapshot::of(
    /// #     &teistro_core::settings::root().validated(teistro_core::settings::ProfileId::new("mine")).unwrap());
    /// # let birth = |jd: f64| serde_json::json!({"instant": jd, "latitudeDeg": 27.7,
    /// #     "longitudeDeg": 85.3, "utcOffsetSeconds": 20700});
    /// # let dataset = teistro_research::dataset::Dataset::from_value(&serde_json::json!({
    /// #     "format": "teistro.dataset/1", "name": "a sample",
    /// #     "records": [
    /// #         {"id": "a", "birth": birth(2447000.25), "rating": "AA", "source": "a register"},
    /// #         {"id": "b", "birth": birth(2448000.75), "rating": "X"},
    /// #         {"id": "c", "birth": birth(2449000.5), "rating": "A"}],
    /// #     "studies": [{"id": "s", "settings": snapshot, "minimumRating": "A",
    /// #         "groups": {"a": 0, "c": 1},
    /// #         "request": {"study": "COUNTS", "rules": {"shipped": ["YOGAS"]}}}]}), "dataset")?;
    /// let (index, study) = dataset.study("s")?;
    /// assert_eq!((index, study.id.as_str()), (0, "s"));
    /// assert_eq!(dataset.study("t").unwrap_err().field(), Some("study"));
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` at `study` when the dataset has no study by that id.
    pub fn study(&self, id: &str) -> Result<(usize, &DatasetStudy), Error> {
        self.studies
            .iter()
            .enumerate()
            .find(|(_, study)| study.id == id)
            .ok_or_else(|| {
                let ids: Vec<&str> = self.studies.iter().map(|s| s.id.as_str()).collect();
                Error::invalid_arg(format!("the dataset has no study `{id}`"))
                    .with_field("study")
                    .with_hint(format!("its studies are {}", ids.join(", ")))
            })
    }

    /// The `research.study` request the study `id` means: its own request,
    /// with the births of the records it admits in the dataset's order;
    /// for `COUNTS` and `COMPARE` their groups as the design; for `TIMED`
    /// each one's event as a subject's.
    ///
    /// ```
    /// use teistro_research::dataset::Dataset;
    ///
    /// # let snapshot = teistro_core::settings::SettingsSnapshot::of(
    /// #     &teistro_core::settings::root().validated(teistro_core::settings::ProfileId::new("mine")).unwrap());
    /// let birth = |jd: f64| serde_json::json!({"instant": jd, "latitudeDeg": 27.7,
    ///     "longitudeDeg": 85.3, "utcOffsetSeconds": 20700});
    /// let dataset = Dataset::from_value(&serde_json::json!({
    ///     "format": "teistro.dataset/1", "name": "a sample",
    ///     "records": [
    ///         {"id": "a", "birth": birth(2447000.25), "rating": "AA"},
    ///         {"id": "b", "birth": birth(2448000.75), "rating": "C"},
    ///         {"id": "c", "birth": birth(2449000.5), "rating": "A"}],
    ///     "studies": [{"id": "s", "settings": snapshot, "minimumRating": "A",
    ///         "groups": {"a": 0, "c": 1},
    ///         "request": {"study": "COUNTS", "rules": {"shipped": ["YOGAS"]}}}]}), "dataset")?;
    /// let asked = dataset.study_request("s")?;
    /// assert_eq!(asked.admitted, ["a", "c"]);
    /// assert_eq!(asked.excluded[0].id, "b");
    /// assert_eq!(asked.request["design"]["groups"], serde_json::json!([0, 1]));
    /// # Ok::<(), teistro_core::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` at `study` for no study by that id; at
    /// `dataset.studies[i].minimumRating` when it admits no record; at
    /// `dataset.studies[i].groups.<id>` for an admitted record with no
    /// group in a `COUNTS` or `COMPARE` study; at
    /// `dataset.records[j].event` for an admitted record with no event in
    /// a `TIMED` one.
    pub fn study_request(&self, id: &str) -> Result<StudyRequest<'_>, Error> {
        let (index, study) = self.study(id)?;
        let under = |why: Error| why.under(&format!("{DATASET}.studies[{index}]"));
        let (admitted, excluded): (Vec<_>, Vec<_>) = self
            .records
            .iter()
            .enumerate()
            .partition(|(_, record)| record.rating.admitted_by(study.minimum_rating));
        if admitted.is_empty() {
            return Err(under(
                Error::invalid_arg(format!(
                    "no record is rated {:?} or better",
                    study.minimum_rating
                ))
                .with_field("minimumRating"),
            ));
        }
        let mut request = study.request.clone();
        let kind = study.request.get("study").and_then(Value::as_str);
        let births = || -> Vec<Value> {
            admitted
                .iter()
                .map(|(_, record)| serde_json::to_value(record.birth).unwrap_or_default())
                .collect()
        };
        if kind == Some("TIMED") {
            let mut subjects = Vec::with_capacity(admitted.len());
            for (at, record) in &admitted {
                let event = record.event.ok_or_else(|| {
                    Error::invalid_arg(format!(
                        "a timed study reads each record's event, and `{}` has none",
                        record.id
                    ))
                    .with_field(format!("{DATASET}.records[{at}].event"))
                })?;
                subjects.push(serde_json::json!({"birth": record.birth, "event": event}));
            }
            request.insert("subjects".to_owned(), Value::Array(subjects));
        } else {
            request.insert("births".to_owned(), Value::Array(births()));
            if matches!(kind, Some("COUNTS" | "COMPARE")) {
                let groups = study.groups.clone().unwrap_or_default();
                let mut design = Vec::with_capacity(admitted.len());
                for (_, record) in &admitted {
                    let group = groups.get(&record.id).ok_or_else(|| {
                        under(
                            Error::invalid_arg(format!(
                                "a {} study reads each admitted record's group, and `{}` has none",
                                kind.unwrap_or_default(),
                                record.id
                            ))
                            .with_field(format!("groups.{}", record.id)),
                        )
                    })?;
                    design.push(Value::from(*group));
                }
                request.insert("design".to_owned(), serde_json::json!({ "groups": design }));
            }
        }
        Ok(StudyRequest {
            study,
            index,
            request: Value::Object(request),
            admitted: admitted.iter().map(|(_, r)| r.id.clone()).collect(),
            excluded: excluded
                .iter()
                .map(|(_, r)| Excluded {
                    id: r.id.clone(),
                    rating: r.rating,
                })
                .collect(),
        })
    }
}

impl DatasetRecord {
    fn checked(&self, ids: &mut BTreeSet<String>) -> Result<(), Error> {
        if self.id.is_empty() {
            return Err(Error::invalid_arg("a record's id is not empty").with_field("id"));
        }
        if !ids.insert(self.id.clone()) {
            return Err(
                Error::invalid_arg(format!("the record id `{}` is given twice", self.id))
                    .with_field("id"),
            );
        }
        self.birth.checked().map_err(|why| why.under("birth"))?;
        if let Some(event) = self.event {
            JulianDay::<Utc>::try_new(event).map_err(|why| Error::from(why).with_field("event"))?;
        }
        Ok(())
    }
}

impl DatasetBirth {
    /// Each number in range, by the core's own quantities.
    fn checked(&self) -> Result<(), Error> {
        JulianDay::<Utc>::try_new(self.instant)
            .map_err(|why| Error::from(why).with_field("instant"))?;
        Latitude::try_new(self.latitude_deg)
            .map_err(|why| Error::from(why).with_field("latitudeDeg"))?;
        Longitude::try_new(self.longitude_deg)
            .map_err(|why| Error::from(why).with_field("longitudeDeg"))?;
        if let Some(altitude) = self.altitude_m {
            Altitude::try_new(altitude).map_err(|why| Error::from(why).with_field("altitudeM"))?;
        }
        UtcOffset::try_from_seconds(self.utc_offset_seconds)
            .map_err(|why| Error::from(why).with_field("utcOffsetSeconds"))?;
        if let Some(minutes) = self
            .uncertainty_minutes
            .filter(|m| !(m.is_finite() && *m >= 0.0))
        {
            return Err(Error::invalid_arg(format!(
                "a time's uncertainty is a number of minutes, 0 or more, not {minutes}"
            ))
            .with_field("uncertaintyMinutes"));
        }
        Ok(())
    }
}

impl DatasetStudy {
    fn checked(&self, records: &BTreeSet<String>, ids: &mut BTreeSet<String>) -> Result<(), Error> {
        if self.id.is_empty() {
            return Err(Error::invalid_arg("a study's id is not empty").with_field("id"));
        }
        if !ids.insert(self.id.clone()) {
            return Err(
                Error::invalid_arg(format!("the study id `{}` is given twice", self.id))
                    .with_field("id"),
            );
        }
        if self.minimum_rating.rank().is_none() {
            return Err(Error::invalid_arg(format!(
                "a study admits rated times, AA to DD; {:?} grades none",
                self.minimum_rating
            ))
            .with_field("minimumRating"));
        }
        self.settings
            .resolved()
            .map_err(|why| why.under("settings"))?;
        let stray = self
            .groups
            .iter()
            .flat_map(BTreeMap::keys)
            .find(|id| !records.contains(*id));
        if let Some(stray) = stray {
            return Err(
                Error::invalid_arg(format!("the group of `{stray}` names no record"))
                    .with_field(format!("groups.{stray}")),
            );
        }
        if let Some(supplied) = SUPPLIED
            .iter()
            .find(|field| self.request.contains_key(**field))
        {
            return Err(Error::invalid_arg(format!(
                "the dataset supplies `{supplied}`, so a study's request does not carry it"
            ))
            .with_field(format!("request.{supplied}")));
        }
        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use serde_json::json;
    use teistro_core::settings::{ProfileId, root};

    use super::*;

    fn snapshot() -> SettingsSnapshot {
        SettingsSnapshot::of(&root().validated(ProfileId::new("mine")).unwrap())
    }

    fn birth(jd: f64) -> Value {
        json!({"instant": jd, "latitudeDeg": 27.7, "longitudeDeg": 85.3, "utcOffsetSeconds": 20700})
    }

    fn dataset() -> Value {
        json!({
            "format": FORMAT, "name": "a sample", "description": "three births",
            "records": [
                {"id": "a", "birth": birth(2_447_000.25), "rating": "AA", "source": "BC", "event": 2_460_000.5},
                {"id": "b", "birth": birth(2_448_000.75), "rating": "X"},
                {"id": "c", "birth": birth(2_449_000.5), "rating": "B", "event": 2_460_100.5}],
            "studies": [
                {"id": "counts", "settings": snapshot(), "minimumRating": "B",
                 "groups": {"a": 0, "c": 1},
                 "request": {"study": "COUNTS", "rules": {"shipped": ["YOGAS"]}}},
                {"id": "timed", "settings": snapshot(), "minimumRating": "DD",
                 "request": {"study": "TIMED", "rules": {}, "dasha": "VIMSHOTTARI"}},
                {"id": "expected", "settings": snapshot(), "minimumRating": "AA",
                 "request": {"study": "EXPECTED", "rules": {}}}]})
    }

    fn refused(edit: impl FnOnce(&mut Value)) -> Error {
        let mut value = dataset();
        edit(&mut value);
        Dataset::from_value(&value, DATASET).unwrap_err()
    }

    #[test]
    fn a_dataset_reads_back_from_its_canonical_json() {
        let read = Dataset::from_value(&dataset(), DATASET).unwrap();
        let written = teistro_core::envelope::canonical_json(&read);
        assert_eq!(Dataset::from_json(&written).unwrap(), read);
    }

    #[test]
    fn each_rule_is_refused_by_its_field() {
        type Edit = Box<dyn FnOnce(&mut Value)>;
        let cases: [(Edit, &str); 13] = [
            (
                Box::new(|v| v["records"][0]["birth"]["latitudeDeg"] = 97.into()),
                "dataset.records[0].birth.latitudeDeg",
            ),
            (
                Box::new(|v| v["format"] = "teistro.dataset/2".into()),
                "dataset.format",
            ),
            (Box::new(|v| v["records"] = json!([])), "dataset.records"),
            (
                Box::new(|v| v["records"][1]["id"] = "".into()),
                "dataset.records[1].id",
            ),
            (
                Box::new(|v| v["records"][1]["id"] = "a".into()),
                "dataset.records[1].id",
            ),
            (
                Box::new(|v| v["records"][0]["birth"]["longitudeDeg"] = 200.into()),
                "dataset.records[0].birth.longitudeDeg",
            ),
            (
                Box::new(|v| v["records"][0]["birth"]["altitudeM"] = 1e9.into()),
                "dataset.records[0].birth.altitudeM",
            ),
            (
                Box::new(|v| v["records"][0]["birth"]["utcOffsetSeconds"] = 999_999.into()),
                "dataset.records[0].birth.utcOffsetSeconds",
            ),
            (
                Box::new(|v| v["records"][0]["birth"]["uncertaintyMinutes"] = (-1).into()),
                "dataset.records[0].birth.uncertaintyMinutes",
            ),
            (
                Box::new(|v| v["studies"][1]["id"] = "counts".into()),
                "dataset.studies[1].id",
            ),
            (
                Box::new(|v| v["studies"][1]["minimumRating"] = "XX".into()),
                "dataset.studies[1].minimumRating",
            ),
            (
                Box::new(|v| v["studies"][0]["groups"]["z"] = 1.into()),
                "dataset.studies[0].groups.z",
            ),
            (
                Box::new(|v| v["studies"][2]["request"]["births"] = json!([])),
                "dataset.studies[2].request.births",
            ),
        ];
        for (edit, field) in cases {
            assert_eq!(refused(edit).field(), Some(field));
        }
        assert_eq!(
            refused(|v| v["records"][0]["rating"] = "Z".into()).field(),
            Some("dataset.records[0].rating")
        );
        assert_eq!(
            refused(|v| v["studies"][0]["id"] = "".into()).field(),
            Some("dataset.studies[0].id")
        );
        let tampered = refused(|v| v["studies"][0]["settings"]["hash"] = "0".repeat(64).into());
        assert_eq!(tampered.field(), Some("dataset.studies[0].settings.hash"));
    }

    /// JSON holds no NaN, so a birth and an event that are no instant are
    /// built here.
    #[test]
    fn an_instant_that_is_no_number_is_refused_by_its_field() {
        let mut dataset = Dataset::from_value(&dataset(), DATASET).unwrap();
        dataset.records[0].event = Some(f64::NAN);
        assert_eq!(
            dataset.checked().unwrap_err().field(),
            Some("records[0].event")
        );
        dataset.records[0].event = None;
        dataset.records[0].birth.uncertainty_minutes = Some(f64::NAN);
        assert_eq!(
            dataset.checked().unwrap_err().field(),
            Some("records[0].birth.uncertaintyMinutes")
        );
        dataset.records[0].birth.instant = f64::INFINITY;
        assert_eq!(
            dataset.checked().unwrap_err().field(),
            Some("records[0].birth.instant")
        );
    }

    #[test]
    fn a_study_request_is_what_the_dataset_supplies() {
        let dataset = Dataset::from_value(&dataset(), DATASET).unwrap();
        let counts = dataset.study_request("counts").unwrap();
        assert_eq!(counts.index, 0);
        assert_eq!(counts.admitted, ["a", "c"]);
        assert_eq!(
            counts.excluded,
            [Excluded {
                id: "b".into(),
                rating: Rating::X
            }]
        );
        assert_eq!(counts.request["births"][1], birth(2_449_000.5));
        assert_eq!(counts.request["design"], json!({"groups": [0, 1]}));

        let timed = dataset.study_request("timed").unwrap();
        assert_eq!(timed.request["subjects"][0]["event"], 2_460_000.5);
        assert!(timed.request.get("births").is_none());

        let expected = dataset.study_request("expected").unwrap();
        assert_eq!(expected.admitted, ["a"]);
        assert!(expected.request.get("design").is_none());

        assert_eq!(
            dataset.study_request("none").unwrap_err().field(),
            Some("study")
        );
    }

    #[test]
    fn what_a_study_cannot_be_given_is_refused_by_its_field() {
        let mut value = dataset();
        value["studies"][0]["groups"] = json!({"a": 0});
        let dataset = Dataset::from_value(&value, DATASET).unwrap();
        assert_eq!(
            dataset.study_request("counts").unwrap_err().field(),
            Some("dataset.studies[0].groups.c")
        );
        let mut value = self::dataset();
        value["records"][2].as_object_mut().unwrap().remove("event");
        let dataset = Dataset::from_value(&value, DATASET).unwrap();
        assert_eq!(
            dataset.study_request("timed").unwrap_err().field(),
            Some("dataset.records[2].event")
        );
        let mut value = self::dataset();
        value["records"][0]["rating"] = "C".into();
        let dataset = Dataset::from_value(&value, DATASET).unwrap();
        assert_eq!(
            dataset.study_request("expected").unwrap_err().field(),
            Some("dataset.studies[2].minimumRating")
        );
    }
}
