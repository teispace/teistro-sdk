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
///
/// ```
/// # use teistro_intl::sdk_root;
/// # use teistro_intl::source::Tree;
/// #
/// # let tree = Tree::load(&sdk_root())?;
/// # let nepali = &tree.locales["ne-Deva-NP"];
///
/// use teistro_intl::pack::{self, MAGIC};
///
/// assert!(pack::build(nepali, "sdk.reason")?.starts_with(&MAGIC));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub const MAGIC: [u8; 4] = *b"TPK1";
/// The bundle's magic: one locale, every namespace, the metadata once.
///
/// ```
/// # use teistro_intl::sdk_root;
/// # use teistro_intl::source::Tree;
/// #
/// # let tree = Tree::load(&sdk_root())?;
/// # let nepali = &tree.locales["ne-Deva-NP"];
///
/// use teistro_intl::pack::{self, BUNDLE_MAGIC};
///
/// assert!(pack::build_bundle(nepali)?.starts_with(&BUNDLE_MAGIC));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub const BUNDLE_MAGIC: [u8; 4] = *b"TPB1";
/// The container format version.
///
/// ```
/// # use teistro_intl::sdk_root;
/// # use teistro_intl::source::Tree;
/// #
/// # let tree = Tree::load(&sdk_root())?;
/// # let nepali = &tree.locales["ne-Deva-NP"];
///
/// use teistro_intl::pack::{self, FORMAT_VERSION, Pack};
///
/// // A pack of another version is refused.
/// let mut bytes = pack::build(nepali, "sdk.reason")?;
/// assert_eq!(u16::from_le_bytes([bytes[4], bytes[5]]), FORMAT_VERSION);
/// bytes[4] = bytes[4].wrapping_add(1);
/// assert!(Pack::parse(&bytes).is_err());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub const FORMAT_VERSION: u16 = 3;

/// Where a file's CRC32 sits, by magic: a pack's after its magic,
/// version, flags, locale, namespace, entry count and arena length; a
/// bundle's after its magic, version, flags, locale and namespace count.
/// The CRC covers every other byte of the file (version 3; version 2's
/// covered the body only, so a damaged locale tag or metadata loaded).
fn checksum(bytes: &[u8], at: usize) -> u32 {
    let mut crc = crc32fast::Hasher::new();
    crc.update(bytes.get(..at).unwrap_or_default());
    crc.update(bytes.get(at + 4..).unwrap_or_default());
    crc.finalize()
}

/// Writes `bytes`' CRC32 at `at`, where the writer left four bytes.
fn seal(bytes: &mut [u8], at: usize) {
    let crc = checksum(bytes, at).to_le_bytes();
    if let Some(slot) = bytes.get_mut(at..at + 4) {
        slot.copy_from_slice(&crc);
    }
}

/// Reads a file's flags, which no version yet sets.
fn no_flags(cursor: &mut Cursor<'_>) -> Result<(), PackError> {
    match cursor.u16()? {
        0 => Ok(()),
        flags => Err(PackError(format!(
            "flags {flags:#x}, which this reader does not know"
        ))),
    }
}

/// The fewest bytes a bundle's index entry takes: a `u16` name length
/// and a `u32` pack length.
const INDEX_ENTRY_MIN: usize = 6;

const KIND_MESSAGE: u8 = 1;
const KIND_ENTITY: u8 = 2;
const TABLE_ENTRY: usize = 16;

/// Why bytes are not a pack.
///
/// ```
/// use teistro_intl::pack::Pack;
///
/// let refused = Pack::parse(b"not a pack").unwrap_err();
/// assert!(!refused.0.is_empty());
/// assert_eq!(refused.to_string(), refused.0);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackError(pub String);

impl fmt::Display for PackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for PackError {}

/// The file a pack of one namespace of a locale is written to and read
/// from: `en-Latn.sdk.entity.tpack`. One rule, so the command line that
/// writes a pack and whatever reads it back cannot disagree.
///
/// ```
/// use teistro_intl::pack;
///
/// assert_eq!(pack::file_name("en-Latn", "sdk.entity"), "en-Latn.sdk.entity.tpack");
/// ```
#[must_use]
pub fn file_name(locale: &str, namespace: &str) -> String {
    format!("{locale}.{namespace}.tpack")
}

/// Builds the pack of one namespace of a locale.
///
/// ```
/// # use teistro_intl::sdk_root;
/// # use teistro_intl::source::Tree;
/// #
/// # let tree = Tree::load(&sdk_root())?;
/// # let nepali = &tree.locales["ne-Deva-NP"];
///
/// use teistro_intl::pack;
///
/// // A namespace the locale lacks is refused, naming it.
/// assert!(pack::build(nepali, "sdk.nothing").unwrap_err().0.contains("sdk.nothing"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
/// covers the whole file and its SHA-256 the payload.
///
/// ```
/// # use teistro_intl::sdk_root;
/// # use teistro_intl::source::Tree;
/// #
/// # let tree = Tree::load(&sdk_root())?;
/// # let nepali = &tree.locales["ne-Deva-NP"];
///
/// use teistro_intl::pack::{self, Bundle};
///
/// let bytes = pack::build_bundle(nepali)?;
/// let bundle = Bundle::parse(&bytes)?;
/// // Every namespace, the metadata once.
/// assert_eq!(bundle.packs().len(), nepali.namespaces.len());
/// assert_eq!(bundle.meta()?, nepali.meta);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
    let sha: [u8; 32] = Sha256::digest(&payload).into();
    let mut out = Vec::with_capacity(64 + payload.len());
    out.extend_from_slice(&BUNDLE_MAGIC);
    out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    push_str(&mut out, &locale.tag)?;
    out.extend_from_slice(&offset(locale.namespaces.len())?.to_le_bytes());
    let crc_at = out.len();
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&sha);
    out.extend_from_slice(&payload);
    seal(&mut out, crc_at);
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
    let sha: [u8; 32] = Sha256::digest(&body).into();
    let mut out = Vec::with_capacity(64 + meta.len() + body.len());
    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    push_str(&mut out, &locale.tag)?;
    push_str(&mut out, namespace)?;
    out.extend_from_slice(&offset(ns.entries.len())?.to_le_bytes());
    out.extend_from_slice(&offset(arena.len())?.to_le_bytes());
    let crc_at = out.len();
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&sha);
    out.extend_from_slice(&offset(meta.len())?.to_le_bytes());
    out.extend_from_slice(&meta);
    out.extend_from_slice(&body);
    seal(&mut out, crc_at);
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
///
/// ```
/// # use teistro_intl::sdk_root;
/// # use teistro_intl::source::Tree;
/// #
/// # let tree = Tree::load(&sdk_root())?;
/// # let nepali = &tree.locales["ne-Deva-NP"];
///
/// use teistro_intl::pack::{self, Pack};
///
/// let bytes = pack::build(nepali, "sdk.reason")?;
/// let reason = Pack::parse(&bytes)?;
/// assert_eq!((reason.locale(), reason.namespace()), ("ne-Deva-NP", "sdk.reason"));
/// assert_eq!(reason.len(), nepali.namespaces["sdk.reason"].entries.len());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
///
/// ```
/// # use teistro_intl::sdk_root;
/// # use teistro_intl::source::Tree;
/// #
/// # let tree = Tree::load(&sdk_root())?;
/// # let nepali = &tree.locales["ne-Deva-NP"];
///
/// use teistro_intl::pack::{self, Pack, PackEntry};
///
/// let bytes = pack::build(nepali, "sdk.entity")?;
/// let entities = Pack::parse(&bytes)?;
/// let Some(PackEntry::Entity(sun)) = entities.get("graha.SUN") else {
///     panic!("the Sun's record");
/// };
/// assert_eq!(sun.name(), "सूर्य");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
    /// ```
    /// # use teistro_intl::sdk_root;
    /// # use teistro_intl::source::Tree;
    /// #
    /// # let tree = Tree::load(&sdk_root())?;
    /// # let nepali = &tree.locales["ne-Deva-NP"];
    ///
    /// use teistro_intl::pack::{self, Pack};
    ///
    /// // Nothing in the file is trusted: one byte changed fails the checksum.
    /// let mut bytes = pack::build(nepali, "sdk.reason")?;
    /// let last = bytes.len() - 1;
    /// bytes[last] ^= 1;
    /// assert!(Pack::parse(&bytes).is_err());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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
        no_flags(&mut cursor)?;
        let locale = cursor.u16_str()?;
        let namespace = cursor.u16_str()?;
        let entries = cursor.u32()? as usize;
        let arena_len = cursor.u32()? as usize;
        let crc_at = cursor.pos;
        let crc = cursor.u32()?;
        // Checked before anything the file says is believed further.
        if checksum(bytes, crc_at) != crc {
            return Err(PackError(String::from("checksum mismatch")));
        }
        let sha256 = cursor.take(32)?;
        let meta_len = cursor.u32()? as usize;
        let meta = cursor.take(meta_len)?;
        let table_len = entries
            .checked_mul(TABLE_ENTRY)
            .ok_or_else(|| PackError(String::from("entry count overflows")))?;
        let table = cursor.take(table_len)?;
        let arena = cursor.take(arena_len)?;
        if cursor.pos != bytes.len() {
            return Err(PackError(String::from("trailing bytes after the arena")));
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
    ///
    /// ```
    /// # use teistro_intl::sdk_root;
    /// # use teistro_intl::source::Tree;
    /// #
    /// # let tree = Tree::load(&sdk_root())?;
    /// # let nepali = &tree.locales["ne-Deva-NP"];
    ///
    /// use teistro_intl::pack::{self, Pack};
    ///
    /// let bytes = pack::build(nepali, "sdk.reason")?;
    /// let reason = Pack::parse(&bytes)?;
    /// assert_eq!(reason.locale(), nepali.tag);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn locale(&self) -> &'a str {
        self.locale
    }

    /// The namespace name.
    ///
    /// ```
    /// # use teistro_intl::sdk_root;
    /// # use teistro_intl::source::Tree;
    /// #
    /// # let tree = Tree::load(&sdk_root())?;
    /// # let nepali = &tree.locales["ne-Deva-NP"];
    ///
    /// use teistro_intl::pack::{self, Pack};
    ///
    /// let bytes = pack::build(nepali, "sdk.reason")?;
    /// let reason = Pack::parse(&bytes)?;
    /// assert_eq!(reason.namespace(), "sdk.reason");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn namespace(&self) -> &'a str {
        self.namespace
    }

    /// The number of entries.
    ///
    /// ```
    /// # use teistro_intl::sdk_root;
    /// # use teistro_intl::source::Tree;
    /// #
    /// # let tree = Tree::load(&sdk_root())?;
    /// # let nepali = &tree.locales["ne-Deva-NP"];
    ///
    /// use teistro_intl::pack::{self, Pack};
    ///
    /// let bytes = pack::build(nepali, "sdk.reason")?;
    /// let reason = Pack::parse(&bytes)?;
    /// assert!(reason.len() > 1);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries
    }

    /// Whether the pack is empty.
    ///
    /// ```
    /// # use teistro_intl::sdk_root;
    /// # use teistro_intl::source::Tree;
    /// #
    /// # let tree = Tree::load(&sdk_root())?;
    /// # let nepali = &tree.locales["ne-Deva-NP"];
    ///
    /// use teistro_intl::pack::{self, Pack};
    ///
    /// let bytes = pack::build(nepali, "sdk.reason")?;
    /// let reason = Pack::parse(&bytes)?;
    /// assert!(!reason.is_empty());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries == 0
    }

    /// The SHA-256 of the key table and arena, lower-case hex.
    ///
    /// ```
    /// # use teistro_intl::sdk_root;
    /// # use teistro_intl::source::Tree;
    /// #
    /// # let tree = Tree::load(&sdk_root())?;
    /// # let nepali = &tree.locales["ne-Deva-NP"];
    ///
    /// use teistro_intl::pack::{self, Pack};
    ///
    /// let bytes = pack::build(nepali, "sdk.reason")?;
    /// let reason = Pack::parse(&bytes)?;
    /// // The same sources pack to the same digest.
    /// let again = pack::build(nepali, "sdk.reason")?;
    /// assert_eq!(Pack::parse(&again)?.content_sha256(), reason.content_sha256());
    /// assert_eq!(reason.content_sha256().len(), 64);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn content_sha256(&self) -> String {
        hex(self.sha256)
    }

    /// The locale metadata carried in the pack.
    ///
    /// ```
    /// # use teistro_intl::sdk_root;
    /// # use teistro_intl::source::Tree;
    /// #
    /// # let tree = Tree::load(&sdk_root())?;
    /// # let nepali = &tree.locales["ne-Deva-NP"];
    ///
    /// use teistro_intl::pack::{self, Pack};
    ///
    /// let bytes = pack::build(nepali, "sdk.reason")?;
    /// let reason = Pack::parse(&bytes)?;
    /// // A pack of its own carries the metadata; one inside a bundle does not.
    /// assert_eq!(reason.meta()?.locale, "ne-Deva-NP");
    /// let bundled = pack::build_bundle(nepali)?;
    /// let bundle = pack::Bundle::parse(&bundled)?;
    /// assert!(bundle.pack("sdk.reason").is_some_and(|inner| inner.meta().is_err()));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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
    ///
    /// ```
    /// # use teistro_intl::sdk_root;
    /// # use teistro_intl::source::Tree;
    /// #
    /// # let tree = Tree::load(&sdk_root())?;
    /// # let nepali = &tree.locales["ne-Deva-NP"];
    ///
    /// use teistro_intl::pack::{self, Pack};
    ///
    /// let bytes = pack::build(nepali, "sdk.reason")?;
    /// let reason = Pack::parse(&bytes)?;
    /// assert!(reason.has_meta());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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
    ///
    /// ```
    /// # use teistro_intl::sdk_root;
    /// # use teistro_intl::source::Tree;
    /// #
    /// # let tree = Tree::load(&sdk_root())?;
    /// # let nepali = &tree.locales["ne-Deva-NP"];
    ///
    /// use teistro_intl::pack::{self, Pack};
    ///
    /// let bytes = pack::build(nepali, "sdk.reason")?;
    /// let reason = Pack::parse(&bytes)?;
    /// use teistro_intl::pack::PackEntry;
    ///
    /// assert!(matches!(reason.get("appName"), Some(PackEntry::Message(_))));
    /// assert_eq!(reason.get("noSuchKey"), None);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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
    ///
    /// ```
    /// # use teistro_intl::sdk_root;
    /// # use teistro_intl::source::Tree;
    /// #
    /// # let tree = Tree::load(&sdk_root())?;
    /// # let nepali = &tree.locales["ne-Deva-NP"];
    ///
    /// use teistro_intl::pack::{self, Pack};
    ///
    /// let bytes = pack::build(nepali, "sdk.reason")?;
    /// let reason = Pack::parse(&bytes)?;
    /// // In key order.
    /// let keys: Vec<&str> = reason.iter().map(|(key, _)| key).collect();
    /// assert!(keys.windows(2).all(|pair| pair[0] < pair[1]));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = (&'a str, PackEntry<'a>)> + '_ {
        (0..self.entries).filter_map(move |i| self.entry_at(i).ok())
    }

    /// The pack's namespace as a source namespace, decoded, in key order.
    ///
    /// ```
    /// # use teistro_intl::sdk_root;
    /// # use teistro_intl::source::Tree;
    /// #
    /// # let tree = Tree::load(&sdk_root())?;
    /// # let nepali = &tree.locales["ne-Deva-NP"];
    ///
    /// use teistro_intl::pack::{self, Pack};
    ///
    /// let bytes = pack::build(nepali, "sdk.reason")?;
    /// let reason = Pack::parse(&bytes)?;
    /// // The same entries as the sources, in key order rather than source order.
    /// assert_eq!(reason.to_namespace().entries, nepali.namespaces["sdk.reason"].entries);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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
///
/// ```
/// # use teistro_intl::sdk_root;
/// # use teistro_intl::source::Tree;
/// #
/// # let tree = Tree::load(&sdk_root())?;
/// # let nepali = &tree.locales["ne-Deva-NP"];
///
/// use teistro_intl::pack::{self, Bundle};
///
/// let bytes = pack::build_bundle(nepali)?;
/// let bundle = Bundle::parse(&bytes)?;
/// assert_eq!(bundle.locale(), "ne-Deva-NP");
/// assert!(bundle.pack("sdk.entity").is_some());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
    /// ```
    /// # use teistro_intl::sdk_root;
    /// # use teistro_intl::source::Tree;
    /// #
    /// # let tree = Tree::load(&sdk_root())?;
    /// # let nepali = &tree.locales["ne-Deva-NP"];
    ///
    /// use teistro_intl::pack::{self, Bundle};
    ///
    /// // A pack is not a bundle.
    /// let single = pack::build(nepali, "sdk.reason")?;
    /// assert!(Bundle::parse(&single).is_err());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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
        no_flags(&mut cursor)?;
        let locale = cursor.u16_str()?;
        let count = cursor.u32()? as usize;
        let crc_at = cursor.pos;
        let crc = cursor.u32()?;
        if checksum(bytes, crc_at) != crc {
            return Err(PackError(String::from("bundle checksum mismatch")));
        }
        let sha256 = cursor.take(32)?;
        let meta_len = cursor.u32()? as usize;
        let meta = cursor.take(meta_len)?;
        // The count is the file's word, and the reader allocates by it:
        // each index entry takes at least its name's length and its own
        // (six bytes), so a count the bytes left cannot hold is refused
        // before anything is reserved for it.
        if count > bytes.len().saturating_sub(cursor.pos) / INDEX_ENTRY_MIN {
            return Err(PackError(format!(
                "an index of {count} packs is longer than the bundle"
            )));
        }
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
    ///
    /// ```
    /// # use teistro_intl::sdk_root;
    /// # use teistro_intl::source::Tree;
    /// #
    /// # let tree = Tree::load(&sdk_root())?;
    /// # let nepali = &tree.locales["ne-Deva-NP"];
    ///
    /// use teistro_intl::pack::{self, Bundle};
    ///
    /// let bytes = pack::build_bundle(nepali)?;
    /// let bundle = Bundle::parse(&bytes)?;
    /// assert_eq!(bundle.locale(), nepali.tag);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn locale(&self) -> &'a str {
        self.locale
    }

    /// The locale metadata, carried once.
    ///
    /// ```
    /// # use teistro_intl::sdk_root;
    /// # use teistro_intl::source::Tree;
    /// #
    /// # let tree = Tree::load(&sdk_root())?;
    /// # let nepali = &tree.locales["ne-Deva-NP"];
    ///
    /// use teistro_intl::pack::{self, Bundle};
    ///
    /// let bytes = pack::build_bundle(nepali)?;
    /// let bundle = Bundle::parse(&bytes)?;
    /// assert_eq!(bundle.meta()?.numbering_system, "deva");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Metadata that does not parse.
    pub fn meta(&self) -> Result<Meta, PackError> {
        serde_json::from_slice(self.meta).map_err(|e| PackError(format!("metadata: {e}")))
    }

    /// The namespaces' packs, in name order.
    ///
    /// ```
    /// # use teistro_intl::sdk_root;
    /// # use teistro_intl::source::Tree;
    /// #
    /// # let tree = Tree::load(&sdk_root())?;
    /// # let nepali = &tree.locales["ne-Deva-NP"];
    ///
    /// use teistro_intl::pack::{self, Bundle};
    ///
    /// let bytes = pack::build_bundle(nepali)?;
    /// let bundle = Bundle::parse(&bytes)?;
    /// // In name order.
    /// let names: Vec<&str> = bundle.packs().iter().map(|inner| inner.namespace()).collect();
    /// assert!(names.windows(2).all(|pair| pair[0] < pair[1]));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn packs(&self) -> &[Pack<'a>] {
        &self.packs
    }

    /// One namespace's pack.
    ///
    /// ```
    /// # use teistro_intl::sdk_root;
    /// # use teistro_intl::source::Tree;
    /// #
    /// # let tree = Tree::load(&sdk_root())?;
    /// # let nepali = &tree.locales["ne-Deva-NP"];
    ///
    /// use teistro_intl::pack::{self, Bundle};
    ///
    /// let bytes = pack::build_bundle(nepali)?;
    /// let bundle = Bundle::parse(&bytes)?;
    /// assert!(bundle.pack("sdk.reason").is_some());
    /// assert!(bundle.pack("sdk.nothing").is_none());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn pack(&self, namespace: &str) -> Option<&Pack<'a>> {
        self.packs.iter().find(|p| p.namespace() == namespace)
    }

    /// The SHA-256 of the payload, lower-case hex, for the provenance
    /// envelope.
    ///
    /// ```
    /// # use teistro_intl::sdk_root;
    /// # use teistro_intl::source::Tree;
    /// #
    /// # let tree = Tree::load(&sdk_root())?;
    /// # let nepali = &tree.locales["ne-Deva-NP"];
    ///
    /// use teistro_intl::pack::{self, Bundle};
    ///
    /// let bytes = pack::build_bundle(nepali)?;
    /// let bundle = Bundle::parse(&bytes)?;
    /// let again = pack::build_bundle(nepali)?;
    /// assert_eq!(Bundle::parse(&again)?.content_sha256(), bundle.content_sha256());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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
/// ```
/// # use teistro_intl::sdk_root;
/// # use teistro_intl::source::Tree;
/// #
/// # let tree = Tree::load(&sdk_root())?;
/// # let nepali = &tree.locales["ne-Deva-NP"];
///
/// use teistro_intl::pack;
///
/// // A bundle brings its whole locale back.
/// let bundle = pack::build_bundle(nepali)?;
/// let rebuilt = pack::locales_from_packs(&[&bundle])?;
/// assert_eq!(rebuilt["ne-Deva-NP"].meta, nepali.meta);
/// assert_eq!(rebuilt["ne-Deva-NP"].namespaces.len(), nepali.namespaces.len());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
#[cfg_attr(coverage_nightly, coverage(off))]
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
        // A byte past the end breaks the checksum, which covers the
        // whole file; sealed again, it is refused as trailing.
        let mut extended = bytes.clone();
        extended.push(0);
        assert!(Pack::parse(&extended).is_err_and(|e| e.0.contains("checksum")));
        seal(&mut extended, crc_at(&bytes));
        assert!(Pack::parse(&extended).is_err_and(|e| e.0.contains("trailing")));
        // The header is covered too: a changed locale tag, and flags
        // this reader does not know even when sealed.
        let mut renamed = bytes.clone();
        renamed[10] ^= 1;
        assert!(Pack::parse(&renamed).is_err_and(|e| e.0.contains("checksum")));
        let mut flagged = bytes.clone();
        flagged[6] = 1;
        seal(&mut flagged, crc_at(&bytes));
        assert!(Pack::parse(&flagged).is_err_and(|e| e.0.contains("flags")));
    }

    /// Where a pack's CRC sits: after its magic, version, flags, locale,
    /// namespace, entry count and arena length.
    fn crc_at(pack: &[u8]) -> usize {
        let len = |at: usize| usize::from(u16::from_le_bytes([pack[at], pack[at + 1]]));
        let locale = len(8);
        let namespace = len(10 + locale);
        12 + locale + namespace + 8
    }

    #[test]
    fn a_bundle_whose_count_outruns_its_bytes_is_refused_unallocated() {
        // The fuzz smoke's input (verify, 2026-10-23): a valid checksum
        // over a count of 0x4E490000 packs, which the reader once
        // reserved 29 GB for before reading the first; re-sealed as
        // version 3, whose checksum covers the header too.
        let bytes: &[u8] = b"TPB1\x03\x00\x00\x00\x00\x00\x00\x00INT\xf1\xb6]\xe2\xcdgV$\xff\xff\xff_r|ici\
            \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
            @2paka\x02\x18\x11\x10\xde#\xa3q8\xcdy\x01\x00\x00{\"loc\x00";
        assert_eq!(bytes.len(), 83);
        // Refused at the count, so the checksum held and the input
        // reaches the reservation it once made.
        assert!(
            Bundle::parse(bytes).is_err_and(|e| e.0.contains("longer than the bundle")),
            "the count is refused"
        );
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
