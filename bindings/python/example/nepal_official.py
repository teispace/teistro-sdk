"""Nepal's official days, and the modern reckoning beside them.

Nepal's patro is computed by the Nepali committee from the Surya
Siddhanta: the tithi, nakshatra and yoga by the text with the committee's
bija, and the sunrise from a modern ephemeris. That is what
``nepal_official`` opens. The modern reckoning, ``nepali-drik``, reads
the same day from the modern sky, and the two name a different sunrise
tithi on a few days a month.

What the shape teaches, and what a reader should copy:

* The official day is a **preset**, not a recipe: unpack
  ``nepal_official(engine)`` into a context's arguments.
* The Surya Siddhanta takes the engine **after** it in the chain for its
  sunrise; an app passes its own, ``nepal_official(teimeris())``. This
  file passes none, so the sunrise is the built-in ephemeris's and it runs
  anywhere.
* Where the two disagree, the reason is a tithi's end against the
  sunrise: the tithi the modern sky still has at sunrise had already ended
  by the text's, or the other way round. Each row prints both ends.
"""

from __future__ import annotations

from teistro import (
    AlmanacDay,
    Altitude,
    Calendar,
    Context,
    Ephemeris,
    Latitude,
    Longitude,
    Observer,
    Teistro,
    date,
    nepal_official,
)

OFFSET_SECONDS = 20700


def clock(jd: float) -> str:
    """A Julian day as the local clock reads it."""
    local = (jd + OFFSET_SECONDS / 86400 + 0.5) % 1
    minutes = round(local * 1440) % 1440
    return f"{minutes // 60:02}:{minutes % 60:02}"


def main() -> None:
    teistro = Teistro.open()
    place = Observer(
        latitude_deg=Latitude(27.7172),
        longitude_deg=Longitude(85.324),
        altitude_m=Altitude(1400),
    )
    with teistro.context(**nepal_official(), locale="en-Latn") as official, teistro.context(
        profile="nepali-drik", locale="en-Latn", ephemeris=Ephemeris.BUILTIN
    ) as modern:

        def name(key: str) -> str:
            return official.intl.entity(key).name

        # The same month under each, one crossing each.
        def month(ctx: Context) -> list[AlmanacDay]:
            return list(
                ctx.almanac.of(
                    from_date=date(Calendar.GREGORIAN, 2025, 11, 1),
                    to_date=date(Calendar.GREGORIAN, 2025, 11, 30),
                    place=place,
                    utc_offset_seconds=OFFSET_SECONDS,
                )
            )

        days = month(official)
        others = month(modern)

        print(f"official {official.profile}, modern {modern.profile}")
        differ = 0
        for day, other in zip(days, others):
            # The first span of a day's limb is the one standing at its
            # sunrise.
            ours = day.tithi[0]
            theirs = other.tithi[0]
            if ours.member == theirs.member:
                continue
            differ += 1
            d = day.day.date
            print(
                f"{d.year}-{d.month:02}-{d.day:02}"
                f"  sunrise {clock(day.day.sunrise)}"
                f"  official {name(ours.member.full_key)} until {clock(ours.whole.to_jd)}"
                f"  modern {name(theirs.member.full_key)} until {clock(theirs.whole.to_jd)}"
            )
        print(f"{differ} of {len(days)} days name another sunrise tithi")


if __name__ == "__main__":
    main()
