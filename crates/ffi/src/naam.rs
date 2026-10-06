//! Naam milan at the C boundary: two names matched star to star, answered
//! as the `naam` blob (`03-design/matching.md`, C291 to C296).
//!
//! The Ashta Koota and the ten considerations cross in the sections and
//! shapes a chart's match crosses as, written by the chart's own column
//! writers, so every binding reads both through one type and one reader.
#![allow(
    unsafe_code,
    reason = "the C boundary: every block carries a SAFETY comment"
)]

use core::ffi::c_char;

use teistro_core::error::{Error, Status};
use teistro_idl::blob::{ColumnData, Writer};

use crate::blob::TsBlob;
use crate::chart::{MatchingColumns, PoruthamColumns};
use crate::context::TsContext;
use crate::support::{text, with_context, write_plain};

/// Which of the eight letter groups a name begins in, by the animal
/// *Muhurta Chintamani* VI.35 gives it (C295).
///
/// Mirrors `teistro::matching::NameVarga` through an **exhaustive** match.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsNameVarga {
    /// The vowels.
    Garuda = 0,
    /// ka kha ga gha ṅa.
    Cat = 1,
    /// ca cha ja jha ña.
    Lion = 2,
    /// ṭa ṭha ḍa ḍha ṇa.
    Dog = 3,
    /// ta tha da dha na.
    Serpent = 4,
    /// pa pha ba bha ma.
    Rat = 5,
    /// ya ra la va.
    Deer = 6,
    /// śa ṣa sa ha.
    Sheep = 7,
}

impl From<teistro::matching::NameVarga> for TsNameVarga {
    fn from(varga: teistro::matching::NameVarga) -> TsNameVarga {
        use teistro::matching::NameVarga;
        match varga {
            NameVarga::Garuda => TsNameVarga::Garuda,
            NameVarga::Cat => TsNameVarga::Cat,
            NameVarga::Lion => TsNameVarga::Lion,
            NameVarga::Dog => TsNameVarga::Dog,
            NameVarga::Serpent => TsNameVarga::Serpent,
            NameVarga::Rat => TsNameVarga::Rat,
            NameVarga::Deer => TsNameVarga::Deer,
            NameVarga::Sheep => TsNameVarga::Sheep,
        }
    }
}

/// How two names' vargas stand (VI.35).
///
/// Mirrors `teistro::matching::VargaRelation` through an **exhaustive**
/// match.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsVargaRelation {
    /// One varga.
    Same = 0,
    /// Each the 5th from the other, eater and eaten.
    Enemy = 1,
    /// Neither.
    Neutral = 2,
}

impl From<teistro::matching::VargaRelation> for TsVargaRelation {
    fn from(relation: teistro::matching::VargaRelation) -> TsVargaRelation {
        use teistro::matching::VargaRelation;
        match relation {
            VargaRelation::Same => TsVargaRelation::Same,
            VargaRelation::Enemy => TsVargaRelation::Enemy,
            VargaRelation::Neutral => TsVargaRelation::Neutral,
        }
    }
}

/// The naam blob of one match.
fn encode(read: &teistro::NaamMilan) -> Result<Vec<u8>, Error> {
    let schema = crate::schemas::naam();
    let mut writer = Writer::new(&schema);
    let names = [read.bride, read.groom];
    let cell: Vec<u8> = names.iter().map(|name| name.cell).collect();
    let nakshatra: Vec<u16> = names
        .iter()
        .map(|name| name.nakshatra.map_or(0, teistro::catalogue::Nakshatra::id))
        .collect();
    let abhijit: Vec<u8> = names
        .iter()
        .map(|name| u8::from(name.nakshatra.is_none()))
        .collect();
    let quarter: Vec<u8> = names.iter().map(|name| name.quarter).collect();
    let varga: Vec<u8> = names
        .iter()
        .map(|name| TsNameVarga::from(name.varga) as u8)
        .collect();
    let kootas = MatchingColumns::of_kootas([&read.ashta])?;
    let poruthams = PoruthamColumns::of_poruthams([&read.porutham])?;
    let write = || -> Result<Vec<u8>, teistro_idl::blob::BlobError> {
        writer.columns(
            "naam_names",
            names.len(),
            &[
                ColumnData::U8(&cell),
                ColumnData::U16(&nakshatra),
                ColumnData::U8(&abhijit),
                ColumnData::U8(&quarter),
                ColumnData::U8(&varga),
            ],
        )?;
        writer.fixed(
            "naam_varga",
            &[u64::from(TsVargaRelation::from(read.varga.relation) as u8).into()],
        )?;
        kootas.write(&mut writer)?;
        poruthams.write(&mut writer)?;
        writer.finish()
    };
    write().map_err(|error| {
        Error::new(
            Status::Internal,
            format!("the naam blob could not be written: {error}"),
        )
    })
}

/// Matches two names star to star (naam milan) and answers with the
/// `naam` blob: each name's first syllable in the śatapada cakra, the
/// varga koota, and the Ashta Koota and the ten considerations of the two
/// name stars (`03-design/matching.md`, C291 to C296).
///
/// `request_json` is `{"bride", "groom", "rules"}`: the two names, in
/// Devanagari or, when `rules.name.latin` is `IAST`, in IAST, and the
/// `NaamRules` with every field optional. A name the cakra does not read,
/// a Latin name while `latin` refuses, or an Abhijit syllable while
/// `abhijit` refuses is `INVALID_ARG`, named under `naam.bride` or
/// `naam.groom`.
///
/// `api: blob=naam`
///
/// # Safety
///
/// `context` must be a live handle; `request_json` NUL-terminated;
/// `out_blob` valid for a write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ts_naam_milan(
    context: *const TsContext,
    request_json: *const c_char,
    out_blob: *mut TsBlob,
) -> Status {
    with_context(context, |_| {
        // SAFETY: the entry point's contract.
        let asked =
            teistro::NaamRequest::from_json(unsafe { text(request_json, "request_json") }?)?;
        let encoded = encode(&asked.answer()?)?;
        // SAFETY: the entry point's contract.
        unsafe { write_plain(out_blob, "out_blob", TsBlob::from_vec(encoded)) }
    })
}
