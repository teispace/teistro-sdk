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
    DashaSystem,
    Ephemeris,
    Latitude,
    Longitude,
    Observer,
    Saham,
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

        # ── And the lord of that year, with the reason ─────────────────
        lord = cast.annual.year_lord
        print(f"year lord {lord.graha.key} at {lord.vishwa}, chosen {lord.chosen.key}")
        for claim in lord.claims:
            aspects = "aspects" if claim.aspects_lagna else "does not aspect"
            print(
                f"  {claim.graha.key:<8} {claim.vishwa}  "
                f"{claim.portfolios} portfolio(s)  {aspects} the lagna"
            )

        # ── The sixteen Tajika yogas answer a matter, not a chart ────
        # Fourteen of them judge the lagnesha against the lord of the house
        # you ask about, so you name the houses: marriage (7) and career
        # (10) here.
        judged = ctx.chart.found(
            instant=when.instant_jd_utc,
            place=place,
            utc_offset_seconds=when.offset_seconds,
            varsha={"through": 30, "place": "birth", "matters": [7, 10]},
        ).praveshas[29].annual
        assert judged is not None
        for matter in judged.matters:
            held = ", ".join(one.yoga.key for one in matter.held) or "none"
            print(
                f"house {matter.house}: {matter.lagnesha.key} with {matter.karyesha.key}, "
                f"held {held}; not answered {', '.join(y.key for y in matter.unanswered)}"
            )

        # ── The sahams: forty-one sensitive points, each a − b + c ───
        # Name the ones you want, or "all"; each comes back with its sign,
        # that sign's lord and the house it fell in, as the source reads
        # them.
        points = ctx.chart.found(
            instant=when.instant_jd_utc,
            place=place,
            utc_offset_seconds=when.offset_seconds,
            varsha={"through": 30, "place": "birth", "sahams": [Saham.PUNYA, Saham.VIVAHA, Saham.KARYA_SIDDHI]},
        ).praveshas[29].annual
        assert points is not None
        for point in points.sahams:
            added = " (a sign added)" if point.added_sign else ""
            print(
                f"{point.saham.key:<12} {point.longitude_deg:6.2f}°  {point.sign.key}, "
                f"lord {point.lord.key}, house {point.house}{added}"
            )
            # Its strength is the source's clauses, reported and never scored.
            strong = ", ".join(c.key for c in point.strong) or "none"
            weak = ", ".join(c.key for c in point.weak) or "none"
            print(f"  strong: {strong}; weak: {weak}")
        # And the seven's Harsha bala that year: four places each is happy in.
        print(", ".join(f"{h.graha.key} {h.total}" for h in points.harsha))

        # ── The annual dashas: the year divided among its lords ───────
        # The Mudda runs round the nine from the birth nakshatra's lord,
        # one lord further each year; the Patyayini is read from the year's
        # own chart, and its lagna's share is a sign's. The Sun is read
        # over the year once for both, and each year closes on the next
        # return.
        divided = ctx.chart.found(
            instant=when.instant_jd_utc,
            place=place,
            utc_offset_seconds=when.offset_seconds,
            varsha={"through": 30, "place": "birth", "dashas": [DashaSystem.MUDDA, DashaSystem.PATYAYINI]},
        ).praveshas[29].annual
        assert divided is not None
        for dasha in divided.dashas:
            days = ", ".join(
                f"{(p.sign or p.lord).key} {p.span.to_jd - p.span.from_jd:.1f}"
                for p in dasha.periods
                if p.level == 1
            )
            print(f"{dasha.system.key}: {days}")

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
