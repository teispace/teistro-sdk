"""A week's panchangam: the five limbs of each day, and its periods.

A chart is consulted once; a panchanga every morning. This is the page a
Nepali or Indian almanac prints, and the SDK computes it in **one
crossing** for the whole week — consecutive days share a boundary, so day
n's next sunrise is day n+1's sunrise, and asking for seven days costs
much less than seven days asked for separately.

What the shape teaches, and what a reader should copy:

  * A limb is a **span**, not a name. "Today's tithi" is a question with
    two answers on most days, and the SDK gives both with the instant
    each gives way — which is what an almanac row prints.
  * A span carries its **own** bounds as well as the clipped ones, so
    "the tithi began yesterday at 21:05" is a fact you can print.
  * A value a day may not have is **absent**, never a sentinel: no
    sankranti is `None`, not Julian day zero.

``Ephemeris.BUILTIN`` selects the analytic ephemeris the SDK carries, so
this file runs anywhere.
"""

from __future__ import annotations

from typing import Any

from teistro import (
    Altitude,
    Calendar,
    Ephemeris,
    Latitude,
    Longitude,
    Observer,
    Teistro,
    TeistroError,
    date,
)

OFFSET_SECONDS = 20700


def clock(jd: float) -> str:
    """A Julian day as the local clock reads it, which is what an almanac
    prints; the offset is the one the request was made under."""
    local = (jd + OFFSET_SECONDS / 86400 + 0.5) % 1
    minutes = round(local * 1440) % 1440
    return f"{minutes // 60:02}:{minutes % 60:02}"


def main() -> None:
    teistro = Teistro.open()
    with teistro.context(
        profile="parashari-classical", locale="ne-Deva-NP", ephemeris=Ephemeris.BUILTIN
    ) as ctx:

        def name(key: str) -> str:
            """The entity's name, or the key when the pack has none.

            A locale pack names most of the catalogue and not all of it:
            `masa` and `direction` have no entries in any of the five the
            SDK ships, so an almanac falls back rather than refusing to
            print.
            """
            try:
                return ctx.intl.entity(key).name
            except TeistroError:
                return key.split(".", 1)[1].lower().replace("_", " ")

        place = Observer(
            latitude_deg=Latitude(27.7172),
            longitude_deg=Longitude(85.324),
            altitude_m=Altitude(1400),
        )

        # ── One crossing for the whole week ───────────────────────────
        week = ctx.almanac.of(
            from_date=date(Calendar.GREGORIAN, 2024, 6, 17),
            to_date=date(Calendar.GREGORIAN, 2024, 6, 23),
            place=place,
            utc_offset_seconds=OFFSET_SECONDS,
        )

        print(
            f"{len(week)} days at {place.latitude_deg}°N "
            f"{place.longitude_deg}°E, one crossing"
        )
        print(f"calendar {week.calendar.key}   model {week.model.split(',')[0]}")
        print("")

        columns = week.decoded.day
        for day in week:
            i = day.index
            print(
                f"{name(day.vara.full_key):<12} "
                f"{columns.year[i]}-{columns.month[i]:02}-{columns.day_of_month[i]:02}"
                f"   sunrise {clock(day.sunrise)}  sunset {clock(day.sunset)}"
                f"   {name(day.month.amanta.full_key)} {name(day.month.paksha.full_key)}"
            )

            # The five limbs. The vara is one of them and is the day's
            # own; the other four are spans, and a day usually has two of
            # each.
            def limb(label: str, spans: list[Any]) -> None:
                printed = []
                for span in spans:
                    # `whole` is the member's own span and `inside` the
                    # clipped one, so a member that began yesterday says
                    # so rather than looking as though it began at
                    # sunrise.
                    began = "‹" if span.whole.from_jd < span.inside.from_jd else " "
                    ends = "›" if span.whole.to_jd > span.inside.to_jd else " "
                    printed.append(
                        f"{began}{name(span.member.full_key)} "
                        f"until {clock(span.inside.to_jd)}{ends}"
                    )
                print(f"  {label:<10} {'  '.join(printed)}")

            limb("tithi", day.tithi)
            limb("nakshatra", day.nakshatra)
            limb("yoga", day.yoga)
            limb("karana", day.karana)

            # The periods a day is planned around. Rahu kalam is the one
            # everybody checks; the choghadiya are what a shop opens on.
            kaalas = "  ".join(
                f"{name(k.kaala.full_key)} {clock(k.at.from_jd)}–{clock(k.at.to_jd)}"
                for k in day.kaalas
            )
            print(f"  {'kaala':<10} {kaalas}")
            auspicious = [c for c in day.choghadiya if c.daytime][:3]
            shown = "  ".join(
                f"{name(c.choghadiya.full_key)} {clock(c.at.from_jd)}"
                for c in auspicious
            )
            print(f"  {'choghadiya':<10} {shown} …")
            if day.abhijit is not None:
                effective = "" if day.abhijit.effective else "  (not effective on a Wednesday)"
                print(
                    f"  {'abhijit':<10} {clock(day.abhijit.at.from_jd)}–"
                    f"{clock(day.abhijit.at.to_jd)}{effective}"
                )
            # Absent is absent: no sankranti is None, and a Moon that did
            # not rise inside the window contributes no event at all.
            if day.sankranti is not None:
                print(
                    f"  {'sankranti':<10} the Sun enters a new sign at "
                    f"{clock(day.sankranti)}"
                )
            moon = "  ".join(
                f"{'rise' if e.rise else 'set'} {clock(e.instant)}"
                for e in day.moon_events
            )
            print(f"  {'moon':<10} {moon or '(neither rise nor set inside the window)'}")
            print("")

        # ── What the ragged layout costs a reader, which is nothing ───
        # Each day's lists are slices of one concatenated column, found
        # by adding up every earlier day's count. The layer does that sum
        # once when the batch is built, so `day.karana` is a slice and
        # not a search.
        counted = sum(len(day.karana) for day in week)
        print(f"{counted} karanas across {len(week)} days, from one blob")
        print(f"settings hash  {ctx.settings_hash[:16]}…")

        # A day on its own is the range of one unwrapped: same crossing,
        # and the answer is a day rather than a list of one.
        one = ctx.almanac.day(
            date=date(Calendar.GREGORIAN, 2024, 6, 21),
            place=place,
            utc_offset_seconds=OFFSET_SECONDS,
        )
        print(
            f"almanac_day    {name(one.vara.full_key)}  {len(one.horas)} horas, "
            f"{len(one.muhurtas)} muhurtas, {len(one.choghadiya)} choghadiya"
        )


if __name__ == "__main__":
    main()
