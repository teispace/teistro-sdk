//! A Python wheel per platform, carrying that platform's library.
//!
//! The binding is `ctypes` over the shared library, so a wheel holds no
//! compiled Python: the sources the package stages, and the one library
//! under `teistro/_lib`, where `Teistro.open` looks before anything else
//! it searches. pip picks the wheel whose platform tag matches the host,
//! so `pip install teistro` loads with nothing downloaded on first use;
//! the pure wheel and the sdist stay, and on a host no wheel is tagged for
//! they install as before and `teistro-install` fetches the library.
//!
//! It is written here rather than by setuptools because setuptools builds
//! a wheel for the machine it runs on, and the release stages every
//! platform's on one. The format is a zip of fixed shape (PEP 427, PEP
//! 491): the files, then a `dist-info` with `METADATA` read from
//! `pyproject.toml`, `WHEEL`, `entry_points.txt`, the licences and a
//! `RECORD` of every file's digest. Every entry is dated 1980-01-01 and the
//! entries are sorted, so two stagings of one release write the same bytes,
//! as the archives do.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};

use crate::platform::Platform;
use crate::zip::{self, Entry};

/// The distribution's name, as the wheel's file and its `dist-info` spell
/// it.
const NAME: &str = "teistro";

/// Where in the package the library goes: the first place
/// `teistro._search` looks.
const LIBRARY_DIRECTORY: &str = "teistro/_lib";

/// Writes `platform`'s wheel into `into` from the package staged at
/// `staged` and the library's bytes, and returns its file name.
pub(crate) fn write(
    staged: &Path,
    into: &Path,
    version: &str,
    platform: &Platform,
    library: &[u8],
) -> io::Result<String> {
    if platform.os == "darwin" {
        held_to_tag(platform, library).map_err(io::Error::other)?;
    }
    let mut files: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    collect(&staged.join(NAME), NAME, &mut files)?;
    files.insert(
        format!(
            "{LIBRARY_DIRECTORY}/{}",
            platform.shared(crate::binding::LIBRARY_STEM)
        ),
        library.to_vec(),
    );
    let dist_info = format!("{NAME}-{version}.dist-info");
    let project = pyproject(staged)?;
    let readme = fs::read_to_string(staged.join("README.md"))?;
    files.insert(
        format!("{dist_info}/METADATA"),
        metadata(&project, version, &readme).into_bytes(),
    );
    files.insert(
        format!("{dist_info}/WHEEL"),
        format!(
            "Wheel-Version: 1.0\nGenerator: teistro-xtask\nRoot-Is-Purelib: false\nTag: py3-none-{}\n",
            platform.wheel_tag
        )
        .into_bytes(),
    );
    files.insert(
        format!("{dist_info}/entry_points.txt"),
        entry_points(&project).into_bytes(),
    );
    for legal in ["LICENSE", "NOTICE"] {
        files.insert(
            format!("{dist_info}/licenses/{legal}"),
            fs::read(staged.join(legal))?,
        );
    }
    let record = format!("{dist_info}/RECORD");
    files.insert(record.clone(), self::record(&files, &record).into_bytes());

    fs::create_dir_all(into)?;
    let name = format!("{NAME}-{version}-py3-none-{}.whl", platform.wheel_tag);
    let entries: Vec<(&String, Entry)> = files
        .iter()
        .map(|(path, data)| (path, Entry::Plain(data.clone())))
        .collect();
    fs::write(
        into.join(&name),
        zip::write(entries.iter().map(|(path, entry)| (*path, entry)))?,
    )?;
    Ok(name)
}

/// Every file under `directory`, keyed by its path in the wheel, skipping
/// the caches a Python run leaves beside its sources.
fn collect(
    directory: &Path,
    prefix: &str,
    files: &mut BTreeMap<String, Vec<u8>>,
) -> io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        let name = path
            .file_name()
            .map_or_else(String::new, |name| name.to_string_lossy().into_owned());
        let compiled = Path::new(&name)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("pyc"));
        if name == "__pycache__" || compiled {
            continue;
        }
        let key = format!("{prefix}/{name}");
        if path.is_dir() {
            collect(&path, &key, files)?;
        } else {
            files.insert(key, fs::read(&path)?);
        }
    }
    Ok(())
}

/// The staged `pyproject.toml`'s `[project]` table.
fn pyproject(staged: &Path) -> io::Result<toml::Table> {
    let text = fs::read_to_string(staged.join("pyproject.toml"))?;
    let mut parsed: toml::Table = text.parse().map_err(io::Error::other)?;
    match parsed.remove("project") {
        Some(toml::Value::Table(project)) => Ok(project),
        _ => Err(io::Error::other("pyproject.toml has no [project] table")),
    }
}

/// A string field of the project, or empty.
fn text<'a>(project: &'a toml::Table, key: &str) -> &'a str {
    project
        .get(key)
        .and_then(toml::Value::as_str)
        .unwrap_or_default()
}

/// A list-of-strings field of the project, or empty.
fn strings<'a>(project: &'a toml::Table, key: &str) -> Vec<&'a str> {
    project
        .get(key)
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
        .collect()
}

/// The core metadata (version 2.4) the project declares, with the readme
/// as its description.
fn metadata(project: &toml::Table, version: &str, readme: &str) -> String {
    let mut lines = vec![
        "Metadata-Version: 2.4".to_string(),
        format!("Name: {}", text(project, "name")),
        format!("Version: {version}"),
        format!("Summary: {}", text(project, "description")),
    ];
    for author in project
        .get("authors")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_table)
    {
        lines.push(format!(
            "Author-email: {} <{}>",
            text(author, "name"),
            text(author, "email")
        ));
    }
    lines.push(format!("License-Expression: {}", text(project, "license")));
    lines.extend([
        "License-File: LICENSE".to_string(),
        "License-File: NOTICE".to_string(),
    ]);
    let keywords = strings(project, "keywords");
    if !keywords.is_empty() {
        lines.push(format!("Keywords: {}", keywords.join(",")));
    }
    for classifier in strings(project, "classifiers") {
        lines.push(format!("Classifier: {classifier}"));
    }
    if let Some(urls) = project.get("urls").and_then(toml::Value::as_table) {
        for (label, url) in urls {
            lines.push(format!(
                "Project-URL: {label}, {}",
                url.as_str().unwrap_or_default()
            ));
        }
    }
    lines.push(format!(
        "Requires-Python: {}",
        text(project, "requires-python")
    ));
    lines.push("Description-Content-Type: text/markdown".to_string());
    format!("{}\n\n{readme}", lines.join("\n"))
}

/// The console scripts the project declares, as `entry_points.txt` lists
/// them.
fn entry_points(project: &toml::Table) -> String {
    let scripts: Vec<String> = project
        .get("scripts")
        .and_then(toml::Value::as_table)
        .into_iter()
        .flatten()
        .map(|(name, target)| format!("{name} = {}\n", target.as_str().unwrap_or_default()))
        .collect();
    format!("[console_scripts]\n{}", scripts.concat())
}

/// `RECORD`: every file's SHA-256, URL-safe base64 without padding, and
/// size; the record itself is listed without either.
fn record(files: &BTreeMap<String, Vec<u8>>, own: &str) -> String {
    let listed: Vec<String> = files
        .iter()
        .map(|(name, data)| {
            let digest = URL_SAFE_NO_PAD.encode(Sha256::digest(data));
            format!("{name},sha256={digest},{}\n", data.len())
        })
        .collect();
    format!("{}{own},,\n", listed.concat())
}

/// Refuses a macOS library that needs a newer system than its wheel's tag
/// says: pip would install it where the loader then refuses it.
fn held_to_tag(platform: &Platform, library: &[u8]) -> Result<(), String> {
    let needed = macos_minimum(library)?;
    let tagged = platform
        .wheel_tag
        .strip_prefix("macosx_")
        .and_then(|rest| {
            let mut parts = rest.splitn(3, '_');
            Some((parts.next()?.parse().ok()?, parts.next()?.parse().ok()?))
        })
        .ok_or_else(|| format!("{} is not a macOS wheel tag", platform.wheel_tag))?;
    if needed > tagged {
        return Err(format!(
            "the {} library needs macOS {}.{}, newer than its wheel's tag {}",
            platform.name(),
            needed.0,
            needed.1,
            platform.wheel_tag
        ));
    }
    Ok(())
}

/// The oldest macOS a 64-bit Mach-O file loads on: `LC_BUILD_VERSION`'s
/// `minos`, or the older `LC_VERSION_MIN_MACOSX`'s `version`, each
/// `xxxx.yy.zz` in nibbles.
fn macos_minimum(file: &[u8]) -> Result<(u32, u32), String> {
    const MAGIC_64: u32 = 0xfeed_facf;
    const LC_VERSION_MIN_MACOSX: u32 = 0x24;
    const LC_BUILD_VERSION: u32 = 0x32;
    let word = |at: usize| -> Result<u32, String> {
        file.get(at..at + 4)
            .and_then(|bytes| bytes.try_into().ok())
            .map(u32::from_le_bytes)
            .ok_or_else(|| "a Mach-O file that ends inside its load commands".to_string())
    };
    if word(0)? != MAGIC_64 {
        return Err("not a 64-bit little-endian Mach-O file".into());
    }
    let commands = word(16)?;
    let mut at = 32;
    for _ in 0..commands {
        let (command, size) = (word(at)?, word(at + 4)?);
        let version = match command {
            LC_BUILD_VERSION => Some(word(at + 12)?),
            LC_VERSION_MIN_MACOSX => Some(word(at + 8)?),
            _ => None,
        };
        if let Some(version) = version {
            return Ok((version >> 16, (version >> 8) & 0xff));
        }
        at += usize::try_from(size).map_err(|err| err.to_string())?;
    }
    Err("a Mach-O file that names no minimum macOS".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The smallest Mach-O header with one `LC_BUILD_VERSION` naming
    /// macOS `major.minor`.
    fn macho(major: u32, minor: u32) -> Vec<u8> {
        let mut file = Vec::new();
        for field in [0xfeed_facf_u32, 0x0100_000c, 0, 6, 1, 24, 0, 0] {
            file.extend(field.to_le_bytes());
        }
        for field in [0x32_u32, 24, 1, (major << 16) | (minor << 8), 0, 0] {
            file.extend(field.to_le_bytes());
        }
        file
    }

    #[test]
    fn a_macos_library_is_held_to_its_tag() {
        let arm = Platform::by_name("darwin-arm64").unwrap();
        assert_eq!(macos_minimum(&macho(11, 0)).unwrap(), (11, 0));
        assert!(held_to_tag(&arm, &macho(11, 0)).is_ok());
        assert!(held_to_tag(&arm, &macho(10, 15)).is_ok());
        assert!(
            held_to_tag(&arm, &macho(13, 0))
                .unwrap_err()
                .contains("macOS 13.0")
        );
    }

    #[test]
    fn a_record_lists_every_file_and_itself_last() {
        let files = BTreeMap::from([("a.py".to_string(), b"x".to_vec())]);
        assert_eq!(
            record(&files, "t-1.dist-info/RECORD"),
            "a.py,sha256=LXEWQrcmsEQBYnyp-6wy9chTD7GQPMTbAiWHF5IaSIE,1\nt-1.dist-info/RECORD,,\n"
        );
    }

    #[test]
    fn the_metadata_is_the_projects() {
        let project: toml::Table = r#"
            name = "teistro"
            description = "An SDK."
            requires-python = ">=3.11"
            license = "Apache-2.0"
            authors = [{ name = "Teispace", email = "support@teispace.com" }]
            keywords = ["astrology", "calendar"]
            classifiers = ["Typing :: Typed"]
            [urls]
            Homepage = "https://example.org"
            [scripts]
            teistro-install = "teistro._install:main"
        "#
        .parse()
        .unwrap();
        let text = metadata(&project, "1.2.3", "# readme\n");
        for line in [
            "Name: teistro",
            "Version: 1.2.3",
            "Author-email: Teispace <support@teispace.com>",
            "License-Expression: Apache-2.0",
            "Keywords: astrology,calendar",
            "Classifier: Typing :: Typed",
            "Project-URL: Homepage, https://example.org",
            "Requires-Python: >=3.11",
        ] {
            assert!(text.lines().any(|one| one == line), "{line}");
        }
        assert!(text.ends_with("\n\n# readme\n"));
        assert_eq!(
            entry_points(&project),
            "[console_scripts]\nteistro-install = teistro._install:main\n"
        );
    }
}
