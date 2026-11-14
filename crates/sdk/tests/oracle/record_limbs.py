#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Record a reference sky's limb instants for the SDK's limb oracle (SR-105).

What it writes (``limbs.json``): for each sample year, every instant at which

- a tithi ends (the Moon's apparent longitude less the Sun's crosses a
  multiple of 12 degrees) and a karana ends (a multiple of 6);
- a nakshatra ends (the Moon's sidereal longitude crosses a multiple of
  13 deg 20 min) and a yoga ends (the sum of the two sidereal longitudes
  does), under each ayanamsha recorded;
- a sankranti falls (the Sun's sidereal longitude crosses a multiple of 30);

in terrestrial time, as a Julian day; and the Sun's rise at a place on
every local date of the year, in UT1, under the SDK's two horizon
conventions read as their definitions (``03-design/sunrise-convention.md``):
the geocentric altitude of the centre equals -34' (standard refraction)
plus the horizontal parallax, less the semidiameter for the upper limb.

Every longitude is apparent and geocentric, of the true equinox of date,
and every sidereal one is that longitude less the **true** ayanamsha (the
mean value plus the nutation in longitude), which is how the Indian
Astronomical Ephemeris defines a nirayana longitude (``True Ayanamsa = Mean
Ayanamsa + Nutation in Longitude``). The oracle is therefore independent
of the profile under test: a profile on another basis misses it by the
nutation in longitude.

Three reference skies, chosen by flag; the committed ``limbs.json`` is the
first's:

``--de440 FILE --swiss-data DIR``
    JPL's DE440 (``de440.bsp``, from
    https://ssd.jpl.nasa.gov/ftp/eph/planets/bsp/de440.bsp, 119,799,808
    bytes, sha256 a4ce9bf9b3282becc9f4b2ac3cebe03a2ae7599981aabd7265fd8482fff7c4b5,
    DAF/SPK, JD 2287184.5 to 2688976.5) read by jplephem for the geometry,
    and the Swiss Ephemeris's reductions and ayanamshas over it (the class
    ``De440`` says how). Evidence rank 1 for the places. The kernel is
    never committed: fetch it, check the digest, and point FILE at it.

``--swiss DIR``
    Swiss Ephemeris (pyswisseph 2.10.03) over the ``.se1`` files in DIR,
    which are fitted to JPL's DE431. Evidence rank 3 (``CLEAN_ROOM.md``:
    reference values for a cross-check). The ayanamshas are Swiss's own
    ``SIDM_LAHIRI`` and ``SIDM_KRISHNAMURTI``, the definitions the SDK's
    catalogue reproduces to 1e-7 arcseconds (``ACCURACY.md``). It also
    measures itself against JPL's DE440 at every instant of the corpus's
    CSPICE recording inside the years (``--states``), so the page can say
    how far the stand-in is from DE440 in seconds of each limb.

``--skyfield-de440 FILE``
    JPL's DE440 itself through Skyfield (``de440.bsp`` or ``de440s.bsp``,
    sha256 in ``fixtures/jpl/manifest.json``), IAU 2006 precession and
    IAU 2000A nutation, Lahiri as the Swiss definition carried by IAU 2006
    general precession (the research note's ``lahiri_check.py``). Written
    to the same layout so the Rust gate reads either. Evidence rank 1.

Run with ``python3 -I``; it imports nothing from the repository. It needs
pyswisseph 2.10.03 (the ``.se1`` files of DIR), and for ``--de440``
jplephem and numpy:

    python3 -I record_limbs.py --de440 de440.bsp --swiss-data DIR --out limbs.json
"""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import math
import sys

YEARS = (1900, 1950, 2000, 2050, 2100)
KATHMANDU = (27.7172, 85.324)
OFFSET_HOURS = 5.75
SEMIDIAMETER_AT_1AU_ARCSEC = 959.23  # IAU 2015 nominal radius, as the SDK's Disc
EARTH_EQUATORIAL_RADIUS_KM = 6378.137
AU_KM = 149_597_870.7
STANDARD_REFRACTION_DEG = 34.0 / 60.0
NAKSHATRA = 360.0 / 27.0


class Sky:
    """Apparent geocentric longitudes and the Sun's place, by backend."""

    name: str
    ayanamshas: tuple[str, ...]
    # Added to the backend's own Delta T where a sunrise reads the Sun's
    # place, to measure how a sunrise moves with Delta T.
    extra_delta_t_days: float = 0.0

    def lon(self, body: str, jd_tt: float) -> float:
        raise NotImplementedError

    def true_ayanamsha(self, member: str, jd_tt: float) -> float:
        raise NotImplementedError

    def sun_altitude_geocentric(self, jd_ut1: float, lat: float, lng: float) -> tuple[float, float]:
        """The Sun's geocentric apparent altitude, degrees, and distance, AU."""
        raise NotImplementedError


class Swiss(Sky):
    def __init__(self, data: str) -> None:
        import swisseph as swe  # type: ignore[import-not-found]

        self.swe = swe
        swe.set_ephe_path(data)
        self.flag = swe.FLG_SWIEPH
        self.name = f"Swiss Ephemeris {swe.version} over its .se1 files (fitted to DE431)"
        self.ayanamshas = ("LAHIRI", "KRISHNAMURTI")
        self.modes = {"LAHIRI": swe.SIDM_LAHIRI, "KRISHNAMURTI": swe.SIDM_KRISHNAMURTI}
        self.bodies = {"sun": swe.SUN, "moon": swe.MOON}

    def lon(self, body: str, jd_tt: float) -> float:
        xx, ret = self.swe.calc(jd_tt, self.bodies[body], self.flag)[:2]
        if ret & 7 != self.swe.FLG_SWIEPH:
            raise SystemExit(f"not the .se1 files at {jd_tt}: flags {ret}")
        return xx[0]

    def true_ayanamsha(self, member: str, jd_tt: float) -> float:
        self.swe.set_sid_mode(self.modes[member], 0, 0)
        # Without NONUT the value carries the nutation in longitude: true.
        return self.swe.get_ayanamsa_ex(jd_tt, self.flag)[1]

    def sun_altitude_geocentric(self, jd_ut1: float, lat: float, lng: float) -> tuple[float, float]:
        swe = self.swe
        jd_tt = jd_ut1 + swe.deltat(jd_ut1) + self.extra_delta_t_days
        xx = swe.calc(jd_tt, swe.SUN, self.flag | swe.FLG_EQUATORIAL)[0]
        ra, dec, dist = math.radians(xx[0]), math.radians(xx[1]), xx[2]
        gast = self.gast_deg(jd_ut1, jd_tt)
        h = math.radians(gast + lng) - ra
        phi = math.radians(lat)
        alt = math.asin(math.sin(phi) * math.sin(dec) + math.cos(phi) * math.cos(dec) * math.cos(h))
        return math.degrees(alt), dist

    def gast_deg(self, jd_ut1: float, jd_tt: float) -> float:
        """Greenwich apparent sidereal time, degrees: IAU 2006's GMST (the
        Earth rotation angle and Capitaine's polynomial) plus the equation
        of the equinoxes, Swiss's nutation in longitude times the cosine of
        the true obliquity.

        Not ``swe.sidtime``: from 2050 (JD 2469807.5) on, Swiss 2.10 turns
        to a long-term formula that sits 1.8 to 1.9 arcseconds from IAU 2006,
        0.12 s of a sunrise, while agreeing with it to 0.003 arcseconds over
        1850 to 2050."""
        t = (jd_tt - 2451545.0) / 36525.0
        du = jd_ut1 - 2451545.0
        era = 360.0 * ((0.7790572732640 + 0.00273781191135448 * du + du) % 1.0)
        poly = (0.014506 + 4612.156534 * t + 1.3915817 * t * t - 0.00000044 * t**3
                - 0.000029956 * t**4 - 0.0000000368 * t**5)
        nut = self.swe.calc(jd_tt, self.swe.ECL_NUT)[0]
        equation_of_equinoxes = nut[2] * math.cos(math.radians(nut[0]))
        return (era + poly / 3600.0 + equation_of_equinoxes) % 360.0

    def delta_t_days(self, jd_ut1: float) -> float:
        return self.swe.deltat(jd_ut1)


class De440(Swiss):
    """JPL's DE440 for the geometry, through jplephem, and the Swiss
    Ephemeris's reductions over it.

    DE440 gives barycentric positions and nothing else: an apparent place of
    date also needs light time, deflection, aberration, precession and
    nutation, and a sunrise the sidereal time. Those are the Swiss
    Ephemeris's (IAU 2006 precession, its nutation, its sidereal time), as
    the ayanamshas are its own ``SIDM_LAHIRI`` and ``SIDM_KRISHNAMURTI``.
    Each place is the Swiss place moved by the difference of the two
    astrometric directions (geocentric, light time applied, ICRS): DE440's
    less the ``.se1`` files'. The difference is a few thousandths of an
    arcsecond, so the rotation of date it rides on changes it by under a
    micro-arcsecond, and the place is DE440's to that.
    """

    C_KM_PER_DAY = 299_792.458 * 86_400.0
    J2000_OBLIQUITY = math.radians(84381.406 / 3600.0)

    def __init__(self, data: str, kernel: str) -> None:
        import jplephem  # type: ignore[import-not-found]
        from jplephem.spk import SPK  # type: ignore[import-not-found]

        super().__init__(data)
        self.kernel = SPK.open(kernel)
        with open(kernel, "rb") as f:
            digest = hashlib.sha256(f.read()).hexdigest()
        self.name = (
            f"JPL DE440 ({kernel.rsplit('/', 1)[-1]}, sha256 {digest}) through jplephem "
            f"{getattr(jplephem, '__version__', '?')} for the geometry; the reductions and the "
            f"ayanamshas of the Swiss Ephemeris {self.swe.version}"
        )
        self.worst_arcsec = {"sun": 0.0, "moon": 0.0}

    @staticmethod
    def _tdb(jd_tt: float) -> float:
        """TDB from TT, the leading periodic term (Fairhead and Bretagnon)."""
        g = math.radians(357.53 + 0.98560028 * (jd_tt - 2451545.0))
        return jd_tt + (0.001657 * math.sin(g) + 0.000014 * math.sin(2 * g)) / 86_400.0

    def _seg(self, centre: int, target: int, jd: float):
        return self.kernel[centre, target].compute(jd)

    def _earth(self, jd: float):
        return self._seg(0, 3, jd) + self._seg(3, 399, jd)

    def _body(self, body: str, jd: float):
        if body == "sun":
            return self._seg(0, 10, jd)
        return self._seg(0, 3, jd) + self._seg(3, 301, jd)

    def _astrometric_de440(self, body: str, jd_tt: float) -> list[float]:
        jd = self._tdb(jd_tt)
        earth = self._earth(jd)
        tau = 0.0
        for _ in range(4):
            r = self._body(body, jd - tau) - earth
            tau = math.sqrt(float(r @ r)) / self.C_KM_PER_DAY
        return [float(x) for x in r]

    def _astrometric_swiss(self, body: str, jd_tt: float) -> list[float]:
        swe = self.swe
        xx = swe.calc(jd_tt, self.bodies[body], self.flag | swe.FLG_J2000 | swe.FLG_ICRS
                         | swe.FLG_NONUT | swe.FLG_NOABERR | swe.FLG_NOGDEFL
                         | swe.FLG_EQUATORIAL | swe.FLG_XYZ)[0]
        return list(xx[:3])

    def _shift(self, body: str, jd_tt: float) -> tuple[float, float, float]:
        """DE440's direction less the files', as (ecliptic longitude, right
        ascension, declination) of J2000, degrees."""
        def angles(v: list[float]) -> tuple[float, float, float]:
            x, y, z = v
            e = self.J2000_OBLIQUITY
            ye, ze = y * math.cos(e) + z * math.sin(e), -y * math.sin(e) + z * math.cos(e)
            lon = math.degrees(math.atan2(ye, x))
            ra = math.degrees(math.atan2(y, x))
            dec = math.degrees(math.atan2(z, math.hypot(x, y)))
            del ze
            return lon, ra, dec
        a = angles(self._astrometric_de440(body, jd_tt))
        b = angles(self._astrometric_swiss(body, jd_tt))
        d = tuple(((p - q + 180.0) % 360.0) - 180.0 for p, q in zip(a, b))
        sep = math.hypot(d[1] * math.cos(math.radians(a[2])), d[2]) * 3600.0
        self.worst_arcsec[body] = max(self.worst_arcsec[body], sep)
        return d  # type: ignore[return-value]

    def lon(self, body: str, jd_tt: float) -> float:
        return super().lon(body, jd_tt) + self._shift(body, jd_tt)[0]

    def sun_altitude_geocentric(self, jd_ut1: float, lat: float, lng: float) -> tuple[float, float]:
        swe = self.swe
        jd_tt = jd_ut1 + swe.deltat(jd_ut1)
        jd_tt += self.extra_delta_t_days
        _, d_ra, d_dec = self._shift("sun", jd_tt)
        xx = swe.calc(jd_tt, swe.SUN, self.flag | swe.FLG_EQUATORIAL)[0]
        ra, dec = math.radians(xx[0] + d_ra), math.radians(xx[1] + d_dec)
        dist = math.sqrt(sum(x * x for x in self._astrometric_de440("sun", jd_tt))) / AU_KM
        gast = self.gast_deg(jd_ut1, jd_tt)
        h = math.radians(gast + lng) - ra
        phi = math.radians(lat)
        alt = math.asin(math.sin(phi) * math.sin(dec) + math.cos(phi) * math.cos(dec) * math.cos(h))
        return math.degrees(alt), dist


class SkyfieldDe440(Sky):
    """DE440 through Skyfield, as the consumer's research note (A.12) ran it."""

    T0_JD, A0 = 2435553.5, 23.250182778 - 0.004658035

    def __init__(self, kernel: str) -> None:
        from skyfield.api import load, load_file  # type: ignore[import-not-found]
        from skyfield.framelib import ecliptic_frame  # type: ignore[import-not-found]
        from skyfield.nutationlib import iau2000a_radians  # type: ignore[import-not-found]

        self.ts = load.timescale(builtin=True)
        self.eph = load_file(kernel)
        self.frame = ecliptic_frame
        self.nut = iau2000a_radians
        self.earth = self.eph["earth"]
        self.targets = {"sun": self.eph["sun"], "moon": self.eph["moon"]}
        with open(kernel, "rb") as f:
            digest = hashlib.sha256(f.read()).hexdigest()
        self.name = f"JPL DE440 through Skyfield ({kernel.rsplit('/', 1)[-1]}, sha256 {digest})"
        self.ayanamshas = ("LAHIRI",)

    @staticmethod
    def _p_a(jd_tt: float) -> float:
        t = (jd_tt - 2451545.0) / 36525
        return (5028.796195 * t + 1.1054348 * t**2 + 0.00007964 * t**3
                - 0.000023857 * t**4 - 0.0000000383 * t**5) / 3600

    def lon(self, body: str, jd_tt: float) -> float:
        t = self.ts.tt_jd(jd_tt)
        return self.earth.at(t).observe(self.targets[body]).apparent().frame_latlon(self.frame)[1].degrees

    def true_ayanamsha(self, member: str, jd_tt: float) -> float:
        t = self.ts.tt_jd(jd_tt)
        mean = self.A0 + self._p_a(jd_tt) - self._p_a(self.T0_JD)
        return mean + math.degrees(self.nut(t)[0])

    def sun_altitude_geocentric(self, jd_ut1: float, lat: float, lng: float) -> tuple[float, float]:
        t = self.ts.ut1_jd(jd_ut1)
        ra, dec, dist = self.earth.at(t).observe(self.targets["sun"]).apparent().radec(epoch="date")
        h = math.radians(t.gast * 15.0 + lng) - ra.radians
        phi = math.radians(lat)
        d = dec.radians
        alt = math.asin(math.sin(phi) * math.sin(d) + math.cos(phi) * math.cos(d) * math.cos(h))
        return math.degrees(alt), dist.au

    def delta_t_days(self, jd_ut1: float) -> float:
        t = self.ts.ut1_jd(jd_ut1)
        return t.tt - jd_ut1


def unwrap(prev: float, value: float) -> float:
    """value moved by whole turns to within half a turn of prev."""
    return value + 360.0 * round((prev - value) / 360.0)


def crossings(f, start: float, end: float, step_deg: float, sample_days: float) -> list[float]:
    """Every instant in [start, end) at which the increasing angle f crosses a multiple of step_deg."""
    out: list[float] = []
    t0, v0 = start, f(start)
    while t0 < end:
        t1 = min(t0 + sample_days, end)
        v1 = unwrap(v0, f(t1))
        k0, k1 = math.floor(v0 / step_deg), math.floor(v1 / step_deg)
        for k in range(k0 + 1, k1 + 1):
            target = k * step_deg
            out.append(solve(f, t0, t1, v0, v1, target))
        t0, v0 = t1, v1
    return out


def solve(f, a: float, b: float, va: float, vb: float, target: float) -> float:
    """f(t) = target (mod 360) in [a, b], by the secant and bisection, to 1e-9 day."""
    for _ in range(100):
        t = a + (target - va) * (b - a) / (vb - va)
        if not a < t < b:
            t = 0.5 * (a + b)
        v = unwrap(va, f(t))
        if v < target:
            a, va = t, v
        else:
            b, vb = t, v
        if b - a < 1e-9 or abs(v - target) < 1e-11:
            return t
    raise SystemExit("no convergence")


def jd_of(year: int, month: int = 1, day: int = 1) -> float:
    """The Julian day of a Gregorian date's 0h."""
    return dt.date(year, month, day).toordinal() + 1721424.5


DELTA_T_STEP_SECONDS = 10.0


def rise(sky: Sky, local_midnight: float, limb: float) -> float:
    """The sunrise after a local midnight, UT1, by the secant and bisection."""
    lat, lng = KATHMANDU

    def g(t: float) -> float:
        alt, dist = sky.sun_altitude_geocentric(t, lat, lng)
        sd = SEMIDIAMETER_AT_1AU_ARCSEC / 3600.0 / dist
        parallax = math.degrees(math.asin(EARTH_EQUATORIAL_RADIUS_KM / (dist * AU_KM)))
        return alt - (-STANDARD_REFRACTION_DEG - limb * sd + parallax)

    a, b = local_midnight + 2.0 / 24.0, local_midnight + 10.0 / 24.0
    ga, gb = g(a), g(b)
    assert ga < 0 < gb, (local_midnight, limb)
    m = a
    for _ in range(200):
        m = a + ga * (a - b) / (gb - ga)
        if not a < m < b:
            m = 0.5 * (a + b)
        gm = g(m)
        if gm < 0:
            a, ga = m, gm
        else:
            b, gb = m, gm
        if b - a < 1e-10 or abs(gm) < 1e-10:
            break
    return m


def sunrises(sky: Sky, year: int) -> dict[str, list[list[float]]]:
    """Each local date's sunrise at Kathmandu under the two conventions, UT1.

    A row is the date's index from 1 January, the sunrise, the Delta T it
    was found under (seconds), and how many seconds the sunrise moves per
    second of Delta T: a sunrise is an instant of the Earth's rotation, so
    a reader on another Delta T model moves the Sun's place under it, and
    the row lets it put the reference on its own model to first order.
    """
    out: dict[str, list[list[float]]] = {"centre_refraction": [], "upper_limb_refraction": []}
    first, last = jd_of(year), jd_of(year + 1)
    day = first
    while day < last:
        local_midnight = day - OFFSET_HOURS / 24.0
        for key, limb in (("centre_refraction", 0.0), ("upper_limb_refraction", 1.0)):
            at = rise(sky, local_midnight, limb)
            delta_t = sky.delta_t_days(at) * 86_400.0
            sky.extra_delta_t_days = DELTA_T_STEP_SECONDS / 86_400.0
            moved = rise(sky, local_midnight, limb)
            sky.extra_delta_t_days = 0.0
            rate = (moved - at) * 86_400.0 / DELTA_T_STEP_SECONDS
            out[key].append([round(day - first), at, round(delta_t, 6), round(rate, 9)])
        day += 1.0
    return out


def record(sky: Sky, years: tuple[int, ...]) -> dict:
    data: dict = {
        "schema": "teistro/limb-oracle/1",
        "source": sky.name,
        "recorded": dt.date.today().isoformat(),
        "recorder": "crates/sdk/tests/oracle/record_limbs.py",
        "definitions": {
            "tithi": "apparent geocentric Moon less Sun, true equinox of date, at a multiple of 12 degrees; TT",
            "karana": "the same at a multiple of 6 degrees; TT",
            "nakshatra": "the Moon's apparent longitude less the true ayanamsha at a multiple of 13 deg 20 min; TT",
            "yoga": "the Moon's and the Sun's apparent longitudes less twice the true ayanamsha at a multiple of 13 deg 20 min; TT",
            "sankranti": "the Sun's apparent longitude less the true ayanamsha at a multiple of 30 degrees; TT",
            "sunrise": "the geocentric altitude of the Sun's centre at -34' + horizontal parallax (centre_refraction) or less the semidiameter of 959.23\" at 1 AU too (upper_limb_refraction), apparent places of the true equator and equinox, Greenwich apparent sidereal time; rows of [day index from 1 January local, UT1, the Delta T in seconds the Sun's place was read under, seconds of sunrise per second of Delta T]",
        },
        "place": {"latitude": KATHMANDU[0], "longitude": KATHMANDU[1], "altitude_m": 0.0, "utc_offset_hours": OFFSET_HOURS},
        "years": {},
    }
    for year in years:
        start, end = jd_of(year), jd_of(year + 1)
        moon = lambda t: sky.lon("moon", t)  # noqa: E731
        sun = lambda t: sky.lon("sun", t)  # noqa: E731
        elong = lambda t: (moon(t) - sun(t)) % 360.0  # noqa: E731
        entry: dict = {
            "tithi": crossings(elong, start, end, 12.0, 0.25),
            "karana": crossings(elong, start, end, 6.0, 0.125),
            "by_ayanamsha": {},
        }
        for member in sky.ayanamshas:
            ay = lambda t, m=member: sky.true_ayanamsha(m, t)  # noqa: E731
            entry["by_ayanamsha"][member] = {
                "nakshatra": crossings(lambda t: (moon(t) - ay(t)) % 360.0, start, end, NAKSHATRA, 0.25),
                "yoga": crossings(lambda t: (moon(t) + sun(t) - 2.0 * ay(t)) % 360.0, start, end, NAKSHATRA, 0.25),
                "sankranti": crossings(lambda t: (sun(t) - ay(t)) % 360.0, start, end, 30.0, 2.0),
            }
        entry["sunrise"] = sunrises(sky, year)
        data["years"][str(year)] = entry
        print(year, {k: len(v) for k, v in entry.items() if isinstance(v, list)}, file=sys.stderr)
    return data


def bridge(sky: Swiss, states_path: str, years: tuple[int, ...]) -> dict:
    """The stand-in's geometric Sun and Moon against DE440's at the corpus's instants."""
    swe = sky.swe
    states = json.load(open(states_path))
    lo, hi = jd_of(min(years)) - 3653, jd_of(max(years) + 1) + 3653
    out: dict = {}
    for target, body, name in (("10", swe.SUN, "sun"), ("301", swe.MOON, "moon")):
        series = next(s for s in states["series"] if s["target"] == target and s["kernel"] == "de440"
                      and s["frame"] == "J2000" and s["correction"] == "NONE" and s["epoch"] == "tdb")
        worst, n = 0.0, 0
        for instant, row in zip(states["instants"], series["rows"]):
            jd = instant["jd"]
            if not lo <= jd <= hi:
                continue
            # TDB read as TT: they differ by under 2 ms, a micro-arcsecond of the Moon.
            xx = swe.calc(jd, body, sky.flag | swe.FLG_J2000 | swe.FLG_ICRS | swe.FLG_NONUT
                             | swe.FLG_TRUEPOS | swe.FLG_EQUATORIAL | swe.FLG_XYZ)[0]
            a, b = xx[:3], row[:3]
            na, nb = math.sqrt(sum(x * x for x in a)), math.sqrt(sum(x * x for x in b))
            cos = sum(x * y for x, y in zip(a, b)) / na / nb
            sep = math.degrees(math.acos(min(1.0, cos))) * 3600.0
            worst, n = max(worst, sep), n + 1
        out[name] = {"worst_arcsec": round(worst, 5), "instants": n}
    out["span"] = f"the corpus's CSPICE instants from JD {lo} to {hi}"
    return out


def main() -> None:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    g = p.add_mutually_exclusive_group(required=True)
    g.add_argument("--swiss", metavar="DIR")
    g.add_argument("--skyfield-de440", metavar="FILE")
    g.add_argument("--de440", metavar="FILE", help="de440.bsp, read by jplephem; needs --swiss-data for the reductions")
    p.add_argument("--swiss-data", metavar="DIR", help="the .se1 files the --de440 reductions run over")
    p.add_argument("--states", metavar="FILE", help="fixtures/jpl/cspice/states.json, for the stand-in's distance from DE440")
    p.add_argument("--years", default=",".join(map(str, YEARS)))
    p.add_argument("--out", required=True)
    a = p.parse_args()
    years = tuple(int(y) for y in a.years.split(","))
    if a.de440:
        if not a.swiss_data:
            p.error("--de440 needs --swiss-data")
        sky: Sky = De440(a.swiss_data, a.de440)
    elif a.swiss:
        sky = Swiss(a.swiss)
    else:
        sky = SkyfieldDe440(a.skyfield_de440)
    data = record(sky, years)
    if isinstance(sky, De440):
        data["de440_shift"] = {
            "worst_arcsec": {k: round(v, 5) for k, v in sky.worst_arcsec.items()},
            "what": "the largest move from the .se1 files' astrometric place to DE440's over every place recorded",
        }
    if a.states and type(sky) is Swiss:
        data["against_de440"] = bridge(sky, a.states, years)
    with open(a.out, "w") as f:
        json.dump(data, f, separators=(",", ":"))
        f.write("\n")


if __name__ == "__main__":
    main()
