//! The JSON request records a binding writes, in
//! `com.teispace.teistro.request`: each read from the façade's own schema
//! of the request, so a request is typed as serde reads it and a misspelt
//! field or a swapped member fails before the call.
//!
//! An object is a record with a builder where it has optional fields, a
//! closed set of keys an enum, a union tagged by a field a sealed
//! interface of records, a key or a one-key object (serde's external
//! tagging) a sealed interface of the keys and the data variants, and a
//! few names or one value a sealed interface of the two. Each writes
//! itself as the tree `Json.write` writes (`toJson`), leaving out what is
//! null so the SDK's default stands, and reads itself back (`of`). A key
//! set that is a boundary enum is that enum, and a catalogue member is the
//! catalogue's enum.

use std::collections::BTreeMap;
use std::fmt::Write;

use super::records::READ;
use super::{File, identifier, javadoc, literal, preamble};
use crate::model::Api;
use crate::names::{binding_type_name, pascal};
use crate::records::{ExternalVariant, RecordField, RecordShape, RecordType, Width};

/// The requests' package.
pub(super) const REQUEST_PACKAGE: &str = "com.teispace.teistro.request";

/// The main package, where the boundary's and the catalogue's enums are.
const MAIN: &str = "com.teispace.teistro";

/// Every request file, with the interface they share, `Keyed`, and `Read`.
pub(super) fn files(api: &Api) -> Vec<File> {
    let dir = REQUEST_PACKAGE.replace('.', "/");
    if api.requests.records.is_empty() {
        return Vec::new();
    }
    let types = Types::of(api);
    let mut files = vec![
        File {
            path: format!("{dir}/Read.java"),
            text: format!("{}{}", preamble(api, REQUEST_PACKAGE), read()),
        },
        File {
            path: format!("{dir}/RequestJson.java"),
            text: format!("{}{REQUEST_JSON}", preamble(api, REQUEST_PACKAGE)),
        },
        File {
            path: format!("{dir}/Keyed.java"),
            text: format!("{}{KEYED}", preamble(api, REQUEST_PACKAGE)),
        },
        File {
            path: format!("{dir}/Requests.java"),
            text: requests(api),
        },
    ];
    for record in &api.requests.records {
        if api.requests.enums.contains_key(&record.name) {
            continue;
        }
        let mut file = |name: &str, text: String| {
            files.push(File {
                path: format!("{dir}/{name}.java"),
                text,
            });
        };
        match &record.shape {
            RecordShape::Object { fields } => {
                file(
                    &record.name,
                    types.object(&record.name, &record.doc, fields, None),
                );
            }
            RecordShape::Keys { values } => {
                let keys: Vec<(String, String)> = values
                    .iter()
                    .map(|value| (value.key.clone(), value.doc.clone()))
                    .collect();
                file(&record.name, types.keys(&record.name, &record.doc, &keys));
            }
            RecordShape::Tagged { tag, variants } => {
                let mut permits = Vec::new();
                for variant in variants {
                    let name = format!("{}{}", record.name, part(&variant.key));
                    let parent = Variant {
                        union: &record.name,
                        tag,
                        key: &variant.key,
                    };
                    file(
                        &name,
                        types.object(&name, &variant.doc, &variant.fields, Some(&parent)),
                    );
                    permits.push((variant.key.clone(), name));
                }
                file(
                    &record.name,
                    types.tagged(&record.name, &record.doc, tag, &permits),
                );
            }
            RecordShape::External { variants } => {
                for (name, text) in types.external(&record.name, &record.doc, variants) {
                    file(&name, text);
                }
            }
            RecordShape::OrNamed { names, of } => {
                for (name, text) in types.or_named(&record.name, &record.doc, names, of) {
                    file(&name, text);
                }
            }
        }
    }
    files
}

/// `Requests`: each record entry point's request type, by the entry
/// point's name, so a request held as JSON is read into its type.
fn requests(api: &Api) -> String {
    let mut out = preamble(api, REQUEST_PACKAGE);
    out.push_str(
        "import java.util.Map;\nimport java.util.Set;\nimport java.util.function.Function;\n\n",
    );
    let entries: Vec<String> = api
        .requests
        .roots
        .iter()
        .map(|root| {
            assert!(
                !api.requests.enums.contains_key(&root.name),
                "{} asks for a boundary enum, which has no reader of its own",
                root.record
            );
            format!(
                "            Map.entry({}, {}::of)",
                literal(&root.record),
                root.name
            )
        })
        .collect();
    let _ = writeln!(
        out,
        "/** Each record entry point's request type, by the entry point's name. */\npublic final class Requests {{\n    private static final Map<String, Function<Object, RequestJson>> READERS = Map.ofEntries(\n{});\n\n    private Requests() {{}}\n\n    /**\n     * The record entry points whose request is typed here; an engine's own\n     * operations stay a JSON value.\n     *\n     * @return the entry points' names\n     */\n    public static Set<String> records() {{\n        return READERS.keySet();\n    }}\n\n    /**\n     * A record entry point's request from its JSON, as {{@code Json.read}}\n     * answers it.\n     *\n     * @param record the entry point's name ({{@code chart.found}})\n     * @param raw the request's JSON value\n     * @return the typed request\n     * @throws IllegalArgumentException for an entry point with no typed\n     *     request, or JSON that is not its request, naming the field\n     */\n    public static RequestJson read(String record, Object raw) {{\n        Function<Object, RequestJson> reader = READERS.get(record);\n        if (reader == null) {{\n            throw new IllegalArgumentException(\"no typed request for the record \" + record);\n        }}\n        return reader.apply(raw);\n    }}\n}}",
        entries.join(",\n")
    );
    out
}

/// A variant of a union tagged by a field: the union, the field, and the
/// variant's key.
struct Variant<'a> {
    union: &'a str,
    tag: &'a str,
    key: &'a str,
}

/// What the emitter knows besides a record: the enums a request names.
struct Types<'a> {
    api: &'a Api,
    /// Each catalogue kind's enum (`graha` is `Graha`).
    catalogue: BTreeMap<&'a str, String>,
    /// The records that are the data of a union's variant, with the union.
    variants: BTreeMap<&'a str, (&'a str, Option<&'a str>)>,
}

impl<'a> Types<'a> {
    fn of(api: &'a Api) -> Types<'a> {
        let catalogue = api
            .enums
            .iter()
            .filter_map(|e| Some((e.kind.as_deref()?, binding_type_name(&e.name))))
            .collect();
        let mut variants = BTreeMap::new();
        for record in &api.requests.records {
            match &record.shape {
                RecordShape::External { variants: list } => {
                    for variant in list {
                        if let Some(RecordType::Record { name }) = &variant.payload {
                            if is_inline_of(name, &record.name, &variant.key) {
                                variants.insert(
                                    name.as_str(),
                                    (record.name.as_str(), Some(variant.key.as_str())),
                                );
                            }
                        }
                    }
                }
                RecordShape::OrNamed {
                    of: RecordType::Record { name },
                    ..
                } if *name == format!("{}Value", record.name) => {
                    variants.insert(name.as_str(), (record.name.as_str(), None));
                }
                _ => {}
            }
        }
        Types {
            api,
            catalogue,
            variants,
        }
    }

    /// The record `name` names, as Java names its type.
    fn record_type(&self, name: &str) -> String {
        match self.api.requests.enums.get(name) {
            Some(boundary) => format!("{MAIN}.{}", binding_type_name(boundary)),
            None => name.to_owned(),
        }
    }

    /// A catalogue kind's enum, fully named.
    fn member_type(&self, catalogue: &str) -> String {
        let name = self.catalogue.get(catalogue).unwrap_or_else(|| {
            unreachable!(
                "`cargo xtask gen ffi` refuses a request naming no catalogue kind (`{catalogue}`)"
            )
        });
        format!("{MAIN}.{name}")
    }

    /// A type as Java holds it; `boxed` where it may be null or sits in a
    /// list.
    fn java_type(&self, ty: &RecordType, boxed: bool) -> String {
        match ty {
            RecordType::Bool => String::from(if boxed { "Boolean" } else { "boolean" }),
            RecordType::Integer { width } => {
                let (primitive, object) = integer_type(*width);
                String::from(if boxed { object } else { primitive })
            }
            RecordType::Number => String::from(if boxed { "Double" } else { "double" }),
            RecordType::Text => String::from("String"),
            RecordType::Record { name } => self.record_type(name),
            RecordType::List { of } => format!("List<{}>", self.java_type(of, true)),
            RecordType::Pair { first, second } => format!(
                "com.teispace.teistro.record.Pair<{}, {}>",
                self.java_type(first, true),
                self.java_type(second, true)
            ),
            RecordType::Member { catalogue } => self.member_type(catalogue),
            RecordType::Keyed { catalogue } => format!("Keyed<{}>", self.member_type(catalogue)),
            RecordType::Map { of } => format!("Map<String, {}>", self.java_type(of, true)),
            RecordType::Json => String::from("Object"),
        }
    }

    /// What writes `expr`, a non-null value of `ty`, as a JSON tree.
    fn write(&self, ty: &RecordType, expr: &str) -> String {
        self.write_at(ty, expr, 0)
    }

    /// [`Types::write`] inside `depth` lambdas, whose parameters it names
    /// apart.
    fn write_at(&self, ty: &RecordType, expr: &str, depth: usize) -> String {
        let item = format!("item{depth}");
        let value = format!("value{depth}");
        let deeper = depth + 1;
        match ty {
            RecordType::Bool
            | RecordType::Integer { .. }
            | RecordType::Number
            | RecordType::Text
            | RecordType::Json => expr.to_owned(),
            RecordType::Record { name } if self.api.requests.enums.contains_key(name) => {
                format!("{expr}.key()")
            }
            RecordType::Record { .. } | RecordType::Keyed { .. } => format!("{expr}.toJson()"),
            RecordType::Member { .. } => format!("{expr}.fullKey()"),
            RecordType::List { of } if writes_itself(of) => expr.to_owned(),
            RecordType::List { of } => format!(
                "{expr}.stream().map({item} -> {}).toList()",
                self.write_at(of, &item, deeper)
            ),
            RecordType::Pair { first, second } => format!(
                "List.of({}, {})",
                self.write_at(first, &format!("{expr}.first()"), depth),
                self.write_at(second, &format!("{expr}.second()"), depth)
            ),
            RecordType::Map { of } if writes_itself(of) => expr.to_owned(),
            RecordType::Map { of } => format!(
                "Read.mapValues({expr}, {value} -> {})",
                self.write_at(of, &value, deeper)
            ),
        }
    }

    /// What reads `expr`, a non-null JSON tree of `ty`, naming `at` in a
    /// refusal.
    fn read(&self, ty: &RecordType, expr: &str, at: &str) -> String {
        self.read_at(ty, expr, at, 0)
    }

    /// [`Types::read`] inside `depth` lambdas, whose parameters it names
    /// apart.
    fn read_at(&self, ty: &RecordType, expr: &str, at: &str, depth: usize) -> String {
        let item = format!("item{depth}");
        let value = format!("value{depth}");
        let deeper = depth + 1;
        match ty {
            RecordType::Bool => format!("Read.bool({expr}, {at})"),
            RecordType::Integer { width } if integer_type(*width).0 == "int" => {
                format!("Read.int32({expr}, {at})")
            }
            RecordType::Integer { .. } => format!("Read.integer({expr}, {at})"),
            RecordType::Number => format!("Read.number({expr}, {at})"),
            RecordType::Text => format!("Read.text({expr}, {at})"),
            RecordType::Record { name } if self.api.requests.enums.contains_key(name) => format!(
                "Read.member({expr}, {at}, {}::byKey)",
                self.record_type(name)
            ),
            RecordType::Record { name } => format!("{name}.of({expr})"),
            RecordType::List { of } => format!(
                "Read.list({expr}, {at}, {item} -> {})",
                self.read_at(of, &item, at, deeper)
            ),
            RecordType::Pair { first, second } => format!(
                "Read.pair({expr}, {at}, first{depth} -> {}, second{depth} -> {})",
                self.read_at(first, &format!("first{depth}"), at, deeper),
                self.read_at(second, &format!("second{depth}"), at, deeper)
            ),
            RecordType::Member { catalogue } => format!(
                "Read.member({expr}, {at}, {}::byKey)",
                self.member_type(catalogue)
            ),
            RecordType::Keyed { catalogue } => format!(
                "Keyed.of({expr}, {at}, {}::byKey)",
                self.member_type(catalogue)
            ),
            RecordType::Map { of } => format!(
                "Read.map({expr}, {at}, {value} -> {})",
                self.read_at(of, &value, at, deeper)
            ),
            RecordType::Json => expr.to_owned(),
        }
    }

    /// An object, or one variant of a union.
    fn object(
        &self,
        name: &str,
        doc: &str,
        fields: &[RecordField],
        parent: Option<&Variant<'_>>,
    ) -> String {
        let external = self.variants.get(name).copied();
        let mut out = preamble(self.api, REQUEST_PACKAGE);
        out.push_str(
            "import java.util.LinkedHashMap;\nimport java.util.List;\nimport java.util.Map;\nimport java.util.Objects;\n\n",
        );
        let mut full = match (doc.is_empty(), external) {
            (false, _) => unlink(doc),
            (true, Some((union, Some(key)))) => {
                format!("The data of a {{@code {union}}} written under `{key}`.")
            }
            (true, _) => format!("A {{@code {name}}} as a request writes it."),
        };
        full.push('\n');
        for field in fields {
            let _ = write!(
                full,
                "\n@param {} {}",
                identifier(&field.name),
                self.field_doc(field)
            );
        }
        out.push_str(&javadoc(&full, "").replace("&#64;param", "@param"));
        let components: Vec<String> = fields
            .iter()
            .map(|f| {
                format!(
                    "{} {}",
                    self.java_type(&f.ty, f.nullable),
                    identifier(&f.name)
                )
            })
            .collect();
        let mut implements = vec![String::from("RequestJson")];
        if let Some(parent) = parent {
            implements.push(parent.union.to_owned());
        } else if let Some((union, _)) = external {
            implements.push(union.to_owned());
        }
        let _ = writeln!(
            out,
            "public record {name}({}) implements {} {{",
            components.join(", "),
            implements.join(", ")
        );
        if let Some(Variant { tag, key, .. }) = parent {
            let _ = writeln!(
                out,
                "    /** The `{tag}` this variant is written under. */\n    public static final String {} = {};\n",
                key_constant(tag),
                literal(key)
            );
        }
        Self::checks(&mut out, name, fields);
        if fields.iter().any(|f| f.nullable) {
            self.builder(&mut out, name, fields);
        }
        self.decoder(&mut out, name, fields, parent, external);
        self.writer(&mut out, fields, parent, external);
        out.push_str("}\n");
        out
    }

    /// A field's doc: its own, or else its type's, and what a null means
    /// where it may be one.
    fn field_doc(&self, field: &RecordField) -> String {
        let mut ty = &field.ty;
        while let RecordType::List { of } | RecordType::Map { of } = ty {
            ty = of;
        }
        let borrowed = match ty {
            RecordType::Record { name } => self
                .api
                .requests
                .records
                .iter()
                .find(|record| record.name == *name)
                .map(|record| record.doc.as_str()),
            _ => None,
        };
        let mut doc = match (field.doc.as_str(), borrowed) {
            ("", Some(doc)) if !doc.is_empty() => unlink(doc.trim_end()),
            ("", _) => format!("The {{@code {}}} field.", field.name),
            (doc, _) => unlink(doc.trim_end()),
        };
        if field.nullable {
            doc.push_str(" Null leaves the SDK's default.");
        }
        doc
    }

    /// The compact constructor: a required field is present, a list or a
    /// map is copied and so unmodifiable, and a whole number is inside its
    /// width.
    fn checks(out: &mut String, name: &str, fields: &[RecordField]) {
        let mut lines = Vec::new();
        for field in fields {
            let id = identifier(&field.name);
            let at = format!("{name}.{}", field.name);
            let boxed_or_object = field.nullable || !is_primitive(&field.ty);
            if !field.nullable && boxed_or_object {
                lines.push(format!(
                    "        Objects.requireNonNull({id}, {});",
                    literal(&format!("`{at}` is required"))
                ));
            }
            let copy = match &field.ty {
                RecordType::List { .. } => Some("List.copyOf"),
                RecordType::Map { .. } => Some("Map.copyOf"),
                _ => None,
            };
            if let Some(copy) = copy {
                lines.push(if field.nullable {
                    format!("        {id} = {id} == null ? null : {copy}({id});")
                } else {
                    format!("        {id} = {copy}({id});")
                });
            }
            if let RecordType::Integer { width: Some(width) } = &field.ty {
                if let Some((min, max)) = bounds(*width) {
                    let check = format!("Read.range({id}, {}, {min}L, {max}L);", literal(&at));
                    lines.push(if field.nullable {
                        format!("        if ({id} != null) {{\n            {check}\n        }}")
                    } else {
                        format!("        {check}")
                    });
                }
            }
        }
        if lines.is_empty() {
            return;
        }
        let _ = writeln!(
            out,
            "    /**\n     * Checks a required field is present and a whole number fits its width, and\n     * copies a list or a map so the record's own cannot change.\n     *\n     * @throws IllegalArgumentException naming the field that is out of range\n     * @throws NullPointerException naming the required field that is missing\n     */\n    public {name} {{\n{}\n    }}\n",
            lines.join("\n")
        );
    }

    /// A builder, for a record with optional fields: each setter takes the
    /// unboxed value, so a literal passes.
    fn builder(&self, out: &mut String, name: &str, fields: &[RecordField]) {
        let _ = writeln!(
            out,
            "    /**\n     * A builder, every field unset: an unset optional field is left out, and the\n     * SDK's default stands.\n     *\n     * @return the builder\n     */\n    public static Builder builder() {{\n        return new Builder();\n    }}\n\n    /** Builds a {{@link {name}}} a field at a time. */\n    public static final class Builder {{"
        );
        for field in fields {
            let _ = writeln!(
                out,
                "        private {} {};",
                self.java_type(&field.ty, true),
                identifier(&field.name)
            );
        }
        let _ = writeln!(out, "\n        private Builder() {{}}\n");
        for field in fields {
            let id = identifier(&field.name);
            let _ = writeln!(
                out,
                "        /**\n         * Sets {{@code {}}}.\n         *\n         * @param {id} {}\n         * @return this builder\n         */\n        public Builder {id}({} {id}) {{\n            this.{id} = {id};\n            return this;\n        }}\n",
                field.name,
                javadoc_line(&self.field_doc(field)),
                self.java_type(&field.ty, false),
            );
        }
        let arguments: Vec<String> = fields
            .iter()
            .map(|f| {
                let id = identifier(&f.name);
                if f.nullable || !is_primitive(&f.ty) {
                    id
                } else {
                    format!(
                        "Objects.requireNonNull({id}, {})",
                        literal(&format!("`{name}.{}` is required", f.name))
                    )
                }
            })
            .collect();
        let _ = writeln!(
            out,
            "        /**\n         * The record.\n         *\n         * @return the record\n         * @throws NullPointerException naming a required field left unset\n         */\n        public {name} build() {{\n            return new {name}({});\n        }}\n    }}\n",
            arguments.join(", ")
        );
    }

    /// `of`: the record from its JSON tree.
    fn decoder(
        &self,
        out: &mut String,
        name: &str,
        fields: &[RecordField],
        parent: Option<&Variant<'_>>,
        external: Option<(&str, Option<&str>)>,
    ) {
        let reads: Vec<String> = fields
            .iter()
            .map(|f| {
                let at = literal(&format!("{name}.{}", f.name));
                let got = format!("json.get({})", literal(&f.name));
                if f.nullable {
                    format!(
                        "Read.orNull({got}, present -> {})",
                        self.read(&f.ty, "present", &at)
                    )
                } else {
                    self.read(&f.ty, &format!("Read.required({got}, {at})"), &at)
                }
            })
            .collect();
        let what = match (parent, external) {
            (Some(Variant { tag, .. }), _) => format!(", with its `{tag}`"),
            (_, Some((_, Some(key)))) => format!(", the object under `{key}`"),
            _ => String::new(),
        };
        let _ = writeln!(
            out,
            "    /**\n     * A {{@code {name}}} from its JSON{what}, as {{@code Json.read}} answers it.\n     *\n     * @param raw the JSON value\n     * @return the record\n     * @throws IllegalArgumentException for JSON that is not one, naming the field\n     */\n    public static {name} of(Object raw) {{\n        Map<String, Object> json = Read.object(raw, {});\n        return new {name}({});\n    }}\n",
            literal(name),
            if reads.is_empty() {
                String::new()
            } else {
                format!("\n            {}", reads.join(",\n            "))
            }
        );
    }

    /// `toJson`: the record as the tree `Json.write` writes, a null field
    /// left out.
    fn writer(
        &self,
        out: &mut String,
        fields: &[RecordField],
        parent: Option<&Variant<'_>>,
        external: Option<(&str, Option<&str>)>,
    ) {
        let mut body = String::from("        Map<String, Object> json = new LinkedHashMap<>();\n");
        if let Some(Variant { tag, .. }) = parent {
            let _ = writeln!(
                body,
                "        json.put({}, {});",
                literal(tag),
                key_constant(tag)
            );
        }
        for field in fields {
            let id = identifier(&field.name);
            let put = format!(
                "json.put({}, {});",
                literal(&field.name),
                self.write(&field.ty, &id)
            );
            if field.nullable {
                let _ = writeln!(
                    body,
                    "        if ({id} != null) {{\n            {put}\n        }}"
                );
            } else {
                let _ = writeln!(body, "        {put}");
            }
        }
        match external {
            Some((_, Some(key))) => {
                let _ = writeln!(body, "        return Map.of({}, json);", literal(key));
            }
            _ => body.push_str("        return json;\n"),
        }
        let _ = writeln!(
            out,
            "    @Override\n    public Object toJson() {{\n{body}    }}"
        );
    }

    /// A closed set of keys, as an enum of them.
    fn keys(&self, name: &str, doc: &str, keys: &[(String, String)]) -> String {
        let mut out = preamble(self.api, REQUEST_PACKAGE);
        out.push_str(&javadoc(&described(doc, name), ""));
        let _ = writeln!(out, "public enum {name} implements RequestJson {{");
        out.push_str(&Self::constants(keys));
        let _ = writeln!(out, ";\n\n{}\n}}", key_methods(name));
        out
    }

    /// An enum's constants, each with its key.
    fn constants(keys: &[(String, String)]) -> String {
        let mut seen = Vec::new();
        keys.iter()
            .map(|(key, doc)| {
                let constant = key_constant(key);
                assert!(
                    !seen.contains(&constant),
                    "two keys are both `{constant}` in Java"
                );
                seen.push(constant.clone());
                let doc = if doc.is_empty() {
                    format!("{{@code {key}}}.")
                } else {
                    unlink(doc)
                };
                format!("{}    {constant}({})", javadoc(&doc, "    "), literal(key))
            })
            .collect::<Vec<_>>()
            .join(",\n")
    }

    /// A union tagged by a field, as a sealed interface of its variants.
    fn tagged(&self, name: &str, doc: &str, tag: &str, permits: &[(String, String)]) -> String {
        let mut out = preamble(self.api, REQUEST_PACKAGE);
        out.push_str("import java.util.Map;\n\n");
        out.push_str(&javadoc(&described(doc, name), ""));
        let names: Vec<&str> = permits.iter().map(|(_, name)| name.as_str()).collect();
        let _ = writeln!(
            out,
            "public sealed interface {name} extends RequestJson permits {} {{\n    /**\n     * A {{@code {name}}} from its JSON, by its `{tag}`.\n     *\n     * @param raw the JSON value\n     * @return the variant\n     * @throws IllegalArgumentException for JSON that is none of them\n     */\n    static {name} of(Object raw) {{\n        Object which = Read.object(raw, {}).get({});",
            names.join(", "),
            literal(name),
            literal(tag)
        );
        for (key, variant) in permits {
            let _ = writeln!(
                out,
                "        if ({}.equals(which)) {{\n            return {variant}.of(raw);\n        }}",
                literal(key)
            );
        }
        let _ = writeln!(
            out,
            "        throw new IllegalArgumentException(\"not a {name}: {tag} \" + which);\n    }}\n}}"
        );
        out
    }

    /// A key or a one-key object: a sealed interface of an enum of the keys
    /// and a record a data variant.
    fn external(
        &self,
        name: &str,
        doc: &str,
        variants: &[ExternalVariant],
    ) -> Vec<(String, String)> {
        let keys: Vec<(String, String)> = variants
            .iter()
            .filter(|v| v.payload.is_none())
            .map(|v| (v.key.clone(), v.doc.clone()))
            .collect();
        let mut files = Vec::new();
        let mut permits = Vec::new();
        let mut body = String::new();
        if !keys.is_empty() {
            permits.push(format!("{name}.Key"));
            for (key, doc) in &keys {
                let constant = key_constant(key);
                let _ = writeln!(
                    body,
                    "{}    {name} {constant} = Key.{constant};\n",
                    javadoc(&unlink(doc), "    ")
                );
            }
        }
        let mut reads = String::new();
        for variant in variants {
            let Some(payload) = &variant.payload else {
                continue;
            };
            let record = match payload {
                RecordType::Record { name: data } if self.variants.contains_key(data.as_str()) => {
                    data.clone()
                }
                _ => {
                    let wrapper = format!("{name}{}", part(&variant.key));
                    files.push((
                        wrapper.clone(),
                        self.wrapper(&wrapper, name, &variant.key, &variant.doc, payload),
                    ));
                    wrapper
                }
            };
            permits.push(record.clone());
            let _ = writeln!(
                reads,
                "            if (json.containsKey({key})) {{\n                return {record}.of(json.get({key}));\n            }}",
                key = literal(&variant.key)
            );
        }
        let mut out = preamble(self.api, REQUEST_PACKAGE);
        out.push_str("import java.util.Map;\n\n");
        out.push_str(&javadoc(&described(doc, name), ""));
        let _ = writeln!(
            out,
            "public sealed interface {name} extends RequestJson permits {} {{\n{body}",
            permits.join(", ")
        );
        let _ = writeln!(
            out,
            "    /**\n     * A {{@code {name}}} from its JSON: a key, or an object of one key.\n     *\n     * @param raw the JSON value\n     * @return the variant\n     * @throws IllegalArgumentException for JSON that is none of them\n     */\n    static {name} of(Object raw) {{"
        );
        if !keys.is_empty() {
            let _ = writeln!(
                out,
                "        if (raw instanceof String) {{\n            return Key.of(raw);\n        }}"
            );
        }
        let _ = writeln!(
            out,
            "        Map<String, Object> json = Read.object(raw, {});\n        if (json.size() == 1) {{\n{reads}        }}\n        throw new IllegalArgumentException(\"not a {name}: \" + raw);\n    }}",
            literal(name)
        );
        if !keys.is_empty() {
            let _ = writeln!(
                out,
                "\n    /** The variants written as a bare key. */\n    enum Key implements {name} {{\n{};\n\n{}\n    }}",
                indent(&Self::constants(&keys), "    "),
                indent(&key_methods("Key"), "    ")
            );
        }
        out.push_str("}\n");
        files.push((name.to_owned(), out));
        files
    }

    /// A data variant whose data is not a record of its own: a record of
    /// one component, written as `{KEY: value}`.
    fn wrapper(
        &self,
        name: &str,
        union: &str,
        key: &str,
        doc: &str,
        payload: &RecordType,
    ) -> String {
        let field = RecordField {
            name: camel_key(key),
            doc: if doc.is_empty() {
                String::from("The value.")
            } else {
                doc.to_owned()
            },
            ty: payload.clone(),
            nullable: false,
        };
        let mut out = preamble(self.api, REQUEST_PACKAGE);
        out.push_str(
            "import java.util.List;\nimport java.util.Map;\nimport java.util.Objects;\n\n",
        );
        let id = identifier(&field.name);
        out.push_str(
            &javadoc(
                &format!(
                    "{}\n\n@param {id} {}",
                    described(doc, name),
                    javadoc_line(&unlink(&field.doc))
                ),
                "",
            )
            .replace("&#64;param", "@param"),
        );
        let _ = writeln!(
            out,
            "public record {name}({} {id}) implements RequestJson, {union} {{",
            self.java_type(payload, false)
        );
        Self::checks(&mut out, name, std::slice::from_ref(&field));
        let _ = writeln!(
            out,
            "    /**\n     * A {{@code {name}}} from the value under `{key}`.\n     *\n     * @param raw the JSON value\n     * @return the record\n     * @throws IllegalArgumentException for JSON that is not one\n     */\n    public static {name} of(Object raw) {{\n        return new {name}({});\n    }}\n\n    @Override\n    public Object toJson() {{\n        return Map.of({}, {});\n    }}\n}}",
            self.read(
                payload,
                "Read.required(raw, \"value\")",
                &literal(&format!("{name}.{}", field.name))
            ),
            literal(key),
            self.write(payload, &id)
        );
        out
    }

    /// A few names or one value: a sealed interface of an enum of the names
    /// and the value, the value made by `value`.
    fn or_named(
        &self,
        name: &str,
        doc: &str,
        names: &[String],
        of: &RecordType,
    ) -> Vec<(String, String)> {
        let mut files = Vec::new();
        let value = match of {
            RecordType::Record { name: data } if self.variants.contains_key(data.as_str()) => {
                data.clone()
            }
            _ => {
                let wrapper = format!("{name}Value");
                let mut out = preamble(self.api, REQUEST_PACKAGE);
                out.push_str(
                    "import java.util.List;\nimport java.util.Map;\nimport java.util.Objects;\n\n",
                );
                let field = RecordField {
                    name: String::from("value"),
                    doc: String::from("The value."),
                    ty: of.clone(),
                    nullable: false,
                };
                out.push_str(
                    &javadoc(
                        &format!("{}\n\n@param value the value", described(doc, name)),
                        "",
                    )
                    .replace("&#64;param", "@param"),
                );
                let _ = writeln!(
                    out,
                    "public record {wrapper}({} value) implements RequestJson, {name} {{",
                    self.java_type(of, false)
                );
                Self::checks(&mut out, &wrapper, std::slice::from_ref(&field));
                let _ = writeln!(
                    out,
                    "    /**\n     * A {{@code {wrapper}}} from its JSON.\n     *\n     * @param raw the JSON value\n     * @return the record\n     * @throws IllegalArgumentException for JSON that is not one\n     */\n    public static {wrapper} of(Object raw) {{\n        return new {wrapper}({});\n    }}\n\n    @Override\n    public Object toJson() {{\n        return {};\n    }}\n}}",
                    self.read(of, "raw", &literal(&format!("{name}.value"))),
                    self.write(of, "value")
                );
                files.push((wrapper.clone(), out));
                wrapper
            }
        };
        let keys: Vec<(String, String)> =
            names.iter().map(|n| (n.clone(), String::new())).collect();
        let mut out = preamble(self.api, REQUEST_PACKAGE);
        out.push_str("import java.util.List;\nimport java.util.Map;\n\n");
        out.push_str(&javadoc(&described(doc, name), ""));
        let mut constants = String::new();
        for (key, _) in &keys {
            let constant = key_constant(key);
            let _ = writeln!(
                constants,
                "    /** {{@code {key}}}. */\n    {name} {constant} = Key.{constant};\n"
            );
        }
        let value_type =
            if value == format!("{name}Value") && !self.variants.contains_key(value.as_str()) {
                self.java_type(of, false)
            } else {
                value.clone()
            };
        let make = if value_type == value {
            String::new()
        } else {
            format!(
                "    /**\n     * The value, as a {{@code {name}}}.\n     *\n     * @param value the value\n     * @return the union\n     */\n    static {name} value({value_type} value) {{\n        return new {value}(value);\n    }}\n\n"
            )
        };
        let _ = writeln!(
            out,
            "public sealed interface {name} extends RequestJson permits {name}.Key, {value} {{\n{constants}{make}    /**\n     * A {{@code {name}}} from its JSON: one of the names, or the value.\n     *\n     * @param raw the JSON value\n     * @return the union\n     * @throws IllegalArgumentException for JSON that is neither\n     */\n    static {name} of(Object raw) {{\n        for (Key key : Key.values()) {{\n            if (key.key().equals(raw)) {{\n                return key;\n            }}\n        }}\n        return {value}.of(raw);\n    }}\n\n    /** The names. */\n    enum Key implements {name} {{\n{};\n\n{}\n    }}\n}}",
            indent(&Self::constants(&keys), "    "),
            indent(&key_methods("Key"), "    ")
        );
        files.push((name.to_owned(), out));
        files
    }
}

/// The methods every key enum has: its key, `of`, and `toJson`.
fn key_methods(name: &str) -> String {
    format!(
        "    private final String key;\n\n    {name}(String key) {{\n        this.key = key;\n    }}\n\n    /**\n     * The key the SDK reads.\n     *\n     * @return the key\n     */\n    public String key() {{\n        return key;\n    }}\n\n    /**\n     * The member a key names.\n     *\n     * @param raw the JSON value\n     * @return the member\n     * @throws IllegalArgumentException for a key the SDK does not read\n     */\n    public static {name} of(Object raw) {{\n        for ({name} member : values()) {{\n            if (member.key.equals(raw)) {{\n                return member;\n            }}\n        }}\n        throw new IllegalArgumentException(\"not a {name}: \" + raw);\n    }}\n\n    @Override\n    public Object toJson() {{\n        return key;\n    }}"
    )
}

/// A type's doc, or a line naming it where its source has none.
fn described(doc: &str, name: &str) -> String {
    if doc.is_empty() {
        format!("A {{@code {name}}} as a request writes it.")
    } else {
        unlink(doc)
    }
}

/// A Rust doc's intra-doc links as their code spans: ``[`Bhaga`](crate::Bhaga)``
/// and ``[`Bhaga`]`` are both `` `Bhaga` ``, since the path names a Rust
/// item a Java reader cannot follow.
fn unlink(doc: &str) -> String {
    let mut out = String::with_capacity(doc.len());
    let mut rest = doc;
    while let Some(start) = rest.find("[`") {
        let Some(len) = rest[start + 2..].find("`]") else {
            break;
        };
        out.push_str(&rest[..start]);
        out.push('`');
        out.push_str(&rest[start + 2..start + 2 + len]);
        out.push('`');
        rest = &rest[start + 2 + len + 2..];
        if let Some(target) = rest
            .strip_prefix('(')
            .and_then(|r| r.find(')').map(|end| &r[end + 1..]))
        {
            rest = target;
        }
    }
    out.push_str(rest);
    out
}

/// Every line of `text` indented by `by`, blank lines left blank.
fn indent(text: &str, by: &str) -> String {
    text.lines()
        .map(|line| {
            if line.is_empty() {
                String::new()
            } else {
                format!("{by}{line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// A doc as one line of a Javadoc tag.
fn javadoc_line(text: &str) -> String {
    javadoc(text, "")
        .lines()
        .filter_map(|line| line.strip_prefix(" * "))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Whether an inline payload record was named after this union's variant.
fn is_inline_of(name: &str, union: &str, key: &str) -> bool {
    name == format!("{union}{}", part(key))
}

/// Whether a type is held unboxed when it is required.
fn is_primitive(ty: &RecordType) -> bool {
    matches!(
        ty,
        RecordType::Bool | RecordType::Integer { .. } | RecordType::Number
    )
}

/// Whether a value of the type is already the tree `Json.write` writes.
fn writes_itself(ty: &RecordType) -> bool {
    matches!(
        ty,
        RecordType::Bool
            | RecordType::Integer { .. }
            | RecordType::Number
            | RecordType::Text
            | RecordType::Json
    )
}

/// A whole number's Java types: `int` where every value fits, `long`
/// otherwise; an unsigned 64-bit one is a `long` the caller keeps
/// non-negative.
fn integer_type(width: Option<Width>) -> (&'static str, &'static str) {
    match width {
        Some(Width { signed: true, bits }) if bits <= 32 => ("int", "Integer"),
        Some(Width {
            signed: false,
            bits,
        }) if bits <= 16 => ("int", "Integer"),
        _ => ("long", "Long"),
    }
}

/// A width's range, where Java's type is wider than it.
fn bounds(width: Width) -> Option<(i64, i64)> {
    match (width.signed, width.bits) {
        (_, 64) | (true, 32) => None,
        (false, bits) => Some((0, (1_i64 << bits) - 1)),
        (true, bits) => Some((-(1_i64 << (bits - 1)), (1_i64 << (bits - 1)) - 1)),
    }
}

/// A key as a Java constant: upper-cased, a separator an underscore, and
/// an underscore before a leading digit.
fn key_constant(key: &str) -> String {
    let upper: String = key
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect();
    if upper.starts_with(|c: char| c.is_ascii_digit()) {
        format!("_{upper}")
    } else {
        upper
    }
}

/// A key as part of a type's name (`WITHIN` is `Within`), as the records
/// reader names an inline type.
fn part(key: &str) -> String {
    let key = key.replace('-', "_");
    if key.chars().any(|c| c.is_ascii_lowercase()) {
        pascal(&key)
    } else {
        pascal(&key.to_ascii_lowercase())
    }
}

/// A key as a field's name (`DAYS` is `days`).
fn camel_key(key: &str) -> String {
    let named = part(key);
    let mut chars = named.chars();
    chars
        .next()
        .map(|first| first.to_lowercase().chain(chars).collect())
        .unwrap_or_default()
}

const REQUEST_JSON: &str = r"/**
 * A request record: it writes itself as the JSON tree {@code Json.write}
 * writes, a null field left out so the SDK's default stands.
 */
public interface RequestJson {
    /**
     * The record as a JSON tree: a {@code Map}, a {@code List}, a string, a
     * number, a boolean.
     *
     * @return the tree
     */
    Object toJson();
}
";

const KEYED: &str = r#"import java.util.Optional;
import java.util.function.Function;

/**
 * A member of a catalogue, or the key of one a context registered: written
 * as the member's full key, or the key as it was given.
 *
 * @param member the catalogue's member, or null for a registered key
 * @param key the registered key, or null for a member
 * @param <M> the catalogue's enum
 */
public record Keyed<M extends com.teispace.teistro.Catalogued>(M member, String key) implements RequestJson {
    /**
     * Checks exactly one of the two is given.
     *
     * @throws IllegalArgumentException when both or neither is
     */
    public Keyed {
        if ((member == null) == (key == null)) {
            throw new IllegalArgumentException("a keyed value is a member or a registered key, one of the two");
        }
    }

    /**
     * A catalogue's member.
     *
     * @param member the member
     * @param <M> the catalogue's enum
     * @return the value
     */
    public static <M extends com.teispace.teistro.Catalogued> Keyed<M> of(M member) {
        return new Keyed<>(member, null);
    }

    /**
     * A member a context registered, by its key, bare or full.
     *
     * @param key the key
     * @param <M> the catalogue's enum
     * @return the value
     */
    public static <M extends com.teispace.teistro.Catalogued> Keyed<M> registered(String key) {
        return new Keyed<>(null, key);
    }

    /**
     * A keyed value from its JSON: the catalogue's member where it names
     * one, a registered key otherwise.
     *
     * @param raw the JSON value
     * @param at where it is, for a refusal
     * @param byKey the catalogue's lookup
     * @param <M> the catalogue's enum
     * @return the value
     * @throws IllegalArgumentException for JSON that is not a key
     */
    public static <M extends com.teispace.teistro.Catalogued> Keyed<M> of(Object raw, String at, Function<String, Optional<M>> byKey) {
        String key = Read.text(raw, at);
        return byKey.apply(key).map(Keyed::of).orElseGet(() -> Keyed.registered(key));
    }

    @Override
    public Object toJson() {
        return member != null ? member.fullKey() : key;
    }
}
"#;

/// The request package's `Read`: the records' checks, with what a
/// request's fields add (a width, a member, a map).
fn read() -> String {
    let shared = READ
        .replace("Pair<", "com.teispace.teistro.record.Pair<")
        .replace("import java.util.ArrayList;\n", "import java.util.ArrayList;\nimport java.util.LinkedHashMap;\nimport java.util.Optional;\n");
    let end = shared.rfind("}\n").unwrap_or(shared.len());
    format!("{}{READ_MORE}}}\n", &shared[..end])
}

const READ_MORE: &str = r#"
    static int int32(Object raw, String at) {
        long value = integer(raw, at);
        range(value, at, Integer.MIN_VALUE, Integer.MAX_VALUE);
        return (int) value;
    }

    static void range(long value, String at, long min, long max) {
        if (value < min || value > max) {
            throw new IllegalArgumentException("`" + at + "` is " + value + ", outside " + min + " to " + max);
        }
    }

    static <M> M member(Object raw, String at, Function<String, Optional<M>> byKey) {
        String key = text(raw, at);
        return byKey.apply(key).orElseThrow(() -> refused(at, "a key the SDK reads", raw));
    }

    static <V> Map<String, V> map(Object raw, String at, Function<Object, V> value) {
        Map<String, V> read = new LinkedHashMap<>();
        for (Map.Entry<String, Object> entry : object(raw, at).entrySet()) {
            read.put(entry.getKey(), value.apply(entry.getValue()));
        }
        return Collections.unmodifiableMap(read);
    }

    static <V> Map<String, Object> mapValues(Map<String, V> map, Function<V, Object> value) {
        Map<String, Object> written = new LinkedHashMap<>();
        for (Map.Entry<String, V> entry : map.entrySet()) {
            written.put(entry.getKey(), value.apply(entry.getValue()));
        }
        return written;
    }
"#;
