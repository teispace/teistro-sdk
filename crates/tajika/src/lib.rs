//! Tajika: the annual chart and what is read from it
//! (`03-design/annual-chart.md`).
//!
//! What is built is the one step everything else needs. The **Varsha
//! Pravesha** is the instant the Sun returns to the longitude it held at
//! birth, and every Tajika judgement is made from the chart cast for it,
//! so an error here is an error in all of them.
//!
//! The rule, its two rivals and what each costs the lagna are measured
//! over every recorded birth in `03-design/annual-chart-measured.md`.
//!
//! ```
//! use teistro_tajika::{Natal, Reading};
//! use teistro_core::quantity::{JulianDay, Utc};
//!
//! let natal = Natal {
//!     instant: JulianDay::<Utc>::literal(2_447_995.489_583_333_5),
//!     sidereal_sun_deg: 0.054_692,
//!     tropical_sun_deg: 23.779_197,
//! };
//! // The mean reading needs no ephemeris at all: it is arithmetic.
//! let mean = teistro_tajika::mean_praveshas(&natal, 3)?;
//! assert_eq!(mean.len(), 3);
//! assert_eq!(mean[0].year, 1);
//! assert!(mean[0].at.get() > natal.instant.get());
//! assert_eq!(Reading::default(), Reading::Sidereal);
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

#![doc(html_no_source)]

mod bala;
mod drishti;
mod muntha;
mod office;
mod varsha;
mod varshesha;
mod yoga;

pub use bala::{
    AnnualSky, Bala, Panchavargiya, Relation, SEVEN, drekkana_lord, hudda_lord, navamsha_lord,
    panchavargiya,
};
pub use drishti::{
    BY_SPEED, Between, Drishti, DrishtiRules, POORNA_DEG, RASHYANTA_DEG, SubDegree, Yoga,
    all as drishtis, all_with_rules as drishtis_with_rules, between, between_with_rules,
    deeptamsha, orb_between, speed_rank,
};
pub use muntha::{DAILY_DEG, MONTHLY_DEG, Muntha, MunthaDegree, muntha};
pub use office::{Office, OfficeBearers, YearCharts, office_bearers, tri_rashi_lord};
pub use varsha::{
    MOST_YEARS, Natal, Pravesha, Reading, SIDEREAL_YEAR_DAYS, STEP_DAYS, mean_praveshas, praveshas,
    years,
};
pub use varshesha::{
    Chosen, Claim, NoneAspects, Varshesha, VarsheshaRules, WEAK_BELOW, aspects, varshesha,
};
pub use yoga::{
    Held, MALEFICS, Qualification, YearYoga, YearYogas, qualification, year_yogas,
    year_yogas_with_rules,
};
