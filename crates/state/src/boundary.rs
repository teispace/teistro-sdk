//! How near a body stands to a classification boundary.
//!
//! The type itself is [`teistro_core::boundary`], because `aspect` wants
//! the same fact about the same angle and a second copy would be a
//! second thing to get wrong. It is re-exported here because this is
//! where the corpus measured it (`03-design/state-tables-measured.md`
//! §8) and where a reader of the planetary state looks for it.
//!
//! **The SDK ships no threshold.** It reports the distance, which is
//! always a fact, and a caller asks whether that is inside whatever
//! tolerance its own provider claims. A constant compiled into the
//! library could not answer the question for two providers of different
//! accuracy, and the corpus's own tolerance file already frames it that
//! way: a classification within the longitude tolerance of a boundary is
//! an edge case and not a failure.

pub use teistro_core::boundary::Boundaries;
