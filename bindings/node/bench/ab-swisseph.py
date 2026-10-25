"""The Swiss Ephemeris rung of the A/B's accuracy (`docs/03-design/ab-baseline.md` §11).

Reads a JSON array of skies on stdin, each an instant, a place and a
convention, and writes the same array's places on stdout: the nine grahas'
sidereal longitudes, the lagna and the twelve cusps, in degrees. It needs
pyswisseph 2.10.03 and the ephemeris files it reads, named by
`SE_EPHE_PATH`; it refuses to answer from the analytic fallback.

A convention is
  ayanamsha  LAHIRI, KRISHNAMURTI or RAMAN
  basis      "true" (the ayanamsha with nutation, Swiss Ephemeris's own
             sidereal place) or "mean" (the apparent tropical place less the
             mean ayanamsha)
  centre     "geocentric" or "topocentric"
  node       "mean" or "true"
  houses     "W" (whole signs) or "P" (Placidus), Swiss Ephemeris's letters
"""

import json
import os
import sys

import swisseph as swe

GRAHAS = [
    ("SUN", swe.SUN),
    ("MOON", swe.MOON),
    ("MARS", swe.MARS),
    ("MERCURY", swe.MERCURY),
    ("JUPITER", swe.JUPITER),
    ("VENUS", swe.VENUS),
    ("SATURN", swe.SATURN),
]
AYANAMSHAS = {"LAHIRI": swe.SIDM_LAHIRI, "KRISHNAMURTI": swe.SIDM_KRISHNAMURTI, "RAMAN": swe.SIDM_RAMAN}


def place(sky):
    swe.set_sid_mode(AYANAMSHAS[sky["ayanamsha"]])
    flags = swe.FLG_SWIEPH | swe.FLG_SPEED
    if sky["centre"] == "topocentric":
        swe.set_topo(sky["lon"], sky["lat"], sky["alt"])
        flags |= swe.FLG_TOPOCTR
    jd = sky["jd"]
    if sky["basis"] == "true":
        sidereal = flags | swe.FLG_SIDEREAL
        offset = 0.0
    else:
        sidereal = flags
        offset = swe.get_ayanamsa_ex_ut(jd, swe.FLG_SWIEPH | swe.FLG_NONUT)[1]

    def longitude(body):
        values, ret = swe.calc_ut(jd, body, sidereal)
        if ret & swe.FLG_MOSEPH or not ret & swe.FLG_SWIEPH:
            sys.exit(f"Swiss Ephemeris answered without its files for body {body}; set SE_EPHE_PATH")
        return (values[0] - offset) % 360.0

    out = {name: longitude(body) for name, body in GRAHAS}
    out["RAHU"] = longitude(swe.MEAN_NODE if sky["node"] == "mean" else swe.TRUE_NODE)
    out["KETU"] = (out["RAHU"] + 180.0) % 360.0
    # Houses take no topocentric flag: the angles are the place's either way.
    house_flags = swe.FLG_SIDEREAL if sky["basis"] == "true" else 0
    # Placidus has no cusps inside a polar circle, and the rung then
    # compares none.
    try:
        cusps, ascmc = swe.houses_ex(jd, sky["lat"], sky["lon"], sky["houses"].encode(), house_flags)
    except swe.Error:
        cusps, ascmc = None, swe.houses_ex(jd, sky["lat"], sky["lon"], b"W", house_flags)[1]
    return {
        "grahas": out,
        "lagna": (ascmc[0] - offset) % 360.0,
        "cusps": None if cusps is None else [(c - offset) % 360.0 for c in cusps[:12]],
    }


def main():
    if not os.environ.get("SE_EPHE_PATH"):
        sys.exit("set SE_EPHE_PATH to the Swiss Ephemeris files")
    swe.set_ephe_path(os.environ["SE_EPHE_PATH"])
    json.dump([place(sky) for sky in json.load(sys.stdin)], sys.stdout)


if __name__ == "__main__":
    main()
