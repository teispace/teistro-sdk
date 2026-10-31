"""The Python row of `docs/03-design/ffi-cost.md`.

The scenario every binding times, printed as one JSON line a key.
`cargo xtask measure-ffi-cost` runs it with `TEISTRO_LIBRARY` naming
the release library.

`call-*` asks for 1 instant and 3 bodies, `grid-*` for 366 instants and
10 bodies. `*-native` is a context over the library's test provider and
`*-hosted` one over a provider written here, answering fixed columns.
`decode` is the decoder alone, on the grid's bytes.
"""

from __future__ import annotations

import json
import time
from typing import Callable, Optional, Sequence

from teistro import Body, EphemerisProvider, PositionAnswer, PositionQuery, Teistro, TimeScale
from teistro._blob import decode_positions
from teistro._ffi import PositionRequest

RUN_SECONDS = 0.05
RUNS = 5
BODIES = (
    Body.SUN,
    Body.MOON,
    Body.MERCURY,
    Body.VENUS,
    Body.MARS,
    Body.JUPITER,
    Body.SATURN,
    Body.MEAN_NODE,
    Body.URANUS,
    Body.NEPTUNE,
)


class Fixed(EphemerisProvider):
    """A provider written in Python: every cell a fixed longitude."""

    name = "ffi-cost-in-python"
    bodies: Sequence[Body] = BODIES
    version = "1"
    data_version = "none"
    speeds = True

    def positions(self, query: PositionQuery) -> Optional[PositionAnswer]:
        cells = query.cell_count
        return PositionAnswer(
            lon=[123.25] * cells,
            lat=[0.0] * cells,
            dist=[1.0] * cells,
            lon_speed=[1.0] * cells,
        )


def timed(key: str, call: Callable[[], object]) -> None:
    for _ in range(100):
        call()
    calls = 1
    while True:
        start = time.perf_counter()
        for _ in range(calls):
            call()
        if time.perf_counter() - start >= RUN_SECONDS or calls >= 1 << 24:
            break
        calls *= 2
    runs = []
    for _ in range(RUNS):
        start = time.perf_counter()
        for _ in range(calls):
            call()
        runs.append((time.perf_counter() - start) * 1e6 / calls)
    runs.sort()
    print(
        json.dumps(
            {
                "binding": "python",
                "key": key,
                "runs": RUNS,
                "calls": calls,
                "median_us": runs[RUNS // 2],
                "min_us": runs[0],
                "max_us": runs[-1],
            }
        )
    )


def main() -> None:
    teistro = Teistro.open()
    one = [2451545.0]
    three = [Body.SUN, Body.MOON, Body.MARS]
    year = [2451545.0 + day for day in range(366)]
    with teistro.context(profile="nepali-drik", test_provider=True) as native, \
            teistro.context(profile="nepali-drik", provider=Fixed()) as hosted:
        # The hosted provider's answer is what it sent, so a figure is
        # never taken over a call that failed or answered something else.
        answered = hosted.positions(instants=year, bodies=list(BODIES))
        if answered.jd_count * answered.body_count != 3660 or answered.at(365, 9).longitude != 123.25:
            raise SystemExit("the hosted provider did not answer the grid it was asked")
        timed("call-native", lambda: native.positions(instants=one, bodies=three).jd_count)
        timed("call-hosted", lambda: hosted.positions(instants=one, bodies=three).jd_count)
        timed("grid-native", lambda: native.positions(instants=year, bodies=list(BODIES)).jd_count)
        timed("grid-hosted", lambda: hosted.positions(instants=year, bodies=list(BODIES)).jd_count)
        request = PositionRequest(
            scale=TimeScale.UT1,
            frame_bits=teistro.pack_frame(teistro.canonical_frame),
            speeds=True,
            jds=year,
            bodies=list(BODIES),
            observer=None,
        )
        raw = native.inner.positions(request)
        timed("decode", lambda: decode_positions(raw).jd_count)


if __name__ == "__main__":
    main()
