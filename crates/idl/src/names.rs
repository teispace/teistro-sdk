//! The naming rules, in one place, so the C header, every binding and the
//! documentation agree on what a thing is called
//! (`docs/02-architecture/06-api-conventions.md`, "Naming").
//!
//! - A Rust boundary type `TsPositionRequest` or `PositionRequestC` has the
//!   binding name `PositionRequest`: the `Ts` prefix and the `C` suffix are
//!   the Rust spellings of "this is the boundary form".
//! - Its C name is the prefix plus the snake case: `ts_position_request`.
//! - An enum member `Status::InvalidArg` is `TS_STATUS_INVALID_ARG` in C; a
//!   catalogue member keeps its key: `TS_GRAHA_PURVA_PHALGUNI`.
//! - A symbol is already `ts_<module>_<verb>` in Rust and stays so.

/// The binding-facing name of a Rust boundary type: `TsContext` becomes
/// `Context`, `PositionRequestC` becomes `PositionRequest`, `Status` stays.
#[must_use]
pub fn binding_type_name(rust_name: &str) -> String {
    let without_prefix = rust_name
        .strip_prefix("Ts")
        .filter(|rest| rest.chars().next().is_some_and(char::is_uppercase))
        .unwrap_or(rust_name);
    let without_suffix = without_prefix
        .strip_suffix('C')
        .filter(|rest| {
            rest.chars().last().is_some_and(char::is_lowercase)
                && rest.chars().next().is_some_and(char::is_uppercase)
        })
        .unwrap_or(without_prefix);
    without_suffix.to_string()
}

/// The C name of a boundary type: the prefix plus the binding name in
/// snake case (`ts_position_request`).
#[must_use]
pub fn c_type_name(prefix: &str, rust_name: &str) -> String {
    format!("{prefix}{}", snake(&binding_type_name(rust_name)))
}

/// The C name of an enum member: `TS_STATUS_INVALID_ARG`; a catalogue
/// member's key is used as it is (`TS_GRAHA_PURVA_PHALGUNI`).
#[must_use]
pub fn c_enum_member(
    prefix: &str,
    enum_rust_name: &str,
    variant: &str,
    key: Option<&str>,
) -> String {
    let member = key.map_or_else(|| screaming(variant), str::to_string);
    format!(
        "{}{}_{member}",
        prefix.to_ascii_uppercase(),
        screaming(&binding_type_name(enum_rust_name))
    )
}

/// The C name of a constant: `TS_ABI_VERSION` stays, `VTABLE_ABI_VERSION`
/// becomes `TS_VTABLE_ABI_VERSION`.
#[must_use]
pub fn c_constant_name(prefix: &str, rust_name: &str) -> String {
    let upper = prefix.to_ascii_uppercase();
    if rust_name.starts_with(&upper) {
        rust_name.to_string()
    } else {
        format!("{upper}{rust_name}")
    }
}

/// The binding-facing method name of a handle function:
/// `ts_context_settings_json` on `TsContext` becomes `settings_json`;
/// `ts_positions` becomes `positions`.
#[must_use]
pub fn method_name(prefix: &str, opaque_rust_name: &str, symbol: &str) -> String {
    let handle_prefix = format!("{prefix}{}_", snake(&binding_type_name(opaque_rust_name)));
    symbol
        .strip_prefix(&handle_prefix)
        .or_else(|| symbol.strip_prefix(prefix))
        .unwrap_or(symbol)
        .to_string()
}

/// `PascalCase` or `camelCase` to `snake_case`; digits stay attached to
/// the word before them (`Ut1` becomes `ut1`).
#[must_use]
pub fn snake(name: &str) -> String {
    separate(name, '_')
}

/// `PascalCase` to `kebab-case`.
#[must_use]
pub fn kebab(name: &str) -> String {
    separate(name, '-')
}

/// `PascalCase` to `SCREAMING_SNAKE_CASE`.
#[must_use]
pub fn screaming(name: &str) -> String {
    snake(name).to_ascii_uppercase()
}

/// `snake_case` to `camelCase`.
#[must_use]
pub fn camel(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut upper = false;
    for c in name.chars() {
        if c == '_' {
            upper = true;
        } else if upper {
            out.extend(c.to_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// `snake_case` to `PascalCase`.
#[must_use]
pub fn pascal(name: &str) -> String {
    let c = camel(name);
    let mut chars = c.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn separate(name: &str, separator: char) -> String {
    let mut out = String::with_capacity(name.len() + 4);
    let chars: Vec<char> = name.chars().collect();
    for (i, &ch) in chars.iter().enumerate() {
        if ch == '_' || ch == '-' {
            out.push(separator);
            continue;
        }
        if ch.is_ascii_uppercase() && i > 0 {
            let previous = chars.get(i - 1).copied().unwrap_or(' ');
            let next_is_lower = chars.get(i + 1).is_some_and(char::is_ascii_lowercase);
            // A boundary before an upper-case letter that follows a lower-case
            // letter or a digit (`positionRequest`, `Ut1Tt`), or that starts a
            // new word inside an acronym run (`ABIVersion` becomes `abi_version`).
            if previous.is_ascii_lowercase()
                || previous.is_ascii_digit()
                || (previous.is_ascii_uppercase() && next_is_lower)
            {
                out.push(separator);
            }
        }
        out.push(ch.to_ascii_lowercase());
    }
    out
}

/// Documentation text with rustdoc's link syntax removed:
/// ``[`Frame::to_bits`]`` becomes `Frame::to_bits`, `[text](url)` becomes
/// `text`, and ``[`X`](crate::path)`` becomes `X`.
#[must_use]
pub fn clean_doc(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(open) = rest.find('[') {
        let (before, after) = rest.split_at(open);
        out.push_str(before);
        let Some(close) = after.find(']') else {
            out.push_str(after);
            return out;
        };
        let inner = &after[1..close];
        let mut tail = &after[close + 1..];
        if let Some(stripped) = tail.strip_prefix('(') {
            if let Some(end) = stripped.find(')') {
                tail = &stripped[end + 1..];
            }
        }
        out.push_str(inner);
        rest = tail;
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_names_lose_their_boundary_spellings() {
        assert_eq!(binding_type_name("TsContext"), "Context");
        assert_eq!(binding_type_name("PositionRequestC"), "PositionRequest");
        assert_eq!(binding_type_name("Status"), "Status");
        assert_eq!(binding_type_name("TsStr"), "Str");
        assert_eq!(binding_type_name("ProviderVtable"), "ProviderVtable");
        assert_eq!(
            c_type_name("ts_", "PositionRequestC"),
            "ts_position_request"
        );
        assert_eq!(c_type_name("ts_", "TsContext"), "ts_context");
        assert_eq!(c_type_name("ts_", "TimeScale"), "ts_time_scale");
    }

    #[test]
    fn members_constants_and_methods_follow_the_rules() {
        assert_eq!(
            c_enum_member("ts_", "Status", "InvalidArg", None),
            "TS_STATUS_INVALID_ARG"
        );
        assert_eq!(
            c_enum_member("ts_", "Graha", "PurvaPhalguni", Some("PURVA_PHALGUNI")),
            "TS_GRAHA_PURVA_PHALGUNI"
        );
        assert_eq!(
            c_enum_member("ts_", "TimeScale", "Ut1", None),
            "TS_TIME_SCALE_UT1"
        );
        assert_eq!(c_constant_name("ts_", "TS_ABI_VERSION"), "TS_ABI_VERSION");
        assert_eq!(
            c_constant_name("ts_", "VTABLE_ABI_VERSION"),
            "TS_VTABLE_ABI_VERSION"
        );
        assert_eq!(
            method_name("ts_", "TsContext", "ts_context_settings_json"),
            "settings_json"
        );
        assert_eq!(method_name("ts_", "TsContext", "ts_positions"), "positions");
    }

    #[test]
    fn case_conversions_handle_digits_and_acronyms() {
        assert_eq!(snake("PositionRequest"), "position_request");
        assert_eq!(snake("Ut1"), "ut1");
        assert_eq!(snake("ABIVersion"), "abi_version");
        assert_eq!(snake("jdUt1"), "jd_ut1");
        assert_eq!(kebab("MeanNode"), "mean-node");
        assert_eq!(screaming("OsculatingApogee"), "OSCULATING_APOGEE");
        assert_eq!(camel("dasha_depth"), "dashaDepth");
        assert_eq!(pascal("dasha_depth"), "DashaDepth");
        assert_eq!(pascal(""), "");
    }

    #[test]
    fn docs_lose_rustdoc_links() {
        assert_eq!(
            clean_doc("See [`Frame::to_bits`]."),
            "See `Frame::to_bits`."
        );
        assert_eq!(clean_doc("A [link](https://x.y) here"), "A link here");
        assert_eq!(clean_doc("[`X`](crate::x::X) and [`Y`]"), "`X` and `Y`");
        assert_eq!(clean_doc("no links [unclosed"), "no links [unclosed");
    }
}
