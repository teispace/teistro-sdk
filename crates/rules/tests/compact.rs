//! **The embedded packs are the repository's** (`build.rs`): each shipped
//! rule pack and table is embedded without the whitespace between its
//! tokens, and reads to the same document as the file laid out for reading.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "a test fails by panicking"
)]

use std::fs;
use std::path::Path;

#[test]
fn every_compacted_pack_reads_as_its_source_and_is_smaller() {
    let mut files = 0;
    for dir in ["rules", "tables"] {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join(dir);
        let compacted = Path::new(env!("OUT_DIR")).join(dir);
        for entry in fs::read_dir(&source).expect("the source directory reads") {
            let path = entry.expect("an entry reads").path();
            if path.extension().is_none_or(|extension| extension != "json") {
                continue;
            }
            let name = path.file_name().expect("a file has a name");
            let laid_out = fs::read_to_string(&path).expect("the source reads");
            let embedded = fs::read_to_string(compacted.join(name))
                .unwrap_or_else(|error| panic!("{name:?} was not compacted: {error}"));
            let read = |text: &str| -> serde_json::Value {
                serde_json::from_str(text).unwrap_or_else(|error| panic!("{name:?}: {error}"))
            };
            assert_eq!(read(&embedded), read(&laid_out), "{name:?}");
            assert!(embedded.len() < laid_out.len(), "{name:?}");
            files += 1;
        }
    }
    assert!(files > 40, "{files} packs found");
}
