//! The generator and the shuffle, `research/shuffle/1` to `/3`
//! (`research.md` §2.2).
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
///
/// ```
/// use teistro_research::ShuffleVersion;
///
/// // The newest walks a small reference set whole.
/// assert_eq!(ShuffleVersion::default(), ShuffleVersion::V3);
/// assert_eq!(serde_json::to_string(&ShuffleVersion::V1)?, r#""research/shuffle/1""#);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ShuffleVersion {
    /// `SplitMix64` keyed per permutation by [`stream_seed`], and
    /// Fisher–Yates drawing each index with Lemire's unbiased bounded
    /// integer; strata shuffled in their key order. An event study's
    /// restricted pairing is drawn by rejection, which can fail for want
    /// of luck.
    #[serde(rename = "research/shuffle/1")]
    V1,
    /// `/1`, except that an event study's restricted pairing is drawn
    /// exactly: the allowed pairings are a Ferrers board (C368), and each
    /// event, earliest first, takes a subject uniformly among those born
    /// before it and not yet taken. Every allowed pairing is equally
    /// likely and none is ever refused for want of luck. Every other draw
    /// is `/1`'s, bit for bit.
    #[serde(rename = "research/shuffle/2")]
    V2,
    /// `/2`, except that a reference set no larger than the permutations
    /// asked is walked whole: every arrangement once, the observed one
    /// among them, so the p-value is exact (`b/N`, Phipson and Smyth
    /// 2010 §3) and costs no more than sampling would have. A larger set
    /// is sampled as under `/2`, bit for bit.
    #[default]
    #[serde(rename = "research/shuffle/3")]
    V3,
}

impl ShuffleVersion {
    /// Every version this build carries, oldest first.
    ///
    /// ```
    /// use teistro_research::ShuffleVersion;
    ///
    /// let names = ShuffleVersion::ALL.map(ShuffleVersion::name);
    /// assert_eq!(names, ["research/shuffle/1", "research/shuffle/2", "research/shuffle/3"]);
    /// ```
    pub const ALL: [ShuffleVersion; 3] =
        [ShuffleVersion::V1, ShuffleVersion::V2, ShuffleVersion::V3];

    /// The name the answer's provenance prints.
    ///
    /// ```
    /// use teistro_research::ShuffleVersion;
    ///
    /// // The name is the serde spelling, so a published answer reruns.
    /// for version in ShuffleVersion::ALL {
    ///     assert_eq!(serde_json::to_value(version)?, version.name());
    /// }
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            ShuffleVersion::V1 => "research/shuffle/1",
            ShuffleVersion::V2 => "research/shuffle/2",
            ShuffleVersion::V3 => "research/shuffle/3",
        }
    }

    /// Whether a reference set small enough is walked whole.
    ///
    /// ```
    /// use teistro_research::ShuffleVersion;
    ///
    /// assert!(ShuffleVersion::V3.enumerates() && !ShuffleVersion::V2.enumerates());
    /// ```
    #[must_use]
    pub const fn enumerates(self) -> bool {
        matches!(self, ShuffleVersion::V3)
    }

    /// Whether an event study's restricted pairing is drawn exactly
    /// rather than by rejection.
    ///
    /// ```
    /// use teistro_research::ShuffleVersion;
    ///
    /// // Only the first draws a restricted pairing by rejection.
    /// assert!(!ShuffleVersion::V1.exact_pairing());
    /// assert!(ShuffleVersion::V2.exact_pairing() && ShuffleVersion::V3.exact_pairing());
    /// ```
    #[must_use]
    pub const fn exact_pairing(self) -> bool {
        !matches!(self, ShuffleVersion::V1)
    }
}

/// What a sampler draws its choices from: a generator, or the digits of
/// an arrangement's rank.
pub(crate) trait Draw {
    /// A choice in `0..bound`.
    fn below(&mut self, bound: u64) -> u64;
}

impl Draw for SplitMix64 {
    fn below(&mut self, bound: u64) -> u64 {
        SplitMix64::below(self, bound)
    }
}

/// An arrangement's rank read as mixed-radix digits, the least
/// significant first. A sampler whose `bound`s never depend on the
/// choices before them (Fisher–Yates, a Ferrers board's pools, each
/// person's instants) is then a bijection from the ranks `0..∏ bound` to
/// its arrangements.
pub(crate) struct Digits(pub(crate) u64);

impl Draw for Digits {
    fn below(&mut self, bound: u64) -> u64 {
        let digit = self.0 % bound;
        self.0 /= bound;
        digit
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
///
/// ```
/// use teistro_research::{SplitMix64, stream_seed};
///
/// // Each permutation's stream depends on the seed and its index alone.
/// assert_ne!(stream_seed(8, 0), stream_seed(8, 1));
/// assert_eq!(SplitMix64::new(stream_seed(8, 5)).next_u64(), SplitMix64::for_permutation(8, 5).next_u64());
/// ```
#[must_use]
pub const fn stream_seed(seed: u64, k: u64) -> u64 {
    finalise(seed ^ finalise(k.wrapping_add(GOLDEN)))
}

/// `SplitMix64`: a 64-bit state advanced by `GOLDEN` and finalised.
///
/// ```
/// use teistro_research::SplitMix64;
///
/// // The published sequence from a zero seed.
/// let mut generator = SplitMix64::new(0);
/// assert_eq!(generator.next_u64(), 0xE220_A839_7B1D_CDAF);
/// assert_eq!(generator.next_u64(), 0x6E78_9E6A_A1B9_65F4);
/// ```
#[derive(Clone, Debug)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// A generator starting at `seed`.
    ///
    /// ```
    /// use teistro_research::SplitMix64;
    ///
    /// // The same seed, the same stream.
    /// let (mut a, mut b) = (SplitMix64::new(42), SplitMix64::new(42));
    /// assert_eq!(a.next_u64(), b.next_u64());
    /// ```
    #[must_use]
    pub const fn new(seed: u64) -> SplitMix64 {
        SplitMix64 { state: seed }
    }

    /// Permutation `k`'s generator under `seed` ([`stream_seed`]).
    ///
    /// ```
    /// use teistro_research::SplitMix64;
    ///
    /// let (mut third, mut fourth) = (SplitMix64::for_permutation(8, 3), SplitMix64::for_permutation(8, 4));
    /// assert_ne!(third.next_u64(), fourth.next_u64());
    /// ```
    #[must_use]
    pub const fn for_permutation(seed: u64, k: u64) -> SplitMix64 {
        SplitMix64::new(stream_seed(seed, k))
    }

    /// The next 64 bits.
    ///
    /// ```
    /// use teistro_research::SplitMix64;
    ///
    /// let mut generator = SplitMix64::new(0);
    /// let _ = generator.next_u64();
    /// // The third of the published sequence.
    /// let _ = generator.next_u64();
    /// assert_eq!(generator.next_u64(), 0x06C4_5D18_8009_454F);
    /// ```
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(GOLDEN);
        finalise(self.state)
    }

    /// A uniform integer in `0..bound`, without bias: Lemire's
    /// multiply-shift, rejecting the low products that would favour
    /// some outcomes ("Fast random integer generation in an interval",
    /// *ACM TOMACS* 29, 2019).
    ///
    /// ```
    /// use teistro_research::SplitMix64;
    ///
    /// let mut generator = SplitMix64::new(9);
    /// assert!((0..1_000).all(|_| generator.below(6) < 6));
    /// assert_eq!(SplitMix64::new(9).below(1), 0);
    /// ```
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
    ///
    /// ```
    /// use teistro_research::SplitMix64;
    ///
    /// // A permutation of the same members, the same one for the same seed.
    /// let mut items: Vec<u8> = (0..10).collect();
    /// SplitMix64::new(8).shuffle(&mut items);
    /// let mut again: Vec<u8> = (0..10).collect();
    /// SplitMix64::new(8).shuffle(&mut again);
    /// assert_eq!(items, again);
    /// items.sort_unstable();
    /// assert_eq!(items, (0..10).collect::<Vec<u8>>());
    /// ```
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.below(i as u64 + 1);
            #[allow(clippy::cast_possible_truncation)]
            items.swap(i, j as usize);
        }
    }
}
