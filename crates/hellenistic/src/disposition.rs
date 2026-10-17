//! Dispositors followed to their end: each planet's chain of dispositors,
//! where it stops, and the chart's final dispositor when every chain stops
//! at the same planet (`03-design/essential-dignities.md` §Dispositors).
//!
//! A planet's dispositor is "the ruler of the sign it is in" (Leo, *How to
//! Judge a Nativity*, p. 130); Lilly speaks of the "Lord of that Signe who
//! disposeth the ☽" (*Christian Astrology*, p. 161). Neither text follows
//! the dispositor's own dispositor, and neither names a final dispositor:
//! the chain and its end are later practice, built here from that one
//! definition applied again (C413). A chain stops where a planet is its
//! own dispositor, in its own sign, or where it comes back to a planet it
//! has already passed, which is a cycle and is reported as one (C415).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::error::Error;

/// Which planet rules each sign, so which planet disposes of a body there
/// (C414).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Rulers {
    /// The seven planets' houses, the catalogue's sign lords (Lilly's
    /// table, p. 104). The default, and the only rulers the essential
    /// dignities read.
    #[default]
    Traditional,
    /// The traditional rulers but for three signs: Uranus rules Aquarius,
    /// Neptune Pisces and Pluto Scorpio, each alone. Leo has Uranus share
    /// Aquarius with Saturn (p. 103) and Pisces "usually regarded as the
    /// house of Neptune" (p. 126); Pluto, found after him, is modern
    /// practice (C414).
    Modern,
}

impl Rulers {
    /// The planet ruling a sign under these rulers.
    ///
    /// ```
    /// use teistro_core::catalogue::{Graha, Rashi};
    /// use teistro_hellenistic::Rulers;
    ///
    /// assert_eq!(Rulers::Traditional.lord(Rashi::Scorpio), Graha::Mars);
    /// assert_eq!(Rulers::Modern.lord(Rashi::Scorpio), Graha::Pluto);
    /// assert_eq!(Rulers::Modern.lord(Rashi::Aries), Graha::Mars);
    /// ```
    #[must_use]
    pub fn lord(self, sign: Rashi) -> Graha {
        match (self, sign) {
            (Rulers::Modern, Rashi::Scorpio) => Graha::Pluto,
            (Rulers::Modern, Rashi::Aquarius) => Graha::Uranus,
            (Rulers::Modern, Rashi::Pisces) => Graha::Neptune,
            _ => sign.attributes().lord,
        }
    }

    /// Every planet that rules a sign under these rulers, so every planet
    /// a chain may need.
    #[must_use]
    pub fn lords(self) -> Vec<Graha> {
        let mut lords: Vec<Graha> = Vec::with_capacity(10);
        for sign in Rashi::ALL {
            let lord = self.lord(sign);
            if !lords.contains(&lord) {
                lords.push(lord);
            }
        }
        lords
    }
}

/// Where a chain of dispositors stops.
///
/// Written `{"OWN_SIGN": "SUN"}` or `{"CYCLE": ["MARS", "SUN"]}`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum ChainEnd {
    /// At a planet in its own sign, which disposes of itself.
    OwnSign(Graha),
    /// At a cycle of two or more planets each in the next one's sign, the
    /// last in the first's: two of them are a mutual reception by house
    /// (Leo, p. 249n). It starts at the member given first, and no member
    /// is chosen over the others (C415).
    Cycle(Vec<Graha>),
}

impl ChainEnd {
    /// The planet in its own sign, or `None` for a cycle.
    #[must_use]
    pub const fn own_sign(&self) -> Option<Graha> {
        match self {
            ChainEnd::OwnSign(planet) => Some(*planet),
            ChainEnd::Cycle(_) => None,
        }
    }
}

/// One body's dispositors, followed until one repeats.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct DispositorChain {
    /// The body the chain starts from.
    pub planet: Graha,
    /// Its dispositor, that planet's dispositor, and so on, up to the
    /// last before the first that repeats: empty for a body in its own
    /// sign, and never holding the body itself or any planet twice.
    pub dispositors: Vec<Graha>,
    /// Where it stops.
    pub end: ChainEnd,
}

/// Every body's chain of dispositors in one chart, and its final
/// dispositor when there is one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Dispositions {
    /// The rulers the chains were followed under.
    pub rulers: Rulers,
    /// One chain a body, in the order the bodies were given.
    pub chains: Vec<DispositorChain>,
    /// Where the chains stop, each once, in the order the chains first
    /// reach it.
    pub ends: Vec<ChainEnd>,
    /// The one planet in its own sign every chain stops at; `None` when
    /// the chains stop at more than one end, or at a cycle (C415).
    pub final_dispositor: Option<Graha>,
}

/// Every body's chain of dispositors, under `rulers`, from the bodies'
/// longitudes in the chart's own zodiac.
///
/// Every planet the rulers name must be among the bodies, since any of
/// them may be a dispositor: the seven under [`Rulers::Traditional`], and
/// Uranus, Neptune and Pluto besides under [`Rulers::Modern`]. An outer
/// planet under the traditional rulers is a body with a chain and is
/// never anyone's dispositor. The nodes rule no sign and are refused
/// (C416).
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_hellenistic::{ChainEnd, Rulers, dispositions};
///
/// // Jupiter in Sagittarius, and every other chain running to him.
/// let read = dispositions(
///     &[
///         (Graha::Saturn, 250.0),  // Sagittarius
///         (Graha::Jupiter, 245.0), // Sagittarius, his own
///         (Graha::Mars, 100.0),    // Cancer, the Moon's
///         (Graha::Sun, 10.0),      // Aries, Mars's
///         (Graha::Venus, 340.0),   // Pisces, Jupiter's
///         (Graha::Mercury, 20.0),  // Aries
///         (Graha::Moon, 355.0),    // Pisces
///     ],
///     Rulers::Traditional,
/// )?;
/// assert_eq!(read.final_dispositor, Some(Graha::Jupiter));
/// let sun = &read.chains[3];
/// assert_eq!(sun.dispositors, [Graha::Mars, Graha::Moon, Graha::Jupiter]);
/// assert_eq!(sun.end, ChainEnd::OwnSign(Graha::Jupiter));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// `INVALID_ARG` naming `planets[k]` for a node or a longitude that is
/// not a number, and `planets` for a body given twice or a ruler missing.
pub fn dispositions(planets: &[(Graha, f64)], rulers: Rulers) -> Result<Dispositions, Error> {
    for (at, &(planet, longitude_deg)) in planets.iter().enumerate() {
        if matches!(planet, Graha::Rahu | Graha::Ketu) {
            return Err(Error::invalid_arg(format!(
                "{planet:?} rules no sign, so disposes of nothing"
            ))
            .with_field(format!("planets[{at}]"))
            .with_hint("leave the nodes out; a node's own dispositor is its sign's lord"));
        }
        if !longitude_deg.is_finite() {
            return Err(
                Error::invalid_arg(format!("{longitude_deg} is not a longitude"))
                    .with_field(format!("planets[{at}]")),
            );
        }
        if planets.iter().take(at).any(|&(other, _)| other == planet) {
            return Err(Error::invalid_arg(format!("{planet:?} is given twice"))
                .with_field("planets")
                .with_hint("give each body once"));
        }
    }
    if let Some(missing) = rulers
        .lords()
        .into_iter()
        .find(|lord| !planets.iter().any(|&(planet, _)| planet == *lord))
    {
        return Err(Error::invalid_arg(format!(
            "{missing:?} rules a sign under the {rulers:?} rulers and is not given"
        ))
        .with_field("planets")
        .with_hint(match rulers {
            Rulers::Modern => {
                "the modern rulers need the seven and Uranus, Neptune and Pluto; or follow the traditional rulers"
            }
            Rulers::Traditional => "the traditional rulers need all seven planets",
        }));
    }
    let lord_of = |planet: Graha| -> Result<Graha, Error> {
        planets
            .iter()
            .find(|&&(one, _)| one == planet)
            .map(|&(_, longitude_deg)| rulers.lord(Rashi::of_longitude(longitude_deg)))
            .ok_or_else(|| Error::internal(format!("{planet:?} rules a sign and was given")))
    };
    let given_at = |planet: Graha| planets.iter().position(|&(one, _)| one == planet);
    let mut chains = Vec::with_capacity(planets.len());
    for &(planet, _) in planets {
        // The body and every dispositor passed, none twice: each lord is
        // among the bodies, so the walk ends within their number.
        let mut passed = vec![planet];
        let mut now = planet;
        let (dispositors, end) = loop {
            let lord = lord_of(now)?;
            if lord == now {
                break (passed.split_off(1), ChainEnd::OwnSign(now));
            }
            if let Some(from) = passed.iter().position(|&one| one == lord) {
                let dispositors = passed.get(1..).unwrap_or_default().to_vec();
                let mut cycle = passed.split_off(from);
                // From the member given first, keeping the order of
                // disposition: each in the next one's sign.
                let first = (0..cycle.len())
                    .min_by_key(|&k| cycle.get(k).copied().and_then(given_at))
                    .unwrap_or(0);
                cycle.rotate_left(first);
                break (dispositors, ChainEnd::Cycle(cycle));
            }
            passed.push(lord);
            now = lord;
        };
        chains.push(DispositorChain {
            planet,
            dispositors,
            end,
        });
    }
    let mut ends: Vec<ChainEnd> = Vec::new();
    for chain in &chains {
        if !ends.contains(&chain.end) {
            ends.push(chain.end.clone());
        }
    }
    let final_dispositor = match ends.as_slice() {
        [one] => one.own_sign(),
        _ => None,
    };
    Ok(Dispositions {
        rulers,
        chains,
        ends,
        final_dispositor,
    })
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index their own results"
    )]

    use super::{ChainEnd, Rulers, dispositions};
    use teistro_core::catalogue::Graha::{
        self, Jupiter, Ketu, Mars, Mercury, Moon, Neptune, Pluto, Saturn, Sun, Uranus, Venus,
    };

    /// The seven in the Chaldean order at these longitudes.
    fn seven(at: [f64; 7]) -> Vec<(Graha, f64)> {
        [Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon]
            .into_iter()
            .zip(at)
            .collect()
    }

    fn chain_of(read: &super::Dispositions, planet: Graha) -> &super::DispositorChain {
        read.chains.iter().find(|one| one.planet == planet).unwrap()
    }

    /// Worked by hand: Saturn in Capricorn, his own; Jupiter in Aquarius
    /// and Mars in Capricorn, both Saturn's; the Sun in Aries, Mars's;
    /// Venus in Leo, the Sun's; Mercury in Taurus, Venus's; the Moon in
    /// Gemini, Mercury's. Every chain runs down to Saturn: the Moon's
    /// through Mercury, Venus, the Sun and Mars.
    #[test]
    fn every_chain_ending_at_one_planet_in_its_own_sign_is_a_final_dispositor() {
        let read = dispositions(
            &seven([280.0, 310.0, 290.0, 15.0, 130.0, 40.0, 70.0]),
            Rulers::Traditional,
        )
        .unwrap();
        assert_eq!(read.final_dispositor, Some(Saturn));
        assert_eq!(read.ends, [ChainEnd::OwnSign(Saturn)]);
        assert_eq!(chain_of(&read, Saturn).dispositors, []);
        assert_eq!(chain_of(&read, Jupiter).dispositors, [Saturn]);
        assert_eq!(
            chain_of(&read, Moon).dispositors,
            [Mercury, Venus, Sun, Mars, Saturn]
        );
        let order: Vec<Graha> = read.chains.iter().map(|one| one.planet).collect();
        assert_eq!(order, [Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon]);
    }

    /// Worked by hand: the Sun in Aries and Mars in Leo, each in the
    /// other's house (Lilly, p. 112), with Venus in her Taurus. Saturn in
    /// Aries runs into the Sun–Mars cycle, Jupiter in Leo too; Mercury
    /// in Libra and the Moon in Taurus run to Venus. Two ends, so no
    /// final dispositor, and the cycle is reported, not resolved.
    #[test]
    fn a_mutual_reception_is_a_cycle_and_two_ends_have_no_final_dispositor() {
        let read = dispositions(
            &seven([10.0, 130.0, 125.0, 5.0, 45.0, 190.0, 50.0]),
            Rulers::Traditional,
        )
        .unwrap();
        // The cycle starts at the member given first: Mars, before the Sun
        // in the Chaldean order.
        let cycle = ChainEnd::Cycle(vec![Mars, Sun]);
        assert_eq!(read.ends, [cycle.clone(), ChainEnd::OwnSign(Venus)]);
        assert_eq!(read.final_dispositor, None);
        assert_eq!(chain_of(&read, Saturn).dispositors, [Mars, Sun]);
        assert_eq!(chain_of(&read, Saturn).end, cycle);
        // A member of the cycle passes the other and stops before itself.
        assert_eq!(chain_of(&read, Sun).dispositors, [Mars]);
        assert_eq!(chain_of(&read, Mars).dispositors, [Sun]);
        assert_eq!(chain_of(&read, Mercury).dispositors, [Venus]);
        assert_eq!(chain_of(&read, Moon).end, ChainEnd::OwnSign(Venus));
    }

    /// Every chain into one cycle is one end and still no final
    /// dispositor: the Moon in Taurus, Venus in Gemini, Mercury in
    /// Cancer — Venus, Mercury and the Moon round in three — and the four
    /// others in signs of those three.
    #[test]
    fn a_single_cycle_names_no_winner() {
        let read = dispositions(
            &seven([35.0, 40.0, 100.0, 160.0, 65.0, 95.0, 50.0]),
            Rulers::Traditional,
        )
        .unwrap();
        assert_eq!(read.ends, [ChainEnd::Cycle(vec![Venus, Mercury, Moon])]);
        assert_eq!(read.final_dispositor, None);
        // Saturn in Taurus: Venus, Mercury, the Moon, then Venus again.
        assert_eq!(chain_of(&read, Saturn).dispositors, [Venus, Mercury, Moon]);
        assert_eq!(chain_of(&read, Moon).dispositors, [Venus, Mercury]);
    }

    /// Two planets in their own signs and nothing else: two ends.
    #[test]
    fn two_planets_in_their_own_signs_are_two_ends() {
        // Saturn in Aquarius, the Sun in Leo; Jupiter in Aquarius, Mars in
        // Leo, Venus in Leo, Mercury in Aquarius, the Moon in Aquarius.
        let read = dispositions(
            &seven([305.0, 310.0, 125.0, 130.0, 135.0, 315.0, 320.0]),
            Rulers::Traditional,
        )
        .unwrap();
        assert_eq!(
            read.ends,
            [ChainEnd::OwnSign(Saturn), ChainEnd::OwnSign(Sun)]
        );
        assert_eq!(read.final_dispositor, None);
    }

    /// Under the modern rulers Mars in Scorpio is Pluto's, and the
    /// traditional rulers give the same sky Mars as its end.
    #[test]
    fn the_modern_rulers_move_three_signs() {
        let mut planets = seven([280.0, 310.0, 215.0, 15.0, 130.0, 40.0, 70.0]);
        // Under the traditional rulers Mars in Scorpio is his own.
        let old = dispositions(&planets, Rulers::Traditional).unwrap();
        assert_eq!(chain_of(&old, Mars).end, ChainEnd::OwnSign(Mars));
        // Pluto in Scorpio, Uranus in Pisces, Neptune in Aquarius: Uranus
        // and Neptune each in the other's sign under the modern rulers.
        planets.extend([(Uranus, 340.0), (Neptune, 305.0), (Pluto, 220.0)]);
        let new = dispositions(&planets, Rulers::Modern).unwrap();
        assert_eq!(chain_of(&new, Mars).dispositors, [Pluto]);
        assert_eq!(chain_of(&new, Mars).end, ChainEnd::OwnSign(Pluto));
        // Jupiter in Aquarius goes to Uranus, then Neptune, and round.
        assert_eq!(chain_of(&new, Jupiter).dispositors, [Uranus, Neptune]);
        assert_eq!(
            chain_of(&new, Jupiter).end,
            ChainEnd::Cycle(vec![Uranus, Neptune])
        );
        // Outer planets under the traditional rulers are bodies only.
        let old = dispositions(&planets, Rulers::Traditional).unwrap();
        assert_eq!(chain_of(&old, Pluto).dispositors, [Mars]);
        assert_eq!(chain_of(&old, Uranus).dispositors, [Jupiter, Saturn]);
    }

    #[test]
    fn a_node_a_repeat_or_a_missing_ruler_is_refused_by_field() {
        let mut planets = seven([280.0, 310.0, 290.0, 15.0, 130.0, 40.0, 70.0]);
        planets.push((Ketu, 10.0));
        let why = dispositions(&planets, Rulers::Traditional).unwrap_err();
        assert_eq!(why.field(), Some("planets[7]"));
        planets[7] = (Sun, 10.0);
        let why = dispositions(&planets, Rulers::Traditional).unwrap_err();
        assert_eq!(why.field(), Some("planets"));
        planets.pop();
        let why = dispositions(&planets, Rulers::Modern).unwrap_err();
        assert_eq!(why.field(), Some("planets"));
        // The first missing in the order of the signs: Scorpio's.
        assert!(why.to_string().contains("Pluto"), "{why}");
        planets.pop();
        let why = dispositions(&planets, Rulers::Traditional).unwrap_err();
        assert!(why.to_string().contains("Moon"), "{why}");
        planets.push((Moon, f64::NAN));
        let why = dispositions(&planets, Rulers::Traditional).unwrap_err();
        assert_eq!(why.field(), Some("planets[6]"));
    }

    #[test]
    fn the_answer_is_written_by_name() {
        let read = dispositions(
            &seven([10.0, 130.0, 125.0, 5.0, 45.0, 190.0, 50.0]),
            Rulers::Traditional,
        )
        .unwrap();
        let text = serde_json::to_string(&read).unwrap();
        assert!(text.contains(r#""rulers":"TRADITIONAL""#), "{text}");
        assert!(text.contains(r#""finalDispositor":null"#), "{text}");
        assert!(text.contains(r#"{"OWN_SIGN":"#), "{text}");
        assert!(text.contains(r#"{"CYCLE":["#), "{text}");
    }
}
