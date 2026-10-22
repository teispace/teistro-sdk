//! Ekadashi held to *Dharmasindhu*'s twelve examples, pp. 11–12
//! (`03-design/festival-rules.md` §8), read off the page images.
//!
//! Each example is almanac lines: the ghati past a sunrise at which the
//! 10th, the 11th and the 12th end. `60 । 1` fills a day and ends a
//! ghati past the next sunrise; *kshaya* ends before any sunrise of its
//! own. The days are the shared synthetic ones, and the 11th's own day
//! (D1) is day 2, the 3rd of January; the expected day is the text's.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index fixed lists"
)]

mod common;

use common::{days_over, ghati};
use teistro_calendar::lunisolar::MonthKind;
use teistro_core::catalogue::{Masa, Tithi};
use teistro_core::interval::Interval;
use teistro_panchanga::festival::{
    DayPart, EkadashiFast, EkadashiRule, Excess, MoonArc, MoonArcs, NakshatraBar, Parana,
    ParanaEdge, ParanaRule, Vedha, Which, Window, ekadashis,
};

use Which::{Earlier, Later};

/// Where a tithi ends: `(day, ghati)` past that day's sunrise, D1 day 2.
type End = (usize, f64);

/// One example: where the 10th, the 11th and the 12th end.
struct Example {
    name: &'static str,
    ends: [End; 3],
    pierced_at: Option<Vedha>,
    excess: Excess,
    /// The text's day for the Vaishnava, the Smarta householder and the
    /// renunciant.
    days: [Which; 3],
}

/// The 9th ends on day 0; the 10th, the 11th and the 12th where the
/// example puts them.
fn fasts(ends: [End; 3]) -> Vec<EkadashiFast> {
    fasts_under(ends, &EkadashiRule::dharmasindhu())
}

/// [`fasts`] under the given rules.
fn fasts_under(ends: [End; 3], rules: &[EkadashiRule]) -> Vec<EkadashiFast> {
    let at = |(day, ghatis): End| ghati(day, ghatis);
    let tithis = [
        (Tithi::KrishnaNavami, ghati(0, 50.0)),
        (Tithi::KrishnaDashami, at(ends[0])),
        (Tithi::KrishnaEkadashi, at(ends[1])),
        (Tithi::KrishnaDwadashi, at(ends[2])),
        (Tithi::KrishnaTrayodashi, ghati(5, 40.0)),
        (Tithi::KrishnaChaturdashi, ghati(6, 30.0)),
    ];
    let days = days_over(6, &tithis, &[], Masa::Kartika, MonthKind::Nija);
    let answer = ekadashis(rules, &days).unwrap();
    assert!(answer.unjudged.is_empty(), "{:?}", answer.unjudged);
    answer.ekadashis
}

const EXAMPLES: [Example; 12] = [
    // p. 11, the Vaishnavas' four, each pure.
    Example {
        name: "V1",
        ends: [(1, 55.0), (3, 1.0), (3, 58.0)],
        pierced_at: None,
        excess: Excess::Eleventh,
        days: [Later, Earlier, Later],
    },
    Example {
        name: "V2",
        ends: [(1, 55.0), (2, 58.0), (4, 1.0)],
        pierced_at: None,
        excess: Excess::Twelfth,
        days: [Later, Earlier, Earlier],
    },
    Example {
        name: "V3",
        ends: [(1, 55.0), (3, 1.0), (4, 5.0)],
        pierced_at: None,
        excess: Excess::Both,
        days: [Later, Later, Later],
    },
    Example {
        name: "V4",
        ends: [(1, 55.0), (2, 57.0), (3, 58.0)],
        pierced_at: None,
        excess: Excess::Neither,
        days: [Earlier, Earlier, Earlier],
    },
    // pp. 11–12, the Smartas' eight.
    Example {
        name: "S1",
        ends: [(1, 58.0), (3, 1.0), (3, 58.0)],
        pierced_at: Some(Vedha::Arunodaya),
        excess: Excess::Eleventh,
        days: [Later, Earlier, Later],
    },
    Example {
        name: "S2",
        ends: [(2, 4.0), (3, 2.0), (3, 58.0)],
        pierced_at: Some(Vedha::Sunrise),
        excess: Excess::Eleventh,
        days: [Later, Earlier, Later],
    },
    Example {
        name: "S3",
        ends: [(1, 58.0), (3, 1.0), (4, 4.0)],
        pierced_at: Some(Vedha::Arunodaya),
        excess: Excess::Both,
        days: [Later, Later, Later],
    },
    Example {
        name: "S4",
        ends: [(2, 2.0), (3, 3.0), (4, 4.0)],
        pierced_at: Some(Vedha::Sunrise),
        excess: Excess::Both,
        days: [Later, Later, Later],
    },
    Example {
        name: "S5",
        ends: [(1, 58.0), (2, 59.0), (4, 1.0)],
        pierced_at: Some(Vedha::Arunodaya),
        excess: Excess::Twelfth,
        // Madhava; Hemadri gives all the later, and the closing says the
        // learned give neither two fasts nor one later for all (C182).
        days: [Later, Earlier, Earlier],
    },
    Example {
        name: "S6",
        ends: [(2, 1.0), (2, 58.0), (4, 1.0)],
        pierced_at: Some(Vedha::Sunrise),
        excess: Excess::Twelfth,
        days: [Later, Later, Later],
    },
    Example {
        name: "S7",
        ends: [(1, 57.0), (2, 58.0), (3, 59.0)],
        pierced_at: Some(Vedha::Arunodaya),
        excess: Excess::Neither,
        // Renunciants are not named here, so they keep the Smarta day.
        days: [Later, Earlier, Earlier],
    },
    Example {
        name: "S8",
        ends: [(2, 2.0), (2, 56.0), (3, 59.0)],
        pierced_at: Some(Vedha::Sunrise),
        excess: Excess::Neither,
        days: [Later, Earlier, Later],
    },
];

const KEYS: [&str; 3] = [
    "EKADASHI_VAISHNAVA",
    "EKADASHI_SMARTA",
    "EKADASHI_SMARTA_RENUNCIANT",
];

#[test]
fn each_of_the_twelve_examples_takes_the_text_s_day_under_each_rule() {
    for example in &EXAMPLES {
        let found = fasts(example.ends);
        let keys: Vec<&str> = found.iter().map(|fast| fast.rule.as_str()).collect();
        assert_eq!(keys, KEYS, "{}: one fast a rule", example.name);
        for (fast, want) in found.iter().zip(example.days) {
            let name = example.name;
            assert_eq!(
                fast.days.each_ref().map(|day| day.day),
                [3, 4],
                "{name}: D1 is day 2"
            );
            assert_eq!(fast.pierced_at, example.pierced_at, "{name}");
            assert_eq!(fast.excess, example.excess, "{name}");
            assert_eq!(fast.choice, want, "{name} {}", fast.rule);
            let day = match want {
                Earlier => 3,
                Later => 4,
            };
            assert_eq!(fast.day.day, day, "{name} {}", fast.rule);
            assert_eq!(
                (fast.tithi, fast.month),
                (Tithi::KrishnaEkadashi, Masa::Kartika)
            );
        }
    }
}

#[test]
fn hemadri_s_view_is_one_cell_away() {
    // C182: V2 and S5, pure with the 12th alone in excess, take the later
    // day for all in Hemadri's view.
    let mut hemadri = EkadashiRule::dharmasindhu().remove(1);
    hemadri.key = "EKADASHI_SMARTA_HEMADRI".to_owned();
    hemadri.table.pure.twelfth = Later;
    let tithis = |ends: [End; 3]| {
        [
            (Tithi::KrishnaNavami, ghati(0, 50.0)),
            (Tithi::KrishnaDashami, ghati(ends[0].0, ends[0].1)),
            (Tithi::KrishnaEkadashi, ghati(ends[1].0, ends[1].1)),
            (Tithi::KrishnaDwadashi, ghati(ends[2].0, ends[2].1)),
            (Tithi::KrishnaTrayodashi, ghati(5, 40.0)),
        ]
    };
    for name in ["V2", "S5"] {
        let example = EXAMPLES.iter().find(|e| e.name == name).unwrap();
        let days = days_over(
            6,
            &tithis(example.ends),
            &[],
            Masa::Kartika,
            MonthKind::Nija,
        );
        let answer = ekadashis(std::slice::from_ref(&hemadri), &days).unwrap();
        assert_eq!(answer.ekadashis[0].choice, Later, "{name}");
    }
}

#[test]
fn an_eleventh_without_its_neighbours_is_unjudged_by_each_rule() {
    // The 11th is the first tithi the days hold: no 10th to pierce it.
    let tithis = [
        (Tithi::KrishnaEkadashi, ghati(1, 30.0)),
        (Tithi::KrishnaDwadashi, ghati(2, 30.0)),
        (Tithi::KrishnaTrayodashi, ghati(3, 30.0)),
    ];
    let days = days_over(6, &tithis, &[], Masa::Kartika, MonthKind::Nija);
    let answer = ekadashis(&EkadashiRule::dharmasindhu(), &days).unwrap();
    assert!(answer.ekadashis.is_empty(), "{:?}", answer.ekadashis);
    let keys: Vec<&str> = answer.unjudged.iter().map(|u| u.rule.as_str()).collect();
    assert_eq!(keys, KEYS);
}

#[test]
fn a_rule_without_a_key_is_refused_by_it() {
    let mut keyless = EkadashiRule::dharmasindhu().remove(0);
    keyless.key.clear();
    let error = ekadashis(&[keyless], &[]).unwrap_err();
    assert_eq!(error.field(), Some("key"));
}

/// The margin is the least time between an edge of the three tithis and
/// an instant the fast was decided from, so the Vaishnava's arunodaya
/// counts for that rule alone: the 10th ends 1½ ghatis past arunodaya
/// (ghati 56) and 2½ before the 11th's sunrise, and every other edge is
/// further from every instant.
#[test]
fn each_rule_s_margin_is_its_own_nearest_instant() {
    let found = fasts([(1, 57.5), (3, 5.0), (4, 10.0)]);
    let ghati_seconds = 86_400.0 / 60.0;
    for fast in &found {
        let expected = if fast.rule == "EKADASHI_VAISHNAVA" {
            1.5
        } else {
            2.5
        } * ghati_seconds;
        assert!(
            (fast.margin_seconds - expected).abs() < 1e-3,
            "{}: {}",
            fast.rule,
            fast.margin_seconds
        );
    }
    assert_eq!(found.len(), KEYS.len());
}

/// A fast's parana under a rule, by key.
fn parana<'a>(found: &'a [EkadashiFast], key: &str) -> &'a Parana {
    found
        .iter()
        .find(|fast| fast.rule == key)
        .and_then(|fast| fast.parana.as_ref())
        .unwrap()
}

/// The window kept: its day (D1 is the 3rd), its start and end as
/// `(day, ghati)` and the clause that set each, and the rule's window.
fn kept(parana: &Parana) -> (u8, End, End, ParanaEdge, ParanaEdge, Window) {
    let kept = parana.kept.unwrap();
    let end = |jd: f64| {
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a few days"
        )]
        let day = jd.floor() as usize;
        (day, ((jd - jd.floor()) * 60.0 * 1e6).round() / 1e6)
    };
    (
        parana.day.day,
        end(kept.span.from.get()),
        end(kept.span.to.get()),
        kept.from_by,
        kept.to_by,
        kept.at,
    )
}

const PRATAH: Window = Window::Part {
    part: DayPart::Pratah,
};
const APARAHNA: Window = Window::Part {
    part: DayPart::Aparahna,
};

/// *Dharmasindhu* p. 19 over the twelve examples' kinds of day: the meal
/// the day after the fast, inside the 12th and past its first quarter, in
/// the morning's fifth (ghatis 0 to 6 of a daylight of thirty), else the
/// afternoon's (18 to 24).
#[test]
fn the_meal_is_kept_in_the_morning_past_the_hari_vasara_else_the_afternoon() {
    use ParanaEdge::{HariVasara, TithiEnd, Window as Edge};
    // V1: the 12th from day 3's ghati 1 to its 58th, so its first quarter
    // ends at ghati 15¼, past the morning: the Smarta's meal is the
    // afternoon's. The Vaishnava fasts day 3, and the 12th is gone by
    // day 4's sunrise: the meal is the 13th's morning.
    let found = fasts([(1, 55.0), (3, 1.0), (3, 58.0)]);
    let smarta = parana(&found, "EKADASHI_SMARTA");
    assert_eq!(smarta.tithi, Tithi::KrishnaDwadashi);
    assert_eq!(
        kept(smarta),
        (4, (3, 18.0), (3, 24.0), Edge, Edge, APARAHNA)
    );
    assert!((smarta.hari_vasara_ends.unwrap().get() - ghati(3, 15.25)).abs() < 1e-9);
    let vaishnava = parana(&found, "EKADASHI_VAISHNAVA");
    assert_eq!(vaishnava.tithi, Tithi::KrishnaTrayodashi);
    assert_eq!(vaishnava.hari_vasara_ends, None);
    assert_eq!(kept(vaishnava), (5, (4, 0.0), (4, 6.0), Edge, Edge, PRATAH));
    // The 12th ends 3 ghatis into the parana day's morning: the meal
    // before it does.
    let found = fasts([(1, 40.0), (2, 30.0), (3, 3.0)]);
    let smarta = parana(&found, "EKADASHI_SMARTA");
    assert_eq!(
        kept(smarta),
        (4, (3, 0.0), (3, 3.0), Edge, TithiEnd, PRATAH)
    );
    // The 12th from day 2's ghati 50 for fifty ghatis: its quarter ends
    // 2½ ghatis into day 3's morning, which opens the meal.
    let found = fasts([(1, 40.0), (2, 50.0), (3, 40.0)]);
    let smarta = parana(&found, "EKADASHI_SMARTA");
    assert_eq!(
        kept(smarta),
        (4, (3, 2.5), (3, 6.0), HariVasara, Edge, PRATAH)
    );
    // The 11th stands past day 3's sunrise and the 12th's quarter ends at
    // ghati 22¼, inside the afternoon.
    let found = fasts([(1, 40.0), (3, 10.0), (3, 59.0)]);
    let smarta = parana(&found, "EKADASHI_SMARTA");
    assert_eq!(
        kept(smarta),
        (4, (3, 22.25), (3, 24.0), HariVasara, Edge, APARAHNA)
    );
}

/// A rule passing no hari vasara still keeps the meal inside the 12th,
/// from its start; one keeping the morning alone has no window when the
/// hari vasara outlasts it, and says so.
#[test]
fn the_12th_s_start_bars_the_meal_and_a_lost_morning_is_no_window() {
    let rule = |at: Vec<Window>, hari_vasara: bool| {
        let mut smarta = EkadashiRule::dharmasindhu().swap_remove(1);
        smarta.parana = Some(ParanaRule {
            at,
            hari_vasara,
            bars: Vec::new(),
        });
        vec![smarta]
    };
    let all_day = Window::DayMuhurta { from: 1, to: 15 };
    let found = fasts_under(
        [(1, 40.0), (3, 10.0), (3, 59.0)],
        &rule(vec![all_day], false),
    );
    assert_eq!(
        kept(parana(&found, "EKADASHI_SMARTA")),
        (
            4,
            (3, 10.0),
            (3, 30.0),
            ParanaEdge::TithiStart,
            ParanaEdge::Window,
            all_day
        )
    );
    let found = fasts_under([(1, 55.0), (3, 1.0), (3, 58.0)], &rule(vec![PRATAH], true));
    let lost = parana(&found, "EKADASHI_SMARTA");
    assert_eq!(lost.kept, None);
    assert!(lost.hari_vasara_ends.is_some());
}

/// A parana with no window, or with one outside the daylight, is refused
/// by the field that holds it.
#[test]
fn a_parana_without_a_daylight_window_is_refused_by_its_field() {
    let refused = |at: Vec<Window>| {
        let mut rule = EkadashiRule::dharmasindhu().swap_remove(0);
        rule.parana = Some(ParanaRule {
            at,
            hari_vasara: true,
            bars: Vec::new(),
        });
        rule.check().unwrap_err()
    };
    assert_eq!(refused(vec![]).field(), Some("parana.at"));
    assert_eq!(
        refused(vec![PRATAH, Window::Pradosha]).field(),
        Some("parana.at[1]")
    );
    assert_eq!(
        refused(vec![Window::DayMuhurta { from: 4, to: 2 }]).field(),
        Some("parana.at[0].to")
    );
}

/// The bright 11th of Ashadha with its 10th ending at day 1's ghati 40,
/// the 11th at day 2's 30th and the 12th at day 3's 40th, so the Smarta's
/// meal is day 3's morning (the 4th, ghatis 0 to 6); the parana barred by
/// Anuradha's first quarter, whose stay is `barred` on day 3, in `kind`.
fn barred_fast(barred: Option<(f64, f64)>, kind: MonthKind) -> Result<EkadashiFast, String> {
    let tithis = [
        (Tithi::ShuklaNavami, ghati(0, 50.0)),
        (Tithi::ShuklaDashami, ghati(1, 40.0)),
        (Tithi::ShuklaEkadashi, ghati(2, 30.0)),
        (Tithi::ShuklaDwadashi, ghati(3, 40.0)),
        (Tithi::ShuklaTrayodashi, ghati(5, 0.0)),
        (Tithi::ShuklaChaturdashi, ghati(6, 0.0)),
    ];
    let mut days = days_over(6, &tithis, &[], Masa::Ashadha, kind);
    let [bar, ..] = NakshatraBar::dharmasindhu();
    let (from_deg, to_deg) = bar.arc_deg();
    if let Some((from, to)) = barred {
        for (k, day) in days.iter_mut().enumerate() {
            day.moon_arcs = MoonArcs::Found(if k == 3 {
                vec![MoonArc {
                    from_deg,
                    to_deg,
                    span: Interval::literal(from, to),
                }]
            } else {
                vec![]
            });
        }
    }
    let mut smarta = EkadashiRule::dharmasindhu().swap_remove(1);
    smarta.parana.as_mut().unwrap().bars = NakshatraBar::dharmasindhu().to_vec();
    let mut answer = ekadashis(&[smarta], &days).unwrap();
    match answer.unjudged.pop() {
        Some(unjudged) => Err(unjudged.why),
        None => Ok(answer.ekadashis.remove(0)),
    }
}

/// A barred share holding the meal's start opens it at its end; one
/// later in the window closes it at its start (C700).
#[test]
fn a_barred_share_moves_the_meal_past_it_or_before_it() {
    use ParanaEdge::{Nakshatra, Window as Edge};
    let open = barred_fast(None, MonthKind::Nija);
    assert!(open.unwrap_err().contains("Almanac::moon_arcs"));
    let fast = barred_fast(Some((ghati(2, 50.0), ghati(3, 3.0))), MonthKind::Nija).unwrap();
    let parana = fast.parana.as_ref().unwrap();
    assert_eq!(
        kept(parana),
        (4, (3, 3.0), (3, 6.0), Nakshatra, Edge, PRATAH)
    );
    assert!(!parana.barred.unwrap().set_aside);
    let fast = barred_fast(Some((ghati(3, 4.0), ghati(3, 10.0))), MonthKind::Nija).unwrap();
    let parana = fast.parana.as_ref().unwrap();
    assert_eq!(
        kept(parana),
        (4, (3, 0.0), (3, 4.0), Edge, Nakshatra, PRATAH)
    );
    // A stay that misses the daylight bars nothing and is not reported.
    let fast = barred_fast(Some((ghati(3, 31.0), ghati(3, 50.0))), MonthKind::Nija).unwrap();
    let parana = fast.parana.as_ref().unwrap();
    assert_eq!(parana.barred, None);
    assert_eq!(kept(parana), (4, (3, 0.0), (3, 6.0), Edge, Edge, PRATAH));
}

/// A share that leaves no window and outlasts the 12th is set aside, the
/// meal kept as though it were not there (*Kaustubha*); in an adhika
/// Ashadha the bar does not hold at all.
#[test]
fn a_share_outlasting_the_12th_is_set_aside_and_an_adhika_month_bars_nothing() {
    use ParanaEdge::Window as Edge;
    let whole_day = Some((ghati(2, 55.0), ghati(3, 45.0)));
    let fast = barred_fast(whole_day, MonthKind::Nija).unwrap();
    let parana = fast.parana.as_ref().unwrap();
    assert!(parana.barred.unwrap().set_aside);
    assert_eq!(kept(parana), (4, (3, 0.0), (3, 6.0), Edge, Edge, PRATAH));
    let fast = barred_fast(whole_day, MonthKind::Adhika).unwrap();
    let parana = fast.parana.as_ref().unwrap();
    assert_eq!(parana.barred, None);
    // A share ending before the 12th does stands, though it leaves no
    // window: the rule's windows say where the meal may be.
    let fast = barred_fast(Some((ghati(2, 55.0), ghati(3, 35.0))), MonthKind::Nija).unwrap();
    let parana = fast.parana.as_ref().unwrap();
    assert!(!parana.barred.unwrap().set_aside);
    assert_eq!(parana.kept, None);
}

/// A bar's JSON names its share only when it is not the whole, and a bar
/// that names a tithi other than an 11th, or a share that does not exist,
/// is refused by its field.
#[test]
fn a_bar_reads_and_writes_its_share_and_refuses_one_that_does_not_exist() {
    let [ashadha, ..] = NakshatraBar::dharmasindhu();
    let json = serde_json::to_string(&ashadha).unwrap();
    assert_eq!(
        json,
        r#"{"tithi":"SHUKLA_EKADASHI","month":"ASHADHA","nakshatra":"ANURADHA","portion":{"parts":4,"which":1}}"#
    );
    assert_eq!(
        serde_json::from_str::<NakshatraBar>(&json).unwrap(),
        ashadha
    );
    let [whole, ..] = NakshatraBar::verse();
    let json = serde_json::to_string(&whole).unwrap();
    assert!(!json.contains("portion"), "{json}");
    let (from, to) = whole.arc_deg();
    assert!((to - from - 360.0 / 27.0).abs() < 1e-12 && (from - 16.0 * 360.0 / 27.0).abs() < 1e-12);
    let refused = |edit: fn(&mut NakshatraBar)| {
        let mut bar = ashadha;
        edit(&mut bar);
        let mut rule = EkadashiRule::dharmasindhu().swap_remove(1);
        rule.parana.as_mut().unwrap().bars = vec![ashadha, bar];
        rule.check().unwrap_err().field().map(str::to_owned)
    };
    assert_eq!(
        refused(|bar| bar.tithi = Tithi::ShuklaDwadashi).as_deref(),
        Some("parana.bars[1].tithi")
    );
    assert_eq!(
        refused(|bar| bar.portion.parts = 13).as_deref(),
        Some("parana.bars[1].portion.parts")
    );
    assert_eq!(
        refused(|bar| bar.portion.which = 5).as_deref(),
        Some("parana.bars[1].portion.which")
    );
}
