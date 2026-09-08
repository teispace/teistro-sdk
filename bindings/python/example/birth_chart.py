"""A birth chart: from a Nepali birth record to the nine grahas placed.

This is the scenario the SDK exists for, and it is not one call. What it
takes, in order:

1.  A birth record as people actually write one — a Bikram Sambat date, a
    local clock time, and a place.
2.  That civil time resolved to an **instant**, which needs the zone's
    history: Nepal was +05:30 until 1986 and +05:45 after, and the
    resolution says which rule it used and from which tzdb.
3.  Positions in a **sidereal** frame. This is the step everyone gets
    wrong. The SDK's canonical frame is *tropical*, because that is what
    an ephemeris computes; a Vedic chart wants the sidereal zodiac, so
    the request names one and the SDK completes it — and stamps every
    step it applied, which this example prints.
4.  Each longitude read as a rashi, a nakshatra and a pada, using the
    catalogue's own members and the locale's own names.

What it does **not** need: an ephemeris of your own. `test_provider=True`
selects the analytic one the SDK carries so this file runs anywhere. Its
numbers are a smooth model, not an ephemeris — see `ephemeris.py` for what that
means in practice.

Run it:

    PYTHONPATH=. python3 example/birth_chart.py
"""

from __future__ import annotations

import dataclasses
from dataclasses import dataclass

from teistro import (
    Body,
    Calendar,
    Context,
    Teistro,
    at,
    date,
    iana_zone,
)
from teistro.catalogue import Ayanamsha, Graha, Nakshatra, Rashi

#: The nine grahas of a Vedic chart. Ketu is not a body an ephemeris
#: answers: it is Rahu's opposite point, so it is computed rather than
#: asked for, and the SDK's own `points` module does the same.
GRAHAS: list[tuple[Graha, Body]] = [
    (Graha.SUN, Body.SUN),
    (Graha.MOON, Body.MOON),
    (Graha.MARS, Body.MARS),
    (Graha.MERCURY, Body.MERCURY),
    (Graha.JUPITER, Body.JUPITER),
    (Graha.VENUS, Body.VENUS),
    (Graha.SATURN, Body.SATURN),
    (Graha.RAHU, Body.MEAN_NODE),
]

#: A nakshatra is a twenty-seventh of the circle; a pada a quarter of one.
NAKSHATRA_DEG = 360.0 / 27.0
PADA_DEG = NAKSHATRA_DEG / 4.0


@dataclass(frozen=True)
class Placement:
    """One graha as a chart shows it."""

    graha: Graha
    longitude: float
    speed: float

    @property
    def rashi(self) -> Rashi:
        """The sign it stands in."""
        return Rashi(int(self.longitude // 30.0))

    @property
    def degree_in_rashi(self) -> float:
        """How far into that sign, in degrees."""
        return self.longitude % 30.0

    @property
    def nakshatra(self) -> Nakshatra:
        """The lunar mansion it stands in."""
        return Nakshatra(int(self.longitude // NAKSHATRA_DEG))

    @property
    def pada(self) -> int:
        """Which quarter of that mansion, 1 to 4."""
        return int((self.longitude % NAKSHATRA_DEG) // PADA_DEG) + 1

    @property
    def retrograde(self) -> bool:
        """Whether it is moving backwards.

        There is no flag at the boundary: a graha is retrograde when its
        longitude is decreasing, which is what the speed column says.
        Rahu always is.
        """
        return self.speed < 0.0


def chart(ctx: Context, teistro: Teistro, instant: float) -> list[Placement]:
    """Every graha at one instant, in the sidereal zodiac.

    One call for the whole grid, never a loop: the boundary takes the
    instants and the bodies together and answers with columns, so asking
    for eight grahas costs one crossing rather than eight.
    """
    # The canonical frame with two fields changed. Everything else — the
    # centre, the corrections, the equinox — is left as the SDK computes
    # it, so this asks for "what you would give me, but sidereal".
    frame = dataclasses.replace(
        teistro.canonical_frame, sidereal=True, ayanamsha=Ayanamsha.LAHIRI
    )
    sky = ctx.positions(
        instants=[instant],
        bodies=[body for _, body in GRAHAS],
        frame=frame,
    )
    return [
        Placement(graha, sky.at(0, index).longitude, sky.at(0, index).longitude_speed)
        for index, (graha, _) in enumerate(GRAHAS)
    ]


def main() -> None:
    teistro = Teistro.open()
    with teistro.context(
        profile="nepali-default", locale="ne-Deva-NP", test_provider=True
    ) as ctx:
        # ── 1. The record, as it would be written on a form ───────────
        birth_day = date(Calendar.BIKRAM_SAMBAT, 2042, 9, 17)
        gregorian = ctx.convert(birth_day, Calendar.GREGORIAN)
        print(
            f"born  BS {birth_day.year}-{birth_day.month:02}-{birth_day.day:02}"
            f"  ({gregorian.year}-{gregorian.month:02}-{gregorian.day:02})"
            f"  00:20  Kathmandu"
        )

        # ── 2. The instant, with the zone's own history ───────────────
        when = ctx.resolve(
            at(birth_day, hour=0, minute=20), iana_zone("Asia/Kathmandu")
        )
        offset = when.offset_seconds
        print(
            f"      JD {when.instant_jd_utc:.6f} UTC"
            f"   offset {offset // 3600:+03}:{abs(offset) % 3600 // 60:02}"
            f"   {when.source.key} (tzdb {when.tzdb_version})"
        )
        if not when.time_known:
            print("      the time of day is not known; this chart is for noon")
        # This record sits on the day Nepal moved from +05:30 to +05:45,
        # which is why the zone's history matters and a fixed offset
        # would be wrong: `iana` above says the answer came from the
        # embedded database rather than from a guess.

        # ── 3. The sky, sidereal ──────────────────────────────────────
        placements = chart(ctx, teistro, when.instant_jd_utc)

        # ── 4. The chart ──────────────────────────────────────────────
        print()
        print(f"{'graha':12} {'':4} {'sign':12} {'deg':>9}  {'nakshatra':14} pada")
        print("─" * 62)
        for placed in placements:
            graha = ctx.entity(placed.graha.full_key)
            rashi = ctx.entity(placed.rashi.full_key)
            nakshatra = ctx.entity(placed.nakshatra.full_key)
            mark = "℞" if placed.retrograde else " "
            print(
                f"{graha.name:12} {graha.glyph or '':2} {mark:1} "
                f"{rashi.name:12} {placed.degree_in_rashi:8.4f}°  "
                f"{nakshatra.name:14} {placed.pada}"
            )

        # ── What the SDK had to do to answer ──────────────────────────
        # Every result carries the steps that produced it. Here the
        # provider answered tropical positions and the SDK applied the
        # ayanamsha and shifted the zodiac; against a provider that
        # answers sidereal natively, those steps would say so instead.
        print()
        sky = ctx.positions(
            instants=[when.instant_jd_utc],
            bodies=[Body.SUN],
            frame=dataclasses.replace(
                teistro.canonical_frame, sidereal=True, ayanamsha=Ayanamsha.LAHIRI
            ),
        )
        steps = ", ".join(
            f"{step['name']}:{step['implementation']}" for step in sky.steps_applied
        )
        print(f"steps applied  {steps}")
        print(f"settings hash  {ctx.settings_hash[:16]}…")


if __name__ == "__main__":
    main()
