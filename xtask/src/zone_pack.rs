//! `cargo xtask zone-pack --from DIR --version V --out FILE`: a zone pack
//! from a tree `zic` compiled (`docs/03-design/zone-packs.md` §3). The
//! caller signs the file with its own key.

use std::fs;
use std::path::{Path, PathBuf};

use teistro_time::ZonePack;

/// What `TZif` begins with (RFC 9636 §3.1).
const TZIF_MAGIC: &[u8] = b"TZif";

/// What the command takes.
const USAGE: &str = "usage: xtask zone-pack --from DIR --version V --out FILE";

/// The trees `zic` may write beside the zones, of the same zones under
/// another clock.
const NOT_ZONES: [&str; 2] = ["posix", "right"];

/// Every `TZif` file under `dir`, as `(name, bytes)`; a `.tab` or a
/// `leapseconds` beside them is no zone.
fn compiled(root: &Path, dir: &Path, into: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|why| format!("{}: {why}", dir.display()))?;
    for entry in entries {
        let path = entry.map_err(|why| why.to_string())?.path();
        let name = path
            .strip_prefix(root)
            .map_err(|why| why.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        if path.is_dir() {
            if !NOT_ZONES.contains(&name.as_str()) {
                compiled(root, &path, into)?;
            }
            continue;
        }
        let bytes = fs::read(&path).map_err(|why| format!("{}: {why}", path.display()))?;
        if bytes.starts_with(TZIF_MAGIC) {
            into.push((name, bytes));
        }
    }
    Ok(())
}

/// The pack the arguments name, written.
fn write(args: &[String]) -> Result<(), String> {
    let (mut from, mut version, mut out) = (None, None, None);
    let mut pairs = args.iter();
    while let Some(arg) = pairs.next() {
        let value = pairs.next().ok_or(USAGE)?;
        match arg.as_str() {
            "--from" => from = Some(PathBuf::from(value)),
            "--version" => version = Some(value.as_str()),
            "--out" => out = Some(PathBuf::from(value)),
            _ => return Err(USAGE.to_string()),
        }
    }
    let (Some(from), Some(version), Some(out)) = (from, version, out) else {
        return Err(USAGE.to_string());
    };
    let mut zones = Vec::new();
    compiled(&from, &from, &mut zones)?;
    let bytes = ZonePack::write(
        version,
        zones
            .iter()
            .map(|(name, tzif)| (name.as_str(), tzif.as_slice())),
    );
    // Read back, so a file this SDK would refuse is never written.
    ZonePack::from_bytes(&bytes).map_err(|why| why.to_string())?;
    fs::write(&out, &bytes).map_err(|why| format!("{}: {why}", out.display()))?;
    println!(
        "{}: {} names, {} bytes, release {version}",
        out.display(),
        zones.len(),
        bytes.len()
    );
    Ok(())
}

/// `cargo xtask zone-pack`.
pub(crate) fn run(args: &[String]) -> i32 {
    match write(args.get(1..).unwrap_or_default()) {
        Ok(()) => 0,
        Err(why) => {
            eprintln!("zone-pack: {why}");
            1
        }
    }
}
