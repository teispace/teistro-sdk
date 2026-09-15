//! The Ishta and Kashta phalas: how far a graha tends to good and to harm,
//! from its Uchcha and Cheshta balas (crux C76).

use teistro_core::catalogue::Graha;
use teistro_core::settings::IshtaKashta;

use super::{GrahaShadbala, ShadbalaChart, apart};

/// The Cheshta bala the phalas read: the Shadbala's own for Mars to Saturn,
/// and for the luminaries a third of the Sun's sayana longitude less three
/// signs and of the Moon's elongation (BPHS ch. 28 vv. 3 and 4; B.V. Raman,
/// Arts. 136–137).
fn cheshta(strength: &GrahaShadbala, chart: &ShadbalaChart, rule: IshtaKashta) -> f64 {
    if rule == IshtaKashta::ShadbalaCheshta {
        return strength.cheshta;
    }
    let [sun, moon, ..] = chart.grahas;
    match strength.graha {
        Graha::Sun => apart(sun.tropical + 90.0, 0.0) / 3.0,
        Graha::Moon => apart(moon.longitude, sun.longitude) / 3.0,
        _ => strength.cheshta,
    }
}

/// What BPHS ch. 28 vv. 2 to 6 read from a graha's Uchcha and Cheshta balas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Phalas {
    /// The Ishta phala, virupas, 0 to 60.
    pub ishta: f64,
    /// The Kashta phala, virupas, 0 to 60.
    pub kashta: f64,
    /// The auspicious rays, 1 to 7: the Uchcha and Cheshta rays' mean (v. 5).
    pub subha_rashmi: f64,
    /// The inauspicious rays, 8 less the auspicious (v. 5).
    pub ashubha_rashmi: f64,
}

/// A bala's rays: 1 + its arc over 30°, the arc three times the bala
/// (vv. 2 to 4).
fn rays(bala: f64) -> f64 {
    1.0 + bala * 3.0 / 30.0
}

/// A graha's phalas under `rule`; the rays are the verses' whichever reading
/// takes the phalas.
pub(crate) fn of(strength: &GrahaShadbala, chart: &ShadbalaChart, rule: IshtaKashta) -> Phalas {
    let uchcha = strength.sthana.uchcha.clamp(0.0, 60.0);
    let cheshta = cheshta(strength, chart, rule).clamp(0.0, 60.0);
    let subha_rashmi = f64::midpoint(rays(uchcha), rays(cheshta));
    let (ishta, kashta) = if rule == IshtaKashta::Rays {
        // V. 6: half of ten times both rays less one, which is the balas' mean.
        let ishta = f64::midpoint((rays(uchcha) - 1.0) * 10.0, (rays(cheshta) - 1.0) * 10.0);
        (ishta, 60.0 - ishta)
    } else {
        (
            (uchcha * cheshta).sqrt(),
            ((60.0 - uchcha) * (60.0 - cheshta)).sqrt(),
        )
    };
    Phalas {
        ishta,
        kashta,
        subha_rashmi,
        ashubha_rashmi: 8.0 - subha_rashmi,
    }
}
