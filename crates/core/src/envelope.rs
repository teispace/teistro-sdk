//! The result envelope: every value the SDK returns travels with the
//! provenance that reproduces it (ADR-0020): versions, the calculation
//! version, hashes of the input and the settings, the provider and the
//! packs, what the time layer applied, the conventions the SDK had to
//! choose, and every warning. The cache key a consumer uses is
//! `(input_hash, settings_hash, calculation_version)`.

use core::fmt::Write as _;

use core::fmt;
use std::borrow::Cow;

use sha2::{Digest, Sha256};

/// The calculation version (ADR-0020): bumped, with a **Numbers** entry
/// in the changelog, whenever any computed number moves; consumers cache
/// on `(input_hash, settings_hash, calculation_version)`.
///
/// ```
/// use teistro_core::envelope::CALCULATION_VERSION;
///
/// assert!(CALCULATION_VERSION >= 1);
/// ```
pub const CALCULATION_VERSION: u32 = 1;

/// The most decimals a **rendering** may ask a number to be written to.
///
/// A double carries about seventeen significant decimal digits, and the
/// SDK's quantities are degrees, days and scores under a million: twelve
/// decimals is finer than a nanoarcsecond and coarser than the noise.
///
/// It is not what the **hash** form writes, which is
/// [`Digits::Shortest`] — a fixed count is not a fixed point, and
/// `03-design/schema-measured.md` §9 measured where it stops being one.
///
/// ```
/// use teistro_core::envelope::{CANONICAL_DECIMALS, canonical_json_at};
///
/// assert_eq!(canonical_json_at(&(1.0_f64 / 3.0), CANONICAL_DECIMALS), "0.333333333333");
/// ```
pub const CANONICAL_DECIMALS: u8 = 12;

/// How a number is written in the canonical form.
///
/// ```
/// use teistro_core::envelope::{Digits, decimal};
///
/// assert_eq!(decimal(0.1 + 0.2, Digits::Shortest), "0.30000000000000004");
/// assert_eq!(decimal(0.1 + 0.2, Digits::Rounded(3)), "0.3");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Digits {
    /// The shortest decimal that reads back as the very same double.
    ///
    /// This is what the **hash** form writes, and it is the only choice
    /// that makes the form a *fixed point*: writing what it wrote gives
    /// the same bytes. A fixed count of decimals does not, because a
    /// double's resolution depends on its magnitude — at a Julian day's
    /// 2.46e6 one unit in the last place is already about 5e-10, so
    /// twelve decimals writes three digits that are the expansion of a
    /// binary value rather than information, and they do not survive a
    /// parse. A consumer that stores a document and hashes it again
    /// would not get the producer's hash, which is the one thing this
    /// form exists to guarantee.
    Shortest,
    /// Rounded to this many decimals, trailing zeros removed.
    ///
    /// This is what a **rendering** writes, and it is never hashed.
    Rounded(u8),
}

/// The canonical JSON of a value: every object's keys in code-point
/// order, no whitespace, and every number as a plain decimal — so two
/// builds and two bindings that agree on the value agree on the bytes
/// and therefore on the hash (ADR-0022).
///
/// Two things are explicit here rather than left to the JSON layer, and
/// both were measured (`03-design/serial-and-the-envelope.md` §3).
///
/// The **sort** is explicit because the JSON layer's map ordering
/// changes with the `preserve_order` feature that any crate in a build
/// may enable.
///
/// The **number grammar** is explicit because Rust's layer writes
/// `1e-6` where JavaScript's writes `0.000001`: two bindings that agree
/// about the number would disagree about the bytes. A number here is a
/// decimal with an optional leading `-`, at least one digit before the
/// point, and the fewest digits after it that still read back as the
/// same double, with trailing zeros removed — never an exponent, never
/// a bare `.5`, never `-0`. Rust's own `Display` for a double is exactly
/// that shortest form and never writes an exponent; JavaScript's
/// `toString` is the same shortest form but *does* write an exponent
/// outside 1e-6 to 1e21, so a binding has to expand those two ranges and
/// needs no float printer of its own beyond that.
///
/// It is **exact**: two doubles that differ at all are written
/// differently, and a value read back from the form writes the same
/// bytes again. A form that rounded would be lossy in a way that is
/// worse than useless here — it would make the hash unreproducible by
/// the consumer storing it (`03-design/schema-measured.md` §9).
///
/// ```
/// use teistro_core::envelope::canonical_json;
///
/// let value = serde_json::json!({"b": 1e-6, "a": [true, null, -0.0]});
/// assert_eq!(canonical_json(&value), r#"{"a":[true,null,0],"b":0.000001}"#);
/// ```
#[must_use]
pub fn canonical_json<T: serde::Serialize + ?Sized>(value: &T) -> String {
    written(value, Digits::Shortest)
}

/// The same, at a chosen number of decimals: what a **rendering** uses,
/// which is never hashed.
///
/// ```
/// use teistro_core::envelope::canonical_json_at;
///
/// assert_eq!(canonical_json_at(&[2.5_f64, 1.0 / 3.0], 2), "[2.5,0.33]");
/// ```
#[must_use]
pub fn canonical_json_at<T: serde::Serialize + ?Sized>(value: &T, decimals: u8) -> String {
    written(value, Digits::Rounded(decimals))
}

/// The canonical bytes of a value with its numbers written one way or
/// the other. Both entry points above are this with their own choice.
///
/// A value serde cannot write is a type defect and never a runtime
/// condition, so it is not an error a caller handles: in a debug build —
/// every test — it stops there, naming what serde said, and a release
/// build writes the empty string. It is loud because it was once silent:
/// a varga scheme serde could not write sealed every document carrying it
/// with the hash of nothing, and nothing noticed.
fn written<T: serde::Serialize + ?Sized>(value: &T, digits: Digits) -> String {
    match canonical::write(value, digits) {
        Ok(text) => text,
        Err(error) => {
            debug_assert!(false, "a value serde cannot write: {error}");
            String::new()
        }
    }
}

/// The hash of a value's canonical JSON.
///
/// ```
/// use teistro_core::envelope::{Hash, canonical_json, content_hash};
///
/// let value = serde_json::json!({"tithi": 3, "paksha": "SHUKLA"});
/// assert_eq!(content_hash(&value), Hash::of(canonical_json(&value).as_bytes()));
/// ```
#[must_use]
pub fn content_hash<T: serde::Serialize + ?Sized>(value: &T) -> Hash {
    Hash::of(canonical_json(value).as_bytes())
}

/// The hash of a list's canonical JSON **and** of each item's, from one
/// serialisation.
///
/// A list's canonical form is `[`, its items' canonical forms joined by
/// `,`, and `]`, so the list's hash can be taken over the items' own bytes
/// as they are written: the first is exactly [`content_hash`] of the
/// slice, and each of the rest is exactly [`content_hash`] of its item.
/// What a batch answers with this is its own hash and every member's,
/// for the price of the batch's — which is what a caller holding one
/// member of a batch needs to stamp it with its own value's hash
/// (`03-design/serial-and-the-envelope.md` §3).
///
/// ```
/// use teistro_core::envelope::{content_hash, content_hashes};
///
/// let items = [1.5_f64, 2.0, -0.25];
/// let (list, each) = content_hashes(&items);
/// assert_eq!(list, content_hash(&items[..]));
/// assert_eq!(each[2], content_hash(&-0.25_f64));
/// ```
#[must_use]
pub fn content_hashes<T: serde::Serialize>(items: &[T]) -> (Hash, Vec<Hash>) {
    let mut list = Sha256::new();
    list.update(b"[");
    let mut each = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            list.update(b",");
        }
        let written = canonical_json(item);
        list.update(written.as_bytes());
        each.push(Hash::of(written.as_bytes()));
    }
    list.update(b"]");
    (Hash(list.finalize().into()), each)
}

mod canonical;

/// A value in the canonical form, at a given number of decimals, written
/// from a `serde_json::Value` tree: the writer the form was first defined
/// by, kept as the reference [`canonical::write`] is held to.
#[cfg(test)]
fn write_canonical(value: &serde_json::Value, digits: Digits) -> String {
    let mut out = String::new();
    push_canonical(value, digits, &mut out);
    out
}

#[cfg(test)]
fn push_canonical(value: &serde_json::Value, digits: Digits, out: &mut String) {
    use serde_json::Value;
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Number(number) => push_number(number, digits, out),
        Value::String(text) => push_string(text, out),
        Value::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                push_canonical(item, digits, out);
            }
            out.push(']');
        }
        Value::Object(fields) => {
            // Code-point order, explicitly, because the JSON layer's map
            // ordering changes with a feature any crate may enable.
            let mut keys: Vec<&String> = fields.keys().collect();
            keys.sort_unstable();
            out.push('{');
            for (index, key) in keys.into_iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                push_string(key, out);
                out.push(':');
                if let Some(nested) = fields.get(key) {
                    push_canonical(nested, digits, out);
                }
            }
            out.push('}');
        }
    }
}

/// A number in the grammar: a decimal, never an exponent, trailing zeros
/// trimmed, and no negative nought.
#[cfg(test)]
fn push_number(number: &serde_json::Number, digits: Digits, out: &mut String) {
    if let Some(integer) = number.as_i64() {
        let _ = write!(out, "{integer}");
    } else if let Some(integer) = number.as_u64() {
        let _ = write!(out, "{integer}");
    } else {
        out.push_str(
            &number
                .as_f64()
                .map_or_else(|| String::from("0"), |double| decimal(double, digits)),
        );
    }
}

/// A double as a plain decimal.
///
/// ```
/// use teistro_core::envelope::{Digits, decimal};
///
/// assert_eq!(decimal(1e21, Digits::Shortest), "1000000000000000000000");
/// assert_eq!(decimal(-0.0, Digits::Shortest), "0");
/// assert_eq!(decimal(-0.0004, Digits::Rounded(3)), "0");
/// assert_eq!(decimal(f64::NAN, Digits::Shortest), "0");
/// ```
#[must_use]
pub fn decimal(value: f64, digits: Digits) -> String {
    let mut out = String::new();
    push_decimal(value, digits, &mut out);
    out
}

/// [`decimal`], written onto the end of `out`.
fn push_decimal(value: f64, digits: Digits, out: &mut String) {
    if !value.is_finite() {
        // JSON has no infinity and no NaN; a value that reached here is
        // a caller's mistake and nought is the only representable
        // answer. `Derived::at` and its like refuse such a value long
        // before this.
        out.push('0');
        return;
    }
    let start = out.len();
    let _ = match digits {
        // Rust's `Display` for a double is the shortest decimal that
        // reads back as the same value, and it never writes an exponent,
        // which is this grammar's other rule. Nothing to do but ask for
        // it.
        Digits::Shortest => write!(out, "{value}"),
        Digits::Rounded(decimals) => write!(out, "{value:.*}", usize::from(decimals)),
    };
    let written = out.get(start..).unwrap_or_default();
    let kept = if written.contains('.') {
        written.trim_end_matches('0').trim_end_matches('.').len()
    } else {
        written.len()
    };
    out.truncate(start + kept);
    // `-0`, `-0.0` and the like are one value with one spelling.
    if matches!(out.get(start..), Some("-0" | "")) {
        out.truncate(start);
        out.push('0');
    }
}

/// A string with the escapes JSON requires and no others, so two writers
/// agree. Most strings need none, and are copied whole.
fn push_string(text: &str, out: &mut String) {
    out.push('"');
    if !text
        .bytes()
        .any(|byte| byte < b' ' || byte == b'"' || byte == b'\\')
    {
        out.push_str(text);
        out.push('"');
        return;
    }
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            control if control < ' ' => {
                let _ = write!(out, "\\u{:04x}", u32::from(control));
            }
            other => out.push(other),
        }
    }
    out.push('"');
}

/// A semantic version.
///
/// ```
/// use teistro_core::envelope::Version;
///
/// assert!(Version::new(1, 10, 0) > Version::new(1, 9, 9));
/// assert_eq!(Version::new(0, 4, 2).to_string(), "0.4.2");
/// ```
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Version {
    /// Major.
    pub major: u16,
    /// Minor.
    pub minor: u16,
    /// Patch.
    pub patch: u16,
}

impl Version {
    /// A version.
    ///
    /// ```
    /// use teistro_core::envelope::Version;
    ///
    /// let version = Version::new(2, 0, 1);
    /// assert_eq!((version.major, version.minor, version.patch), (2, 0, 1));
    /// ```
    #[must_use]
    pub const fn new(major: u16, minor: u16, patch: u16) -> Version {
        Version {
            major,
            minor,
            patch,
        }
    }

    /// A version from `major.minor.patch`, as Cargo prints it; a
    /// pre-release or build suffix after `-` or `+` is ignored.
    ///
    /// ```
    /// use teistro_core::envelope::Version;
    ///
    /// assert_eq!(Version::parse("1.2.3-rc.1+build"), Some(Version::new(1, 2, 3)));
    /// assert_eq!(Version::parse("1.2"), None);
    /// assert_eq!(Version::parse("1.2.3.4"), None);
    /// ```
    #[must_use]
    pub fn parse(text: &str) -> Option<Version> {
        let core = text.split(['-', '+']).next()?;
        let mut parts = core.split('.').map(|p| p.parse::<u16>().ok());
        let version = Version::new(parts.next()??, parts.next()??, parts.next()??);
        parts.next().is_none().then_some(version)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// A SHA-256, rendered as sixty-four hex digits.
///
/// ```
/// use teistro_core::envelope::Hash;
///
/// let hash = Hash::of(b"abc");
/// assert_eq!(
///     hash.to_string(),
///     "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
/// );
/// assert_eq!(serde_json::to_value(hash).unwrap(), hash.to_string());
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hash([u8; 32]);

impl Hash {
    /// The hash of some bytes.
    ///
    /// ```
    /// use teistro_core::envelope::Hash;
    ///
    /// assert!(Hash::of(&[]).to_string().starts_with("e3b0c442"));
    /// ```
    #[must_use]
    pub fn of(bytes: &[u8]) -> Hash {
        Hash(Sha256::digest(bytes).into())
    }

    /// The bytes.
    ///
    /// ```
    /// use teistro_core::envelope::Hash;
    ///
    /// assert_eq!(Hash::of(b"abc").bytes()[..2], [0xba, 0x78]);
    /// ```
    #[must_use]
    pub const fn bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// From sixty-four hex digits.
    ///
    /// ```
    /// use teistro_core::envelope::Hash;
    ///
    /// let hash = Hash::of(b"abc");
    /// assert_eq!(Hash::from_hex(&hash.to_string()), Some(hash));
    /// assert_eq!(Hash::from_hex(&hash.to_string().to_uppercase()), Some(hash));
    /// assert_eq!(Hash::from_hex("ba78"), None);
    /// ```
    #[must_use]
    pub fn from_hex(hex: &str) -> Option<Hash> {
        if hex.len() != 64 {
            return None;
        }
        let mut out = [0u8; 32];
        for (i, slot) in out.iter_mut().enumerate() {
            let pair = hex.get(2 * i..2 * i + 2)?;
            *slot = u8::from_str_radix(pair, 16).ok()?;
        }
        Some(Hash(out))
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash({self})")
    }
}

impl serde::Serialize for Hash {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> serde::Deserialize<'de> for Hash {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Hash, D::Error> {
        let text = String::deserialize(deserializer)?;
        Hash::from_hex(&text).ok_or_else(|| serde::de::Error::custom("a SHA-256 is 64 hex digits"))
    }
}

hand_schema!(Hash, "Hash", {
    "type": "string",
    "description": "A SHA-256, as sixty-four hex digits.",
    "pattern": "^[0-9a-fA-F]{64}$",
});

/// What the provider was.
///
/// ```
/// # use teistro_core::envelope::{Hash, Provenance, Version};
/// # let provenance = Provenance::new(
/// #     Version::new(0, 1, 0), 1, 1, "nepali-default",
/// #     Hash::of(b"settings"), Hash::of(b"input"),
/// # );
/// assert!(provenance.provider.name.is_empty());
/// assert!(provenance.provider.data_hashes.is_empty());
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ProviderStamp {
    /// The provider's name.
    pub name: String,
    /// Its version.
    pub version: String,
    /// What identifies its data.
    pub data_version: String,
    /// The hashes of the data files it read.
    pub data_hashes: Vec<(String, Hash)>,
    /// The tier, when the provider has tiers.
    pub tier: Option<String>,
    /// The frame it returned, as its key.
    pub frame: String,
    /// The flags actually used, as names.
    pub flags_used: Vec<String>,
}

/// A pack that took part.
///
/// ```
/// use teistro_core::envelope::{Hash, PackStamp};
///
/// let pack = PackStamp {
///     id: "festivals".into(),
///     version: "1.0.0".into(),
///     hash: Hash::of(b"pack"),
/// };
/// assert_eq!(serde_json::to_value(&pack).unwrap()["hash"], Hash::of(b"pack").to_string());
/// ```
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PackStamp {
    /// The pack id.
    pub id: String,
    /// Its version.
    pub version: String,
    /// Its content hash.
    pub hash: Hash,
}

/// What the time layer applied.
///
/// ```
/// use teistro_core::envelope::TimeStamp;
///
/// let time = TimeStamp::default();
/// assert_eq!(time.delta_t_seconds, 0.0);
/// assert_eq!(time.uncertainty_seconds, None);
/// ```
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TimeStamp {
    /// The Delta T model's key.
    pub delta_t_model: String,
    /// Delta T in seconds at the instant.
    pub delta_t_seconds: f64,
    /// The leap-second table version.
    pub leap_table: String,
    /// The tzdb version.
    pub tzdb_version: String,
    /// What was applied when tzdb had no rule (`LMT before 1880` and the like).
    pub time_basis_applied: Option<String>,
    /// Delta T's uncertainty in seconds for deep-time instants.
    pub uncertainty_seconds: Option<f64>,
}

/// How a calendar date was resolved (`docs/03-design/calendar-bikram-sambat.md`).
///
/// The strings are borrowed when they name a shipped table, whose
/// authority, edition and model are static text, and owned when a
/// calendar of the consumer's own names its source: a date read from the
/// Bikram Sambat table allocates nothing, and the type still carries
/// whatever another calendar wants to say.
///
/// ```
/// use teistro_core::envelope::{CalendarResolution, MonthDay};
///
/// let divergent = CalendarResolution::Divergent {
///     tabular: MonthDay { month: 1, day: 32 },
///     computed: MonthDay { month: 2, day: 1 },
///     model: "SURYA_SIDDHANTA".into(),
/// };
/// assert_eq!(serde_json::to_value(&divergent).unwrap()["kind"], "DIVERGENT");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CalendarResolution {
    /// A mathematical definition; exact by construction.
    Defined,
    /// From the authority's published table.
    Tabular {
        /// The authority.
        authority: Cow<'static, str>,
        /// The table's edition.
        edition: Cow<'static, str>,
    },
    /// Computed outside the table's range.
    Computed {
        /// The rule or model.
        model: Cow<'static, str>,
    },
    /// Inside the range and the two disagree; the table was followed.
    Divergent {
        /// The table's month and day.
        tabular: MonthDay,
        /// The computed month and day.
        computed: MonthDay,
        /// The rule or model that computed the other answer.
        model: Cow<'static, str>,
    },
}

/// A month and a day inside a year, for a divergence report.
///
/// ```
/// use teistro_core::envelope::MonthDay;
///
/// assert_eq!(MonthDay { month: 4, day: 9 }.to_string(), "04-09");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MonthDay {
    /// The month, 1-based.
    pub month: u8,
    /// The day, 1-based.
    pub day: u8,
}

impl fmt::Display for MonthDay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}-{:02}", self.month, self.day)
    }
}

/// A classical model answered rather than the modern one.
///
/// ```
/// use teistro_core::envelope::Deviation;
///
/// let deviation = Deviation {
///     model: "SURYA_SIDDHANTA".into(),
///     detail: "the Sun's longitude".into(),
/// };
/// assert_eq!(deviation.model, "SURYA_SIDDHANTA");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Deviation {
    /// The model (`SURYA_SIDDHANTA`).
    pub model: String,
    /// What it changed.
    pub detail: String,
}

/// A convention the SDK had to choose to terminate.
///
/// ```
/// # use teistro_core::envelope::{Hash, Provenance, Version};
/// # let mut provenance = Provenance::new(
/// #     Version::new(0, 1, 0), 1, 1, "nepali-default",
/// #     Hash::of(b"settings"), Hash::of(b"input"),
/// # );
/// provenance.convention("frame.ayanamsha", "LAHIRI", "the profile's default");
/// assert_eq!(provenance.applied_conventions[0].value, "LAHIRI");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Convention {
    /// The knob.
    pub knob: String,
    /// The value applied.
    pub value: String,
    /// Why.
    pub reason: String,
}

/// Whether every row that took part is verified (ADR-0018).
///
/// ```
/// use teistro_core::envelope::Confidence;
///
/// assert_eq!(Confidence::default(), Confidence::Verified);
/// assert_eq!(serde_json::to_value(Confidence::Unverified).unwrap(), "UNVERIFIED");
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Confidence {
    /// Every table row used is verified.
    #[default]
    Verified,
    /// At least one traditional row took part.
    Unverified,
}

/// A fallback that was used.
///
/// ```
/// use teistro_core::envelope::Fallback;
///
/// let fallback = Fallback {
///     what: "tzdb".into(),
///     reason: "no rule before 1920".into(),
/// };
/// assert_eq!(fallback.what, "tzdb");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Fallback {
    /// What fell back.
    pub what: String,
    /// Why.
    pub reason: String,
}

/// A warning: a code, and a message key with slots for localisation.
///
/// Its schema is named `ProvenanceWarning`, because every binding types it
/// by that name and Python's own `Warning` is a builtin a module should not
/// shadow; the wire is unchanged.
///
/// ```
/// # use teistro_core::envelope::{Hash, Provenance, Version};
/// # let mut provenance = Provenance::new(
/// #     Version::new(0, 1, 0), 1, 1, "nepali-default",
/// #     Hash::of(b"settings"), Hash::of(b"input"),
/// # );
/// provenance.warn("DEPRECATED_KEY", Some("warning.deprecated"), vec![("key".into(), "OLD".into())]);
/// assert_eq!(provenance.warnings[0].code, "DEPRECATED_KEY");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "ProvenanceWarning"))]
pub struct Warning {
    /// The stable code (`DEPRECATED_KEY`).
    pub code: String,
    /// The message key, when there is one.
    pub key: Option<String>,
    /// The slots.
    pub slots: Vec<(String, String)>,
}

/// Everything that reproduces a result.
///
/// It is written with one field more than it has: `answer_key`, the
/// [`Provenance::answer_key`] of the rest, so every binding reads the key a
/// stored answer is reused under from the JSON it decodes, and none
/// computes it again. The key is ignored where a provenance is read back,
/// and taken afresh where it is written.
///
/// ```
/// # use teistro_core::envelope::{Hash, Provenance, Version};
/// # let provenance = Provenance::new(
/// #     Version::new(0, 1, 0), 1, 1, "nepali-default",
/// #     Hash::of(b"settings"), Hash::of(b"input"),
/// # );
/// assert_eq!(provenance.profile, "nepali-default");
/// let json = serde_json::to_value(&provenance).unwrap();
/// assert_eq!(json["confidence"], "VERIFIED");
/// assert_eq!(json["answer_key"], provenance.answer_key().to_string());
/// // Read back, the key is taken from the rest again.
/// let read: Provenance = serde_json::from_value(json).unwrap();
/// assert_eq!(read, provenance);
/// ```
#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(transform = written_with_its_answer_key))]
pub struct Provenance {
    /// The SDK's version.
    pub sdk_version: Version,
    /// The versions of the modules that took part.
    pub module_versions: Vec<(String, Version)>,
    /// The calculation version (ADR-0020).
    pub calculation_version: u32,
    /// The catalogue version.
    pub catalogue_version: u32,
    /// The profile id.
    pub profile: String,
    /// The hash of the resolved settings.
    pub settings_hash: Hash,
    /// The hash of the input.
    pub input_hash: Hash,
    /// The provider.
    pub provider: ProviderStamp,
    /// The packs.
    pub packs: Vec<PackStamp>,
    /// What the time layer applied.
    pub time: TimeStamp,
    /// The calendar resolution, when a calendar took part.
    pub calendar: Option<CalendarResolution>,
    /// A classical model's deviation, when one answered.
    pub deviation: Option<Deviation>,
    /// The conventions applied.
    pub applied_conventions: Vec<Convention>,
    /// The confidence.
    pub confidence: Confidence,
    /// The fallbacks used.
    pub fallbacks_used: Vec<Fallback>,
    /// The warnings.
    pub warnings: Vec<Warning>,
    /// The hash of the canonical serialisation of the value.
    pub content_hash: Hash,
}

/// Every field as derived serde writes it, in declaration order, and the
/// answer key after them. The destructuring names every field, so a field
/// added to [`Provenance`] fails to compile here until it is written.
// lint: serialised-type-describes-itself (described by its derive, with the one field written beyond its own added by `written_with_its_answer_key`)
impl serde::Serialize for Provenance {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct as _;
        let Provenance {
            sdk_version,
            module_versions,
            calculation_version,
            catalogue_version,
            profile,
            settings_hash,
            input_hash,
            provider,
            packs,
            time,
            calendar,
            deviation,
            applied_conventions,
            confidence,
            fallbacks_used,
            warnings,
            content_hash,
        } = self;
        let mut out = serializer.serialize_struct("Provenance", 18)?;
        out.serialize_field("sdk_version", sdk_version)?;
        out.serialize_field("module_versions", module_versions)?;
        out.serialize_field("calculation_version", calculation_version)?;
        out.serialize_field("catalogue_version", catalogue_version)?;
        out.serialize_field("profile", profile)?;
        out.serialize_field("settings_hash", settings_hash)?;
        out.serialize_field("input_hash", input_hash)?;
        out.serialize_field("provider", provider)?;
        out.serialize_field("packs", packs)?;
        out.serialize_field("time", time)?;
        out.serialize_field("calendar", calendar)?;
        out.serialize_field("deviation", deviation)?;
        out.serialize_field("applied_conventions", applied_conventions)?;
        out.serialize_field("confidence", confidence)?;
        out.serialize_field("fallbacks_used", fallbacks_used)?;
        out.serialize_field("warnings", warnings)?;
        out.serialize_field("content_hash", content_hash)?;
        out.serialize_field("answer_key", &self.answer_key())?;
        out.end()
    }
}

/// The schema's `answer_key`, the field [`Provenance`] is written with
/// beyond its own: a hash, as `content_hash` is, and required, since every
/// provenance the SDK writes carries it.
#[cfg(feature = "schema")]
fn written_with_its_answer_key(schema: &mut schemars::Schema) {
    let Some(object) = schema.as_object_mut() else {
        return;
    };
    let hash = object
        .get("properties")
        .and_then(|properties| properties.get("content_hash"))
        .and_then(serde_json::Value::as_object)
        .map(|content| {
            let mut key = content.clone();
            key.insert(
                "description".to_owned(),
                serde_json::Value::from(
                    "The key a stored answer is reused under: `Provenance::answer_key`, the \
                     SHA-256 of the canonical JSON of the SDK, calculation and catalogue \
                     versions, the settings and input hashes, the provider's name, version, \
                     data version, data hashes and tier, and the packs. Written on every \
                     provenance, and ignored where one is read.",
                ),
            );
            serde_json::Value::Object(key)
        });
    if let (Some(hash), Some(serde_json::Value::Object(properties))) =
        (hash, object.get_mut("properties"))
    {
        properties.insert("answer_key".to_owned(), hash);
    }
    if let Some(serde_json::Value::Array(required)) = object.get_mut("required") {
        required.push(serde_json::Value::from("answer_key"));
    }
}

impl Provenance {
    /// A provenance with the identifying fields set and the rest empty.
    ///
    /// ```
    /// # use teistro_core::envelope::{Hash, Provenance, Version};
    /// # let provenance = Provenance::new(
    /// #     Version::new(0, 1, 0), 1, 1, "nepali-default",
    /// #     Hash::of(b"settings"), Hash::of(b"input"),
    /// # );
    /// assert_eq!(provenance.content_hash, Hash::of(&[]));
    /// assert!(provenance.packs.is_empty() && provenance.warnings.is_empty());
    /// ```
    #[must_use]
    pub fn new(
        sdk_version: Version,
        calculation_version: u32,
        catalogue_version: u32,
        profile: impl Into<String>,
        settings_hash: Hash,
        input_hash: Hash,
    ) -> Provenance {
        Provenance {
            sdk_version,
            module_versions: Vec::new(),
            calculation_version,
            catalogue_version,
            profile: profile.into(),
            settings_hash,
            input_hash,
            provider: ProviderStamp::default(),
            packs: Vec::new(),
            time: TimeStamp::default(),
            calendar: None,
            deviation: None,
            applied_conventions: Vec::new(),
            confidence: Confidence::Verified,
            fallbacks_used: Vec::new(),
            warnings: Vec::new(),
            content_hash: Hash::of(&[]),
        }
    }

    /// The cache key every binding documents.
    ///
    /// ```
    /// # use teistro_core::envelope::{Hash, Provenance, Version};
    /// # let provenance = Provenance::new(
    /// #     Version::new(0, 1, 0), 1, 1, "nepali-default",
    /// #     Hash::of(b"settings"), Hash::of(b"input"),
    /// # );
    /// let (input, settings, version) = provenance.cache_key();
    /// assert_eq!((input, settings, version), (Hash::of(b"input"), Hash::of(b"settings"), 1));
    /// ```
    #[must_use]
    pub fn cache_key(&self) -> (Hash, Hash, u32) {
        (
            self.input_hash,
            self.settings_hash,
            self.calculation_version,
        )
    }

    /// **The key a stored answer is reused under**, by any client in any
    /// language (`03-design/serial-and-the-envelope.md` §3, *The answer key*): the
    /// SHA-256 of the canonical JSON of
    /// `{sdk_version, calculation_version, catalogue_version,
    /// settings_hash, input_hash, provider: {name, version, data_version,
    /// data_hashes, tier}, packs}`, each as this provenance writes it.
    ///
    /// [`Provenance::cache_key`] names the request and the settings; this
    /// names everything else an answer is a function of, the engine and
    /// its data above all, so that an app keeping a year of festivals
    /// recomputes it when the engine, its data, a pack or the SDK changes
    /// and at no other time. The frame and the flags are left out: they
    /// follow from the settings and the provider.
    ///
    /// ```
    /// # use teistro_core::envelope::{Hash, Provenance, Version};
    /// # let provenance = Provenance::new(
    /// #     Version::new(0, 1, 0), 1, 1, "nepali-default",
    /// #     Hash::of(b"settings"), Hash::of(b"input"),
    /// # );
    /// let mut newer_data = provenance.clone();
    /// newer_data.provider.data_version = "2".into();
    /// assert_ne!(provenance.answer_key(), newer_data.answer_key());
    ///
    /// // The flags are not part of it: they follow from the settings.
    /// let mut flagged = provenance.clone();
    /// flagged.provider.flags_used.push("SPEED".into());
    /// assert_eq!(provenance.answer_key(), flagged.answer_key());
    /// ```
    #[must_use]
    pub fn answer_key(&self) -> Hash {
        let ProviderStamp {
            name,
            version,
            data_version,
            data_hashes,
            tier,
            ..
        } = &self.provider;
        content_hash(&serde_json::json!({
            "sdk_version": self.sdk_version,
            "calculation_version": self.calculation_version,
            "catalogue_version": self.catalogue_version,
            "settings_hash": self.settings_hash,
            "input_hash": self.input_hash,
            "provider": {
                "name": name,
                "version": version,
                "data_version": data_version,
                "data_hashes": data_hashes,
                "tier": tier,
            },
            "packs": self.packs,
        }))
    }

    /// Records a convention.
    ///
    /// ```
    /// # use teistro_core::envelope::{Hash, Provenance, Version};
    /// # let mut provenance = Provenance::new(
    /// #     Version::new(0, 1, 0), 1, 1, "nepali-default",
    /// #     Hash::of(b"settings"), Hash::of(b"input"),
    /// # );
    /// provenance.convention("time.delta_t", "ESPENAK_MEEUS", "outside the table");
    /// assert_eq!(provenance.applied_conventions.len(), 1);
    /// ```
    pub fn convention(&mut self, knob: &str, value: &str, reason: &str) {
        self.applied_conventions.push(Convention {
            knob: knob.to_string(),
            value: value.to_string(),
            reason: reason.to_string(),
        });
    }

    /// Records a warning.
    ///
    /// ```
    /// # use teistro_core::envelope::{Hash, Provenance, Version};
    /// # let mut provenance = Provenance::new(
    /// #     Version::new(0, 1, 0), 1, 1, "nepali-default",
    /// #     Hash::of(b"settings"), Hash::of(b"input"),
    /// # );
    /// provenance.warn("DEEP_TIME", None, Vec::new());
    /// assert_eq!(provenance.warnings[0].key, None);
    /// ```
    pub fn warn(&mut self, code: &str, key: Option<&str>, slots: Vec<(String, String)>) {
        self.warnings.push(Warning {
            code: code.to_string(),
            key: key.map(str::to_string),
            slots,
        });
    }
}

/// A value with its provenance.
///
/// ```
/// # use teistro_core::envelope::{Hash, Provenance, Version};
/// # let provenance = Provenance::new(
/// #     Version::new(0, 1, 0), 1, 1, "nepali-default",
/// #     Hash::of(b"settings"), Hash::of(b"input"),
/// # );
/// use teistro_core::envelope::Envelope;
///
/// let envelope = Envelope::sealing(42_u32, provenance);
/// assert_eq!(envelope.value, 42);
/// ```
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "{T}Envelope"))]
pub struct Envelope<T> {
    /// The value.
    pub value: T,
    /// How it was produced.
    pub provenance: Provenance,
}

impl<T> Envelope<T> {
    /// Wraps a value.
    ///
    /// ```
    /// # use teistro_core::envelope::{Hash, Provenance, Version};
    /// # let provenance = Provenance::new(
    /// #     Version::new(0, 1, 0), 1, 1, "nepali-default",
    /// #     Hash::of(b"settings"), Hash::of(b"input"),
    /// # );
    /// use teistro_core::envelope::Envelope;
    ///
    /// let envelope = Envelope::new("value", provenance);
    /// assert_eq!(envelope.provenance.content_hash, Hash::of(&[]));
    /// ```
    #[must_use]
    pub const fn new(value: T, provenance: Provenance) -> Envelope<T> {
        Envelope { value, provenance }
    }

    /// Wraps a value and **fills the provenance's content hash with the
    /// hash of that value**, which is what the field is for.
    ///
    /// `Provenance::new` leaves it `Hash::of(&[])`, and
    /// [`serial-and-the-envelope.md`](../../../docs/03-design/serial-and-the-envelope.md)
    /// §2 found why that is a shape problem rather than a bug in a
    /// producer: a value and its stamp are built separately and joined
    /// at the end, so the one field that cannot be filled until the
    /// value exists is the one everybody forgets. This is the join, and
    /// the four callers that each wrote the same mending line — the C
    /// boundary's chart and panchanga entry points, and the Rust
    /// façade's two areas — call it instead.
    ///
    /// **It belongs where a value is published, not where it is
    /// produced**, and that was measured rather than argued. Sealing
    /// inside `Founder::found` and `Almanac::between` made every caller
    /// pay a full canonical serialisation of the value for a field many
    /// of them discard: the instruction-count gate put `panchanga`
    /// **8.8% over** its base for ten days of it. A producer's caller may
    /// want the numbers and nothing else; a consumer handed an envelope
    /// must have a true hash. So the boundary seals, the façade's areas
    /// seal, and a crate calling a producer directly pays for what it
    /// uses.
    ///
    /// ```
    /// # use teistro_core::envelope::{Hash, Provenance, Version};
    /// # let provenance = Provenance::new(
    /// #     Version::new(0, 1, 0), 1, 1, "nepali-default",
    /// #     Hash::of(b"settings"), Hash::of(b"input"),
    /// # );
    /// use teistro_core::envelope::{Envelope, content_hash};
    ///
    /// let envelope = Envelope::sealing([1, 2, 3], provenance);
    /// assert_eq!(envelope.provenance.content_hash, content_hash(&[1, 2, 3]));
    /// ```
    #[must_use]
    pub fn sealing(value: T, provenance: Provenance) -> Envelope<T>
    where
        T: serde::Serialize,
    {
        let content_hash = content_hash(&value);
        Envelope {
            value,
            provenance: Provenance {
                content_hash,
                ..provenance
            },
        }
    }

    /// A batch sealed with its own hash, and every member's hash beside
    /// it, from one serialisation ([`content_hashes`]): what a caller that
    /// hands out members one at a time stamps each with.
    ///
    /// ```
    /// # use teistro_core::envelope::{Hash, Provenance, Version};
    /// # let provenance = Provenance::new(
    /// #     Version::new(0, 1, 0), 1, 1, "nepali-default",
    /// #     Hash::of(b"settings"), Hash::of(b"input"),
    /// # );
    /// use teistro_core::envelope::{Envelope, content_hash};
    ///
    /// let (batch, each) = Envelope::sealing_each(vec!["a", "b"], provenance);
    /// assert_eq!(batch.provenance.content_hash, content_hash(&["a", "b"]));
    /// assert_eq!(each[1], content_hash("b"));
    /// ```
    #[must_use]
    pub fn sealing_each<I>(value: T, provenance: Provenance) -> (Envelope<T>, Vec<Hash>)
    where
        T: AsRef<[I]>,
        I: serde::Serialize,
    {
        let (content_hash, each) = content_hashes(value.as_ref());
        let sealed = Envelope {
            value,
            provenance: Provenance {
                content_hash,
                ..provenance
            },
        };
        (sealed, each)
    }

    /// Maps the value, keeping the provenance.
    ///
    /// ```
    /// # use teistro_core::envelope::{Hash, Provenance, Version};
    /// # let provenance = Provenance::new(
    /// #     Version::new(0, 1, 0), 1, 1, "nepali-default",
    /// #     Hash::of(b"settings"), Hash::of(b"input"),
    /// # );
    /// use teistro_core::envelope::Envelope;
    ///
    /// let envelope = Envelope::sealing(21_u32, provenance).map(|value| value * 2);
    /// assert_eq!(envelope.value, 42);
    /// ```
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Envelope<U> {
        Envelope {
            value: f(self.value),
            provenance: self.provenance,
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;

    /// The list's hash and each item's, from one serialisation, are the
    /// hashes the items would have alone — for an empty list, one item,
    /// and records whose keys and numbers the canonical form reorders and
    /// respells.
    #[test]
    fn a_batch_hashes_as_its_list_and_each_member_as_itself() {
        let empty: [serde_json::Value; 0] = [];
        let (list, each) = content_hashes(&empty);
        assert_eq!(list, content_hash(&empty[..]));
        assert!(each.is_empty(), "{each:?}");

        let records = [
            serde_json::json!({"b": 1e-7, "a": [1.0, -0.0], "c": "é\n"}),
            serde_json::json!(null),
            serde_json::json!({"z": {"y": 2, "x": true}}),
        ];
        for count in 1..=records.len() {
            let some = &records[..count];
            let (list, each) = content_hashes(some);
            assert_eq!(list, content_hash(some), "{count} records");
            for (item, hash) in some.iter().zip(&each) {
                assert_eq!(*hash, content_hash(item));
            }
        }
    }

    /// A client reproduces the answer key from the provenance as written,
    /// and the key moves with the engine and its data, and not with what
    /// an answer carries beside its inputs.
    #[test]
    fn the_answer_key_is_the_written_provenance_s_and_names_the_engine() {
        let mut provenance = Provenance::new(
            Version::new(0, 1, 0),
            3,
            2,
            "nepali-committee",
            Hash::of(b"s"),
            Hash::of(b"i"),
        );
        provenance.provider = ProviderStamp {
            name: "teimeris".to_owned(),
            version: "0.1.0".to_owned(),
            data_version: "sepl_18,semo_18".to_owned(),
            data_hashes: vec![("sepl_18.se1".to_owned(), Hash::of(b"d"))],
            tier: None,
            frame: "geocentric".to_owned(),
            flags_used: vec!["SPEED".to_owned()],
        };
        let written = serde_json::to_value(&provenance).unwrap();
        let pick = serde_json::json!({
            "sdk_version": written["sdk_version"],
            "calculation_version": written["calculation_version"],
            "catalogue_version": written["catalogue_version"],
            "settings_hash": written["settings_hash"],
            "input_hash": written["input_hash"],
            "provider": {
                "name": written["provider"]["name"],
                "version": written["provider"]["version"],
                "data_version": written["provider"]["data_version"],
                "data_hashes": written["provider"]["data_hashes"],
                "tier": written["provider"]["tier"],
            },
            "packs": written["packs"],
        });
        let key = provenance.answer_key();
        assert_eq!(key, content_hash(&pick));

        let mut beside = provenance.clone();
        beside.provider.frame = "topocentric".to_owned();
        beside.warnings.push(Warning {
            code: "DEPRECATED_KEY".to_owned(),
            key: None,
            slots: Vec::new(),
        });
        beside.content_hash = Hash::of(b"another answer");
        assert_eq!(beside.answer_key(), key);

        let mut engine = provenance.clone();
        engine.provider.version = "0.2.0".to_owned();
        assert_ne!(engine.answer_key(), key);
        let mut data = provenance;
        data.provider.data_hashes[0].1 = Hash::of(b"another file");
        assert_ne!(data.answer_key(), key);
    }

    #[test]
    fn hashes_versions_and_the_envelope_round_trip() {
        let hash = Hash::of(b"teistro");
        assert_eq!(hash.to_string().len(), 64);
        assert_eq!(Hash::from_hex(&hash.to_string()), Some(hash));
        assert_eq!(Hash::from_hex("xyz"), None);
        assert_eq!(Version::new(1, 2, 3).to_string(), "1.2.3");
        assert_eq!(Version::parse("1.2.3"), Some(Version::new(1, 2, 3)));
        assert_eq!(
            Version::parse("0.4.0-alpha.1+build"),
            Some(Version::new(0, 4, 0))
        );
        assert_eq!(Version::parse("1.2"), None);
        assert_eq!(Version::parse("1.2.3.4"), None);
        let mut provenance =
            Provenance::new(Version::new(0, 1, 0), 1, 1, "nepali-default", hash, hash);
        provenance.convention(
            "polar_policy",
            "FALLBACK_WHOLE_SIGN",
            "Placidus undefined at 70N",
        );
        provenance.warn(
            "DEPRECATED_KEY",
            Some("sdk.warning.deprecatedKey"),
            vec![("key".into(), "VISHKAMBA".into())],
        );
        provenance.calendar = Some(CalendarResolution::Divergent {
            tabular: MonthDay { month: 2, day: 32 },
            computed: MonthDay { month: 3, day: 1 },
            model: Cow::Borrowed("a model"),
        });
        let envelope = Envelope::new(42u32, provenance.clone());
        let json = serde_json::to_string(&envelope).unwrap_or_default();
        let back: Envelope<u32> = serde_json::from_str(&json).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(back, envelope);
        assert_eq!(back.provenance.cache_key(), (hash, hash, 1));
        assert_eq!(envelope.map(|v| v * 2).value, 84);
        assert!(json.contains("\"kind\":\"DIVERGENT\""));
    }

    #[test]
    fn canonical_json_escapes_what_json_requires_and_nothing_else() {
        let text = "plain नेपाल";
        assert_eq!(canonical_json(text), "\"plain नेपाल\"");
        let escaped = "a\"b\\c\nd\re\tf\u{1}g/h";
        assert_eq!(
            canonical_json(escaped),
            "\"a\\\"b\\\\c\\nd\\re\\tf\\u0001g/h\""
        );
        assert_eq!(
            canonical_json(&(-3_i64, u64::MAX, 0.5_f64, -0.0_f64)),
            "[-3,18446744073709551615,0.5,0]"
        );
    }

    #[test]
    fn canonical_json_sorts_every_object_whatever_the_map_order() {
        #[derive(serde::Serialize)]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        struct Unsorted {
            zeta: u8,
            alpha: Inner,
            mid: Vec<Inner>,
        }
        #[derive(serde::Serialize)]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        struct Inner {
            second: u8,
            first: u8,
        }
        let value = Unsorted {
            zeta: 1,
            alpha: Inner {
                second: 2,
                first: 3,
            },
            mid: vec![Inner {
                second: 4,
                first: 5,
            }],
        };
        assert_eq!(
            canonical_json(&value),
            r#"{"alpha":{"first":3,"second":2},"mid":[{"first":5,"second":4}],"zeta":1}"#
        );
        assert_eq!(
            content_hash(&value),
            Hash::of(canonical_json(&value).as_bytes())
        );
    }

    /// Sealing stamps the value's own hash over whatever the provenance
    /// carried, and a batch stamps each member's beside the list's.
    #[test]
    fn a_seal_stamps_the_value_s_hash() {
        let stale = Hash::of(b"stale");
        let provenance = Provenance::new(Version::new(0, 1, 0), 1, 1, "p", stale, stale);
        let sealed = Envelope::sealing([3_u8, 1], provenance.clone());
        assert_eq!(sealed.provenance.content_hash, content_hash(&[3_u8, 1]));
        assert_eq!(sealed.provenance.settings_hash, stale);
        let (batch, each) = Envelope::sealing_each(vec![1_u8, 2], provenance);
        assert_eq!(batch.provenance.content_hash, content_hash(&[1_u8, 2]));
        assert_eq!(each, [content_hash(&1_u8), content_hash(&2_u8)]);
    }

    #[test]
    fn small_values_write_as_their_documents_read_them() {
        assert_eq!(canonical_json_at(&[1.23456_f64, 2.0], 2), "[1.23,2]");
        assert_eq!(decimal(f64::NAN, Digits::Shortest), "0");
        assert_eq!(decimal(f64::NEG_INFINITY, Digits::Rounded(3)), "0");
        let hash = Hash::of(b"");
        assert_eq!(hash.bytes().len(), 32);
        assert_eq!(format!("{hash:?}"), format!("Hash({hash})"));
        assert_eq!(MonthDay { month: 3, day: 7 }.to_string(), "03-07");
    }
}
