//! `verify.claims`: what a text claims of the time, judged against the
//! sankalpa the SDK computes at the instant and the place
//! (`03-design/lexicon.md` §3, C1095).
//!
//! Every expectation is read back from `almanac.sankalpa` and the lexicon
//! rather than written down, so the test holds the grammar and not one
//! day's sky.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]

use teistro::verify::{Claim, ClaimsRequest, Unread, Verdict};
use teistro::{Context, Ephemeris, LexiconRequest};

/// 2026-01-15 08:00 Nepal time, 02:15 UTC, at Kathmandu.
const AT: &str = r#""at": 2461055.59375, "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400, "utcOffsetSeconds": 20700"#;

fn context() -> Context {
    Context::builder()
        .profile("nepali-drik")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("a context")
}

fn claims(sdk: &Context, text: &str) -> Vec<Claim> {
    let asked = ClaimsRequest::from_json(&format!(
        r#"{{"text": {}, {AT}}}"#,
        serde_json::to_string(text).unwrap()
    ))
    .unwrap();
    sdk.verify().claims(&asked).unwrap().value.claims
}

/// The sankalpa at [`AT`], every member a full key.
fn sankalpa(sdk: &Context) -> serde_json::Value {
    let asked = teistro::SankalpaRequest::from_json(&format!("{{{AT}}}")).unwrap();
    sdk.almanac().sankalpa_request(&asked).unwrap().value
}

/// A member's English name.
fn name(sdk: &Context, key: &str) -> String {
    let kind = key.split_once('.').unwrap().0;
    let lexicon = sdk
        .intl()
        .lexicon(
            &LexiconRequest::from_json(&format!(r#"{{"locale": "en-Latn", "kinds": ["{kind}"]}}"#))
                .unwrap(),
        )
        .unwrap();
    let entry = lexicon.entries.iter().find(|e| e.key == key).unwrap();
    entry
        .forms
        .iter()
        .find(|f| f.form == "name")
        .unwrap()
        .text
        .clone()
}

/// Another member of a kind, whose name is not the computed one's.
fn another(sdk: &Context, key: &str) -> String {
    let kind = key.split_once('.').unwrap().0;
    let lexicon = sdk
        .intl()
        .lexicon(
            &LexiconRequest::from_json(&format!(r#"{{"locale": "en-Latn", "kinds": ["{kind}"]}}"#))
                .unwrap(),
        )
        .unwrap();
    let own = name(sdk, key);
    lexicon
        .entries
        .iter()
        .map(|e| {
            e.forms
                .iter()
                .find(|f| f.form == "name")
                .unwrap()
                .text
                .clone()
        })
        .find(|other| *other != own && !other.contains(&own) && !own.contains(other.as_str()))
        .unwrap()
}

fn verdicts(found: &[Claim]) -> Vec<(String, Verdict, Option<Unread>)> {
    found
        .iter()
        .map(|c| (c.text.clone(), c.verdict, c.reason))
        .collect()
}

#[test]
fn a_claim_of_the_day_is_held_or_contradicted_by_the_sankalpa() {
    let sdk = context();
    let computed = sankalpa(&sdk);
    for (kind, pointer) in [
        ("tithi", "/tithi"),
        ("nakshatra", "/nakshatra"),
        ("vara", "/vara"),
        ("yoga", "/yoga"),
        ("ritu", "/ritu"),
        ("samvatsara", "/year/samvatsara"),
    ] {
        let key = computed.pointer(pointer).unwrap().as_str().unwrap();
        let held = claims(&sdk, &format!("The {kind} is {}.", name(&sdk, key)));
        assert_eq!(held.len(), 1, "{kind}: {held:?}");
        assert_eq!(held[0].kind.as_deref(), Some(kind), "{held:?}");
        assert_eq!(held[0].verdict, Verdict::Held, "{held:?}");
        assert_eq!(held[0].computed.as_deref(), Some(key));
        let wrong = claims(&sdk, &format!("The {kind} is {}.", another(&sdk, key)));
        assert_eq!(wrong[0].verdict, Verdict::Contradicted, "{wrong:?}");
        assert_eq!(wrong[0].computed.as_deref(), Some(key));
        assert!(!wrong[0].claimed.iter().any(|k| k == key));
    }
}

#[test]
fn a_graha_is_judged_in_the_sign_the_sankalpa_places_it() {
    let sdk = context();
    let computed = sankalpa(&sdk);
    let sign = |at: &str| computed[at].as_str().unwrap().to_owned();
    let (sun, jupiter, moon) = (sign("sun"), sign("jupiter"), sign("moon"));
    let text = format!(
        "The Sun is in {}. Jupiter is in {}. The Moon is in {}.",
        name(&sdk, &sun),
        name(&sdk, &jupiter),
        another(&sdk, &moon)
    );
    let found = claims(&sdk, &text);
    assert_eq!(found.len(), 3, "{found:?}");
    for (claim, graha, key) in [
        (&found[0], "graha.SUN", &sun),
        (&found[1], "graha.JUPITER", &jupiter),
        (&found[2], "graha.MOON", &moon),
    ] {
        assert_eq!(claim.graha.as_deref(), Some(graha), "{claim:?}");
        assert_eq!(claim.kind.as_deref(), Some("rashi"));
        assert_eq!(claim.computed.as_deref(), Some(key.as_str()));
    }
    assert_eq!(found[0].verdict, Verdict::Held);
    assert_eq!(found[1].verdict, Verdict::Held);
    assert_eq!(found[2].verdict, Verdict::Contradicted);
}

#[test]
fn the_moons_nakshatra_is_the_days() {
    let sdk = context();
    let key = sankalpa(&sdk)["nakshatra"].as_str().unwrap().to_owned();
    let found = claims(
        &sdk,
        &format!("चन्द्रमा {}मा छ।", {
            let lexicon = sdk
                .intl()
                .lexicon(
                    &LexiconRequest::from_json(r#"{"locale": "ne", "kinds": ["nakshatra"]}"#)
                        .unwrap(),
                )
                .unwrap();
            let entry = lexicon.entries.iter().find(|e| e.key == key).unwrap();
            entry.forms[0].text.clone()
        }),
    );
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].graha.as_deref(), Some("graha.MOON"));
    assert_eq!(found[0].verdict, Verdict::Held, "{found:?}");
}

#[test]
fn a_placement_the_sankalpa_does_not_compute_is_unread() {
    let sdk = context();
    let found = claims(&sdk, "Mars is in Aries. The Sun is in Rohini.");
    assert_eq!(
        verdicts(&found),
        [
            (
                String::from("Aries"),
                Verdict::Unread,
                Some(Unread::NotComputed)
            ),
            (
                String::from("Rohini"),
                Verdict::Unread,
                Some(Unread::NotComputed)
            ),
        ]
    );
}

#[test]
fn a_sentence_not_of_now_is_unread_with_why() {
    let sdk = context();
    for (text, why) in [
        ("Is today Ekadashi?", Unread::Question),
        ("Today is not Ekadashi.", Unread::Negated),
        ("It isn't Ekadashi", Unread::Negated),
        ("आज एकादशी होइन", Unread::Negated),
        ("आज एकादशी नहीं है", Unread::Negated),
        ("Tomorrow is Ekadashi.", Unread::OtherTime),
        ("भोलि एकादशी हो", Unread::OtherTime),
        ("कल एकादशी है", Unread::OtherTime),
    ] {
        let found = claims(&sdk, text);
        assert_eq!(found.len(), 1, "{text}: {found:?}");
        assert_eq!(found[0].verdict, Verdict::Unread, "{text}");
        assert_eq!(found[0].reason, Some(why), "{text}");
        assert_eq!(found[0].computed, None, "{text}");
        assert_eq!(found[0].kind.as_deref(), Some("tithi"), "{text}");
    }
}

#[test]
fn ambiguity_is_answered_with_every_candidate() {
    let sdk = context();
    // Shravana is a month and a nakshatra; nothing in the sentence parts them.
    let both = claims(&sdk, "आज श्रावण हो");
    assert_eq!(both[0].reason, Some(Unread::Ambiguous), "{both:?}");
    assert_eq!(both[0].kind, None);
    assert_eq!(both[0].claimed, ["masa.SHRAVANA", "nakshatra.SHRAVANA"]);
    // The kind's own word does.
    let star = claims(&sdk, "आज श्रावण नक्षत्र हो");
    assert_eq!(star[0].kind.as_deref(), Some("nakshatra"), "{star:?}");
    assert_ne!(star[0].verdict, Verdict::Unread);
    // A sign with no graha, and two grahas, say whose sign nowhere.
    let sign = claims(&sdk, "Today is Capricorn.");
    assert_eq!(sign[0].reason, Some(Unread::Ambiguous), "{sign:?}");
    let two = claims(&sdk, "The Sun and the Moon are in Capricorn.");
    assert_eq!(two[0].reason, Some(Unread::Ambiguous), "{two:?}");
}

#[test]
fn a_fortnights_word_settles_the_tithi() {
    let sdk = context();
    for (text, key, span) in [
        (
            "Today is Shukla Ekadashi.",
            "tithi.SHUKLA_EKADASHI",
            "Shukla Ekadashi",
        ),
        ("आज कृष्ण एकादशी हो", "tithi.KRISHNA_EKADASHI", "कृष्ण एकादशी"),
        (
            "Kartik sudi ekadashi",
            "tithi.SHUKLA_EKADASHI",
            "sudi ekadashi",
        ),
    ] {
        let found = claims(&sdk, text);
        let tithi = found
            .iter()
            .find(|c| c.kind.as_deref() == Some("tithi"))
            .expect(text);
        assert_eq!(tithi.claimed, [key], "{text}");
        assert_eq!(tithi.text, span, "{text}");
        assert!(
            found
                .iter()
                .all(|c| c.claimed.iter().all(|k| !k.starts_with("samvatsara."))),
            "{text}: {found:?}"
        );
    }
}

#[test]
fn spans_are_counted_in_scalar_values_across_sentences() {
    let sdk = context();
    let text = "नमस्ते। आज एकादशी हो।";
    let found = claims(&sdk, text);
    assert_eq!(found.len(), 1);
    let at: String = text
        .chars()
        .skip(found[0].start)
        .take(found[0].end - found[0].start)
        .collect();
    assert_eq!(at, "एकादशी");
}

#[test]
fn a_text_with_no_claim_answers_none_and_a_bad_locale_is_named() {
    let sdk = context();
    assert_eq!(claims(&sdk, "The quick brown fox. The Moon.").len(), 0);
    let asked =
        ClaimsRequest::from_json(&format!(r#"{{"text": "x", "locale": "de", {AT}}}"#)).unwrap();
    let refused = sdk.verify().claims(&asked).unwrap_err();
    assert_eq!(refused.field(), Some("claims.locale"));
}
