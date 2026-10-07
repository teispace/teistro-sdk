//! The oldest C library a shipped Linux artefact loads on.
//!
//! A shared library built on a runner links against that runner's glibc,
//! and the dynamic loader refuses it on any older one: a library built on
//! Ubuntu 24.04 asks for symbols at `GLIBC_2.39`'s level and will not load
//! on Debian 11, RHEL 8 or Ubuntu 22.04. Nothing in the build says so; the
//! only record is the version-needed table in the file itself, so this is
//! read there.
//!
//! The table is `.gnu.version_r` (`SHT_GNU_verneed`): for each library the
//! file depends on, a list of the symbol versions it needs, each named by
//! an offset into the string table the section links to. Every name of the
//! form `GLIBC_<major>.<minor>` is a version the loader must find, and the
//! greatest of them is the floor. Only the 64-bit little-endian layout is
//! read, because it is the only one the SDK ships for Linux (x86-64 and
//! 64-bit Arm); any other is refused rather than misread.

use std::fs;
use std::path::Path;

use crate::platform::Platform;

/// A glibc version: major and minor.
pub(crate) type Glibc = (u32, u32);

/// The section type of the version-needed table.
const SHT_GNU_VERNEED: u32 = 0x6fff_fffe;

/// The greatest `GLIBC_x.y` an ELF file needs, or `None` when it needs no
/// versioned glibc symbol at all.
pub(crate) fn glibc_needed(elf: &[u8]) -> Result<Option<Glibc>, String> {
    if elf.get(..4) != Some(b"\x7fELF".as_slice()) {
        return Err("not an ELF file".into());
    }
    // EI_CLASS 2 is 64-bit and EI_DATA 1 is little-endian.
    if elf.get(4) != Some(&2) || elf.get(5) != Some(&1) {
        return Err("not a 64-bit little-endian ELF file, the only layout the SDK ships".into());
    }
    let section_table = usize_at(elf, 0x28)?;
    let entry_size = usize::from(u16_at(elf, 0x3a)?);
    let sections = usize::from(u16_at(elf, 0x3c)?);
    let header = |index: usize| -> Result<usize, String> {
        index
            .checked_mul(entry_size)
            .and_then(|offset| offset.checked_add(section_table))
            .ok_or_else(|| "the section table runs past the file".to_string())
    };
    let mut greatest: Option<Glibc> = None;
    for index in 0..sections {
        let at = header(index)?;
        if u32_at(elf, at + 0x04)? != SHT_GNU_VERNEED {
            continue;
        }
        let table = usize_at(elf, at + 0x18)?;
        let entries = u32_at(elf, at + 0x2c)?;
        let strings = usize_at(elf, header(usize_from(u32_at(elf, at + 0x28)?)?)? + 0x18)?;
        // Each Elf64_Verneed: vn_version u16, vn_cnt u16, vn_file u32,
        // vn_aux u32, vn_next u32. Each Elf64_Vernaux: vna_hash u32,
        // vna_flags u16, vna_other u16, vna_name u32, vna_next u32. Both
        // chain by an offset from the record that names it.
        let mut need = table;
        for _ in 0..entries {
            let count = u16_at(elf, need + 2)?;
            let mut aux = need + usize_from(u32_at(elf, need + 8)?)?;
            for _ in 0..count {
                let name = cstr_at(elf, strings + usize_from(u32_at(elf, aux + 8)?)?)?;
                if let Some(version) = glibc_version(name) {
                    greatest = greatest.max(Some(version));
                }
                aux += usize_from(u32_at(elf, aux + 12)?)?;
            }
            need += usize_from(u32_at(elf, need + 12)?)?;
        }
    }
    Ok(greatest)
}

/// `GLIBC_2.28` as `(2, 28)`; anything else (`GLIBC_PRIVATE`, another
/// library's versions) as `None`.
fn glibc_version(name: &str) -> Option<Glibc> {
    let (major, minor) = name.strip_prefix("GLIBC_")?.split_once('.')?;
    // A third component (`GLIBC_2.2.5`) is a patch level: it sorts below
    // the next minor, so the first two decide the floor.
    let minor = minor.split('.').next()?;
    Some((major.parse().ok()?, minor.parse().ok()?))
}

/// Holds each file to the platform's glibc floor, and says what each
/// needs. A platform without a floor (macOS, Windows) passes untouched.
pub(crate) fn check(platform: &Platform, files: &[&Path]) -> Result<(), String> {
    let Some(floor) = platform.glibc_floor else {
        return Ok(());
    };
    for file in files {
        let bytes = fs::read(file).map_err(|err| format!("{}: {err}", file.display()))?;
        let needed = glibc_needed(&bytes).map_err(|err| format!("{}: {err}", file.display()))?;
        let name = file.file_name().map_or_else(
            || file.display().to_string(),
            |name| name.to_string_lossy().into_owned(),
        );
        let shown = needed.map_or_else(
            || "no versioned glibc symbol".to_string(),
            |(a, b)| format!("GLIBC_{a}.{b}"),
        );
        if needed.is_some_and(|needed| needed > floor) {
            return Err(format!(
                "{name} needs {shown}, above {}'s floor of GLIBC_{}.{}: it would not load on the older \
                 distributions the build matrix promises; build it against the floor",
                platform.name(),
                floor.0,
                floor.1
            ));
        }
        println!(
            "floor  {name} needs {shown} (floor GLIBC_{}.{})",
            floor.0, floor.1
        );
    }
    Ok(())
}

fn usize_from(value: u32) -> Result<usize, String> {
    usize::try_from(value).map_err(|_| "an offset does not fit this machine".to_string())
}

fn bytes_at<const N: usize>(elf: &[u8], at: usize) -> Result<[u8; N], String> {
    at.checked_add(N)
        .and_then(|end| elf.get(at..end))
        .and_then(|slice| slice.try_into().ok())
        .ok_or_else(|| format!("the file ends before offset {at:#x}"))
}

fn u16_at(elf: &[u8], at: usize) -> Result<u16, String> {
    bytes_at(elf, at).map(u16::from_le_bytes)
}

fn u32_at(elf: &[u8], at: usize) -> Result<u32, String> {
    bytes_at(elf, at).map(u32::from_le_bytes)
}

fn usize_at(elf: &[u8], at: usize) -> Result<usize, String> {
    let value = bytes_at(elf, at).map(u64::from_le_bytes)?;
    usize::try_from(value).map_err(|_| "an offset does not fit this machine".to_string())
}

fn cstr_at(elf: &[u8], at: usize) -> Result<&str, String> {
    let tail = elf
        .get(at..)
        .ok_or_else(|| format!("a name at {at:#x} lies past the file"))?;
    let end = tail
        .iter()
        .position(|&byte| byte == 0)
        .ok_or("a name runs past the file")?;
    std::str::from_utf8(tail.get(..end).unwrap_or_default())
        .map_err(|_| "a name is not UTF-8".to_string())
}

#[cfg(test)]
mod tests {
    use super::{glibc_needed, glibc_version};

    /// The smallest ELF that carries a version-needed table: a header, a
    /// string table, one `libc.so.6` entry needing the named versions,
    /// and a section table of three entries (null, strings, verneed).
    fn elf_needing(versions: &[&str]) -> Vec<u8> {
        let mut strings = b"\0libc.so.6\0".to_vec();
        let mut names = Vec::new();
        for version in versions {
            names.push(u32::try_from(strings.len()).unwrap_or_default());
            strings.extend_from_slice(version.as_bytes());
            strings.push(0);
        }
        let mut verneed = Vec::new();
        let count = u16::try_from(versions.len()).unwrap_or_default();
        verneed.extend_from_slice(&1u16.to_le_bytes());
        verneed.extend_from_slice(&count.to_le_bytes());
        verneed.extend_from_slice(&1u32.to_le_bytes()); // vn_file: "libc.so.6"
        verneed.extend_from_slice(&16u32.to_le_bytes()); // vn_aux
        verneed.extend_from_slice(&0u32.to_le_bytes()); // vn_next
        for (i, name) in names.iter().enumerate() {
            verneed.extend_from_slice(&0u32.to_le_bytes()); // vna_hash
            verneed.extend_from_slice(&0u16.to_le_bytes());
            verneed.extend_from_slice(&0u16.to_le_bytes());
            verneed.extend_from_slice(&name.to_le_bytes());
            let next: u32 = if i + 1 == names.len() { 0 } else { 16 };
            verneed.extend_from_slice(&next.to_le_bytes());
        }
        let strings_at = 64usize;
        let verneed_at = strings_at + strings.len();
        let table_at = verneed_at + verneed.len();
        let mut elf = vec![0u8; 64];
        elf[..6].copy_from_slice(b"\x7fELF\x02\x01");
        elf[0x28..0x30].copy_from_slice(&(table_at as u64).to_le_bytes());
        elf[0x3a..0x3c].copy_from_slice(&64u16.to_le_bytes());
        elf[0x3c..0x3e].copy_from_slice(&3u16.to_le_bytes());
        elf.extend_from_slice(&strings);
        elf.extend_from_slice(&verneed);
        let section = |kind: u32, offset: usize, link: u32, info: u32| {
            let mut header = vec![0u8; 64];
            header[0x04..0x08].copy_from_slice(&kind.to_le_bytes());
            header[0x18..0x20].copy_from_slice(&(offset as u64).to_le_bytes());
            header[0x28..0x2c].copy_from_slice(&link.to_le_bytes());
            header[0x2c..0x30].copy_from_slice(&info.to_le_bytes());
            header
        };
        elf.extend(section(0, 0, 0, 0));
        elf.extend(section(3, strings_at, 0, 0));
        elf.extend(section(0x6fff_fffe, verneed_at, 1, 1));
        elf
    }

    #[test]
    fn the_floor_is_the_greatest_glibc_version_named() {
        let elf = elf_needing(&["GLIBC_2.2.5", "GLIBC_2.34", "GLIBC_PRIVATE", "GLIBC_2.28"]);
        assert_eq!(glibc_needed(&elf), Ok(Some((2, 34))));
    }

    #[test]
    fn a_minor_version_compares_as_a_number() {
        // 2.9 is older than 2.17, which a string comparison would invert.
        let elf = elf_needing(&["GLIBC_2.9", "GLIBC_2.17"]);
        assert_eq!(glibc_needed(&elf), Ok(Some((2, 17))));
    }

    #[test]
    fn a_file_with_no_glibc_versions_has_no_floor() {
        assert_eq!(glibc_needed(&elf_needing(&["GLIBC_PRIVATE"])), Ok(None));
    }

    #[test]
    fn anything_but_a_64_bit_little_endian_elf_is_refused() {
        assert!(glibc_needed(b"MZ\x90\x00").is_err());
        let mut big_endian = elf_needing(&["GLIBC_2.17"]);
        big_endian[5] = 2;
        assert!(glibc_needed(&big_endian).is_err());
        let truncated = elf_needing(&["GLIBC_2.17"]);
        assert!(glibc_needed(&truncated[..100]).is_err());
    }

    #[test]
    fn a_version_name_reads_as_major_and_minor() {
        assert_eq!(glibc_version("GLIBC_2.28"), Some((2, 28)));
        assert_eq!(glibc_version("GLIBC_2.2.5"), Some((2, 2)));
        assert_eq!(glibc_version("GLIBC_PRIVATE"), None);
        assert_eq!(glibc_version("GCC_3.0"), None);
    }
}
