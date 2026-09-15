//! Dashas as rows over a kernel, the balance at birth, and the period tree
//! read without building it (Phase 5; `docs/03-design/dasha-kernels.md`,
//! measured in `docs/03-design/dasha-measured.md`).
//!
//! - [`row`]: a nakshatra-seeded system as data, its lords and years and the
//!   map from the Moon's nakshatra to its first lord;
//! - [`balance`]: what remains of the first period, spatially or temporally,
//!   and how it is written;
//! - [`tree`]: the dasha of a birth, its periods by path, and the chain
//!   running at an instant, through the [`Timeline`] every kind shares;
//! - [`rashi`]: the sign-based (Jaimini) systems as rows over their own
//!   kernel, measured in `docs/03-design/rashi-dashas-measured.md`;
//! - [`reading`]: a dasha as a chart document carries it, its periods as
//!   rows to the settings' depth.
//!
//! Three things the corpus settled that a reading of the texts does not
//! (`dasha-measured.md`):
//!
//! - **the birth period's sub-periods are compressed** into its balance by
//!   default, and sized against the whole period on request (crux C48);
//! - **the cycle ends** after its last lord by default, and repeats on
//!   request;
//! - **a boundary is an exact share of its parent**, so children partition
//!   their parent and nothing accumulates down the tree.
//!
//! ```
//! use teistro_core::angle::Nas;
//! use teistro_core::catalogue::{DashaSystem, Graha};
//! use teistro_core::quantity::{Degrees, Depth, JulianDay};
//! use teistro_core::settings::root;
//! use teistro_dasha::{Birth, Dasha, Rules, Timeline, VIMSHOTTARI};
//!
//! // A Moon a third of the way into Anuradha, Saturn's nakshatra.
//! let moon = Nas::from_degrees(Degrees::try_new(16.0 * 360.0 / 27.0 + 360.0 / 81.0)?);
//! let birth = Birth { instant: JulianDay::literal(2_447_995.5), moon, moon_span: None };
//! let rules = Rules::of(&root().dasha, DashaSystem::Vimshottari);
//! let dasha = Dasha::new(&VIMSHOTTARI, &birth, rules)?;
//!
//! // Two thirds of Saturn's nineteen years remain.
//! assert_eq!(dasha.balance().written.years, 12);
//! // Twenty years on: the mahadasha, antardasha and pratyantardasha.
//! let later = JulianDay::literal(2_447_995.5 + 20.0 * 365.25);
//! let chain = dasha.at(later, Depth::try_new(3)?);
//! assert_eq!(chain.iter().next().map(|period| period.lord), Some(Graha::Mercury));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod balance;
pub mod rashi;
pub mod reading;
pub mod row;
pub mod tree;

pub use balance::{BalanceAtBirth, Written};
pub use rashi::{Footedness, Parity, RASHI_ROWS, RashiChart, RashiDasha, RashiRow, rashi_row};
pub use reading::{DashaReading, PeriodRow};
pub use row::{Count, Lord, ROWS, Seat, UduRow, VIMSHOTTARI, row};
pub use tree::{Birth, Chain, Dasha, MAX_DEPTH, Path, Period, Rules, Timeline};
