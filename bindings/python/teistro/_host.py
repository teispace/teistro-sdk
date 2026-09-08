"""An ephemeris provider written in Python, bound into the port's vtable.

(`docs/02-architecture/07-binding-architecture.md`, "Ports across the
boundary".)

HAND-WRITTEN. The architecture puts the port adapter in the ergonomic
layer because every binding wraps its own callback mechanism: napi in
Node, `NativeCallable.isolateLocal` in Dart, and here a
`ctypes.CFUNCTYPE` trampoline, which the interpreter calls with the GIL
held. That is exactly the boundary's contract — one context, one thread
at a time — so the SDK calling back into Python inside a call this thread
made is the only way it is ever reached.

What the adapter does is small, because the port carries the machinery:
it describes the provider once into the capabilities struct, and turns
one call per grid into the columns the SDK allocated.

Two things here are not optional politeness:

- a trampoline is **kept on the object**, because a `ctypes` callback
  that is collected leaves the library holding a dangling function
  pointer;
- every trampoline catches everything, because a Python exception that
  escapes a `ctypes` callback prints a traceback and returns **zero**,
  which the port reads as success.
"""

from __future__ import annotations

import ctypes
from typing import Any, Optional, Sequence

from ._ffi import (
    VTABLE_ABI_VERSION,
    CapabilitiesFn,
    Frame,
    Observer,
    PositionsFn,
    TeistroLibrary,
    _CapabilitiesStruct,
    _PositionColumnsStruct,
    _PositionRequestStruct,
    _ProviderVtableStruct,
    frame_canonical,
    frame_pack,
)
from .catalogue import (
    Astronomy,
    Body,
    DistanceUnit,
    ProviderCode,
    SpeedModel,
    TimeScale,
)


class PositionQuery:
    """The grid a provider is asked to fill: one call for the whole of it,
    never a loop.

    Cell `i * len(bodies) + b` is body `b` at instant `i`, which is the
    order the answer's columns are read in.
    """

    def __init__(
        self,
        *,
        scale: TimeScale,
        frame_bits: int,
        speeds: bool,
        observer: Optional[Observer],
        jds: Sequence[float],
        bodies: Sequence[Body],
    ) -> None:
        self.scale = scale
        """The scale the instants are on."""
        self.frame_bits = frame_bits
        """The frame the positions are wanted in, packed."""
        self.speeds = speeds
        """Whether speeds are wanted."""
        self.observer = observer
        """The place a topocentric frame is seen from, `None` otherwise."""
        self.jds = jds
        """The instants, on `scale`."""
        self.bodies = bodies
        """The bodies, in the order the cells run."""

    @property
    def cell_count(self) -> int:
        """How many cells the answer must hold."""
        return len(self.jds) * len(self.bodies)


class PositionAnswer:
    """The columns a provider answers with.

    A column left out is zeroes, which is what a provider that computes no
    speeds means; `lon`, `lat`, `dist` and `status` must be as long as
    `PositionQuery.cell_count`.

    **Answering at all asserts the answer is in the frame that was asked
    for.** `frame_bits` left out means "the frame you asked for", not
    "my own": a provider that computes in one frame must compare
    `PositionQuery.frame_bits` with its own and return `None` instead,
    and the SDK will then ask again in `EphemerisProvider.native_frame`
    and complete the rest — applying the ayanamsha, shifting the zodiac,
    and stamping each step it took. Answering the wrong frame silently is
    the one mistake this class makes easy, so it is named here.
    """

    def __init__(
        self,
        *,
        lon: Sequence[float],
        lat: Sequence[float],
        dist: Sequence[float],
        lon_speed: Optional[Sequence[float]] = None,
        lat_speed: Optional[Sequence[float]] = None,
        dist_speed: Optional[Sequence[float]] = None,
        status: Optional[Sequence[int]] = None,
        source: Optional[Sequence[int]] = None,
        frame_bits: Optional[int] = None,
    ) -> None:
        self.lon = lon
        self.lat = lat
        self.dist = dist
        self.lon_speed = lon_speed
        self.lat_speed = lat_speed
        self.dist_speed = dist_speed
        self.status = status
        self.source = source
        self.frame_bits = frame_bits


class EphemerisProvider:
    """An ephemeris the SDK can drive, written in Python.

    Everything but `name`, `bodies` and `positions` has a default, because
    a provider that answers the canonical frame with apparent geocentric
    positions is the common case and should not have to say so.

    ```python
    class StraightLine(EphemerisProvider):
        name = "straight-line"
        bodies = [Body.SUN]

        def positions(self, query):
            zeros = [0.0] * query.cell_count
            return PositionAnswer(lon=zeros, lat=zeros, dist=zeros)
    ```
    """

    #: What the provider is, stamped in every result's provenance.
    name: str = ""
    #: The bodies it answers. A request for another is refused by name
    #: before the call reaches `positions`.
    bodies: Sequence[Body] = ()
    #: Its version; empty by default.
    version: str = ""
    #: What identifies its data, an ephemeris file's edition.
    data_version: str = ""
    #: The first Julian day it covers; year 0 by default.
    jd_min: float = 1721057.5
    #: The last Julian day it covers; year 3000 by default.
    jd_max: float = 2816787.5
    #: The frame it returns natively; the SDK's canonical frame by
    #: default. A request in another frame reaches `positions` first, and
    #: answering `None` there has the SDK ask again in this one.
    native_frame: Optional[Frame] = None
    #: Whether it computes speeds.
    speeds: bool = True
    #: Whether identical requests give identical bits. A provider that is
    #: not deterministic must say so, because the conformance contract
    #: rests on it (ADR-0022).
    deterministic: bool = True
    #: What its distances are measured in.
    distance_unit: DistanceUnit = DistanceUnit.ASTRONOMICAL_UNITS
    #: What its speeds are.
    speed_model: SpeedModel = SpeedModel.DERIVATIVE
    #: Whether it is modern astronomy or a classical text's model.
    astronomy: Astronomy = Astronomy.MODERN

    def positions(self, query: PositionQuery) -> Optional[PositionAnswer]:
        """The positions for a whole grid, or `None` for "not in that
        frame", in which case the SDK asks again in `native_frame` and
        completes the rest itself, stamping every step it applied.

        Compare `query.frame_bits` with the frame you compute in before
        answering: returning an answer says it is in the frame that was
        asked for (`PositionAnswer`). `example/your_own_ephemeris.py`
        does exactly that.
        """
        raise NotImplementedError


class HostProvider:
    """An `EphemerisProvider` bound into the port's vtable.

    The vtable, the capability strings and the trampolines live as long as
    this object, which the context that uses them owns; `close` releases
    them, and the context is closed first.
    """

    def __init__(self, lib: TeistroLibrary, provider: EphemerisProvider) -> None:
        if not provider.name:
            raise ValueError(
                "a provider must have a name, which every result is stamped with"
            )
        if not provider.bodies:
            raise ValueError("a provider must answer at least one body")
        self.provider = provider
        self._lib = lib
        #: What the provider raised, kept for the layer above to re-raise:
        #: only a code crosses the C boundary, and a provider written in
        #: Python has more to say than a code.
        self.raised: Optional[BaseException] = None
        self._closed = False
        # Held so the interpreter cannot collect a function pointer the
        # library still has.
        self._capabilities_fn: Any = CapabilitiesFn(self._fill_capabilities)
        self._positions_fn: Any = PositionsFn(self._fill_positions)
        self._capabilities = self._describe()
        self._vtable = _ProviderVtableStruct()
        self._vtable.struct_size = ctypes.sizeof(_ProviderVtableStruct)
        self._vtable.abi_version = VTABLE_ABI_VERSION
        self._vtable.capabilities = self._capabilities_fn
        self._vtable.positions = self._positions_fn

    @property
    def vtable(self) -> Any:
        """The vtable pointer the SDK drives the provider through."""
        return ctypes.byref(self._vtable)

    def _describe(self) -> _CapabilitiesStruct:
        """The capabilities, described once: the SDK reads them whenever it
        asks, and nothing about a provider changes while it is bound.
        """
        provider = self.provider
        # Kept on the object: the struct holds pointers into these.
        self._name = provider.name.encode("utf-8")
        self._version = provider.version.encode("utf-8")
        self._data_version = provider.data_version.encode("utf-8")
        self._bodies: Any = (ctypes.c_uint16 * len(provider.bodies))(
            *(int(body) for body in provider.bodies)
        )
        frame = provider.native_frame or frame_canonical(self._lib)
        out = _CapabilitiesStruct()
        out.struct_size = ctypes.sizeof(_CapabilitiesStruct)
        out.speeds = 1 if provider.speeds else 0
        out.deterministic = 1 if provider.deterministic else 0
        out.tier = 0
        out.distance_unit = int(provider.distance_unit)
        out.speed_model = int(provider.speed_model)
        out.astronomy = int(provider.astronomy)
        out.name = self._name
        out.version = self._version
        out.data_version = self._data_version
        out.jd_min = provider.jd_min
        out.jd_max = provider.jd_max
        out.bodies = ctypes.cast(self._bodies, ctypes.POINTER(ctypes.c_uint16))
        out.body_count = len(provider.bodies)
        out.native_frame_bits = frame_pack(self._lib, frame)
        out.overrides = 0
        out.ayanamsha_count = 0
        out.hash_count = 0
        return out

    def _fill_capabilities(self, user_data: Any, out: Any) -> int:
        if not out:
            return int(ProviderCode.INVALID)
        try:
            # The SDK reads what it asked for out of the struct described
            # once, so a struct of another size is still answered.
            size = out.contents.struct_size
            ctypes.memmove(
                out,
                ctypes.byref(self._capabilities),
                min(size, ctypes.sizeof(_CapabilitiesStruct)),
            )
            out.contents.struct_size = size
            return int(ProviderCode.OK)
        except BaseException as error:  # noqa: BLE001 — nothing may escape
            self.raised = error
            return int(ProviderCode.REFUSED)

    def _fill_positions(self, user_data: Any, request: Any, out: Any) -> int:
        if not request or not out:
            return int(ProviderCode.INVALID)
        try:
            return self._answer(request.contents, out.contents)
        except BaseException as error:  # noqa: BLE001 — nothing may escape
            # Only a code crosses; the sentence is kept for the layer above.
            self.raised = error
            return int(ProviderCode.REFUSED)

    def _answer(
        self, request: _PositionRequestStruct, out: _PositionColumnsStruct
    ) -> int:
        query = self._read(request)
        refusal = self._validate(query)
        if refusal is not None:
            return refusal
        cells = query.cell_count
        if out.capacity < cells:
            return int(ProviderCode.INVALID)
        answer = self.provider.positions(query)
        # Nothing means "not in that frame"; the SDK asks again in ours.
        if answer is None:
            return int(ProviderCode.UNSUPPORTED)
        # Every column the provider supplied, not only the three it must:
        # a speed column of the wrong length silently padded with zeroes
        # is a wrong answer, and a wrong answer is worse than a refusal.
        for name, column in (
            ("lon", answer.lon),
            ("lat", answer.lat),
            ("dist", answer.dist),
            ("lon_speed", answer.lon_speed),
            ("lat_speed", answer.lat_speed),
            ("dist_speed", answer.dist_speed),
        ):
            if column is not None and len(column) != cells:
                self.raised = ValueError(
                    f"the provider returned {len(column)} values in `{name}` "
                    f"for {cells} cells"
                )
                return int(ProviderCode.REFUSED)
        out.frame_bits = (
            query.frame_bits if answer.frame_bits is None else answer.frame_bits
        )
        for column, values in (
            (out.lon, answer.lon),
            (out.lat, answer.lat),
            (out.dist, answer.dist),
            (out.lon_speed, answer.lon_speed),
            (out.lat_speed, answer.lat_speed),
            (out.dist_speed, answer.dist_speed),
        ):
            _write(column, values, cells)
        _write(out.status, answer.status, cells)
        _write(out.source, answer.source, cells)
        return int(ProviderCode.OK)

    def _read(self, request: _PositionRequestStruct) -> PositionQuery:
        """The request as a Python value.

        The instants and the ids are the SDK's memory, valid for the length
        of the call, so they are copied.
        """
        return PositionQuery(
            scale=TimeScale(request.scale),
            frame_bits=request.frame_bits,
            speeds=request.speeds != 0,
            observer=(
                None
                if request.has_observer == 0
                else Observer._of(request.observer)
            ),
            jds=[request.jds[i] for i in range(request.jd_count)],
            bodies=[Body(request.bodies[i]) for i in range(request.body_count)],
        )

    def _validate(self, query: PositionQuery) -> Optional[int]:
        """What this side checks before the provider is asked.

        Only the coverage span: a topocentric frame without an observer, a
        body the provider never declared and an instant that is not a
        number are all refused by the port itself, on the SDK's side of
        the boundary, where the sentence survives into a `TeistroError`
        that names what is missing. Checking them again here would be a
        second copy of the same policy, saying it worse.
        """
        for jd in query.jds:
            if jd < self.provider.jd_min or jd > self.provider.jd_max:
                self.raised = ValueError(
                    f"the instant {jd} is outside the provider's coverage "
                    f"({self.provider.jd_min} to {self.provider.jd_max})"
                )
                return int(ProviderCode.OUT_OF_RANGE)
        return None

    def close(self) -> None:
        """Releases the trampolines and the memory the vtable points at.

        The context that used them must be closed first, which
        `Context.close` sees to: dropping a trampoline the library still
        held would leave it with a dangling function pointer. Closing
        twice is allowed.
        """
        if self._closed:
            return
        self._closed = True
        # Nothing here is freed by hand — the interpreter owns all of it —
        # but letting go of the references is what makes that happen, and
        # it is also what stops a callback after the context is gone.
        self._capabilities_fn = None
        self._positions_fn = None
        self._bodies = None

    @property
    def closed(self) -> bool:
        """Whether the vtable has been released."""
        return self._closed


def _write(column: Any, values: Optional[Sequence[float]], cells: int) -> None:
    """Fills one of the SDK's columns, with zeroes where a provider left
    the column out."""
    if not column:
        return
    for index in range(cells):
        column[index] = 0 if values is None or index >= len(values) else values[index]
