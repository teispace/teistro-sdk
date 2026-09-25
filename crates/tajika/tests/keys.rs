//! Each hand-written `key()` holds to serde's spelling of the same member:
//! the key a Rust caller prints is the one every binding reads back and
//! every stored document holds.

#![allow(
    clippy::unwrap_used,
    reason = "a test unwraps what it builds and fails by panicking"
)]

use serde::Serialize;
use teistro_tajika::{Chosen, Reading, Saham, StrongClause, WeakClause, YearYoga};

/// Every member's `key()` against the string serde writes for it.
fn holds<T: Serialize + Copy>(members: &[T], key: fn(T) -> &'static str) {
    for &member in members {
        let written = serde_json::to_value(member).unwrap();
        assert_eq!(written.as_str(), Some(key(member)));
    }
}

#[test]
fn every_key_is_the_one_serde_writes() {
    holds(&YearYoga::ALL, YearYoga::key);
    holds(&Saham::ALL, Saham::key);
    holds(&StrongClause::ALL, StrongClause::key);
    holds(&WeakClause::ALL, WeakClause::key);
    holds(&Chosen::ALL, Chosen::key);
    holds(&Reading::ALL, Reading::key);
}
