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

use teistro::House;
use teistro::catalogue::DashaSystem;
use teistro::catalogue::{Graha, Rashi};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::tajika::{
    AnnualStates, Bala, Chosen, Favour, MOST_YEARS, MoonBenefic, MoonMayRule, MunthaDegree,
    Qualification, RASHYANTA_DEG, Reading, SEVEN, Strength, TambiraMover, YOGA_STRONG_FROM,
    YOGA_WEAK_BELOW, YearYoga, YearYogas, Yoga, YogaRules,
};
use teistro::{
    AddSign, AnnualDasha, AnnualDashaRules, ChartRequest, Context, Document, Ephemeris,
    HarshaGrade, HarshaRules, HousePoints, PeriodRow, RogaReading, Saham, SahamReading, SahamRules,
    SahamTerm, UtcOffset, VenusPlace, YearClock, YearDasha,
};

use crate::births::{Birth, CHARTS, births};
use crate::generated::{Output, check, write};
use crate::measure::{Claim, capitalised, count, fill, listed, plural, spelled, table, times};

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

/// The context every reading here is made under; a worker builds its own,
/// since a context serves one thread.
fn conformance() -> Result<Context, String> {
    Context::builder()
        .profile("conformance-baseline")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .map_err(|why| format!("the conformance profile: {why}"))
}

fn page(root: &Path) -> Result<String, String> {
    let sdk = conformance()?;
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
    let swept = sweep(&sdk, &births)?;
    the_kinds(&mut out, &swept);
    the_sixteen(&mut out, &swept);
    the_floors(&mut out, &swept)?;
    what_spoils_an_ithasala(&mut out, &sdk, &swept);
    what_happens_next(&mut out, &swept);
    the_sahams(&mut out, &swept);
    the_harsha(&mut out, &swept);
    the_strength(&mut out, &swept);
    the_kuttha(&mut out, &swept);
    the_year_lords(&mut out, &swept);
    the_annual_dashas(&mut out, &swept);
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
    /// The Sun and Mars of that chart, the source's own worked pair.
    sun_and_mars: teistro::Between,
    /// How many of the twenty-one pairs make a yoga.
    yogas: usize,
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
    let pairs = sdk
        .chart()
        .drishtis(&annual)
        .map_err(|why| format!("{profile}: {why}"))?;
    let sun_and_mars = pairs
        .iter()
        .find(|pair| {
            [pair.faster, pair.slower].contains(&Graha::Sun)
                && [pair.faster, pair.slower].contains(&Graha::Mars)
        })
        .copied()
        .ok_or_else(|| String::from("the Sun and Mars"))?;
    let yogas = pairs.iter().filter(|pair| pair.yoga.is_some()).count();
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
        sun_and_mars,
        yogas,
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
    the_aspects(out, &geo);
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

/// The source's worked Ithasala, and how much of a chart makes a yoga.
/// How many years of each recorded birth the yoga sweep follows.
///
/// Forty, not the Muntha's hundred and twenty: the sweep founds a chart
/// per year rather than rotating a sign, and forty years of fifty-five
/// births is already two thousand annual charts and twenty-one pairs
/// each. It is the span a reader of a birth chart actually asks about.
const SWEEP_YEARS: u16 = 40;

/// The lower floors §11 reads the corpus under, in Vishwa units: below
/// each, a planet with no dignity is **weak**.
///
/// Five is the default, where Charak's office-bearer floor and the
/// graded scale's *Nirbali* meet; four and six are its neighbours; eight
/// and ten walk up to where the default strong floor begins, above
/// which the two would cross.
const WEAK_FLOORS: [i64; 5] = [4, 5, 6, 8, 10];

/// The upper floors, in Vishwa units: from each, a planet is **strong**
/// on its bala alone.
///
/// Ten is the default, the graded scale's *Poorna*; five is the reading
/// with no middle at all; fifteen is *Parakrami*; and twenty is the top
/// of the scale, which only a planet perfect in all five parts could
/// reach, so under it only the dignities hold anyone up.
const STRONG_FLOORS: [i64; 6] = [5, 8, 10, 12, 15, 20];

/// What one candidate floor makes of the corpus, on its own side of it:
/// weak readings and weak pairs below a lower floor, strong ones from
/// an upper.
#[derive(Clone, Copy, Default)]
struct AtFloor {
    /// Readings of a planet on this side.
    readings: usize,
    /// Judged matters with **both** lords on this side.
    both: usize,
}

/// Every kind of pair the sweep found, counted.
#[derive(Default)]
struct Kinds {
    /// Years asked for: one birth's [`SWEEP_YEARS`] times the births.
    asked: usize,
    /// Returns the search actually produced. Short of `asked` where the
    /// built-in ephemeris's span runs out before the fortieth year.
    returned: usize,
    /// Annual charts founded.
    charts: usize,
    /// Pairs read, twenty-one to a chart.
    pairs: usize,
    /// Pairs whose signs aspect each other at all.
    aspecting: usize,
    /// Vartamana: behind by a degree or more, inside the orb.
    vartamana: usize,
    /// Poorna stated by the table: behind by less than a degree.
    poorna_stated: usize,
    /// The contested band: past by less than a degree.
    disputed: usize,
    /// Ishrafa on every reading: past by a degree or more.
    ishrafa: usize,
    /// Bhavishyat: outside the orb, reaching from a sign's end.
    bhavishyat: usize,
    /// Years whose annual chart the SDK **refused** to found, and the
    /// births they belong to.
    ///
    /// A refusal is not a skip. A sweep that swallowed one would report a
    /// smaller corpus as a cleaner one, so every year asked for is
    /// accounted for here and named on the page.
    refused: usize,
    /// Which births those years belong to, each named once.
    refused_births: Vec<String>,
    /// Births whose returns ran out before [`SWEEP_YEARS`] — a different
    /// absence from a refusal, and counted apart from one.
    cut_short: Vec<String>,
    /// Why, in the SDK's own words, from the first refusal seen.
    refusal: String,
    /// Questions asked: one per chart per house of the twelve.
    matters: usize,
    /// Matters whose two lords are one planet, so no pair is judged.
    same_lord: usize,
    /// How often each of the sixteen held, in `YearYoga::ALL` order.
    yogas: [usize; 16],
    /// How often each clause of the source's `unqualified` held of the
    /// Moon, in the order the definition states them: exalted,
    /// debilitated, aspected, own Hudda, own Drekkana, own Navamsha.
    ///
    /// Khallasara needs every one of them false. Counting the clauses
    /// rather than the verdict is what turns "it never holds" from a
    /// shrug into a reason.
    moon_clauses: [usize; 6],
    /// Charts in which the Moon was unqualified on every clause.
    moon_unqualified: usize,
    /// Charts whose lagna is ruled by a luminary, which rules one sign.
    ///
    /// The same-lord count has an identity to satisfy: every chart
    /// contributes its first house, and every chart but these
    /// contributes one more. Counting the term separately is what lets
    /// the page state that identity instead of printing a number nobody
    /// can check.
    luminary_lagna: usize,
    /// Readings of a planet's strength: seven to a chart.
    strengths_read: usize,
    /// Readings strong by a **dignity** alone — exalted, or in a sign it
    /// rules — which no floor can make weak.
    dignified: usize,
    /// Every reading's Vishwa bala, counted in bands of two units from
    /// the bottom of the scale: `[0, 2)`, `[2, 4)`, … `[18, 20]`.
    vishwa_bands: [usize; 10],
    /// The weakest reading seen, which bounds how low a floor can matter.
    weakest: Option<Bala>,
    /// What each of [`WEAK_FLOORS`] makes of the corpus, in that order.
    weak_side: [AtFloor; WEAK_FLOORS.len()],
    /// What each of [`STRONG_FLOORS`] makes of it.
    strong_side: [AtFloor; STRONG_FLOORS.len()],
    /// Readings neither strong nor weak under the default floors,
    /// counted from the default verdict itself rather than by difference,
    /// so the partition it closes is a check and not an identity.
    middling: usize,
    /// Judged matters whose pair is neither both weak nor both strong
    /// under the default floors, counted the same way.
    mixed: usize,
    /// Readings of a planet that are retrograde, and that are combust,
    /// under the profile's combustion table: the ground Rudda stands on.
    retrograde: usize,
    /// See [`Kinds::retrograde`].
    combust: usize,
    /// For each clause of an affliction, in the order the source states
    /// them, the Ruddas in which it held of either lord. Sized from the
    /// clauses the module reports, not fixed here, so a clause it gains
    /// is counted rather than dropped.
    rudda_clauses: Vec<usize>,
    /// Ruddas that held on *under malefic influence* and nothing else,
    /// which bounds every narrower reading of that clause (crux C118).
    rudda_malefic_alone: usize,
    /// Judged matters whose pair do not aspect: the most Nakta, Yamaya
    /// and Tambira can hold in between them.
    unaspecting: usize,
    /// Of those, the ones whose karyesha stands at a sign's end.
    karyesha_at_end: usize,
    /// Of those, the ones whose karyesha is not retrograde, and so goes
    /// on into the next sign: the most Tambira can hold in.
    karyesha_moving_on: usize,
    /// Matters in which Tambira holds when **either** lord may move.
    tambira_either: usize,
    /// Matters with an Ithasala and the Moon, not one of the pair, at a
    /// sign's end: the most Gairi-Kamboola could hold in were every such
    /// Moon unqualified.
    moon_at_end: usize,
    /// Of those, the ones whose Moon was unqualified.
    moon_at_end_unqualified: usize,
    /// What each reading of the sahams makes of every chart.
    sahams: SahamCounts,
    /// What the Harsha bala makes of every chart.
    harsha: HarshaCounts,
    /// What a saham's strength makes of every saham of every chart.
    strength: StrengthCounts,
    /// What Kuttha makes of every judged matter.
    kuttha: KutthaCounts,
    /// The lord of every year, under Charak's chain and the Nilakanthi's.
    lords: LordCounts,
    /// The three annual dashas of every year.
    dashas: DashaCounts,
}

/// The annual dashas of every recorded year (`03-design/annual-dashas.md`).
#[derive(Default)]
struct DashaCounts {
    /// Years read, each for all three systems at once.
    years: usize,
    /// Periods listed, mahadashas and antardashas, over all three.
    periods: usize,
    /// Years whose next return the sweep also found, so that the Sun's
    /// clock can be held to closing on it.
    closings: usize,
    /// The farthest the Sun's clock closed from the next return, seconds.
    worst_close_s: f64,
    /// The farthest an even spread stood from the Sun's clock at any of the
    /// Mudda's boundaries, days.
    worst_even_days: f64,
    /// Patyayini years in which two lords share a krishamsha to the
    /// nanoarcsecond, so that the tie rule decided the order.
    ties: usize,
    /// Patyayini years that open with the lagna.
    lagna_first: usize,
}

impl DashaCounts {
    /// Another worker's counts added to these.
    fn absorb(&mut self, other: &DashaCounts) {
        self.years += other.years;
        self.periods += other.periods;
        self.closings += other.closings;
        self.worst_close_s = self.worst_close_s.max(other.worst_close_s);
        self.worst_even_days = self.worst_even_days.max(other.worst_even_days);
        self.ties += other.ties;
        self.lagna_first += other.lagna_first;
    }
}

/// The lord of every recorded year, by the step that chose it.
#[derive(Default)]
struct LordCounts {
    /// Years, one per annual chart read.
    years: usize,
    /// Under Charak's chain, in `Chosen::ALL` order.
    charak: [usize; Chosen::ALL.len()],
    /// Under the Nilakanthi's reading of the Moon, the same order.
    nilakanthi: [usize; Chosen::ALL.len()],
    /// Years in which the chain would have given the Moon the year under
    /// the default: it stepped down, or it was succeeded.
    moon_led: usize,
    /// Of the default's successors, those that hold no portfolio.
    outside: usize,
    /// Years whose lord the Nilakanthi's reading changes.
    changed: usize,
    /// Years in which the Moon holds the year under the default: only a
    /// Moon in Cancer, succeeding itself.
    moon_rules: usize,
}

/// Kuttha over every judged matter: where it held under each reading of
/// the Moon, and which clause turned away a pair strong enough to ask.
#[derive(Default)]
struct KutthaCounts {
    /// Matters whose two lords are both strong: the most Kuttha can hold
    /// in, counted here from `favour` so the pass can compare it with the
    /// floors' own partition.
    strong_pairs: usize,
    /// Matters in which `favour` finds both lords favoured, which must be
    /// exactly those in which the yoga held.
    favoured: usize,
    /// Matters in which Kuttha holds under the commentary's waxing Moon.
    waxing: usize,
    /// Strong-pair matters in which Kuttha held under the waxing Moon and
    /// not under the default: none, since the reading only removes a
    /// benefic. Counted rather than assumed, so the pass can fail on it.
    waxing_only: usize,
    /// For each clause, in `Favour::clauses` order, the strong-pair
    /// matters it turned away by failing of either lord.
    turned_away: [usize; 4],
    /// Of those, the ones in which it was the only clause to fail.
    alone: [usize; 4],
    /// The clauses' own names, taken from the first favour read, so the
    /// page never types one out.
    names: [&'static str; 4],
}

/// Every saham of every chart, sorted by which of the source's two lists
/// it meets.
#[derive(Default)]
struct StrengthCounts {
    /// Placements meeting some strong clause and no weak one.
    strong_only: usize,
    /// Some weak clause and no strong one.
    weak_only: usize,
    /// Clauses on both lists at once.
    both: usize,
    /// How often each strong clause holds, in `SahamStrength::strong`
    /// order.
    strong: [usize; 12],
    /// How often each weak clause holds, in `SahamStrength::weak` order.
    weak: [usize; 5],
}

/// The Harsha bala of every chart, under the verse's reading and the
/// program's.
#[derive(Default)]
struct HarshaCounts {
    /// How often each of the seven, in `SEVEN` order, holds each grade,
    /// Nirbala to extraordinary.
    grades: [[usize; 5]; 7],
    /// Charts whose Venus holds another total with its joy in the 12th.
    venus_moved: usize,
    /// Charts whose Sun takes the day-and-night part: must be the charts
    /// read by day, since the Sun is male.
    sun_dina_ratri: usize,
    /// Charts whose Moon takes it: must be the charts read by night.
    moon_dina_ratri: usize,
}

/// The sahams of every chart, under the source's readings and each rival.
///
/// Every column is one per saham, in `Saham::ALL` order.
struct SahamCounts {
    /// Charts cast between sunrise and sunset, read by the day formulas.
    by_day: usize,
    /// Sahams carried one sign on under the default reading.
    added: Vec<usize>,
    /// Sahams whose longitude moves when no sign is ever added. Must be
    /// exactly `added`: a pass that could not see that would be counting
    /// something other than the rule.
    never_moved: Vec<usize>,
    /// Sahams standing in another sign when "between" counts whole signs.
    signs_moved: Vec<usize>,
    /// Sahams standing in another sign under equal houses from the lagna.
    equal_moved: Vec<usize>,
    /// Charts whose Roga stands in another sign under the Saturn reading.
    roga_moved: usize,
}

impl Default for SahamCounts {
    fn default() -> SahamCounts {
        let column = || vec![0; Saham::ALL.len()];
        SahamCounts {
            by_day: 0,
            added: column(),
            never_moved: column(),
            signs_moved: column(),
            equal_moved: column(),
            roga_moved: 0,
        }
    }
}

/// Every pair of every annual chart of every recorded birth, sorted into
/// Table X-3's four kinds and the one band the source does not place.
///
/// It reads through `sdk.chart().drishtis`, so the module it measures is
/// the module that answers — a pass that kept its own copy of the rule
/// would agree with itself and prove nothing.
fn sweep(sdk: &Context, births: &[Birth]) -> Result<Kinds, String> {
    let mut kinds = Kinds::default();
    let mut dasha_years = Vec::new();
    for birth in births {
        let years = sdk
            .chart()
            .praveshas(&birth.document, Reading::Sidereal, SWEEP_YEARS)
            .map_err(|why| format!("{}: its returns: {why}", birth.name))?;
        kinds.asked += usize::from(SWEEP_YEARS);
        kinds.returned += years.len();
        if years.len() < usize::from(SWEEP_YEARS) {
            kinds.cut_short.push(birth.name.clone());
        }
        for (index, year) in years.iter().copied().enumerate() {
            let next = years.get(index + 1).map(|after| after.at.get());
            // A high-latitude birth in its polar summer has no sunrise to
            // divide a day by, and the SDK refuses rather than inventing
            // one. Counted and named, never swallowed.
            let annual = match sdk.chart().reading(year.at, &birth.request()) {
                Ok(envelope) => envelope.value,
                Err(why) => {
                    kinds.refused += 1;
                    if kinds.refusal.is_empty() {
                        kinds.refusal = why.to_string();
                    }
                    if kinds.refused_births.last() != Some(&birth.name) {
                        kinds.refused_births.push(birth.name.clone());
                    }
                    continue;
                }
            };
            let pairs = sdk
                .chart()
                .drishtis(&annual)
                .map_err(|why| format!("{}: its pairs: {why}", birth.name))?;
            kinds.charts += 1;
            kinds.pairs += pairs.len();
            let how = sdk
                .chart()
                .qualification(&annual, Graha::Moon)
                .map_err(|why| format!("{}: the Moon's qualification: {why}", birth.name))?;
            // Walked by name, never by index: a clause reordered in the
            // struct would otherwise relabel a row of the page silently.
            for (at, (_, holds)) in how.clauses().into_iter().enumerate() {
                if holds {
                    if let Some(slot) = kinds.moon_clauses.get_mut(at) {
                        *slot += 1;
                    }
                }
            }
            if how.is_unqualified() {
                kinds.moon_unqualified += 1;
            }
            let chart_strengths = strengths_of(sdk, &annual, &birth.name, &mut kinds)?;
            let states = sdk
                .chart()
                .annual_states(&annual)
                .map_err(|why| format!("{}: its states: {why}", birth.name))?;
            kinds.retrograde += states.retrograde.len();
            kinds.combust += states.combust.len();
            let lagna = Rashi::from_id(sign_of(annual.foundation.lagna_deg))
                .ok_or_else(|| format!("{}: a lagna in no sign", birth.name))?;
            if matches!(lagna.attributes().lord, Graha::Sun | Graha::Moon) {
                kinds.luminary_lagna += 1;
            }
            let chart = Asked {
                annual: &annual,
                name: &birth.name,
                strengths: &chart_strengths,
                states: &states,
                moon_unqualified: how.is_unqualified(),
            };
            ask_every_house(sdk, &chart, &mut kinds)?;
            read_the_year(sdk, birth, &annual, year.year, &mut kinds)?;
            dasha_years.push(DashaYear {
                birth,
                annual: annual.clone(),
                year: year.year,
                next,
            });
            for pair in pairs {
                if !pair.drishti.is_aspect() {
                    continue;
                }
                kinds.aspecting += 1;
                if pair.disputed() {
                    kinds.disputed += 1;
                }
                match pair.yoga {
                    Some(Yoga::IthasalaVartamana) => kinds.vartamana += 1,
                    Some(Yoga::IthasalaPoorna) if !pair.disputed() => kinds.poorna_stated += 1,
                    Some(Yoga::IthasalaBhavishyat) => kinds.bhavishyat += 1,
                    Some(Yoga::Ishrafa) => kinds.ishrafa += 1,
                    _ => {}
                }
            }
        }
    }
    kinds.dashas = count_every_annual_dasha(&dasha_years)?;
    sweep_holds(&kinds)?;
    Ok(kinds)
}

/// One year the annual dashas are counted over: gathered by the sweep and
/// counted after it, on every core, since the Sun's clock is the one reading
/// in the sweep that costs more than the rest of a year's together.
struct DashaYear<'b> {
    birth: &'b Birth,
    annual: Document,
    year: u16,
    next: Option<f64>,
}

/// Every gathered year's annual dashas, counted by as many workers as the
/// machine has cores, each with its own context, over consecutive runs of
/// the years. The runs are joined in order and each stops at its first
/// refusal, so the refusal reported is the sweep's first, as it would be
/// counted on one thread; the counts are sums and greatest values, which do
/// not depend on the order they are taken in.
fn count_every_annual_dasha(years: &[DashaYear<'_>]) -> Result<DashaCounts, String> {
    let workers = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
    let run = years.len().div_ceil(workers).max(1);
    std::thread::scope(|scope| {
        let counted: Vec<_> = years
            .chunks(run)
            .map(|part| {
                scope.spawn(move || {
                    let sdk = conformance()?;
                    let mut counts = DashaCounts::default();
                    for one in part {
                        count_annual_dashas(
                            &sdk,
                            one.birth,
                            &one.annual,
                            one.year,
                            one.next,
                            &mut counts,
                        )?;
                    }
                    Ok::<DashaCounts, String>(counts)
                })
            })
            .collect();
        let mut total = DashaCounts::default();
        for worker in counted {
            let part = worker
                .join()
                .map_err(|_| String::from("a worker counting the annual dashas panicked"))??;
            total.absorb(&part);
        }
        Ok(total)
    })
}

/// What one year's chart gives the sahams, the Harsha bala and a saham's
/// strength: the readings that need the year and not only its pairs.
fn read_the_year(
    sdk: &Context,
    birth: &Birth,
    annual: &Document,
    year: u16,
    kinds: &mut Kinds,
) -> Result<(), String> {
    count_sahams(sdk, annual, &birth.name, &mut kinds.sahams)?;
    count_harsha(sdk, annual, &birth.name, &mut kinds.harsha)?;
    let lord = count_year_lord(
        sdk,
        &birth.document,
        annual,
        year,
        &birth.name,
        &mut kinds.lords,
    )?;
    count_strength(sdk, annual, lord, &birth.name, &mut kinds.strength)
}

/// Every identity the sweep's counts must satisfy, each a failure and not
/// a sentence.
fn sweep_holds(kinds: &Kinds) -> Result<(), String> {
    // The same-lord count is decomposable, so decompose it and refuse a
    // run that disagrees: every chart contributes its first house, and
    // every chart but a luminary-ruled one contributes a second. A
    // printed figure nobody can check is the part of a generated page
    // that rots, so this is a failure and not a sentence.
    let expected = 2 * kinds.charts - kinds.luminary_lagna;
    if expected != kinds.same_lord {
        return Err(format!(
            "the same-lord count does not decompose: 2 x {} charts less {} \
             luminary lagnas is {expected}, and {} matters were counted",
            kinds.charts, kinds.luminary_lagna, kinds.same_lord,
        ));
    }
    floors_hold(kinds)?;
    projections_hold(kinds)?;
    sahams_hold(&kinds.sahams)?;
    harsha_holds(kinds.charts, kinds.sahams.by_day, &kinds.harsha)
}

/// One annual chart, with what the sweep already read of it.
struct Asked<'a> {
    annual: &'a Document,
    name: &'a str,
    strengths: &'a ChartStrengths,
    states: &'a AnnualStates,
    moon_unqualified: bool,
}

/// Asks one annual chart all twelve of its matters, and counts what each
/// answered.
fn ask_every_house(sdk: &Context, chart: &Asked<'_>, kinds: &mut Kinds) -> Result<(), String> {
    let (annual, name, chart_strengths) = (chart.annual, chart.name, chart.strengths);
    for number in 1..=12u8 {
        let house =
            House::try_new(number).map_err(|why| format!("{name}: house {number}: {why}"))?;
        let asked = sdk
            .chart()
            .tajika_yogas(annual, house)
            .map_err(|why| format!("{name}: its yogas: {why}"))?;
        kinds.matters += 1;
        if asked.same_lord {
            kinds.same_lord += 1;
        } else {
            partition(chart_strengths, asked.lagnesha, asked.karyesha, kinds);
            count_kuttha(sdk, chart, house, &asked, &mut kinds.kuttha)?;
        }
        // Counted by **matter**, through `holds`: one matter can hold a
        // yoga through several planets -- Manau through each malefic, Nakta
        // and Yamaya through each intermediary, Dutthottha-Davira through
        // each strong third -- and a column of shares of the matters asked
        // must not count one matter twice.
        for (slot, yoga) in kinds.yogas.iter_mut().zip(YearYoga::ALL) {
            if asked.holds(yoga) == Some(true) {
                *slot += 1;
            }
        }
        what_spoiled(&asked, kinds);
        what_enters(sdk, chart, house, &asked, kinds)?;
    }
    Ok(())
}

/// Counts one judged matter into Kuttha's tally, from the chart's
/// favours and from the yoga itself, and refuses a matter where the two
/// disagree.
fn count_kuttha(
    sdk: &Context,
    chart: &Asked<'_>,
    house: House,
    asked: &YearYogas,
    counts: &mut KutthaCounts,
) -> Result<(), String> {
    let name = chart.name;
    let favour_of = |graha: Graha| {
        chart
            .strengths
            .favour
            .iter()
            .find(|one| one.graha == graha)
            .copied()
            .ok_or_else(|| format!("{name}: no favour read for {graha:?}"))
    };
    let lords = [favour_of(asked.lagnesha)?, favour_of(asked.karyesha)?];
    let favoured = lords.iter().all(|one| one.is_favoured());
    let held = asked.holds(YearYoga::Kuttha) == Some(true);
    if favoured != held {
        return Err(format!(
            "{name}: house {}: both lords favoured is {favoured}, and Kuttha held is {held}",
            house.get()
        ));
    }
    counts.favoured += usize::from(favoured);
    if !lords.iter().all(|one| one.strength.is_strong()) {
        return Ok(());
    }
    counts.strong_pairs += 1;
    counts.names = lords[0].clauses().map(|(clause, _)| clause);
    // A clause fails the matter where it fails of either lord.
    let failing =
        lords
            .map(Favour::clauses)
            .into_iter()
            .fold([false; 4], |mut failing, clauses| {
                for (slot, (_, holds)) in failing.iter_mut().zip(clauses) {
                    *slot |= !holds;
                }
                failing
            });
    let failures = failing.iter().filter(|fails| **fails).count();
    for ((away, alone), fails) in counts
        .turned_away
        .iter_mut()
        .zip(counts.alone.iter_mut())
        .zip(failing)
    {
        *away += usize::from(fails);
        *alone += usize::from(fails && failures == 1);
    }
    // Asked of every strong pair, not only where the default held, so that
    // "the waxing reading only removes" is a count that can fail.
    let waxing = sdk
        .chart()
        .tajika_yogas_with_rules(
            chart.annual,
            house,
            YogaRules {
                moon_benefic: MoonBenefic::Waxing,
                ..YogaRules::default()
            },
        )
        .map_err(|why| format!("{name}: its yogas, the Moon waxing: {why}"))?
        .holds(YearYoga::Kuttha)
        == Some(true);
    counts.waxing += usize::from(waxing);
    counts.waxing_only += usize::from(waxing && !held);
    Ok(())
}

/// Counts the ground Gairi-Kamboola and Tambira stand on in one matter:
/// the steps between "the pair can be reached from the next sign" and
/// "it was", so that a count near zero says which step emptied it.
fn what_enters(
    sdk: &Context,
    chart: &Asked<'_>,
    house: House,
    asked: &YearYogas,
    kinds: &mut Kinds,
) -> Result<(), String> {
    let Some(pair) = asked.between else {
        return Ok(());
    };
    let at_end = |graha: Graha| {
        chart
            .annual
            .foundation
            .graha(graha)
            .is_some_and(|placed| placed.longitude_deg.rem_euclid(30.0) >= RASHYANTA_DEG)
    };
    let lords = [asked.lagnesha, asked.karyesha];
    if asked.holds(YearYoga::Ithasala) == Some(true)
        && !lords.contains(&Graha::Moon)
        && at_end(Graha::Moon)
    {
        kinds.moon_at_end += 1;
        if chart.moon_unqualified {
            kinds.moon_at_end_unqualified += 1;
        }
    }
    if pair.drishti.is_aspect() {
        return Ok(());
    }
    kinds.unaspecting += 1;
    if at_end(asked.karyesha) {
        kinds.karyesha_at_end += 1;
        if !chart.states.is_retrograde(asked.karyesha) {
            kinds.karyesha_moving_on += 1;
        }
    }
    // Asked again only where the wider reading could differ: with neither
    // lord at a sign's end, neither reading has anything to move.
    if lords.into_iter().any(at_end) {
        let either = sdk
            .chart()
            .tajika_yogas_with_rules(
                chart.annual,
                house,
                YogaRules {
                    tambira: TambiraMover::EitherLord,
                    ..YogaRules::default()
                },
            )
            .map_err(|why| format!("{}: its yogas, either lord moving: {why}", chart.name))?;
        if either.holds(YearYoga::Tambira) == Some(true) {
            kinds.tambira_either += 1;
        }
    }
    Ok(())
}

/// The two projected yogas sit under chains of ceilings, each a subset of
/// the one before, and the wider Tambira under none of the narrower's
/// counts. A run that breaks one is a module that projects something it
/// should not, so it fails rather than printing.
fn projections_hold(kinds: &Kinds) -> Result<(), String> {
    let chain = [
        (
            "Tambira",
            vec![
                ("matters whose pair do not aspect", kinds.unaspecting),
                ("with the karyesha at a sign's end", kinds.karyesha_at_end),
                ("and not retrograde", kinds.karyesha_moving_on),
                ("Tambira", held_in(kinds, YearYoga::Tambira)),
            ],
        ),
        (
            "Gairi-Kamboola",
            vec![
                (
                    "matters with an Ithasala",
                    held_in(kinds, YearYoga::Ithasala),
                ),
                ("with the Moon at a sign's end", kinds.moon_at_end),
                ("and unqualified", kinds.moon_at_end_unqualified),
                ("Gairi-Kamboola", held_in(kinds, YearYoga::GairiKamboola)),
            ],
        ),
    ];
    for (yoga, steps) in chain {
        for pair in steps.windows(2) {
            if let [(wider, above), (narrower, below)] = pair {
                if below > above {
                    return Err(format!(
                        "{yoga}: {below} {narrower} is more than the {above} {wider} it narrows"
                    ));
                }
            }
        }
    }
    let (narrow, wide) = (held_in(kinds, YearYoga::Tambira), kinds.tambira_either);
    if wide < narrow {
        return Err(format!(
            "Tambira held in {narrow} matters moving the karyesha and in only {wide} moving either"
        ));
    }
    Ok(())
}

/// Which of the seven stand on one side of each candidate floor, in
/// that floor list's order.
type Sides = Vec<Vec<(Graha, bool)>>;

/// One annual chart's strengths: which of the seven are weak at each
/// lower floor, which strong at each upper one, and the default verdict
/// for each.
struct ChartStrengths {
    weak: Sides,
    strong: Sides,
    default: Vec<Strength>,
    /// Each of the seven's standing to Kuttha's clauses, read once for the
    /// chart and looked up by every matter.
    favour: Vec<Favour>,
}

/// Rules with the lower floor at `weak_below` and the upper at
/// `strong_from`, everything else the default.
fn floors(weak_below: i64, strong_from: i64) -> YogaRules {
    YogaRules {
        weak_below: Bala::new(weak_below, 0, 0),
        strong_from: Bala::new(strong_from, 0, 0),
        ..YogaRules::default()
    }
}

/// Which of the seven one `side` of `rules` puts a planet on, read
/// through the façade.
fn side_of(
    sdk: &Context,
    annual: &Document,
    name: &str,
    rules: YogaRules,
    side: fn(Strength) -> bool,
    at: &mut AtFloor,
) -> Result<Vec<(Graha, bool)>, String> {
    let mut row = Vec::with_capacity(SEVEN.len());
    for graha in SEVEN {
        let on = side(
            sdk.chart()
                .strength_with_rules(annual, graha, rules)
                .map_err(|why| format!("{name}: {graha:?}'s strength: {why}"))?,
        );
        if on {
            at.readings += 1;
        }
        row.push((graha, on));
    }
    Ok(row)
}

/// One annual chart's strengths under every candidate floor, and the
/// facts about them that no floor moves.
///
/// Read through the façade rather than recomputed, so the pass measures
/// the module that answers: a pass that kept its own copy of `is_weak`
/// would agree with itself and prove nothing. Each side holds the other
/// floor at its default, which is safe because the two are separable —
/// weak turns only on the lower floor and strong only on the upper, so
/// long as they do not cross, and no pair in the lists does.
fn strengths_of(
    sdk: &Context,
    annual: &Document,
    name: &str,
    kinds: &mut Kinds,
) -> Result<ChartStrengths, String> {
    let default = YogaRules::default();
    let strong_from = default.strong_from.units();
    let weak_below = default.weak_below.units();
    let mut weak = Vec::with_capacity(WEAK_FLOORS.len());
    for (units, at) in WEAK_FLOORS.into_iter().zip(kinds.weak_side.iter_mut()) {
        let rules = floors(units, strong_from);
        weak.push(side_of(sdk, annual, name, rules, Strength::is_weak, at)?);
    }
    let mut strong = Vec::with_capacity(STRONG_FLOORS.len());
    for (units, at) in STRONG_FLOORS.into_iter().zip(kinds.strong_side.iter_mut()) {
        let rules = floors(weak_below, units);
        strong.push(side_of(sdk, annual, name, rules, Strength::is_strong, at)?);
    }
    // What no floor moves, and the default verdict, read once.
    let mut verdicts = Vec::with_capacity(SEVEN.len());
    for graha in SEVEN {
        let how = sdk
            .chart()
            .strength(annual, graha)
            .map_err(|why| format!("{name}: {graha:?}'s strength: {why}"))?;
        kinds.strengths_read += 1;
        if how.exalted || how.own_sign {
            kinds.dignified += 1;
        }
        if how.is_middling() {
            kinds.middling += 1;
        }
        // Twenty itself, reachable only by a planet perfect in all five
        // parts, belongs to the top band and not to an eleventh.
        let band = usize::try_from(how.vishwa.units()).unwrap_or(0).min(19) / 2;
        if let Some(slot) = kinds.vishwa_bands.get_mut(band) {
            *slot += 1;
        }
        kinds.weakest = Some(kinds.weakest.map_or(how.vishwa, |low| low.min(how.vishwa)));
        verdicts.push(how);
    }
    let favour = SEVEN
        .into_iter()
        .map(|graha| {
            sdk.chart()
                .favour(annual, graha)
                .map_err(|why| format!("{name}: {graha:?}'s favour: {why}"))
        })
        .collect::<Result<_, _>>()?;
    Ok(ChartStrengths {
        weak,
        strong,
        default: verdicts,
        favour,
    })
}

/// Counts one judged matter into every floor's tally, and into the
/// default partition.
fn partition(chart: &ChartStrengths, lagnesha: Graha, karyesha: Graha, kinds: &mut Kinds) {
    let both = |row: &Vec<(Graha, bool)>| {
        let on = |graha: Graha| row.iter().any(|(one, is)| *one == graha && *is);
        on(lagnesha) && on(karyesha)
    };
    for (row, at) in chart.weak.iter().zip(kinds.weak_side.iter_mut()) {
        if both(row) {
            at.both += 1;
        }
    }
    for (row, at) in chart.strong.iter().zip(kinds.strong_side.iter_mut()) {
        if both(row) {
            at.both += 1;
        }
    }
    let of = |graha: Graha| chart.default.iter().copied().find(|one| one.graha == graha);
    if let (Some(lagnesha), Some(karyesha)) = (of(lagnesha), of(karyesha)) {
        let both_weak = lagnesha.is_weak() && karyesha.is_weak();
        let both_strong = lagnesha.is_strong() && karyesha.is_strong();
        if !both_weak && !both_strong {
            kinds.mixed += 1;
        }
    }
}

/// Where a floor list holds `bala`, or a refusal naming the list.
///
/// The page's "default" rows are found by value from the library's own
/// constants rather than by a hard-coded index, so moving a default
/// moves the page, and a default the list does not carry fails the pass.
fn position_of(list: &[i64], bala: Bala, which: &str) -> Result<usize, String> {
    list.iter()
        .position(|units| Bala::new(*units, 0, 0) == bala)
        .ok_or_else(|| format!("the {which} floors do not include the default, {bala}"))
}

/// The things every reading of the floors must satisfy, refused rather
/// than printed when one does not.
///
/// Each is a fact about the **module**, not about the corpus, so a
/// failure here is a defect in `Strength` and not a surprising number:
///
/// 1. under the default floors, readings **partition** into weak,
///    middling and strong, and judged matters into both weak, both
///    strong and the rest;
/// 2. raising the lower floor never makes a planet **less** weak, and
///    raising the upper never makes one **more** strong;
/// 3. no upper floor weakens a **dignified** planet;
/// 4. where the two floors are equal there is **no middle**, so the
///    lower table and the upper meet exactly — the one check that ties
///    the two tables to each other;
/// 5. no yoga needing a weak pair **held** in more matters than had one.
fn floors_hold(kinds: &Kinds) -> Result<(), String> {
    let judged = kinds.matters - kinds.same_lord;
    let weak_at = position_of(&WEAK_FLOORS, YOGA_WEAK_BELOW, "lower")?;
    let strong_at = position_of(&STRONG_FLOORS, YOGA_STRONG_FROM, "upper")?;
    let weak = kinds.weak_side.get(weak_at).copied().unwrap_or_default();
    let strong = kinds
        .strong_side
        .get(strong_at)
        .copied()
        .unwrap_or_default();

    let readings = weak.readings + kinds.middling + strong.readings;
    if readings != kinds.strengths_read {
        return Err(format!(
            "{} weak + {} middling + {} strong readings is {readings}, and {} were read",
            weak.readings, kinds.middling, strong.readings, kinds.strengths_read,
        ));
    }
    let matters = weak.both + kinds.mixed + strong.both;
    if matters != judged {
        return Err(format!(
            "{} both weak + {} mixed + {} both strong is {matters}, and {judged} matters were judged",
            weak.both, kinds.mixed, strong.both,
        ));
    }
    for (low, high) in kinds.weak_side.iter().zip(kinds.weak_side.iter().skip(1)) {
        if high.readings < low.readings {
            return Err(String::from(
                "raising the lower floor made a planet less weak",
            ));
        }
    }
    for (low, high) in kinds
        .strong_side
        .iter()
        .zip(kinds.strong_side.iter().skip(1))
    {
        if high.readings > low.readings {
            return Err(String::from(
                "raising the upper floor made a planet more strong",
            ));
        }
    }
    if let Some(at) = kinds
        .strong_side
        .iter()
        .find(|at| at.readings < kinds.dignified)
    {
        return Err(format!(
            "an upper floor left {} readings strong, and {} are dignified, which none can weaken",
            at.readings, kinds.dignified,
        ));
    }
    let lower = kinds.weak_side.get(weak_at).copied().unwrap_or_default();
    let equal = position_of(&STRONG_FLOORS, YOGA_WEAK_BELOW, "upper")?;
    let upper = kinds.strong_side.get(equal).copied().unwrap_or_default();
    if lower.readings + upper.readings != kinds.strengths_read {
        return Err(format!(
            "with both floors at {YOGA_WEAK_BELOW}, {} weak + {} strong readings leave a middle \
             of {}, where there can be none",
            lower.readings,
            upper.readings,
            kinds
                .strengths_read
                .abs_diff(lower.readings + upper.readings),
        ));
    }
    ceilings_hold(kinds, weak.both, strong.both)?;
    kuttha_holds(kinds, strong.both)
}

/// What Kuttha's counts must satisfy: the strong pairs it read are the
/// floors' own, the lords' favours agree with the yoga, the waxing Moon
/// only removes, and no strong pair is turned away for want of strength.
fn kuttha_holds(kinds: &Kinds, strong_pairs: usize) -> Result<(), String> {
    let counts = &kinds.kuttha;
    let held = held_in(kinds, YearYoga::Kuttha);
    if counts.strong_pairs != strong_pairs {
        return Err(format!(
            "Kuttha read {} strong pairs, and the default floors found {strong_pairs}",
            counts.strong_pairs
        ));
    }
    if counts.favoured != held {
        return Err(format!(
            "both lords were favoured in {} matters, and Kuttha held in {held}",
            counts.favoured
        ));
    }
    if counts.waxing_only != 0 || counts.waxing > held {
        return Err(format!(
            "the waxing Moon held Kuttha in {} matters, {} of them where the default did not;              a reading that only removes a benefic cannot add a Kuttha",
            counts.waxing, counts.waxing_only
        ));
    }
    if counts.turned_away.first().copied() != Some(0) {
        return Err(String::from(
            "a strong pair was turned away as not powerful, which it cannot be",
        ));
    }
    let mut away = counts.turned_away.iter().zip(&counts.alone);
    if let Some((turned, alone)) = away.find(|(turned, alone)| alone > turned) {
        return Err(format!(
            "a clause turned away {alone} matters alone and only {turned} in all"
        ));
    }
    Ok(())
}

/// How many matters a yoga held in.
fn held_in(kinds: &Kinds, yoga: YearYoga) -> usize {
    YearYoga::ALL
        .iter()
        .position(|one| *one == yoga)
        .and_then(|at| kinds.yogas.get(at).copied())
        .unwrap_or_default()
}

/// No yoga holds in more matters than its definition allows: a judgement
/// **upon** an Ithasala no more often than an Ithasala stands, and one
/// needing a **weak pair**, or a **strong** one, no more often than there
/// was one.
///
/// Both groupings are read from `YearYoga` itself rather than kept as a
/// list here, so a yoga built later is held to its ceiling without this
/// pass being edited.
fn ceilings_hold(kinds: &Kinds, weak_pairs: usize, strong_pairs: usize) -> Result<(), String> {
    let ithasalas = held_in(kinds, YearYoga::Ithasala);
    for yoga in YearYoga::ALL {
        let held = held_in(kinds, yoga);
        if yoga.is_chart_fact() && held % 12 != 0 {
            return Err(format!(
                "{yoga:?} is a fact about a chart and held in {held} matters, which is not \
                 all twelve of some charts and none of the rest"
            ));
        }
        if yoga.judges_an_ithasala() && held > ithasalas {
            return Err(format!(
                "{yoga:?} held in {held} matters, and an Ithasala stood in only {ithasalas}"
            ));
        }
        if yoga.needs_no_aspect() && held > kinds.unaspecting {
            return Err(format!(
                "{yoga:?} held in {held} matters, and only {} had a pair that did not aspect",
                kinds.unaspecting
            ));
        }
        if yoga.needs_a_weak_pair() && held > weak_pairs {
            return Err(format!(
                "{yoga:?} held in {held} matters, and only {weak_pairs} had a weak pair for it to need"
            ));
        }
        if yoga.needs_a_strong_pair() && held > strong_pairs {
            return Err(format!(
                "{yoga:?} held in {held} matters, and only {strong_pairs} had a strong pair for it to need"
            ));
        }
    }
    Ok(())
}

/// Counts which affliction spoilt each Rudda of one matter, from the
/// afflictions the module itself reported with it.
fn what_spoiled(asked: &YearYogas, kinds: &mut Kinds) {
    for rudda in asked.held.iter().filter(|one| one.yoga == YearYoga::Rudda) {
        let Some(pair) = rudda.afflictions else {
            continue;
        };
        let clauses = pair.map(teistro::tajika::Affliction::clauses);
        let width = clauses.iter().map(|one| one.len()).max().unwrap_or(0);
        if kinds.rudda_clauses.len() < width {
            kinds.rudda_clauses.resize(width, 0);
        }
        for (at, slot) in kinds.rudda_clauses.iter_mut().enumerate() {
            let held = clauses
                .iter()
                .any(|one| one.get(at).is_some_and(|(_, holds)| *holds));
            if held {
                *slot += 1;
            }
        }
        // Counted against the clause list rather than by naming the other
        // four, so a clause the module gains later is not silently read
        // as absent.
        let malefic_only = pair.iter().all(|one| {
            let holding = one.clauses().iter().filter(|(_, holds)| *holds).count();
            holding == usize::from(one.under_malefic)
        });
        if malefic_only {
            kinds.rudda_malefic_alone += 1;
        }
    }
}

fn the_kinds(out: &mut String, kinds: &Kinds) {
    let share = |part: usize| share(part, kinds.aspecting);
    let _ = write!(
        out,
        "\n## 9. The four kinds, over the recorded years\n\n\
         The source's Table X-3 gives the Ithasala **three** kinds and sets \
         Ishrafa a degree away from them. This sorts every pair of every \
         annual chart of every recorded birth into them — {} charts, {} \
         pairs, of which {} stand in signs that aspect at all — through \
         `sdk.chart().drishtis`, so what is counted is what the module \
         answers.\n\n\
         | kind | what puts a pair there | pairs | of those that aspect |\n\
         |---|---|---|---|\n\
         | **Vartamana** | behind by a degree or more, inside the orb | {} | {} |\n\
         | **Poorna** | behind by less than a degree | {} | {} |\n\
         | **Bhavishyat** | outside the orb, reaching from a sign's end | {} | {} |\n\
         | **Ishrafa** | past by a degree or more, inside the orb | {} | {} |\n\
         | *the contested band* | past by less than a degree | **{}** | **{}** |\n\n\
         ### What turns on the last row\n\n\
         The last row is the one thing the source's two accounts do not \
         settle (crux C112), and the count is why it is carried as a \
         reading rather than decided quietly. Under the chapter's prose \
         those {} pairs are **Ishrafa**, generally unfavourable and drawing \
         apart. Under Table X-3 read so that its rows interlock they are \
         **Poorna**, the most fulfilled thing a pair can be. Under the \
         table read at its narrowest they are nothing at all. One band, \
         three answers, and the three are not near each other.\n\n\
         Two things about the size of it. It is {} of every pair that \
         aspects — not a rounding margin, and about a twelfth of every \
         Ishrafa. And it is almost exactly the size of the **Poorna the \
         table states outright** beside it, {} against {}: the two sit \
         symmetrically either side of an exact aspect, which is the \
         argument for reading Poorna as covering both. A reading on which \
         one side of exactness is immediate fulfilment and the other side \
         is nothing would have to explain the asymmetry, and the book does \
         not.\n\n\
         `SubDegree` carries all three and defaults to Poorna, which is \
         the only reading under which the degree the table prints does any \
         work at all. `Between::disputed` marks the pairs, so a reader can \
         say which judgements are contested.\n\n\
         ### What the sweep could not reach\n\n\
         {}\n",
        count(kinds.charts),
        count(kinds.pairs),
        count(kinds.aspecting),
        count(kinds.vartamana),
        share(kinds.vartamana),
        count(kinds.poorna_stated),
        share(kinds.poorna_stated),
        count(kinds.bhavishyat),
        share(kinds.bhavishyat),
        count(kinds.ishrafa),
        share(kinds.ishrafa),
        count(kinds.disputed),
        share(kinds.disputed),
        count(kinds.disputed),
        share(kinds.disputed),
        count(kinds.poorna_stated),
        count(kinds.disputed),
        refusals(kinds),
    );
}

/// What the sweep asked for and did not get, named.
///
/// Fifty-five births times [`SWEEP_YEARS`] is a fixed number of years
/// asked for; anything short of it is stated with the births it belongs
/// to and the SDK's own words for why, because a pass that reported only
/// what it managed would get greener as the corpus got harder.
fn refusals(kinds: &Kinds) -> String {
    let named = |names: &[String]| {
        let quoted: Vec<String> = names.iter().map(|name| format!("`{name}`")).collect();
        listed(&quoted)
    };
    let mut out = format!(
        "Every recorded birth was asked for {} years, which is **{}** \
         years over the {} of them, and {} charts were read. The whole \
         of the difference is accounted for below, because a sweep that \
         reported only what it managed would get greener as the corpus \
         got harder.\n\n",
        SWEEP_YEARS,
        count(kinds.asked),
        count(kinds.asked / usize::from(SWEEP_YEARS).max(1)),
        count(kinds.charts),
    );
    if kinds.cut_short.is_empty() {
        out.push_str("Every birth's search reached its fortieth year. ");
    } else {
        let _ = write!(
            out,
            "**{}** never returned from the search at all, belonging to \
             {}: {}. The built-in ephemeris's span runs out before those \
             births reach their fortieth year, and the SDK answers the \
             years it covers rather than refusing the whole request. ",
            plural(kinds.asked - kinds.returned, "year"),
            plural(kinds.cut_short.len(), "birth"),
            named(&kinds.cut_short),
        );
    }
    if kinds.refused == 0 {
        out.push_str("Every return the search did produce was founded and read.");
    } else {
        let _ = write!(
            out,
            "A further **{}** returned but could not be **founded**, \
             belonging to {}: {}. The SDK refuses rather than inventing \
             a day — *{}* — because a birth above the polar circle in \
             its own summer has no sunrise to divide a day by, and the \
             hora and ghati a chart is built on are measured from one. \
             That is a documented bound of the corpus \
             (`05-testing/01-golden-vectors.md`, note 13) and not a \
             fault of the aspects.",
            plural(kinds.refused, "year"),
            plural(kinds.refused_births.len(), "birth"),
            named(&kinds.refused_births),
            kinds.refusal,
        );
    }
    out
}

fn the_aspects(out: &mut String, geo: &Worked) {
    let pair = &geo.sun_and_mars;
    let _ = write!(
        out,
        "## 8. The aspects, and the source's worked Ithasala\n\n\
         A pair of planets is governed by the **mean** of their two \
         deeptamshas, and is coming together — **Ithasala** — when the \
         faster of them is behind the slower. Behind is **degrees within \
         the sign**, the completed signs deleted, which is the source's own \
         instruction and the opposite of what a longitude would say.\n\n\
         The source's Table X-3 gives that coming-together **three \
         kinds**, which §9 counts over the whole corpus; this pair is \
         the **Vartamana**, the present one, because the Sun is behind \
         Mars by more than the single degree that would make it already \
         fulfilled.\n\n\
         Its worked pair is the Sun at Leo 3°50′ and Mars at Scorpio 7°42′, \
         three whole signs further on. Read from the chart the SDK \
         founded:\n\n\
         | | measured | the source |\n\
         |---|---|---|\n\
         | the faster of the two | {:?} | the Sun |\n\
         | their orb, the mean of 15° and 8° | {:.2}° | 11°30′ |\n\
         | apart, within their signs | {:.2}° | 3°52′ |\n\
         | what they make | {} | Ithasala |\n\n\
         Of that chart's twenty-one pairs, **{}** make a yoga and the rest \
         make none — most of them because they stand in the neutral houses, \
         where no closeness is an aspect. Only the ones that make something \
         cross the boundary.\n",
        pair.faster,
        pair.orb_deg,
        pair.apart_deg,
        pair.yoga
            .map_or_else(|| String::from("none"), |yoga| format!("{yoga:?}")),
        geo.yogas,
    );
}

/// The sixteen, over every matter of every recorded year.
fn the_sixteen(out: &mut String, kinds: &Kinds) {
    let built: Vec<&YearYoga> = YearYoga::ALL.iter().filter(|one| one.is_built()).collect();
    let rows: String = YearYoga::ALL
        .iter()
        .enumerate()
        .map(|(at, yoga)| {
            let held = kinds.yogas.get(at).copied().unwrap_or_default();
            match yoga.awaiting() {
                None => format!(
                    "| **{yoga:?}** | built | {} | {} |\n",
                    count(held),
                    share(held, kinds.matters)
                ),
                Some(why) => format!("| {yoga:?} | *awaiting* | -- | {why} |\n"),
            }
        })
        .collect();
    let _ = write!(
        out,
        "\n## 10. The sixteen yogas, and the matters they answer\n\n\
         Fourteen of the sixteen are not facts about a chart. They are \
         judgements about a **pair** — the *lagnesha*, the lord of the \
         annual lagna, and the *karyesha*, the lord of the house the \
         matter asked about belongs to — so the same year answers \
         differently for each of the twelve houses. Every chart above is \
         asked all twelve, which is **{}** questions. Ikabala and Induvara \
         are the two exceptions: facts about a chart, so each holds in all \
         twelve of a chart's matters or in none — **{}** charts and **{}**, \
         a divisibility the pass checks.\n\n\
         | yoga | | held | of the matters asked |\n\
         |---|---|---:|---|\n\
         {}\n\n\
         **{} of the sixteen are built** and the other {} are listed at \
         every call rather than left out of the answer, because *did not \
         hold* and *cannot be told* are different statements. \
         `YearYogas::holds` answers `None` for those {}, never `false`.\n\n\
         ### The first house is never a pair\n\n\
         **{}** of the {} matters — {} — have one planet for both lords, \
         so there is no pair to judge. That is not an edge case that \
         crept in: the **first** house is the lagna itself, so its lord \
         is the lagnesha by definition, and a question about the \
         native's own self can never be one of these fourteen \
         judgements. One further house is like it under a lagna ruled by \
         one of the five that rule two signs, and none is under Cancer \
         or Leo, where the luminaries rule one each. The answer reports \
         it as `same_lord` rather than returning an empty list that \
         would read as *nothing holds*.\n\n\
         The count decomposes, and `cargo xtask muntha` **fails** if it \
         ever stops decomposing, because a printed figure nobody can \
         check is the part of a generated page that rots. Every one of \
         the {} charts contributes its first house, and every chart but \
         the {} whose lagna a **luminary** rules contributes one more, \
         since the Sun rules Leo alone and the Moon Cancer alone: {} + \
         ({} − {}) = **{}**.\n",
        count(kinds.matters),
        count(held_in(kinds, YearYoga::Ikabala) / 12),
        count(held_in(kinds, YearYoga::Induvara) / 12),
        rows.trim_end(),
        capitalised(&spelled(built.len())),
        spelled(YearYoga::ALL.len() - built.len()),
        spelled(YearYoga::ALL.len() - built.len()),
        count(kinds.same_lord),
        count(kinds.matters),
        share(kinds.same_lord, kinds.matters),
        count(kinds.charts),
        count(kinds.luminary_lagna),
        count(kinds.charts),
        count(kinds.charts),
        count(kinds.luminary_lagna),
        count(2 * kinds.charts - kinds.luminary_lagna),
    );
    why_khallasara_is_rare(out, kinds);
}

/// Why an unqualified Moon is so nearly unreachable, clause by clause.
///
/// Split out of [`the_sixteen`] because it is a finding of its own and
/// not a paragraph of that one: the definition it measures is the
/// source's, the rarity is structural rather than a fact about this
/// corpus, and the reasoning belongs beside the count it explains.
fn why_khallasara_is_rare(out: &mut String, kinds: &Kinds) {
    let _ = write!(
        out,
        "### Why Khallasara is so rare\n\n\
         Khallasara and Gairi-Kamboola both need an **unqualified** Moon, \
         which the source defines outright: neither exalted nor \
         debilitated, nor aspected or associated, nor in its own Hudda, \
         Drekkana or Navamsha. Every clause must be false at once, and \
         over the {} annual charts the Moon managed it **{}**, one chart \
         in {}. The \
         clause that does the disqualifying is not the interesting one \
         to guess at, so it is counted:\n\n\
         | clause | charts |\n\
         |---|---:|\n\
         {}\n\n\
         **The last row is a zero that had to be explained rather than \
         printed.** The Hudda is the Egyptian terms, which divide every \
         sign among Mars, Mercury, Jupiter, Venus and Saturn and give \
         the luminaries nothing — so for the one planet this definition \
         is ever applied to, that clause is **vacuous**. It is kept in \
         the code because the source states it and a reader comparing \
         the two should find all six.\n\n\
         **The first row is almost the whole of it**, and it is \
         structural rather than accidental: Tajika counts **eight of the \
         twelve** sign relations as an aspect — only the 2nd, 6th, 8th \
         and 12th are nothing at all — so a Moon that nothing aspects \
         needs all six of the others inside those four houses at once. \
         The source's own worked chart cannot do it at any degree of the \
         Moon's circle, because three of its planets share Leo and two \
         more sit in the signs either side, a spacing no single sign is \
         neutral to. That is a fact about the definition and not about \
         this corpus, and it is why C115 asks whether *aspected* here is \
         narrower than *by any of the seven*.\n",
        count(kinds.charts),
        times(kinds.moon_unqualified),
        count(
            kinds
                .charts
                .checked_div(kinds.moon_unqualified)
                .unwrap_or(kinds.charts),
        ),
        clause_rows(kinds),
    );
}

/// §12: what spoils an Ithasala, and what the corpus says the two
/// readings the source leaves open (cruxes C118 and C119) can move.
fn what_spoils_an_ithasala(out: &mut String, sdk: &Context, kinds: &Kinds) {
    let ithasalas = held_in(kinds, YearYoga::Ithasala);
    let ruddas = held_in(kinds, YearYoga::Rudda);
    // The clause names are the type's own, read off an affliction with
    // none, so a clause renamed or added there is renamed or added here.
    let none = teistro::tajika::Affliction {
        graha: Graha::Moon,
        retrograde: false,
        combust: false,
        debilitated: false,
        trika: false,
        under_malefic: false,
    };
    let mut rows: Vec<(&str, usize)> = none
        .clauses()
        .into_iter()
        .map(|(name, _)| name)
        .zip(
            kinds
                .rudda_clauses
                .iter()
                .copied()
                .chain(std::iter::repeat(0)),
        )
        .collect();
    // Commonest first, the source's order among equals: which clause does
    // the spoiling reads off the top.
    rows.sort_by_key(|row| std::cmp::Reverse(row.1));
    let rows = rows
        .into_iter()
        .map(|(name, held)| format!("| {name} | {} | {} |", count(held), share(held, ruddas)))
        .collect::<Vec<_>>()
        .join("\n");
    let _ = write!(
        out,
        "\n## 12. What spoils an Ithasala\n\n\
         Rudda is the Ithasala spoilt: one where either of the pair is \
         \"retrograde, combust, debilitated, in the 6th, 8th or 12th, or \
         under malefic influence\". Retrograde and combustion are not in a \
         chart's longitudes, so they are read from its graha states — over \
         the corpus's {} readings, **{}** are retrograde and **{}** combust, \
         under the `{}` combustion table the profile sets.\n\n\
         Rudda held in **{}** of the {} matters in which an Ithasala stood \
         ({}). Each clause, counted in the Ruddas where it held of either \
         lord — so a Rudda with two afflictions is counted twice:\n\n\
         | clause | Ruddas | of them |\n\
         |---|---:|---|\n\
         {rows}\n\n\
         ### The reach of the two readings left open\n\n\
         *Under malefic influence* is read as Manau reads it — joined, or \
         aspected inimically, by Mars or Saturn — with the pair's own \
         malefic counting against its partner (crux C118). **{}** Ruddas \
         ({}) held on that clause and nothing else. That bounds every \
         **narrower** reading at once: excluding the partner, or any \
         stricter sense of influence, can remove those and no other. A \
         **wider** one — any aspect at all — could only add Ruddas, and at \
         most the **{}** Ithasalas not spoilt now.\n\n\
         **Read literally, Rudda spoils almost every Ithasala**, and no \
         reading of that one clause changes it: the narrowest would still \
         leave **{}** of the {} spoilt ({}). The breadth is the list \
         itself — any one of five common afflictions, on either of two \
         planets — which is the mirror of *unqualified* (crux C115), read \
         so strictly that Khallasara almost never holds. It is recorded \
         rather than corrected, because no text in reach narrows the list; \
         every Rudda carries both lords' clauses, so a reader who holds a \
         narrower reading can apply it without the yoga being \
         rewritten.\n\n\
         Duhphali-kuttha held in **{}**, and Durapha in **{}**. Durapha's \
         list is read as alternatives each lord must meet one of (crux \
         C119); whatever the reading, it needs a weak pair first, and §11 \
         puts that ceiling at **{}**. `cargo xtask muntha` fails if either \
         ceiling is ever exceeded: every judgement upon an Ithasala is held \
         under the Ithasala's own count, and every yoga needing a weak pair \
         under the weak pairs', each read from `YearYoga` itself.\n",
        count(kinds.strengths_read),
        count(kinds.retrograde),
        count(kinds.combust),
        sdk.settings().state.combustion_orbs,
        count(ruddas),
        count(ithasalas),
        share(ruddas, ithasalas),
        count(kinds.rudda_malefic_alone),
        share(kinds.rudda_malefic_alone, ruddas),
        count(ithasalas.saturating_sub(ruddas)),
        count(ruddas.saturating_sub(kinds.rudda_malefic_alone)),
        count(ithasalas),
        share(ruddas.saturating_sub(kinds.rudda_malefic_alone), ithasalas),
        count(held_in(kinds, YearYoga::DuhphaliKuttha)),
        count(held_in(kinds, YearYoga::Durapha)),
        count(
            position_of(&WEAK_FLOORS, YOGA_WEAK_BELOW, "lower")
                .ok()
                .and_then(|at| kinds.weak_side.get(at))
                .map_or(0, |at| at.both)
        ),
    );
}

/// §11: where weak ends and strong begins, which the source says only
/// for the year lord (crux C116).
fn the_floors(out: &mut String, kinds: &Kinds) -> Result<(), String> {
    let weak_at = position_of(&WEAK_FLOORS, YOGA_WEAK_BELOW, "lower")?;
    let strong_at = position_of(&STRONG_FLOORS, YOGA_STRONG_FROM, "upper")?;
    let weak = kinds.weak_side.get(weak_at).copied().unwrap_or_default();
    let strong = kinds
        .strong_side
        .get(strong_at)
        .copied()
        .unwrap_or_default();
    let bands = kinds
        .vishwa_bands
        .iter()
        .enumerate()
        .map(|(band, readings)| {
            let low = band * 2;
            let close = if band + 1 == kinds.vishwa_bands.len() {
                ']'
            } else {
                ')'
            };
            format!(
                "| [{low}, {}{close} | {} | {} |",
                low + 2,
                count(*readings),
                share(*readings, kinds.strengths_read)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let _ = write!(
        out,
        "\n## 11. Strong and weak, and the floors between them\n\n\
         Five of the six strength yogas turn on whether a planet is \
         **strong** or **weak** — Rudda alone does not — and Charak never \
         says where either begins. He gives a figure once, for the \
         office-bearers when he chooses the year lord: below five units of \
         Vishwa bala, the Muntha lord takes the year instead. A second book \
         grades the whole scale — under five *Nirbali*, strengthless; five \
         to ten *Madhya*, middling; ten to fifteen *Poorna*, fully strong; \
         above fifteen *Parakrami* — and the two meet at five. So the yogas' \
         *weak* is read as *Nirbali* and their *strong* as *Poorna* or \
         better, which leaves a **middling** band between that is neither. \
         Both floors are `YogaRules` fields, and this section measures what \
         each costs (crux C116).\n\n\
         Strength is a disjunction — \"exalted, in its own house or \
         otherwise strong\" — so a **dignified** planet is strong under any \
         floor at all. Of the {} readings, seven to each of the {} charts, \
         **{}** ({}) are dignified and beyond either floor's reach.\n\n\
         ### Where the seven stand\n\n\
         | Vishwa bala | readings | |\n\
         |---|---:|---|\n\
         {bands}\n\n\
         The weakest reading anywhere in the corpus is **{}**, on a scale \
         of twenty. Under the default floors, **{}** readings ({}) are \
         weak, **{}** ({}) middling and **{}** ({}) strong.\n",
        count(kinds.strengths_read),
        count(kinds.charts),
        count(kinds.dignified),
        share(kinds.dignified, kinds.strengths_read),
        kinds
            .weakest
            .map_or_else(|| String::from("--"), |low| low.to_string()),
        count(weak.readings),
        share(weak.readings, kinds.strengths_read),
        count(kinds.middling),
        share(kinds.middling, kinds.strengths_read),
        count(strong.readings),
        share(strong.readings, kinds.strengths_read),
    );
    the_two_floors(out, kinds, weak_at, strong_at);
    what_the_floors_cost(out, kinds, weak, strong);
    Ok(())
}

/// One table per floor, each holding the other at its default.
fn the_two_floors(out: &mut String, kinds: &Kinds, weak_at: usize, strong_at: usize) {
    let judged = kinds.matters - kinds.same_lord;
    let rows = |list: &[i64], side: &[AtFloor], default: usize| {
        list.iter()
            .zip(side)
            .enumerate()
            .map(|(which, (units, at))| {
                let floor = if which == default {
                    format!("**{units}** (default)")
                } else {
                    units.to_string()
                };
                format!(
                    "| {floor} | {} | {} | {} |",
                    share(at.readings, kinds.strengths_read),
                    count(at.both),
                    share(at.both, judged),
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let _ = write!(
        out,
        "\nOf the {} matters asked, **{}** have two distinct lords to judge. \
         The two floors are **separable** — whether a planet is weak turns \
         only on the lower and whether it is strong only on the upper, so \
         long as they do not cross — so each table moves one and holds the \
         other at its default.\n\n\
         ### The lower floor: where weak ends\n\n\
         | below | readings weak | both lords weak | of the judged |\n\
         |---:|---:|---:|---:|\n\
         {}\n\n\
         ### The upper floor: where strong begins\n\n\
         | from | readings strong | both lords strong | of the judged |\n\
         |---:|---:|---:|---:|\n\
         {}\n",
        count(kinds.matters),
        count(judged),
        rows(&WEAK_FLOORS, &kinds.weak_side, weak_at),
        rows(&STRONG_FLOORS, &kinds.strong_side, strong_at),
    );
}

/// What the default floors cost the yogas that read them, and the
/// identities the pass holds both tables to.
fn what_the_floors_cost(out: &mut String, kinds: &Kinds, weak: AtFloor, strong: AtFloor) {
    let judged = kinds.matters - kinds.same_lord;
    let held = YearYoga::ALL
        .iter()
        .position(|one| *one == YearYoga::DutthotthaDavira)
        .and_then(|at| kinds.yogas.get(at).copied())
        .unwrap_or_default();
    let _ = write!(
        out,
        "\n**At the default floors, both lords are weak in {} of the {} \
         judged matters ({}), both strong in {} ({}), and the other {} are \
         mixed.** The first is a **ceiling**, not a count of anything that \
         held: Dutthottha-Davira and Durapha both require a weak pair before \
         asking anything else, so neither can hold in more matters than it \
         allows. Dutthottha-Davira held in **{}**. Kuttha, which wants both \
         strong, has the second to work with.\n\n\
         Five things hold of both tables, and `cargo xtask muntha` **fails** \
         if any stops holding, because each is a fact about `Strength` and \
         not about this corpus: under the default floors the readings \
         **partition** into weak, middling and strong, and the judged \
         matters into both weak, both strong and mixed, each part counted \
         on its own rather than by difference; raising the lower floor never \
         makes a planet less weak, nor raising the upper one more strong; no \
         upper floor falls below the {} dignified readings; where the two \
         floors are **equal** there is no middle, so the lower table's row \
         at five and the upper table's meet exactly — the one check that \
         ties the two tables to each other; and no yoga that needs a weak \
         pair holds in more matters than had one.\n",
        count(weak.both),
        count(judged),
        share(weak.both, judged),
        count(strong.both),
        share(strong.both, judged),
        count(kinds.mixed),
        count(held),
        count(kinds.dignified),
    );
}

/// One row per clause of the source's definition, named by the
/// definition itself and ordered as it states them.
///
/// The names come from `Qualification::clauses`, so a clause renamed or
/// reordered in the type moves this table with it rather than leaving a
/// row labelled with the wrong count.
fn clause_rows(kinds: &Kinds) -> String {
    let empty = Qualification {
        graha: Graha::Moon,
        exalted: false,
        debilitated: false,
        aspected: false,
        own_hudda: false,
        own_drekkana: false,
        own_navamsha: false,
    };
    let mut rows: Vec<(usize, &'static str, usize)> = empty
        .clauses()
        .into_iter()
        .enumerate()
        .map(|(at, (name, _))| (at, name, kinds.moon_clauses.get(at).copied().unwrap_or(0)))
        .collect();
    // Commonest first: the point of the table is which clause does the
    // disqualifying, and that reads off the top rather than out of the
    // definition's own order.
    rows.sort_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(&b.0)));
    rows.into_iter()
        .map(|(_, name, held)| {
            // A zero that is structural says so, because a bare zero and
            // "this can never happen" are different facts.
            let note = if held == 0 && name.contains("Hudda") {
                " — *and never can be*"
            } else {
                ""
            };
            format!("| {name} | {}{note} |", count(held))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// A count as a share of a total, to one place; `--` where the total is
/// nothing, because a share of no questions is not zero.
fn share(part: usize, of: usize) -> String {
    if of == 0 {
        return String::from("--");
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "counts of a few tens of thousands, printed to one place"
    )]
    let percent = part as f64 * 100.0 / of as f64;
    format!("{percent:.1}%")
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

/// §13: the two yogas that ask where a planet at a sign's end will stand
/// in the next, and the steps between the ground they need and their
/// holding (crux C120, C121).
fn what_happens_next(out: &mut String, kinds: &Kinds) {
    let tambira = held_in(kinds, YearYoga::Tambira);
    let gairi = held_in(kinds, YearYoga::GairiKamboola);
    let ithasalas = held_in(kinds, YearYoga::Ithasala);
    let _ = write!(
        out,
        "\n## 13. What happens next: Gairi-Kamboola and Tambira\n\n\
         Two of the sixteen ask where a planet at a sign's end will stand \
         **on entering the next**. The module answers by moving that one \
         planet to the next sign's first degree, the other six where they \
         are, and asking the same Ithasala question of the sky that \
         leaves (crux C120). The source's worked Gairi-Kamboola, Chart \
         X-17, comes out as printed under it. Both counts sit under \
         chains of ceilings, each step a subset of the one before, and \
         `cargo xtask muntha` fails if any step exceeds the one above it.\n\n\
         | step | Gairi-Kamboola | step | Tambira |\n\
         |---|---:|---|---:|\n\
         | an Ithasala | {} | the pair do not aspect | {} |\n\
         | the Moon, not one of the pair, at a sign's end | {} | the karyesha at a sign's end | {} |\n\
         | that Moon unqualified | {} | and not retrograde | {} |\n\
         | **held** | **{}** | **held** | **{}** |\n\n\
         **The step from the second row to the third is the source's \
         *unqualified*,** which §10 found the Moon meeting in {} of {} \
         charts. Whatever Gairi-Kamboola loses there is C115's to move, \
         not this yoga's: narrowing that one reading is what would widen \
         both it and Khallasara. Where it holds, Khallasara \
         does not, though every printed clause of Khallasara may: the \
         source's own comment excludes a Moon at a sign's end from it, \
         because that Moon completes the Ithasala rather than standing \
         apart (crux C121).\n\n\
         **Tambira's own readings move it.** Letting either lord be the \
         one at a sign's end (the source's \"some authorities\", \
         `TambiraMover::EitherLord`) holds it in **{}** matters to the \
         definition's {}. A retrograde karyesha is going back, not on, \
         so it enters nothing, and a call without the chart's states \
         cannot answer for Tambira at all.\n",
        count(ithasalas),
        count(kinds.unaspecting),
        count(kinds.moon_at_end),
        count(kinds.karyesha_at_end),
        count(kinds.moon_at_end_unqualified),
        count(kinds.karyesha_moving_on),
        count(gairi),
        count(tambira),
        count(kinds.moon_unqualified),
        count(kinds.charts),
        count(kinds.tambira_either),
        count(tambira),
    );
}

/// Reads one chart's forty-one sahams under the source's rules and each
/// rival, and counts where the rivals move them.
///
/// Through `sdk.chart().sahams_with_rules`, the path a consumer calls, so
/// what is counted is the module that answers.
fn count_sahams(
    sdk: &Context,
    annual: &Document,
    name: &str,
    counts: &mut SahamCounts,
) -> Result<(), String> {
    let read = |rules: SahamRules| {
        sdk.chart()
            .sahams_with_rules(annual, &Saham::ALL, rules)
            .map_err(|why| format!("{name}: its sahams: {why}"))
    };
    let source = SahamRules::default();
    let base = read(source)?;
    let never = read(SahamRules {
        add_sign: AddSign::Never,
        ..source
    })?;
    let signs = read(SahamRules {
        add_sign: AddSign::Signs,
        ..source
    })?;
    let equal = read(SahamRules {
        houses: HousePoints::Equal,
        ..source
    })?;
    let roga = read(SahamRules {
        roga: RogaReading::Saturn,
        ..source
    })?;
    if base.by_day {
        counts.by_day += 1;
    }
    let moved = |rival: &SahamReading, column: &mut Vec<usize>, by_sign: bool| {
        for ((slot, one), other) in column.iter_mut().zip(&base.points).zip(&rival.points) {
            let differs = if by_sign {
                one.point.sign != other.point.sign
            } else {
                // Round the circle, so 359.999… and 0 are the same place.
                let apart = (one.point.longitude_deg - other.point.longitude_deg + 540.0)
                    .rem_euclid(360.0)
                    - 180.0;
                apart.abs() > 1e-9
            };
            if differs {
                *slot += 1;
            }
        }
    };
    for (slot, asked) in counts.added.iter_mut().zip(&base.points) {
        if asked.point.added_sign {
            *slot += 1;
        }
    }
    moved(&never, &mut counts.never_moved, false);
    moved(&signs, &mut counts.signs_moved, true);
    moved(&equal, &mut counts.equal_moved, true);
    let at = Saham::Roga.index();
    if base.points.get(at).map(|p| p.point.sign) != roga.points.get(at).map(|p| p.point.sign) {
        counts.roga_moved += 1;
    }
    Ok(())
}

/// Whether any factor of a saham, by day or by night, is of a kind.
fn reads(saham: Saham, kind: fn(SahamTerm) -> bool) -> bool {
    let formula = saham.formula(SahamRules::default());
    [formula.day, formula.night]
        .iter()
        .flat_map(|triple| [triple.a, triple.b, triple.c])
        .any(kind)
}

/// Whether a saham reads a house a reading can move.
///
/// The first house's point is the lagna under every reading, so a saham
/// reading only its lord — Samarthya and Manmatha, "the lord of the
/// ascendant" — reads the lagna's lord and cannot move with the houses.
fn reads_a_house(saham: Saham) -> bool {
    reads(saham, |term| match term {
        SahamTerm::House(house) | SahamTerm::HouseLord(house) => house.get() != 1,
        _ => false,
    })
}

/// Whether a saham reads another saham, and so inherits its sign.
fn reads_a_saham(saham: Saham) -> bool {
    reads(saham, |term| matches!(term, SahamTerm::Saham(_)))
}

/// The identities the saham counts must satisfy, refused when they do not.
fn sahams_hold(counts: &SahamCounts) -> Result<(), String> {
    for saham in Saham::ALL {
        let at = saham.index();
        let (added, never) = (counts.added[at], counts.never_moved[at]);
        // One that reads another moves with it too, so only its own sign
        // is a count the identity can hold.
        if !reads_a_saham(saham) && added != never {
            return Err(format!(
                "{saham:?} took the added sign {added} times and moved {never} times \
                 when it was never added: the two must be one count"
            ));
        }
        if !reads_a_house(saham) && counts.equal_moved[at] != 0 {
            return Err(format!(
                "{saham:?} reads no house and moved {} times under equal houses",
                counts.equal_moved[at]
            ));
        }
    }
    for (one, other) in [
        (Saham::Vidya, Saham::Guru),
        (Saham::Raja, Saham::Pitri),
        (Saham::Kshama, Saham::Kali),
    ] {
        let row = |saham: Saham| {
            let at = saham.index();
            (
                counts.added[at],
                counts.signs_moved[at],
                counts.equal_moved[at],
            )
        };
        if row(one) != row(other) {
            return Err(format!(
                "{one:?} shares {other:?}'s formula and not its counts"
            ));
        }
    }
    Ok(())
}

/// §14: the sahams, and what each reading the tradition divides over
/// moves.
fn the_sahams(out: &mut String, kinds: &Kinds) {
    let counts = &kinds.sahams;
    let charts = kinds.charts;
    let mut rows = String::new();
    for saham in Saham::ALL {
        let at = saham.index();
        let equal = if reads_a_house(saham) {
            share(counts.equal_moved[at], charts)
        } else {
            String::from("no house past the first")
        };
        let _ = writeln!(
            rows,
            "| {} {saham:?} | {} | {} | {equal} |",
            at + 1,
            share(counts.added[at], charts),
            share(counts.signs_moved[at], charts),
        );
    }
    let total = |column: &[usize]| column.iter().sum::<usize>();
    let placed = charts * Saham::ALL.len();
    let housed = Saham::ALL.iter().filter(|s| reads_a_house(**s)).count();
    let derived: Vec<usize> = Saham::ALL
        .iter()
        .filter(|saham| reads_a_saham(**saham))
        .map(|saham| saham.index())
        .collect();
    let _ = write!(
        out,
        "\n## 14. The sahams, and what each reading moves\n\n\
         A saham is **a − b + c**, carried one sign further when c does not \
         fall between b and a counted from b (`03-design/tajika-sahams.md`). \
         The source gives forty-one, and each is read here in every chart \
         under its readings and under each rival, through \
         `sdk.chart().sahams_with_rules`. {} of the {} charts open by day \
         and read the day formulas.\n\n\
         **Two identities hold, and the pass fails if either does not:** a \
         saham reading no other moves when no sign is ever added exactly as \
         often as it took one, and a saham that reads no house never moves \
         under equal houses — Samarthya and Manmatha among them, which read only the \
         lagna's lord. The {} that read another saham also move with \
         it: with no sign ever added they move in {} placements to the {} \
         in which they took a sign themselves, because Punya, Guru and Vidya \
         lose theirs. Over all forty-one, {} of {} placements took the \
         sign. The three pairs the \
         source gives one formula each — Vidya and Guru, Raja and Pitri, \
         Kshama and Kali — agree in every column.\n\n\
         **Counting \"between\" in whole signs moves {} placements in all \
         ({}).** That is the reading a widely used program applies; the \
         source's own birth-chart Punya refutes it, the Sun, lagna and \
         Moon all in Leo. **Equal houses from the lagna put {}** of the \
         placements of the {} sahams that read a house past the first in \
         another sign \
         than Sripati's mid-points do, which the source builds from the \
         lagna and the midheaven and prints. This profile's own chalit is \
         Vehlow's, equal houses centred on the lagna, which is why the \
         default reads Sripati's from the angles and not the chart's \
         chalit: read off the chalit, this column was zero. **Roga's \
         second reading** stands in another sign in **{}** of {} charts.\n\n\
         | saham | took the added sign | another sign, whole signs | another sign, equal houses |\n\
         |---|---:|---:|---:|\n{rows}",
        count(counts.by_day),
        count(charts),
        spelled(derived.len()),
        count(derived.iter().map(|at| counts.never_moved[*at]).sum()),
        count(derived.iter().map(|at| counts.added[*at]).sum()),
        count(total(&counts.added)),
        count(placed),
        count(total(&counts.signs_moved)),
        share(total(&counts.signs_moved), placed),
        share(total(&counts.equal_moved), charts * housed),
        spelled(housed),
        count(counts.roga_moved),
        count(charts),
    );
}

/// One chart's Harsha bala under both readings of Venus's place, refused
/// if the rival moves anything but Venus.
fn count_harsha(
    sdk: &Context,
    annual: &Document,
    name: &str,
    counts: &mut HarshaCounts,
) -> Result<(), String> {
    let verse = sdk
        .chart()
        .harsha(annual)
        .map_err(|why| format!("{name}: its Harsha bala: {why}"))?;
    let program = sdk
        .chart()
        .harsha_with_rules(
            annual,
            HarshaRules {
                venus: VenusPlace::Twelfth,
            },
        )
        .map_err(|why| format!("{name}: its Harsha bala, Venus in the 12th: {why}"))?;
    for (at, (one, rival)) in verse.iter().zip(&program).enumerate() {
        if let Some(slot) = counts
            .grades
            .get_mut(at)
            .and_then(|row| row.get_mut(one.grade as usize))
        {
            *slot += 1;
        }
        if one.graha == Graha::Venus {
            counts.venus_moved += usize::from(one.total != rival.total);
        } else if one != rival {
            return Err(format!(
                "{name}: Venus's place moved {:?}'s Harsha bala",
                one.graha
            ));
        }
        match one.graha {
            Graha::Sun => counts.sun_dina_ratri += usize::from(one.dina_ratri),
            Graha::Moon => counts.moon_dina_ratri += usize::from(one.dina_ratri),
            _ => {}
        }
    }
    Ok(())
}

/// The planets that can never hold all four parts: each one's house of joy
/// is of the other gender — the Sun's 9th feminine, Venus's 5th and
/// Saturn's 12th masculine — so the first and third parts exclude each
/// other. Listed so the pass fails both ways: one of these reaching twenty
/// is a broken rule, and one of the others never reaching it is a sweep
/// that did not reach the charts it counts.
const NEVER_TWENTY: [Graha; 3] = [Graha::Sun, Graha::Venus, Graha::Saturn];

/// The day-and-night part is the year's part of the day and nothing else:
/// the male Sun holds it in every chart read by day, the female Moon in
/// every chart read by night. And twenty is reached by exactly the four
/// planets not in [`NEVER_TWENTY`].
fn harsha_holds(charts: usize, by_day: usize, counts: &HarshaCounts) -> Result<(), String> {
    for (graha, grades) in SEVEN.iter().zip(&counts.grades) {
        let twenty = grades
            .get(HarshaGrade::Extraordinary as usize)
            .copied()
            .unwrap_or(0);
        if NEVER_TWENTY.contains(graha) == (twenty > 0) {
            return Err(format!(
                "{graha:?} holds all four parts in {twenty} charts, and the list \
                 of planets that never can says the opposite"
            ));
        }
    }
    if counts.sun_dina_ratri != by_day || counts.moon_dina_ratri != charts - by_day {
        return Err(format!(
            "the Harsha bala's day part does not decompose: the Sun holds it \
             in {} charts and the Moon in {}, where {by_day} of {charts} open by day",
            counts.sun_dina_ratri, counts.moon_dina_ratri,
        ));
    }
    Ok(())
}

fn the_harsha(out: &mut String, kinds: &Kinds) {
    let counts = &kinds.harsha;
    let charts = kinds.charts;
    let mut rows = String::new();
    for (graha, grades) in SEVEN.iter().zip(&counts.grades) {
        let cells: Vec<String> = grades.iter().map(|n| share(*n, charts)).collect();
        let _ = writeln!(rows, "| {graha:?} | {} |", cells.join(" | "));
    }
    let placed = charts * SEVEN.len();
    let at_most = |grade: HarshaGrade| -> usize {
        counts
            .grades
            .iter()
            .map(|row| row.iter().take(grade as usize + 1).sum::<usize>())
            .sum()
    };
    let twenty = placed - at_most(HarshaGrade::PoornaBali);
    let _ = write!(
        out,
        "\n## 15. The Harsha bala\n\n\
         Four places a planet is \"happy\" in, five units each: its house of \
         joy, its exaltation or own sign, a house of its own gender, and the \
         year's own part of the day (`03-design/tajika-harsha.md`). Read in \
         every chart through `sdk.chart().harsha`.\n\n\
         **The source says fifteen is generally the most a planet reaches \
         and twenty rather unusual.** Over {} placements of the seven, \
         {} hold fifteen or less and **{}** hold all four ({}).\n\n\
         **Venus's rival place moves Venus alone, and the pass fails if it \
         moves anything else.** Read as a widely used program reads it, in \
         the 12th rather than the verse's 5th, Venus holds another total \
         in **{}** of {} charts ({}). The day part decomposes too: the Sun \
         holds it in exactly the {} charts read by day, the Moon in exactly \
         the {} read by night.\n\n\
         **{} never hold all four, and the pass fails if one does or if any \
         other never does.** Each one's house of joy is of the other gender \
         — the Sun's 9th is feminine, Venus's 5th and Saturn's 12th \
         masculine — so the place and the gender parts exclude each other, \
         and their twenties are zeros by the rule rather than by chance.\n\n\
         | planet | Nirbala, 0 | Alpabali, 5 | Madhya Bali, 10 | Poorna Bali, 15 | twenty |\n\
         |---|---:|---:|---:|---:|---:|\n{rows}",
        count(placed),
        count(at_most(HarshaGrade::PoornaBali)),
        count(twenty),
        share(twenty, placed),
        count(counts.venus_moved),
        count(charts),
        share(counts.venus_moved, charts),
        count(counts.sun_dina_ratri),
        count(counts.moon_dina_ratri),
        listed(&NEVER_TWENTY.map(|graha| format!("{graha:?}"))),
    );
}

/// One year's lord under both readings of the Moon, counted by step, and
/// the default's lord returned for the readings that need it.
///
/// Refused if a lord is not among its own claimants without succeeding
/// the Moon, if the Moon holds the year by any step but its own sign, or
/// if a year the Moon did not lead reads differently under the two.
fn count_year_lord(
    sdk: &Context,
    birth: &Document,
    annual: &Document,
    year: u16,
    name: &str,
    counts: &mut LordCounts,
) -> Result<Graha, String> {
    let ask = |moon| {
        sdk.chart()
            .varshesha(
                birth,
                annual,
                year,
                teistro::VarsheshaRules {
                    moon,
                    ..teistro::VarsheshaRules::default()
                },
            )
            .map_err(|why| format!("{name}: its year lord, the Moon {moon:?}: {why}"))
    };
    let charak = ask(MoonMayRule::PassedOver)?;
    let nilakanthi = ask(MoonMayRule::Ithasala)?;
    counts.years += 1;
    for (read, tally) in [
        (&charak, &mut counts.charak),
        (&nilakanthi, &mut counts.nilakanthi),
    ] {
        let at = Chosen::ALL
            .iter()
            .position(|step| *step == read.chosen)
            .ok_or_else(|| format!("{name}: {:?} is not in Chosen::ALL", read.chosen))?;
        if let Some(slot) = tally.get_mut(at) {
            *slot += 1;
        }
        let claimed = read.claims.iter().any(|claim| claim.graha == read.graha);
        if !claimed && !read.chosen.succeeds_the_moon() {
            return Err(format!(
                "{name}: {:?} holds the year by {:?} and is not among its claimants",
                read.graha, read.chosen
            ));
        }
        if read.graha == Graha::Moon && read.chosen != Chosen::MoonsSignLord {
            return Err(format!(
                "{name}: the Moon holds the year by {:?}",
                read.chosen
            ));
        }
    }
    if !charak.moon_passed_over && charak.graha != nilakanthi.graha {
        return Err(format!(
            "{name}: the Moon did not lead, and the two readings disagree"
        ));
    }
    for read in [&charak, &nilakanthi] {
        if matches!(
            read.chosen,
            Chosen::AnnualLagnaLordUnaspected | Chosen::StrongestUnaspected
        ) {
            return Err(format!(
                "{name}: {:?} chose the year, and its reading was not asked",
                read.chosen
            ));
        }
    }
    counts.moon_led += usize::from(charak.moon_passed_over);
    counts.moon_rules += usize::from(charak.graha == Graha::Moon);
    counts.changed += usize::from(charak.graha != nilakanthi.graha);
    if charak.chosen.succeeds_the_moon()
        && !charak
            .claims
            .iter()
            .any(|claim| claim.graha == charak.graha)
    {
        counts.outside += 1;
    }
    Ok(charak.graha)
}

/// Every saham of one chart, read for its strength under the year's own
/// lord, and sorted by the lists it meets.
///
/// Refused if a saham whose lord conjoins it has a friend for company
/// under positional friendship: two planets in one sign are enemies
/// there, which is the design page's claim and is held here.
fn count_strength(
    sdk: &Context,
    annual: &Document,
    lord: Graha,
    name: &str,
    counts: &mut StrengthCounts,
) -> Result<(), String> {
    let read = sdk
        .chart()
        .saham_strength(annual, &Saham::ALL, Some(lord))
        .map_err(|why| format!("{name}: its sahams' strength: {why}"))?;
    for one in &read {
        if one.lord_conjoins && one.with_friend {
            return Err(format!(
                "{name}: {:?}'s lord conjoins it and a friend keeps it company, \
                 which positional friendship forbids",
                one.saham
            ));
        }
        let strong = one.strong();
        let weak = one.weak();
        for (slot, (_, holds)) in counts.strong.iter_mut().zip(strong) {
            *slot += usize::from(holds);
        }
        for (slot, (_, holds)) in counts.weak.iter_mut().zip(weak) {
            *slot += usize::from(holds);
        }
        match (
            strong.iter().any(|(_, holds)| *holds),
            weak.iter().any(|(_, holds)| *holds),
        ) {
            (true, false) => counts.strong_only += 1,
            (false, true) => counts.weak_only += 1,
            (true, true) => counts.both += 1,
            // Strong (c), "its lord aspects or conjoins it", and weak (c),
            // "neither aspects nor conjoins it", are each other's
            // negation, so every saham meets one list: a saham meeting
            // neither is a broken walker, not a finding.
            (false, false) => {
                return Err(format!(
                    "{name}: {:?} meets neither list, which the two (c) clauses forbid",
                    one.saham
                ));
            }
        }
    }
    Ok(())
}

fn the_strength(out: &mut String, kinds: &Kinds) {
    let counts = &kinds.strength;
    let placed = counts.strong_only + counts.weak_only + counts.both;
    let mut rows = String::new();
    // The names are the walker's own, never typed out here, so a clause
    // renamed renames its row.
    for (clause, held) in teistro::StrongClause::ALL.iter().zip(&counts.strong) {
        let _ = writeln!(
            rows,
            "| strong | {} | {} |",
            clause.name(),
            share(*held, placed)
        );
    }
    for (clause, held) in teistro::WeakClause::ALL.iter().zip(&counts.weak) {
        let _ = writeln!(
            rows,
            "| weak | {} | {} |",
            clause.name(),
            share(*held, placed)
        );
    }
    let _ = write!(
        out,
        "\n## 16. A saham's strength\n\n\
         The source lists what makes a saham strong and what makes it weak, \
         and judges its worked sahams in words, never by a score \
         (`03-design/tajika-saham-strength.md`). Every one of the forty-one \
         is read in every chart through `sdk.chart().saham_strength`, under \
         the year's own lord.\n\n\
         **The two lists are not two verdicts.** Of {} placements, {} meet a \
         strong clause and no weak one, {} a weak clause and no strong one, \
         and **{} meet clauses on both lists** ({}). A score would have to \
         weigh one list against the other, which is a rule the source does \
         not state; the report gives the clauses. **None meets neither, and \
         the pass fails if one does**: the strong list's \"its lord aspects \
         or conjoins it\" and the weak list's \"neither aspects nor conjoins \
         it\" are each other's negation, so every saham meets one of them.\n\n\
         **Under positional friendship a saham whose lord conjoins it never \
         has a friend for company**, since two planets in one sign are \
         enemies there, and the pass fails if one does.\n\n\
         | list | clause | holds |\n|---|---|---:|\n{rows}",
        count(placed),
        count(counts.strong_only),
        count(counts.weak_only),
        count(counts.both),
        share(counts.both, placed),
    );
}

fn the_kuttha(out: &mut String, kinds: &Kinds) {
    let counts = &kinds.kuttha;
    let held = held_in(kinds, YearYoga::Kuttha);
    let judged = kinds.matters - kinds.same_lord;
    let mut rows = String::new();
    // Named by the walker, so a clause renamed renames its row.
    let clauses = counts
        .names
        .iter()
        .zip(counts.turned_away.iter().zip(&counts.alone));
    for (clause, (away, alone)) in clauses {
        let _ = writeln!(rows, "| {clause} | {} | {} |", count(*away), count(*alone));
    }
    let _ = write!(
        out,
        "\n## 17. Kuttha\n\n\
         Kuttha wants both lords powerful, in a kendra or a panaphara, under \
         a benefic's aspect and no malefic's (`03-design/tajika-yogas.md`, \
         crux C117). It held in **{}** of the {} judged matters ({}). It can \
         hold only where both lords are strong, and the default floors find \
         **{}** such matters, so {} of them were turned away by a clause \
         other than strength.\n\n\
         **Which clause turns a strong pair away.** A clause fails a matter \
         where it fails of either lord. The last column counts the matters \
         it turned away alone, the near misses: one clause read otherwise \
         and the Kuttha would have held. *Powerful* reads zero by \
         construction, since only strong pairs are counted, and the pass \
         fails if it does not.\n\n\
         | clause | turned away | alone |\n|---|---:|---:|\n{rows}\n\
         **The commentary's full Moon** (`moon_benefic: waxing`) holds \
         Kuttha in **{}** matters to the default's {}. It can only remove \
         a benefic, so it can never add a Kuttha. The pass asks it of every \
         strong pair and fails if it adds one. It also fails if `favour` \
         and the yoga disagree about a single matter, or if Kuttha's strong \
         pairs differ from the floors' own count.\n",
        count(held),
        count(judged),
        share(held, judged),
        count(counts.strong_pairs),
        count(counts.strong_pairs - held),
        count(counts.waxing),
        count(held),
    );
}

fn the_year_lords(out: &mut String, kinds: &Kinds) {
    let counts = &kinds.lords;
    let mut rows = String::new();
    for ((step, charak), nilakanthi) in Chosen::ALL
        .iter()
        .zip(&counts.charak)
        .zip(&counts.nilakanthi)
    {
        let _ = writeln!(
            rows,
            "| `{step:?}` | {} | {} |",
            count(*charak),
            count(*nilakanthi)
        );
    }
    let succeeded = counts
        .charak
        .iter()
        .zip(Chosen::ALL)
        .filter(|(_, step)| step.succeeds_the_moon())
        .map(|(held, _)| held)
        .sum::<usize>();
    let _ = write!(
        out,
        "\n## 18. The lord of the year, over the recorded years\n\n\
         Every recorded year's lord, through `sdk.chart().varshesha`, by the \
         step of the chain that chose it (`03-design/varshesha.md`). Charak's \
         chain passes the Moon over for the next claimant that aspects and, \
         where there is none, for its **Ithasala successor**. The *Tajika \
         Nilakanthi*'s own view takes the successor at once \
         (`moon: ithasala`).\n\n\
         | step | Charak | Nilakanthi |\n|---|---:|---:|\n{rows}\n\
         Of {} years, the Moon would have led the chain in **{}**. Under \
         Charak it was succeeded in **{}**: the case the build answered \
         with the Moon itself until the successor was built. Of those \
         successors, {} hold no portfolio, which only the Moon's Ithasala \
         allows. The Moon holds the year in {}, and only ever as a Moon in \
         Cancer succeeding itself. The Nilakanthi's reading changes the \
         lord of **{}** years.\n\n\
         The rows for the lagna lord's and the strongest's readings of an \
         unaspected lagna are zero by construction, since neither is asked \
         here. The others are what the corpus holds. The pass fails if a \
         lord is not among its claimants without succeeding the Moon, if \
         the Moon holds the year by any other step, if the two readings \
         disagree about a year the Moon did not lead, or if a reading not \
         asked is ever counted.\n",
        count(counts.years),
        count(counts.moon_led),
        count(succeeded),
        count(counts.outside),
        count(counts.moon_rules),
        count(counts.changed),
    );
}

/// The three annual dashas of one year, all at once, held to what every
/// year's must satisfy: each opens on the return, its mahadashas run end to
/// end to the year's close and each one's antardashas end to end across
/// it, and the Sun's clock closes on the next return.
fn count_annual_dashas(
    sdk: &Context,
    birth: &Birth,
    annual: &Document,
    year: u16,
    next: Option<f64>,
    counts: &mut DashaCounts,
) -> Result<(), String> {
    let rules = AnnualDashaRules::default();
    let all = sdk
        .chart()
        .annual_dashas(
            &birth.document,
            annual,
            year,
            &teistro::tajika::ANNUAL_DASHAS,
            rules,
        )
        .map_err(|why| format!("{} year {year}: its annual dashas: {why}", birth.name))?;
    counts.years += 1;
    let at = |what: &str| format!("{} year {year}: {what}", birth.name);
    for dasha in &all {
        counts.periods += dasha.periods.len();
        if dasha.year.from != annual.foundation.instant {
            return Err(at(&format!(
                "the {:?} does not open on the return",
                dasha.system
            )));
        }
        let mahadashas: Vec<&PeriodRow> = dasha
            .periods
            .iter()
            .filter(|row| !row.path.contains('/'))
            .collect();
        runs_end_to_end(&mahadashas, dasha.year.from.get(), dasha.year.to.get())
            .map_err(|why| at(&format!("the {:?}'s mahadashas {why}", dasha.system)))?;
        for maha in &mahadashas {
            let prefix = format!("{}/", maha.path);
            let children: Vec<&PeriodRow> = dasha
                .periods
                .iter()
                .filter(|row| row.path.starts_with(&prefix))
                .collect();
            runs_end_to_end(&children, maha.interval.from.get(), maha.interval.to.get()).map_err(
                |why| {
                    at(&format!(
                        "the {:?}'s {} antardashas {why}",
                        dasha.system, maha.path
                    ))
                },
            )?;
        }
        if let Some(next) = next {
            let off_s = (dasha.year.to.get() - next).abs() * 86_400.0;
            if off_s >= 0.05 {
                return Err(at(&format!(
                    "the {:?}'s Sun clock closes {off_s} s from the next return",
                    dasha.system
                )));
            }
            if dasha.system == DashaSystem::Mudda {
                counts.closings += 1;
                counts.worst_close_s = counts.worst_close_s.max(off_s);
            }
        }
        if dasha.system == DashaSystem::Patyayini {
            if dasha
                .ring
                .ring
                .iter()
                .skip(1)
                .any(|share| share.weight == 0.0)
            {
                counts.ties += 1;
            }
            if dasha
                .ring
                .ring
                .first()
                .is_some_and(|share| share.sign.is_some())
            {
                counts.lagna_first += 1;
            }
        }
    }
    let sun = all
        .iter()
        .find(|dasha| dasha.system == DashaSystem::Mudda)
        .ok_or_else(|| at("no Mudda among the three"))?;
    // The even clock runs between the same two returns, so the Mudda on it
    // is the same ring on a clock of two knots: nothing to read from the
    // sky that the Sun's clock has not read already.
    let even_rules = AnnualDashaRules {
        clock: YearClock::Even,
        ..rules
    };
    let even = teistro::dasha::Clock::even(sun.year)
        .and_then(|clock| YearDasha::new(sun.ring.clone(), clock, even_rules.birth_period))
        .map(|dasha| AnnualDasha::of(&dasha, DashaSystem::Mudda, year, even_rules, sun.seed))
        .map_err(|why| at(&format!("the Mudda on an even clock: {why}")))?;
    for (one, other) in sun.periods.iter().zip(&even.periods) {
        counts.worst_even_days = counts
            .worst_even_days
            .max((one.interval.to.get() - other.interval.to.get()).abs());
    }
    Ok(())
}

/// Periods that run end to end from `from` to `to`, each beginning where
/// the last ended; a period that runs for no time is not listed and so
/// cannot break the run.
#[expect(
    clippy::float_cmp,
    reason = "a boundary is an exact share of its parent, so the ends must equal it and not be near it"
)]
fn runs_end_to_end(periods: &[&PeriodRow], from: f64, to: f64) -> Result<(), String> {
    let (Some(first), Some(last)) = (periods.first(), periods.last()) else {
        return Err(String::from("are none"));
    };
    if first.interval.from.get() != from || last.interval.to.get() != to {
        return Err(String::from("do not span their parent"));
    }
    if let Some(gap) = periods
        .windows(2)
        .find(|pair| matches!(pair, [one, other] if one.interval.to != other.interval.from))
    {
        return Err(format!("break between {} and the next", gap[0].path));
    }
    Ok(())
}

/// §19: the three annual dashas over every recorded year.
fn the_annual_dashas(out: &mut String, kinds: &Kinds) {
    let counts = &kinds.dashas;
    let _ = write!(
        out,
        "\n## 19. The annual dashas, over the recorded years\n\n\
         The Mudda, the Varsha Yogini and the Patyayini of every recorded \
         year, through `sdk.chart().annual_dashas` under the default readings \
         (`03-design/annual-dashas.md`): the Sun's clock and the birth \
         Moon's balance. {} years were read, {} periods listed to the \
         antardasha.\n\n\
         Every one of them opens on its return. In every one, the \
         mahadashas run end to end from the return to the year's close, and \
         each mahadasha's antardashas run end to end across it. The pass \
         fails on the first that does not. Where the sweep also found the \
         next return ({} years), the Sun's clock closes on it, the worst by \
         **{:.4} s**: the clock's last knot and the return's own search, \
         each held to the search's tolerance, find one crossing.\n\n\
         **An even spread between the returns stands up to {:.2} days** from \
         the Sun's clock at one of the Mudda's boundaries: the equation of \
         centre at the boundary less its value at the return, which can \
         reach twice its 1.92°. So C122 is a real choice and not a rounding: \
         the clock moves a period's end by days.\n\n\
         The Patyayini's tie rule decided the order in **{}** years{} The \
         rule is built because the verse states it, and a chart of a \
         consumer's may be the one where it holds. The year opened with the \
         lagna in {}.\n",
        count(counts.years),
        count(counts.periods),
        count(counts.closings),
        counts.worst_close_s,
        counts.worst_even_days,
        count(counts.ties),
        if counts.ties == 0 {
            ": no two of the eight shared a krishamsha to the nanoarcsecond."
        } else {
            ", each a pair sharing a krishamsha to the nanoarcsecond."
        },
        count(counts.lagna_first),
    );
}
