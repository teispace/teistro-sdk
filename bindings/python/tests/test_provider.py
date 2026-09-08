"""An ephemeris written in Python, driven by the SDK.

The port's contract is one call per whole grid, never a loop, and a
provider that answers `None` means "not in that frame" rather than
"nothing". What this file holds is that a Python callable really is
reached through the vtable, that its answer comes back as the SDK's own
result, and — the part that has no equivalent in a compiled binding —
that an exception raised inside a `ctypes` callback reaches the caller
instead of printing a traceback and returning zero, which the port would
read as success.
"""

from __future__ import annotations

import unittest
from typing import Optional, Sequence

from teistro import (
    Body,
    EphemerisProvider,
    PositionAnswer,
    PositionQuery,
    Status,
    TeistroError,
)
from tests.support import PROFILE, WithLibrary


class StraightLine(EphemerisProvider):
    """A provider that answers a line: enough to prove the path, and
    nothing an ephemeris would recognise."""

    name = "straight-line"
    bodies: Sequence[Body] = (Body.SUN, Body.MOON)
    version = "1.0"
    data_version = "none"
    speeds = True

    def __init__(self) -> None:
        self.asked = 0
        self.last: Optional[PositionQuery] = None

    def positions(self, query: PositionQuery) -> Optional[PositionAnswer]:
        self.asked += 1
        self.last = query
        cells = query.cell_count
        return PositionAnswer(
            lon=[(index * 10.0) % 360.0 for index in range(cells)],
            lat=[0.0] * cells,
            dist=[1.0] * cells,
            lon_speed=[1.0] * cells,
        )


class Raising(StraightLine):
    """A provider that fails the way a real one does: with a sentence."""

    name = "raising"

    def positions(self, query: PositionQuery) -> Optional[PositionAnswer]:
        raise RuntimeError("the ephemeris file is not where it said it was")


class NotInThatFrame(StraightLine):
    """A provider that answers only in its own frame."""

    name = "native-only"

    def positions(self, query: PositionQuery) -> Optional[PositionAnswer]:
        self.asked += 1
        return None


class AProviderWrittenInPython(WithLibrary):
    def test_it_is_asked_once_for_a_whole_grid(self) -> None:
        provider = StraightLine()
        with self.teistro.context(profile=PROFILE, provider=provider) as ctx:
            sky = ctx.positions(
                instants=[2451545.0, 2451546.0, 2451547.0],
                bodies=[Body.SUN, Body.MOON],
            )
        self.assertEqual(provider.asked, 1, "one call, not six")
        assert provider.last is not None
        self.assertEqual(provider.last.cell_count, 6)
        self.assertEqual(list(provider.last.jds), [2451545.0, 2451546.0, 2451547.0])
        self.assertEqual(list(provider.last.bodies), [Body.SUN, Body.MOON])
        self.assertEqual(sky.cell_count, 6)
        self.assertAlmostEqual(sky.at(0, 0).longitude, 0.0)
        self.assertAlmostEqual(sky.at(0, 1).longitude, 10.0)
        self.assertAlmostEqual(sky.at(1, 0).longitude, 20.0)

    def test_its_name_is_stamped_on_the_result(self) -> None:
        provider = StraightLine()
        with self.teistro.context(profile=PROFILE, provider=provider) as ctx:
            sky = ctx.positions(instants=[2451545.0], bodies=[Body.SUN])
            self.assertIs(ctx.provider, provider)
        provenance = sky.provenance_of
        self.assertIn("straight-line", repr(provenance))

    def test_a_body_it_never_declared_is_refused_by_name(self) -> None:
        provider = StraightLine()
        with self.teistro.context(profile=PROFILE, provider=provider) as ctx:
            with self.assertRaises(Exception) as caught:
                ctx.positions(instants=[2451545.0], bodies=[Body.MARS])
        self.assertIn("mars", str(caught.exception))
        self.assertEqual(provider.asked, 0, "it was never asked")

    def test_an_instant_outside_its_coverage_is_refused(self) -> None:
        class Narrow(StraightLine):
            name = "narrow"
            jd_min = 2451545.0
            jd_max = 2451546.0

        provider = Narrow()
        with self.teistro.context(profile=PROFILE, provider=provider) as ctx:
            with self.assertRaises(Exception) as caught:
                ctx.positions(instants=[2400000.0], bodies=[Body.SUN])
        self.assertIn("coverage", str(caught.exception))

    def test_what_it_raises_reaches_the_caller(self) -> None:
        # This is the one that matters: a Python exception escaping a
        # `ctypes` callback prints a traceback and returns zero, which the
        # port reads as success. The adapter catches everything and keeps
        # the exception for the layer above to re-raise.
        provider = Raising()
        with self.teistro.context(profile=PROFILE, provider=provider) as ctx:
            with self.assertRaises(RuntimeError) as caught:
                ctx.positions(instants=[2451545.0], bodies=[Body.SUN])
        self.assertIn("not where it said it was", str(caught.exception))

    def test_none_means_not_in_that_frame(self) -> None:
        provider = NotInThatFrame()
        with self.teistro.context(profile=PROFILE, provider=provider) as ctx:
            with self.assertRaises(TeistroError) as caught:
                ctx.positions(instants=[2451545.0], bodies=[Body.SUN])
        self.assertNotEqual(caught.exception.status, Status.OK)
        self.assertGreaterEqual(provider.asked, 1)

    def test_a_provider_with_no_name_or_no_bodies_is_refused_when_it_binds(self) -> None:
        class Nameless(StraightLine):
            name = ""

        class Empty(StraightLine):
            bodies: Sequence[Body] = ()

        with self.assertRaises(ValueError):
            self.teistro.context(profile=PROFILE, provider=Nameless())
        with self.assertRaises(ValueError):
            self.teistro.context(profile=PROFILE, provider=Empty())


if __name__ == "__main__":
    unittest.main()
