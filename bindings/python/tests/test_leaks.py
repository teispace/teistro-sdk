"""The binding gives back the native memory it takes
(`docs/05-testing/01-quality-bar.md`, "leak checks per binding").

A context, a chart and a positions answer each hold memory the library
allocated, which only the binding's frees give back: `close`, the
finaliser of a context dropped unclosed, and the free after a blob is
copied out. Each cycle runs a tenth of its count to settle the
allocator, then its count measured, and the process's peak resident set
must not grow by more than `BUDGET`. A context kept rather than freed
costs hundreds of kilobytes, so one leaked a cycle overruns the budget
within a few cycles; over the two thousand each family runs, so does a
leak of five hundred bytes a cycle.

Peak resident memory is what `getrusage` reports on Linux and macOS;
Windows has no `resource` module and the test is skipped there.
"""

from __future__ import annotations

import gc
import sys
import unittest
from typing import Callable

from teistro import Altitude, Body, Ephemeris, Latitude, Longitude, Observer
from tests.support import LOCALE, PROFILE, WithLibrary

#: The growth of the peak resident set a run may show, in bytes.
BUDGET = 1 << 20

#: The measured cycles of each family.
CYCLES = 2_000


def peak() -> int:
    """The process's peak resident set, in bytes."""
    if sys.platform == "win32":
        raise unittest.SkipTest("getrusage is POSIX")
    import resource

    used = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
    # macOS reports bytes, Linux kibibytes.
    return used if sys.platform == "darwin" else used * 1024


@unittest.skipIf(sys.platform == "win32", "getrusage is POSIX")
class NothingIsKept(WithLibrary):
    def grows(self, cycles: int, cycle: Callable[[], None]) -> int:
        """How much the peak grew over `cycles` measured runs of `cycle`."""
        for _ in range(cycles // 10):
            cycle()
        gc.collect()
        before = peak()
        for _ in range(cycles):
            cycle()
        gc.collect()
        return peak() - before

    def test_a_context_closed_or_dropped_gives_back_its_memory(self) -> None:
        def closed() -> None:
            with self.teistro.context(test_provider=True) as ctx:
                self.assertTrue(ctx.profile)

        def dropped() -> None:
            ctx = self.teistro.context(test_provider=True)
            self.assertTrue(ctx.profile)

        for cycle in (closed, dropped):
            with self.subTest(cycle=cycle.__name__):
                self.assertLessEqual(self.grows(CYCLES, cycle), BUDGET)

    def test_charts_and_positions_give_back_their_answers(self) -> None:
        place = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )
        with self.teistro.context(profile=PROFILE, locale=LOCALE, ephemeris=Ephemeris.BUILTIN) as ctx:

            def answered() -> None:
                ctx.chart.found(instant=2451545.0, place=place, utc_offset_seconds=20700)
                ctx.positions(instants=[2451545.0, 2451546.0], bodies=[Body.SUN, Body.MOON])

            self.assertLessEqual(self.grows(CYCLES, answered), BUDGET)


if __name__ == "__main__":
    unittest.main()
