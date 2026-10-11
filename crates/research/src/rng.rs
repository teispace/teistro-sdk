//! The generator and the shuffle, `research/shuffle/1` (`research.md` §2.2).
//!
//! Both are part of the answer's contract, so they are written here and
//! never taken from a dependency whose minor release could move a
//! published p-value. A change to either is a new [`ShuffleVersion`];
//! the old one stays selectable so a published study can be rerun.

use serde::{Deserialize, Serialize};

/// The golden-ratio increment of `SplitMix64` (Steele, Lea and Flood,
/// "Fast splittable pseudorandom number generators", OOPSLA 2014).
const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;

/// The generator and shuffle a test ran under. It is in the answer and
/// in the request's hash, so a reader can rerun a published study.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ShuffleVersion {
    /// `SplitMix64` keyed per permutation by [`stream_seed`], and
    /// Fisher–Yates drawing each index with Lemire's unbiased bounded
    /// integer; strata shuffled in their key order.
    #[default]
    #[serde(rename = "research/shuffle/1")]
    V1,
}

impl ShuffleVersion {
    /// Every version this build carries, oldest first.
    pub const ALL: [ShuffleVersion; 1] = [ShuffleVersion::V1];

    /// The name the answer's provenance prints.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            ShuffleVersion::V1 => "research/shuffle/1",
        }
    }
}

/// `SplitMix64`'s output function (Stafford's "Mix13" variant), a
/// bijection on 64 bits.
#[must_use]
const fn finalise(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// The seed of permutation `k`'s stream under `seed`.
///
/// A permutation's draws depend on the study's seed and its own index
/// alone, never on how many permutations came before it or on which
/// thread drew it, which is what makes the answer the same at every
/// thread count.
#[must_use]
pub const fn stream_seed(seed: u64, k: u64) -> u64 {
    finalise(seed ^ finalise(k.wrapping_add(GOLDEN)))
}

/// `SplitMix64`: a 64-bit state advanced by [`GOLDEN`] and finalised.
#[derive(Clone, Debug)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// A generator starting at `seed`.
    #[must_use]
    pub const fn new(seed: u64) -> SplitMix64 {
        SplitMix64 { state: seed }
    }

    /// Permutation `k`'s generator under `seed` ([`stream_seed`]).
    #[must_use]
    pub const fn for_permutation(seed: u64, k: u64) -> SplitMix64 {
        SplitMix64::new(stream_seed(seed, k))
    }

    /// The next 64 bits.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(GOLDEN);
        finalise(self.state)
    }

    /// A uniform integer in `0..bound`, without bias: Lemire's
    /// multiply-shift, rejecting the low products that would favour
    /// some outcomes ("Fast random integer generation in an interval",
    /// *ACM TOMACS* 29, 2019).
    ///
    /// # Panics
    ///
    /// When `bound` is zero, which no caller here can pass.
    pub fn below(&mut self, bound: u64) -> u64 {
        assert!(bound > 0, "an empty range has no member to draw");
        let mut product = u128::from(self.next_u64()) * u128::from(bound);
        // The low half, which the high half's bias lives in.
        #[allow(clippy::cast_possible_truncation)]
        let mut low = product as u64;
        if low < bound {
            let threshold = bound.wrapping_neg() % bound;
            while low < threshold {
                product = u128::from(self.next_u64()) * u128::from(bound);
                #[allow(clippy::cast_possible_truncation)]
                {
                    low = product as u64;
                }
            }
        }
        #[allow(clippy::cast_possible_truncation)]
        let high = (product >> 64) as u64;
        high
    }

    /// Fisher–Yates, from the last position down (Durstenfeld's form),
    /// each swap's partner drawn by [`below`](Self::below).
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.below(i as u64 + 1);
            #[allow(clippy::cast_possible_truncation)]
            items.swap(i, j as usize);
        }
    }
}
