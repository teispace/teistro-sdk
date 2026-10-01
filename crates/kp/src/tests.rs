#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]

use teistro_core::Nas;
use teistro_core::catalogue::Graha::{
    Jupiter, Ketu, Mars, Mercury, Moon, Rahu, Saturn, Sun, Venus,
};
use teistro_core::catalogue::{Graha, Nakshatra};
use teistro_dasha::VIMSHOTTARI_LORDS;

use super::*;

/// A longitude from its sign (0 for Aries) and its degrees, minutes and
/// seconds inside the sign.
fn at(sign: i64, degrees: i64, minutes: i64, seconds: i64) -> Nas {
    Nas::new(
        sign * Nas::PER_SIGN
            + degrees * Nas::PER_DEGREE
            + minutes * Nas::PER_ARCMINUTE
            + seconds * Nas::PER_ARCSECOND,
    )
}

/// One nanoarcsecond.
const ONE: Nas = Nas::new(1);

/// Rows of the 249 table as KP Reader I prints them: the number, where it
/// opens, and its sign, star and sub lords. Number 1, the two numbers the
/// Reader works (48 and 74), the rows either side of two sign ends, and
/// the last twelve, every row whose print is legible in the scan.
#[rustfmt::skip]
const PRINTED: [(u16, Nas, Graha, Graha, Graha); 18] = {
    const fn row(n: u16, sign: i64, d: i64, m: i64, s: i64, lords: (Graha, Graha, Graha)) -> (u16, Nas, Graha, Graha, Graha) {
        let at = sign * Nas::PER_SIGN + d * Nas::PER_DEGREE + m * Nas::PER_ARCMINUTE + s * Nas::PER_ARCSECOND;
        (n, Nas::new(at), lords.0, lords.1, lords.2)
    }
    [
        row(1, 0, 0, 0, 0, (Mars, Ketu, Ketu)),
        row(48, 2, 8, 40, 0, (Mercury, Rahu, Jupiter)),
        row(74, 3, 14, 53, 20, (Moon, Saturn, Jupiter)),
        row(207, 9, 27, 53, 20, (Saturn, Mars, Saturn)),
        row(208, 10, 0, 0, 0, (Saturn, Mars, Mercury)),
        row(228, 10, 29, 26, 40, (Saturn, Jupiter, Moon)),
        row(229, 11, 0, 0, 0, (Jupiter, Jupiter, Moon)),
        row(238, 11, 12, 6, 40, (Jupiter, Saturn, Mars)),
        row(239, 11, 12, 53, 20, (Jupiter, Saturn, Rahu)),
        row(240, 11, 14, 53, 20, (Jupiter, Saturn, Jupiter)),
        row(241, 11, 16, 40, 0, (Jupiter, Mercury, Mercury)),
        row(242, 11, 18, 33, 20, (Jupiter, Mercury, Ketu)),
        row(243, 11, 19, 20, 0, (Jupiter, Mercury, Venus)),
        row(244, 11, 21, 33, 20, (Jupiter, Mercury, Sun)),
        row(245, 11, 22, 13, 20, (Jupiter, Mercury, Moon)),
        row(246, 11, 23, 20, 0, (Jupiter, Mercury, Mars)),
        row(247, 11, 24, 6, 40, (Jupiter, Mercury, Rahu)),
        row(248, 11, 26, 6, 40, (Jupiter, Mercury, Jupiter)),
    ]
};

#[test]
fn the_generated_table_is_the_printed_one() {
    for (n, start, sign, star, sub) in PRINTED {
        let number = KpNumber::new(n).unwrap();
        assert_eq!(number.start(), start, "{n}");
        let lords = number.lords();
        assert_eq!(
            (lords.sign, lords.star.lord, lords.sub.lord),
            (sign, star, sub),
            "{n}"
        );
    }
    // The Reader's last row, 249, closes the zodiac at 30° Pisces.
    let last = KpNumber::new(249).unwrap();
    assert_eq!(last.start(), at(11, 27, 53, 20));
    assert_eq!(last.span().end, Nas::ZERO);
    assert_eq!(last.lords().sub.lord, Graha::Saturn);
}

/// Six subs straddle a sign's end, each counted twice: Rahu's in the Sun's
/// three stars and the Moon's in Jupiter's three. Every other number is
/// one whole sub.
#[test]
fn a_sign_s_end_splits_six_subs_and_opens_every_sign() {
    let mut split = Vec::new();
    let mut opens_a_sign = Vec::new();
    for number in KpNumber::all() {
        let start = number.start();
        let lords = number.lords();
        if start.in_sign() == Nas::ZERO {
            opens_a_sign.push(number.get());
        }
        if lords.sub.span == number.span() {
            continue;
        }
        // Either half of a sub a sign's end cuts: the second opens the sign.
        if lords.sub.span.start == start {
            assert_eq!(number.span().end.in_sign(), Nas::ZERO, "{number}");
        } else {
            assert_eq!(start.in_sign(), Nas::ZERO, "{number}");
            split.push((number.get(), lords.star.lord, lords.sub.lord));
        }
    }
    assert_eq!(
        opens_a_sign,
        [1, 23, 42, 63, 84, 106, 125, 146, 167, 189, 208, 229]
    );
    assert_eq!(
        split,
        [
            (23, Sun, Rahu),
            (63, Jupiter, Moon),
            (106, Sun, Rahu),
            (146, Jupiter, Moon),
            (189, Sun, Rahu),
            (229, Jupiter, Moon),
        ]
    );
}

/// The numbers tile the circle without a gap, each keeping its lords from
/// its first nanoarcsecond to its last, and a longitude finds its number
/// on either side of each edge.
#[test]
fn the_numbers_tile_the_circle_and_are_found_from_either_side() {
    assert_eq!(KpNumber::all().len(), 249);
    let mut previous: Option<KpNumber> = None;
    for number in KpNumber::all() {
        let span = number.span();
        if let Some(previous) = previous {
            assert_eq!(previous.span().end, span.start);
            assert_eq!(KpNumber::of(span.start - ONE), previous);
        }
        assert_eq!(KpNumber::of(span.start), number);
        let last = span.end - ONE;
        assert_eq!(KpNumber::of(last), number);
        let (first, end) = (lords(span.start), lords(last));
        assert_eq!(
            (first.sign, first.star.lord, first.sub.lord),
            (end.sign, end.star.lord, end.sub.lord),
            "{number}"
        );
        previous = Some(number);
    }
    assert_eq!(KpNumber::of(Nas::ZERO - ONE).get(), 249);
}

#[test]
fn a_number_outside_the_table_is_refused_by_its_field() {
    for bad in [0, 250, 256, 1800] {
        let error = KpNumber::new(bad).unwrap_err();
        assert_eq!(error.field(), Some("number"), "{bad}");
        assert!(error.to_string().contains(&bad.to_string()), "{error}");
    }
    let number: KpNumber = serde_json::from_str("74").unwrap();
    assert_eq!(serde_json::to_string(&number).unwrap(), "74");
    assert!(serde_json::from_str::<KpNumber>("250").is_err());
    assert_eq!(format!("{number} {number:?}"), "74 KpNumber(74)");
}

/// A cheap deterministic spread of longitudes over the circle, the edges
/// of every sub among them.
fn longitudes() -> impl Iterator<Item = Nas> {
    let mut state = 0x9E37_79B9_7F4A_7C15_u64;
    let drawn = std::iter::repeat_with(move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        Nas::new(i64::try_from(state % u64::try_from(Nas::CIRCLE).unwrap()).unwrap())
    });
    KpNumber::all()
        .flat_map(|number| [number.start(), number.start() - ONE])
        .chain(drawn.take(4_000))
}

/// The star is the catalogue's Vimshottari lord of the nakshatra, the
/// sub's width is its lord's years × 6′40″ exactly, and each level holds
/// the longitude inside its parent.
#[test]
fn every_level_holds_the_longitude_inside_its_parent() {
    for at in longitudes() {
        let deep = chain::<{ MAX_LEVELS }>(at);
        let star = deep.levels()[0];
        assert_eq!(star.lord, at.nakshatra().attributes().vimshottari_lord);
        assert_eq!(star.span.width(), Nas::PER_NAKSHATRA);
        let sub = deep.levels()[1];
        let years = VIMSHOTTARI_LORDS
            .iter()
            .find(|lord| lord.graha == sub.lord)
            .unwrap()
            .years;
        assert_eq!(sub.span.width(), i64::from(years) * 400_000_000_000);
        for pair in deep.levels().windows(2) {
            let (parent, child) = (pair[0].span, pair[1].span);
            assert!(child.contains(at), "{at:?}: {child:?}");
            assert!(parent.contains(child.start), "{at:?}");
            assert!(child.width() <= parent.width());
        }
        // A shallower chain is the deeper one's first levels.
        let lords = lords(at);
        assert_eq!(
            [lords.star, lords.sub, lords.sub_sub],
            deep.levels()[..3],
            "{at:?}"
        );
        assert_eq!(chain::<2>(at).levels()[..], deep.levels()[..2]);
        assert_eq!(lords.sign, at.sign().attributes().lord);
    }
}

/// Each level's parts start from its own lord and fill it: the sub-subs
/// of Ashwini's Venus sub are the nine from Venus, in order, edge to edge.
#[test]
fn a_level_s_nine_parts_run_from_its_own_lord_and_fill_it() {
    let sub = lords(Nas::new(Nas::PER_DEGREE)).sub;
    assert_eq!(sub.lord, Graha::Venus);
    let mut at = sub.span.start;
    let mut seen = Vec::new();
    while at != sub.span.end {
        let sub_sub = lords(at).sub_sub;
        assert_eq!(sub_sub.span.start, at);
        seen.push(sub_sub.lord);
        at = sub_sub.span.end;
    }
    assert_eq!(
        seen,
        [Venus, Sun, Moon, Mars, Rahu, Jupiter, Saturn, Mercury, Ketu]
    );
}

/// A sub-sub's edge that falls between two nanoarcseconds is decided in
/// integers: Ashwini's first sub-sub (Ketu's 7 of Ketu's 7) ends at
/// 46′40″ × 7/120, 163 333 333 333⅓ nanoarcseconds, so the last longitude
/// inside it is …333 and the first past it …334.
#[test]
fn an_edge_between_two_nanoarcseconds_is_decided_exactly() {
    let edge = 163_333_333_333;
    assert_eq!(lords(Nas::new(edge)).sub_sub.lord, Graha::Ketu);
    assert_eq!(lords(Nas::new(edge + 1)).sub_sub.lord, Graha::Venus);
    assert_eq!(lords(Nas::ZERO).sub_sub.span.end, Nas::new(edge + 1));
    assert_eq!(
        lords(Nas::new(edge + 1)).sub_sub.span.start,
        Nas::new(edge + 1)
    );
}

/// The margin is the arc to the nearer edge, and the last sub of Revati
/// wraps its end to 0° Aries.
#[test]
fn the_margin_is_the_arc_to_the_nearer_edge_across_zero_aries() {
    let at = at(2, 9, 0, 0);
    let sub = lords(at).sub.span;
    assert_eq!(sub.margin(at), 20 * Nas::PER_ARCMINUTE);
    assert_eq!(sub.margin(sub.start), 0);
    assert_eq!(sub.margin(sub.end - ONE), 1);
    let last = lords(Nas::ZERO - ONE).sub.span;
    assert_eq!(last.end, Nas::ZERO);
    assert!(last.contains(Nas::ZERO - ONE));
    assert!(!last.contains(Nas::ZERO));
    assert_eq!(last.margin(Nas::ZERO - ONE), 1);
}

/// The lords of a nakshatra's start are its own: star and sub alike.
#[test]
fn every_nakshatra_opens_with_its_own_lord_at_every_level() {
    for nakshatra in Nakshatra::ALL {
        let start = Nas::new(i64::from(nakshatra.id()) * Nas::PER_NAKSHATRA);
        let chain = chain::<5>(start);
        let lord = nakshatra.attributes().vimshottari_lord;
        assert!(
            chain
                .levels()
                .iter()
                .all(|level| level.lord == lord && level.span.start == start),
            "{nakshatra:?}"
        );
    }
}

#[test]
fn a_chain_serialises_as_its_levels_and_lords_by_name() {
    let json = serde_json::to_value(lords(Nas::ZERO)).unwrap();
    assert_eq!(json["sign"], "MARS");
    assert_eq!(json["subSub"]["lord"], "KETU");
    let chain = serde_json::to_value(chain::<2>(Nas::ZERO)).unwrap();
    assert_eq!(
        chain[1]["span"]["end"],
        46 * Nas::PER_ARCMINUTE + 40 * Nas::PER_ARCSECOND
    );
}

/// A planet is in the house whose cusp it follows, whatever sign the cusp
/// is in, and on a cusp it is in that cusp's house.
#[test]
fn a_planet_is_in_the_house_whose_cusp_it_follows() {
    // Unequal cusps, as Placidus's are, the 1st at 25° Pisces.
    let degrees = [355, 20, 48, 80, 110, 140, 175, 200, 228, 260, 290, 320];
    let cusps = degrees.map(|deg| Nas::new(deg * Nas::PER_DEGREE));
    for (at, house) in [
        (355, 1),
        (0, 1),
        (19, 1),
        (20, 2),
        (47, 2),
        (330, 12),
        (354, 12),
    ] {
        assert_eq!(
            house_of(&cusps, Nas::new(at * Nas::PER_DEGREE)),
            house,
            "{at}°"
        );
    }
    assert_eq!(house_of(&cusps, cusps[1] - ONE), 1);
    let chart = KpChart::new(
        teistro_core::catalogue::HouseSystem::Placidus,
        cusps,
        [
            Position::new(Saturn, Nas::new(100 * Nas::PER_DEGREE)).retrograde(true),
            Position::new(Moon, cusps[6]),
        ],
    );
    let saturn = chart.planet(Saturn).unwrap();
    assert_eq!((saturn.house, saturn.retrograde), (4, true));
    assert_eq!(saturn.lords, lords(saturn.longitude));
    assert_eq!(chart.planet(Moon).unwrap().house, 7);
    assert!(chart.planet(Sun).is_none());
    // Every cusp numbered and read.
    for house in 1..=12 {
        let cusp = chart.cusp(house).unwrap();
        assert_eq!(cusp.house, house);
        assert_eq!(cusp.lords, lords(cusp.longitude));
    }
    assert!(chart.cusp(0).is_none() && chart.cusp(13).is_none());
}

/// Equal cusps from 0° Aries, so each house is its sign, and eight planets
/// placed so every level of the 1st house has something in it.
fn worked(node_aspects: teistro_core::settings::NodeAspects) -> Significators {
    let cusps =
        std::array::from_fn(|house| Nas::new(30 * i64::try_from(house).unwrap() * Nas::PER_DEGREE));
    let at = |degrees: i64| Nas::new(degrees * Nas::PER_DEGREE);
    let chart = KpChart::new(
        teistro_core::catalogue::HouseSystem::Placidus,
        cusps,
        [
            // Aries 10°, Ashwini (Ketu's star): the 1st.
            Position::new(Sun, at(10)),
            Position::new(Rahu, at(10)).retrograde(true),
            // Libra 10°, Swati (Rahu's): the 7th.
            Position::new(Ketu, at(190)).retrograde(true),
            // Taurus 20°, Rohini (the Moon's own): the 2nd.
            Position::new(Moon, at(50)),
            // Cancer 5°, Pushya (Saturn's): the 4th, the 1st's lord.
            Position::new(Mars, at(95)),
            // Sagittarius 5°, Mula (Ketu's): the 9th.
            Position::new(Jupiter, at(245)),
            // Libra 20°, Vishakha (Jupiter's): the 7th.
            Position::new(Saturn, at(200)),
            // Capricorn 15°, Shravana (the Moon's): the 10th.
            Position::new(Venus, at(285)),
        ],
    );
    Significators::of(&chart, node_aspects)
}

/// The Reader's four levels and the two beyond them, worked by hand.
#[test]
fn a_house_is_signified_level_by_level_as_the_reader_orders_it() {
    use teistro_core::settings::NodeAspects;
    let significators = worked(NodeAspects::None);
    let first = significators.house(1).unwrap();
    // (b) the Sun and Rahu stand in it; (a) Ketu is in Swati, Rahu's star.
    assert_eq!(first.occupants, [Sun, Rahu]);
    assert_eq!(first.in_occupants_stars, [Ketu]);
    // (d) Mars owns Aries; (c) nothing stands in Mars's stars.
    assert_eq!(first.lord, Mars);
    assert!(first.in_lords_star.is_empty(), "{:?}", first.in_lords_star);
    assert_eq!(first.in_order(), [Ketu, Sun, Rahu, Mars]);
    // (e) Saturn shares Libra with Ketu; (f) Mars's 7th reaches Venus in
    // Capricorn, and Saturn, already conjoined, is not counted twice.
    assert_eq!(first.conjoined, [Saturn]);
    assert_eq!(first.aspected, [Venus]);
    // Jupiter is aspected only by Saturn, who is (e) and not a level.
    assert!(!first.aspected.contains(&Jupiter));

    // The inverse: the Sun is in Ketu's star, Ketu is in the 7th, and the
    // Sun owns Leo, the 5th.
    let sun = significators.signified_by(Sun);
    assert_eq!(
        (sun.by_star, sun.occupies, sun.by_lords_star, sun.owns),
        (vec![7], vec![1], vec![], vec![5])
    );
    // Mars owns the 1st and the 8th, and the Sun's and Rahu's stars are
    // not Mars's, so nothing reaches him through (c) of another house.
    assert_eq!(significators.signified_by(Mars).owns, [1, 8]);
    assert!(
        significators.signified_by(Rahu).by_lords_star.is_empty(),
        "{:?}",
        significators.signified_by(Rahu).by_lords_star
    );
}

/// Reader VI: a node gives the results of the planets it is conjoined
/// with, then of its star's lord, then of the planets aspecting it, and
/// last of its sign's lord.
#[test]
fn a_node_gives_results_in_the_reader_s_order() {
    use teistro_core::settings::NodeAspects;
    let significators = worked(NodeAspects::None);
    let rahu = significators.node(Rahu).unwrap();
    assert_eq!(rahu.conjoined, [Sun]);
    assert_eq!(rahu.star_lord, Ketu);
    // Jupiter's 5th and Saturn's 7th reach Aries; Ketu opposite is not an
    // agency.
    assert_eq!(rahu.aspecting, [Jupiter, Saturn]);
    assert_eq!(rahu.sign_lord, Mars);
    assert_eq!(rahu.in_order(), [Sun, Ketu, Jupiter, Saturn, Mars]);
    let ketu = significators.node(Ketu).unwrap();
    // Saturn shares Libra; Swati is Rahu's; the Sun's 7th and Mars's 4th
    // reach Libra; Venus owns it.
    assert_eq!(ketu.in_order(), [Saturn, Rahu, Sun, Mars, Venus]);
    assert!(significators.node(Sun).is_none());
}

/// A node's own aspects are the settings': under `FIVE_SEVEN_NINE` Rahu in
/// Aries reaches Jupiter in Sagittarius, the 9th, and Jupiter joins the
/// 1st's (f).
#[test]
fn a_node_s_aspects_follow_the_settings() {
    use teistro_core::settings::NodeAspects;
    assert!(
        !worked(NodeAspects::None)
            .house(1)
            .unwrap()
            .aspected
            .contains(&Jupiter)
    );
    assert!(
        worked(NodeAspects::FiveSevenNine)
            .house(1)
            .unwrap()
            .aspected
            .contains(&Jupiter)
    );
}

/// Reader III: a sign between two cusps touching neither is intercepted,
/// reported with its house and never a level.
#[test]
fn a_sign_touching_no_cusp_is_intercepted_in_the_house_holding_it() {
    use teistro_core::catalogue::Rashi;
    use teistro_core::settings::NodeAspects;
    let degrees = [0, 25, 70, 100, 130, 160, 180, 205, 250, 280, 310, 340];
    let cusps = degrees.map(|deg| Nas::new(deg * Nas::PER_DEGREE));
    let chart = KpChart::new(teistro_core::catalogue::HouseSystem::Placidus, cusps, []);
    let significators = Significators::of(&chart, NodeAspects::None);
    let intercepted: Vec<(u8, Vec<Rashi>, Vec<Graha>)> = significators
        .houses
        .iter()
        .filter(|house| !house.intercepted.is_empty())
        .map(|house| {
            (
                house.house,
                house.intercepted.clone(),
                house.intercepted_lords(),
            )
        })
        .collect();
    assert_eq!(
        intercepted,
        [
            (2, vec![Rashi::Taurus], vec![Venus]),
            (8, vec![Rashi::Scorpio], vec![Mars]),
        ]
    );
    // Two cusps in one sign do not make its neighbour intercepted twice,
    // and the lord of a house is its cusp's: the 8th opens in Libra.
    assert_eq!(significators.house(8).unwrap().lord, Venus);
}

/// KP Reader VI's worked horary (the third child's house): a Sunday, the
/// Moon in Sagittarius in Venus's star, the lagna in Libra, Jupiter and
/// Ketu together in the 12th in the Sun's stars, and Rahu in Pisces in
/// Jupiter's. The Reader's text leaves the lagna's star unnamed; it is
/// Jupiter's Vishakha, the one reading under which Rahu joins as an agent
/// rather than as a ruler outright.
fn readers_horary(retrograde: &[Graha]) -> KpChart {
    let cusps = std::array::from_fn(|house| {
        Nas::new((205 + 30 * i64::try_from(house).unwrap()) * Nas::PER_DEGREE)
    });
    let at = |degrees: i64| Nas::new(degrees * Nas::PER_DEGREE);
    let planets = [
        (Sun, 10),
        (Moon, 260),
        (Mars, 60),
        (Mercury, 200),
        (Jupiter, 155),
        (Venus, 230),
        (Saturn, 100),
        (Rahu, 331),
        (Ketu, 151),
    ]
    .map(|(graha, degrees)| {
        Position::new(graha, at(degrees)).retrograde(retrograde.contains(&graha))
    });
    KpChart::new(
        teistro_core::catalogue::HouseSystem::Placidus,
        cusps,
        planets,
    )
}

#[test]
fn the_reader_s_ruling_planets_are_reproduced() {
    use teistro_core::settings::NodeRulers;
    let chart = readers_horary(&[]);
    let ruling = RulingPlanets::of(&chart, Sun, RulingRules::READER);
    assert_eq!(ruling.accepted(), [Jupiter, Venus, Sun, Rahu, Ketu]);
    assert_eq!(
        ruling.ruler(Jupiter).unwrap().reasons,
        [Reason::LagnaStar, Reason::MoonSign]
    );
    assert_eq!(
        ruling.ruler(Rahu).unwrap().reasons,
        [Reason::Agent {
            of: Jupiter,
            by: Agency::InItsSign
        }]
    );
    assert_eq!(
        ruling.ruler(Ketu).unwrap().reasons,
        [Reason::Agent {
            of: Jupiter,
            by: Agency::Conjoined
        }]
    );
    // Under the sign alone Ketu, in Mercury's Virgo, is no agent.
    let sign = RulingRules {
        node_rulers: NodeRulers::Sign,
        ..RulingRules::READER
    };
    assert_eq!(
        RulingPlanets::of(&chart, Sun, sign).accepted(),
        [Jupiter, Venus, Sun, Rahu]
    );
}

/// Seven under the baseline engine's count: the lagna's sub at Libra 25°
/// is Mercury's and the Moon's at Sagittarius 20° Rahu's, so Rahu rules
/// outright and only Ketu is an agent.
#[test]
fn with_subs_the_lagna_s_and_the_moon_s_sub_lords_rule_too() {
    use teistro_core::settings::RulingCount;
    let rules = RulingRules {
        count: RulingCount::WithSubs,
        ..RulingRules::READER
    };
    let ruling = RulingPlanets::of(&readers_horary(&[]), Sun, rules);
    assert_eq!(
        ruling.accepted(),
        [Jupiter, Venus, Mercury, Rahu, Sun, Ketu]
    );
    assert_eq!(ruling.ruler(Mercury).unwrap().reasons, [Reason::LagnaSub]);
    assert_eq!(ruling.ruler(Rahu).unwrap().reasons, [Reason::MoonSub]);
}

/// A ruler in a retrograde planet's star is rejected; one in a retrograde
/// planet's sub only under `STAR_OR_SUB`, and that reach is reported
/// either way. A ruler's own retrogression is delay, not rejection, and a
/// node's motion rejects nobody.
#[test]
fn a_retrograde_planet_rejects_by_its_star_and_under_the_other_reading_its_sub() {
    use teistro_core::settings::RetrogradeRejection;
    // Venus at Scorpio 20° is in Jyeshtha, Mercury's; the Sun at Aries 10°
    // is in Ketu's Ashwini and Saturn's sub.
    let chart = readers_horary(&[Mercury, Saturn, Venus, Rahu, Ketu]);
    let ruling = RulingPlanets::of(&chart, Sun, RulingRules::READER);
    let venus = ruling.ruler(Venus).unwrap();
    assert_eq!(
        venus.rejected_by,
        Some(Rejection {
            retrograde: Mercury,
            by_star: true
        })
    );
    assert!(venus.retrograde);
    let sun = ruling.ruler(Sun).unwrap();
    assert_eq!(sun.rejected_by, None);
    assert_eq!(
        sun.rejected_by_sub,
        Some(Rejection {
            retrograde: Saturn,
            by_star: false
        })
    );
    // Jupiter stands in the Sun's star and Rahu in Jupiter's: neither is
    // rejected, and a node is never retrograde as a ruler.
    assert_eq!(ruling.accepted(), [Jupiter, Sun, Rahu, Ketu]);
    assert!(!ruling.ruler(Rahu).unwrap().retrograde);

    let star_or_sub = RulingRules {
        retrograde_rejection: RetrogradeRejection::StarOrSub,
        ..RulingRules::READER
    };
    // Jupiter at Virgo 5° is in Saturn's sub too (153° to 155°06′40″), so
    // the sub reading rejects it with the Sun.
    let ruling = RulingPlanets::of(&chart, Sun, star_or_sub);
    assert_eq!(ruling.accepted(), [Rahu, Ketu]);
    assert_eq!(
        ruling
            .ruler(Jupiter)
            .unwrap()
            .rejected_by
            .map(|r| r.retrograde),
        Some(Saturn)
    );
}
