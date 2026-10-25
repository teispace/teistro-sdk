"""The binding gives back the native memory it takes
(`docs/05-testing/01-quality-bar.md`, "leak checks per binding").

A context, a chart and a positions answer each hold memory the library
allocated, which only the binding's frees give back: `close`, the
finaliser of a context dropped unclosed, and the free after a blob is
copied out. The measure is the C allocator's own count of the bytes in
use, which the library's allocations go through: `malloc_zone_statistics`
on macOS and glibc's `mallinfo2` on Linux. The resident set will not do:
on macOS a process keeping thousands of contexts was seen to *shrink*
from 2.3 GB to 1.0 GB as the system compressed the idle pages.

Each family runs its cycles once to settle, then `RUNS` times measured,
and at least one measured run must leave the bytes in use within
`BUDGET` of where it began: a leak grows them in every run, while what
the interpreter allocates for itself moves them a little, and now and
then once. A context is about two megabytes, so one kept a cycle
overruns the budget at once; a leak of 150 bytes a cycle overruns it
over the cycles of one run. Where the allocator cannot be read
(Windows, musl), the test is skipped.

It is skipped on Intel macOS too, where the zones' `size_in_use` is not
the bytes live. There it climbed about 1.5 MB every 2000 positions calls
with the nano zone on or off and the memo off, while `heap(1)` counted
the same 34 241 blocks after 500 calls and after 4500, and the library's
own counting-allocator test (`crates/ffi/tests/leaks.rs`) got every byte
back on the same runner. The same build reads flat on Apple silicon.

And under a sanitizer gate (`$TEISTRO_SANITIZER`), whose runtime owns
`malloc` and holds freed blocks back on purpose, so they never read as
given back.
"""

from __future__ import annotations

import ctypes
import gc
import os
import platform
import sys
import unittest
from typing import Callable, Optional

from teistro import Altitude, Body, Ephemeris, Latitude, Longitude, Observer
from tests.support import LOCALE, PROFILE, WithLibrary

#: How far the bytes in use may move over a measured run.
BUDGET = 64 * 1024

#: The cycles of one run.
CYCLES = 500

#: The measured runs of each family.
RUNS = 3


class _ZoneStatistics(ctypes.Structure):
    """macOS's `malloc_statistics_t`."""

    _fields_ = [
        ("blocks_in_use", ctypes.c_uint),
        ("size_in_use", ctypes.c_size_t),
        ("max_size_in_use", ctypes.c_size_t),
        ("size_allocated", ctypes.c_size_t),
    ]


class _Mallinfo2(ctypes.Structure):
    """glibc's `struct mallinfo2`."""

    _fields_ = [
        (name, ctypes.c_size_t)
        for name in (
            "arena",
            "ordblks",
            "smblks",
            "hblks",
            "hblkhd",
            "usmblks",
            "fsmblks",
            "uordblks",
            "fordblks",
            "keepcost",
        )
    ]


def _reader() -> Optional[Callable[[], int]]:
    """How to read the bytes in use, where the allocator says."""
    if sys.platform == "win32":
        return None
    libc = ctypes.CDLL(None)
    if sys.platform == "darwin":
        if platform.machine() == "x86_64":
            return None
        statistics = libc.malloc_zone_statistics
        statistics.argtypes = [ctypes.c_void_p, ctypes.POINTER(_ZoneStatistics)]
        statistics.restype = None

        def zones() -> int:
            stats = _ZoneStatistics()
            statistics(None, ctypes.byref(stats))
            return int(stats.size_in_use)

        return zones
    mallinfo2 = getattr(libc, "mallinfo2", None)
    if mallinfo2 is None:
        return None
    mallinfo2.argtypes = []
    mallinfo2.restype = _Mallinfo2

    def arenas() -> int:
        info = mallinfo2()
        # The small blocks in use and the large ones mapped on their own.
        return int(info.uordblks + info.hblkhd)

    return arenas


IN_USE = _reader()


@unittest.skipIf(IN_USE is None, "the allocator's bytes in use cannot be read faithfully here")
@unittest.skipIf(
    bool(os.environ.get("TEISTRO_SANITIZER")),
    "a sanitizer's runtime holds freed blocks back",
)
class NothingIsKept(WithLibrary):
    def assert_gives_back(self, cycle: Callable[[], None]) -> None:
        """At least one of `RUNS` runs grows the bytes in use within `BUDGET`.

        A failure names every run's growth and one run's over four times the
        cycles: a leak grows with the cycles, and what the allocator keeps
        for itself does not.
        """
        assert IN_USE is not None

        def run(cycles: int) -> int:
            gc.collect()
            before = IN_USE()
            for _ in range(cycles):
                cycle()
            gc.collect()
            return IN_USE() - before

        run(CYCLES)
        grown = [run(CYCLES) for _ in range(RUNS)]
        if min(grown) <= BUDGET:
            return
        longer = run(4 * CYCLES)
        self.fail(
            f"the bytes in use grew {grown} over {RUNS} runs of {CYCLES} cycles "
            f"({min(grown) / CYCLES:.0f} a cycle at least), and {longer} over "
            f"{4 * CYCLES}; the budget is {BUDGET}"
        )

    def test_a_context_closed_or_dropped_gives_back_its_memory(self) -> None:
        def closed() -> None:
            with self.teistro.context(test_provider=True) as ctx:
                self.assertTrue(ctx.profile)

        def dropped() -> None:
            ctx = self.teistro.context(test_provider=True)
            self.assertTrue(ctx.profile)

        for cycle in (closed, dropped):
            with self.subTest(cycle=cycle.__name__):
                self.assert_gives_back(cycle)

    def test_charts_and_positions_give_back_their_answers(self) -> None:
        place = Observer(
            latitude_deg=Latitude(27.7172), longitude_deg=Longitude(85.324), altitude_m=Altitude(1400)
        )
        with self.teistro.context(profile=PROFILE, locale=LOCALE, ephemeris=Ephemeris.BUILTIN) as ctx:

            def answered() -> None:
                ctx.chart.found(instant=2451545.0, place=place, utc_offset_seconds=20700)
                ctx.positions(instants=[2451545.0, 2451546.0], bodies=[Body.SUN, Body.MOON])

            self.assert_gives_back(answered)


if __name__ == "__main__":
    unittest.main()
