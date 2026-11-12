//! The regions a festival set is kept in (SR-89,
//! `03-design/festival-metadata.md` §2a): a country by its ISO 3166-1
//! alpha-2 code (`IN`, `NP`), a state by its ISO 3166-2 code (`IN-TN`), so
//! a place a geo provider resolves selects its sets with no table of the
//! consumer's. The names the state lists were first shipped under
//! (`TAMIL_NADU`) stay as aliases. A part of a country no ISO code names,
//! such as Nepal's `KATHMANDU_VALLEY` (three districts across one province)
//! or the `HILLS` and `TERAI` a notice divides the country into, keeps its
//! own key.

/// A region the shipped sets name: its key and the name it was shipped
/// under before ISO codes, if any.
///
/// ```
/// use teistro_panchanga::festival::region::REGIONS;
///
/// let kerala = REGIONS.iter().find(|region| region.key == "IN-KL").ok_or("shipped")?;
/// assert_eq!(kerala.alias, Some("KERALA"));
/// # Ok::<(), &str>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    /// The key a rule carries: an ISO 3166 code where one exists.
    pub key: &'static str,
    /// The key it was first shipped under, read as this one.
    pub alias: Option<&'static str>,
}

/// Every region a shipped set names, sorted by key.
///
/// ```
/// use teistro_panchanga::festival::region::REGIONS;
///
/// assert!(REGIONS.iter().any(|region| region.key == "KATHMANDU_VALLEY" && region.alias.is_none()));
/// ```
pub const REGIONS: [Region; 13] = [
    region("HILLS", None),
    region("IN", None),
    region("IN-AP", None),
    region("IN-AS", Some("ASSAM")),
    region("IN-GJ", None),
    region("IN-KA", None),
    region("IN-KL", Some("KERALA")),
    region("IN-MH", Some("MAHARASHTRA")),
    region("IN-TG", None),
    region("IN-TN", Some("TAMIL_NADU")),
    region("IN-WB", Some("WEST_BENGAL")),
    region("KATHMANDU_VALLEY", None),
    region("NP", None),
    region("TERAI", None),
];

/// A row of [`REGIONS`].
const fn region(key: &'static str, alias: Option<&'static str>) -> Region {
    Region { key, alias }
}

/// The key a region is held under: an alias read as its ISO code, and any
/// other key as it is, so a key no shipped set names still matches a
/// caller's own rule that names it.
///
/// ```
/// use teistro_panchanga::festival::region;
///
/// assert_eq!(region::canonical("TAMIL_NADU"), "IN-TN");
/// assert_eq!(region::canonical("IN-TN"), "IN-TN");
/// assert_eq!(region::canonical("KATHMANDU_VALLEY"), "KATHMANDU_VALLEY");
/// assert_eq!(region::canonical("IN-UP"), "IN-UP");
/// ```
#[must_use]
pub fn canonical(key: &str) -> &str {
    REGIONS
        .iter()
        .find(|region| region.alias == Some(key))
        .map_or(key, |region| region.key)
}

/// The country of an ISO 3166 code: `IN` of `IN-TN` and of `IN`; `None`
/// for a key that is not one.
///
/// ```
/// use teistro_panchanga::festival::region;
///
/// assert_eq!(region::country("IN-TN"), Some("IN"));
/// assert_eq!(region::country("NP"), Some("NP"));
/// assert_eq!(region::country("NP-P3"), Some("NP"));
/// assert_eq!(region::country("TERAI"), None);
/// assert_eq!(region::country("in-tn"), None);
/// ```
#[must_use]
pub fn country(code: &str) -> Option<&str> {
    let (country, part) = code
        .split_once('-')
        .map_or((code, None), |(c, p)| (c, Some(p)));
    let upper = |text: &str| text.bytes().all(|b| b.is_ascii_uppercase());
    let subdivision = |text: &str| {
        (1..=3).contains(&text.len())
            && text
                .bytes()
                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
    };
    (country.len() == 2 && upper(country) && part.is_none_or(subdivision)).then_some(country)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn the_table_is_sorted_and_each_key_and_alias_is_one_region() {
        let keys: Vec<&str> = REGIONS.iter().map(|region| region.key).collect();
        let mut sorted = keys.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(keys, sorted, "sorted, no key twice");
        for region in REGIONS {
            if let Some(alias) = region.alias {
                assert!(!keys.contains(&alias), "{alias} is an alias and a key");
                assert_eq!(canonical(alias), region.key);
                assert_eq!(
                    country(region.key),
                    Some("IN"),
                    "every alias is an Indian state's"
                );
            }
            assert_eq!(canonical(region.key), region.key);
        }
    }

    #[test]
    fn a_country_is_read_only_from_a_well_formed_code() {
        for bad in [
            "", "I", "IND", "IN-", "IN-TNXX", "IN_TN", "in", "IN-tn", "-TN",
        ] {
            assert_eq!(country(bad), None, "{bad:?}");
        }
        assert_eq!(country("GB-ENG"), Some("GB"));
    }
}
