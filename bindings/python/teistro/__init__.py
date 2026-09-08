"""The Teistro SDK for Python: the layer a consumer uses.

HAND-WRITTEN, and thin on purpose. Everything beneath it is generated
from the API description: the `ctypes` declarations and the value classes
(`_ffi.py`), the catalogue's enums (`catalogue.py`) and the result-blob
decoders (`_blob.py`). What this file adds is what a generator cannot
know: where the shared library is, defaults, JSON in and out, and the
small conveniences a decoded result deserves.

```python
from teistro import Teistro, Body

teistro = Teistro.open()
with teistro.context(test_provider=True) as sky:
    sun = sky.positions(instants=[2451545.0], bodies=[Body.SUN]).at(0, 0)
    print(sun.longitude)
```
"""

from __future__ import annotations

import ctypes
import ctypes.util
import json
import os
import sys
from dataclasses import dataclass
from pathlib import Path
from types import TracebackType
from typing import Any, Iterator, Mapping, Optional, Sequence

from . import messages as intl
from ._blob import (
    BlobError,
    Charts,
    IntlRender,
    Positions,
    decode_charts,
    decode_intl_render,
    decode_positions,
)
from ._ffi import (
    CONTEXT_TEST_PROVIDER,
    GENERATED_ABI_VERSION,
    GENERATED_SDK_VERSION,
    Altitude,
    CalendarDate,
    ChartRequest,
    CivilDateTime,
    CivilTime,
    ContextOptions,
    DeltaT,
    Error,
    Frame,
    Hash,
    IntlLoaded,
    Latitude,
    Longitude,
    Observer,
    PositionRequest,
    TeistroContext,
    TeistroError,
    TeistroLibrary,
    TimeConversion,
    ZoneResolution,
    ZoneSpec,
    abi_version,
    build_info,
    calendar_fixed_of_jd,
    calendar_jd_of_fixed,
    catalogue_version,
    default_profile,
    frame_canonical,
    frame_pack,
    frame_unpack,
    sdk_version,
)
from ._host import (
    EphemerisProvider,
    HostProvider,
    PositionAnswer,
    PositionQuery,
)
from ._install import (
    InstallError,
    host_platform,
    install,
    installed_library,
    library_file_name,
)
from ._prebuilt import PREBUILT_VERSION
from .catalogue import (
    Ayanamsha,
    Body,
    Calendar,
    Centre,
    ChartKind,
    Graha,
    HouseSystem,
    Vara,
    Coordinates,
    Era,
    Resolution,
    Scale,
    Status,
    TimeScale,
    ZoneKind,
)

__all__ = [
    "Altitude",
    "Ayanamsha",
    "BlobError",
    "Bhava",
    "Body",
    "BuildInfo",
    "Calendar",
    "CalendarDate",
    "Centre",
    "Chart",
    "ChartBatch",
    "ChartKind",
    "Charts",
    "CivilDateTime",
    "CivilTime",
    "Context",
    "Coordinates",
    "DeltaT",
    "EphemerisProvider",
    "Era",
    "Error",
    "Frame",
    "Hash",
    "InstallError",
    "IntlLoaded",
    "IntlRender",
    "Latitude",
    "Longitude",
    "Observer",
    "PlacedGraha",
    "Placement",
    "PositionAnswer",
    "PositionQuery",
    "Positions",
    "Resolution",
    "Scale",
    "Status",
    "Teistro",
    "TeistroError",
    "TimeConversion",
    "TimeScale",
    "ZoneKind",
    "ZoneResolution",
    "ZoneSpec",
    "Cell",
    "PREBUILT_VERSION",
    "at",
    "date",
    "fixed_zone",
    "host_platform",
    "iana_zone",
    "install",
    "installed_library",
    "intl",
    "library_file_name",
    "local_mean_zone",
    "when_unknown",
]

#: The environment variable that names the shared library, which wins over
#: every other place it is looked for.
PATH_VARIABLE = "TEISTRO_LIBRARY"


@dataclass(frozen=True)
class BuildInfo:
    """What an open library says about its own build.

    Written when the library is compiled, so it costs nothing to ask and
    cannot disagree with the library it describes.
    """

    sdk: str
    abi: int
    catalogue: int
    commit: str
    dirty: bool
    profile: str
    target: str
    optimised: bool
    debug_assertions: bool
    sanitizer: str
    compiler: str

    @classmethod
    def of(cls, document: str) -> BuildInfo:
        """The build as `ts_build_info` writes it."""
        try:
            found = json.loads(document)
        except ValueError as error:
            raise TeistroError(
                Status.INTERNAL,
                f"the library did not describe its build: {error}",
            ) from error

        def text(key: str) -> str:
            value = found.get(key)
            return "" if value is None else str(value)

        def number(key: str) -> int:
            value = found.get(key)
            return int(value) if isinstance(value, (int, float)) else 0

        return cls(
            sdk=text("sdk"),
            abi=number("abi"),
            catalogue=number("catalogue"),
            commit=text("commit"),
            dirty=found.get("dirty") is True,
            profile=text("profile"),
            target=text("target"),
            optimised=found.get("optimised") is True,
            debug_assertions=found.get("debug_assertions") is True,
            sanitizer=text("sanitizer"),
            compiler=text("compiler"),
        )


def refuse_build(info: BuildInfo, *, named: bool) -> Optional[str]:
    """Why this library may not be loaded, or `None` when it may.

    The language half and the native half must be from the same build
    (`02-architecture/07-binding-architecture.md`, "Loading and
    identity"): another ABI or version is a different API, a sanitizer
    build is not what a consumer meant to run, and an unoptimised build is
    refused only when it was searched out rather than named outright.
    """
    if info.abi != GENERATED_ABI_VERSION:
        return (
            f"the library implements ABI {info.abi} and this package was "
            f"generated for ABI {GENERATED_ABI_VERSION}"
        )
    if info.sdk != GENERATED_SDK_VERSION:
        return (
            f"the library is SDK {info.sdk} and this package was generated "
            f"for {GENERATED_SDK_VERSION}"
        )
    if info.sanitizer:
        return f"the library is a {info.sanitizer} build, which is not for use"
    if not named and not info.optimised:
        return (
            "the library found is an unoptimised build; name it outright with "
            f"${PATH_VARIABLE} or Teistro.open(path=...) to use it anyway"
        )
    return None


class Teistro:
    """The SDK's shared library, opened once and shared by every context.

    `open` finds it; `context` builds a context on it; the static calls of
    the C ABI are its properties and methods, so nothing needs the
    generated layer unless you want it, and `library` hands that over when
    you do.
    """

    def __init__(self, library: TeistroLibrary, build: BuildInfo) -> None:
        self.library = library
        """The generated declarations, for a call this layer does not wrap."""
        self.build = build
        """What the open library says about its own build."""

    @classmethod
    def open(cls, path: Optional[str | Path] = None) -> Teistro:
        """Opens the shared library and checks that it is the build these
        declarations were generated from.

        `path` names the library outright. Without one, the SDK looks at
        `$TEISTRO_LIBRARY`, then in this package's own `_lib` directory,
        then at what `teistro-install` wrote, then in the workspace's
        `target/release` and `target/debug`, and finally asks the
        platform's loader for the bare name.

        Raises `TeistroError` with `Status.UNSUPPORTED` when the library is
        not that build, and `FileNotFoundError` naming every place it
        looked when there is nothing to open.
        """
        named = path is not None
        opened = ctypes.CDLL(str(path)) if named else _search()
        library = TeistroLibrary(opened)
        build = BuildInfo.of(build_info(library))
        refusal = refuse_build(build, named=named)
        if refusal is not None:
            raise TeistroError(Status.UNSUPPORTED, refusal)
        return cls(library, build)

    @property
    def abi(self) -> int:
        """The ABI version the open library implements."""
        return abi_version(self.library)

    @property
    def version(self) -> str:
        """The SDK version the open library is."""
        return sdk_version(self.library)

    @property
    def catalogue(self) -> int:
        """The catalogue schema version every result's provenance stamps."""
        return catalogue_version(self.library)

    @property
    def default_profile_id(self) -> str:
        """The profile a context uses when its options name none."""
        return default_profile(self.library)

    @property
    def canonical_frame(self) -> Frame:
        """The frame the SDK computes in unless another is asked for."""
        return frame_canonical(self.library)

    def pack_frame(self, frame: Frame) -> int:
        """A frame as the bits the port carries it as."""
        return frame_pack(self.library, frame)

    def unpack_frame(self, bits: int) -> Frame:
        """The frame those bits stand for."""
        return frame_unpack(self.library, bits)

    def julian_day_of_fixed(self, fixed: int) -> float:
        """The Julian day at noon of a fixed day number."""
        return calendar_jd_of_fixed(self.library, fixed)

    def fixed_of_julian_day(self, jd: float) -> tuple[int, float]:
        """The fixed day a Julian day falls in, and the fraction into it."""
        found = calendar_fixed_of_jd(self.library, jd)
        return found.value, found.fraction

    def context(
        self,
        *,
        profile: Optional[str] = None,
        settings: Optional[Mapping[str, object]] = None,
        settings_json: Optional[str] = None,
        locale: Optional[str] = None,
        provider: Optional[EphemerisProvider] = None,
        test_provider: bool = False,
    ) -> Context:
        """A context: settings, a locale and an ephemeris.

        `settings` is a patch over the profile, as a mapping — the shape
        the Node and Dart bindings take, so one example reads in all
        three. `settings_json` takes the same patch already serialised,
        for a caller who has the document rather than the mapping; giving
        both is refused rather than one silently winning.

        `provider` binds an ephemeris written in Python;
        `test_provider=True` selects the analytic one the SDK carries, and
        neither leaves the context without an ephemeris, so a request for
        positions is refused with `Status.CAPABILITY`.
        """
        if settings is not None and settings_json is not None:
            raise ValueError(
                "settings and settings_json are the same patch twice; "
                "give one of them"
            )
        if settings is not None:
            settings_json = json.dumps(settings, separators=(",", ":"))
        host = None if provider is None else HostProvider(self.library, provider)
        options = ContextOptions(
            flags=CONTEXT_TEST_PROVIDER if test_provider else 0,
            profile=profile,
            settings_json=settings_json,
            locale=locale,
        )
        inner = TeistroContext._new(
            self.library,
            options,
            None if host is None else host.vtable,
        )
        return Context(self, inner, host)


def _candidates() -> list[str]:
    """Every place the library is looked for, in order."""
    here = Path(__file__).resolve().parent
    workspace = here.parents[2] if len(here.parents) > 2 else here
    name = library_file_name()
    found = [
        os.environ.get(PATH_VARIABLE, ""),
        # Where a per-platform wheel would put it, so that shipping one
        # later is a packaging change and not a change here.
        str(here / "_lib" / name),
        str(installed_library()),
        str(workspace / "target" / "release" / name),
        str(workspace / "target" / "debug" / name),
    ]
    return [candidate for candidate in found if candidate]


def _search() -> ctypes.CDLL:
    """Opens the first library that is where it might be."""
    looked = _candidates()
    for candidate in looked:
        if Path(candidate).is_file():
            return ctypes.CDLL(candidate)
    # The platform's own loader, which finds one installed system-wide.
    bare = ctypes.util.find_library("teistro_ffi")
    if bare is not None:
        return ctypes.CDLL(bare)
    raise FileNotFoundError(
        "no Teistro library found. Looked at:\n  "
        + "\n  ".join(looked)
        + "\nRun `teistro-install`, or build it with "
        "`cargo build --release -p teistro-ffi`, or set "
        f"${PATH_VARIABLE}."
    )


@dataclass(frozen=True)
class Cell:
    """One body at one instant, as the positions grid holds it."""

    longitude: float
    latitude: float
    distance: float
    longitude_speed: float
    latitude_speed: float
    distance_speed: float
    status: int
    source: int


class Context:
    """A context, and everything a consumer asks of one.

    A context frees its native memory when it is collected, so `close` is
    the explicit form rather than the only one (ADR-0007); `with` is the
    idiomatic one.
    """

    def __init__(
        self,
        teistro: Teistro,
        inner: TeistroContext,
        host: Optional[HostProvider] = None,
    ) -> None:
        self.teistro = teistro
        """The library this context was built on."""
        self.inner = inner
        """The generated context, for a call this layer does not wrap."""
        self._host = host
        self.messages = intl.Messages(_Renderer(self))
        """The typed accessors: every message of the SDK, by its key."""

    # ── The context itself ────────────────────────────────────────────

    @property
    def profile(self) -> str:
        """The id of the profile the settings came from."""
        return self.inner.profile()

    @property
    def settings_json(self) -> str:
        """The resolved settings as their canonical JSON document."""
        return self.inner.settings_json()

    @property
    def settings(self) -> Any:
        """The resolved settings, parsed."""
        return json.loads(self.settings_json)

    @property
    def settings_hash(self) -> str:
        """The settings hash, as the hexadecimal every result stamps."""
        return self.inner.settings_hash().bytes.hex()

    @property
    def locale(self) -> str:
        """The locale tag messages are rendered in."""
        return self.inner.intl_locale()

    @locale.setter
    def locale(self, tag: str) -> None:
        self.inner.intl_set_locale(tag)

    @property
    def last_error(self) -> Error:
        """The outcome of the last call on the context."""
        return self.inner.last_error()

    @property
    def provider(self) -> Optional[EphemerisProvider]:
        """The ephemeris written in Python this context was given."""
        return None if self._host is None else self._host.provider

    def close(self) -> None:
        """Frees the context, and then whatever its provider held."""
        self.inner.close()
        if self._host is not None:
            self._host.close()

    def __enter__(self) -> Context:
        return self

    def __exit__(
        self,
        kind: Optional[type[BaseException]],
        value: Optional[BaseException],
        traceback: Optional[TracebackType],
    ) -> None:
        self.close()

    # ── The calendars ─────────────────────────────────────────────────

    def date_of(self, calendar: Calendar, fixed: int) -> CalendarDate:
        """The date a fixed day number is, in a calendar."""
        return self.inner.calendar_from_fixed(calendar, fixed)

    def fixed_of(self, date: CalendarDate) -> int:
        """The fixed day number a date is."""
        return self.inner.calendar_to_fixed(date)

    def convert(self, date: CalendarDate, into: Calendar) -> CalendarDate:
        """The same day in another calendar."""
        return self.inner.calendar_convert(date, into)

    def weekday_of(self, date: CalendarDate) -> int:
        """The weekday of a date as its ISO number: Monday `1`, Sunday `7`.

        Not the catalogue's `Vara`, which counts from Sunday: a vara is
        `weekday_of(day) % 7`, and the panchanga example does exactly
        that.
        """
        return self.inner.calendar_weekday(date)

    def month_length(self, calendar: Calendar, year: int, month: int) -> int:
        """How many days a month has."""
        return self.inner.calendar_month_length(calendar, year, month)

    def is_leap(self, calendar: Calendar, year: int) -> bool:
        """Whether a year is a leap year in a calendar."""
        return self.inner.calendar_is_leap(calendar, year) != 0

    # ── Time ──────────────────────────────────────────────────────────

    def resolve(self, civil: CivilDateTime, zone: ZoneSpec) -> ZoneResolution:
        """The instant a civil date and time in a zone stands for."""
        return self.inner.time_resolve(civil, zone)

    def civil_of(
        self, jd_utc: float, zone: ZoneSpec, calendar: Calendar
    ) -> tuple[CivilDateTime, ZoneResolution]:
        """The civil date and time an instant is, in a zone."""
        found = self.inner.time_civil(jd_utc, zone, calendar)
        return found.civil, found.resolution

    def convert_time(self, jd: float, scale: Scale, into: Scale) -> TimeConversion:
        """The same instant on another time scale.

        `Scale` and not `TimeScale`: the time layer knows UTC as well as
        the two the port carries, and the two enums agree on the ids they
        share, so passing the wrong one would convert from the wrong
        scale without any complaint.
        """
        return self.inner.time_convert(jd, scale, into)

    def delta_t(self, jd_ut1: float) -> DeltaT:
        """TT less UT1 at an instant, and where the value came from."""
        return self.inner.time_delta_t(jd_ut1)

    # ── Keys ──────────────────────────────────────────────────────────

    def key_id(self, key: str) -> int:
        """The catalogue id a key stands for."""
        return self.inner.key_parse(key)

    def key_name(self, identifier: int) -> str:
        """The key an id stands for."""
        return self.inner.key_name(identifier)

    # ── The locale engine ─────────────────────────────────────────────

    def render(self, key: str, params: Optional[Mapping[str, object]] = None) -> IntlRender:
        """A message rendered in the context's locale."""
        return decode_intl_render(
            self.inner.intl_render(key, json.dumps({} if params is None else params))
        )

    def has(self, key: str) -> bool:
        """Whether the current locale carries a message."""
        return self.inner.intl_has(key) != 0

    def entity(self, key: str) -> intl.EntityForms:
        """A catalogued entity's forms in the current locale."""
        return intl.EntityForms.of(self.inner.intl_entity(key))

    def transliterate(self, text: str, source: str = "Deva", into: str = "Latn") -> str:
        """Text from one script into another."""
        return self.inner.intl_transliterate(text, source, into)

    def load_pack(self, data: bytes) -> IntlLoaded:
        """Loads a locale pack's bytes into the engine."""
        return self.inner.intl_load_pack(data)

    # ── Positions ─────────────────────────────────────────────────────

    def positions(
        self,
        *,
        instants: Sequence[float],
        bodies: Sequence[Body],
        scale: TimeScale = TimeScale.UT1,
        frame: Optional[Frame] = None,
        speeds: bool = True,
        observer: Optional[Observer] = None,
    ) -> PositionGrid:
        """The positions of a grid of bodies at a grid of instants.

        The cells run instants outermost: cell `i * len(bodies) + j` is
        instant `i`, body `j`, which is what `at` reads.
        """
        if not instants:
            raise ValueError("a request needs at least one instant")
        if not bodies:
            raise ValueError("a request needs at least one body")
        bits = self.teistro.pack_frame(
            self.teistro.canonical_frame if frame is None else frame
        )
        request = PositionRequest(
            scale=scale,
            frame_bits=bits,
            speeds=speeds,
            jds=list(instants),
            bodies=list(bodies),
            observer=observer,
        )
        return PositionGrid(
            decode_positions(self._through_provider(lambda: self.inner.positions(request)))
        )

    def found(
        self,
        *,
        instant: float,
        place: Observer,
        utc_offset_seconds: int,
        kind: ChartKind = ChartKind.NATAL,
    ) -> Chart:
        """Founds a chart at an instant and a place.

        Everything but this is the context's settings, so two charts
        founded under one context are comparable and the settings hash
        says why. The clock is here because nothing else knows it: a
        chart's day runs from a local sunrise and its date is a civil
        date, and a longitude gives local *mean* time rather than a civil
        offset.

        A profile whose frame is topocentric needs a provider that
        answers topocentric natively; the completion's centre step is
        Phase 3's (`03-design/chart-at-the-boundary.md` §8).
        """
        return self.found_many(
            instants=[instant],
            place=place,
            utc_offset_seconds=utc_offset_seconds,
            kind=kind,
        ).at(0)

    def found_many(
        self,
        *,
        instants: Sequence[float],
        place: Observer,
        utc_offset_seconds: int,
        kind: ChartKind = ChartKind.NATAL,
    ) -> ChartBatch:
        """Founds a chart at each of many instants, at one place, in one
        crossing.

        The founder shares the settings and the solar model across the
        batch, so a hundred instants cost one setup rather than a hundred
        — which is what a rectification pass wants. A batch of none is an
        empty result rather than an error.
        """
        request = ChartRequest(
            kind=kind,
            instants=list(instants),
            latitude_deg=place.latitude_deg,
            longitude_deg=place.longitude_deg,
            altitude_m=place.altitude_m,
            utc_offset_seconds=utc_offset_seconds,
        )
        return ChartBatch(
            decode_charts(self._through_provider(lambda: self.inner.chart_found(request)))
        )

    def _through_provider(self, call: Any) -> Any:
        """Runs a call that may reach a provider written in Python, and
        re-raises what the provider raised.

        Only a code crosses the C boundary, so without this a provider's
        own message would be lost and the caller would see the port's
        summary of it instead.
        """
        if self._host is None:
            return call()
        self._host.raised = None
        try:
            return call()
        except TeistroError as refusal:
            raised = self._host.raised
            if raised is None:
                raise
            # The provider's own exception, with the boundary's refusal
            # kept as its cause: a caller catches the type it wrote, and
            # a traceback still shows what the port made of it.
            raise raised from refusal

@dataclass(frozen=True)
class Placement:
    """Where a graha sits in a set of bhavas."""

    bhava: int
    """The bhava, 1 to 12."""

    method: HouseSystem
    """The house system that produced it."""

    through: float
    """How far through the bhava it is, 0 to 1."""

    from_madhya_deg: float
    """Its distance from the bhava's centre, degrees."""


@dataclass(frozen=True)
class PlacedGraha:
    """One graha of a chart, read out of the batch's columns."""

    graha: Graha
    """Which graha."""

    longitude_deg: float
    """Its longitude in the chart's zodiac, degrees."""

    tropical_deg: float
    """Its tropical longitude, degrees."""

    latitude_deg: float
    """Its ecliptic latitude, degrees."""

    distance_au: float
    """Its distance, astronomical units."""

    speed_deg_per_day: float
    """Its longitude speed, degrees per day."""

    house: Placement
    """The bhava for "which house is it in"."""

    placement: Placement
    """The bhava of the chart's chalit, which is a different question."""

    @property
    def retrograde(self) -> bool:
        """Whether its longitude speed is negative."""
        return self.speed_deg_per_day < 0


@dataclass(frozen=True)
class Bhava:
    """One of the twelve bhavas."""

    madhya_deg: float
    """The bhava's centre, degrees."""

    sandhi_deg: float
    """The bhava's opening cusp, degrees."""


class Chart:
    """One founded chart: a view over its batch, not a copy.

    Every property reads the batch's columns at this chart's index, so a
    chart costs nothing until something is asked of it, and holding one
    holds the whole blob rather than a copy of a slice of it.
    """

    def __init__(self, batch: "ChartBatch", index: int) -> None:
        self.batch = batch
        """The batch this chart belongs to."""
        self.index = index
        """Where in that batch it sits."""

    @property
    def instant(self) -> float:
        """The instant the chart is cast for, as a Julian day (UTC)."""
        return self.batch.decoded.cast.instant[self.index]

    @property
    def lagna_deg(self) -> float:
        """The lagna at the instant, in the chart's zodiac, degrees."""
        return self.batch.decoded.cast.lagna_deg[self.index]

    @property
    def day_lagna_deg(self) -> float:
        """The lagna at the sunrise that opened the day, degrees."""
        return self.batch.decoded.cast.day_lagna_deg[self.index]

    @property
    def ayanamsha_offset_deg(self) -> float:
        """The ayanamsha applied at this instant, degrees; zero if tropical."""
        return self.batch.decoded.cast.ayanamsha_offset_deg[self.index]

    @property
    def kind(self) -> ChartKind:
        """What kind of chart this is."""
        return self.batch.kind

    @property
    def vara(self) -> Vara:
        """The weekday the chart's day carries."""
        return Vara(self.batch.decoded.day.vara[self.index])

    @property
    def sunrise(self) -> float:
        """The sunrise that opened the chart's day, as a Julian day (UTC)."""
        return self.batch.decoded.day.sunrise[self.index]

    @property
    def hora_lord(self) -> Graha:
        """The graha that rules the hora holding the instant."""
        return Graha(self.batch.decoded.timing.hora_lord[self.index])

    @property
    def grahas(self) -> list[PlacedGraha]:
        """The grahas, in the catalogue's order, one object each.

        The columns underneath are views over the blob's bytes, charts
        outermost; this reads this chart's stride out of them into the
        shape an application wants, which is a row.
        """
        columns = self.batch.decoded.grahas
        count = self.batch.decoded.graha_count
        base = self.index * count
        return [
            PlacedGraha(
                graha=Graha(columns.graha[i]),
                longitude_deg=columns.longitude_deg[i],
                tropical_deg=columns.tropical_deg[i],
                latitude_deg=columns.latitude_deg[i],
                distance_au=columns.distance_au[i],
                speed_deg_per_day=columns.speed_deg_per_day[i],
                house=Placement(
                    bhava=columns.house_bhava[i],
                    method=HouseSystem(columns.house_method[i]),
                    through=columns.house_through[i],
                    from_madhya_deg=columns.house_from_madhya_deg[i],
                ),
                placement=Placement(
                    bhava=columns.placement_bhava[i],
                    method=HouseSystem(columns.placement_method[i]),
                    through=columns.placement_through[i],
                    from_madhya_deg=columns.placement_from_madhya_deg[i],
                ),
            )
            for i in range(base, base + count)
        ]

    @property
    def houses(self) -> list[Bhava]:
        """The twelve bhavas for "which house is it in", first to twelfth."""
        return self._bhavas(self.batch.decoded.houses)

    @property
    def chalit(self) -> list[Bhava]:
        """The twelve bhavas of the chart's chalit."""
        return self._bhavas(self.batch.decoded.chalit)

    def _bhavas(self, columns: Any) -> list[Bhava]:
        base = self.index * 12
        return [
            Bhava(madhya_deg=columns.madhya_deg[i], sandhi_deg=columns.sandhi_deg[i])
            for i in range(base, base + 12)
        ]


class ChartBatch:
    """A batch of founded charts at one place, read one chart at a time."""

    def __init__(self, decoded: Charts) -> None:
        self.decoded = decoded
        """The blob as its generated decoder read it."""

    def __len__(self) -> int:
        """How many charts the batch holds."""
        return self.decoded.chart_count

    def at(self, index: int) -> Chart:
        """One chart of the batch, by index."""
        if not 0 <= index < len(self):
            raise IndexError(f"chart {index} is outside a batch of {len(self)}")
        return Chart(self, index)

    def __getitem__(self, index: int) -> Chart:
        """The same as `at`, so a batch indexes as well as iterates."""
        return self.at(index)

    def __iter__(self) -> Iterator[Chart]:
        """Every chart, in the order the instants were asked for."""
        return (Chart(self, index) for index in range(len(self)))

    @property
    def kind(self) -> ChartKind:
        """What kind of chart these are."""
        return ChartKind(self.decoded.kind)

    @property
    def place(self) -> Observer:
        """The place they were all founded at."""
        return Observer(
            latitude_deg=Latitude(self.decoded.latitude_deg),
            longitude_deg=Longitude(self.decoded.longitude_deg),
            altitude_m=Altitude(self.decoded.altitude_m),
        )

    @property
    def model(self) -> str:
        """The solar model that reckoned the days, as it describes itself."""
        return self.decoded.model

    @property
    def steps_applied(self) -> Any:
        """The completion steps the SDK applied, in order."""
        return json.loads(self.decoded.steps)

    @property
    def provenance(self) -> str:
        """The provenance envelope, as the canonical JSON it is stamped as."""
        return self.decoded.provenance


class PositionGrid:
    """A decoded positions blob, with the grid read cell by cell."""

    def __init__(self, decoded: Positions) -> None:
        self.decoded = decoded
        """The blob as its generated decoder read it."""

    @property
    def instant_count(self) -> int:
        """How many instants the grid covers."""
        return self.decoded.jd_count

    @property
    def body_count(self) -> int:
        """How many bodies the grid covers."""
        return self.decoded.body_count

    @property
    def cell_count(self) -> int:
        """How many cells there are."""
        return self.decoded.cells.length

    @property
    def time_scale(self) -> TimeScale:
        """The scale the instants are on."""
        return TimeScale(self.decoded.scale)

    @property
    def body_keys(self) -> list[Body]:
        """The bodies, in the order the cells run."""
        return [Body(identifier) for identifier in self.decoded.bodies.body]

    @property
    def steps_applied(self) -> Any:
        """The completion steps the SDK applied, in order."""
        return json.loads(self.decoded.steps)

    @property
    def provenance(self) -> str:
        """The provenance envelope, as the canonical JSON it is stamped as."""
        return self.decoded.provenance

    @property
    def provenance_of(self) -> Any:
        """The provenance envelope, parsed."""
        return json.loads(self.decoded.provenance)

    def frame(self, teistro: Teistro) -> Frame:
        """The frame the values are in."""
        return teistro.unpack_frame(self.decoded.frame_bits)

    def at(self, instant: int, body: int) -> Cell:
        """One cell of the grid."""
        if not 0 <= instant < self.instant_count or not 0 <= body < self.body_count:
            raise IndexError(
                f"({instant}, {body}) is outside a "
                f"{self.instant_count}x{self.body_count} grid"
            )
        index = instant * self.body_count + body
        cells = self.decoded.cells
        return Cell(
            longitude=cells.lon[index],
            latitude=cells.lat[index],
            distance=cells.dist[index],
            longitude_speed=cells.lon_speed[index],
            latitude_speed=cells.lat_speed[index],
            distance_speed=cells.dist_speed[index],
            status=cells.status[index],
            source=cells.source[index],
        )


class _Renderer:
    """What the typed message accessors reach the engine through."""

    def __init__(self, context: Context) -> None:
        self._context = context

    def render(self, key: str, params: Mapping[str, object] = {}) -> str:
        return self._context.render(key, params).text

    def entity(self, key: str) -> intl.EntityForms:
        return self._context.entity(key)


def date(calendar: Calendar, year: int, month: int, day: int) -> CalendarDate:
    """A date in a calendar, without naming the fields a call fills in.

    The era and the era year are what the call resolves them to, and the
    resolution is `DEFINED`, which is what a date a caller states means.

    ```python
    date(Calendar.GREGORIAN, 2015, 4, 14)
    ```
    """
    return CalendarDate(
        calendar=calendar,
        year=year,
        era_year=0,
        month=month,
        day=day,
        resolution=Resolution.DEFINED,
        computed_month=0,
        computed_day=0,
    )


def at(
    day: CalendarDate,
    *,
    hour: int = 0,
    minute: int = 0,
    second: int = 0,
    nanos: int = 0,
) -> CivilDateTime:
    """A date at a time of day.

    ```python
    at(date(Calendar.GREGORIAN, 1986, 1, 1), hour=0, minute=20)
    ```
    """
    return CivilDateTime(
        date=day,
        time=CivilTime(
            hour=hour, minute=minute, second=second, has_time=True, nanos=nanos
        ),
    )


def when_unknown(day: CalendarDate) -> CivilDateTime:
    """A date whose time of day is unknown.

    Nothing guesses one. Unless the profile sets ``time.unknown_time``, a
    resolution refuses it by name and the hint says what to choose; under
    ``NOON`` it resolves with ``time_known`` false and a
    ``time-unknown-fallback`` warning, and under ``SUNRISE`` it needs the
    place and a solar model.
    """
    return CivilDateTime(
        date=day,
        time=CivilTime(hour=0, minute=0, second=0, has_time=False, nanos=0),
    )


def iana_zone(name: str) -> ZoneSpec:
    """A zone of the embedded database, by its IANA name."""
    return ZoneSpec(
        kind=ZoneKind.IANA,
        offset_seconds=0,
        longitude_deg=Longitude(0),
        zone=name,
    )


def fixed_zone(offset_seconds: int) -> ZoneSpec:
    """A fixed offset from UTC, in seconds east."""
    return ZoneSpec(
        kind=ZoneKind.FIXED,
        offset_seconds=offset_seconds,
        longitude_deg=Longitude(0),
    )


def local_mean_zone(longitude_deg: Longitude) -> ZoneSpec:
    """Local mean time at a longitude east of Greenwich, which is what a
    chart from before the zone existed is cast in."""
    return ZoneSpec(
        kind=ZoneKind.LOCAL_MEAN,
        offset_seconds=0,
        longitude_deg=longitude_deg,
    )


def _module_version() -> str:  # pragma: no cover — read by tooling
    return GENERATED_SDK_VERSION


__version__ = GENERATED_SDK_VERSION

if sys.version_info < (3, 11):  # pragma: no cover — the floor is declared
    raise RuntimeError("the Teistro SDK needs Python 3.11 or later")
