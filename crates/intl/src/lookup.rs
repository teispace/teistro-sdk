//! Which loaded locale a caller's tag means (`03-design/intl-engine-and-packs.md`,
//! "Choosing a locale"): RFC 4647 §3.4 lookup, widened for the SDK's long tags.
//!
//! RFC 4647 lookup truncates the asked range from the end until it equals a
//! tag. The SDK's tags carry their script and often a region, so `ne` would
//! never equal `ne-Deva-NP`: at each length a tag the range is a prefix of
//! counts too, as RFC 4647 §3.3.1's basic filtering reads it. The lookup's
//! last step, a default, is left out on purpose: a tag that means nothing
//! loaded is refused, and never answers in another language.

/// Why nothing was chosen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Missed<'a> {
    /// No loaded tag answers the range.
    Unknown,
    /// Several do, and nothing says which.
    Ambiguous(Vec<&'a str>),
}

/// The script a language is written in unless a tag says otherwise, for the
/// languages the SDK ships: CLDR's `likelySubtags`. It settles `sa` between
/// `sa-Deva` and `sa-Latn`, and a language not listed is refused as ambiguous
/// rather than settled by the order the tags sort in.
const LIKELY_SCRIPTS: [(&str, &str); 4] = [
    ("en", "Latn"),
    ("hi", "Deva"),
    ("ne", "Deva"),
    ("sa", "Deva"),
];

fn is_script(subtag: &str) -> bool {
    subtag.len() == 4 && subtag.bytes().all(|byte| byte.is_ascii_alphabetic())
}

/// Well formed as a language range with no wildcard: subtags of one to eight
/// letters or digits.
fn well_formed(range: &str) -> bool {
    range.split('-').all(|subtag| {
        (1..=8).contains(&subtag.len()) && subtag.bytes().all(|byte| byte.is_ascii_alphanumeric())
    })
}

/// Whether `tag` opens with `prefix` and a further subtag.
fn extends(tag: &str, prefix: &str) -> bool {
    tag.as_bytes().get(prefix.len()) == Some(&b'-')
        && tag
            .get(..prefix.len())
            .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
}

/// The one of `under` the language's likely script names, if exactly one is.
fn likely<'a>(under: &[&'a str]) -> Option<&'a str> {
    let mut written = under.iter().filter(|tag| {
        let mut subtags = tag.split('-');
        let language = subtags.next().unwrap_or_default();
        let script = subtags.next().unwrap_or_default();
        LIKELY_SCRIPTS
            .iter()
            .any(|(known, usual)| known.eq_ignore_ascii_case(language) && *usual == script)
    });
    let found = written.next().copied();
    found.filter(|_| written.next().is_none())
}

/// The loaded tag `range` means, compared without regard to case.
///
/// A stated script is never truncated away: `hi-Latn` asks for Latin letters,
/// and a Devanagari locale does not answer it.
pub(crate) fn lookup<'a>(range: &str, loaded: &[&'a str]) -> Result<&'a str, Missed<'a>> {
    if !well_formed(range) {
        return Err(Missed::Unknown);
    }
    let subtags: Vec<&str> = range.split('-').collect();
    let floor = if subtags.get(1).is_some_and(|subtag| is_script(subtag)) {
        2
    } else {
        1
    };
    let mut length = subtags.len();
    while length >= floor {
        let prefix = subtags.get(..length).unwrap_or_default().join("-");
        if let Some(tag) = loaded.iter().find(|tag| tag.eq_ignore_ascii_case(&prefix)) {
            return Ok(tag);
        }
        let under: Vec<&'a str> = loaded
            .iter()
            .copied()
            .filter(|tag| extends(tag, &prefix))
            .collect();
        match under.as_slice() {
            [] => {}
            [one] => return Ok(one),
            many => return likely(many).ok_or_else(|| Missed::Ambiguous(under.clone())),
        }
        length -= 1;
        // RFC 4647 §3.4: a singleton never ends a truncated range.
        while length > floor
            && subtags
                .get(length - 1)
                .is_some_and(|subtag| subtag.len() == 1)
        {
            length -= 1;
        }
    }
    Err(Missed::Unknown)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{Missed, lookup};

    const SHIPPED: [&str; 5] = ["en-Latn", "hi-Deva-IN", "ne-Deva-NP", "sa-Deva", "sa-Latn"];

    #[test]
    fn a_shipped_tag_answers_itself_in_any_case() {
        for tag in SHIPPED {
            assert_eq!(lookup(tag, &SHIPPED), Ok(tag));
        }
        assert_eq!(lookup("NE-deva-np", &SHIPPED), Ok("ne-Deva-NP"));
    }

    #[test]
    fn a_short_tag_finds_the_long_one() {
        for (range, tag) in [
            ("ne", "ne-Deva-NP"),
            ("NE", "ne-Deva-NP"),
            ("ne-NP", "ne-Deva-NP"),
            ("ne-Deva", "ne-Deva-NP"),
            ("en", "en-Latn"),
            ("en-US", "en-Latn"),
            ("hi", "hi-Deva-IN"),
            ("hi-IN", "hi-Deva-IN"),
            ("sa", "sa-Deva"),
            ("sa-IN", "sa-Deva"),
        ] {
            assert_eq!(lookup(range, &SHIPPED), Ok(tag), "{range}");
        }
    }

    #[test]
    fn a_long_tag_truncates_to_a_shorter_one() {
        for (range, tag) in [
            ("en-Latn-GB", "en-Latn"),
            ("ne-Deva-NP-u-nu-latn", "ne-Deva-NP"),
            ("sa-Latn-IN", "sa-Latn"),
            ("sa-Deva-x-mine", "sa-Deva"),
        ] {
            assert_eq!(lookup(range, &SHIPPED), Ok(tag), "{range}");
        }
    }

    #[test]
    fn a_stated_script_is_not_given_up() {
        for range in ["hi-Latn", "ne-Latn-NP", "en-Deva", "sa-Arab"] {
            assert_eq!(lookup(range, &SHIPPED), Err(Missed::Unknown), "{range}");
        }
    }

    #[test]
    fn nothing_answers_for_a_language_not_loaded() {
        for range in ["de", "de-Latn-DE", "xx-Nope", "nep", "n"] {
            assert_eq!(lookup(range, &SHIPPED), Err(Missed::Unknown), "{range}");
        }
        assert_eq!(lookup("hi", &["en-Latn"]), Err(Missed::Unknown));
    }

    #[test]
    fn what_is_not_a_range_is_refused() {
        for range in [
            "",
            "*",
            "ne-*",
            "ne_NP",
            "ne-",
            "-ne",
            "ne--NP",
            "ne-toolongsubtag",
        ] {
            assert_eq!(lookup(range, &SHIPPED), Err(Missed::Unknown), "{range:?}");
        }
    }

    #[test]
    fn two_tags_with_nothing_to_settle_them_are_refused_by_name() {
        let loaded = ["mai-Deva-NP", "mai-Tirh"];
        assert_eq!(
            lookup("mai", &loaded),
            Err(Missed::Ambiguous(loaded.to_vec()))
        );
        let two = ["ne-Deva-IN", "ne-Deva-NP"];
        assert_eq!(lookup("ne", &two), Err(Missed::Ambiguous(two.to_vec())));
        assert_eq!(lookup("ne-Deva-IN", &two), Ok("ne-Deva-IN"));
    }
}
