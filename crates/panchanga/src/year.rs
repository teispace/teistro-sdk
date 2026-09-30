//! The lunar year and the name the sixty-year cycle gives it
//! (`03-design/samvatsara-measured.md`).
//!
//! A lunar year opens at the new moon that begins its first Chaitra — an
//! adhika Chaitra when there is one — and its first day is the one whose
//! sunrise follows that new moon, Chaitra Shukla Pratipada. Nepal's
//! panchanga committee states the naming rule in terms: the samvatsara
//! that meets the day of Chaitra Shukla Pratipada is said in the year's
//! rites. So the name is read at that sunrise, and a Jovian year that
//! begins after one pratipada and ends before the next names no year:
//! it is *lupta*.
//!
//! This module finds the openings in the almanac's own sky, the one that
//! named its days' months, so a day's month and its year agree. The
//! Jovian years are the text's, whatever the sky
//! (`teistro_calendar::samvatsara`).

use serde::{Deserialize, Serialize};
use teistro_astro::events::{Longitudes, Search};
use teistro_calendar::samvatsara::{JovianYear, Parameters};
use teistro_core::catalogue::{Kind, Masa, Samvatsara, write_in_full};
use teistro_core::error::{Error, Status};
use teistro_core::quantity::{JulianDay, Ut1, Utc};
use teistro_core::settings::SamvatsaraCount;
use teistro_port_ephemeris::{Lattice, Quantity};

use crate::limb::{self, Sidereal, Zodiac};

/// A synodic month, days.
const SYNODIC_DAYS: f64 = teistro_calendar::lunisolar::SYNODIC_DAYS;

/// How far either side of an estimate a new moon is looked for, days:
/// half a month, so the window holds exactly one.
const NEAR_DAYS: f64 = 14.7;

/// The most steps the opening search takes from its estimate. The
/// estimate counts months by name, so it is off by the adhika and kshaya
/// months between, at most two in a year; a search needing more is a
/// sky answering nonsense.
const MOST_STEPS: usize = 4;

/// One lunar year: when it opened, the name it carries, and the Jovian
/// years that ran in it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct LunarYear {
    /// The name the year carries under `calendars.samvatsara`.
    pub samvatsara: Samvatsara,
    /// Which count named it.
    pub count: SamvatsaraCount,
    /// The Vikrama year: the Shaka year plus 135.
    pub vikrama: i32,
    /// The Shaka year, whose number the southern count reads.
    pub shaka: i32,
    /// The new moon that opened the year's first Chaitra.
    pub opened: JulianDay<Utc>,
    /// The sunrise that opened Chaitra Shukla Pratipada, the year's first
    /// day, which is where the name is read.
    pub began: JulianDay<Utc>,
    /// The next year's first sunrise, which ends this one.
    pub ended: JulianDay<Utc>,
    /// The Jovian years running between `began` and `ended`, in order:
    /// the text's, whichever count names the year.
    pub jovian: Vec<JovianYear>,
    /// The Jovian year that began after this year's first sunrise and
    /// ended by the next year's, and so names no year. Only the
    /// Barhaspatya count expunges; the southern count names every lunar
    /// year in order.
    pub lupta: Option<Samvatsara>,
}

impl LunarYear {
    /// A year from its opening, its two sunrises, and the Jovian counts
    /// that name it and the next year ([`name_counts`]); the southern
    /// count ignores them and reads the Shaka year.
    ///
    /// # Errors
    ///
    /// Only a Jovian year whose bounds leave the Julian-day range.
    pub fn of(
        count: SamvatsaraCount,
        opened: JulianDay<Utc>,
        began: JulianDay<Utc>,
        ended: JulianDay<Utc>,
        (this, next): (i64, i64),
    ) -> Result<LunarYear, Error> {
        let params = &Parameters::TEXT;
        let mut jovian = vec![JovianYear::at(params, began)?];
        while let Some(last) = jovian.last().copied() {
            if last.to.get() >= ended.get() {
                break;
            }
            jovian.push(JovianYear::numbered(params, last.count + 1)?);
        }
        let shaka = teistro_calendar::gregorian::year_from_fixed(
            teistro_calendar::FixedDay::from_jd(opened).0,
        ) - 78;
        let (samvatsara, lupta) = if count == SamvatsaraCount::Chandramana {
            (
                teistro_calendar::samvatsara::chandramana(i64::from(shaka)),
                None,
            )
        } else {
            // The name the count passed over on the way to the next year's:
            // a Jovian year that ran in this one and names none.
            let skipped = if next - this >= 2 {
                Some(JovianYear::numbered(params, this + 1)?.member)
            } else {
                None
            };
            (JovianYear::numbered(params, this)?.member, skipped)
        };
        Ok(LunarYear {
            samvatsara,
            count,
            vikrama: shaka + 135,
            shaka,
            opened,
            began,
            ended,
            jovian,
            lupta,
        })
    }

    /// Whether an instant falls in the year: from its first sunrise up
    /// to the next year's.
    #[must_use]
    pub fn contains(&self, instant: JulianDay<Utc>) -> bool {
        self.began.get() <= instant.get() && instant.get() < self.ended.get()
    }

    /// Where a year names catalogue members, and of which kind: a dotted
    /// path through each year. What a boundary section writes in full;
    /// the crate's tests hold it to serde, both ways.
    pub const MEMBERS: [(&'static str, Kind); 3] = [
        ("samvatsara", Kind::Samvatsara),
        ("lupta", Kind::Samvatsara),
        ("jovian.member", Kind::Samvatsara),
    ];

    /// Years as JSON, every catalogue member written as its full key
    /// where [`LunarYear::MEMBERS`] says: what a boundary section carries
    /// and seals.
    ///
    /// # Errors
    ///
    /// `INTERNAL` if the years do not serialise, which a value this crate
    /// built cannot do.
    pub fn in_full(years: &[LunarYear]) -> Result<serde_json::Value, Error> {
        let mut value = serde_json::to_value(years).map_err(|error| {
            Error::internal(format!("the lunar years did not serialise: {error}"))
        })?;
        for (path, kind) in LunarYear::MEMBERS {
            write_in_full(&mut value, path, kind);
        }
        Ok(value)
    }
}

/// The longest run of years whose running count rises by one that can
/// follow a repeated name: a repeat needs a Jovian year spanning a
/// 354-day lunar year, and the run ends at the next 384-day one, which
/// the calendar puts at most three years on. `samvatsara-measured.md`
/// holds this against the full recursion over six centuries.
pub const MOST_ADVANCED_YEARS: usize = 4;

/// The Jovian count naming the year at `index` of a run of consecutive
/// lunar years, from the count running at each one's pratipada (oldest
/// first), or `None` when the years before it are too few to say.
///
/// `BARHASPATYA_RUNNING` takes the count as it runs. `BARHASPATYA` never
/// lets a name name two years: a year takes the running count or one
/// more than the year before's name, whichever is later. So a year whose
/// running count repeats the year before's is named one ahead, and the
/// lead carries through the years after it whose count rises by one,
/// until a year whose count rises by two, which is where the expunged
/// name falls. Walking back from the year: a repeat behind an unbroken
/// run of ones leads, a rise of two ends it.
#[must_use]
pub fn name_counts(running: &[i64], index: usize, count: SamvatsaraCount) -> Option<i64> {
    let own = *running.get(index)?;
    if count != SamvatsaraCount::Barhaspatya {
        return Some(own);
    }
    for back in 1..=MOST_ADVANCED_YEARS {
        let later = *running.get(index.checked_sub(back - 1)?)?;
        let earlier = *running.get(index.checked_sub(back)?)?;
        match later - earlier {
            0 => return Some(own + 1),
            1 => {}
            _ => return Some(own),
        }
    }
    Some(own)
}

/// The new moon that opened the first Chaitra at or before an instant.
///
/// The month holding the instant is found, and the months back to
/// Chaitra counted by name; the estimate is then corrected by looking
/// at the new moon it lands near, since an adhika month between puts it
/// a month late and a kshaya one a month early. A Chaitra whose previous
/// month is also Chaitra is the nija one after an adhika Chaitra, and
/// the year opened with the adhika.
///
/// # Errors
///
/// The provider's refusal, or a sky in which the search does not settle,
/// which is `NOT_CONVERGED` naming the instant.
pub fn chaitra_opening<S: Longitudes + ?Sized>(
    tropical: &S,
    zodiac: Zodiac,
    at: JulianDay<Utc>,
) -> Result<JulianDay<Utc>, Error> {
    let window = teistro_core::interval::Interval::new(at, at)?;
    let month = limb::lunar_month_span(tropical, window, zodiac)?;
    let masa = limb::masa_at(tropical, month.from, zodiac)?;
    let back = f64::from(months_after_chaitra(masa));
    let mut guess = new_moon_near(tropical, zodiac, month.from.get() - back * SYNODIC_DAYS)?;
    for _ in 0..MOST_STEPS {
        let after = months_after_chaitra(limb::masa_at(tropical, guess, zodiac)?);
        let step = match after {
            0 => {
                let before = new_moon_near(tropical, zodiac, guess.get() - SYNODIC_DAYS)?;
                if limb::masa_at(tropical, before, zodiac)? == Masa::Chaitra {
                    return Ok(before);
                }
                return Ok(guess);
            }
            // Later than Chaitra: an adhika month between put the
            // estimate late.
            1..=6 => -SYNODIC_DAYS,
            // Earlier: a kshaya month between put it early.
            _ => SYNODIC_DAYS,
        };
        guess = new_moon_near(tropical, zodiac, guess.get() + step)?;
    }
    Err(Error::new(
        Status::NotConverged,
        format!("the lunar year holding {at} was not found in {MOST_STEPS} months either way"),
    ))
}

/// How many months a month's name comes after Chaitra: 0 for Chaitra, 11
/// for Phalguna.
fn months_after_chaitra(masa: Masa) -> u8 {
    let after = (masa.id() + 12 - Masa::Chaitra.id()) % 12;
    u8::try_from(after).unwrap_or(0)
}

/// The new moon within half a month of an estimate.
fn new_moon_near<S: Longitudes + ?Sized>(
    tropical: &S,
    zodiac: Zodiac,
    estimate: f64,
) -> Result<JulianDay<Utc>, Error> {
    let source = Sidereal::over(tropical, zodiac);
    let found = Search::new(&source, Quantity::ELONGATION, Lattice::single(0.0)).between(
        JulianDay::<Ut1>::literal(estimate - NEAR_DAYS),
        JulianDay::<Ut1>::literal(estimate + NEAR_DAYS),
    )?;
    let nearest = found
        .iter()
        .map(|event| event.instant.get())
        .min_by(|a, b| (a - estimate).abs().total_cmp(&(b - estimate).abs()))
        .ok_or_else(|| {
            Error::new(
                Status::NotConverged,
                format!("no new moon within {NEAR_DAYS} days of {estimate}"),
            )
        })?;
    JulianDay::try_new(nearest).map_err(Error::from)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "a test fails by panicking on what it built itself, and indexes it"
    )]

    use std::collections::BTreeSet;

    use serde_json::Value;

    use super::*;

    /// Every dotted path to a string in a JSON value, a list's items
    /// sharing their list's path.
    fn string_paths(value: &Value, path: &str, into: &mut BTreeSet<String>) {
        match value {
            Value::String(_) => {
                into.insert(path.to_owned());
            }
            Value::Array(items) => {
                for item in items {
                    string_paths(item, path, into);
                }
            }
            Value::Object(fields) => {
                for (key, inner) in fields {
                    let path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{path}.{key}")
                    };
                    string_paths(inner, &path, into);
                }
            }
            _ => {}
        }
    }

    /// VS 2077 by the text's count: Ananda ran inside it and was expunged,
    /// so every member field is written.
    fn expunging() -> LunarYear {
        let jd = |day: f64| JulianDay::<Utc>::try_new(day).unwrap();
        let began = jd(2_458_935.5);
        let this = JovianYear::at(&Parameters::TEXT, began).unwrap().count;
        LunarYear::of(
            SamvatsaraCount::Barhaspatya,
            jd(2_458_934.0),
            began,
            jd(2_459_319.5),
            (this, this + 2),
        )
        .unwrap()
    }

    /// `LunarYear::MEMBERS` names every catalogue member a year holds and
    /// nothing else: each string in its JSON is listed there or here as
    /// not a member, and each listed path is present.
    #[test]
    fn a_year_names_its_catalogue_members_where_its_table_says() {
        // The knob that named the year, which is a setting's value.
        const NOT_MEMBERS: [&str; 1] = ["count"];
        let year = expunging();
        assert!(year.lupta.is_some());
        let mut found = BTreeSet::new();
        string_paths(&serde_json::to_value(&year).unwrap(), "", &mut found);
        let members: BTreeSet<String> = LunarYear::MEMBERS
            .iter()
            .map(|(path, _)| (*path).to_owned())
            .collect();
        let plain: BTreeSet<String> = NOT_MEMBERS.iter().map(|path| (*path).to_owned()).collect();
        assert!(members.is_disjoint(&plain));
        let listed: BTreeSet<String> = members.union(&plain).cloned().collect();
        assert_eq!(
            found, listed,
            "a string path is unlisted, or a listed one is gone"
        );
    }

    /// The full keys read back into the years they were written from.
    #[test]
    fn years_written_in_full_read_back() {
        let years = vec![expunging()];
        let full = LunarYear::in_full(&years).unwrap();
        assert_eq!(full[0]["samvatsara"], "samvatsara.PRAMADICHA");
        assert_eq!(full[0]["lupta"], "samvatsara.ANANDA");
        assert_eq!(full[0]["jovian"][0]["member"], "samvatsara.PRAMADICHA");
        let back: Vec<LunarYear> = serde_json::from_value(full).unwrap();
        assert_eq!(back, years);
    }
}
