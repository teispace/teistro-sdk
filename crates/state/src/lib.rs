//! What a graha *is*, as opposed to where it is.
//!
//! Where a graha stands the foundation answers. Whether it is strong or
//! ruined, at home or in exile, burnt by the Sun, going backwards, at war
//! with a neighbour, awake or asleep is this crate, and almost everything
//! above reads it: the strengths weigh it, the rules test it, the
//! interpretation says it.
//!
//! Every rule here was measured against the corpus's 837 recorded
//! readings before it was written
//! (`03-design/state-tables-measured.md`, over
//! `03-design/state-and-avasthas.md`). Nothing needs a provider: it is
//! all arithmetic over a founded chart, which is why the whole crate is
//! testable against the corpus.
//!
//! Four things worth knowing:
//!
//! - **Moolatrikona is checked before exaltation.** Three grahas have a
//!   moolatrikona span inside their exaltation sign.
//! - **A body in its own sign is its own friend.** The catalogue cannot
//!   say it — a graha is in none of its own three lists — so the rule
//!   lives here.
//! - **The shadow grahas take a reduced ladder** that ends at neutral.
//! - **Deep combustion needs a table that gives a deeper orb.** The
//!   default profile names the Surya Siddhanta's, which gives one orb
//!   per body and nothing inside it, so under it a body is combust or it
//!   is not ([`burn`]).
//! - **Six avasthas are not decidable**, and the crate reports nothing
//!   where it cannot decide rather than a plausible guess. They are the
//!   ones whose definitions read "or aspected by". Three of them — the
//!   lajjitadi — now carry a *necessary* condition measured over the
//!   whole corpus, so the answer is [`Holds::No`] where that fails and
//!   [`Holds::Undecided`] only where it holds: 1301 of 1953 questions
//!   the module used to leave open are now answered
//!   (`03-design/aspect-drishti-measured.md` §7).
//!
//! ```
//! use teistro_core::catalogue::{Dignity, Graha, Rashi, Relationship};
//! use teistro_state::dignity::{dignity, natural};
//!
//! // The Sun in Leo is in its own sign — and its own friend, which no
//! // table can say.
//! assert_eq!(natural(Graha::Sun, Rashi::Leo), Relationship::Friend);
//! // Leo 0° to 20° is its moolatrikona, and past that its own sign.
//! assert_eq!(
//!     dignity(Graha::Sun, Rashi::Leo, 10.0, Relationship::Friend),
//!     Dignity::Mooltrikona
//! );
//! assert_eq!(
//!     dignity(Graha::Sun, Rashi::Leo, 25.0, Relationship::Friend),
//!     Dignity::OwnSign
//! );
//! ```

pub mod avastha;
pub mod boundary;
pub mod burn;
pub mod chart;
pub mod dignity;

pub use avastha::{AtWar, Holds, Lajjitadi, Placement, War};
pub use boundary::Boundaries;
pub use burn::{Applied, Burning, Combustion, Orbs};
pub use chart::{GrahaState, Motion, state};
pub use dignity::Friendship;
