"""A birth chart: from a Nepali birth record to the nine grahas placed.

This is the scenario the SDK exists for, and it is not one call. What it
takes, in order:

1.  A birth record as people actually write one — a Bikram Sambat date, a
    local clock time, and a place.
2.  That civil time resolved to an **instant**, which needs the zone's
    history: Nepal was +05:30 until 1986 and +05:45 after, and the
    resolution says which rule it used and from which tzdb.
3.  The chart **founded** at that instant and place. The SDK's canonical
    frame is *tropical*, because that is what an ephemeris computes; a
    Vedic chart wants the sidereal zodiac, and the profile says which
    ayanamsha and which centre, so the founder asks for that frame and
    the SDK completes it — and stamps every step it applied, which this
    example prints. Positions asked for directly would not know the
    profile wants the Moon seen from Kathmandu rather than from the
    Earth's centre, which moves it most of a degree.
4.  Each longitude read as a rashi, a nakshatra and a pada, using the
    catalogue's own members and the locale's own names.

What it does **not** need: an ephemeris of your own, a data file, a
network, or a second library. `ephemeris=Ephemeris.BUILTIN` selects the
one the SDK carries, so every position below is a real sky and this file
runs anywhere the package installs — see `ephemeris.py` for how to bind
one of your own instead.

Run it:

    PYTHONPATH=. python3 example/birth_chart.py
"""

from __future__ import annotations

from teistro import (
    Altitude,
    Calendar,
    ChartKind,
    Ephemeris,
    Latitude,
    Longitude,
    Observer,
    Teistro,
    TeistroError,
    at,
    date,
    iana_zone,
    when_unknown,
)
from teistro.catalogue import Nakshatra, Rashi

#: A nakshatra is a twenty-seventh of the circle; a pada a quarter of one.
NAKSHATRA_DEG = 360.0 / 27.0
PADA_DEG = NAKSHATRA_DEG / 4.0


def rashi_of(longitude: float) -> tuple[Rashi, float]:
    """The sign a longitude stands in, and how far into it."""
    return Rashi(int(longitude // 30.0)), longitude % 30.0


def nakshatra_of(longitude: float) -> tuple[Nakshatra, int]:
    """The lunar mansion a longitude stands in, and which quarter of it, 1 to 4."""
    return (
        Nakshatra(int(longitude // NAKSHATRA_DEG)),
        int((longitude % NAKSHATRA_DEG) // PADA_DEG) + 1,
    )


def main() -> None:
    teistro = Teistro.open()
    with teistro.context(
        profile="nepali-default",
        locale="ne-Deva-NP",
        ephemeris=Ephemeris.BUILTIN,
    ) as ctx:
        # ── 1. The record, as it would be written on a form ───────────
        birth_day = date(Calendar.BIKRAM_SAMBAT, 2042, 9, 17)
        gregorian = ctx.calendar.convert(birth_day, Calendar.GREGORIAN)
        print(
            f"born  BS {birth_day.year}-{birth_day.month:02}-{birth_day.day:02}"
            f"  ({gregorian.year}-{gregorian.month:02}-{gregorian.day:02})"
            f"  00:20  Kathmandu"
        )

        # ── 2. The instant, with the zone's own history ───────────────
        when = ctx.time.resolve(
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

        # ── 3. The chart ──────────────────────────────────────────────
        place = Observer(
            latitude_deg=Latitude(27.7172),
            longitude_deg=Longitude(85.324),
            altitude_m=Altitude(1400),
        )
        chart = ctx.chart.found(
            instant=when.instant_jd_utc,
            place=place,
            utc_offset_seconds=when.offset_seconds,
            kind=ChartKind.NATAL,
        )

        print()
        print("graha             sign               deg  nakshatra      pada bhava")
        print("─" * 67)
        for placed in chart.grahas:
            graha = ctx.intl.entity(placed.graha.full_key)
            rashi, degrees = rashi_of(placed.longitude_deg)
            nakshatra, pada = nakshatra_of(placed.longitude_deg)
            # There is no retrograde flag to trust blindly: a graha is
            # retrograde when its longitude is decreasing, which is what
            # the speed says, and `retrograde` is that comparison, named.
            mark = "℞" if placed.retrograde else " "
            # The bhava is the chart's placement system's answer -- the
            # question most of the tradition answers with "in the seventh".
            print(
                f"{graha.name:12} {graha.glyph or '':2} {mark:1} "
                f"{ctx.intl.entity(rashi.full_key).name:12} {degrees:8.4f}°  "
                f"{ctx.intl.entity(nakshatra.full_key).name:14} {pada}   "
                f"{placed.house.bhava:2}"
            )

        # ── 4. What the chart is measured in, and against ─────────────
        print()
        lagna, into = rashi_of(chart.lagna_deg)
        print(
            f"lagna          {chart.lagna_deg:.4f}° -- {ctx.intl.entity(lagna.full_key).name}"
            f" at {into:.4f}°, vara {chart.vara.full_key}"
        )
        # `None`, and it means what it says: a tropical chart has no
        # ayanamsha, not an ayanamsha of nought.
        if chart.ayanamsha is not None:
            applied = chart.ayanamsha.full_key
        elif chart.ayanamsha_custom:
            applied = "custom"
        else:
            applied = None
        if applied is None:
            print("ayanamsha      tropical, none applied")
        else:
            print(f"ayanamsha      {chart.ayanamsha_offset_deg:.6f}° applied ({applied})")
        print(f"steps applied  {', '.join(chart.batch.steps_applied)}")
        # The provenance envelope stamps the settings, the provider and
        # the time layer; it is what a stored chart keeps in order to say
        # what computed it.
        print(f"settings hash  {ctx.settings_hash[:16]}…")

    # ── A birth with no recorded time ─────────────────────────────────
    # The commonest data problem in the field, and the SDK does **not**
    # pick a time for you. `when_unknown` says the time is unknown; what
    # happens next is the profile's `time.unknown_time` policy, and by
    # default there is none, so the call is refused with a hint naming
    # the choices.
    print("")
    no_time = when_unknown(birth_day)
    for policy in (None, "NOON", "MIDNIGHT"):
        settings = None if policy is None else {"time": {"unknown_time": policy}}
        with teistro.context(
            profile="nepali-default",
            locale="ne-Deva-NP",
            ephemeris=Ephemeris.BUILTIN,
            settings=settings,
        ) as scoped:
            label = (policy or "refuse").ljust(9)
            try:
                resolved = scoped.time.resolve(no_time, iana_zone("Asia/Kathmandu"))
            except TeistroError as error:
                print(f"{label} {error.message}")
                print(f"{' ' * 10}hint: {error.hint}")
            else:
                # The warnings are catalogue members here rather than
                # strings, so the key is what to print; it is the same
                # word in every binding.
                warnings = (
                    ", ".join(w.key for w in resolved.warnings) or "(no warning)"
                )
                print(
                    f"{label} JD {resolved.instant_jd_utc:.6f}"
                    f"  time known {str(resolved.time_known).lower()}  {warnings}"
                )
    # MIDNIGHT is refused for a different reason, and it is this record's
    # own: the clocks jumped at midnight on this very date, so 00:00
    # never happened in Kathmandu. A chart cast on a guessed midnight
    # would have been cast on a time that does not exist.
    # NOON answers, and says so twice — `time_known` is false and the
    # resolution carries a `time-unknown-fallback` warning — so a stored
    # chart can never quietly claim a birth time it never had.


if __name__ == "__main__":
    main()
