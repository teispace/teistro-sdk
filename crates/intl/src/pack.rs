//! The `.tpack` container: one locale, one namespace, a sorted key table
//! over a byte arena, a CRC for integrity and a SHA-256 for provenance,
//! the locale's metadata carried along so a runtime needs nothing else.
//! Reads are zero-copy over the bytes and every offset is bounds-checked;
//! nothing in the file is trusted.
//!
//! Messages are stored as their source text rather than a pre-parsed
//! tree: the spike measured parsing at a microsecond a message, which a
//! per-key cache makes a one-time cost, and source text keeps packs
//! diffable and a third smaller.

use std::collections::BTreeMap;
use std::fmt;

use sha2::{Digest, Sha256};

use crate::source::{Entity, Entry, LocaleSource, Meta, Namespace};

/// The first four bytes of every pack.
pub const MAGIC: [u8; 4] = *b"TPK1";
/// The bundle's magic: one locale, every namespace, the metadata once.
pub const BUNDLE_MAGIC: [u8; 4] = *b"TPB1";
/// The container format version.
pub const FORMAT_VERSION: u16 = 2;

const KIND_MESSAGE: u8 = 1;
const KIND_ENTITY: u8 = 2;
const TABLE_ENTRY: usize = 16;

/// Why bytes are not a pack.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackError(pub String);

impl fmt::Display for PackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for PackError {}

/// Builds the pack of one namespace of a locale.
///
/// # Errors
///
/// A key or value too long for the table, or the locale lacking the
/// namespace.
pub fn build(locale: &LocaleSource, namespace: &str) -> Result<Vec<u8>, PackError> {
    build_with(locale, namespace, true)
}

/// One locale as a bundle: the metadata once, then every namespace as a
/// pack without metadata, each verified on its own; the container's CRC
/// and SHA-256 cover the whole payload.
///
/// # Errors
///
/// A namespace that cannot be packed.
pub fn build_bundle(locale: &LocaleSource) -> Result<Vec<u8>, PackError> {
    let meta = serde_json::to_vec(&locale.meta).map_err(|e| PackError(e.to_string()))?;
    let mut payload = Vec::new();
    payload.extend_from_slice(&offset(meta.len())?.to_le_bytes());
    payload.extend_from_slice(&meta);
    let mut bodies = Vec::new();
    for name in locale.namespaces.keys() {
        let body = build_with(locale, name, false)?;
        push_str(&mut payload, name)?;
        payload.extend_from_slice(&offset(body.len())?.to_le_bytes());
        bodies.push(body);
    }
    for body in bodies {
        payload.extend_from_slice(&body);
    }
    let crc = crc32fast::hash(&payload);
    let sha: [u8; 32] = Sha256::digest(&payload).into();
    let mut out = Vec::with_capacity(64 + payload.len());
    out.extend_from_slice(&BUNDLE_MAGIC);
    out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    push_str(&mut out, &locale.tag)?;
    out.extend_from_slice(&offset(locale.namespaces.len())?.to_le_bytes());
    out.extend_from_slice(&crc.to_le_bytes());
    out.extend_from_slice(&sha);
    out.extend_from_slice(&payload);
    Ok(out)
}

fn build_with(
    locale: &LocaleSource,
    namespace: &str,
    with_meta: bool,
) -> Result<Vec<u8>, PackError> {
    let ns = locale
        .namespaces
        .get(namespace)
        .ok_or_else(|| PackError(format!("{} has no namespace {namespace}", locale.tag)))?;
    let meta = if with_meta {
        serde_json::to_vec(&locale.meta).map_err(|e| PackError(e.to_string()))?
    } else {
        Vec::new()
    };
    let mut arena: Vec<u8> = Vec::new();
    let mut table: Vec<u8> = Vec::with_capacity(ns.entries.len() * TABLE_ENTRY);
    // `BTreeMap` iterates in byte order, which is the order lookups expect.
    for (key, entry) in &ns.entries {
        let key_off = offset(arena.len())?;
        let key_len =
            u16::try_from(key.len()).map_err(|_| PackError(format!("key `{key}` is too long")))?;
        arena.extend_from_slice(key.as_bytes());
        let value_off = offset(arena.len())?;
        let kind = match entry {
            Entry::Message(source) => {
                arena.extend_from_slice(source.as_bytes());
                KIND_MESSAGE
            }
            Entry::Entity(entity) => {
                encode_entity(entity, &mut arena)?;
                KIND_ENTITY
            }
        };
        let value_len = offset(arena.len() - value_off as usize)?;
        table.extend_from_slice(&key_off.to_le_bytes());
        table.extend_from_slice(&key_len.to_le_bytes());
        table.push(kind);
        table.push(0);
        table.extend_from_slice(&value_off.to_le_bytes());
        table.extend_from_slice(&value_len.to_le_bytes());
    }
    let mut body = table;
    body.extend_from_slice(&arena);
    let crc = crc32fast::hash(&body);
    let sha: [u8; 32] = Sha256::digest(&body).into();
    let mut out = Vec::with_capacity(64 + meta.len() + body.len());
    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    push_str(&mut out, &locale.tag)?;
    push_str(&mut out, namespace)?;
    out.extend_from_slice(&offset(ns.entries.len())?.to_le_bytes());
    out.extend_from_slice(&offset(arena.len())?.to_le_bytes());
    out.extend_from_slice(&crc.to_le_bytes());
    out.extend_from_slice(&sha);
    out.extend_from_slice(&offset(meta.len())?.to_le_bytes());
    out.extend_from_slice(&meta);
    out.extend_from_slice(&body);
    Ok(out)
}

fn offset(len: usize) -> Result<u32, PackError> {
    u32::try_from(len).map_err(|_| PackError(String::from("the pack exceeds 4 GiB")))
}

fn push_str(out: &mut Vec<u8>, s: &str) -> Result<(), PackError> {
    let len = u16::try_from(s.len()).map_err(|_| PackError(format!("`{s}` is too long")))?;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(s.as_bytes());
    Ok(())
}

fn push_short(out: &mut Vec<u8>, s: &str) -> Result<(), PackError> {
    let len =
        u8::try_from(s.len()).map_err(|_| PackError(format!("`{s}` is too long for a form")))?;
    out.push(len);
    out.extend_from_slice(s.as_bytes());
    Ok(())
}

fn encode_entity(entity: &Entity, out: &mut Vec<u8>) -> Result<(), PackError> {
    let count =
        u8::try_from(entity.forms.len()).map_err(|_| PackError(String::from("too many forms")))?;
    out.push(count);
    for (form, value) in &entity.forms {
        push_short(out, form)?;
        push_str(out, value)?;
    }
    push_short(out, entity.gender.as_deref().unwrap_or_default())?;
    push_short(out, entity.glyph.as_deref().unwrap_or_default())?;
    Ok(())
}

/// A bounds-checked cursor over bytes.
struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn take(&mut self, len: usize) -> Result<&'a [u8], PackError> {
        let end = self
            .pos
            .checked_add(len)
            .filter(|end| *end <= self.bytes.len())
            .ok_or_else(|| PackError(format!("truncated at byte {}", self.pos)))?;
        let slice = self.bytes.get(self.pos..end).unwrap_or_default();
        self.pos = end;
        Ok(slice)
    }

    fn u8(&mut self) -> Result<u8, PackError> {
        Ok(self.take(1)?.first().copied().unwrap_or_default())
    }

    fn u16(&mut self) -> Result<u16, PackError> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([
            b.first().copied().unwrap_or_default(),
            b.get(1).copied().unwrap_or_default(),
        ]))
    }

    fn u32(&mut self) -> Result<u32, PackError> {
        let b = self.take(4)?;
        let mut array = [0u8; 4];
        array.copy_from_slice(b);
        Ok(u32::from_le_bytes(array))
    }

    fn str(&mut self, len: usize) -> Result<&'a str, PackError> {
        core::str::from_utf8(self.take(len)?)
            .map_err(|_| PackError(format!("invalid UTF-8 at byte {}", self.pos)))
    }

    fn short_str(&mut self) -> Result<&'a str, PackError> {
        let len = self.u8()? as usize;
        self.str(len)
    }

    fn u16_str(&mut self) -> Result<&'a str, PackError> {
        let len = self.u16()? as usize;
        self.str(len)
    }
}

/// A pack over borrowed bytes.
#[derive(Clone, Copy)]
pub struct Pack<'a> {
    locale: &'a str,
    namespace: &'a str,
    meta: &'a [u8],
    table: &'a [u8],
    arena: &'a [u8],
    sha256: &'a [u8],
    entries: usize,
}

impl fmt::Debug for Pack<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pack")
            .field("locale", &self.locale)
            .field("namespace", &self.namespace)
            .field("entries", &self.entries)
            .finish_non_exhaustive()
    }
}

/// One entry read from a pack.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackEntry<'a> {
    /// A message's source text, borrowed.
    Message(&'a str),
    /// An entity, decoded.
    Entity(Entity),
}

impl<'a> Pack<'a> {
    /// Parses and verifies: magic, version, lengths, the checksum, every
    /// table entry inside the arena, keys valid UTF-8 and sorted.
    ///
    /// # Errors
    ///
    /// Anything that is not a well-formed pack.
    pub fn parse(bytes: &'a [u8]) -> Result<Pack<'a>, PackError> {
        let mut cursor = Cursor { bytes, pos: 0 };
        if cursor.take(4)? != MAGIC {
            return Err(PackError(String::from("not a pack: bad magic")));
        }
        let version = cursor.u16()?;
        if version != FORMAT_VERSION {
            return Err(PackError(format!(
                "format version {version}; this reader knows {FORMAT_VERSION}"
            )));
        }
        let _flags = cursor.u16()?;
        let locale = cursor.u16_str()?;
        let namespace = cursor.u16_str()?;
        let entries = cursor.u32()? as usize;
        let arena_len = cursor.u32()? as usize;
        let crc = cursor.u32()?;
        let sha256 = cursor.take(32)?;
        let meta_len = cursor.u32()? as usize;
        let meta = cursor.take(meta_len)?;
        let table_len = entries
            .checked_mul(TABLE_ENTRY)
            .ok_or_else(|| PackError(String::from("entry count overflows")))?;
        let body_start = cursor.pos;
        let table = cursor.take(table_len)?;
        let arena = cursor.take(arena_len)?;
        if cursor.pos != bytes.len() {
            return Err(PackError(String::from("trailing bytes after the arena")));
        }
        let body = bytes.get(body_start..).unwrap_or_default();
        if crc32fast::hash(body) != crc {
            return Err(PackError(String::from("checksum mismatch")));
        }
        let pack = Pack {
            locale,
            namespace,
            meta,
            table,
            arena,
            sha256,
            entries,
        };
        // Keys are checked here; values are checked as they are read, the
        // checksum having already covered the bytes.
        let mut previous: Option<&str> = None;
        for index in 0..entries {
            let key = pack.key_at(index)?;
            if previous.is_some_and(|p| p >= key) {
                return Err(PackError(format!("keys are not sorted at `{key}`")));
            }
            previous = Some(key);
        }
        Ok(pack)
    }

    /// The locale tag.
    #[must_use]
    pub fn locale(&self) -> &'a str {
        self.locale
    }

    /// The namespace name.
    #[must_use]
    pub fn namespace(&self) -> &'a str {
        self.namespace
    }

    /// The number of entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries
    }

    /// Whether the pack is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries == 0
    }

    /// The SHA-256 of the key table and arena, lower-case hex.
    #[must_use]
    pub fn content_sha256(&self) -> String {
        hex(self.sha256)
    }

    /// The locale metadata carried in the pack.
    ///
    /// # Errors
    ///
    /// Metadata that does not parse, or a pack that carries none because
    /// its bundle does.
    pub fn meta(&self) -> Result<Meta, PackError> {
        if self.meta.is_empty() {
            return Err(PackError(String::from(
                "this pack carries no metadata; its bundle does",
            )));
        }
        serde_json::from_slice(self.meta).map_err(|e| PackError(format!("metadata: {e}")))
    }

    /// Whether the pack carries its locale's metadata.
    #[must_use]
    pub fn has_meta(&self) -> bool {
        !self.meta.is_empty()
    }

    /// The key of a table row, without decoding its value.
    fn key_at(&self, index: usize) -> Result<&'a str, PackError> {
        let row = self
            .table
            .get(index * TABLE_ENTRY..(index + 1) * TABLE_ENTRY)
            .ok_or_else(|| PackError(format!("no table row {index}")))?;
        let mut cursor = Cursor { bytes: row, pos: 0 };
        let key_off = cursor.u32()? as usize;
        let key_len = cursor.u16()? as usize;
        let mut keys = Cursor {
            bytes: self.arena,
            pos: key_off,
        };
        keys.str(key_len)
    }

    fn entry_at(&self, index: usize) -> Result<(&'a str, PackEntry<'a>), PackError> {
        let row = self
            .table
            .get(index * TABLE_ENTRY..(index + 1) * TABLE_ENTRY)
            .ok_or_else(|| PackError(format!("no table row {index}")))?;
        let mut cursor = Cursor { bytes: row, pos: 0 };
        let key_off = cursor.u32()? as usize;
        let key_len = cursor.u16()? as usize;
        let kind = cursor.u8()?;
        let _pad = cursor.u8()?;
        let value_off = cursor.u32()? as usize;
        let value_len = cursor.u32()? as usize;
        let mut keys = Cursor {
            bytes: self.arena,
            pos: key_off,
        };
        let key = keys.str(key_len)?;
        let mut values = Cursor {
            bytes: self.arena,
            pos: value_off,
        };
        let value = values.take(value_len)?;
        let entry = match kind {
            KIND_MESSAGE => PackEntry::Message(
                core::str::from_utf8(value)
                    .map_err(|_| PackError(format!("`{key}`: invalid UTF-8")))?,
            ),
            KIND_ENTITY => PackEntry::Entity(decode_entity(value)?),
            other => return Err(PackError(format!("`{key}`: unknown entry kind {other}"))),
        };
        Ok((key, entry))
    }

    /// The entry at a key, by binary search over the sorted table.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<PackEntry<'a>> {
        let mut low = 0usize;
        let mut high = self.entries;
        while low < high {
            let mid = low + (high - low) / 2;
            match self.key_at(mid).ok()?.cmp(key) {
                core::cmp::Ordering::Less => low = mid + 1,
                core::cmp::Ordering::Greater => high = mid,
                core::cmp::Ordering::Equal => return self.entry_at(mid).ok().map(|(_, e)| e),
            }
        }
        None
    }

    /// Every entry, in key order.
    pub fn iter(&self) -> impl Iterator<Item = (&'a str, PackEntry<'a>)> + '_ {
        (0..self.entries).filter_map(move |i| self.entry_at(i).ok())
    }

    /// The pack's namespace as a source namespace, decoded, in key order.
    #[must_use]
    pub fn to_namespace(&self) -> Namespace {
        let mut namespace = Namespace::default();
        for (key, entry) in self.iter() {
            let entry = match entry {
                PackEntry::Message(source) => Entry::Message(source.to_string()),
                PackEntry::Entity(entity) => Entry::Entity(entity),
            };
            namespace.insert(key.to_string(), entry);
        }
        namespace
    }
}

fn decode_entity(bytes: &[u8]) -> Result<Entity, PackError> {
    let mut cursor = Cursor { bytes, pos: 0 };
    let count = cursor.u8()?;
    let mut forms = BTreeMap::new();
    for _ in 0..count {
        let form = cursor.short_str()?.to_string();
        let value = cursor.u16_str()?.to_string();
        forms.insert(form, value);
    }
    let gender = cursor.short_str()?;
    let glyph = cursor.short_str()?;
    Ok(Entity {
        forms,
        gender: (!gender.is_empty()).then(|| gender.to_string()),
        glyph: (!glyph.is_empty()).then(|| glyph.to_string()),
    })
}

/// A parsed bundle: one locale's metadata and its namespaces as packs.
pub struct Bundle<'a> {
    locale: &'a str,
    meta: &'a [u8],
    packs: Vec<Pack<'a>>,
    sha256: &'a [u8],
}

impl fmt::Debug for Bundle<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Bundle")
            .field("locale", &self.locale)
            .field(
                "namespaces",
                &self.packs.iter().map(Pack::namespace).collect::<Vec<_>>(),
            )
            .finish_non_exhaustive()
    }
}

impl<'a> Bundle<'a> {
    /// Parses and verifies a bundle: magic, version, the container's
    /// checksum, then every embedded pack on its own terms.
    ///
    /// # Errors
    ///
    /// Anything that is not a well-formed bundle.
    pub fn parse(bytes: &'a [u8]) -> Result<Bundle<'a>, PackError> {
        let mut cursor = Cursor { bytes, pos: 0 };
        if cursor.take(4)? != BUNDLE_MAGIC {
            return Err(PackError(String::from("not a bundle: bad magic")));
        }
        let version = cursor.u16()?;
        if version != FORMAT_VERSION {
            return Err(PackError(format!(
                "format version {version}; this reader knows {FORMAT_VERSION}"
            )));
        }
        let _flags = cursor.u16()?;
        let locale = cursor.u16_str()?;
        let count = cursor.u32()? as usize;
        let crc = cursor.u32()?;
        let sha256 = cursor.take(32)?;
        let payload = bytes.get(cursor.pos..).unwrap_or_default();
        if crc32fast::hash(payload) != crc {
            return Err(PackError(String::from("bundle checksum mismatch")));
        }
        let meta_len = cursor.u32()? as usize;
        let meta = cursor.take(meta_len)?;
        let mut index = Vec::with_capacity(count);
        for _ in 0..count {
            let name = cursor.u16_str()?;
            let len = cursor.u32()? as usize;
            index.push((name, len));
        }
        let mut packs = Vec::with_capacity(count);
        for (name, len) in index {
            let body = cursor.take(len)?;
            let pack = Pack::parse(body)?;
            if pack.namespace() != name || pack.locale() != locale {
                return Err(PackError(format!(
                    "the bundle's index names {locale} {name}, the pack says {} {}",
                    pack.locale(),
                    pack.namespace()
                )));
            }
            packs.push(pack);
        }
        if cursor.pos != bytes.len() {
            return Err(PackError(String::from(
                "trailing bytes after the last pack",
            )));
        }
        Ok(Bundle {
            locale,
            meta,
            packs,
            sha256,
        })
    }

    /// The locale tag.
    #[must_use]
    pub fn locale(&self) -> &'a str {
        self.locale
    }

    /// The locale metadata, carried once.
    ///
    /// # Errors
    ///
    /// Metadata that does not parse.
    pub fn meta(&self) -> Result<Meta, PackError> {
        serde_json::from_slice(self.meta).map_err(|e| PackError(format!("metadata: {e}")))
    }

    /// The namespaces' packs, in name order.
    #[must_use]
    pub fn packs(&self) -> &[Pack<'a>] {
        &self.packs
    }

    /// One namespace's pack.
    #[must_use]
    pub fn pack(&self, namespace: &str) -> Option<&Pack<'a>> {
        self.packs.iter().find(|p| p.namespace() == namespace)
    }

    /// The SHA-256 of the payload, lower-case hex, for the provenance
    /// envelope.
    #[must_use]
    pub fn content_sha256(&self) -> String {
        hex(self.sha256)
    }
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::with_capacity(64), |mut out, b| {
        let _ = write!(out, "{b:02x}");
        out
    })
}

/// Locale sources rebuilt from packs and bundles, grouped by locale: a
/// bundle brings its metadata and every namespace; a pack brings one
/// namespace and its metadata, which a locale takes from the first file
/// that carries it. Entries come back in key order because a pack does
/// not carry source order.
///
/// # Errors
///
/// A file that does not parse, or a pack without metadata whose locale no
/// other file described.
pub fn locales_from_packs(files: &[&[u8]]) -> Result<BTreeMap<String, LocaleSource>, PackError> {
    let mut locales: BTreeMap<String, LocaleSource> = BTreeMap::new();
    for bytes in files {
        if bytes.starts_with(&BUNDLE_MAGIC) {
            let bundle = Bundle::parse(bytes)?;
            let locale = locale_entry(&mut locales, bundle.locale(), || bundle.meta())?;
            for pack in bundle.packs() {
                locale
                    .namespaces
                    .insert(pack.namespace().to_string(), pack.to_namespace());
            }
        } else {
            let pack = Pack::parse(bytes)?;
            let locale = locale_entry(&mut locales, pack.locale(), || pack.meta())?;
            locale
                .namespaces
                .insert(pack.namespace().to_string(), pack.to_namespace());
        }
    }
    Ok(locales)
}

/// The locale being rebuilt, created from the file's metadata on first
/// sight.
fn locale_entry<'l>(
    locales: &'l mut BTreeMap<String, LocaleSource>,
    tag: &str,
    meta: impl FnOnce() -> Result<Meta, PackError>,
) -> Result<&'l mut LocaleSource, PackError> {
    if !locales.contains_key(tag) {
        let meta = meta()?;
        locales.insert(
            tag.to_string(),
            LocaleSource {
                tag: tag.to_string(),
                meta,
                namespaces: BTreeMap::new(),
            },
        );
    }
    locales
        .get_mut(tag)
        .ok_or_else(|| PackError(format!("no locale {tag}")))
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;
    use crate::render::{Intl, Params, Value, params};
    use crate::source::{Tree, sdk_root};

    fn tree() -> Tree {
        Tree::load(&sdk_root()).unwrap_or_else(|e| panic!("{e}"))
    }

    #[test]
    fn every_namespace_round_trips_through_a_pack() {
        let tree = tree();
        for locale in tree.locales.values() {
            for (name, namespace) in &locale.namespaces {
                let bytes = build(locale, name).unwrap_or_else(|e| panic!("{e}"));
                let pack = Pack::parse(&bytes).unwrap_or_else(|e| panic!("{e}"));
                assert_eq!(pack.locale(), locale.tag);
                assert_eq!(pack.namespace(), name);
                assert_eq!(pack.len(), namespace.entries.len());
                assert_eq!(pack.to_namespace().entries, namespace.entries);
                assert_eq!(pack.meta().unwrap_or_else(|e| panic!("{e}")), locale.meta);
                for (key, entry) in &namespace.entries {
                    let found = pack.get(key).unwrap_or_else(|| panic!("{key}"));
                    match (entry, found) {
                        (Entry::Message(a), PackEntry::Message(b)) => assert_eq!(a, b),
                        (Entry::Entity(a), PackEntry::Entity(b)) => assert_eq!(*a, b),
                        (a, b) => panic!("{key}: {a:?} became {b:?}"),
                    }
                }
                assert_eq!(pack.get("nothing.here"), None);
                assert_eq!(pack.content_sha256().len(), 64);
            }
        }
    }

    #[test]
    fn a_bundle_carries_the_metadata_once_and_every_namespace() {
        let tree = tree();
        let nepali = tree
            .locales
            .get("ne-Deva-NP")
            .unwrap_or_else(|| panic!("the Nepali locale"));
        let bytes = build_bundle(nepali).unwrap_or_else(|e| panic!("{e}"));
        let bundle = Bundle::parse(&bytes).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(bundle.locale(), "ne-Deva-NP");
        assert_eq!(bundle.meta().unwrap_or_else(|e| panic!("{e}")), nepali.meta);
        assert_eq!(bundle.packs().len(), nepali.namespaces.len());
        assert_eq!(bundle.content_sha256().len(), 64);
        for (name, namespace) in &nepali.namespaces {
            let pack = bundle.pack(name).unwrap_or_else(|| panic!("{name}"));
            assert!(
                !pack.has_meta(),
                "a bundled pack carries no metadata of its own"
            );
            assert!(pack.meta().is_err_and(|e| e.0.contains("bundle")));
            assert_eq!(pack.to_namespace().entries, namespace.entries);
        }
        assert!(bundle.pack("sdk.nowhere").is_none());
        // Smaller than the separate packs: the metadata is paid once.
        let separate: usize = nepali
            .namespaces
            .keys()
            .map(|name| build(nepali, name).unwrap_or_else(|e| panic!("{e}")).len())
            .sum();
        assert!(bytes.len() < separate, "{} against {separate}", bytes.len());
        // The rebuilt locale is the source locale; a bundled pack on its own
        // is refused for want of metadata.
        let locales = locales_from_packs(&[&bytes]).unwrap_or_else(|e| panic!("{e}"));
        let rebuilt = locales
            .get("ne-Deva-NP")
            .unwrap_or_else(|| panic!("the rebuilt locale"));
        assert_eq!(rebuilt.meta, nepali.meta);
        for (name, namespace) in &nepali.namespaces {
            assert_eq!(
                rebuilt.namespaces.get(name).map(|n| &n.entries),
                Some(&namespace.entries)
            );
        }
        let alone = build_with(nepali, "sdk.reason", false).unwrap_or_else(|e| panic!("{e}"));
        assert!(locales_from_packs(&[&alone]).is_err_and(|e| e.0.contains("bundle")));
        // Corruption is caught at the container before any pack is read.
        let mut flipped = bytes.clone();
        if let Some(last) = flipped.last_mut() {
            *last ^= 0x55;
        }
        assert!(Bundle::parse(&flipped).is_err_and(|e| e.0.contains("checksum")));
        for cut in [0, 3, 10, 40, bytes.len() / 2, bytes.len() - 1] {
            assert!(Bundle::parse(&bytes[..cut]).is_err(), "cut at {cut}");
        }
        let pack_bytes = build(nepali, "sdk.reason").unwrap_or_else(|e| panic!("{e}"));
        assert!(Bundle::parse(&pack_bytes).is_err_and(|e| e.0.contains("magic")));
    }

    #[test]
    fn corruption_and_truncation_are_refused() {
        let tree = tree();
        let base = tree.base().unwrap_or_else(|| panic!("base"));
        let bytes = build(base, "sdk.reason").unwrap_or_else(|e| panic!("{e}"));
        let mut flipped = bytes.clone();
        if let Some(last) = flipped.last_mut() {
            *last ^= 0x55;
        }
        assert!(Pack::parse(&flipped).is_err_and(|e| e.0.contains("checksum")));
        for cut in [0, 3, 10, 40, bytes.len() / 2, bytes.len() - 1] {
            assert!(Pack::parse(&bytes[..cut]).is_err(), "cut at {cut}");
        }
        let mut wrong_magic = bytes.clone();
        wrong_magic[0] = b'X';
        assert!(Pack::parse(&wrong_magic).is_err_and(|e| e.0.contains("magic")));
        let mut extended = bytes.clone();
        extended.push(0);
        assert!(Pack::parse(&extended).is_err_and(|e| e.0.contains("trailing")));
    }

    #[test]
    fn an_engine_over_packs_renders_the_same_bits_as_one_over_sources() {
        let tree = tree();
        let mut packs: Vec<Vec<u8>> = Vec::new();
        for locale in tree.locales.values() {
            for name in locale.namespaces.keys() {
                packs.push(build(locale, name).unwrap_or_else(|e| panic!("{e}")));
            }
        }
        let borrowed: Vec<&[u8]> = packs.iter().map(Vec::as_slice).collect();
        let locales = locales_from_packs(&borrowed).unwrap_or_else(|e| panic!("{e}"));
        for (tag, locale) in &tree.locales {
            let rebuilt = locales.get(tag).unwrap_or_else(|| panic!("{tag}"));
            assert_eq!(rebuilt.meta, locale.meta);
            for (name, namespace) in &locale.namespaces {
                assert_eq!(
                    rebuilt.namespaces.get(name).map(|n| &n.entries),
                    Some(&namespace.entries)
                );
            }
        }
        let mut from_packs = Intl::new(locales).unwrap_or_else(|e| panic!("{e}"));
        let mut from_sources = Intl::from_tree(&tree).unwrap_or_else(|e| panic!("{e}"));
        for locale in ["en-Latn", "ne-Deva-NP"] {
            from_packs
                .set_locale(locale)
                .unwrap_or_else(|e| panic!("{e}"));
            from_sources
                .set_locale(locale)
                .unwrap_or_else(|e| panic!("{e}"));
            let p: Params = params([
                ("graha", Value::entity("graha.MARS")),
                ("bhava", Value::Int(3)),
                ("longitude", Value::Num(222.5763)),
            ]);
            for key in [
                "sdk.reason.grahaInBhava",
                "sdk.reason.grahaAt",
                "sdk.reason.welcome",
            ] {
                assert_eq!(
                    from_packs.render(key, &p),
                    from_sources.render(key, &p),
                    "{locale} {key}"
                );
            }
        }
    }
}
