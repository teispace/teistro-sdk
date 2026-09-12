"""Rectification: a birth time known only to the hour, narrowed by lagna.

A birth record that says "some time before dawn" is the commonest hard
case in the field. What narrows it is the **lagna** — it moves through
all twelve signs in a day, so it changes sign every couple of hours, and
a family that remembers the ascendant remembers something the clock does
not.

The point of this example is the shape of the call. A rectification pass
wants many charts at one place, and ``found_many`` founds them in **one
crossing**: the settings are resolved once, the solar model is built
once, and the day each instant belongs to is reckoned against the same
sunrise. Founding them one at a time would give the same numbers and pay
the setup for every one of them.

``Ephemeris.BUILTIN`` selects the analytic ephemeris the SDK carries, so
this file runs anywhere.
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
    at,
    date,
    iana_zone,
)
from teistro.catalogue import Graha, Rashi

FROM_HOUR = 0
TO_HOUR = 3
EVERY_MINUTES = 10


def main() -> None:
    teistro = Teistro.open()
    # `nepali-default` is what a Nepali birth record is cast under, and
    # its frame is **topocentric**: the chart is seen from the hill the
    # record was written on rather than from the centre of the Earth. The
    # completion does that step itself over any provider
    # (`03-design/topocentric-measured.md`), so the analytic one below is
    # enough, and the steps printed at the end name it.
    with teistro.context(
        profile="nepali-default", locale="ne-Deva-NP", ephemeris=Ephemeris.BUILTIN
    ) as ctx:
        # The record: a Bikram Sambat date, a place, and an hour nobody
        # is sure of. Everything below narrows the last of those.
        born = date(Calendar.BIKRAM_SAMBAT, 2045, 9, 17)
        place = Observer(
            latitude_deg=Latitude(27.7172),
            longitude_deg=Longitude(85.324),
            altitude_m=Altitude(1400),
        )

        # One resolution fixes the zone and the offset; the candidates
        # are then arithmetic on the instant, which is what a Julian day
        # is for.
        start = ctx.time.resolve(at(born, hour=FROM_HOUR), iana_zone("Asia/Kathmandu"))
        step = EVERY_MINUTES / (24 * 60)
        count = (TO_HOUR - FROM_HOUR) * 60 // EVERY_MINUTES
        instants = [start.instant_jd_utc + i * step for i in range(count)]

        # ── One crossing for every candidate ──────────────────────────
        charts = ctx.chart.found_many(
            instants=instants,
            place=place,
            utc_offset_seconds=start.offset_seconds,
        )

        print(
            f"{len(charts)} candidate charts, {EVERY_MINUTES} minutes apart, "
            f"in one crossing"
        )
        print(
            f"place  {place.latitude_deg}°N {place.longitude_deg}°E   "
            f"{charts.kind.key}"
        )
        print("")
        print("local   lagna        sign            moon         bhava")
        print("─" * 58)

        previous: int | None = None
        for chart in charts:
            sign = int(chart.lagna_deg // 30)
            moon = next(g for g in chart.grahas if g.graha is Graha.MOON)
            rashi = ctx.intl.entity(Rashi(sign).full_key)
            minutes = FROM_HOUR * 60 + chart.index * EVERY_MINUTES
            changed = "   ← lagna changes sign" if previous not in (None, sign) else ""
            print(
                f"{minutes // 60:02}:{minutes % 60:02}   "
                f"{chart.lagna_deg:9.4f}°  "
                f"{rashi.name:<14} "
                f"{moon.longitude_deg:9.4f}°  "
                f"{moon.house.bhava:>2}{changed}"
            )
            previous = sign

        # ── What the batch shares, and what it does not ───────────────
        # The place, the settings, the solar model and the completion
        # steps are one to a batch: they are what "the same chart at a
        # different minute" holds constant. The instant, the lagna, the
        # day and the timing are per chart. The provenance envelope
        # stamps the batch as a whole, so a rectification run reproduces
        # as one thing.
        print("")
        print(f"model          {charts.model}")
        print(f"steps applied  {', '.join(charts.steps_applied)}")
        print(f"settings hash  {ctx.settings_hash[:16]}…")

        # A batch of one is the ordinary case, and `found` is the same
        # crossing with the batch unwrapped: the answer is a chart, not a
        # list of one.
        single = ctx.chart.found(
            instant=start.instant_jd_utc,
            place=place,
            utc_offset_seconds=start.offset_seconds,
        )
        print("")
        print(
            f"found(one)     lagna {single.lagna_deg:.4f}°  "
            f"vara {single.vara.key}  "
            f"hora lord {single.hora_lord.key}"
        )


if __name__ == "__main__":
    main()
