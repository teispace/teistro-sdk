//! Every recorded arudha pada of the conformance corpus, through this crate
//! (`docs/03-design/arudhas-measured.md`): all twelve houses of every chart
//! that records them, their first count, their exception, and the arudha
//! lagna the foundation records again.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index JSON by key"
)]

use std::path::Path;

use serde_json::Value;
use teistro_core::catalogue::{Graha, Rashi};
use teistro_points::arudha::arudha_padas;

fn sign(value: &Value) -> Rashi {
    Rashi::from_id(u16::try_from(value.as_u64().unwrap()).unwrap()).unwrap()
}

#[test]
fn every_recorded_pada_is_reproduced() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/baseline");
    let (mut charts, mut padas) = (0, 0);
    for dir in ["charts", "variants"] {
        for entry in std::fs::read_dir(root.join(dir)).unwrap().flatten() {
            let fixture: Value =
                serde_json::from_str(&std::fs::read_to_string(entry.path()).unwrap()).unwrap();
            let Some(recorded) = fixture["houses"]["arudha_padas"].as_array() else {
                continue;
            };
            if fixture["positions"].is_null() || fixture["foundation"].is_null() {
                continue;
            }
            charts += 1;
            let name = entry.file_name();
            let bodies = &fixture["positions"]["bodies"];
            let lagna = sign(&fixture["foundation"]["lagna"]["sign_index"]);
            let ours = arudha_padas(lagna, |graha: Graha| {
                sign(&bodies[graha.key()]["sign_index"])
            });
            assert_eq!(recorded.len(), 12, "{name:?}");
            for (pada, want) in ours.iter().zip(recorded) {
                padas += 1;
                assert_eq!(u64::from(pada.house), want["house"].as_u64().unwrap());
                assert_eq!(
                    pada.sign,
                    sign(&want["sign_index"]),
                    "{name:?} house {}",
                    pada.house
                );
                assert_eq!(
                    pada.counted,
                    sign(&want["original_sign_index"]),
                    "{name:?} house {}",
                    pada.house
                );
                assert_eq!(
                    Some(pada.moved),
                    want["exception_applied"].as_bool(),
                    "{name:?} house {}",
                    pada.house
                );
            }
            assert_eq!(
                ours[0].sign,
                sign(&fixture["foundation"]["arudha_lagna_sign_index"]),
                "{name:?}"
            );
        }
    }
    assert_eq!((charts, padas), (71, 852));
}
