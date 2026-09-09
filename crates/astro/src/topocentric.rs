//! The topocentric step: a position seen from a place on the Earth
//! rather than from its centre.
//!
//! Designed from the measurement in
//! `docs/03-design/topocentric-measured.md`, which falsified the reading
//! the design page had carried since Phase 2 — "the observer's geocentric
//! position (WGS84) and the parallax". A displacement is what a
//! topocentric position obviously is, and a displacement alone leaves a
//! third of an arcsecond on every body, on Saturn as much as on the
//! Moon. It takes four things:
//!
//! 1. the station's own **aberration**, because the observer is moving
//!    four hundred metres a second and the light arrives from a
//!    direction turned by that, one and a half parts in a million
//!    whatever the body's distance;
//! 2. the displacement applied to the direction the light actually came
//!    from, not to the apparent one the Earth's own motion has already
//!    turned by twenty arcseconds;
//! 3. the body carried over the **light time the station saves** by
//!    standing up to an Earth radius nearer than the centre;
//! 4. nothing at all done to a point that is a **direction** rather than
//!    a place ([`Body::is_placed`]).
//!
//! Terms 2 and 3 are each worth about a third of an arcsecond on the
//! Moon and nothing on anything else, and only together: applied one at
//! a time they make the answer worse.
//!
//! The velocity is transformed with the position rather than differenced
//! numerically, because the whole state is known: the step changes the
//! Moon's longitude speed by up to five and a half degrees a day, a
//! third of its own value, so a step that moved positions and left
//! speeds alone would be wrong by more than it corrected.

use teistro_core::angle::normalise_deg;
use teistro_core::quantity::{JulianDay, Place, Tt, Ut1};
use teistro_port_ephemeris::{Body, Cell, Corrections};

use crate::iau::vector::{self, Vector3};
use crate::iau::{AULT, CMPS, DAU, DAYSEC, DEG2RAD, RAD2DEG, apparent};
use crate::sky::{self, Spherical};

/// A velocity in astronomical units a day, as a fraction of the speed of
/// light.
const PER_C: f64 = DAU / (DAYSEC * CMPS);

/// The light time of one astronomical unit, days.
const LIGHT_TIME_PER_AU: f64 = AULT / DAYSEC;

/// Where the observer is, how fast, and what the sky around them is
/// doing: everything one instant's bodies share.
///
/// Built in the true equator and equinox of date, which is the frame
/// [`sky::observer`] answers in; [`Station::in_ecliptic`] turns it into
/// the ecliptic of the same date, so that columns in either coordinate
/// system are transformed in the frame they are already in rather than
/// rotated twice a cell.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Station {
    /// The station's geocentric position, astronomical units.
    pub position_au: Vector3,
    /// Its velocity, astronomical units a day.
    pub velocity_au_per_day: Vector3,
    /// Its acceleration, astronomical units a day squared.
    pub acceleration_au_per_day2: Vector3,
    /// The Earth's barycentric velocity, astronomical units a day.
    pub earth_au_per_day: Vector3,
    /// The Sun's distance, astronomical units.
    pub sun_distance_au: f64,
}

impl Station {
    /// The station at a place and an instant, in the true equator and
    /// equinox of date.
    ///
    /// ```
    /// use teistro_astro::topocentric::Station;
    /// use teistro_core::quantity::{JulianDay, Place, Tt, Ut1};
    ///
    /// let place = Place::try_from_degrees(27.7172, 85.324, 1_400.0).expect("Kathmandu");
    /// let at = Station::at(place, JulianDay::<Ut1>::J2000, JulianDay::<Tt>::literal(2_451_545.0));
    /// assert!((at.sun_distance_au - 0.9833).abs() < 1e-3);
    /// ```
    #[must_use]
    pub fn at(place: Place, ut1: JulianDay<Ut1>, tt: JulianDay<Tt>) -> Station {
        let observer = sky::observer(place, ut1, tt);
        let earth = sky::earth_at(tt);
        Station {
            position_au: observer.position_au,
            velocity_au_per_day: observer.velocity_au_per_day,
            acceleration_au_per_day2: observer.acceleration_au_per_day2,
            earth_au_per_day: earth.velocity_au_per_day,
            sun_distance_au: earth.sun_distance_au,
        }
    }

    /// The same station with every vector turned into the ecliptic of
    /// date, for columns that are ecliptic.
    #[must_use]
    pub fn in_ecliptic(self, obliquity_deg: f64) -> Station {
        let (se, ce) = (obliquity_deg * DEG2RAD).sin_cos();
        let turn = |v: Vector3| [v[0], v[1] * ce + v[2] * se, -v[1] * se + v[2] * ce];
        Station {
            position_au: turn(self.position_au),
            velocity_au_per_day: turn(self.velocity_au_per_day),
            acceleration_au_per_day2: turn(self.acceleration_au_per_day2),
            earth_au_per_day: turn(self.earth_au_per_day),
            sun_distance_au: self.sun_distance_au,
        }
    }

    /// One cell as this station sees it: the same longitude and latitude
    /// convention, the same units, in the same coordinate system the cell
    /// is already in.
    ///
    /// A body that is not [placed](Body::is_placed) is returned
    /// unchanged, because no observer sees a direction displaced.
    /// `corrections` says which of the terms belong: the aberration ones
    /// are skipped for a geometric position, and the light-time carry for
    /// one with no light time.
    #[must_use]
    pub fn seen(&self, cell: Cell, body: Body, corrections: Corrections, speeds: bool) -> Cell {
        if !body.is_placed() || !cell.is_ok() {
            return cell;
        }
        let apparent_direction = vector::s2c(cell.lon * DEG2RAD, cell.lat * DEG2RAD);
        // The provider's direction is apparent, so the Earth's own motion
        // has already turned it; the displacement belongs on the direction
        // the light came from.
        let natural = if corrections.aberration {
            natural_direction(
                &apparent_direction,
                &self.earth_beta(),
                self.sun_distance_au,
            )
        } else {
            apparent_direction
        };
        let mut position = vector::sxp(cell.dist, &natural);
        let velocity = rate_of(cell);
        if corrections.light_time {
            // The station stands nearer the body than the centre does, so
            // the light it sees left later, and the body had not travelled
            // as far. What travels is the body's barycentric path.
            let straight = subtract(position, self.position_au);
            let saved = (cell.dist - vector::pm(&straight)) * LIGHT_TIME_PER_AU;
            position = [
                velocity[0].mul_add(saved, position[0]) + self.earth_au_per_day[0] * saved,
                velocity[1].mul_add(saved, position[1]) + self.earth_au_per_day[1] * saved,
                velocity[2].mul_add(saved, position[2]) + self.earth_au_per_day[2] * saved,
            ];
        }
        let relative = subtract(position, self.position_au);
        let relative_rate = subtract(velocity, self.velocity_au_per_day);
        let (distance, line) = vector::pn(&relative);
        let closing = vector::pdp(&relative, &relative_rate);
        // The direction's own rate: the velocity with the part along the
        // line of sight taken out and the distance divided through.
        let scale = closing / (distance * distance * distance);
        let turning = [
            relative_rate[0] / distance - relative[0] * scale,
            relative_rate[1] / distance - relative[1] * scale,
            relative_rate[2] / distance - relative[2] * scale,
        ];
        let (direction, direction_rate) = if corrections.aberration {
            let beta = self.observer_beta();
            let bm1 = (1.0 - vector::pdp(&beta, &beta)).sqrt();
            (
                apparent::ab(&line, &beta, self.sun_distance_au, bm1),
                add(turning, vector::sxp(PER_C, &self.acceleration_au_per_day2)),
            )
        } else {
            (line, turning)
        };
        let (lon, lat) = vector::c2s(&direction);
        let (lon_speed, lat_speed) = if speeds {
            rates(&direction, &direction_rate)
        } else {
            (0.0, 0.0)
        };
        Cell {
            lon: normalise_deg(lon * RAD2DEG),
            lat: lat * RAD2DEG,
            dist: distance,
            lon_speed,
            lat_speed,
            dist_speed: if speeds { closing / distance } else { 0.0 },
            ..cell
        }
    }

    /// The Earth's velocity as a fraction of the speed of light.
    fn earth_beta(&self) -> Vector3 {
        vector::sxp(PER_C, &self.earth_au_per_day)
    }

    /// The observer's own velocity as a fraction of the speed of light:
    /// the Earth's and the station's together.
    fn observer_beta(&self) -> Vector3 {
        vector::sxp(PER_C, &add(self.earth_au_per_day, self.velocity_au_per_day))
    }
}

/// The direction the light came from, given the direction it appears to
/// come from and the observer's velocity: `eraAb` inverted by three
/// fixed-point steps, which converges at the square of an angle that is
/// twenty arcseconds to begin with.
fn natural_direction(apparent_direction: &Vector3, beta: &Vector3, sun_au: f64) -> Vector3 {
    let bm1 = (1.0 - vector::pdp(beta, beta)).sqrt();
    let mut natural = *apparent_direction;
    for _ in 0..3 {
        let forward = apparent::ab(&natural, beta, sun_au, bm1);
        natural = vector::pn(&[
            natural[0] + apparent_direction[0] - forward[0],
            natural[1] + apparent_direction[1] - forward[1],
            natural[2] + apparent_direction[2] - forward[2],
        ])
        .1;
    }
    natural
}

/// A cell's velocity as a Cartesian rate, the same units and coordinates
/// its position is in.
fn rate_of(cell: Cell) -> Vector3 {
    let (sl, cl) = (cell.lon * DEG2RAD).sin_cos();
    let (sb, cb) = (cell.lat * DEG2RAD).sin_cos();
    let (dl, db) = (cell.lon_speed * DEG2RAD, cell.lat_speed * DEG2RAD);
    [
        cell.dist_speed
            .mul_add(cb * cl, -cell.dist * (sb * db * cl + cb * sl * dl)),
        cell.dist_speed
            .mul_add(cb * sl, cell.dist * (cb * cl * dl - sb * db * sl)),
        cell.dist_speed.mul_add(sb, cell.dist * cb * db),
    ]
}

/// The longitude and latitude rates of a direction and its own rate,
/// degrees a day.
fn rates(p: &Vector3, v: &Vector3) -> (f64, f64) {
    let planar = p[0].mul_add(p[0], p[1] * p[1]);
    let radius = vector::pm(p);
    if planar <= 0.0 || radius <= 0.0 {
        return (0.0, 0.0);
    }
    let along = p[0].mul_add(v[0], p[1] * v[1]);
    (
        (p[0] * v[1] - p[1] * v[0]) / planar * RAD2DEG,
        (v[2] * planar - p[2] * along) / (radius * radius * planar.sqrt()) * RAD2DEG,
    )
}

fn add(left: Vector3, right: Vector3) -> Vector3 {
    [left[0] + right[0], left[1] + right[1], left[2] + right[2]]
}

fn subtract(left: Vector3, right: Vector3) -> Vector3 {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

/// The spherical position a direction points at, degrees.
#[must_use]
pub fn direction_of(p: &Vector3) -> Spherical {
    let (lon, lat) = vector::c2s(p);
    Spherical {
        lon_deg: normalise_deg(lon * RAD2DEG),
        lat_deg: lat * RAD2DEG,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_port_ephemeris::{CellStatus, Source};

    use super::*;

    fn cell(lon: f64, lat: f64, dist: f64) -> Cell {
        Cell {
            lon,
            lat,
            dist,
            lon_speed: 0.5,
            lat_speed: 0.01,
            dist_speed: 1e-4,
            status: CellStatus::Ok,
            source: Source::UNKNOWN,
        }
    }

    fn kathmandu() -> Place {
        Place::try_from_degrees(27.7172, 85.324, 1_400.0).unwrap()
    }

    fn station() -> Station {
        Station::at(
            kathmandu(),
            JulianDay::<Ut1>::literal(2_460_000.5),
            JulianDay::<Tt>::literal(2_460_000.5),
        )
    }

    #[test]
    fn a_station_at_the_centre_of_the_earth_sees_what_the_centre_sees() {
        let nowhere = Station {
            position_au: [0.0; 3],
            velocity_au_per_day: [0.0; 3],
            acceleration_au_per_day2: [0.0; 3],
            earth_au_per_day: [0.0; 3],
            sun_distance_au: 1.0,
        };
        let before = cell(123.456, -2.5, 0.9);
        let after = nowhere.seen(before, Body::Mars, Corrections::APPARENT, true);
        assert!((after.lon - before.lon).abs() < 1e-9, "{}", after.lon);
        assert!((after.lat - before.lat).abs() < 1e-9);
        assert!((after.dist - before.dist).abs() < 1e-12);
        assert!((after.lon_speed - before.lon_speed).abs() < 1e-9);
        assert!((after.lat_speed - before.lat_speed).abs() < 1e-9);
        assert!((after.dist_speed - before.dist_speed).abs() < 1e-12);
    }

    #[test]
    fn a_direction_and_a_failed_cell_are_left_where_they_are() {
        let station = station();
        let before = cell(100.0, 0.0, 0.002_569_555);
        for direction in [Body::MeanNode, Body::TrueNode, Body::MeanApogee] {
            assert_eq!(
                station.seen(before, direction, Corrections::APPARENT, true),
                before,
                "{direction:?} was displaced"
            );
        }
        // The osculating apogee is a place, so it moves.
        assert_ne!(
            station.seen(before, Body::OsculatingApogee, Corrections::APPARENT, true),
            before
        );
        let failed = Cell::failed(CellStatus::OutOfRange);
        assert_eq!(
            station.seen(failed, Body::Moon, Corrections::APPARENT, true),
            failed
        );
    }

    #[test]
    fn the_moon_moves_by_a_parallax_and_a_distant_body_barely_moves() {
        let station = station().in_ecliptic(23.44);
        let moon = cell(100.0, 0.0, 0.002_569_555);
        let saturn = cell(100.0, 0.0, 9.5);
        let moved = |before: Cell| {
            let after = station.seen(before, Body::Moon, Corrections::APPARENT, true);
            teistro_core::angle::difference_deg(after.lon, before.lon).abs() * 3_600.0
        };
        let near = moved(moon);
        let far = moved(saturn);
        assert!((300.0..4_000.0).contains(&near), "the Moon moved {near}″");
        assert!(far < 5.0, "Saturn moved {far}″");
        // The distance changes by about the Earth's radius, 4.26e-5 au.
        let after = station.seen(moon, Body::Moon, Corrections::APPARENT, true);
        assert!((after.dist - moon.dist).abs() < 4.3e-5);
    }

    #[test]
    fn a_geometric_frame_takes_the_displacement_and_no_aberration() {
        let station = station();
        let before = cell(210.0, 1.0, 1.5);
        let apparent = station.seen(before, Body::Mars, Corrections::APPARENT, true);
        let geometric = station.seen(before, Body::Mars, Corrections::GEOMETRIC, true);
        // The two differ by the observer's own aberration, which is a
        // fifth of an arcsecond at this latitude, and by the annual one
        // the apparent reading takes off and puts back.
        let apart =
            teistro_core::angle::difference_deg(apparent.lon, geometric.lon).abs() * 3_600.0;
        assert!((0.01..60.0).contains(&apart), "{apart}″");
        // Both moved the body: the displacement is in each.
        for seen in [apparent, geometric] {
            assert!((seen.dist - before.dist).abs() > 1e-8);
        }
    }

    #[test]
    fn asking_for_no_speeds_leaves_the_rates_at_nought() {
        let station = station();
        let before = cell(45.0, 0.0, 0.01);
        let still = station.seen(before, Body::Moon, Corrections::APPARENT, false);
        assert_eq!(
            (still.lon_speed, still.lat_speed, still.dist_speed),
            (0.0, 0.0, 0.0)
        );
        let moving = station.seen(before, Body::Moon, Corrections::APPARENT, true);
        assert!(moving.lon_speed.abs() > 0.0);
    }

    #[test]
    fn turning_into_the_ecliptic_and_back_is_the_identity() {
        let station = station();
        let there_and_back = station.in_ecliptic(23.44).in_ecliptic(-23.44);
        let apart = |left: Vector3, right: Vector3| vector::pm(&subtract(left, right));
        assert!(apart(there_and_back.position_au, station.position_au) < 1e-15);
        assert!(
            apart(
                there_and_back.velocity_au_per_day,
                station.velocity_au_per_day
            ) < 1e-15
        );
        assert!(
            apart(
                there_and_back.acceleration_au_per_day2,
                station.acceleration_au_per_day2
            ) < 1e-15
        );
        assert!(apart(there_and_back.earth_au_per_day, station.earth_au_per_day) < 1e-15);
        assert!(direction_of(&[1.0, 0.0, 0.0]).lon_deg.abs() < 1e-12);
    }
}
