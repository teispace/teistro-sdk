//! The `sdk-only` byte-identity check: under the `sdk-only` override
//! policy a chart is a function of the provider's **native positions** and
//! of nothing else the provider offers (ADR-0009, ADR-0013).
//!
//! That is the promise that makes cross-provider identity possible: two
//! providers that agree on their native positions give the same bytes,
//! whatever else each declares. So the check founds every reading the
//! façade can make twice under `sdk-only` — once over the provider, once
//! over [`NativeFrameOnly`], the same provider reduced to its native frame
//! with every override and every other frame taken away — and requires
//! the two sealed documents to be identical, byte for byte, provenance
//! included. A difference is reported by the first field it reaches, as
//! ADR-0022 asks of every divergence.
//!
//! A check that cannot fail proves nothing, so the same readings are made
//! under `prefer-native` as well and the report says how many of them the
//! provider's own overrides changed: for a provider that declares none,
//! the identity holds trivially and the report says so.
//!
//! ```
//! use teistro_ephemeris_kit::sdk_only;
//! use teistro_port_ephemeris::{EphemerisProvider, TestProvider};
//!
//! let open = || -> Box<dyn EphemerisProvider> { Box::new(TestProvider::new()) };
//! let check = sdk_only::check(&open);
//! assert!(check.passed, "{}", check.detail);
//! ```

use serde_json::Value;
use teistro::{ChartRequest, Context, Ephemeris, UtcOffset};
use teistro_core::quantity::{JulianDay, Place, Utc};
use teistro_core::settings::OverridePolicy;
use teistro_port_ephemeris::{
    Capabilities, EphemerisProvider, Frame, Overrides, PositionColumns, PositionRequest,
    ProviderError,
};

use crate::kit::{self, Check};

/// The check's name in a report.
///
/// ```
/// # use teistro_port_ephemeris::{EphemerisProvider, TestProvider};
/// #
/// # let open = || -> Box<dyn EphemerisProvider> { Box::new(TestProvider::new()) };
///
/// use teistro_ephemeris_kit::sdk_only::{NAME, check};
///
/// assert_eq!(check(&open).name, NAME);
/// ```
pub const NAME: &str = "sdk-only: a chart is its native positions";

/// How the check opens a provider: the façade owns what it computes with,
/// so it is given a fresh one for each context it builds.
///
/// ```
/// use teistro_ephemeris_kit::sdk_only::Open;
/// use teistro_port_ephemeris::{EphemerisProvider, TestProvider};
///
/// // A fresh provider each call: a context owns the one it is given.
/// let fresh = || -> Box<dyn EphemerisProvider> { Box::new(TestProvider::new()) };
/// let open: Open<'_> = &fresh;
/// assert_eq!(open().capabilities(), open().capabilities());
/// ```
pub type Open<'a> = &'a dyn Fn() -> Box<dyn EphemerisProvider>;

/// A provider reduced to its native frame: the positions it computes in the
/// frame it declares, and nothing else — no override, no ayanamsha, no
/// other frame, no engine operations.
///
/// This is what `sdk-only` promises a chart depends on, so a chart founded
/// over it under that policy is the reference the provider's own chart is
/// held to.
///
/// ```
/// use teistro_ephemeris_kit::sdk_only::NativeFrameOnly;
/// use teistro_port_ephemeris::{EphemerisProvider, Overrides, TestProvider};
///
/// let reduced = NativeFrameOnly::new(TestProvider::new());
/// assert_eq!(reduced.capabilities().overrides, Overrides::NONE);
/// ```
pub struct NativeFrameOnly<P> {
    inner: P,
    native: Frame,
}

impl<P> core::fmt::Debug for NativeFrameOnly<P> {
    /// The frame it answers in, which is all it adds to the provider.
    fn fmt(&self, out: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        out.debug_struct("NativeFrameOnly")
            .field("native", &self.native)
            .finish_non_exhaustive()
    }
}

impl<P: EphemerisProvider> NativeFrameOnly<P> {
    /// The provider, reduced.
    ///
    /// ```
    /// use teistro_ephemeris_kit::sdk_only::NativeFrameOnly;
    /// use teistro_port_ephemeris::{EphemerisProvider, TestProvider};
    ///
    /// // The native frame kept; the ayanamshas and engine operations gone.
    /// let provider = TestProvider::new();
    /// let reduced = NativeFrameOnly::new(TestProvider::new());
    /// assert_eq!(reduced.capabilities().native_frame, provider.capabilities().native_frame);
    /// assert!(reduced.capabilities().ayanamshas.is_empty());
    /// ```
    pub fn new(inner: P) -> NativeFrameOnly<P> {
        let native = inner.capabilities().native_frame;
        NativeFrameOnly { inner, native }
    }
}

impl<P: EphemerisProvider> EphemerisProvider for NativeFrameOnly<P> {
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            overrides: Overrides::NONE,
            ayanamshas: Vec::new(),
            native: false,
            ..self.inner.capabilities()
        }
    }

    fn positions(&self, request: &PositionRequest<'_>) -> Result<PositionColumns, ProviderError> {
        if request.frame != self.native {
            return Err(ProviderError::unsupported(
                "a frame other than the native one",
            ));
        }
        self.inner.positions(request)
    }
}

/// Where the readings are made: the middle of the kit's instants across
/// the provider's coverage, at the kit's three sea-level places — the
/// tropics, near the polar circle and the far south, which between them
/// reach every override a reading can take. A reading is every section the
/// façade makes, so three of them cost what the fifteen instants the kit's
/// own checks take would cost five times over, and prove nothing more.
fn births(capabilities: &Capabilities) -> Vec<(JulianDay<Utc>, Place)> {
    let instants = kit::instants(capabilities);
    let middle = instants
        .get(instants.len() / 2)
        .copied()
        .unwrap_or(2_451_545.0);
    kit::places()
        .into_iter()
        .map(|(_, place)| (JulianDay::<Utc>::literal(middle), place))
        .collect()
}

/// Every reading over one provider under one policy, each as the JSON of
/// its sealed envelope, or the refusal it gave: a refusal both sides give
/// alike is identity too.
fn readings(
    provider: Box<dyn EphemerisProvider>,
    policy: OverridePolicy,
    births: &[(JulianDay<Utc>, Place)],
) -> Result<Vec<Result<Value, String>>, String> {
    let patch = serde_json::json!({ "provider": { "overrides": policy } });
    let sdk = Context::builder()
        .settings_json(patch.to_string())
        .ephemeris([Ephemeris::Provider(provider)])
        .build()
        .map_err(|why| format!("the context: {why}"))?;
    Ok(births
        .iter()
        .map(|(instant, place)| {
            let request = ChartRequest::at(*place, UtcOffset::UTC).with_everything();
            sdk.chart()
                .reading(*instant, &request)
                .map_err(|why| why.to_string())
                .and_then(|sealed| serde_json::to_value(sealed).map_err(|why| why.to_string()))
        })
        .collect())
}

/// The first place two JSON values part, as a path from the root, or
/// `None` when they are the same value.
///
/// ```
/// use serde_json::json;
/// use teistro_ephemeris_kit::sdk_only::first_difference;
///
/// let left = json!({"positions": [{"deg": 10.0}, {"deg": 20.0}]});
/// let right = json!({"positions": [{"deg": 10.0}, {"deg": 20.5}]});
/// assert_eq!(first_difference(&left, &right, "").as_deref(), Some(".positions[1].deg"));
/// assert_eq!(first_difference(&left, &left, ""), None);
/// ```
#[must_use]
pub fn first_difference(left: &Value, right: &Value, path: &str) -> Option<String> {
    match (left, right) {
        (Value::Object(left), Value::Object(right)) => left
            .iter()
            .find_map(|(key, value)| match right.get(key) {
                None => Some(format!("{path}.{key}")),
                Some(other) => first_difference(value, other, &format!("{path}.{key}")),
            })
            .or_else(|| {
                right
                    .keys()
                    .find(|key| !left.contains_key(*key))
                    .map(|key| format!("{path}.{key}"))
            }),
        (Value::Array(left), Value::Array(right)) if left.len() == right.len() => left
            .iter()
            .zip(right)
            .enumerate()
            .find_map(|(at, (value, other))| {
                first_difference(value, other, &format!("{path}[{at}]"))
            }),
        _ if left == right => None,
        _ => Some(path.to_owned()),
    }
}

/// The first reading two runs part at, named by its place in the list and
/// its field.
fn parted(left: &[Result<Value, String>], right: &[Result<Value, String>]) -> Option<String> {
    left.iter()
        .zip(right)
        .enumerate()
        .find_map(|(at, pair)| match pair {
            (Ok(left), Ok(right)) => first_difference(left, right, &format!("readings[{at}]")),
            (Err(left), Err(right)) if left == right => None,
            (left, right) => Some(format!(
                "readings[{at}]: {} against {}",
                describe(left),
                describe(right)
            )),
        })
}

fn describe(reading: &Result<Value, String>) -> String {
    match reading {
        Ok(_) => String::from("a document"),
        Err(why) => format!("the refusal `{why}`"),
    }
}

/// What comparing a provider with its native frame alone found.
///
/// ```
/// # use teistro_port_ephemeris::{EphemerisProvider, TestProvider};
/// #
/// # let open = || -> Box<dyn EphemerisProvider> { Box::new(TestProvider::new()) };
///
/// use teistro_core::settings::OverridePolicy;
/// use teistro_ephemeris_kit::sdk_only::parting;
///
/// // Three births, at the kit's three places.
/// let found = parting(&open, OverridePolicy::SdkOnly)?;
/// assert_eq!(found.readings, 3);
/// assert!(found.founded <= found.readings);
/// # Ok::<(), String>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parting {
    /// How many readings were made on each side.
    pub readings: usize,
    /// How many of them founded a document rather than refusing.
    pub founded: usize,
    /// The first field the two sides part at, or `None` when every reading
    /// is identical.
    pub at: Option<String>,
}

/// Makes every reading under a policy over the provider and over
/// [`NativeFrameOnly`] of it, and names the first field they part at.
///
/// Under `sdk-only` they must never part; under `prefer-native` they part
/// wherever one of the provider's overrides reaches a chart, which is what
/// makes a pass under `sdk-only` mean something.
///
/// ```
/// # use teistro_port_ephemeris::{EphemerisProvider, TestProvider};
/// #
/// # let open = || -> Box<dyn EphemerisProvider> { Box::new(TestProvider::new()) };
///
/// use teistro_core::settings::OverridePolicy;
/// use teistro_ephemeris_kit::sdk_only::parting;
///
/// // Under `sdk-only` the two sides never part.
/// assert_eq!(parting(&open, OverridePolicy::SdkOnly)?.at, None);
/// # Ok::<(), String>(())
/// ```
///
/// # Errors
///
/// A context the SDK cannot build over the provider.
pub fn parting(open: Open<'_>, policy: OverridePolicy) -> Result<Parting, String> {
    let births = births(&open().capabilities());
    let own = readings(open(), policy, &births)?;
    let reduced = readings(Box::new(NativeFrameOnly::new(open())), policy, &births)?;
    Ok(Parting {
        readings: own.len(),
        founded: own.iter().filter(|reading| reading.is_ok()).count(),
        at: parted(&own, &reduced),
    })
}

/// Runs the check.
///
/// It passes when every reading under `sdk-only` is identical over the
/// provider and over its native frame alone. Its detail says where the two
/// part under `prefer-native`, so a reader can tell a check that held
/// against real overrides from one that held because none reaches a chart.
///
/// ```
/// # use teistro_port_ephemeris::{EphemerisProvider, TestProvider};
/// #
/// # let open = || -> Box<dyn EphemerisProvider> { Box::new(TestProvider::new()) };
///
/// use teistro_ephemeris_kit::sdk_only::check;
///
/// let check = check(&open);
/// assert!(check.passed, "{}", check.detail);
/// assert!(check.detail.starts_with("3 readings"));
/// ```
#[must_use]
pub fn check(open: Open<'_>) -> Check {
    let (sdk_only, native) = match (
        parting(open, OverridePolicy::SdkOnly),
        parting(open, OverridePolicy::PreferNative),
    ) {
        (Ok(sdk_only), Ok(native)) => (sdk_only, native),
        (Err(why), _) | (_, Err(why)) => return Check::fail(NAME, why),
    };
    if let Some(at) = sdk_only.at {
        return Check::fail(
            NAME,
            format!("the provider's chart is not its native positions' chart: they part at `{at}`"),
        );
    }
    let overrides = match native.at {
        Some(at) => format!("under `prefer-native` its overrides make them part, first at `{at}`"),
        None => String::from(
            "no override of the provider's reaches a chart even under `prefer-native`, so the identity holds trivially",
        ),
    };
    Check::pass(
        NAME,
        format!(
            "{} readings, {} founded, byte-identical to the native frame's; {overrides}",
            sdk_only.readings, sdk_only.founded
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::first_difference;
    use serde_json::json;

    #[test]
    fn the_first_difference_is_named_by_its_path() {
        let left = json!({"a": {"b": [1, 2, 3]}, "c": true});
        assert_eq!(first_difference(&left, &left, ""), None);
        let moved = json!({"a": {"b": [1, 5, 3]}, "c": true});
        assert_eq!(
            first_difference(&left, &moved, "").as_deref(),
            Some(".a.b[1]")
        );
        let shorter = json!({"a": {"b": [1, 2]}, "c": true});
        assert_eq!(
            first_difference(&left, &shorter, "").as_deref(),
            Some(".a.b")
        );
        let more = json!({"a": {"b": [1, 2, 3]}, "c": true, "d": 0});
        assert_eq!(first_difference(&left, &more, "").as_deref(), Some(".d"));
    }
}
