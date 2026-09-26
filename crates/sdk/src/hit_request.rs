//! The transit hit list through the façade: every ingress and station of a
//! window, against one chart (`03-design/transit-hit-list.md`).

use teistro_core::catalogue::Graha;
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_gochar::GRAHAS;

/// A kind of event a hit list reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum HitKind {
    /// A graha entering a sign.
    SignIngress,
    /// A graha entering a nakshatra.
    NakshatraIngress,
    /// A graha standing still, turning retrograde or direct.
    Station,
}

impl HitKind {
    /// Every kind, the default a request asks for.
    pub const ALL: [HitKind; 3] = [
        HitKind::SignIngress,
        HitKind::NakshatraIngress,
        HitKind::Station,
    ];
}

/// The window to search, whose events and which grahas'.
///
/// ```
/// use teistro::catalogue::Graha;
/// use teistro::quantity::{JulianDay, Utc};
/// use teistro::{HitKind, HitRequest};
///
/// // Saturn's and Jupiter's sign changes and stations over ten years.
/// let asked = HitRequest::between(JulianDay::<Utc>::literal(2_460_676.5), JulianDay::literal(2_464_329.0))
///     .with_grahas([Graha::Saturn, Graha::Jupiter])
///     .with_kinds([HitKind::SignIngress, HitKind::Station]);
/// assert_eq!(asked.grahas().len(), 2);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct HitRequest {
    from: JulianDay<Utc>,
    to: JulianDay<Utc>,
    grahas: Vec<Graha>,
    kinds: Vec<HitKind>,
}

impl HitRequest {
    /// Every event of all nine grahas between two instants.
    #[must_use]
    pub fn between(from: JulianDay<Utc>, to: JulianDay<Utc>) -> HitRequest {
        HitRequest {
            from,
            to,
            grahas: GRAHAS.to_vec(),
            kinds: HitKind::ALL.to_vec(),
        }
    }

    /// The same request, for these grahas only.
    #[must_use]
    pub fn with_grahas(mut self, grahas: impl IntoIterator<Item = Graha>) -> HitRequest {
        self.grahas = grahas.into_iter().collect();
        self
    }

    /// The same request, for these kinds of event only.
    #[must_use]
    pub fn with_kinds(mut self, kinds: impl IntoIterator<Item = HitKind>) -> HitRequest {
        self.kinds = kinds.into_iter().collect();
        self
    }

    /// The window's start.
    #[must_use]
    pub const fn from(&self) -> JulianDay<Utc> {
        self.from
    }

    /// The window's end.
    #[must_use]
    pub const fn to(&self) -> JulianDay<Utc> {
        self.to
    }

    /// The grahas asked about.
    #[must_use]
    pub fn grahas(&self) -> &[Graha] {
        &self.grahas
    }

    /// Whether a kind of event was asked for.
    #[must_use]
    pub fn asks(&self, kind: HitKind) -> bool {
        self.kinds.contains(&kind)
    }

    /// The request checked before any search: a window that runs forward,
    /// and at least one graha and one kind; a refusal names the field.
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming `to`, `grahas` or `kinds`.
    pub fn check(&self) -> Result<(), Error> {
        if self.to.get() <= self.from.get() {
            return Err(Error::invalid_arg(format!(
                "a hit list's window must run forward, and {} is not after {}",
                self.to.get(),
                self.from.get()
            ))
            .with_field("to"));
        }
        if self.grahas.is_empty() {
            return Err(
                Error::invalid_arg("no graha to search the transits of").with_field("grahas")
            );
        }
        if self.kinds.is_empty() {
            return Err(Error::invalid_arg("no kind of event to report").with_field("kinds"));
        }
        Ok(())
    }
}
