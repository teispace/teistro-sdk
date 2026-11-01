//! A festival pack read as a catalogue (`03-design/festival-metadata.md`
//! §3, SR-02): every rule it holds, the form it is reckoned by and what it
//! is, so a consumer sees the values it can filter on and the fields no
//! source gives.

use serde::{Deserialize, Serialize};
use teistro_core::error::Error;
use teistro_panchanga::festival::{FestivalMeta, PublishedEntry, SubjectTo};

use super::AlmanacArea;
use crate::festival_request::{FestivalPack, pack};

/// The field a pack listing's refusals are named under.
const PACK: &str = "pack";

/// A pack's name, as a binding writes the request.
///
/// ```
/// use teistro::{FestivalPack, PackRequest};
///
/// let asked = PackRequest::from_json(r#"{"pack": "INDIA_CENTRAL"}"#)?;
/// assert_eq!(asked.pack, FestivalPack::IndiaCentral);
/// assert_eq!(PackRequest::from_json(r#"{"pack": "X"}"#).unwrap_err().field(), Some("pack.pack"));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackRequest {
    /// The pack.
    pub pack: FestivalPack,
}

/// [`PackRequest`] as the bindings write it.
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "PackRequest"))]
pub(crate) struct PackAsked {
    /// A pack the SDK ships, by name.
    pack: String,
}

impl PackRequest {
    /// The request a binding writes, read and checked.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming the key under `pack` that is not JSON of its
    /// kind, or a name no pack has.
    pub fn from_json(text: &str) -> Result<PackRequest, Error> {
        let asked: PackAsked = teistro_core::strict::read(text, PACK)?;
        Ok(PackRequest {
            pack: pack(&asked.pack, "pack.pack")?,
        })
    }
}

/// How a pack's row finds its day.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PackForm {
    /// A tithi or nakshatra at a time of day.
    Lunar,
    /// An Ekadashi fast.
    Ekadashi,
    /// A count of days from another rule's.
    Following,
    /// A date of a calendar, or the Sun's entry.
    Dated,
    /// No rule: the days a publisher printed, for the years it printed them.
    Published,
}

/// One rule of a pack.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PackRow {
    /// Its key, as its days are named.
    pub key: String,
    /// How its day is found.
    pub form: PackForm,
    /// Where its day comes from.
    pub source: String,
    /// What it is.
    #[serde(default, skip_serializing_if = "FestivalMeta::is_empty")]
    pub meta: FestivalMeta,
    /// The fields of `meta` no source gives, by name: `kind`, `tradition`,
    /// `communities`, `regions`, `fast`, and `fast.kind`, `fast.start` or
    /// `fast.end` when no reading of the fast gives it.
    pub unsourced: Vec<String>,
    /// For a published row, what may move its days.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject_to: Option<SubjectTo>,
    /// For a published row, the last year with a day: a later year holds
    /// none, and is not reckoned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_year: Option<i32>,
}

impl PackRow {
    fn of(key: &str, form: PackForm, source: &str, meta: &FestivalMeta) -> PackRow {
        PackRow {
            key: key.to_owned(),
            form,
            source: source.to_owned(),
            meta: meta.clone(),
            unsourced: meta.unsourced().into_iter().map(str::to_owned).collect(),
            subject_to: None,
            last_year: None,
        }
    }
}

/// A pack's rules.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PackListing {
    /// The pack's name.
    pub pack: String,
    /// Its rows, by key.
    pub rules: Vec<PackRow>,
}

impl FestivalPack {
    /// Its rows, by key.
    ///
    /// ```
    /// use teistro::{FestivalPack, PackForm};
    ///
    /// let listed = FestivalPack::IndiaCentral.listing();
    /// let eid = listed.rules.iter().find(|row| row.key == "EID_AL_FITR").ok_or("listed")?;
    /// assert_eq!((eid.form, eid.last_year), (PackForm::Published, Some(2027)));
    /// assert!(eid.subject_to.is_some());
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn listing(self) -> PackListing {
        let lunar = self.rules();
        let fasts = self.ekadashis();
        let counted = self.following();
        let dates = self.dated();
        let mut rules: Vec<PackRow> = lunar
            .iter()
            .map(|r| PackRow::of(&r.key, PackForm::Lunar, &r.source, &r.meta))
            .chain(
                fasts
                    .iter()
                    .map(|r| PackRow::of(&r.key, PackForm::Ekadashi, &r.source, &r.meta)),
            )
            .chain(
                counted
                    .iter()
                    .map(|r| PackRow::of(&r.key, PackForm::Following, &r.source, &r.meta)),
            )
            .chain(
                dates
                    .iter()
                    .map(|r| PackRow::of(&r.key, PackForm::Dated, &r.source, &r.meta)),
            )
            .collect();
        for set in self.published() {
            for entry in &set.entries {
                let PublishedEntry::Add {
                    key,
                    day,
                    subject_to,
                    meta,
                    ..
                } = entry
                else {
                    continue;
                };
                if let Some(row) = rules.iter_mut().find(|row| row.key == *key) {
                    row.last_year = row.last_year.max(Some(day.year));
                    row.subject_to = row.subject_to.or(*subject_to);
                } else {
                    let mut row = PackRow::of(key, PackForm::Published, &set.publisher, meta);
                    row.last_year = Some(day.year);
                    row.subject_to = *subject_to;
                    rules.push(row);
                }
            }
        }
        rules.sort_by(|a, b| a.key.cmp(&b.key));
        PackListing {
            pack: self.key().to_owned(),
            rules,
        }
    }
}

impl AlmanacArea<'_> {
    /// Every rule of a pack with what it is (`03-design/festival-metadata.md`
    /// §3): nothing is reckoned, so a context without an ephemeris answers.
    ///
    /// ```
    /// use teistro::{Context, FestivalPack, PackRequest};
    ///
    /// let sdk = Context::builder().build().map_err(|_| "a context")?;
    /// let listed = sdk.almanac().festival_pack(&PackRequest { pack: FestivalPack::NepalPanchanga });
    /// let row = listed.rules.iter().find(|row| row.key == "GAI_JATRA").ok_or("listed")?;
    /// assert_eq!(row.meta.communities, ["NEWAR"]);
    /// assert_eq!(row.unsourced, ["tradition", "fast"]);
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn festival_pack(self, request: &PackRequest) -> PackListing {
        request.pack.listing()
    }
}
