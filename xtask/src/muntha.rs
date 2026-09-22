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

use teistro::catalogue::Rashi;
use teistro::tajika::{MOST_YEARS, MunthaDegree};
use teistro::{Context, Ephemeris};

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
