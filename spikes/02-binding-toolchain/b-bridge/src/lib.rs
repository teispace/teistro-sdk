//! Spike 2, option B: the Diplomat bridge over the slice.
//!
//! The same slice as option A, exposed the way Diplomat wants it: enums
//! and plain structs copied across, opaque types behind pointers with
//! accessor methods, `Result` for errors. The tree-shaped result has no
//! Diplomat spelling (no recursive structs, no output slices in the
//! JavaScript and Dart backends), so `Chart` is an opaque with row
//! accessors and a bulk `dasha_rows_into` for the backends that take
//! mutable primitive slices. The host provider is a trait, which the
//! JavaScript and Dart backends of Diplomat 0.16 do not support: the method
//! that takes it is disabled for them, and the result page records the
//! tool's refusal when it is not.

// The bridge macro expands to the extern "C" surface with its own unsafe
// code and its own naming; the module is the contract, the expansion is
// Diplomat's.
#[allow(
    unsafe_code,
    unreachable_pub,
    missing_docs,
    missing_debug_implementations,
    clippy::all,
    clippy::pedantic,
    reason = "Diplomat's macro owns the expansion of this module"
)]
#[diplomat::bridge]
mod ffi {
    use teistro_spike_slice as slice;

    /// The ayanamsha catalogue.
    #[diplomat::enum_convert(slice::Ayanamsha)]
    pub enum Ayanamsha {
        Lahiri,
        Raman,
        Krishnamurti,
    }

    /// Which lunar node the provider is asked for.
    #[diplomat::enum_convert(slice::NodeKind)]
    pub enum NodeKind {
        Mean,
        /// Renamed for Dart, where `true` is a keyword the backend does not escape.
        #[diplomat::attr(dart, rename = "trueNode")]
        True,
    }

    /// The nine bodies.
    #[diplomat::enum_convert(slice::Body)]
    pub enum Body {
        Sun,
        Moon,
        Mars,
        Mercury,
        Jupiter,
        Venus,
        Saturn,
        Rahu,
        Ketu,
    }

    /// Why a call failed; the codes match the slice's.
    pub enum ErrorCode {
        DepthOutOfRange = 1,
        JulianDayNotFinite = 2,
        Provider = 3,
        PositionNotFinite = 4,
    }

    /// The settings of a context.
    pub struct Settings {
        /// The ayanamsha.
        pub ayanamsha: Ayanamsha,
        /// The lunar node.
        pub node: NodeKind,
        /// Dasha levels to build, 1 to 5.
        pub dasha_depth: u8,
    }

    /// A tropical position, the provider's answer.
    pub struct Position {
        /// Tropical ecliptic longitude in degrees.
        pub longitude_deg: f64,
        /// Ecliptic latitude in degrees.
        pub latitude_deg: f64,
        /// Longitude speed in degrees per day.
        pub speed_deg_per_day: f64,
    }

    /// A classified sidereal position.
    pub struct BodyPosition {
        pub body: Body,
        pub longitude_deg: f64,
        pub longitude_nas: i64,
        pub latitude_deg: f64,
        pub speed_deg_per_day: f64,
        pub sign: u8,
        pub nakshatra: u8,
        pub pada: u8,
        pub retrograde: bool,
    }

    /// One row of the tree in pre-order.
    pub struct DashaRow {
        pub level: u8,
        pub lord: Body,
        pub parent: i32,
        pub start_jd: f64,
        pub end_jd: f64,
    }

    /// The ephemeris port as a Diplomat trait: what a host provider
    /// implements where the backend supports traits.
    pub trait EphemerisPort {
        fn position(&self, jd_ut: f64, body: Body) -> Position;
    }

    /// A context: settings plus a provider.
    #[diplomat::opaque]
    pub struct Context(slice::Context);

    impl Context {
        /// Creates a context over the built-in analytic test provider.
        #[diplomat::attr(auto, constructor)]
        pub fn create(settings: Settings) -> Result<Box<Context>, ErrorCode> {
            let inner = slice::Context::new(
                super::to_slice_settings(&settings),
                Box::new(slice::TestProvider),
            )?;
            Ok(Box::new(Context(inner)))
        }

        /// Creates a context over a host provider, where the backend can
        /// pass one (C, C++, Kotlin; not JavaScript or Dart in Diplomat 0.16).
        #[diplomat::attr(any(js, dart), disable)]
        pub fn create_with_provider(
            settings: Settings,
            provider: impl EphemerisPort + 'static,
        ) -> Result<Box<Context>, ErrorCode> {
            let inner = slice::Context::new(
                super::to_slice_settings(&settings),
                Box::new(super::HostPort(provider)),
            )?;
            Ok(Box::new(Context(inner)))
        }

        /// The settings the context was built with.
        pub fn settings(&self) -> Settings {
            let s = self.0.settings();
            Settings {
                ayanamsha: s.ayanamsha.into(),
                node: s.node.into(),
                dasha_depth: s.dasha_depth,
            }
        }

        /// The one batch call.
        pub fn compute_chart(&self, jd_ut: f64) -> Result<Box<Chart>, ErrorCode> {
            let chart = self.0.compute_chart(jd_ut)?;
            let rows = chart.dasha_rows();
            Ok(Box::new(Chart { chart, rows }))
        }
    }

    /// A computed chart, read through accessors.
    #[diplomat::opaque]
    pub struct Chart {
        chart: slice::Chart,
        rows: Vec<slice::DashaRow>,
    }

    impl Chart {
        /// The instant the chart was computed for.
        #[diplomat::attr(auto, getter)]
        pub fn jd_ut(&self) -> f64 {
            self.chart.jd_ut
        }

        /// The ayanamsha value applied, in degrees.
        #[diplomat::attr(auto, getter)]
        pub fn ayanamsha_deg(&self) -> f64 {
            self.chart.ayanamsha_deg
        }

        /// The number of positions, nine.
        #[diplomat::attr(auto, getter)]
        pub fn position_count(&self) -> u32 {
            u32::try_from(self.chart.positions.len()).unwrap_or(u32::MAX)
        }

        /// One position by index.
        pub fn position(&self, index: u32) -> Option<BodyPosition> {
            let p = self.chart.positions.get(usize::try_from(index).ok()?)?;
            Some(BodyPosition {
                body: p.body.into(),
                longitude_deg: p.longitude_deg,
                longitude_nas: p.longitude_nas,
                latitude_deg: p.latitude_deg,
                speed_deg_per_day: p.speed_deg_per_day,
                sign: p.sign,
                nakshatra: p.nakshatra,
                pada: p.pada,
                retrograde: p.retrograde,
            })
        }

        /// The number of tree rows.
        #[diplomat::attr(auto, getter)]
        pub fn dasha_row_count(&self) -> u32 {
            u32::try_from(self.rows.len()).unwrap_or(u32::MAX)
        }

        /// One row of the tree by index, in pre-order.
        pub fn dasha_row(&self, index: u32) -> Option<DashaRow> {
            let r = self.rows.get(usize::try_from(index).ok()?)?;
            Some(DashaRow {
                level: r.level,
                lord: r.lord.into(),
                parent: r.parent,
                start_jd: r.start_jd,
                end_jd: r.end_jd,
            })
        }

        /// Fills caller-owned columns with every row, for backends that
        /// pass mutable primitive slices; returns the rows written.
        #[diplomat::attr(any(js, dart), disable)]
        pub fn dasha_rows_into(
            &self,
            level: &mut [u8],
            lord: &mut [u8],
            parent: &mut [i32],
            start_jd: &mut [f64],
            end_jd: &mut [f64],
        ) -> u32 {
            let n = [
                level.len(),
                lord.len(),
                parent.len(),
                start_jd.len(),
                end_jd.len(),
            ]
            .into_iter()
            .chain(std::iter::once(self.rows.len()))
            .min()
            .unwrap_or(0);
            for (i, r) in self.rows.iter().take(n).enumerate() {
                if let (Some(a), Some(b), Some(c), Some(d), Some(e)) = (
                    level.get_mut(i),
                    lord.get_mut(i),
                    parent.get_mut(i),
                    start_jd.get_mut(i),
                    end_jd.get_mut(i),
                ) {
                    *a = r.level;
                    *b = r.lord.index();
                    *c = r.parent;
                    *d = r.start_jd;
                    *e = r.end_jd;
                }
            }
            u32::try_from(n).unwrap_or(u32::MAX)
        }
    }

    /// The number of tree nodes a chart of `depth` levels holds.
    #[diplomat::opaque]
    pub struct Info;

    impl Info {
        /// The node count for a depth.
        pub fn node_count_for_depth(depth: u8) -> u32 {
            u32::try_from(slice::Chart::node_count_for_depth(depth)).unwrap_or(u32::MAX)
        }
    }
}

use teistro_spike_slice as slice;

/// A host provider seen from the slice as an [`slice::EphemerisPort`].
struct HostPort<P: ffi::EphemerisPort>(P);

impl<P: ffi::EphemerisPort> slice::EphemerisPort for HostPort<P> {
    fn position(&self, jd_ut: f64, body: slice::Body) -> Result<slice::Position, i32> {
        let p = self.0.position(jd_ut, body.into());
        Ok(slice::Position {
            longitude_deg: p.longitude_deg,
            latitude_deg: p.latitude_deg,
            speed_deg_per_day: p.speed_deg_per_day,
        })
    }
}

impl From<slice::Error> for ffi::ErrorCode {
    fn from(error: slice::Error) -> Self {
        match error {
            slice::Error::DepthOutOfRange { .. } => ffi::ErrorCode::DepthOutOfRange,
            slice::Error::JulianDayNotFinite => ffi::ErrorCode::JulianDayNotFinite,
            slice::Error::Provider { .. } => ffi::ErrorCode::Provider,
            slice::Error::PositionNotFinite { .. } => ffi::ErrorCode::PositionNotFinite,
        }
    }
}

fn to_slice_settings(settings: &ffi::Settings) -> slice::Settings {
    slice::Settings {
        ayanamsha: settings.ayanamsha.into(),
        node: settings.node.into(),
        dasha_depth: settings.dasha_depth,
    }
}
