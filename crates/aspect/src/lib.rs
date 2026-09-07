//! Which bodies reach which, and how strongly.
//!
//! Three relations, and the module's first job is to keep them apart,
//! because they are not variants of one thing:
//!
//! - the **graha drishti** ([`drishti`]), a body's gaze counted in
//!   houses from the sign it stands in, full or in quarters;
//! - the **rashi drishti** ([`rashi`]), a relation between *signs* that
//!   the Jaimini reading works in, and which is always mutual;
//! - the **conjunction** ([`conjunction`]), which is co-presence in a
//!   sign with no orb at all, and beneath it the [`orb`] engine that
//!   measures a separation against a stated tolerance for the traditions
//!   that do use one.
//!
//! Over the conformance corpus's 6696 ordered pairs of bodies the first
//! two agree on 4053 and each sees relations the other does not, so a
//! module that quietly offered one where a caller asked for the other
//! would be wrong about a third of the time.
//!
//! **This is the first module of Phase 4 the corpus cannot check.** It
//! records no aspect of any kind — the falsification pass established
//! that by searching every key of all 115 fixture files — so the rules
//! here rest on the project's own research page and the tests carry the
//! weight the corpus usually would: the space of signs, houses and
//! grahas is small enough to exhaust, and it is exhausted rather than
//! sampled (`03-design/aspect-drishti-measured.md`, over
//! `03-design/aspect-and-drishti.md`).
//!
//! Three things worth knowing:
//!
//! - **A drishti is counted from the sign**, which is what "the seventh
//!   from it" means. Counted from the bhava it is a different relation.
//! - **A mutual full aspect is the seventh, or Mars and Saturn three
//!   signs apart.** The pass proposed the first alone and the
//!   measurement refused it.
//! - **The sphuta drishti does not ship.** No source in this project
//!   gives its construction, so [`drishti::Strength::virupas`] is the
//!   whole-sign value and nothing interpolates between the houses (crux
//!   C45).
//!
//! ```
//! use teistro_aspect::drishti::{Strength, between};
//! use teistro_aspect::rashi;
//! use teistro_core::catalogue::{Graha, Rashi};
//!
//! // Saturn in Aries reaches Gemini — its third — in full, which only
//! // Saturn does.
//! assert_eq!(between(Graha::Saturn, Rashi::Aries, Rashi::Gemini), Strength::Full);
//! assert_eq!(between(Graha::Sun, Rashi::Aries, Rashi::Gemini), Strength::Quarter);
//! // The Jaimini reading is about the signs alone, and looks both ways.
//! assert!(rashi::aspects(Rashi::Aries, Rashi::Leo));
//! assert!(rashi::aspects(Rashi::Leo, Rashi::Aries));
//! ```

pub mod chart;
pub mod conjunction;
pub mod drishti;
pub mod orb;
pub mod rashi;

pub use chart::{Aspects, Drishti, Mutual};
pub use drishti::Strength;
pub use orb::{Angle, Hit, Moving};
