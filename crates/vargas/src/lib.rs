//! The divisional charts: one table-driven evaluator, twenty-one rows.
//!
//! A varga answers "which sign does this longitude fall in, in this
//! chart", and every one of the twenty-one the SDK ships answers it the
//! same way: sort the sign into a group, take the part of the sign the
//! longitude falls in, and either step through the signs from somewhere
//! or read the answer off a list.
//!
//! That is a strong claim, and unlike almost everything else the SDK
//! computes it can be **decided outright** — a divisional chart is a
//! function of one sidereal longitude, and the conformance corpus records
//! both the longitude and the answer. `cargo xtask vargas` derives each
//! chart's table from the corpus, independently of this crate, and holds
//! the rows to it: 19 530 recorded placements over 93 fixtures and two
//! zodiacs, with nothing left over
//! (`03-design/varga-tables-measured.md`, over
//! `03-design/varga-kernel.md`).
//!
//! Three things worth knowing before reading the code:
//!
//! - **The spans belong to the group.** D30's odd signs are cut 5, 5, 8,
//!   7, 5 degrees and its even signs the same widths reversed, so one
//!   chart has two span rules and `spans` sits inside a [`scheme::Group`].
//! - **`divisions` names the chart** and is not always its part count:
//!   D30 is called thirty and cuts a sign into five.
//! - **The part index is integer arithmetic** on the canonical angle
//!   (ADR-0016). The recording engine computes it as `floor(deg / (30/N))`
//!   in floating point, and none of 30/7, 30/11, 30/27 or 0.2 is
//!   representable, so a body exactly on a part boundary can land either
//!   side of it depending on the platform. Here it cannot.
//!
//! ```
//! use teistro_core::angle::Nas;
//! use teistro_core::catalogue::{Rashi, Varga};
//! use teistro_core::quantity::Degrees;
//! use teistro_vargas::{Scheme, place, sign};
//!
//! // Eleven and a half degrees of Taurus: the fourth navamsha of a sign
//! // that begins its own at Capricorn, so the body lands in Aries.
//! let longitude = Nas::from_degrees(Degrees::try_new(41.5).expect("finite"));
//! assert_eq!(sign(&Scheme::of(Varga::D9), longitude), Rashi::Aries);
//!
//! // An arbitrary chart the texts do not name takes a stated convention.
//! let d37 = Scheme::cyclic(37).expect("inside the limit");
//! assert_eq!(d37.key(), "D37");
//! assert_eq!(place(&d37, longitude).part, 14);
//! ```

pub mod change;
pub mod chart;
pub mod evaluate;
pub mod scheme;

pub use chart::{Axis, GrahaPlacement, VargaChart, chart, every_chart};
pub use evaluate::{Placement, place, sign, table};
pub use scheme::{Classifier, Group, MOST_DIVISIONS, Map, SCHEMES, Scheme, SignBase, Spans};
