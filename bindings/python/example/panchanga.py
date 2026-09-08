"""The five limbs of a day, computed from the boundary alone.

A panchanga is the Hindu almanac's five parts — tithi, vara, nakshatra,
yoga and karana — and every one of them except the weekday is a function
of **two longitudes**: the Sun's and the Moon's, in the sidereal zodiac.
So a binding can compute a whole panchanga from `positions` and
`weekday`, without any part of the chart layer.

The arithmetic is the SDK's own (`crates/panchanga/src/limb.rs`):

| limb      | from                       | divisions | each |
|-----------|----------------------------|-----------|------|
| tithi     | Moon − Sun                 | 30        | 12°  |
| karana    | Moon − Sun                 | 60        | 6°   |
| nakshatra | Moon                       | 27        | 360/27° |
| yoga      | Moon + Sun                 | 27        | 360/27° |
| vara      | the weekday                | 7         | a day |

The karana is the one that is not a plain division: sixty half-tithis
make a lunar month and they are **not** a cycle of eleven. Kimstughna
opens the month, Shakuni, Chatushpada and Naga close it, and the seven
movable karanas repeat through everything between. `karana_of` below is
the SDK's rule, transcribed, and the SDK holds it to the corpus's own
successor relation over 109 consecutive pairs.

What this example is honest about: a limb here is the one holding **at
the instant asked for**. A printed almanac gives the limb at sunrise and
the time it ends, which needs a boundary search over the Moon's motion —
that is what `crates/panchanga` does with a real ephemeris, and what a
binding cannot yet ask for.

Run it:

    PYTHONPATH=. python3 example/panchanga.py
"""

from __future__ import annotations

import dataclasses
from dataclasses import dataclass

from teistro import Body, Calendar, Context, Teistro, at, date, iana_zone
from teistro.catalogue import Ayanamsha, Karana, Nakshatra, Tithi, Vara, Yoga

NAKSHATRA_DEG = 360.0 / 27.0
YOGA_DEG = 360.0 / 27.0
TITHI_DEG = 12.0
KARANA_DEG = 6.0


def karana_of(half_tithi: int) -> Karana:
    """The karana that a half-tithi of the lunar month is.

    Transcribed from `crates/panchanga/src/limb.rs`. Sixty of them make a
    month: Kimstughna opens it, Shakuni, Chatushpada and Naga close it,
    and the seven movable karanas fill everything between, Bava first
    from the second half of the first tithi.
    """
    half = half_tithi % 60
    if half == 0:
        return Karana.KIMSTUGHNA
    if half == 57:
        return Karana.SHAKUNI
    if half == 58:
        return Karana.CHATUSHPADA
    if half == 59:
        return Karana.NAGA
    return Karana((half - 1) % 7)


@dataclass(frozen=True)
class Panchanga:
    """The five limbs at one instant, with the numbers behind them."""

    sun: float
    moon: float
    tithi: Tithi
    vara: Vara
    nakshatra: Nakshatra
    yoga: Yoga
    karana: Karana

    @property
    def elongation(self) -> float:
        """How far the Moon stands ahead of the Sun, 0 to 360."""
        return (self.moon - self.sun) % 360.0

    @property
    def paksha(self) -> str:
        """The fortnight: waxing to the full moon, waning after it."""
        return "shukla" if self.elongation < 180.0 else "krishna"

    @property
    def tithi_elapsed(self) -> float:
        """How far through the tithi the Moon has come, as a fraction.

        A printed almanac gives the *time* the tithi ends; this is the
        fraction, which is what the fraction of a boundary search would
        start from.
        """
        return (self.elongation % TITHI_DEG) / TITHI_DEG


def panchanga_at(ctx: Context, teistro: Teistro, instant: float, weekday: int) -> Panchanga:
    """The five limbs at an instant.

    `weekday` is the ISO weekday of the **civil day** the instant belongs
    to, which the caller has because it asked the calendar for it: a vara
    is a property of the day, not of the moment.
    """
    frame = dataclasses.replace(
        teistro.canonical_frame, sidereal=True, ayanamsha=Ayanamsha.LAHIRI
    )
    sky = ctx.positions(
        instants=[instant], bodies=[Body.SUN, Body.MOON], frame=frame
    )
    sun = sky.at(0, 0).longitude
    moon = sky.at(0, 1).longitude
    elongation = (moon - sun) % 360.0
    return Panchanga(
        sun=sun,
        moon=moon,
        tithi=Tithi(int(elongation // TITHI_DEG)),
        # The boundary's weekday is ISO (Monday 1 … Sunday 7) and a vara
        # counts from Sunday, so the one becomes the other by `% 7`.
        vara=Vara(weekday % 7),
        nakshatra=Nakshatra(int(moon // NAKSHATRA_DEG)),
        yoga=Yoga(int(((moon + sun) % 360.0) // YOGA_DEG)),
        karana=karana_of(int(elongation // KARANA_DEG)),
    )


def main() -> None:
    teistro = Teistro.open()
    with teistro.context(
        profile="nepali-default", locale="ne-Deva-NP", test_provider=True
    ) as ctx:
        # Nepali New Year: the first day of Baisakh, BS 2082.
        day = date(Calendar.BIKRAM_SAMBAT, 2082, 1, 1)
        gregorian = ctx.convert(day, Calendar.GREGORIAN)
        # Six in the morning stands in for sunrise, which the almanac
        # would use and which needs the rise-and-set solver.
        when = ctx.resolve(at(day, hour=6), iana_zone("Asia/Kathmandu"))
        found = panchanga_at(
            ctx, teistro, when.instant_jd_utc, ctx.weekday_of(day)
        )

        print(
            f"BS {day.year}-{day.month:02}-{day.day:02}"
            f"  ({gregorian.year}-{gregorian.month:02}-{gregorian.day:02})"
            f"  06:00 Kathmandu"
        )
        print(
            f"  sun {found.sun:8.4f}°   moon {found.moon:8.4f}°"
            f"   elongation {found.elongation:8.4f}°"
        )
        print()

        for label, member in (
            ("tithi", found.tithi),
            ("vara", found.vara),
            ("nakshatra", found.nakshatra),
            ("yoga", found.yoga),
            ("karana", found.karana),
        ):
            entity = ctx.entity(member.full_key)
            print(
                f"  {label:10} {entity.name:14} {entity.iast:18} "
                f"({member.key})"
            )
        print()
        print(f"  paksha     {found.paksha}")
        print(f"  tithi is   {found.tithi_elapsed:.1%} elapsed at this instant")

        # The Sun on this day is the reason the year turns: BS begins at
        # the Mesha Sankranti, when the Sun enters Aries. At six in the
        # morning it has not quite arrived, which is why the almanac's
        # own year-start is an instant and not a date.
        print(
            f"  the Sun stands {(360.0 - found.sun) % 360.0:.4f}° short of Aries,"
            " which is what the new year waits for"
        )


if __name__ == "__main__":
    main()
