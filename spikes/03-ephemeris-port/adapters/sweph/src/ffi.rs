//! The Swiss Ephemeris entry points the adapter uses, declared by hand:
//! the library's header is C and the adapter needs nine functions, not a
//! generated binding of four hundred. Every declaration is checked against
//! `swephexp.h` of the sources being compiled.

#![allow(
    unsafe_code,
    reason = "the declarations of the C entry points; each call site carries its own safety argument"
)]

use std::os::raw::{c_char, c_int};

/// The length the library documents for its message buffers (`AS_MAXCH`).
pub(crate) const MESSAGE_LEN: usize = 256;

/// `SE_ECL_NUT`: the pseudo-body whose `swe_calc` answer is the obliquity
/// and nutation.
pub(crate) const ECL_NUT: c_int = -1;

/// The `SEFLG_*` bits the adapter sets or reads.
pub(crate) mod flag {
    /// `SEFLG_JPLEPH`.
    pub(crate) const JPLEPH: i32 = 1;
    /// `SEFLG_SWIEPH`.
    pub(crate) const SWIEPH: i32 = 2;
    /// `SEFLG_MOSEPH`.
    pub(crate) const MOSEPH: i32 = 4;
    /// `SEFLG_HELCTR`.
    pub(crate) const HELCTR: i32 = 8;
    /// `SEFLG_TRUEPOS`: no light-time correction.
    pub(crate) const TRUEPOS: i32 = 16;
    /// `SEFLG_J2000`.
    pub(crate) const J2000: i32 = 32;
    /// `SEFLG_NONUT`.
    pub(crate) const NONUT: i32 = 64;
    /// `SEFLG_SPEED`.
    pub(crate) const SPEED: i32 = 256;
    /// `SEFLG_NOGDEFL`.
    pub(crate) const NOGDEFL: i32 = 512;
    /// `SEFLG_NOABERR`.
    pub(crate) const NOABERR: i32 = 1024;
    /// `SEFLG_EQUATORIAL`.
    pub(crate) const EQUATORIAL: i32 = 2048;
    /// `SEFLG_BARYCTR`.
    pub(crate) const BARYCTR: i32 = 16384;
    /// `SEFLG_TOPOCTR`.
    pub(crate) const TOPOCTR: i32 = 32768;
    /// `SEFLG_SIDEREAL`.
    pub(crate) const SIDEREAL: i32 = 65536;
}

unsafe extern "C" {
    /// `char *swe_version(char *svers)`: writes the version into `svers`.
    pub(crate) fn swe_version(svers: *mut c_char) -> *mut c_char;
    /// `void swe_set_ephe_path(const char *path)`: process-wide.
    pub(crate) fn swe_set_ephe_path(path: *const c_char);
    /// `int32 swe_calc_ut(double, int32, int32, double *xx, char *serr)`.
    pub(crate) fn swe_calc_ut(
        tjd_ut: f64,
        ipl: c_int,
        iflag: i32,
        xx: *mut f64,
        serr: *mut c_char,
    ) -> i32;
    /// `int32 swe_calc(double, int32, int32, double *xx, char *serr)`.
    pub(crate) fn swe_calc(
        tjd_et: f64,
        ipl: c_int,
        iflag: i32,
        xx: *mut f64,
        serr: *mut c_char,
    ) -> i32;
    /// `void swe_set_topo(double geolon, double geolat, double geoalt)`:
    /// process-wide.
    pub(crate) fn swe_set_topo(geolon: f64, geolat: f64, geoalt: f64);
    /// `void swe_set_sid_mode(int32 sid_mode, double t0, double ayan_t0)`:
    /// process-wide.
    pub(crate) fn swe_set_sid_mode(sid_mode: i32, t0: f64, ayan_t0: f64);
    /// `int32 swe_get_ayanamsa_ex_ut(double, int32, double *daya, char *serr)`.
    pub(crate) fn swe_get_ayanamsa_ex_ut(
        tjd_ut: f64,
        iflag: i32,
        daya: *mut f64,
        serr: *mut c_char,
    ) -> i32;
    /// `int32 swe_get_ayanamsa_ex(double, int32, double *daya, char *serr)`.
    pub(crate) fn swe_get_ayanamsa_ex(
        tjd_et: f64,
        iflag: i32,
        daya: *mut f64,
        serr: *mut c_char,
    ) -> i32;
    /// `double swe_deltat_ex(double tjd, int32 iflag, char *serr)`: days.
    pub(crate) fn swe_deltat_ex(tjd: f64, iflag: i32, serr: *mut c_char) -> f64;
}
