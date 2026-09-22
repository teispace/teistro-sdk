"""The annual chart: the one instant every Tajika judgement is made from.

A birth chart is cast for a birth. An **annual** chart is cast for the
moment the Sun comes back to the longitude it held then — once a year,
about twenty minutes earlier than the clock would say, and never on the
birthday itself (`docs/03-design/annual-chart.md`).

What it teaches:

1. **The boundary answers the instant, not the chart.** Whether the annual
   chart is cast for the birthplace or for where you live now is a
   question the schools answer differently, so the SDK hands you the
   instant and you found the chart with the place you mean.
2. **Which longitude is a choice with a name.** `sidereal` is the
   tradition's; `tropical` is the Western solar return and is most of a
   circle of lagna away by the fortieth year; `mean` is the older
   arithmetic and needs no ephemeris at all. None of them is a fallback
   for another.
3. **Fewer than you asked for is the answer**, not a refusal: an ephemeris
   that ends before your hundredth year says so by giving you the years it
   has.

The record is `birth_chart.py`'s own, so the two can be read side by side.
"""

from __future__ import annotations

from teistro import (
    Altitude,
    Calendar,
    Ephemeris,
    Latitude,
    Longitude,
    Observer,
    Teistro,
    TeistroError,
    at,
    date,
    iana_zone,
)


def main() -> None:
    teistro = Teistro.open()
    with teistro.context(
        profile="nepali-default", ephemeris=Ephemeris.BUILTIN
    ) as ctx:
        birth_day = date(Calendar.GREGORIAN, 1990, 4, 14)
        when = ctx.time.resolve(
            at(birth_day, hour=5, minute=30), iana_zone("Asia/Kathmandu")
        )
        place = Observer(
            latitude_deg=Latitude(27.7172),
            longitude_deg=Longitude(85.324),
            altitude_m=Altitude(1400),
        )

        # ── The years a birth opens ──────────────────────────────────
        chart = ctx.chart.found(
            instant=when.instant_jd_utc,
            place=place,
            utc_offset_seconds=when.offset_seconds,
            varsha={"reading": "sidereal", "through": 40},
        )
        years = chart.praveshas
        print(f"returns computed: {len(years)}")
        thirtieth = next(one for one in years if one.year == 30)
        print(f"the thirtieth year opens at jd {thirtieth.instant:.6f}")

        # A return is about a sidereal year after the last, never a
        # calendar one.
        gaps = [b.instant - a.instant for a, b in zip(years, years[1:])]
        print(f"between returns: {min(gaps):.4f} to {max(gaps):.4f} days")

        # ── The chart of that year, cast where you choose ────────────
        annual = ctx.chart.found(
            instant=thirtieth.instant,
            place=place,
            utc_offset_seconds=when.offset_seconds,
        )
        print(
            f"natal lagna {chart.lagna_deg:.3f}°, "
            f"annual lagna {annual.lagna_deg:.3f}°"
        )

        # ── Its five office-bearers, cast where you say ───────────────
        # Here the birthplace, the one Tajika text read casts every chart
        # for; a residence is {"observer": Observer(...), ...} instead.
        cast = ctx.chart.found(
            instant=when.instant_jd_utc,
            place=place,
            utc_offset_seconds=when.offset_seconds,
            varsha={"through": 30, "place": "birth"},
        ).praveshas[29]
        assert cast.annual is not None
        b = cast.annual.office_bearers
        five = " ".join(
            lord.key for lord in (b.muntha, b.janma_lagna, b.varsha_lagna, b.tri_rashi, b.dina_ratri)
        )
        part = "by day" if cast.annual.by_day else "by night"
        print(f"muntha in {cast.muntha.sign.key}; office-bearers {five}, {part}")

        # ── The readings are named, and they are not each other ──────
        for reading in ("sidereal", "tropical", "mean"):
            one = ctx.chart.found(
                instant=when.instant_jd_utc,
                place=place,
                utc_offset_seconds=when.offset_seconds,
                varsha={"reading": reading, "through": 30},
            ).praveshas
            apart = (one[29].instant - years[29].instant) * 24
            print(f"{reading:<9} thirtieth year, {apart:.2f} hours from the sidereal one")

        # ── What it refuses, and by which field ──────────────────────
        try:
            ctx.chart.found(
                instant=when.instant_jd_utc,
                place=place,
                utc_offset_seconds=when.offset_seconds,
                varsha={"reading": "sidereal", "through": 0},
            )
        except TeistroError as error:
            print(f"refused  {error.field}: {error.message}")


if __name__ == "__main__":
    main()
