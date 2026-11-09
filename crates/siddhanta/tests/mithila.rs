//! The Vishwavidyalaya Panchang's printed days
//! (`tests/data/vishwavidyalaya-panchang.txt`) against the text under the
//! Makarandasarini's bija, its day counted from the text's sunrise carried
//! by the bhujantara, at the place the almanac states: Mithila's almanacs
//! as `mithila-makaranda` computes them (`docs/03-design/makaranda.md`).
//!
//! The bounds are the measured ones, not a claim of agreement to the
//! minute: the residual follows the Moon's anomaly (C1225) and, in the
//! tithi only, the Sun's (C1226), and neither is closed.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "tests fail by panicking and read a small data file"
)]

use std::path::Path;

use teistro_calendar::gregorian::fixed_from_gregorian;
use teistro_core::catalogue::Graha;
use teistro_core::quantity::{JulianDay, Latitude, Longitude, Ut1};
use teistro_port_ephemeris::EphemerisProvider;
use teistro_siddhanta::{Bija, Parameters, SiddhantaProvider, SuryaSiddhanta, Trig};

/// The place the almanac states every Mithila panchang is computed for
/// (front matter, 2022-23): 85°30′ E, 26°35′ N, palabha 6 angula.
const LONGITUDE_DEG: f64 = 85.5;
const LATITUDE_DEG: f64 = 26.0 + 35.0 / 60.0;

/// One printed day.
struct Row {
    date: String,
    tithi: u32,
    tithi_end: Option<f64>,
    nakshatra: u32,
    nakshatra_end: Option<f64>,
    day_length: f64,
}

/// `ghati|pala` to ghatis; `?|?` is an end the page does not settle.
fn ghatis(text: &str) -> Option<f64> {
    if text.contains('?') {
        return None;
    }
    let (g, p) = text.split_once('|').expect("ghati|pala");
    Some(g.parse::<f64>().expect("ghatis") + p.parse::<f64>().expect("palas") / 60.0)
}

/// A data file's rows, split on whitespace, its comments and blank lines
/// left out.
fn data(name: &str) -> Vec<Vec<String>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data")
        .join(name);
    std::fs::read_to_string(path)
        .expect("a data file")
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .map(|line| line.split_whitespace().map(String::from).collect())
        .collect()
}

fn rows() -> Vec<Row> {
    data("vishwavidyalaya-panchang.txt")
        .into_iter()
        .map(|f| {
            assert_eq!(f.len(), 6, "{f:?}");
            Row {
                date: f[0].clone(),
                tithi: f[1].parse().expect("a tithi"),
                tithi_end: ghatis(&f[2]),
                nakshatra: f[3].parse().expect("a nakshatra"),
                nakshatra_end: ghatis(&f[4]),
                day_length: ghatis(&f[5]).expect("a day length"),
            }
        })
        .collect()
}

/// The model `mithila-makaranda` opens, or the text without a bija.
fn model(bija: &Bija) -> SuryaSiddhanta {
    SuryaSiddhanta::new(Parameters::TEXT.with_bija(bija), Trig::Table)
}

/// The date's sunrise carried by the bhujantara, and its day length in
/// ghatis.
fn sunrise(model: &SuryaSiddhanta, date: &str) -> (f64, f64) {
    let mut parts = date
        .split('-')
        .map(|p| p.parse::<i64>().expect("a date part"));
    let (y, m, d) = (
        parts.next().unwrap() as i32,
        parts.next().unwrap() as u8,
        parts.next().unwrap() as u8,
    );
    let midnight_ut = fixed_from_gregorian(y, m, d)
        .jd_at_midnight()
        .expect("a Julian day")
        .get();
    // Local noon of the civil date, then the local mean midnight before it.
    let noon = JulianDay::<Ut1>::literal(midnight_ut + 0.5 - LONGITUDE_DEG / 360.0);
    let midnight = SuryaSiddhanta::local_mean_midnight(noon, Longitude::literal(LONGITUDE_DEG));
    let arc = model
        .day_arc(midnight, Latitude::literal(LATITUDE_DEG))
        .expect("the Sun rises in Mithila");
    let rise = arc.sunrise.get() + model.bhujantara_days(arc.sunrise.get());
    (rise, (arc.sunset.get() - arc.sunrise.get()) * 60.0)
}

fn moon(model: &SuryaSiddhanta, jd: f64) -> f64 {
    model.moon(JulianDay::<Ut1>::literal(jd)).longitude.get()
}

fn elongation(model: &SuryaSiddhanta, jd: f64) -> f64 {
    (moon(model, jd) - model.sun_longitude_deg(jd)).rem_euclid(360.0)
}

/// The instant near `guess` at which `angle` reaches `target`, by secant
/// steps.
fn when(angle: impl Fn(f64) -> f64, target: f64, guess: f64) -> f64 {
    let off = |jd: f64| (angle(jd) - target + 540.0).rem_euclid(360.0) - 180.0;
    let mut t = guess;
    for _ in 0..20 {
        let rate = (off(t + 0.01) - off(t)) / 0.01;
        let step = off(t) / rate;
        t -= step;
        if step.abs() < 1e-7 {
            break;
        }
    }
    t
}

/// Each printed end less the model's, minutes (positive: the model later),
/// and how many days' sunrise tithi and nakshatra agree with the rows'.
struct Measured {
    tithi: Vec<f64>,
    nakshatra: Vec<f64>,
    tithi_days: usize,
    nakshatra_days: usize,
}

fn measure(bija: &Bija) -> Measured {
    let model = model(bija);
    let mut out = Measured {
        tithi: Vec::new(),
        nakshatra: Vec::new(),
        tithi_days: 0,
        nakshatra_days: 0,
    };
    for row in rows() {
        let (rise, _) = sunrise(&model, &row.date);
        if let Some(end) = row.tithi_end.filter(|g| *g < 60.0) {
            let target = f64::from(row.tithi % 30) * 12.0;
            let ours = when(|jd| elongation(&model, jd), target, rise + end / 60.0);
            out.tithi.push((ours - rise) * 1440.0 - end * 24.0);
        }
        if let Some(end) = row.nakshatra_end.filter(|g| *g < 60.0) {
            let target = f64::from(row.nakshatra % 27) * 360.0 / 27.0;
            let ours = when(|jd| moon(&model, jd), target, rise + end / 60.0);
            out.nakshatra.push((ours - rise) * 1440.0 - end * 24.0);
        }
        let tithi = (elongation(&model, rise) / 12.0) as u32 + 1;
        let nakshatra = (moon(&model, rise).rem_euclid(360.0) * 27.0 / 360.0) as u32 + 1;
        out.tithi_days += usize::from(tithi == row.tithi);
        out.nakshatra_days += usize::from(nakshatra == row.nakshatra);
    }
    out
}

fn within(minutes: &[f64], bound: f64) -> usize {
    minutes.iter().filter(|m| m.abs() <= bound).count()
}

#[test]
fn the_data_is_eight_years_of_printed_days() {
    assert_eq!(rows().len(), 162);
    // One block a year, each headed by the almanac's year (`1998-99, ...`).
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/vishwavidyalaya-panchang.txt");
    let text = std::fs::read_to_string(path).expect("the printed days");
    let years: Vec<&str> = text
        .lines()
        .filter_map(|line| line.strip_prefix("# "))
        .filter(|rest| {
            rest.as_bytes().get(4) == Some(&b'-') && rest[..4].bytes().all(|b| b.is_ascii_digit())
        })
        .map(|rest| &rest[..7])
        .collect();
    assert_eq!(
        years,
        [
            "1980-81", "1992-93", "1995-96", "1998-99", "2005-06", "2012-13", "2015-16", "2022-23"
        ]
    );
}

#[test]
fn the_texts_day_at_palabha_six_is_the_printed_day_length() {
    let model = model(&Bija::MAKARANDA);
    for row in rows() {
        let (_, length) = sunrise(&model, &row.date);
        let palas = (length - row.day_length) * 60.0;
        // Measured within 12.1 palas (under five minutes) either way.
        assert!(palas.abs() <= 15.0, "{}: {palas:.1} palas", row.date);
    }
}

#[test]
fn mithilas_printed_ends_are_the_makarandas_within_measured_bounds() {
    let m = measure(&Bija::MAKARANDA);
    let (t, n) = (&m.tithi, &m.nakshatra);
    assert_eq!((t.len(), n.len()), (156, 150));
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    // Measured (exact trigonometry): tithi +0.1 ± 6.4 minutes, 139 of 156
    // within ten, the worst 23.1; nakshatra −0.7 ± 4.1, 149 of 150 within
    // ten, the worst 20.8 (the 1998 slip the data file names).
    assert!(
        mean(t).abs() < 2.0 && mean(n).abs() < 2.0,
        "{} {}",
        mean(t),
        mean(n)
    );
    assert!(
        within(t, 10.0) >= 135,
        "tithi within ten: {}",
        within(t, 10.0)
    );
    assert!(
        within(n, 10.0) >= 147,
        "nakshatra within ten: {}",
        within(n, 10.0)
    );
    assert_eq!(within(t, 25.0), t.len(), "{t:?}");
    assert_eq!(within(n, 22.0), n.len(), "{n:?}");
    // The day each row names: one sunrise tithi (an end printed minutes
    // after sunrise) falls the other side.
    assert!(m.tithi_days >= 161, "{}", m.tithi_days);
    assert_eq!(m.nakshatra_days, 162);
}

#[test]
fn without_the_bija_the_text_misses_by_a_quarter_of_an_hour() {
    let m = measure(&Bija::NONE);
    // Measured: 69 of 156 tithi ends and 61 of 150 nakshatra ends within
    // ten minutes, each spread 13.6 minutes.
    assert!(within(&m.tithi, 10.0) < 90, "{}", within(&m.tithi, 10.0));
    assert!(
        within(&m.nakshatra, 10.0) < 90,
        "{}",
        within(&m.nakshatra, 10.0)
    );
}

#[test]
fn the_provider_stamps_the_makaranda_and_its_sunrise() {
    let provider = SiddhantaProvider::with_bija(Bija::MAKARANDA).with_bhujantara();
    let stamp = provider.capabilities().identity.data_version;
    for part in [
        "moon_apsis -4",
        "moon_node +4",
        "jupiter -8",
        "bhujantara sunrise",
    ] {
        assert!(stamp.contains(part), "{stamp}");
    }
}

/// The print's clock (C1229 a): local apparent time, its sunrise 12 hours
/// less half the printed day length, as the front matter (2022-23, PDF p.
/// 45) says and every sunrise cell read agrees. The R-shaped glyph is a 1,
/// so 14 July 2022's "५।R४" is 05:14, not 05:24.
#[test]
fn the_printed_clock_is_local_apparent_time() {
    // Minutes after local apparent midnight of a sunrise whose daylight is
    // `ghatis` long.
    let sunrise = |ghatis: f64| 720.0 - ghatis * 12.0;
    // 14 July 2022: day length 33|48, sunrise printed 05:14 (PDF p. 11).
    let july = sunrise(33.0 + 48.0 / 60.0);
    assert_eq!(july.floor(), 5.0 * 60.0 + 14.0);
    // Its tithi ends at 41|21, printed "रा. ०९।४६": 21:46.
    assert_eq!(
        (july + (41.0 + 21.0 / 60.0) * 24.0).floor(),
        21.0 * 60.0 + 46.0
    );
    // 31 October 2022: day length 27|39, sunrise printed 06:28. The front
    // matter's worked example takes it to standard time: less the
    // velantara (the equation of time, 16 minutes that day) and less the
    // standard meridian's deshantara, (85.5° − 82.5°) at four minutes a
    // degree, 06:00 IST.
    let october = sunrise(27.0 + 39.0 / 60.0);
    assert_eq!(october.floor(), 6.0 * 60.0 + 28.0);
    let deshantara = (LONGITUDE_DEG - 82.5) * 4.0;
    assert_eq!((october - 16.0 - deshantara).floor(), 6.0 * 60.0);
}

/// The ingresses the 2022-23 print gives in ghatis after its sunrise:
/// civil date, the sidereal longitude the Sun reaches, and the printed
/// ghati|pala.
const INGRESSES: [(&str, f64, f64); 4] = [
    // "कर्के रविः १४।४० मु." (PDF p. 11).
    ("2022-07-17", 90.0, 14.0 + 40.0 / 60.0),
    // Kanya (PDF p. 15).
    ("2022-09-17", 150.0, 42.0 + 35.0 / 60.0),
    // The Sun into Vishakha, 15 × 13°20′ (PDF p. 18).
    ("2022-11-07", 200.0, 6.0 + 29.0 / 60.0),
    // Mesha (PDF p. 29, read in the OCR).
    ("2023-04-14", 0.0, 29.0 + 20.0 / 60.0),
];

#[test]
fn the_printed_ingresses_are_the_makarandas_sun() {
    let model = model(&Bija::MAKARANDA);
    for (date, target, printed) in INGRESSES {
        let (rise, _) = sunrise(&model, date);
        let ours = when(
            |jd| model.sun_longitude_deg(jd),
            target,
            rise + printed / 60.0,
        );
        let after = (ours - rise) * 60.0;
        let minutes = (after - printed) * 24.0;
        // The ingress falls on the civil day the print gives it.
        assert!((0.0..60.0).contains(&after), "{date}: {after}");
        // Measured, model less print: Karka +6.2, Kanya +14.0, Vishakha
        // +11.9 and Mesha -10.2 minutes, about half an arcminute of the
        // Sun at most. Not fitted (C1229 b).
        assert!(minutes.abs() <= 15.0, "{date}: {minutes:+.2} minutes");
    }
}

const STARS: [Graha; 5] = [
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];

/// `sign|degree|minute|second` to degrees.
fn longitude(text: &str) -> f64 {
    let parts: Vec<f64> = text
        .split('|')
        .map(|p| p.parse::<f64>().expect("a figure"))
        .collect();
    parts[0] * 30.0 + parts[1] + parts[2] / 60.0 + parts[3] / 3600.0
}

/// One printed day's star planets: the date, whether it is a Thursday,
/// and Mars, Mercury, Jupiter, Venus and Saturn in degrees.
struct Planets {
    date: String,
    thursday: bool,
    longitudes: Vec<f64>,
}

fn planet_rows() -> Vec<Planets> {
    data("vishwavidyalaya-planets.txt")
        .into_iter()
        .map(|f| {
            assert_eq!(f.len(), 7, "{f:?}");
            Planets {
                date: f[0].clone(),
                thursday: f[1] == "T",
                longitudes: f[2..].iter().map(|text| longitude(text)).collect(),
            }
        })
        .collect()
}

/// Each printed star planet less the model's, arcminutes, positive when
/// the model is ahead, at *miśramāna*: 30 ghatis and half the day after
/// the model's sunrise, the local apparent midnight that ends the civil
/// date (C1229 a; the printed 46|54 of 14 July 2022 is that sum).
fn planet_residuals(model: &SuryaSiddhanta) -> Vec<(String, bool, Graha, f64)> {
    let mut out = Vec::new();
    for row in planet_rows() {
        let (rise, length) = sunrise(model, &row.date);
        let at = JulianDay::<Ut1>::literal(rise + (30.0 + length / 2.0) / 60.0);
        for (graha, printed) in STARS.into_iter().zip(&row.longitudes) {
            let ours = model
                .graha(graha, at)
                .expect("a star planet")
                .longitude
                .get();
            let minutes = ((ours - printed + 540.0).rem_euclid(360.0) - 180.0) * 60.0;
            out.push((row.date.clone(), row.thursday, graha, minutes));
        }
    }
    out
}

/// C1227: the print's star planets are the text's four steps (II.43 to
/// 44) under the Makaranda's bija, the very procedure the Makaranda's
/// three steps tabulate (`equation::four_step`). The print computes them
/// on Thursdays and fills the days between (C1280).
#[test]
fn the_printed_star_planets_are_the_texts_four_steps() {
    let residuals = planet_residuals(&model(&Bija::MAKARANDA));
    assert_eq!(residuals.len(), 29 * 5);
    let thursdays: Vec<_> = residuals.iter().filter(|r| r.1).collect();
    assert_eq!(thursdays.len(), 25);
    for (date, _, graha, minutes) in &thursdays {
        // Measured: within 1.35′ on all 25 (rms 0.7′); 14 July 2022 Mars
        // +1.08′, Mercury +0.72′, Jupiter −0.30′, Venus +0.03′, Saturn
        // −0.44′.
        assert!(minutes.abs() <= 1.5, "{date} {graha:?}: {minutes:+.2}′");
    }
    // Without the bija Mercury, Jupiter, Venus and Saturn miss by degrees.
    for (date, thursday, graha, minutes) in planet_residuals(&model(&Bija::NONE)) {
        if thursday && graha != Graha::Mars {
            assert!(minutes.abs() > 60.0, "{date} {graha:?}: {minutes:+.2}′");
        }
    }
}

/// The days between Thursdays step by a constant daily motion (Mars,
/// Mercury, Venus and Saturn) or by hand near a station (Mars, Jupiter),
/// so they miss the model by more than the Thursdays do (C1280).
#[test]
fn between_thursdays_the_print_fills_in() {
    // Measured: every other filled day within 4.3′ (Mars on 2 November
    // 2022, three days after its station).
    let slips: Vec<_> = planet_residuals(&model(&Bija::MAKARANDA))
        .into_iter()
        .filter(|(_, thursday, _, minutes)| !thursday && minutes.abs() > 5.0)
        .collect();
    // Mercury's lost carry, 25 to 27 July 2022: a degree, give or take
    // the filling's own minutes (C1281).
    assert_eq!(slips.len(), 3, "{slips:?}");
    for (date, _, graha, minutes) in slips {
        assert_eq!(graha, Graha::Mercury, "{date}");
        assert!((minutes - 60.0).abs() <= 5.0, "{date}: {minutes:+.2}′");
    }
}
