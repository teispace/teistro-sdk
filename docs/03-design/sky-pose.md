# The sky pose

Status: `built`, 2026-11-13.

SR-58's device pose (`consumers/addendum-3.md`, P3 by addendum 5). A "look
up" screen holds the phone up to the sky and draws over the camera's
picture: which grahas, junction stars, ecliptic points and horizon points
fall in the frame, and where. `almanac.skyPose` is the second record on the
sky view (`sky-view.md` §7): it reads `almanac.skyView`'s request with a
`pose` beside it, answers the sky view's value unchanged as its `sky`, and
adds where each of the sky's points falls on the screen.

Built in `crates/astro/src/sky_pose.rs`, reached as `AlmanacArea::sky_pose`,
the record `almanac.skyPose`, `ts_almanac_sky_pose` at the C boundary and
`almanac.skyPose` (`sky_pose`) in every binding.

## 1. The request

Every field of `almanac.skyView`'s request (`sky-view.md`), as that record
reads it, and `pose`, which is required:

| Field of `pose` | Meaning | Range |
|---|---|---|
| `azimuthDeg` | where the camera's line of sight points, from **true** north through east | −360 to 360, answered in `[0, 360)` |
| `altitudeDeg` | the line of sight's height above the airless horizon | −90 to 90 |
| `rollDeg` | the turn of the picture about the line of sight, positive as the user turns the phone **clockwise** while looking at its screen; 0 when the picture's up leans to the zenith | −180 to 180 |
| `horizontalFovDeg` | the field of view across the picture, edge to edge | above 0, below 180 |
| `aspectRatio` | the picture's width over its height, as drawn | 0.1 to 10 |

A field out of range, missing, or a key the pose does not read is refused
by name under `skyPose.pose` (`skyPose.pose.horizontalFovDeg`); the sky
view's own fields are refused under `skyPose` (`skyPose.eclipticStepDeg`).
`almanac.skyView` still refuses a `pose` by name (`skyView.pose`): a pose
is asked of this record, never ignored by the other.

**Why angles, not a sensor's matrix.** Each platform states its attitude in
its own frame, with its own axes and its own north (§3); a matrix handed
through would carry that frame into the SDK and leave its reading to a
guess. Three angles of the camera against the local horizon and true north
mean one thing on every platform, and §3 gives the one conversion each
platform needs.

## 2. Frames and formulas

**The local frame** is east, north, up (E, N, U), the sky view's: the
azimuth runs from north through east and the altitude from the airless
horizon (`sky-view.md` §2). A direction of azimuth *A* and altitude *h* is

    d(A, h) = (sin A cos h, cos A cos h, sin h).

**The camera.** Its line of sight is `f = d(A₀, h₀)`. Before any roll, the
picture's up is the direction of rising altitude and its right the
direction of rising azimuth:

    u₀ = (−sin A₀ sin h₀, −cos A₀ sin h₀, cos h₀),    r₀ = f × u₀ = (cos A₀, −sin A₀, 0).

The roll ρ turns the picture's up towards its right:

    u = cos ρ · u₀ + sin ρ · r₀,    r = cos ρ · r₀ − sin ρ · u₀.

These are defined at every pose, the zenith and the nadir included: there
the azimuth still says which way the picture's up points (pointing at the
zenith with azimuth *A₀*, the picture's up is the azimuth *A₀* + 180°, the
way a phone tipped back from facing *A₀* holds it).

**The projection is rectilinear (gnomonic)**, the one a camera's lens makes
of straight lines and the one its picture is: a point is carried to the
picture plane along the line through the eye. For a direction `d`, with
`z = d·f`, the point is **in front** when `z > 0`, and then

    X = (d·r) / z,    Y = (d·u) / z,

which are Snyder's gnomonic projection centred on the line of sight
(Snyder, *Map Projections — A Working Manual*, USGS Professional Paper
1395, 1987, ch. 22, eqs. 22-4 and 22-5 with *k′* = 1/cos *c*, eq. 22-3, and
cos *c* by eq. 5-3; written here with vectors, *X* is his *x* and *Y* his
*y* with *φ₁* = *h₀*, *λ*₀ = *A₀*). The picture's half-widths are

    tₓ = tan(H / 2),    t_y = tₓ / aspectRatio,

so the vertical field of view, answered as `verticalFovDeg`, is
`2 atan(t_y)`. The screen point, normalised with the origin at the top
left, is

    x = 0.5 + X / (2 tₓ),    y = 0.5 − Y / (2 t_y).

A point is **in view** when it is in front and `0 ≤ x ≤ 1` and
`0 ≤ y ≤ 1`, edges included.

**What is drawn where.** A graha, a star and an ecliptic point are drawn at
their **refracted** altitude and their azimuth: the camera sees the
refracted sky. The airless altitudes stay in `sky`, unchanged. A horizon
point is drawn at altitude nought, the airless geometric horizon the sky
view lists (`sky-view.md` §2). Below the horizon nothing is refracted
(C973), so a point there is drawn where it is.

**The way back.** A screen point is a direction again:

    X = (2x − 1) tₓ,    Y = (1 − 2y) t_y,    d = normalise(X r + Y u + f),

and `azimuth = atan2(d_E, d_N)`, `altitude = asin(d_U)`. Rust has it as
`Pose::direction_at` (a tap on the screen named as a direction); a property
test holds projecting and back to 1e-9°.

## 3. The pose from a device

Each platform gives a rotation from the device's own axes to a local
frame. The device's axes are the same on all three: *x* to the right of the
screen, *y* to its top, *z* out of the screen towards the user. The back
camera looks along −*z*.

The one recipe, once a platform's matrix `M` gives the device's axes in E,
N, U:

1. `f = −M·ẑ` (the back camera's line of sight), `u = M·ŷ` (the picture's
   up; for a picture drawn in landscape, the device axis that points to the
   top of the picture as drawn: `+x` when the phone is turned a quarter
   counter-clockwise, `−x` when turned clockwise).
2. `A₀ = atan2(f_E, f_N)`, `h₀ = asin(f_U)`.
3. `ρ = atan2(u·r₀, u·u₀)`, with `u₀` and `r₀` from §2.
4. Within 1e-6 of the zenith or the nadir, `f` names no azimuth: take
   `ρ = 0` and `A₀ = atan2(−u_E, −u_N)` at the zenith, `atan2(u_E, u_N)` at
   the nadir. Any (*A₀*, *ρ*) with the same `u` is the same picture.
5. Add the magnetic declination (east positive) to `A₀` when the platform's
   north is magnetic. The SDK does not reckon it: a geomagnetic model is
   the platform's (Android `GeomagneticField.getDeclination`, iOS through
   `xTrueNorthZVertical` or Core Location's `trueHeading`).

`Pose::from_axes` in `crates/astro/src/sky_pose.rs` is this recipe, tested
against §2 at every altitude the tests reach, the zenith and the nadir
included.

| Platform | The matrix `M` (device → E, N, U) | Its north |
|---|---|---|
| Web, `deviceorientationabsolute` | W3C *DeviceOrientation Event*, §3.1 and Appendix A.1: α, β, γ are intrinsic Tait–Bryan angles Z−X′−Y″ (α in [0, 360) about *z*, β in [−180, 180) about *x′*, γ in [−90, 90) about *y″*) and `R = R_Z(α) R_X(β) R_Y(γ)` takes a device vector into the earth frame *X* east, *Y* north, *Z* up: `M = R` | the platform's; Chrome on Android reports magnetic north, so add the declination unless the platform states true north |
| Android, `SensorManager.getRotationMatrix` (or `getRotationMatrixFromVector` on `TYPE_ROTATION_VECTOR`) | the documentation: *R* transforms "a vector from the device coordinate system to the world's coordinate system", *X* = *Y* × *Z* roughly east, *Y* to the magnetic North Pole, *Z* up, row-major: `M[i][j] = R[3i + j]`. `remapCoordinateSystem` is not needed: step 1 names the axes | magnetic: add `GeomagneticField(lat, lon, alt, time).getDeclination()` |
| iOS, Core Motion `CMAttitude.rotationMatrix` under `xTrueNorthZVertical` | the reference frame's *X* is true north and *Z* vertical, so its *Y* is west; the device's axes in it are the matrix's **rows**, `(m11, m12, m13)` for *x* and so on (the reading Apple's own *pARk* sample makes; Core Motion's reference does not say which way the matrix turns, so step 6 checks it on the device). Then *E* = −*Y*, *N* = *X*, *U* = *Z* | true under `xTrueNorthZVertical`; under `xMagneticNorthZVertical` add the declination |

6. **The check on a device.** Held upright in portrait, screen to the user,
   the back camera facing true north: the pose is azimuth 0, altitude 0,
   roll 0. Tipped back to face the zenith, altitude 90. A platform whose
   reading gives azimuth 180 or altitude −90 there has its matrix the
   other way round: transpose it.

**The field of view** is the drawn picture's, not the sensor's: a preview
that fills the screen by cropping shows less than the sensor does. iOS
gives the horizontal field of the active format
(`AVCaptureDevice.activeFormat.videoFieldOfView`); Android gives it as
`2 atan(w / 2f)` from `SENSOR_INFO_PHYSICAL_SIZE`'s width *w* and
`LENS_INFO_AVAILABLE_FOCAL_LENGTHS`'s *f*; the web has no camera intrinsics,
and an app calibrates or asks. Scale either by the share of the sensor's
width the preview shows.

## 4. Edge cases

| Case | What happens |
|---|---|
| Pointing at the zenith or the nadir | the formulas of §2 hold; the azimuth says which way the picture's up points. A point at the zenith seen by a camera at altitude *h₀* stands at `x = 0.5`, `y = 0.5 − cot h₀ / (2 t_y)` at roll 0, whatever the azimuth |
| Roll | turns the picture: at roll 90° what was to the right is up |
| A field of view at or above 180°, or at or below 0° | refused by name (`skyPose.pose.horizontalFovDeg`): a rectilinear picture of a half-sky is infinitely wide |
| A point behind the camera (`z ≤ 0`), or exactly 90° off the line of sight | not in view; `x` and `y` are null. Its `offAxisDeg` is still given, so a drawing can point an arrow to it |
| A point in front but outside the frame | not in view; `x` and `y` are given, outside `[0, 1]`, so a drawing can clip a line of the ecliptic or the horizon at the frame's edge. A line joining a point with null `x` is broken there |
| Magnetic declination | the consumer's (§3 step 5); the request's azimuth is true |
| Refraction | the refracted altitude is drawn, the airless one kept in `sky` (§2) |

## 5. The answer

One `SkyPose`, sealed in an envelope:

| Field | Meaning |
|---|---|
| `sky` | `almanac.skyView`'s value for the same request, unchanged (`sky-view.md` §1) |
| `pose` | the pose asked, its azimuth in `[0, 360)` |
| `verticalFovDeg` | the field of view top to bottom, `2 atan(tan(H/2) / aspectRatio)` |
| `screen` | `{grahas, stars, ecliptic, horizon}`: one placement for each point of the same list of `sky`, in the same order |

A placement is `inView` (boolean), `x` and `y` (the normalised screen point,
origin top left, null behind the camera) and `offAxisDeg` (the angle from
the line of sight, 0 to 180). The `n`th placement of `screen.grahas` is
`sky.grahas[n]`'s; so for every list.

**The rising lagna point** is `sky.ecliptic`'s `ASCENDANT`, and its
placement is the one at its index in `screen.ecliptic`.

**The Moon's junction stars** are the nakshatras' yogataras, the
lunar mansions' junction stars, which `stars` names by default
(`sky-view.md` §3): each is placed with its nakshatra beside it in `sky`.
Which nakshatra the Moon stands in is a sidereal reading under the
context's ayanamsha, which the day's record answers (`almanac.days`, the
nakshatra and its end); the sky view and the pose do not repeat it
(`sky-view.md` §2, *Sidereal*).

The provenance's input hash is of the sky view's input and the pose, and it
names the projection under `sky.pose` beside the sky view's
`sky.refraction`.

## 6. Tests

`crates/astro/src/sky_pose.rs` holds the analytic cases: a point on the
line of sight is at (0.5, 0.5); a point 45° to the side with a 90° field is
on the frame's edge, in view within it and out past it; roll 90° turns a
point on the right to the top; a point behind the camera is out with no
screen point; the zenith's `y` at roll 0 is `0.5 − cot h₀ / (2 t_y)` at any
azimuth, and the nadir's mirror; the way back recovers the direction over
random poses and screen points (proptest); and `Pose::from_axes` recovers
a pose from its own axes. `crates/sdk/tests/sky_pose.rs` holds the record:
its `sky` is `almanac.skyView`'s value bit for bit, the Moon's
placement re-derived from its own azimuth and altitude, and the refusals
by name. Every binding's parity runner prints the same lines
(`sky-pose-*`).

## 7. Cost

The sky view's, and per point a dot product and a division.
