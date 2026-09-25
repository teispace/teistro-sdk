//! The Kaala bala: a graha's strength by the time of birth — the hour, the
//! fortnight, the third of the day or night, the lords of the year, month, day
//! and hour, and its declination.

use teistro_core::catalogue::Graha;
use teistro_core::math;
use teistro_core::settings::{KaalaLords, Kranti, Nathonnatha, PreDawnNight, SunAyana};

use super::{KaalaBala, ShadbalaChart, ShadbalaRules, elongation, is_benefic, seat, sign_of};

/// The recording engine's obliquity, which [`Kranti::Ecliptic`] reads.
pub const ENGINE_OBLIQUITY_DEG: f64 = 23.4393;

/// The Hindu astronomers' greatest declination, which [`Kranti::HinduTable`]
/// reads.
pub const HINDU_OBLIQUITY_DEG: f64 = 24.0;

/// The declination, arcminutes, at each 15° of the sayana bhuja from the
/// equinox (B.V. Raman, Art. 73): 362′, then 341′, 299′, 236′, 150′ and 52′
/// more.
const HINDU_KRANTI: [f64; 7] = [0.0, 362.0, 703.0, 1002.0, 1238.0, 1388.0, 1440.0];

/// Days elapsed from creation to 1 January 1860 (Burgess's 714,404,108,573
/// less the day itself), so that the count is a Sunday at nought as the
/// Surya Siddhanta's creation was; the corpus confirms its weekday on every
/// day the engine resolved correctly (crux C67).
const AHARGANA_1860: i64 = 714_404_108_572;
/// The Julian day number of 1 January 1860.
const JDN_1860: i64 = 2_400_411;

/// The lords of the year and month a chart's Kaala bala reads.
pub(crate) struct Lords {
    year: Option<Graha>,
    month: Graha,
}

impl Lords {
    /// The Abda and Masa lords under `rule`.
    pub(crate) fn of(chart: &ShadbalaChart, rule: KaalaLords) -> Lords {
        if rule == KaalaLords::Sankranti {
            let [sun, ..] = chart.grahas;
            return Lords {
                year: chart.sankranti_lord,
                month: sign_of(sun.longitude).attributes().lord,
            };
        }
        // The ahargana counted to and including the day of birth, as the
        // chapter and B.V. Raman (Arts. 60 and 61) divide it; its weekdays
        // count from Sunday.
        let days = AHARGANA_1860 + chart.civil_day - JDN_1860 + 1;
        Lords {
            year: Some(weekday_lord(days.div_euclid(360) * 3)),
            month: weekday_lord(days.div_euclid(30) * 2),
        }
    }
}

/// The weekday lord `n` days after a Sunday.
pub(crate) fn weekday_lord(n: i64) -> Graha {
    const FROM_SUNDAY: [Graha; 7] = [
        Graha::Sun,
        Graha::Moon,
        Graha::Mars,
        Graha::Mercury,
        Graha::Jupiter,
        Graha::Venus,
        Graha::Saturn,
    ];
    usize::try_from(n.rem_euclid(7))
        .ok()
        .and_then(|i| FROM_SUNDAY.get(i).copied())
        .unwrap_or(Graha::Sun)
}

/// A graha's Kaala bala, before any Yuddha bala.
pub(crate) fn of(
    graha: Graha,
    chart: &ShadbalaChart,
    rules: &ShadbalaRules,
    lords: &Lords,
) -> KaalaBala {
    let award = |lord: bool, virupas: f64| if lord { virupas } else { 0.0 };
    let own_ayana = ayana(graha, chart, rules.kranti);
    KaalaBala {
        nathonnatha: nathonnatha(graha, chart, rules),
        paksha: paksha(graha, chart, rules),
        tribhaga: tribhaga(graha, chart, rules),
        abda: award(lords.year == Some(graha), 15.0),
        masa: award(lords.month == graha, 30.0),
        vara: award(chart.weekday_lord == graha, 45.0),
        hora: award(chart.hora_lord == graha, 60.0),
        ayana: match (graha, rules.sun_ayana) {
            (Graha::Sun, SunAyana::Doubled) => 2.0 * own_ayana,
            (Graha::Sun, _) => 0.0,
            _ => own_ayana,
        },
        yuddha: 0.0,
    }
}

/// A triangle over an arc: nothing at its ends or outside, 60 at its middle.
fn peak(elapsed: f64, length: f64) -> f64 {
    let half = length / 2.0;
    ((1.0 - (elapsed - half).abs() / half) * 60.0).max(0.0)
}

pub(crate) fn is_daylight(chart: &ShadbalaChart) -> bool {
    chart.instant >= chart.sunrise && chart.instant < chart.sunset
}

/// Whether the engine's reading loses the night a pre-dawn birth falls in.
fn night_lost(chart: &ShadbalaChart, rules: &ShadbalaRules) -> bool {
    chart.after_midnight && rules.pre_dawn_night == PreDawnNight::SameEvening
}

/// The hour's strength (vv. 8 and 9).
pub(crate) fn nathonnatha(graha: Graha, chart: &ShadbalaChart, rules: &ShadbalaRules) -> f64 {
    let night_graha = matches!(graha, Graha::Moon | Graha::Mars | Graha::Saturn);
    if graha == Graha::Mercury {
        return 60.0;
    }
    let daylight = is_daylight(chart);
    if rules.nathonnatha == Nathonnatha::Arc {
        return match (night_graha, daylight) {
            (false, true) => peak(chart.instant - chart.sunrise, chart.sunset - chart.sunrise),
            (true, false) if !night_lost(chart, rules) => peak(
                chart.instant - chart.sunset,
                chart.next_sunrise - chart.sunset,
            ),
            _ => 0.0,
        };
    }
    // The unnata: the ghatis (60 a day) from midnight, 0 to 30.
    let unnata = if daylight {
        30.0 - (chart.instant - f64::midpoint(chart.sunrise, chart.sunset)).abs() * 60.0
    } else {
        (chart.instant - f64::midpoint(chart.sunset, chart.next_sunrise)).abs() * 60.0
    }
    .clamp(0.0, 30.0);
    let nata = 2.0 * (30.0 - unnata);
    if night_graha { nata } else { 60.0 - nata }
}

/// The fortnight's strength (vv. 10 and 11): a third of the Moon's elongation
/// for a benefic, 60 less that for a malefic, the Moon's doubled.
pub(crate) fn paksha(graha: Graha, chart: &ShadbalaChart, rules: &ShadbalaRules) -> f64 {
    let bright = elongation(chart) / 3.0;
    let value = if is_benefic(graha, chart, rules.benefics) {
        bright
    } else {
        60.0 - bright
    };
    if graha == Graha::Moon {
        2.0 * value
    } else {
        value
    }
}

/// 60 to the lord of the third of the day or night, and always to Jupiter
/// (v. 12).
fn tribhaga(graha: Graha, chart: &ShadbalaChart, rules: &ShadbalaRules) -> f64 {
    if graha == Graha::Jupiter {
        return 60.0;
    }
    let (start, end, lords) = if is_daylight(chart) {
        (
            chart.sunrise,
            chart.sunset,
            [Graha::Mercury, Graha::Sun, Graha::Saturn],
        )
    } else if night_lost(chart, rules) {
        return 0.0;
    } else {
        (
            chart.sunset,
            chart.next_sunrise,
            [Graha::Moon, Graha::Venus, Graha::Mars],
        )
    };
    let thirds = (chart.instant - start) / (end - start) * 3.0;
    let lord = if thirds < 1.0 {
        lords.first()
    } else if thirds < 2.0 {
        lords.get(1)
    } else {
        lords.last()
    };
    if lord == Some(&graha) { 60.0 } else { 0.0 }
}

/// A graha's declination, degrees, under `rule`, and the obliquity the Ayana
/// formula scales it by.
fn kranti(graha: Graha, chart: &ShadbalaChart, rule: Kranti) -> (f64, f64) {
    let at = seat(&chart.grahas, graha);
    if rule == Kranti::HinduTable {
        let sayana = at.tropical.rem_euclid(360.0);
        let within = sayana.rem_euclid(180.0);
        let bhuja = if within > 90.0 {
            180.0 - within
        } else {
            within
        };
        let step = bhuja / 15.0;
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a bhuja of 0 to 90 degrees is 0 to 6 steps"
        )]
        let whole = (step.floor() as usize).min(5);
        let before = HINDU_KRANTI.get(whole).copied().unwrap_or_default();
        let after = HINDU_KRANTI.get(whole + 1).copied().unwrap_or(before);
        #[allow(clippy::cast_precision_loss, reason = "0 to 5")]
        let fraction = step - whole as f64;
        let minutes = before + (after - before) * fraction;
        let sign = if sayana < 180.0 { 1.0 } else { -1.0 };
        (sign * minutes / 60.0, HINDU_OBLIQUITY_DEG)
    } else {
        let (obliquity, latitude) = if rule == Kranti::True {
            (chart.obliquity, at.latitude)
        } else {
            (ENGINE_OBLIQUITY_DEG, 0.0)
        };
        let (e, b, l) = (
            obliquity.to_radians(),
            latitude.to_radians(),
            at.tropical.to_radians(),
        );
        let declination =
            math::asin(math::sin(b) * math::cos(e) + math::cos(b) * math::sin(e) * math::sin(l))
                .to_degrees();
        (declination, obliquity)
    }
}

/// A graha's Ayana bala, before the Sun's is doubled (vv. 15 to 17).
pub(crate) fn ayana(graha: Graha, chart: &ShadbalaChart, rule: Kranti) -> f64 {
    let (declination, obliquity) = kranti(graha, chart, rule);
    let signed = match graha {
        Graha::Mercury => declination.abs(),
        Graha::Sun | Graha::Mars | Graha::Jupiter | Graha::Venus => declination,
        _ => -declination,
    };
    ((obliquity + signed) / (2.0 * obliquity) * 60.0).clamp(0.0, 60.0)
}
