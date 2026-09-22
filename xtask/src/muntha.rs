//! The falsification pass over the **Muntha**: the birth lagna progressed
//! one sign for each completed year.
//!
//! The corpus records no Muntha, and no annual chart of any kind. But it
//! records the Muntha's **only input**: every recorded chart carries the
//! recording engine's lagna. Every Muntha a birth will ever have is that
//! sign rotated by a count of years, so holding the SDK's lagna against
//! the recorded one falsifies all of them at once — and a lagna founded on
//! the wrong side of a sign boundary moves the Muntha for the whole of a
//! life, which is why the margin to the boundary is measured beside it.
//!
//! What the corpus cannot settle is the arithmetic's own off-by-one: the
//! source states the rule in years **completed** and numbers its worked
//! chart by the year of life it **opens**. That is measured as a rival
//! over the recorded births, because a rule with no recording to check it
//! is still a rule two readings can disagree about.
//!
//! `cargo xtask muntha` writes the page and `check-muntha` regenerates it
//! in memory and fails on any difference.

use std::fmt::Write as _;
use std::path::Path;

use teistro::catalogue::{Graha, Rashi};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::tajika::{MOST_YEARS, MunthaDegree, Reading};
use teistro::{ChartRequest, Context, Ephemeris, UtcOffset};

use crate::births::{Birth, CHARTS, births};
use crate::generated::{Output, check, write};
use crate::measure::{Claim, count, fill, listed, plural, table};

const PAGE: &str = "docs/03-design/muntha-measured.md";

/// How many years each recorded birth is followed for.
///
/// Ten turns of the zodiac, so every birth's Muntha stands in every sign
/// ten times and the rival's one blind spot is met as often as it can be.
const YEARS: u16 = 120;

/// The source's worked chart: Leo rising, forty years complete, and the
/// Muntha in Sagittarius (K.S. Charak, *A Textbook of Varshaphala*, ch. V).
const WORKED_LAGNA_DEG: f64 = 4.0 * 30.0 + 4.5;
const WORKED_YEARS: u16 = 40;
const WORKED_MUNTHA: Rashi = Rashi::Sagittarius;

/// One birth's lagna, founded and recorded, and how near a sign boundary
/// the recording puts it.
struct Lagna {
    name: String,
    /// The SDK's lagna minus the recording's, arcseconds.
    apart_arcsec: f64,
    /// How far the recorded lagna is from the nearer of its sign's two
    /// boundaries, degrees: the error the Muntha tolerates before it moves
    /// for a lifetime.
    margin_deg: f64,
    /// Whether the SDK's lagna and the recording's fall in one sign.
    same_sign: bool,
}

fn lagna_of(birth: &Birth) -> Lagna {
    let founded = birth.document.foundation.lagna_deg.rem_euclid(360.0);
    let recorded = birth.recorded_lagna_deg.rem_euclid(360.0);
    let mut apart = founded - recorded;
    if apart > 180.0 {
        apart -= 360.0;
    } else if apart < -180.0 {
        apart += 360.0;
    }
    let inside = recorded % 30.0;
    Lagna {
        name: birth.name.clone(),
        apart_arcsec: apart * 3600.0,
        margin_deg: inside.min(30.0 - inside),
        same_sign: sign_of(founded) == birth.recorded_lagna_sign,
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a longitude folded into 0..360 divides by thirty into 0..12"
)]
fn sign_of(longitude_deg: f64) -> u16 {
    (longitude_deg.rem_euclid(360.0) / 30.0) as u16 % 12
}

/// What the rival reading costs over every recorded birth and year.
struct Rival {
    cases: usize,
    sign_agrees: usize,
    lord_agrees: usize,
    /// The sign pairs, as `from→to`, on which the two agree about the lord.
    blind_spots: Vec<String>,
}

fn rival(sdk: &Context, births: &[Birth]) -> Result<Rival, String> {
    let mut found = Rival {
        cases: 0,
        sign_agrees: 0,
        lord_agrees: 0,
        blind_spots: Vec::new(),
    };
    for birth in births {
        for year in 1..=YEARS {
            let right = muntha(sdk, birth, year, MunthaDegree::SignStart)?;
            let opened = muntha(sdk, birth, year + 1, MunthaDegree::SignStart)?;
            found.cases += 1;
            found.sign_agrees += usize::from(right.sign == opened.sign);
            if right.lord == opened.lord {
                found.lord_agrees += 1;
                let pair = format!("{:?}→{:?}", right.sign, opened.sign);
                if !found.blind_spots.contains(&pair) {
                    found.blind_spots.push(pair);
                }
            }
        }
    }
    Ok(found)
}

/// The Muntha through the **façade**, which is what a consumer calls.
fn muntha(
    sdk: &Context,
    birth: &Birth,
    years: u16,
    degree: MunthaDegree,
) -> Result<teistro::Muntha, String> {
    sdk.chart()
        .muntha(&birth.document, years, degree)
        .map_err(|why| format!("{}: year {years}: {why}", birth.name))
}

/// Every year of every birth, held against the recorded lagna rotated.
fn against_the_recording(sdk: &Context, births: &[Birth]) -> Result<(usize, usize), String> {
    let (mut wrong, mut of) = (0, 0);
    for birth in births {
        for year in 0..=YEARS {
            let found = muntha(sdk, birth, year, MunthaDegree::SignStart)?;
            let want = Rashi::ALL[usize::from((birth.recorded_lagna_sign + year % 12) % 12)];
            of += 1;
            wrong += usize::from(found.sign != want || found.lord != want.attributes().lord);
        }
    }
    Ok((wrong, of))
}

fn page(root: &Path) -> Result<String, String> {
    let sdk = Context::builder()
        .profile("conformance-baseline")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .map_err(|why| format!("the conformance profile: {why}"))?;
    let births = births(root, &sdk)?;
    if births.is_empty() {
        return Err(format!("{CHARTS} records no chart"));
    }
    let mut out = String::from(
        "# The Muntha, measured\n\n\
         Status: `generated` by `cargo xtask muntha` over the conformance \
         corpus's recorded births. Do not edit: `check-muntha` regenerates this \
         page and fails on any difference. The design it measures is \
         [`muntha.md`](muntha.md).\n\n\
         The **Muntha** is the birth lagna's sign advanced one sign for each \
         completed year of life. It is the first of the annual chart's five \
         office-bearers and the one that takes the year's lordship when none of \
         the others qualifies.\n\n\
         **The corpus records no Muntha, and no annual chart of any kind.** It \
         records the Muntha's only input: the recording engine's lagna, on every \
         chart. Every Muntha a birth will ever have is that sign rotated by a \
         count of years, so the recorded lagna falsifies all of them at once.\n\n",
    );
    what_holds(&mut out, &sdk, &births)?;
    the_margin(&mut out, &births);
    the_rival(&mut out, &sdk, &births)?;
    the_degree(&mut out, &sdk, &births)?;
    the_worked_year(&mut out)?;
    Ok(fill(&out))
}

fn what_holds(out: &mut String, sdk: &Context, births: &[Birth]) -> Result<(), String> {
    let worked = teistro::tajika::muntha(WORKED_LAGNA_DEG, WORKED_YEARS, MunthaDegree::SignStart)
        .map_err(|why| format!("the worked chart: {why}"))?;
    let (wrong, of) = against_the_recording(sdk, births)?;
    let claims = [
        Claim::counted(
            format!(
                "the source's worked chart: Leo rising, {WORKED_YEARS} years complete, \
                 the Muntha in {WORKED_MUNTHA:?}"
            ),
            usize::from(worked.sign != WORKED_MUNTHA),
            1,
        ),
        Claim::counted(
            format!(
                "every year 0 to {YEARS} of every recorded birth: the recorded lagna's \
                 sign advanced by the years complete, and that sign's lord"
            ),
            wrong,
            of,
        ),
    ];
    let _ = write!(
        out,
        "## 1. What holds\n\n{}\n\
         The first row is the only value in the source that checks the rule end \
         to end. The second is {} Munthas across {}, each asked of the façade \
         (`sdk.chart().muntha`) and held against the **recording's** lagna rather \
         than the SDK's, which is what makes it a check and not the rule \
         agreeing with itself.\n\n",
        table(&claims),
        count(of),
        plural(births.len(), "recorded birth"),
    );
    Ok(())
}

impl Lagna {
    /// How much of its own margin this birth's founding error spends: the
    /// one comparison that asks whether *this* birth's Muntha came near
    /// moving. The worst error anywhere against the tightest margin
    /// anywhere is two different births, and says nothing about either.
    fn spent(&self) -> f64 {
        let margin = self.margin_deg * 3600.0;
        if margin > 0.0 {
            self.apart_arcsec.abs() / margin
        } else {
            f64::INFINITY
        }
    }
}

fn the_margin(out: &mut String, births: &[Birth]) {
    let lagnas: Vec<Lagna> = births.iter().map(lagna_of).collect();
    let apart = lagnas
        .iter()
        .map(|one| one.apart_arcsec.abs())
        .fold(0.0_f64, f64::max);
    let flipped: Vec<String> = lagnas
        .iter()
        .filter(|one| !one.same_sign)
        .map(|one| format!("`{}`", one.name))
        .collect();
    let mut tightest: Vec<&Lagna> = lagnas.iter().collect();
    tightest.sort_by(|a, b| b.spent().total_cmp(&a.spent()));
    let _ = write!(
        out,
        "## 2. How wrong the lagna may be\n\n\
         A Muntha is only as right as the lagna under it, and the lagna fails it \
         in one way only: by falling on the other side of a sign boundary, which \
         moves the Muntha for the **whole of a life** rather than for a year. So \
         the unit that matters is not the lagna's error but the error against \
         the distance to the nearest boundary.\n\n\
         The SDK's founded lagna is at most **{apart:.1} arcseconds** from the \
         recording's over {}, and it falls in a different sign from the \
         recording's on {}.\n\n\
         | birth | recorded lagna to the nearest boundary | founded minus recorded | of its margin |\n\
         |---|---|---|---|\n",
        plural(births.len(), "birth"),
        if flipped.is_empty() {
            String::from("none of them")
        } else {
            listed(&flipped)
        },
    );
    for one in tightest.iter().take(5) {
        let _ = writeln!(
            out,
            "| `{}` | {:.4}° | {:+.1}″ | {:.2}% |",
            one.name,
            one.margin_deg,
            one.apart_arcsec,
            100.0 * one.spent()
        );
    }
    let nearest = lagnas
        .iter()
        .map(|one| one.margin_deg)
        .fold(f64::INFINITY, f64::min);
    let worst = tightest.first().map_or(0.0, |one| one.spent());
    let _ = write!(
        out,
        "\nThe five that spent most of their own margin, most first — each birth's \
         founding error against **that birth's** distance to a boundary. The \
         worst spends **{:.2}%** of it. The closest any recorded lagna comes to a \
         boundary is {nearest:.4}° ({:.0} arcseconds), and that birth's founding \
         error is smaller still.\n\n\
         Comparing the worst error anywhere with the tightest margin anywhere \
         would read as {:.0}% of the way to moving a Muntha, and it would be two \
         different births: the error sits on one and the margin on another.\n\n",
        100.0 * worst,
        nearest * 3600.0,
        if nearest > 0.0 {
            100.0 * apart / (nearest * 3600.0)
        } else {
            f64::INFINITY
        },
    );
}

fn the_rival(out: &mut String, sdk: &Context, births: &[Birth]) -> Result<(), String> {
    let found = rival(sdk, births)?;
    let _ = write!(
        out,
        "## 3. The rival: the year of life it opens\n\n\
         The source states the rule in years **completed** — \"add to the lagna \
         sign the number of completed years of life\" — and numbers its own \
         worked chart by the year of life it **opens**: the chart with forty \
         years complete is the one it calls the forty-first year's. A reader \
         who takes the chart's number for the count progresses the Muntha one \
         sign too far.\n\n\
         | over every recorded birth, years 1 to {YEARS} | cases |\n\
         |---|---|\n\
         | the two readings agree on the Muntha's **sign** | {} of {} |\n\
         | they agree on the sign's **lord** | {} of {} |\n\n\
         They never agree on the sign, and they agree on the lord only across \
         {}: the one step in the zodiac where a single planet rules both sides, \
         Saturn holding Capricorn and Aquarius. So the rival is wrong everywhere \
         and **invisible once in twelve** — often enough that a spot check can \
         land on it and pass. `Pravesha::year` counts returns and the Muntha's \
         argument is named `completed_years` for this reason.\n\n",
        count(found.sign_agrees),
        count(found.cases),
        count(found.lord_agrees),
        count(found.cases),
        listed(&found.blind_spots),
    );
    Ok(())
}

fn the_degree(out: &mut String, sdk: &Context, births: &[Birth]) -> Result<(), String> {
    let (mut least, mut most, mut crossing) = (f64::INFINITY, 0.0_f64, 0_usize);
    for birth in births {
        let start = muntha(sdk, birth, 1, MunthaDegree::SignStart)?;
        let carried = muntha(sdk, birth, 1, MunthaDegree::NatalDegree)?;
        if start.sign != carried.sign {
            return Err(format!(
                "{}: the two degree readings disagree on the sign at the return",
                birth.name
            ));
        }
        let offset = carried.longitude_deg - start.longitude_deg;
        least = least.min(offset);
        most = most.max(offset);
        // Carrying the natal degree, a year's thirty degrees leave the sign
        // before the next return unless the lagna sat on a boundary.
        let before_the_end = (offset > 0.0) && (offset < 30.0);
        crossing += usize::from(before_the_end);
    }
    let _ = write!(
        out,
        "## 4. The two readings of the degree\n\n\
         The source progresses the Muntha 2°30′ a month and 5′ a day, which \
         fills a sign in exactly a year only if the year begins at the sign's \
         first degree — `MunthaDegree::SignStart`, the default. The rival \
         carries the natal lagna's degree into each new sign \
         (`MunthaDegree::NatalDegree`). Both give the same sign at the return, \
         on all {}.\n\n\
         They part by the natal lagna's degree within its sign, which over the \
         recorded births runs from {least:.2}° to {most:.2}°. Carried forward, a \
         year of progression takes the Muntha out of its sign **before** the \
         next return on {} — every birth whose lagna is not on a boundary — so \
         the two readings differ for any Tajika aspect taken to the Muntha late \
         in a year and for nothing else (crux C107).\n\n\
         The cap is {MOST_YEARS} years, shared with the returns.\n",
        plural(births.len(), "birth"),
        plural(crossing, "birth"),
    );
    Ok(())
}

/// The source's worked birth: Bombay, 20 August 1944, 07:11 IST (K.S.
/// Charak, *A Textbook of Varshaphala*, Chart III-1).
const WORKED_BIRTH_JD_UTC: f64 = 2_431_322.570_138_889;

/// What the source prints for its forty-first year's chart, as
/// (sign index, degrees, minutes).
const PRINTED_LAGNA: (f64, f64, f64) = (7.0, 9.0, 26.0);
const PRINTED_SUN: (f64, f64, f64) = (4.0, 3.0, 50.0);
const PRINTED_MOON: (f64, f64, f64) = (1.0, 9.0, 40.0);
/// 13:17:29 IST, hours.
const PRINTED_RETURN_IST_H: f64 = 13.0 + 17.0 / 60.0 + 29.0 / 3600.0;
const PRINTED_FIVE: [Graha; 5] = [
    Graha::Jupiter,
    Graha::Sun,
    Graha::Mars,
    Graha::Mars,
    Graha::Sun,
];

/// One profile's answer to the source's worked year.
struct Worked {
    return_seconds: f64,
    lagna_arcmin: f64,
    sun_arcmin: f64,
    moon_arcmin: f64,
    five: [Graha; 5],
    true_minus_mean_min: f64,
    /// Every planet's Vishwa bala, in the catalogue's order.
    vishwa: Vec<(Graha, String)>,
    strongest: Graha,
    /// The year lord, with every claimant's strength, aspect and count.
    year_lord: teistro::Varshesha,
}

fn worked_under(profile: &str) -> Result<Worked, String> {
    let sdk = Context::builder()
        .profile(profile)
        .ephemeris([Ephemeris::Builtin])
        .build()
        .map_err(|why| format!("{profile}: {why}"))?;
    let place = Place::new(
        Latitude::try_new(18.0 + 58.0 / 60.0).map_err(|why| why.to_string())?,
        Longitude::try_new(72.0 + 50.0 / 60.0).map_err(|why| why.to_string())?,
        Altitude::try_new(11.0).map_err(|why| why.to_string())?,
    );
    let offset = UtcOffset::try_from_seconds(19_800).map_err(|why| why.to_string())?;
    let bombay = ChartRequest::at(place, offset);
    let found = |at: JulianDay<Utc>| {
        sdk.chart()
            .reading(at, &bombay)
            .map(|envelope| envelope.value)
            .map_err(|why| format!("{profile}: founding: {why}"))
    };
    let birth = found(JulianDay::<Utc>::literal(WORKED_BIRTH_JD_UTC))?;
    let fortieth = |reading| {
        sdk.chart()
            .praveshas(&birth, reading, 40)
            .map_err(|why| format!("{profile}: {why}"))?
            .into_iter()
            .find(|one| one.year == 40)
            .ok_or_else(|| format!("{profile}: no fortieth return"))
    };
    let mean = fortieth(Reading::Mean)?;
    let true_return = fortieth(Reading::Sidereal)?;
    let annual = found(mean.at)?;
    let year = &annual.foundation;
    let at = |graha| {
        year.graha(graha)
            .map(|placed| placed.longitude_deg)
            .ok_or_else(|| format!("{profile}: no {graha:?}"))
    };
    let off = |deg: f64, (sign, degrees, minutes): (f64, f64, f64)| {
        (deg - (sign * 30.0 + degrees + minutes / 60.0)) * 60.0
    };
    let bearers = sdk
        .chart()
        .office_bearers(&birth, &annual, 40)
        .map_err(|why| format!("{profile}: {why}"))?;
    let bala = sdk
        .chart()
        .panchavargiya(&annual)
        .map_err(|why| format!("{profile}: {why}"))?;
    let strongest = bala
        .iter()
        .max_by_key(|one| one.vishwa)
        .ok_or_else(|| String::from("seven strengths"))?
        .graha;
    let vishwa = bala
        .iter()
        .map(|one| (one.graha, one.vishwa.to_string()))
        .collect();
    let year_lord = sdk
        .chart()
        .varshesha(&birth, &annual, 40, teistro::VarsheshaRules::default())
        .map_err(|why| format!("{profile}: {why}"))?;
    let ist = (mean.at.get() + 0.5 + 5.5 / 24.0).fract() * 24.0;
    Ok(Worked {
        return_seconds: (ist - PRINTED_RETURN_IST_H) * 3600.0,
        lagna_arcmin: off(year.lagna_deg, PRINTED_LAGNA),
        sun_arcmin: off(at(Graha::Sun)?, PRINTED_SUN),
        moon_arcmin: off(at(Graha::Moon)?, PRINTED_MOON),
        five: [
            bearers.muntha,
            bearers.janma_lagna,
            bearers.varsha_lagna,
            bearers.tri_rashi,
            bearers.dina_ratri,
        ],
        true_minus_mean_min: (true_return.at.get() - mean.at.get()) * 1440.0,
        vishwa,
        strongest,
        year_lord,
    })
}

fn the_worked_year(out: &mut String) -> Result<(), String> {
    let geo = worked_under("parashari-classical")?;
    let topo = worked_under("conformance-baseline")?;
    let five = |one: &Worked| {
        if one.five == PRINTED_FIVE {
            String::from("all five as printed")
        } else {
            format!("{:?}", one.five)
        }
    };
    let _ = write!(
        out,
        "\n## 5. The source's worked year, end to end\n\n\
         The source works one birth all the way through — Bombay, 20 August \
         1944, 07:11 IST — to its forty-first year's chart and that chart's \
         five office-bearers (Chart III-1). It is the only rank-2 value in \
         reach that checks the whole pipeline at once: the return, the chart \
         it founds, and the lords read from both. Its return is the **mean** \
         one — its Dhruvanka of 1d 6h 6m 29s for forty years is forty mean \
         sidereal years modulo a week — so that is the reading held here.\n\n\
         | against what the source prints | the default profile (geocentric, mean ayanamsha) | the conformance profile (topocentric, nutated) |\n\
         |---|---|---|\n\
         | the return, 13:17:29 IST | {:+.1} s | {:+.1} s |\n\
         | the annual lagna, Scorpio 9°26′ | {:+.1}′ | {:+.1}′ |\n\
         | the Sun, Leo 3°50′ | {:+.1}′ | {:+.1}′ |\n\
         | the Moon, Taurus 9°40′ | {:+.1}′ | **{:+.1}′** |\n\
         | the office-bearers, Jupiter, Sun, Mars, Mars, Sun | {} | {} |\n\
         | the true return, after the mean one | {:+.2} min | {:+.2} min |\n\n\
         The source prints whole arcminutes and seconds. On the default \
         profile the worst of its three positions is {:.1}′ out and its return \
         {:.1} s, which is an ephemeris a generation apart agreeing to \
         arcminutes and not a rounding. **Its positions are geocentric**: under \
         the topocentric profile its Moon is almost a degree out, which is \
         the Moon's parallax at Bombay and not an error. And the \"few \
         minutes\" it sets aside between the true return and the mean one are \
         the Sun's own perturbations on a mean ayanamsha; on a nutated one \
         nutation adds several more, which the source does not apply. Neither \
         moves an office-bearer.\n\n\
         ## 6. The five-fold strength, against the source's own table\n\n\
         The source tabulates the **Panchavargiya bala** of all seven planets \
         of that chart (Table VI-10) — five parts each, their total, and the \
         Vishwa bala the year lord is chosen by. Every one of those figures is \
         reproduced from the chart **the SDK founded**, not from the source's \
         own longitudes, and the arithmetic is exact: a unit holds 3600 \
         sub-sub units and nothing rounds. Every figure below equals the one \
         the source prints; all 49 of its cells — five parts, a total and a \
         Vishwa bala for each of the seven — are compared one by one in \
         `crates/tajika`'s own tests, which is where a wrong relation, table \
         cell, truncation or division would land.\n\n\
         | | {} |\n\
         |---|{}\n\
         | the Vishwa bala | {} |\n\n\
         The strongest is **{:?}**, as the source has it. The year lord is not \
         the strongest of the seven but the strongest of the five \
         office-bearers.\n\n",
        geo.return_seconds,
        topo.return_seconds,
        geo.lagna_arcmin,
        topo.lagna_arcmin,
        geo.sun_arcmin,
        topo.sun_arcmin,
        geo.moon_arcmin,
        topo.moon_arcmin,
        five(&geo),
        five(&topo),
        geo.true_minus_mean_min,
        topo.true_minus_mean_min,
        [geo.lagna_arcmin, geo.sun_arcmin, geo.moon_arcmin]
            .into_iter()
            .map(f64::abs)
            .fold(0.0, f64::max),
        geo.return_seconds.abs(),
        geo.vishwa
            .iter()
            .map(|(graha, _)| format!("{graha:?}"))
            .collect::<Vec<String>>()
            .join(" | "),
        "---|".repeat(geo.vishwa.len()),
        geo.vishwa
            .iter()
            .map(|(_, bala)| bala.clone())
            .collect::<Vec<String>>()
            .join(" | "),
        geo.strongest,
    );
    the_year_lord(out, &geo);
    Ok(())
}

/// The year lord of the source's worked chart, and the whole reckoning it
/// comes out of.
fn the_year_lord(out: &mut String, geo: &Worked) {
    let _ = write!(
        out,
        "## 7. The lord of that year\n\n\
         The year lord is **not** the strongest planet, nor even the strongest \
         office-bearer: it is the strongest office-bearer that **aspects the \
         annual lagna**. The source's own chart is the case that shows why the \
         rule needs all three parts, and the SDK reproduces its reckoning \
         claimant by claimant.\n\n\
         | claimant | Vishwa bala | portfolios | aspects the lagna |\n\
         |---|---|---|---|\n",
    );
    for claim in &geo.year_lord.claims {
        let _ = writeln!(
            out,
            "| {:?} | {} | {} | {} |",
            claim.graha,
            claim.vishwa,
            claim.portfolios,
            if claim.aspects_lagna { "yes" } else { "**no**" }
        );
    }
    let _ = write!(
        out,
        "\nJupiter leads on strength and stands in the **second** from the \
         lagna, a neutral house that gives no Tajika aspect, so the source \
         disqualifies it in as many words. The lord of the year is **{:?}** at \
         {}, chosen as `{:?}`. Saturn is stronger than any of them at {} and \
         holds no portfolio, so it never enters the reckoning at all.\n",
        geo.year_lord.graha,
        geo.year_lord.vishwa,
        geo.year_lord.chosen,
        geo.vishwa
            .iter()
            .find(|(graha, _)| *graha == teistro::catalogue::Graha::Saturn)
            .map_or_else(|| String::from("—"), |(_, bala)| bala.clone()),
    );
}

pub(crate) fn generate(root: &Path) -> i32 {
    match page(root) {
        Ok(text) => write(root, &[Output::new(PAGE, text)]),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    match page(root) {
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask muntha") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}
