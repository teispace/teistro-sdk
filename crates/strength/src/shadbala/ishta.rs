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

/// A graha's Ishta and Kashta phalas, virupas, 0 to 60.
pub(crate) fn of(strength: &GrahaShadbala, chart: &ShadbalaChart, rule: IshtaKashta) -> (f64, f64) {
    let uchcha = strength.sthana.uchcha.clamp(0.0, 60.0);
    let cheshta = cheshta(strength, chart, rule).clamp(0.0, 60.0);
    if rule == IshtaKashta::Rays {
        // BPHS ch. 28 vv. 2 to 6: each ray is 1 + arc/30, the Ishta half of
        // ten times both rays less one, which is the balas' mean.
        let rays = |bala: f64| 1.0 + bala * 3.0 / 30.0;
        let ishta = f64::midpoint((rays(uchcha) - 1.0) * 10.0, (rays(cheshta) - 1.0) * 10.0);
        return (ishta, 60.0 - ishta);
    }
    (
        (uchcha * cheshta).sqrt(),
        ((60.0 - uchcha) * (60.0 - cheshta)).sqrt(),
    )
}
