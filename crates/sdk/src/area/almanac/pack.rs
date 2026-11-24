//! A festival pack read as a catalogue (`03-design/festival-metadata.md`
//! §3, SR-02): every rule it holds, the form it is reckoned by and what it
//! is, so a consumer sees the values it can filter on and the fields no
//! source gives.

use serde::{Deserialize, Serialize};
use teistro_core::error::Error;
use teistro_panchanga::festival::{
    FestivalMeta, Holiday, HolidaySet, PublishedEntry, Significance, SubjectTo,
};

use super::AlmanacArea;
use super::about::FestivalAbout;
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
/// assert!(!asked.about);
/// assert!(PackRequest::from_json(r#"{"pack": "NEPAL", "about": true}"#)?.about);
/// assert_eq!(PackRequest::from_json(r#"{"pack": "X"}"#).unwrap_err().field(), Some("pack.pack"));
/// # Ok::<(), teistro::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackRequest {
    /// The pack.
    pub pack: FestivalPack,
    /// Whether to describe each festival and group the SDK describes
    /// (`03-design/festival-about.md`, SR-153); false leaves the listing
    /// as it was.
    pub about: bool,
}

/// [`PackRequest`] as the bindings write it.
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "PackRequest"))]
pub(crate) struct PackAsked {
    /// A pack the SDK ships, by name.
    pack: String,
    /// Whether to describe each festival: what it is, how it is kept and
    /// who keeps it where.
    #[serde(default, skip_serializing_if = "core::ops::Not::not")]
    about: bool,
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
            about: asked.about,
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
    /// Its tier among a day's observances: the one `meta` is marked with,
    /// else what its fields say (`03-design/festival-metadata.md` §7).
    pub significance: Significance,
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
            significance: meta.tier(),
            subject_to: None,
            last_year: None,
        }
    }
}

/// A pack's rules.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PackListing {
    /// The pack's name.
    pub pack: String,
    /// Its rows, by key.
    pub rules: Vec<PackRow>,
    /// The holidays a government's notices give the pack's country, year
    /// by year, each year's in its notice's order
    /// (`03-design/nepal-gazette-holidays.md`): each names its year and its
    /// festivals by key, kept apart from the notice's words.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub holidays: Vec<Holiday>,
    /// Asked with `about`, each of the pack's rules and groups the SDK
    /// describes, its rules first, by key (`03-design/festival-about.md`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub about: Vec<FestivalAbout>,
    /// Asked with `about`, the keys of the pack's rules and groups the SDK
    /// does not describe yet, in the same order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unwritten: Vec<String>,
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
            holidays: self
                .holiday_sets()
                .iter()
                .flat_map(|set| set.holidays())
                .collect(),
            about: Vec::new(),
            unwritten: Vec::new(),
        }
    }

    /// Its listing with each rule and group described: the rows' own meta,
    /// a group's its main member's (`03-design/festival-about.md` §4).
    fn described(self) -> PackListing {
        let mut listed = self.listing();
        let groups = self.groups();
        let subjects = listed
            .rules
            .iter()
            .map(|row| (row.key.as_str(), &row.meta))
            .chain(groups.iter().filter_map(|group| {
                let main = listed.rules.iter().find(|row| row.key == group.main)?;
                Some((group.key.as_str(), &main.meta))
            }));
        let mut about: Vec<FestivalAbout> = Vec::new();
        let mut unwritten: Vec<String> = Vec::new();
        for (key, meta) in subjects {
            if about.iter().any(|said| said.key == key) || unwritten.iter().any(|k| k == key) {
                continue;
            }
            match FestivalAbout::of(key, meta) {
                Some(said) => about.push(said),
                None => unwritten.push(key.to_owned()),
            }
        }
        listed.about = about;
        listed.unwritten = unwritten;
        listed
    }

    /// The holiday notices of the pack's country: Nepal's gazette, every
    /// year's, for Nepal's two packs, none for a pack whose country's notice is not
    /// in hand.
    ///
    /// ```
    /// use teistro::FestivalPack;
    /// use teistro::festival::HolidaySet;
    ///
    /// assert_eq!(FestivalPack::NepalPanchanga.holiday_sets(), [HolidaySet::NepalGazette]);
    /// assert!(FestivalPack::IndiaCentral.holiday_sets().is_empty());
    /// ```
    #[must_use]
    pub const fn holiday_sets(self) -> &'static [HolidaySet] {
        match self {
            FestivalPack::Nepal | FestivalPack::NepalPanchanga => &[HolidaySet::NepalGazette],
            _ => &[],
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
    /// let listed = sdk.almanac().festival_pack(&PackRequest { pack: FestivalPack::NepalPanchanga, about: false });
    /// let row = listed.rules.iter().find(|row| row.key == "GAI_JATRA").ok_or("listed")?;
    /// assert_eq!(row.meta.communities, ["NEWAR"]);
    /// assert_eq!(row.unsourced, ["tradition", "fast"]);
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn festival_pack(self, request: &PackRequest) -> PackListing {
        if request.about {
            request.pack.described()
        } else {
            request.pack.listing()
        }
    }
}
