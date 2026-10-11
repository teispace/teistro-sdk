//! The one zip writer the packages share: a wheel and a jar are both a zip
//! of fixed shape, written here rather than by `zip` or `jar` so the entry
//! order and the dates are the caller's and two stagings of one release
//! write the same bytes.
//!
//! Every entry is deflated and dated 1980-01-01 00:00, with Unix mode 0644
//! (0755 for a program, through [`write_modes`]), no extra fields and no directory entries (no reader of either format
//! needs them). An entry can arrive already deflated, so a library two
//! archives carry is packed once.

use std::io::{self, Write};

use flate2::Compression;
use flate2::write::DeflateEncoder;

/// One file's bytes, as given or already deflated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Entry {
    /// The bytes, to be deflated as written.
    Plain(Vec<u8>),
    /// Bytes deflated by [`pack`], with what the headers need of the
    /// original.
    Packed {
        /// The CRC-32 of the original bytes.
        crc: u32,
        /// The original length.
        size: usize,
        /// The deflated bytes.
        deflated: Vec<u8>,
    },
}

/// Deflates bytes once, for an entry more than one archive carries.
pub(crate) fn pack(data: &[u8]) -> io::Result<Entry> {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(data)?;
    Ok(Entry::Packed {
        crc: crc32fast::hash(data),
        size: data.len(),
        deflated: encoder.finish()?,
    })
}

/// A zip of `entries`, in the order given, each mode 0644.
pub(crate) fn write<'e, N: AsRef<str> + 'e>(
    entries: impl IntoIterator<Item = (N, &'e Entry)>,
) -> io::Result<Vec<u8>> {
    write_modes(
        entries
            .into_iter()
            .map(|(name, entry)| (name, entry, 0o644)),
    )
}

/// A zip of `entries`, in the order given, each with its Unix mode: 0755
/// for a program a reader should be able to run once unpacked.
pub(crate) fn write_modes<'e, N: AsRef<str> + 'e>(
    entries: impl IntoIterator<Item = (N, &'e Entry, u32)>,
) -> io::Result<Vec<u8>> {
    /// 1980-01-01 in MS-DOS date format: day 1, month 1, year 0.
    const DOS_DATE: u16 = 0x21;
    let mut out = Vec::new();
    let mut central = Vec::new();
    let mut count = 0_usize;
    for (name, entry, mode) in entries {
        let name = name.as_ref();
        let packed_here;
        let (crc, size, packed) = match entry {
            Entry::Plain(data) => {
                packed_here = pack(data)?;
                let Entry::Packed {
                    crc,
                    size,
                    deflated,
                } = &packed_here
                else {
                    unreachable!("pack answers a packed entry")
                };
                (*crc, *size, deflated)
            }
            Entry::Packed {
                crc,
                size,
                deflated,
            } => (*crc, *size, deflated),
        };
        let offset = u32::try_from(out.len()).map_err(io::Error::other)?;
        let sizes = [
            u32::try_from(packed.len()).map_err(io::Error::other)?,
            u32::try_from(size).map_err(io::Error::other)?,
        ];
        let name_length = u16::try_from(name.len()).map_err(io::Error::other)?;
        // The fields every header shares: version needed (2.0), no flags,
        // deflate, the time and date, the digest and both sizes, the name.
        let common = |into: &mut Vec<u8>| {
            for field in [20_u16, 0, 8, 0, DOS_DATE] {
                into.extend(field.to_le_bytes());
            }
            into.extend(crc.to_le_bytes());
            into.extend(sizes[0].to_le_bytes());
            into.extend(sizes[1].to_le_bytes());
            into.extend(name_length.to_le_bytes());
        };
        out.extend(0x0403_4b50_u32.to_le_bytes());
        common(&mut out);
        out.extend(0_u16.to_le_bytes());
        out.extend(name.as_bytes());
        out.extend(packed);

        central.extend(0x0201_4b50_u32.to_le_bytes());
        // Made by Unix (3), zip 2.0, so the mode below is read.
        central.extend(0x0314_u16.to_le_bytes());
        common(&mut central);
        for field in [0_u16, 0, 0, 0] {
            central.extend(field.to_le_bytes());
        }
        central.extend(((0o100_000 | mode) << 16).to_le_bytes());
        central.extend(offset.to_le_bytes());
        central.extend(name.as_bytes());
        count += 1;
    }
    let count = u16::try_from(count).map_err(io::Error::other)?;
    let start = u32::try_from(out.len()).map_err(io::Error::other)?;
    let size = u32::try_from(central.len()).map_err(io::Error::other)?;
    out.extend(&central);
    out.extend(0x0605_4b50_u32.to_le_bytes());
    for field in [0_u16, 0, count, count] {
        out.extend(field.to_le_bytes());
    }
    out.extend(size.to_le_bytes());
    out.extend(start.to_le_bytes());
    out.extend(0_u16.to_le_bytes());
    Ok(out)
}

/// One entry as the central directory records it.
struct Record {
    name: String,
    /// Where its local header starts.
    offset: usize,
    /// Its deflated length.
    packed: usize,
}

/// A little-endian field of a zip, or why it is not there.
fn field(bytes: &[u8], at: usize, width: usize) -> io::Result<usize> {
    let slice = bytes.get(at..at + width).ok_or_else(foreign)?;
    Ok(slice
        .iter()
        .rev()
        .fold(0_usize, |sum, byte| (sum << 8) | usize::from(*byte)))
}

/// The refusal of bytes this writer did not write.
fn foreign() -> io::Error {
    io::Error::other("not a zip this writer wrote")
}

/// The central directory's records, in order.
fn directory(bytes: &[u8]) -> io::Result<Vec<Record>> {
    let end = bytes.len().checked_sub(22).ok_or_else(foreign)?;
    if bytes.get(end..end + 4) != Some(b"PK\x05\x06".as_slice()) {
        return Err(foreign());
    }
    let count = field(bytes, end + 10, 2)?;
    let mut at = field(bytes, end + 16, 4)?;
    let mut found = Vec::with_capacity(count);
    for _ in 0..count {
        if bytes.get(at..at + 4) != Some(b"PK\x01\x02".as_slice()) {
            return Err(foreign());
        }
        let length = field(bytes, at + 28, 2)?;
        let extra = field(bytes, at + 30, 2)? + field(bytes, at + 32, 2)?;
        let name = bytes.get(at + 46..at + 46 + length).ok_or_else(foreign)?;
        found.push(Record {
            name: String::from_utf8(name.to_vec()).map_err(io::Error::other)?,
            offset: field(bytes, at + 42, 4)?,
            packed: field(bytes, at + 20, 4)?,
        });
        at += 46 + length + extra;
    }
    Ok(found)
}

/// The names of a zip's entries, in order, read from its central
/// directory: what a gate reads a package back by.
pub(crate) fn names(bytes: &[u8]) -> io::Result<Vec<String>> {
    Ok(directory(bytes)?
        .into_iter()
        .map(|record| record.name)
        .collect())
}

/// An entry's bytes, inflated, or nothing when the zip has no such entry.
pub(crate) fn read(bytes: &[u8], name: &str) -> io::Result<Option<Vec<u8>>> {
    let Some(record) = directory(bytes)?
        .into_iter()
        .find(|record| record.name == name)
    else {
        return Ok(None);
    };
    let at = record.offset;
    let start = at + 30 + field(bytes, at + 26, 2)? + field(bytes, at + 28, 2)?;
    let packed = bytes
        .get(start..start + record.packed)
        .ok_or_else(foreign)?;
    let mut data = Vec::new();
    io::Read::read_to_end(&mut flate2::read::DeflateDecoder::new(packed), &mut data)?;
    Ok(Some(data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    fn plain(entries: &[(&str, &[u8])]) -> Vec<(String, Entry)> {
        entries
            .iter()
            .map(|(name, data)| ((*name).to_owned(), Entry::Plain(data.to_vec())))
            .collect()
    }

    fn written(entries: &[(String, Entry)]) -> Vec<u8> {
        write(entries.iter().map(|(name, entry)| (name, entry))).unwrap()
    }

    #[test]
    fn a_zip_holds_its_files_in_the_order_given() {
        let entries = plain(&[("a", b"first"), ("b", b"second")]);
        let bytes = written(&entries);
        assert_eq!(&bytes[..4], b"PK\x03\x04");
        assert_eq!(&bytes[30..31], b"a");
        let end = bytes.len() - 22;
        assert_eq!(&bytes[end..end + 4], b"PK\x05\x06");
        assert_eq!(u16::from_le_bytes([bytes[end + 10], bytes[end + 11]]), 2);
        assert_eq!(written(&entries), bytes, "the same files, the same bytes");
        assert_eq!(names(&bytes).unwrap(), ["a", "b"]);
        assert_eq!(read(&bytes, "b").unwrap().unwrap(), b"second");
        assert_eq!(read(&bytes, "c").unwrap(), None);
        assert!(names(b"not a zip").is_err());
    }

    /// The bytes the wheel's own writer wrote before it moved here: the
    /// move must not change a single wheel.
    #[test]
    fn the_writer_writes_the_bytes_the_wheel_wrote() {
        // In the key order the wheel passes its files in.
        let entries = plain(&[
            ("teistro-1.0.0.dist-info/RECORD", b"a,,\n"),
            ("teistro/__init__.py", b"print('x')\n"),
            ("teistro/_lib/libteistro_ffi.so", &[0_u8, 1, 2, 3, 255]),
        ]);
        let digest = Sha256::digest(written(&entries));
        assert_eq!(crate::hashes::hex(&digest), GOLDEN);
    }

    /// The SHA-256 of the golden case, from the writer before the move.
    const GOLDEN: &str = "910be239d2e1a82497ad8fdbc1be93299b3319883d0ac11d2da775299a6c01b6";

    #[test]
    fn a_packed_entry_writes_as_its_plain_twin() {
        let data = b"the same library twice".to_vec();
        let packed = vec![("lib".to_owned(), pack(&data).unwrap())];
        let plain = vec![("lib".to_owned(), Entry::Plain(data))];
        assert_eq!(written(&packed), written(&plain));
    }
}
