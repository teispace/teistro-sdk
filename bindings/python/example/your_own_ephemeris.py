"""An ephemeris of your own, and what happens when it goes wrong.

The SDK computes no positions itself: it asks a **provider**, and a
provider written in Python is a first-class one. That is the point of
the port — an application that already has an ephemeris, a cache, or a
table of precomputed positions can put it behind the SDK and get the
whole chart layer for free.

The contract is small and worth reading carefully:

- **One call for the whole grid, never a loop.** The SDK hands over
  every instant and every body at once and expects every cell back.
- **Answer `None` for "not in that frame".** A provider that computes
  equatorial positions says so once, in `native_frame`, and returns
  `None` when asked for anything else; the SDK then asks again in the
  provider's own frame and completes the rest itself, stamping each step.
  This is why an engine that knows nothing about the ecliptic can still
  serve a Vedic chart.
- **Say what you cover.** `bodies`, `jd_min` and `jd_max` are checked
  *before* the provider is called, so a request it cannot serve is
  refused by name rather than by a wrong answer.
- **Raising is allowed.** An exception is carried across the boundary as
  a refusal code and re-raised on the caller's side, so the sentence is
  not lost. That matters more here than in a compiled binding: an
  exception that escaped a `ctypes` callback would print a traceback and
  return **zero**, which the port would read as success. The adapter
  catches everything so that cannot happen.

Run it:

    PYTHONPATH=. python3 example/your_own_ephemeris.py
"""

from __future__ import annotations

import dataclasses
import json
from typing import Optional, Sequence

from teistro.catalogue import Ayanamsha
from teistro import (
    Body,
    EphemerisProvider,
    PositionAnswer,
    PositionQuery,
    Status,
    Teistro,
    TeistroError,
)


class TableEphemeris(EphemerisProvider):
    """An ephemeris backed by whatever you already have.

    This one is a two-body toy — a circular Sun and Moon — standing in
    for the real thing: a `.se1` reader, a JPL kernel, a database of
    precomputed rows, or a cache in front of any of them. What matters
    is the shape, not the arithmetic.
    """

    name = "table-ephemeris"
    version = "1.0.0"
    #: What identifies the data, not the code. A result's provenance
    #: carries it, so two runs against different data are distinguishable
    #: even when the code is identical.
    data_version = "demo-rows-2025a"
    bodies: Sequence[Body] = (Body.SUN, Body.MOON)
    #: Cover only what you have. A request outside this is refused before
    #: `positions` is ever called.
    jd_min = 2451545.0
    jd_max = 2469807.0

    def __init__(self, teistro: Optional[Teistro] = None) -> None:
        self.calls = 0
        self.cells = 0
        self.refusals = 0
        #: The one frame this provider computes in, as the port packs it.
        #: `None` means "answer whatever is asked", which is what a
        #: provider may say only when it really can.
        self.wanted_frame = (
            None if teistro is None else teistro.pack_frame(teistro.canonical_frame)
        )

    def positions(self, query: PositionQuery) -> Optional[PositionAnswer]:
        # **Check the frame first.** Answering at all asserts that the
        # answer is in the frame that was asked for; a provider that
        # computes only its own must say so by returning `None`, and the
        # SDK then asks again in `native_frame` and completes the rest.
        # This one computes tropical ecliptic longitudes and nothing else.
        if self.wanted_frame is not None and query.frame_bits != self.wanted_frame:
            self.refusals += 1
            return None

        self.calls += 1
        self.cells += query.cell_count

        # Cells run instants outermost: cell `i * len(bodies) + j` is
        # instant `i`, body `j`. Building the columns in that order is
        # the whole of the contract.
        lon: list[float] = []
        for jd in query.jds:
            days = jd - 2451545.0
            for body in query.bodies:
                rate = 0.9856 if body == Body.SUN else 13.1764
                start = 280.46 if body == Body.SUN else 218.32
                lon.append((start + rate * days) % 360.0)

        cells = query.cell_count
        return PositionAnswer(
            lon=lon,
            lat=[0.0] * cells,
            dist=[1.0] * cells,
            # Speeds are optional; a column left out is zeroes. Saying
            # `speeds = False` on the class would tell the SDK not to
            # expect them at all.
            lon_speed=[
                0.9856 if body == Body.SUN else 13.1764
                for _ in query.jds
                for body in query.bodies
            ],
        )


class Broken(TableEphemeris):
    """A provider that fails the way a real one does: with a sentence."""

    name = "broken"

    def positions(self, query: PositionQuery) -> Optional[PositionAnswer]:
        raise FileNotFoundError("ephemeris file de431.eph is not where the index says")


def reason(error: BaseException) -> str:
    """The sentence inside a refusal, whatever kind it is.

    A library refusal carries its hint as well, which belongs in front of
    a person and not in the middle of a demonstration.
    """
    return error.message if isinstance(error, TeistroError) else str(error)


def main() -> None:
    teistro = Teistro.open()

    # ── The happy path ────────────────────────────────────────────────
    provider = TableEphemeris()
    with teistro.context(profile="parashari-classical", provider=provider) as ctx:
        sky = ctx.positions(
            instants=[2451545.0 + day for day in range(7)],
            bodies=[Body.SUN, Body.MOON],
        )
        print(f"asked    {provider.calls} time(s) for {provider.cells} cells")
        print(f"answered {sky.cell_count} cells over {sky.instant_count} days")
        print(
            f"  sun  {sky.at(0, 0).longitude:8.4f}°"
            f" -> {sky.at(6, 0).longitude:8.4f}° in a week"
        )
        print(
            f"  moon {sky.at(0, 1).longitude:8.4f}°"
            f" -> {sky.at(6, 1).longitude:8.4f}° in a week"
        )
        # The provider's own name and data version are stamped on the
        # answer, which is how a stored chart says what computed it.
        stamp = json.dumps(sky.provenance_of["provider"], separators=(",", ":"))
        print(f"  stamped as {stamp}")

    # ── A body it never declared ──────────────────────────────────────
    provider = TableEphemeris()
    with teistro.context(profile="parashari-classical", provider=provider) as ctx:
        try:
            ctx.positions(instants=[2451545.0], bodies=[Body.SATURN])
        except Exception as error:  # noqa: BLE001 — the point is what it says
            print()
            print(f"refused  {reason(error)}")
            print(f"         and the provider was asked {provider.calls} times")

    # ── An instant outside its coverage ───────────────────────────────
    provider = TableEphemeris()
    with teistro.context(profile="parashari-classical", provider=provider) as ctx:
        try:
            ctx.positions(instants=[2200000.0], bodies=[Body.SUN])
        except Exception as error:  # noqa: BLE001
            print(f"refused  {reason(error)}")

    # ── A frame it does not compute ───────────────────────────────────
    # This provider computes tropical positions and declares no native
    # frame, so the canonical one is what it answers. Ask for a sidereal
    # zodiac and the SDK does the rest, naming every step it applied —
    # which is how an engine that knows nothing about the ayanamsha can
    # still serve a Vedic chart.
    provider = TableEphemeris(teistro)
    with teistro.context(profile="parashari-classical", provider=provider) as ctx:
        tropical = ctx.positions(instants=[2451545.0], bodies=[Body.SUN])
        sidereal = ctx.positions(
            instants=[2451545.0],
            bodies=[Body.SUN],
            frame=dataclasses.replace(
                teistro.canonical_frame, sidereal=True, ayanamsha=Ayanamsha.LAHIRI
            ),
        )
        steps = ", ".join(
            f"{step['name']}:{step['implementation']}"
            for step in sidereal.steps_applied
        )
        print()
        print(
            f"frames   the provider answered {tropical.at(0, 0).longitude:.4f}°"
            f" tropical; a sidereal request is"
            f" {sidereal.at(0, 0).longitude:.4f}°"
        )
        print(f"         it refused the frame {provider.refusals} time(s), and the")
        print(f"         SDK completed it: {steps}")

    # ── When the provider itself fails ────────────────────────────────
    with teistro.context(profile="parashari-classical", provider=Broken()) as ctx:
        try:
            ctx.positions(instants=[2451545.0], bodies=[Body.SUN])
        except FileNotFoundError as error:
            print()
            print(f"raised   {type(error).__name__}: {error}")
            print("         the exception itself crossed back, not just a code")

    # ── A refusal a user should see ───────────────────────────────────
    # Every refusal from the library carries a status a program can match
    # on and, where the boundary knows one, the field at fault and a hint
    # to act on. That is what to put in front of a person.
    with teistro.context(profile="parashari-classical") as ctx:
        try:
            ctx.key_id("graha.SUNN")
        except TeistroError as error:
            print()
            print(f"status   {error.status.key}")
            print(f"message  {error.message}")
            print(f"detail   {error.detail}")
            print(f"hint     {error.hint}")
            print(
                "         a program matches on `status`; a person reads the"
                " message and the hint"
            )


if __name__ == "__main__":
    main()
