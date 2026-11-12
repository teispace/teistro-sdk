//! The sky pose (`docs/03-design/sky-pose.md`): a device's camera pointed
//! at the sky, and where each point of a [`SkyView`] falls in its picture.
//!
//! The camera is stated by its line of sight (an azimuth from true north
//! through east and an altitude), its roll about that line, its horizontal
//! field of view and its picture's aspect ratio. The picture is the
//! rectilinear (gnomonic) projection about the line of sight (Snyder 1987,
//! ch. 22), normalised to `[0, 1]` with the origin at the top left.
//!
//! ```
//! use teistro_astro::sky_pose::Pose;
//!
//! // Facing due east on the horizon, a 60° field, a 3:4 portrait picture.
//! let pose = Pose::new(90.0, 0.0, 0.0, 60.0, 0.75)?;
//! // The line of sight is the centre.
//! let centre = pose.place(90.0, 0.0);
//! assert!(centre.in_view && (centre.x.unwrap_or(0.0) - 0.5).abs() < 1e-12);
//! // A point to the north is to the left, and one behind is not drawn.
//! assert!(pose.place(80.0, 0.0).x.is_some_and(|x| x < 0.5));
//! let behind = pose.place(270.0, 0.0);
//! assert!(!behind.in_view && behind.x.is_none() && behind.off_axis_deg > 179.0);
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

use serde::{Deserialize, Serialize};
use teistro_core::angle::normalise_deg;
use teistro_core::error::Error;
use teistro_core::math;

use crate::iau::vector::{Vector3, pdp, pn};
use crate::iau::{DEG2RAD, RAD2DEG};
use crate::sky_view::{EclipticPoint, EclipticPointKind, SkyView};

/// Where a camera points and what it frames: its line of sight, its roll
/// and its field (`03-design/sky-pose.md` §1).
///
/// ```
/// use teistro_astro::sky_pose::Pose;
///
/// let pose = Pose::new(-90.0, 30.0, 0.0, 60.0, 1.0)?;
/// // An azimuth is answered in [0, 360).
/// assert_eq!(pose.azimuth_deg, 270.0);
/// // A square picture is as tall as it is wide.
/// assert!((pose.vertical_fov_deg() - 60.0).abs() < 1e-12);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// Read from JSON it is unchecked, as a request carries it; [`Pose::new`]
/// checks it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "DevicePose"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Pose {
    /// Where the line of sight points, degrees in `[0, 360)` from true
    /// north through east.
    pub azimuth_deg: f64,
    /// The line of sight's height above the airless horizon, degrees,
    /// −90 to 90.
    pub altitude_deg: f64,
    /// The picture's turn about the line of sight, degrees, −180 to 180:
    /// positive as the phone is turned clockwise seen from its screen, 0
    /// when the picture's up leans to the zenith.
    pub roll_deg: f64,
    /// The field of view across the picture, edge to edge, degrees, above
    /// 0 and below 180.
    pub horizontal_fov_deg: f64,
    /// The picture's width over its height, 0.1 to 10.
    pub aspect_ratio: f64,
}

/// A point of the sky placed in the picture.
///
/// ```
/// use teistro_astro::sky_pose::Placement;
///
/// let edge = Placement { in_view: true, x: Some(1.0), y: Some(0.5), off_axis_deg: 45.0 };
/// assert!(edge.in_view);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "ScreenPlacement"))]
#[serde(rename_all = "camelCase")]
pub struct Placement {
    /// Whether it is in front of the camera and inside the frame, edges
    /// included.
    pub in_view: bool,
    /// Across the picture, 0 at its left edge and 1 at its right; outside
    /// `[0, 1]` beyond them, and null behind the camera.
    pub x: Option<f64>,
    /// Down the picture, 0 at its top edge and 1 at its bottom; outside
    /// `[0, 1]` beyond them, and null behind the camera.
    pub y: Option<f64>,
    /// Its angle from the line of sight, degrees, 0 to 180.
    pub off_axis_deg: f64,
}

/// Each point of a sky view placed in the picture: one placement for each
/// point of the sky's list of the same name, in the same order.
///
/// ```
/// use teistro_astro::sky_pose::SkyScreen;
///
/// let empty = SkyScreen { grahas: vec![], stars: vec![], ecliptic: vec![], horizon: vec![] };
/// assert!(empty.grahas.is_empty());
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SkyScreen {
    /// The sky's grahas, at their refracted altitudes.
    pub grahas: Vec<Placement>,
    /// The sky's stars, at their refracted altitudes.
    pub stars: Vec<Placement>,
    /// The sky's ecliptic points, at their refracted altitudes.
    pub ecliptic: Vec<Placement>,
    /// The sky's horizon points, at altitude nought.
    pub horizon: Vec<Placement>,
}

/// A sky view seen through a camera: the sky unchanged, the pose, and
/// where each of the sky's points falls in the picture.
///
/// ```
/// # use teistro_astro::sky_view::{SkyViewOptions, sky_view};
/// # use teistro_astro::{Completion, DeltaTModel};
/// # use teistro_core::quantity::{JulianDay, Place, Ut1};
/// # use teistro_core::settings::{Node, OverridePolicy};
/// # use teistro_port_ephemeris::TestProvider;
/// use teistro_astro::sky_pose::{Pose, sky_pose};
///
/// # let provider = TestProvider::new();
/// # let source = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let place = Place::try_from_degrees(27.7172, 85.324, 1_400.0)?;
/// let sky = sky_view(&source, place, JulianDay::<Ut1>::J2000, DeltaTModel::TableThenModel, Node::Mean, &SkyViewOptions::default())?;
/// let seen = sky_pose(sky, Pose::new(90.0, 0.0, 0.0, 60.0, 0.75)?);
/// // One placement for each of the sky's points, in the sky's order.
/// assert_eq!(seen.screen.grahas.len(), seen.sky.grahas.len());
/// assert_eq!(seen.screen.horizon.len(), seen.sky.horizon.len());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SkyPose {
    /// The sky view, unchanged.
    pub sky: SkyView,
    /// The pose it is seen in.
    pub pose: Pose,
    /// The field of view top to bottom, degrees.
    pub vertical_fov_deg: f64,
    /// Where each of the sky's points falls.
    pub screen: SkyScreen,
}

/// The camera's axes in east, north, up, and the picture's half-widths
/// at unit distance.
struct Camera {
    forward: Vector3,
    up: Vector3,
    right: Vector3,
    half_width: f64,
    half_height: f64,
}

/// A direction of an azimuth and an altitude, degrees, as a unit vector
/// in east, north, up.
fn direction(azimuth_deg: f64, altitude_deg: f64) -> Vector3 {
    let (sin_a, cos_a) = math::sin_cos(azimuth_deg * DEG2RAD);
    let (sin_h, cos_h) = math::sin_cos(altitude_deg * DEG2RAD);
    [sin_a * cos_h, cos_a * cos_h, sin_h]
}

/// The azimuth and the altitude of a unit vector in east, north, up,
/// degrees.
fn angles_of(unit: &Vector3) -> (f64, f64) {
    let [east, north, up] = *unit;
    (
        normalise_deg(math::atan2(east, north) * RAD2DEG),
        math::asin(up.clamp(-1.0, 1.0)) * RAD2DEG,
    )
}

/// The picture's up and right before any roll, at a line of sight of an
/// azimuth and an altitude: the directions of rising altitude and of
/// rising azimuth (§2).
fn unrolled(azimuth_deg: f64, altitude_deg: f64) -> (Vector3, Vector3) {
    let (sin_a, cos_a) = math::sin_cos(azimuth_deg * DEG2RAD);
    let (sin_h, cos_h) = math::sin_cos(altitude_deg * DEG2RAD);
    (
        [-sin_a * sin_h, -cos_a * sin_h, cos_h],
        [cos_a, -sin_a, 0.0],
    )
}

/// `a·p + b·q`.
fn mix(a: f64, p: &Vector3, b: f64, q: &Vector3) -> Vector3 {
    [
        a * p[0] + b * q[0],
        a * p[1] + b * q[1],
        a * p[2] + b * q[2],
    ]
}

/// A value refused by its field unless it lies in a range.
fn within(value: f64, (least, most): (f64, f64), field: &str, unit: &str) -> Result<f64, Error> {
    if (least..=most).contains(&value) {
        Ok(value)
    } else {
        Err(
            Error::invalid_arg(format!("`{field}` is {least} to {most}{unit}, not {value}"))
                .with_field(field),
        )
    }
}

impl Pose {
    /// The azimuths taken, degrees; answered in `[0, 360)`.
    pub const AZIMUTHS_DEG: (f64, f64) = (-360.0, 360.0);

    /// The altitudes taken, degrees.
    pub const ALTITUDES_DEG: (f64, f64) = (-90.0, 90.0);

    /// The rolls taken, degrees.
    pub const ROLLS_DEG: (f64, f64) = (-180.0, 180.0);

    /// The aspect ratios taken.
    pub const ASPECT_RATIOS: (f64, f64) = (0.1, 10.0);

    /// A pose, its azimuth answered in `[0, 360)`.
    ///
    /// ```
    /// use teistro_astro::sky_pose::Pose;
    ///
    /// let read: Pose = serde_json::from_str(r#"{"azimuthDeg": -10, "altitudeDeg": 5, "rollDeg": 0, "horizontalFovDeg": 60, "aspectRatio": 0.75}"#)?;
    /// assert_eq!(read.checked()?.azimuth_deg, 350.0);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// ```
    /// use teistro_astro::sky_pose::Pose;
    ///
    /// let refused = Pose::new(0.0, 0.0, 0.0, 180.0, 1.0).unwrap_err();
    /// assert_eq!(refused.field(), Some("horizontalFovDeg"));
    /// assert_eq!(Pose::new(0.0, 91.0, 0.0, 60.0, 1.0).unwrap_err().field(), Some("altitudeDeg"));
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming `azimuthDeg` outside −360 to 360,
    /// `altitudeDeg` outside −90 to 90, `rollDeg` outside −180 to 180,
    /// `horizontalFovDeg` at or below 0 or at or above 180 (a rectilinear
    /// picture of a half-sky is infinitely wide), or `aspectRatio` outside
    /// 0.1 to 10.
    pub fn new(
        azimuth_deg: f64,
        altitude_deg: f64,
        roll_deg: f64,
        horizontal_fov_deg: f64,
        aspect_ratio: f64,
    ) -> Result<Pose, Error> {
        let azimuth_deg = within(azimuth_deg, Self::AZIMUTHS_DEG, "azimuthDeg", " degrees")?;
        if !(horizontal_fov_deg > 0.0 && horizontal_fov_deg < 180.0) {
            return Err(Error::invalid_arg(format!(
                "a horizontal field of view is above 0 and below 180 degrees, not {horizontal_fov_deg}"
            ))
            .with_field("horizontalFovDeg")
            .with_hint("a rectilinear picture of half the sky or more is infinitely wide"));
        }
        Ok(Pose {
            azimuth_deg: normalise_deg(azimuth_deg),
            altitude_deg: within(altitude_deg, Self::ALTITUDES_DEG, "altitudeDeg", " degrees")?,
            roll_deg: within(roll_deg, Self::ROLLS_DEG, "rollDeg", " degrees")?,
            horizontal_fov_deg,
            aspect_ratio: within(aspect_ratio, Self::ASPECT_RATIOS, "aspectRatio", "")?,
        })
    }

    /// The pose checked as [`Pose::new`] checks it: one read from JSON.
    ///
    /// ```
    /// use teistro_astro::sky_pose::Pose;
    ///
    /// let read: Pose = serde_json::from_str(
    ///     r#"{"azimuthDeg":-90,"altitudeDeg":30,"rollDeg":0,"horizontalFovDeg":60,"aspectRatio":1}"#,
    /// )?;
    /// // Read, it is as written; checked, its azimuth is in [0, 360).
    /// assert_eq!(read.azimuth_deg, -90.0);
    /// assert_eq!(read.checked()?.azimuth_deg, 270.0);
    /// // A field of half the sky or more is refused.
    /// let wide: Pose = serde_json::from_str(
    ///     r#"{"azimuthDeg":0,"altitudeDeg":0,"rollDeg":0,"horizontalFovDeg":200,"aspectRatio":1}"#,
    /// )?;
    /// assert!(wide.checked().is_err());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`Pose::new`].
    pub fn checked(self) -> Result<Pose, Error> {
        Pose::new(
            self.azimuth_deg,
            self.altitude_deg,
            self.roll_deg,
            self.horizontal_fov_deg,
            self.aspect_ratio,
        )
    }

    /// The pose of a camera whose line of sight and picture's up are
    /// given as vectors in east, north, up: the recipe of
    /// `03-design/sky-pose.md` §3, which turns any platform's attitude
    /// into a pose. Neither need be a unit vector, nor the up square to
    /// the line of sight. At the zenith or the nadir the line of sight
    /// names no azimuth, and the up names it with no roll.
    ///
    /// ```
    /// use teistro_astro::sky_pose::Pose;
    ///
    /// // Upright, facing north: the back camera looks north, the screen's top is up.
    /// let pose = Pose::from_axes([0.0, 1.0, 0.0], [0.0, 0.0, 1.0], 60.0, 0.75)?;
    /// assert_eq!((pose.azimuth_deg, pose.altitude_deg, pose.roll_deg), (0.0, 0.0, 0.0));
    /// // Tipped back to the zenith: the screen's top points south.
    /// let up = Pose::from_axes([0.0, 0.0, 1.0], [0.0, -1.0, 0.0], 60.0, 0.75)?;
    /// assert_eq!((up.azimuth_deg, up.altitude_deg, up.roll_deg), (0.0, 90.0, 0.0));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` naming `forward` for a line of sight that is no
    /// direction (zero or not finite), `up` for an up along the line of
    /// sight, and [`Pose::new`]'s refusals of the field.
    pub fn from_axes(
        forward: Vector3,
        up: Vector3,
        horizontal_fov_deg: f64,
        aspect_ratio: f64,
    ) -> Result<Pose, Error> {
        let (length, unit) = pn(&forward);
        if !(length.is_finite() && length > 0.0) {
            return Err(
                Error::invalid_arg("the line of sight is no direction").with_field("forward")
            );
        }
        let up_refused =
            || Error::invalid_arg("the up lies along the line of sight").with_field("up");
        if !up.iter().all(|component| component.is_finite()) {
            return Err(up_refused());
        }
        let (mut azimuth_deg, altitude_deg) = angles_of(&unit);
        let [up_east, up_north, _] = up;
        let level = math::hypot(unit[0], unit[1]);
        let roll_deg = if level < 1e-6 {
            // At the zenith the picture's up points away from the azimuth
            // the camera was tipped back from; at the nadir, towards it.
            let (east, north) = if unit[2] > 0.0 {
                (-up_east, -up_north)
            } else {
                (up_east, up_north)
            };
            if math::hypot(east, north) == 0.0 {
                return Err(up_refused());
            }
            azimuth_deg = normalise_deg(math::atan2(east, north) * RAD2DEG);
            0.0
        } else {
            let (up0, right0) = unrolled(azimuth_deg, altitude_deg);
            let (along_up, along_right) = (pdp(&up, &up0), pdp(&up, &right0));
            if along_up == 0.0 && along_right == 0.0 {
                return Err(up_refused());
            }
            math::atan2(along_right, along_up) * RAD2DEG
        };
        Pose::new(
            azimuth_deg,
            altitude_deg,
            roll_deg,
            horizontal_fov_deg,
            aspect_ratio,
        )
    }

    /// The field of view top to bottom, degrees:
    /// `2 atan(tan(H/2) / aspectRatio)`.
    ///
    /// ```
    /// use teistro_astro::sky_pose::Pose;
    ///
    /// // A portrait picture is taller than it is wide.
    /// assert!(Pose::new(0.0, 0.0, 0.0, 60.0, 0.5)?.vertical_fov_deg() > 60.0);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn vertical_fov_deg(&self) -> f64 {
        2.0 * math::atan(self.camera().half_height) * RAD2DEG
    }

    /// The camera's axes and half-widths.
    fn camera(&self) -> Camera {
        let forward = direction(self.azimuth_deg, self.altitude_deg);
        let (up0, right0) = unrolled(self.azimuth_deg, self.altitude_deg);
        let (sin_r, cos_r) = math::sin_cos(self.roll_deg * DEG2RAD);
        let half_width = math::tan(self.horizontal_fov_deg / 2.0 * DEG2RAD);
        Camera {
            forward,
            up: mix(cos_r, &up0, sin_r, &right0),
            right: mix(cos_r, &right0, -sin_r, &up0),
            half_width,
            half_height: half_width / self.aspect_ratio,
        }
    }

    /// Where a direction of an azimuth and an altitude, degrees, falls in
    /// the picture.
    ///
    /// ```
    /// use teistro_astro::sky_pose::Pose;
    ///
    /// let pose = Pose::new(0.0, 0.0, 0.0, 90.0, 1.0)?;
    /// // 30° up from the centre of a 90° field is a third of the way to the top.
    /// let high = pose.place(0.0, 30.0);
    /// assert!((high.y.unwrap_or(0.0) - (0.5 - 30f64.to_radians().tan() / 2.0)).abs() < 1e-12);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn place(&self, azimuth_deg: f64, altitude_deg: f64) -> Placement {
        self.camera().place(&direction(azimuth_deg, altitude_deg))
    }

    /// The direction a point of the picture shows, as its azimuth and
    /// altitude in degrees: the projection run back (§2), so a tap names
    /// the sky under it. A point outside `[0, 1]` is a direction too.
    ///
    /// ```
    /// use teistro_astro::sky_pose::Pose;
    ///
    /// let pose = Pose::new(200.0, 40.0, 15.0, 70.0, 0.6)?;
    /// let (azimuth, altitude) = pose.direction_at(0.5, 0.5);
    /// assert!((azimuth - 200.0).abs() < 1e-9 && (altitude - 40.0).abs() < 1e-9);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn direction_at(&self, x: f64, y: f64) -> (f64, f64) {
        let camera = self.camera();
        let across = (2.0 * x - 1.0) * camera.half_width;
        let down = (1.0 - 2.0 * y) * camera.half_height;
        let seen = mix(across, &camera.right, down, &camera.up);
        let seen = [
            seen[0] + camera.forward[0],
            seen[1] + camera.forward[1],
            seen[2] + camera.forward[2],
        ];
        angles_of(&pn(&seen).1)
    }
}

impl Camera {
    /// Where a unit direction falls in the picture.
    fn place(&self, direction: &Vector3) -> Placement {
        let depth = pdp(direction, &self.forward);
        let (across, upward) = (pdp(direction, &self.right), pdp(direction, &self.up));
        // atan2 of the off-axis and on-axis parts, not acos of the depth:
        // acos is ill-conditioned at the centre, where a depth one rounding
        // short of 1 reads as 1e-6 degrees off the line of sight.
        let off_axis_deg = math::atan2(math::hypot(across, upward), depth) * RAD2DEG;
        if depth <= 0.0 {
            return Placement {
                in_view: false,
                x: None,
                y: None,
                off_axis_deg,
            };
        }
        let x = 0.5 + across / depth / (2.0 * self.half_width);
        let y = 0.5 - upward / depth / (2.0 * self.half_height);
        Placement {
            in_view: (0.0..=1.0).contains(&x) && (0.0..=1.0).contains(&y),
            x: Some(x),
            y: Some(y),
            off_axis_deg,
        }
    }
}

impl SkyPose {
    /// The rising lagna point, the ecliptic's `ASCENDANT`, with its
    /// placement; none at the one instant the ecliptic lies in the
    /// horizon.
    ///
    /// ```
    /// # use teistro_astro::sky_view::{SkyViewOptions, sky_view};
    /// # use teistro_astro::{Completion, DeltaTModel};
    /// # use teistro_core::quantity::{JulianDay, Place, Ut1};
    /// # use teistro_core::settings::{Node, OverridePolicy};
    /// # use teistro_port_ephemeris::TestProvider;
    /// use teistro_astro::sky_pose::{Pose, sky_pose};
    ///
    /// # let provider = TestProvider::new();
    /// # let source = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// # let place = Place::try_from_degrees(27.7172, 85.324, 1_400.0)?;
    /// let sky = sky_view(&source, place, JulianDay::<Ut1>::J2000, DeltaTModel::TableThenModel, Node::Mean, &SkyViewOptions::default())?;
    /// let rising_azimuth = sky.ecliptic.iter().find(|p| p.kind == teistro_astro::sky_view::EclipticPointKind::Ascendant).map_or(90.0, |p| p.azimuth_deg);
    /// // Facing the rising point on the horizon puts it in the middle.
    /// let seen = sky_pose(sky, Pose::new(rising_azimuth, 0.0, 0.0, 60.0, 0.75)?);
    /// let (point, placed) = seen.ascendant().ok_or("a rising point")?;
    /// assert!(placed.in_view && (placed.x.unwrap_or(0.0) - 0.5).abs() < 1e-9);
    /// assert!(point.altitude_deg.abs() < 1e-9);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn ascendant(&self) -> Option<(&EclipticPoint, &Placement)> {
        self.sky
            .ecliptic
            .iter()
            .zip(&self.screen.ecliptic)
            .find(|(point, _)| point.kind == EclipticPointKind::Ascendant)
    }

    /// The pose as JSON, every catalogue member of its sky written as
    /// its full key ([`SkyView::in_full`]): what a boundary section
    /// carries and seals.
    ///
    /// # Errors
    ///
    /// `INTERNAL` if it does not serialise, which a value this crate built
    /// cannot do.
    ///
    /// ```
    /// # use teistro_astro::sky_view::{SkyViewOptions, sky_view};
    /// # use teistro_astro::{Completion, DeltaTModel};
    /// # use teistro_core::quantity::{JulianDay, Place, Ut1};
    /// # use teistro_core::settings::{Node, OverridePolicy};
    /// # use teistro_port_ephemeris::TestProvider;
    /// use teistro_astro::sky_pose::{Pose, sky_pose};
    ///
    /// # let provider = TestProvider::new();
    /// # let source = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// # let place = Place::try_from_degrees(27.7172, 85.324, 1_400.0)?;
    /// let sky = sky_view(&source, place, JulianDay::<Ut1>::J2000, DeltaTModel::TableThenModel, Node::Mean, &SkyViewOptions::default())?;
    /// let full = sky_pose(sky.clone(), Pose::new(90.0, 0.0, 0.0, 60.0, 0.75)?).in_full()?;
    /// // The sky inside is the sky's own full form; the pose and screen beside it.
    /// assert_eq!(full["sky"], sky.in_full()?);
    /// assert!(full["pose"].is_object() && full["screen"].is_object());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn in_full(&self) -> Result<serde_json::Value, Error> {
        let mut value = serde_json::to_value(self)
            .map_err(|error| Error::internal(format!("a sky pose did not serialise: {error}")))?;
        if let Some(sky) = value.get_mut("sky") {
            *sky = self.sky.in_full()?;
        }
        Ok(value)
    }
}

/// A sky view seen in a pose: each graha, star and ecliptic point placed
/// at its refracted altitude and its azimuth, as the camera sees the
/// refracted sky, and each horizon point at altitude nought.
///
/// ```
/// # use teistro_astro::sky_view::{SkyViewOptions, sky_view};
/// # use teistro_astro::{Completion, DeltaTModel};
/// # use teistro_core::quantity::{JulianDay, Place, Ut1};
/// # use teistro_core::settings::{Node, OverridePolicy};
/// # use teistro_port_ephemeris::TestProvider;
/// use teistro_astro::sky_pose::{Pose, sky_pose};
///
/// # let provider = TestProvider::new();
/// # let source = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let place = Place::try_from_degrees(27.7172, 85.324, 1_400.0)?;
/// let sky = sky_view(&source, place, JulianDay::<Ut1>::J2000, DeltaTModel::TableThenModel, Node::Mean, &SkyViewOptions::default())?;
/// let seen = sky_pose(sky.clone(), Pose::new(0.0, 90.0, 0.0, 60.0, 0.75)?);
/// // The sky is answered unchanged, and every point placed.
/// assert_eq!(seen.sky, sky);
/// assert_eq!(seen.screen.stars.len(), 27);
/// // A camera at the zenith sees no point of the horizon.
/// assert!(seen.screen.horizon.iter().all(|placed| !placed.in_view));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[must_use]
pub fn sky_pose(sky: SkyView, pose: Pose) -> SkyPose {
    let camera = pose.camera();
    let at =
        |azimuth_deg: f64, altitude_deg: f64| camera.place(&direction(azimuth_deg, altitude_deg));
    let screen = SkyScreen {
        grahas: sky
            .grahas
            .iter()
            .map(|placed| at(placed.azimuth_deg, placed.refracted_altitude_deg))
            .collect(),
        stars: sky
            .stars
            .iter()
            .map(|placed| at(placed.azimuth_deg, placed.refracted_altitude_deg))
            .collect(),
        ecliptic: sky
            .ecliptic
            .iter()
            .map(|point| at(point.azimuth_deg, point.refracted_altitude_deg))
            .collect(),
        horizon: sky
            .horizon
            .iter()
            .map(|point| at(point.azimuth_deg, 0.0))
            .collect(),
    };
    SkyPose {
        vertical_fov_deg: 2.0 * math::atan(camera.half_height) * RAD2DEG,
        sky,
        pose,
        screen,
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::float_cmp,
        clippy::indexing_slicing,
        reason = "tests fail by panicking, index what they built and compare what is exact"
    )]

    use proptest::prelude::*;
    use teistro_core::angle::difference_deg;

    use super::*;
    use crate::iau::vector::sxp;

    /// The camera's line of sight and picture's up.
    fn axes(pose: &Pose) -> (Vector3, Vector3) {
        let camera = pose.camera();
        (camera.forward, camera.up)
    }

    fn pose(azimuth: f64, altitude: f64, roll: f64, fov: f64, aspect: f64) -> Pose {
        Pose::new(azimuth, altitude, roll, fov, aspect).unwrap()
    }

    #[test]
    fn the_line_of_sight_is_the_centre_at_every_pose() {
        for (azimuth, altitude, roll) in [
            (0.0, 0.0, 0.0),
            (123.4, 37.0, -61.0),
            (300.0, -80.0, 170.0),
            (45.0, 90.0, 33.0),
            (210.0, -90.0, -90.0),
        ] {
            let placed = pose(azimuth, altitude, roll, 63.0, 0.56).place(azimuth, altitude);
            assert!(placed.in_view, "{azimuth} {altitude} {roll}");
            assert!((placed.x.unwrap() - 0.5).abs() < 1e-12, "{placed:?}");
            assert!((placed.y.unwrap() - 0.5).abs() < 1e-12, "{placed:?}");
            assert!(placed.off_axis_deg < 1e-9, "{placed:?}");
        }
    }

    #[test]
    fn a_point_on_the_frame_edge_is_in_and_past_it_is_out() {
        // A 90° field across a square picture facing north: 45° east is
        // the right edge and 45° up the top edge.
        let square = pose(0.0, 0.0, 0.0, 90.0, 1.0);
        let edge = square.place(45.0, 0.0);
        assert!((edge.x.unwrap() - 1.0).abs() < 1e-12 && (edge.y.unwrap() - 0.5).abs() < 1e-12);
        assert!((edge.off_axis_deg - 45.0).abs() < 1e-12);
        assert!(square.place(44.999, 0.0).in_view);
        assert!(!square.place(45.001, 0.0).in_view);
        let top = square.place(0.0, 45.0);
        assert!(top.y.unwrap().abs() < 1e-12 && (top.x.unwrap() - 0.5).abs() < 1e-12);
        assert!(square.place(0.0, 44.999).in_view && !square.place(0.0, 45.001).in_view);
        // Past the edge a point is still drawn, outside [0, 1].
        let past = square.place(60.0, 0.0);
        assert!(!past.in_view && past.x.unwrap() > 1.0);
        assert!((past.x.unwrap() - (0.5 + 3f64.sqrt() / 2.0)).abs() < 1e-12);
    }

    #[test]
    fn a_roll_of_ninety_degrees_turns_the_right_to_the_top() {
        let right = pose(0.0, 0.0, 0.0, 90.0, 1.0).place(30.0, 0.0);
        let turned = pose(0.0, 0.0, 90.0, 90.0, 1.0).place(30.0, 0.0);
        let expected = 30f64.to_radians().tan() / 2.0;
        assert!((right.x.unwrap() - (0.5 + expected)).abs() < 1e-12);
        assert!((right.y.unwrap() - 0.5).abs() < 1e-12);
        assert!((turned.x.unwrap() - 0.5).abs() < 1e-12, "{turned:?}");
        assert!(
            (turned.y.unwrap() - (0.5 - expected)).abs() < 1e-12,
            "{turned:?}"
        );
        // And the zenith's side, up at roll 0, is now on the left.
        let high = pose(0.0, 0.0, 90.0, 90.0, 1.0).place(0.0, 30.0);
        assert!(
            (high.x.unwrap() - (0.5 - expected)).abs() < 1e-12,
            "{high:?}"
        );
        // A roll of −90° turns it the other way, and 180° turns it over.
        let back = pose(0.0, 0.0, -90.0, 90.0, 1.0).place(30.0, 0.0);
        assert!((back.y.unwrap() - (0.5 + expected)).abs() < 1e-12);
        let over = pose(0.0, 0.0, 180.0, 90.0, 1.0).place(30.0, 0.0);
        assert!((over.x.unwrap() - (0.5 - expected)).abs() < 1e-12);
    }

    #[test]
    fn a_point_behind_the_camera_is_out_with_no_screen_point() {
        let north = pose(0.0, 0.0, 0.0, 170.0, 1.0);
        for (azimuth, altitude) in [(180.0, 0.0), (95.0, 0.0), (265.0, 0.0), (135.0, -20.0)] {
            let placed = north.place(azimuth, altitude);
            assert!(!placed.in_view && placed.x.is_none() && placed.y.is_none());
            assert!(placed.off_axis_deg > 90.0, "{placed:?}");
        }
        // Square to the line of sight the cosine rounds either side of 0,
        // so the point may get a screen point, but never one in the frame.
        for azimuth in [90.0, 270.0] {
            let placed = north.place(azimuth, 0.0);
            assert!(!placed.in_view, "{placed:?}");
            assert!((placed.off_axis_deg - 90.0).abs() < 1e-9, "{placed:?}");
        }
        // Just in front of the side is far outside the frame.
        let side = north.place(89.0, 0.0);
        assert!(side.x.unwrap() > 1.0 && !side.in_view);
    }

    /// At roll 0 the zenith stands straight above the centre, `cot h₀ /
    /// (2 t_y)` above it, whatever the azimuth; the nadir as far below.
    #[test]
    fn the_zenith_and_the_nadir_stand_above_and_below_the_centre() {
        for azimuth in [0.0, 77.0, 180.0, 359.0] {
            for altitude in [30.0, 60.0, 89.0] {
                let camera = pose(azimuth, altitude, 0.0, 120.0, 0.5);
                let half_height = 60f64.to_radians().tan() / 0.5;
                let cot = 1.0 / altitude.to_radians().tan();
                let zenith = camera.place(0.0, 90.0);
                assert!((zenith.x.unwrap() - 0.5).abs() < 1e-12, "{zenith:?}");
                assert!(
                    (zenith.y.unwrap() - (0.5 - cot / (2.0 * half_height))).abs() < 1e-12,
                    "{azimuth} {altitude} {zenith:?}"
                );
                assert!((zenith.off_axis_deg - (90.0 - altitude)).abs() < 1e-9);
                let low = pose(azimuth, -altitude, 0.0, 120.0, 0.5).place(0.0, -90.0);
                assert!((low.x.unwrap() - 0.5).abs() < 1e-12);
                assert!((low.y.unwrap() - (0.5 + cot / (2.0 * half_height))).abs() < 1e-12);
            }
            // Pointing at the zenith puts it in the middle, and the
            // horizon of the camera's azimuth below it: the picture's up
            // is the opposite azimuth.
            let up = pose(azimuth, 90.0, 0.0, 60.0, 1.0);
            let zenith = up.place(123.0, 90.0);
            assert!(
                (zenith.x.unwrap() - 0.5).abs() < 1e-12 && (zenith.y.unwrap() - 0.5).abs() < 1e-12
            );
            assert!(up.place(azimuth, 60.0).y.unwrap() > 0.5);
            assert!(up.place(azimuth + 180.0, 60.0).y.unwrap() < 0.5);
            assert!(!up.place(azimuth, 0.0).in_view);
            // The nadir mirrors it: the picture's up is the azimuth itself.
            let down = pose(azimuth, -90.0, 0.0, 60.0, 1.0);
            assert!(down.place(azimuth, -60.0).y.unwrap() < 0.5);
        }
    }

    #[test]
    fn a_field_of_a_half_sky_and_other_bad_poses_are_refused_by_field() {
        for (bad, field) in [
            (Pose::new(0.0, 0.0, 0.0, 180.0, 1.0), "horizontalFovDeg"),
            (Pose::new(0.0, 0.0, 0.0, 200.0, 1.0), "horizontalFovDeg"),
            (Pose::new(0.0, 0.0, 0.0, 0.0, 1.0), "horizontalFovDeg"),
            (Pose::new(0.0, 0.0, 0.0, f64::NAN, 1.0), "horizontalFovDeg"),
            (Pose::new(361.0, 0.0, 0.0, 60.0, 1.0), "azimuthDeg"),
            (Pose::new(f64::NAN, 0.0, 0.0, 60.0, 1.0), "azimuthDeg"),
            (Pose::new(0.0, -90.5, 0.0, 60.0, 1.0), "altitudeDeg"),
            (Pose::new(0.0, 0.0, 181.0, 60.0, 1.0), "rollDeg"),
            (Pose::new(0.0, 0.0, 0.0, 60.0, 0.0), "aspectRatio"),
            (Pose::new(0.0, 0.0, 0.0, 60.0, 11.0), "aspectRatio"),
            (
                Pose::from_axes([0.0; 3], [0.0, 0.0, 1.0], 60.0, 1.0),
                "forward",
            ),
            (
                Pose::from_axes([f64::NAN, 1.0, 0.0], [0.0, 0.0, 1.0], 60.0, 1.0),
                "forward",
            ),
            (
                Pose::from_axes([0.0, 1.0, 0.0], [0.0, 2.0, 0.0], 60.0, 1.0),
                "up",
            ),
            (
                Pose::from_axes([0.0, 0.0, 1.0], [0.0, 0.0, 1.0], 60.0, 1.0),
                "up",
            ),
        ] {
            assert_eq!(bad.unwrap_err().field(), Some(field));
        }
        assert_eq!(pose(-360.0, 0.0, 0.0, 60.0, 1.0).azimuth_deg, 0.0);
        assert_eq!(pose(360.0, 0.0, 0.0, 60.0, 1.0).azimuth_deg, 0.0);
    }

    #[test]
    fn a_pose_is_read_back_from_its_own_axes() {
        for (azimuth, altitude, roll) in [
            (0.0, 0.0, 0.0),
            (10.0, 20.0, 30.0),
            (250.0, -45.0, -170.0),
            (359.0, 89.9, 90.0),
            (90.0, -89.9, -45.0),
        ] {
            let asked = pose(azimuth, altitude, roll, 65.0, 0.75);
            let (forward, up) = axes(&asked);
            // A longer, leaning up names the same picture.
            let leaning = mix(3.0, &up, 0.2, &forward);
            let back = Pose::from_axes(sxp(2.0, &forward), leaning, 65.0, 0.75).unwrap();
            assert!(
                difference_deg(back.azimuth_deg, azimuth).abs() < 1e-9,
                "{back:?}"
            );
            assert!((back.altitude_deg - altitude).abs() < 1e-9, "{back:?}");
            assert!(difference_deg(back.roll_deg, roll).abs() < 1e-9, "{back:?}");
        }
        // At the zenith and the nadir the pose read back is another name
        // for the same picture: every point lands where it did.
        for (altitude, roll) in [(90.0, 40.0), (-90.0, -130.0)] {
            let asked = pose(70.0, altitude, roll, 80.0, 1.2);
            let (forward, up) = axes(&asked);
            let back = Pose::from_axes(forward, up, 80.0, 1.2).unwrap();
            assert_eq!(back.roll_deg, 0.0);
            for (azimuth, height) in [
                (10.0, 50.0 * altitude.signum()),
                (200.0, 70.0 * altitude.signum()),
            ] {
                let (one, other) = (asked.place(azimuth, height), back.place(azimuth, height));
                assert!(
                    (one.x.unwrap() - other.x.unwrap()).abs() < 1e-9,
                    "{one:?} {other:?}"
                );
                assert!(
                    (one.y.unwrap() - other.y.unwrap()).abs() < 1e-9,
                    "{one:?} {other:?}"
                );
            }
        }
    }

    proptest! {
        /// The way back recovers the direction: a screen point run back
        /// and projected again lands where it was, and a direction in
        /// front projected and run back is itself.
        #[test]
        fn the_way_back_recovers_the_direction(
            azimuth in 0.0f64..360.0,
            altitude in -90.0f64..=90.0,
            roll in -180.0f64..=180.0,
            fov in 1.0f64..170.0,
            aspect in 0.2f64..5.0,
            x in -0.5f64..1.5,
            y in -0.5f64..1.5,
        ) {
            let camera = pose(azimuth, altitude, roll, fov, aspect);
            let (seen_azimuth, seen_altitude) = camera.direction_at(x, y);
            let placed = camera.place(seen_azimuth, seen_altitude);
            prop_assert!((placed.x.unwrap() - x).abs() < 1e-9, "{placed:?}");
            prop_assert!((placed.y.unwrap() - y).abs() < 1e-9, "{placed:?}");
            prop_assert_eq!(placed.in_view, (0.0..=1.0).contains(&x) && (0.0..=1.0).contains(&y));
            let (again_azimuth, again_altitude) = camera.direction_at(placed.x.unwrap(), placed.y.unwrap());
            prop_assert!((again_altitude - seen_altitude).abs() < 1e-9);
            // The azimuth is only defined off the poles.
            if seen_altitude.abs() < 89.0 {
                prop_assert!(difference_deg(again_azimuth, seen_azimuth).abs() < 1e-9);
            }
        }
    }
}
