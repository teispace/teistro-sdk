//! The JSON records that cross as text inside a result blob, in
//! `com.teispace.teistro.record`: an object as a Java record, a closed set
//! of keys as an enum, a union tagged by `kind` as a sealed interface of
//! records, each with `of`, its decoder from what `Json.read` answers.
//! Their own package, so a record's name never meets an enum's.

use std::fmt::Write;

use super::{File, identifier, javadoc, literal, preamble};
use crate::model::Api;
use crate::names::{camel, pascal, screaming};
use crate::records::{RecordField, RecordShape, RecordType, RecordVariant};

/// The records' package.
pub(super) const RECORD_PACKAGE: &str = "com.teispace.teistro.record";

/// Every record file, and `Read`, the checks their decoders share.
pub(super) fn files(api: &Api) -> Vec<File> {
    let dir = RECORD_PACKAGE.replace('.', "/");
    let mut files = vec![
        File {
            path: format!("{dir}/Read.java"),
            text: format!("{}{READ}", preamble(api, RECORD_PACKAGE)),
        },
        File {
            path: format!("{dir}/Pair.java"),
            text: format!("{}{PAIR}", preamble(api, RECORD_PACKAGE)),
        },
    ];
    for record in &api.records {
        let text = match &record.shape {
            RecordShape::Object { fields } => {
                render_object(api, &record.name, &record.doc, fields, None)
            }
            RecordShape::Keys { values } => render_keys(api, &record.name, &record.doc, values),
            RecordShape::Tagged { tag, variants } => {
                for variant in variants {
                    files.push(File {
                        path: format!("{dir}/{}.java", variant_name(&record.name, variant)),
                        text: render_object(
                            api,
                            &variant_name(&record.name, variant),
                            &variant.doc,
                            &variant.fields,
                            Some((&record.name, tag, &variant.key)),
                        ),
                    });
                }
                render_union(api, &record.name, &record.doc, tag, variants)
            }
        };
        files.push(File {
            path: format!("{dir}/{}.java", record.name),
            text,
        });
    }
    files
}

/// A variant's type: the union's name, then its key (`CalendarResolutionTabular`).
fn variant_name(union: &str, variant: &RecordVariant) -> String {
    format!("{union}{}", pascal(&variant.key.to_ascii_lowercase()))
}

/// A record type as Java holds it; `boxed` for a value that may be null or
/// sits in a list.
fn java_type(ty: &RecordType, boxed: bool) -> String {
    match ty {
        RecordType::Bool => String::from(if boxed { "Boolean" } else { "boolean" }),
        RecordType::Integer => String::from(if boxed { "Long" } else { "long" }),
        RecordType::Number => String::from(if boxed { "Double" } else { "double" }),
        RecordType::Text => String::from("String"),
        RecordType::Record { name } => name.clone(),
        RecordType::List { of } => format!("List<{}>", java_type(of, true)),
        RecordType::Pair { first, second } => {
            format!(
                "Pair<{}, {}>",
                java_type(first, true),
                java_type(second, true)
            )
        }
    }
}

/// What decodes `expr`, an `Object` of type `ty`, naming `at` in a refusal.
fn decode(ty: &RecordType, expr: &str, at: &str) -> String {
    match ty {
        RecordType::Bool => format!("Read.bool({expr}, {at})"),
        RecordType::Integer => format!("Read.integer({expr}, {at})"),
        RecordType::Number => format!("Read.number({expr}, {at})"),
        RecordType::Text => format!("Read.text({expr}, {at})"),
        RecordType::Record { name } => format!("{name}.of({expr})"),
        RecordType::List { of } => format!(
            "Read.list({expr}, {at}, item -> {})",
            decode(of, "item", at)
        ),
        RecordType::Pair { first, second } => format!(
            "Read.pair({expr}, {at}, a -> {}, b -> {})",
            decode(first, "a", at),
            decode(second, "b", at)
        ),
    }
}

/// An object, or one variant of a union.
fn render_object(
    api: &Api,
    name: &str,
    doc: &str,
    fields: &[RecordField],
    parent: Option<(&str, &str, &str)>,
) -> String {
    let mut out = preamble(api, RECORD_PACKAGE);
    out.push_str("import java.util.List;\nimport java.util.Map;\n\n");
    let mut full = doc.to_string();
    full.push('\n');
    for field in fields {
        let _ = write!(
            full,
            "\n@param {} {}{}",
            identifier(&camel(&field.name)),
            field.doc.replace('\n', " "),
            if field.nullable { " May be null." } else { "" }
        );
    }
    out.push_str(&javadoc(&full, "").replace("&#64;param", "@param"));
    let components: Vec<String> = fields
        .iter()
        .map(|f| {
            format!(
                "{} {}",
                java_type(&f.ty, f.nullable),
                identifier(&camel(&f.name))
            )
        })
        .collect();
    let implements =
        parent.map_or_else(String::new, |(union, _, _)| format!(" implements {union}"));
    let _ = writeln!(
        out,
        "@SuppressWarnings(\"unused\")\npublic record {name}({}){implements} {{",
        components.join(", ")
    );
    if let Some((_, tag, key)) = parent {
        let _ = writeln!(
            out,
            "    /** The `{tag}` this variant is written under. */\n    public static final String {} = {};\n\n    @Override\n    public String {}() {{\n        return {};\n    }}\n",
            screaming(tag),
            literal(key),
            identifier(&camel(tag)),
            screaming(tag)
        );
    }
    let reads: Vec<String> = fields
        .iter()
        .map(|f| {
            let at = literal(&format!("{name}.{}", f.name));
            let got = format!("json.get({})", literal(&f.name));
            if f.nullable {
                format!(
                    "Read.orNull({got}, value -> {})",
                    decode(&f.ty, "value", &at)
                )
            } else {
                decode(&f.ty, &format!("Read.required({got}, {at})"), &at)
            }
        })
        .collect();
    let _ = writeln!(
        out,
        "    /**\n     * A `{name}` from its JSON, as {{@code Json.read}} answers it.\n     *\n     * @param raw the JSON value\n     * @return the record\n     * @throws IllegalArgumentException for JSON that is not one, naming the field\n     */\n    public static {name} of(Object raw) {{\n        Map<String, Object> json = Read.object(raw, {});\n        return new {name}({});\n    }}\n}}",
        literal(name),
        if reads.is_empty() {
            String::new()
        } else {
            format!("\n            {}", reads.join(",\n            "))
        }
    );
    out
}

/// A closed set of keys, as an enum of them.
fn render_keys(api: &Api, name: &str, doc: &str, values: &[crate::records::RecordKey]) -> String {
    let mut out = preamble(api, RECORD_PACKAGE);
    out.push_str(&javadoc(doc, ""));
    let _ = writeln!(out, "public enum {name} {{");
    let constants: Vec<String> = values
        .iter()
        .map(|v| {
            format!(
                "{}    {}({})",
                javadoc(&v.doc, "    "),
                screaming(&v.key),
                literal(&v.key)
            )
        })
        .collect();
    let _ = writeln!(
        out,
        "{};\n\n    private final String key;\n\n    {name}(String key) {{\n        this.key = key;\n    }}\n\n    /**\n     * The key the SDK writes.\n     *\n     * @return the key\n     */\n    public String key() {{\n        return key;\n    }}\n\n    /**\n     * The member a key names.\n     *\n     * @param raw the JSON value\n     * @return the member\n     * @throws IllegalArgumentException for a key the SDK does not write\n     */\n    public static {name} of(Object raw) {{\n        for ({name} member : values()) {{\n            if (member.key.equals(raw)) {{\n                return member;\n            }}\n        }}\n        throw new IllegalArgumentException(\"not a {name}: \" + raw);\n    }}\n}}",
        constants.join(",\n")
    );
    out
}

/// A union tagged by a field, as a sealed interface of its variants.
fn render_union(api: &Api, name: &str, doc: &str, tag: &str, variants: &[RecordVariant]) -> String {
    let mut out = preamble(api, RECORD_PACKAGE);
    out.push_str("import java.util.Map;\n\n");
    out.push_str(&javadoc(doc, ""));
    let names: Vec<String> = variants.iter().map(|v| variant_name(name, v)).collect();
    let _ = writeln!(
        out,
        "public sealed interface {name} permits {} {{\n    /**\n     * The `{tag}` the variant is written under.\n     *\n     * @return the tag\n     */\n    String {}();\n\n    /**\n     * A `{name}` from its JSON, by its `{tag}`.\n     *\n     * @param raw the JSON value\n     * @return the variant\n     * @throws IllegalArgumentException for JSON that is none of them\n     */\n    static {name} of(Object raw) {{\n        Map<String, Object> json = Read.object(raw, {});\n        Object which = json.get({});",
        names.join(", "),
        identifier(&camel(tag)),
        literal(name),
        literal(tag)
    );
    for (variant, variant_name) in variants.iter().zip(&names) {
        let _ = writeln!(
            out,
            "        if ({}.equals(which)) {{\n            return {variant_name}.of(raw);\n        }}",
            literal(&variant.key)
        );
    }
    let _ = writeln!(
        out,
        "        throw new IllegalArgumentException(\"not a {name}: {tag} \" + which);\n    }}\n}}"
    );
    out
}

const PAIR: &str = r"/**
 * Two values the SDK writes as a two-element array.
 *
 * @param first the first
 * @param second the second
 * @param <A> the first's type
 * @param <B> the second's type
 */
public record Pair<A, B>(A first, B second) {}
";

const READ: &str = r#"import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.function.Function;

/** The checks every record's decoder shares, each naming where the JSON stopped being the record. */
final class Read {
    private Read() {}

    private static IllegalArgumentException refused(String at, String wanted, Object found) {
        return new IllegalArgumentException("`" + at + "` is not " + wanted + ": " + found);
    }

    @SuppressWarnings("unchecked")
    static Map<String, Object> object(Object raw, String at) {
        if (raw instanceof Map<?, ?> map) {
            return (Map<String, Object>) map;
        }
        throw refused(at, "an object", raw);
    }

    static Object required(Object raw, String at) {
        if (raw == null) {
            throw new IllegalArgumentException("`" + at + "` is missing");
        }
        return raw;
    }

    static <T> T orNull(Object raw, Function<Object, T> read) {
        return raw == null ? null : read.apply(raw);
    }

    static boolean bool(Object raw, String at) {
        if (raw instanceof Boolean flag) {
            return flag;
        }
        throw refused(at, "a boolean", raw);
    }

    static long integer(Object raw, String at) {
        if (raw instanceof Long number) {
            return number;
        }
        throw refused(at, "an integer", raw);
    }

    static double number(Object raw, String at) {
        if (raw instanceof Number number) {
            return number.doubleValue();
        }
        throw refused(at, "a number", raw);
    }

    static String text(Object raw, String at) {
        if (raw instanceof String text) {
            return text;
        }
        throw refused(at, "a string", raw);
    }

    static <T> List<T> list(Object raw, String at, Function<Object, T> item) {
        if (!(raw instanceof List<?> items)) {
            throw refused(at, "a list", raw);
        }
        List<T> read = new ArrayList<>(items.size());
        for (Object one : items) {
            read.add(item.apply(one));
        }
        return Collections.unmodifiableList(read);
    }

    static <A, B> Pair<A, B> pair(Object raw, String at, Function<Object, A> first, Function<Object, B> second) {
        if (!(raw instanceof List<?> items) || items.size() != 2) {
            throw refused(at, "a pair", raw);
        }
        return new Pair<>(first.apply(items.get(0)), second.apply(items.get(1)));
    }
}
"#;
