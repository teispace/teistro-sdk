//! The lords of a longitude at every level, found exactly.

use serde::{Deserialize, Serialize};
use teistro_core::Nas;
use teistro_core::catalogue::Graha;
use teistro_dasha::{Lord, VIMSHOTTARI_LORDS, VIMSHOTTARI_YEARS};

/// The deepest level [`chain`] reaches: the star and eight levels below it.
///
/// Nine levels down the shortest part is still 1 900 nanoarcseconds wide,
/// and the scaled comparison stays inside `i128`. Practice stops far
/// sooner: the Readers judge by the sub, and software offers the sub-sub
/// and one or two levels more.
pub const MAX_LEVELS: usize = 9;

/// The arc one level spans: the first nanoarcsecond inside it and the
/// first past it.
///
/// A sub's edges are whole nanoarcseconds (its lord's years × 6′40″); a
/// deeper level's generally fall between two, and `start` and `end` are
/// then the first whole ones on or after each edge, so a longitude is in
/// the level exactly when [`Span::contains`] says it is. `end` wraps to 0°
/// for the last part of Revati.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    /// The first longitude inside the level.
    pub start: Nas,
    /// The first longitude past it.
    pub end: Nas,
}

impl Span {
    /// How wide the level is, in nanoarcseconds.
    #[must_use]
    pub const fn width(self) -> i64 {
        self.start.arc_to(self.end).get()
    }

    /// Whether a longitude is in the level.
    #[must_use]
    pub const fn contains(self, at: Nas) -> bool {
        self.start.arc_to(at).get() < self.width()
    }

    /// The arc from a longitude inside the level to its nearer edge, in
    /// nanoarcseconds: back to `start` or on to `end`, whichever is
    /// shorter.
    ///
    /// What a sub lord's reliability turns on. The lagna moves about a
    /// degree in four minutes, so a birth time known to the minute cannot
    /// settle a sub whose margin is a few arcminutes, and a rectification
    /// reads this to know which way the lord changes first.
    #[must_use]
    pub fn margin(self, at: Nas) -> i64 {
        debug_assert!(self.contains(at), "{at:?} is outside {self:?}");
        self.start.arc_to(at).get().min(at.arc_to(self.end).get())
    }
}

/// One level of a longitude's chain: its lord and the arc it spans.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Level {
    /// The Vimshottari lord ruling the level.
    pub lord: Graha,
    /// The arc the level spans.
    pub span: Span,
}

impl Level {
    /// A slot before it is filled.
    const UNFILLED: Level = Level {
        lord: Graha::Ketu,
        span: Span {
            start: Nas::ZERO,
            end: Nas::ZERO,
        },
    };
}

/// A longitude's lords as KP reads them: the sign's, the star's, the
/// sub's and the sub-sub's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Lords {
    /// The lord of the sign.
    pub sign: Graha,
    /// The lord of the nakshatra, and the nakshatra's arc.
    pub star: Level,
    /// The lord of the sub, the level the Readers judge by, and its arc.
    pub sub: Level,
    /// The lord of the sub-sub, and its arc.
    pub sub_sub: Level,
}

/// A longitude's lords as KP reads them.
///
/// ```
/// use teistro_core::Nas;
/// use teistro_core::catalogue::Graha;
///
/// // 0° Aries opens everything: Mars's sign, Ketu's star, Ketu's sub.
/// let lords = teistro_kp::lords(Nas::ZERO);
/// assert_eq!(lords.sign, Graha::Mars);
/// assert_eq!(lords.sub.lord, Graha::Ketu);
/// // Ketu's sub is Ketu's 7 years of the 120: 7 × 6′40″ = 46′40″.
/// assert_eq!(lords.sub.span.width(), 46 * Nas::PER_ARCMINUTE + 40 * Nas::PER_ARCSECOND);
/// ```
#[must_use]
pub fn lords(at: Nas) -> Lords {
    let mut levels = [Level::UNFILLED; 3];
    descend(at, &mut levels);
    let [star, sub, sub_sub] = levels;
    Lords {
        sign: at.sign().attributes().lord,
        star,
        sub,
        sub_sub,
    }
}

/// The first `N` levels of a longitude's chain, the star's first.
///
/// `N` is 1 to [`MAX_LEVELS`], checked when the call is compiled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chain<const N: usize> {
    levels: [Level; N],
}

impl<const N: usize> Chain<N> {
    /// The levels, the star's first and the deepest last.
    #[must_use]
    pub const fn levels(&self) -> &[Level; N] {
        &self.levels
    }

    /// The deepest level asked for.
    #[must_use]
    #[allow(
        clippy::indexing_slicing,
        reason = "`chain` is the only way to build one, and it refuses an `N` of 0"
    )]
    pub const fn deepest(&self) -> Level {
        self.levels[N - 1]
    }
}

impl<const N: usize> Serialize for Chain<N> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(&self.levels)
    }
}

/// The first `N` levels of a longitude's chain: `chain::<2>` is the star
/// and the sub, `chain::<5>` reaches two levels past the sub-sub.
///
/// ```
/// use teistro_core::Nas;
/// use teistro_core::catalogue::Graha;
///
/// let at = Nas::new(69 * Nas::PER_DEGREE);
/// let chain = teistro_kp::chain::<5>(at);
/// let lords: Vec<Graha> = chain.levels().iter().map(|level| level.lord).collect();
/// assert_eq!(&lords[..3], [Graha::Rahu, Graha::Jupiter, Graha::Saturn]);
/// assert!(chain.deepest().span.contains(at));
/// ```
#[must_use]
pub fn chain<const N: usize>(at: Nas) -> Chain<N> {
    const {
        assert!(
            N >= 1 && N <= MAX_LEVELS,
            "a chain is 1 to MAX_LEVELS levels"
        );
    }
    let mut levels = [Level::UNFILLED; N];
    descend(at, &mut levels);
    Chain { levels }
}

/// The `k`th lord of the Vimshottari sequence, counted round from Ketu.
#[allow(clippy::indexing_slicing, reason = "reduced modulo the length")]
const fn nth(k: usize) -> Lord {
    VIMSHOTTARI_LORDS[k % VIMSHOTTARI_LORDS.len()]
}

/// Fills `levels` from the star down.
///
/// Everything is in nanoarcseconds from the nakshatra's start, scaled by
/// 120 once per level below the star, so every edge down to the deepest
/// asked for is an integer: a part is its parent's width / 120 × its
/// lord's years, and the parent's width carries a factor of 120 for every
/// level still to come. Each level's nine parts start from its own lord.
fn descend(at: Nas, levels: &mut [Level]) {
    let whole = i128::from(VIMSHOTTARI_YEARS);
    let below = u32::try_from(levels.len().saturating_sub(1)).unwrap_or(u32::MAX);
    let scale = whole.pow(below);
    let inside = at.in_nakshatra();
    let base = Nas::new(at.get() - inside.get());
    let position = i128::from(inside.get()) * scale;
    let mut first = usize::from(at.nakshatra_index().get());
    let mut start = 0_i128;
    let mut width = i128::from(Nas::PER_NAKSHATRA) * scale;
    for (depth, level) in levels.iter_mut().enumerate() {
        if depth > 0 {
            let parent = width;
            for step in 0..VIMSHOTTARI_LORDS.len() {
                width = parent / whole * i128::from(nth(first + step).years);
                // The last part ends where its parent does.
                if position < start + width || step + 1 == VIMSHOTTARI_LORDS.len() {
                    first += step;
                    break;
                }
                start += width;
            }
        }
        *level = Level {
            lord: nth(first).graha,
            span: Span {
                start: edge(base, start, scale),
                end: edge(base, start + width, scale),
            },
        };
    }
}

/// The first whole nanoarcsecond on or after a scaled edge.
#[allow(
    clippy::cast_possible_truncation,
    reason = "the quotient is at most one nakshatra"
)]
fn edge(base: Nas, scaled: i128, scale: i128) -> Nas {
    let whole = (scaled + scale - 1) / scale;
    Nas::new(base.get() + whole as i64)
}
