"""A year of the sky in one call, and what to do with the columns.

The boundary takes a **grid** — instants by bodies — and answers with a
result blob whose sections are columns. That shape is the whole reason a
year of positions costs one crossing rather than 365, and it is what a
service computing tables, transits or ingresses should be using.

Three things this example is really about:

1.  **One call, not a loop.** 366 instants by 3 bodies is 1098 cells in
    a single request. Asking day by day would cross the boundary 366
    times and recompute the provider's own setup each time.
2.  **The columns are views, not copies.** `positions.decoded.cells.lon`
    is a `memoryview` over the blob's own bytes. `numpy.asarray` wraps
    it without copying, so a table of a million cells costs one
    allocation. numpy is not a dependency; the buffer protocol is.
3.  **What the answer says about itself.** Every result carries the
    steps applied and a provenance envelope with the settings hash — the
    two things a cache key and an audit trail are made of.

``Ephemeris.BUILTIN`` computes with the analytic ephemeris the SDK
carries, so this file runs anywhere with nothing installed. It is the
fallback rather than the intended path — most consumers should be on a
real engine — but it is astronomy: the scans below find sign ingresses
and retrograde stations because the sky has them.

Run it:

    PYTHONPATH=. python3 example/ephemeris.py
"""

from __future__ import annotations

import dataclasses
import json

from teistro import Body, Context, Ephemeris, Teistro
from teistro.catalogue import Ayanamsha, Graha, Rashi

#: A year from the start of 2025, one sample a day at noon UTC.
START_JD = 2460676.5
DAYS = 366
#: A body is what an ephemeris answers; a graha is what a chart names.
#: They are different catalogues and the lunar node is where they part —
#: `MEAN_NODE` is the body, `RAHU` the graha — so the two are paired
#: explicitly rather than derived from each other's spelling.
BODIES: list[tuple[Body, Graha]] = [
    (Body.SUN, Graha.SUN),
    (Body.MARS, Graha.MARS),
    (Body.MEAN_NODE, Graha.RAHU),
]


def ingresses(longitudes: object, day_count: int, stride: int, column: int) -> list[tuple[int, Rashi]]:
    """Every day on which a body changed sign.

    Reads one body's column out of the grid. Cells run instants
    outermost, so body `column` at day `i` is cell `i * stride + column`.
    """
    found: list[tuple[int, Rashi]] = []
    previous: Rashi | None = None
    for day in range(day_count):
        sign = Rashi(int(longitudes[day * stride + column] // 30.0))  # type: ignore[index]
        if previous is not None and sign != previous:
            found.append((day, sign))
        previous = sign
    return found


def stations(speeds: object, day_count: int, stride: int, column: int) -> list[tuple[int, str]]:
    """Every day on which a body turned, direct to retrograde or back.

    The same shape as `ingresses` over a different column: a station is a
    sign change in `lon_speed` rather than in `lon`. One grid answers
    both, which is the reason to ask for a grid.
    """
    found: list[tuple[int, str]] = []
    for day in range(1, day_count):
        before = speeds[(day - 1) * stride + column]  # type: ignore[index]
        after = speeds[day * stride + column]  # type: ignore[index]
        if (before < 0) != (after < 0):
            found.append((day, "retrograde" if after < 0 else "direct"))
    return found


def main() -> None:
    teistro = Teistro.open()
    with teistro.context(
        profile="nepali-default", locale="ne-Deva-NP", ephemeris=Ephemeris.BUILTIN
    ) as ctx:
        # ── Which build am I talking to? ──────────────────────────────
        # A service checks this once at start-up. The binding already
        # refuses a library that is not the build it was generated from;
        # this is how to log what it did load.
        build = teistro.build
        print(
            f"library  Teistro {build.sdk}  ABI {build.abi}"
            f"  catalogue {build.catalogue}  {build.target}"
            f"  {build.commit[:8]}{'-dirty' if build.dirty else ''}"
        )

        # ── One call for the whole year ───────────────────────────────
        frame = dataclasses.replace(
            teistro.canonical_frame, sidereal=True, ayanamsha=Ayanamsha.LAHIRI
        )
        instants = [START_JD + day for day in range(DAYS)]
        sky = ctx.positions(
            instants=instants, bodies=[body for body, _ in BODIES], frame=frame
        )
        print(
            f"grid     {sky.instant_count} instants x {sky.body_count} bodies"
            f" = {sky.cell_count} cells in one call"
        )

        # ── The columns ───────────────────────────────────────────────
        cells = sky.decoded.cells
        print(
            f"columns  lon is a {type(cells.lon).__name__}"
            f" of {cells.lon.format!r}, {cells.lon.nbytes} bytes,"
            f" read-only={cells.lon.readonly}"
        )
        # numpy, when a caller has it, wraps this without copying:
        #     import numpy as np
        #     longitudes = np.asarray(cells.lon).reshape(DAYS, len(BODIES))

        # ── What the columns are for ──────────────────────────────────
        print()
        for column, (body, graha) in enumerate(BODIES):
            name = ctx.intl.entity(graha.full_key).name
            crossings = ingresses(cells.lon, DAYS, sky.body_count, column)
            turns = stations(cells.lon_speed, DAYS, sky.body_count, column)
            speed = cells.lon_speed[column]
            direction = "retrograde" if speed < 0 else "direct"
            print(
                f"  {body.key:10} {name:8} {direction:10} at {speed:+8.4f}°/day,"
                f" {len(crossings)} sign change(s), {len(turns)} station(s)"
            )
            for day, sign in crossings[:3]:
                sign_name = ctx.intl.entity(sign.full_key).name
                print(f"      day {day:3}  enters {sign.key:12} {sign_name}")
            if len(crossings) > 3:
                print(f"      … and {len(crossings) - 3} more")
            for day, into in turns:
                print(f"      day {day:3}  turns  {into}")

        # ── What the answer says about itself ─────────────────────────
        print()
        steps = ", ".join(
            f"{step['name']}:{step['implementation']}" for step in sky.steps_applied
        )
        print(f"steps    {steps}")
        provenance = sky.provenance_of
        print(f"profile  {provenance['profile']}")
        print(f"hash     {provenance['settings_hash']}")
        print(
            "         two contexts with the same settings hash compute the"
            " same numbers, so it is the cache key"
        )
        # The whole envelope is canonical JSON: byte-identical across
        # every binding, which is what makes it safe to hash and store.
        print(f"envelope {len(json.dumps(provenance, separators=(',', ':')))} bytes of canonical JSON")


if __name__ == "__main__":
    main()
