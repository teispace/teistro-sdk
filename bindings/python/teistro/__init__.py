"""The Teistro SDK for Python: the layer a consumer uses.

HAND-WRITTEN, and thin on purpose. Everything beneath it is generated
from the API description: the `ctypes` declarations and the value classes
(`_ffi.py`), the catalogue's enums (`catalogue.py`) and the result-blob
decoders (`_blob.py`). What this file adds is what a generator cannot
know: where the shared library is, defaults, JSON in and out, and the
small conveniences a decoded result deserves.

```python
from teistro import Body, Ephemeris, Teistro

teistro = Teistro.open()
with teistro.context(ephemeris=Ephemeris.BUILTIN) as sky:
    sun = sky.positions(instants=[2451545.0], bodies=[Body.SUN]).at(0, 0)
    print(sun.longitude)
```
"""

from __future__ import annotations

import ctypes
import ctypes.util
import itertools
import json
import math
import os
import sys
from dataclasses import dataclass
from functools import cached_property
from pathlib import Path
from types import MappingProxyType, TracebackType
from typing import Any, Callable, Dict, Generic, Iterator, List, Literal, Mapping, NamedTuple, Optional, Sequence, Tuple, TypedDict, TypeVar, Union

from . import messages as intl
from ._blob import (
    BlobError,
    Charts,
    IntlRender,
    Panchanga,
    Positions,
    decode_charts,
    decode_intl_render,
    decode_panchanga,
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
    PanchangaRequest,
    PositionRequest,
    TeistroContext,
    TeistroProvider,
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
    AvasthaCheshta,
    AvasthaSayanadi,
    Ayana,
    Ayanamsha,
    Balance,
    Ekadhipatya,
    Shodhana,
    TajikaDrishti,
    TajikaYoga,
    YearYoga,
    Saham,
    SahamStrong,
    SahamWeak,
    HarshaGrade,
    TajikaRelation,
    Affliction,
    Vaiseshikamsa,
    DashaPhase,
    Nature,
    VarsheshaChosen,
    VimshopakaScoring,
    Body,
    Calendar,
    Centre,
    ChartKind,
    ChartLayout,
    Choghadiya,
    DashaSystem,
    DayPart,
    Direction,
    Ephemeris,
    Graha,
    HouseSystem,
    Kaala,
    Karana,
    LunarMonth,
    Masa,
    MonthKind,
    MuhurtaYoga,
    Nakshatra,
    Paksha,
    Panchaka,
    Rashi,
    Tithi,
    AvasthaBaladi,
    AvasthaDeeptadi,
    AvasthaJagradadi,
    AvasthaLajjitadi,
    Burning,
    Dignity,
    Point,
    Quadrant,
    Relationship,
    Vara,
    Varga,
    Strength,
    Yoga,
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
    "Engine",
    "Ayanamsha",
    "BlobError",
    "Abhijit",
    "Almanac",
    "AlmanacDay",
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
    "Ephemeris",
    "Era",
    "Error",
    "Frame",
    "Hash",
    "InstallError",
    "IntlLoaded",
    "IntlRender",
    "MessagePart",
    "Latitude",
    "Longitude",
    "ChoghadiyaPeriod",
    "HeldYoga",
    "Hora",
    "Interval",
    "KaalaPeriod",
    "MoonEvent",
    "Month",
    "Muhurta",
    "Observer",
    "PlacedGraha",
    "Placement",
    "PositionAnswer",
    "PositionQuery",
    "Positions",
    "Span",
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
    "message_parts",
    "when_unknown",
    # The divisional charts: the catalogue member a caller names and the
    # three shapes a chart's `vargas` answers with.
    "Varga",
    # Chart geometry: the layouts a chart is drawn in, and what a drawing is.
    "ChartLayout",
    "Drawing",
    "LayoutRow",
    "PlanItem",
    "PlanRequest",
    "Plans",
    "RuleRequest",
    "RulesReading",
    "ShippedRules",
    "Theme",
    "ThemeContent",
    "ThemeRecord",
    "ThemeStyle",
    "DrawnCell",
    "DrawnMark",
    "Outline",
    "UnitPoint",
    "LineSegment",
    "QuadSegment",
    "ArcSegment",
    "DerivedPoint",
    "Drishti",
    # The dashas: the system a caller names and what a chart answers with.
    "Balance",
    "Dasha",
    "DashaBalance",
    "DashaPeriod",
    "DashaSystem",
    "Nakshatra",
    "Rashi",
    "WrittenBalance",
    # The Ashtakavarga: what a chart answers with, and the two readings.
    "Ashtakavarga",
    "GrahaAshtakavarga",
    "Ekadhipatya",
    "Shodhana",
    # The Bhava bala: what a chart answers with.
    "BhavaBala",
    "BhavaStrength",
    # The Shadbala: what a chart answers with.
    "GrahaShadbala",
    "KaalaBala",
    "Shadbala",
    "SthanaBala",
    # The dasha phala: what a chart answers with, and where a dasha's effects come.
    "DashaPhalaReading",
    "DashaPhase",
    "GrahaDashaPhala",
    "Nature",
    # The Vaiseshikamsa: what a chart answers with, and its names.
    "GrahaVaiseshikamsa",
    "Vaiseshikamsa",
    "VaiseshikamsaReading",
    "VaiseshikamsaStanding",
    # The Vimshopaka: what a chart answers with, and the two scorings.
    "GrahaVimshopaka",
    "Vimshopaka",
    "VimshopakaScoring",
    "AvasthaBaladi",
    "AvasthaCheshta",
    "AvasthaDeeptadi",
    "AvasthaSayanadi",
    "AvasthaJagradadi",
    "AvasthaLajjitadi",
    "Burning",
    "Combustion",
    "Dignity",
    "Friendship",
    "GrahaState",
    "DashaDefinition",
    "AnnualChart",
    "Bala",
    "Muntha",
    "OfficeBearers",
    "Pravesha",
    "VarshaPlace",
    "TajikaDrishti",
    "TajikaPair",
    "TajikaYoga",
    "VarsheshaChosen",
    "VarsheshaRules",
    "YearClaim",
    "YearLord",
    "VarshaRequest",
    "YearYoga",
    "Affliction",
    "Afflictions",
    "DrishtiRules",
    "HeldYearYoga",
    "TajikaBetween",
    "TajikaMatter",
    "YogaRules",
    "Saham",
    "SahamRules",
    "TajikaSaham",
    "SahamStrong",
    "SahamWeak",
    "SahamSeven",
    "SahamStrengthReadings",
    "HarshaGrade",
    "HarshaRules",
    "HarshaBala",
    "TajikaRelation",
    "AnnualDasha",
    "AnnualDashaRules",
    "AnnualDashaShare",
    "DashaRing",
    "YearOfDays",
    "RashiDashaDefinition",
    "UduDashaDefinition",
    "DashaLord",
    "Sayanadi",
    "Lajjitadi",
    "Quadrant",
    "Relationship",
    "ServiceBhava",
    "War",
    "EdgeDistance",
    "Strength",
    "VargaChart",
    "VargaPlacement",
    "PlacedInVarga",
]

#: The environment variable that names the shared library, which wins over
#: every other place it is looked for.
PATH_VARIABLE = "TEISTRO_LIBRARY"

#: The member a span carries, which differs per limb.
T = TypeVar("T")


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


@dataclass(frozen=True, slots=True)
class Plugin:
    """An adapter's descriptor: the platform binary its package ships,
    and that adapter's own configuration.

    What the configuration means is the adapter's to say and its
    package's to type; the SDK hands it over as JSON and reads none of
    it (ADR-0029).
    """

    plugin: str
    """The adapter's platform binary."""
    config: Optional[Mapping[str, object]] = None
    """That adapter's own options."""


EphemerisChoice = Ephemeris | Plugin
"""One entry of an ephemeris chain: one of the SDK's own by name, or an
adapter's descriptor."""


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
        ephemeris: Optional[EphemerisChoice | Sequence[EphemerisChoice]] = None,
        test_provider: bool = False,
        layouts: Sequence[LayoutRow] = (),
        dasha_systems: Sequence[DashaDefinition] = (),
    ) -> Context:
        """A context: settings, a locale and an ephemeris.

        `layouts` are chart layouts of your own, to draw in beside the
        shipped ones: each a row as `sdk.chart.layout(key)` answers it, with
        a key of its own, checked by the rules a shipped row passes
        (`03-design/chart-geometry.md` §7f).

        `dasha_systems` are dasha systems of your own, of either kernel, each
        a `DashaDefinition` naming its `kernel`, asked for in a request's
        `dashas` by `"dasha_system.<KEY>"` and checked by the rules a shipped
        row passes (`03-design/dasha-kernels.md`).

        `settings` is a patch over the profile, as a mapping — the shape
        the Node and Dart bindings take, so one example reads in all
        three. `settings_json` takes the same patch already serialised,
        for a caller who has the document rather than the mapping; giving
        both is refused rather than one silently winning.

        `provider` binds an ephemeris written in Python. `ephemeris`
        names one of the SDK's own instead: `Ephemeris.BUILTIN` is the
        analytic ephemeris the SDK carries, which needs no files, no
        network and no licence beyond the SDK's own, and is what lets a
        chart compute with nothing else installed; `Ephemeris.TEST` is the
        test provider, whose positions are **not astronomy**.

        `ephemeris` is one entry or an **ordered chain**, tried in order
        (ADR-0029). An entry is an adapter's own descriptor -- `Plugin`,
        what a package like `teistro_ephemeris_teimeris` exports,
        carrying the platform binary it ships and that adapter's own
        configuration -- or one of the SDK's own by name.

        A chain is a caller **saying** they will accept the fallback. One
        entry is one entry: a context asked for an engine and given the
        built-in without being told is the silence this refuses.

        `test_provider=True` is the older spelling of `Ephemeris.TEST` and
        still works; `ephemeris` wins when both are given (ADR-0028).
        Naming none of them leaves the context without an ephemeris, so a
        request for positions is refused with `Status.CAPABILITY`.
        """
        if settings is not None and settings_json is not None:
            raise ValueError(
                "settings and settings_json are the same patch twice; "
                "give one of them"
            )
        # Two ways to answer one question, so both together is a refusal
        # rather than one silently winning -- the same rule the settings
        # patch has just above.
        if provider is not None and ephemeris is not None:
            raise ValueError(
                "provider and ephemeris each name the ephemeris to compute "
                "with; give one of them"
            )
        if settings is not None:
            settings_json = json.dumps(settings, separators=(",", ":"))
        if isinstance(layouts, (str, bytes)) or not isinstance(layouts, Sequence):
            raise TeistroError(
                Status.INVALID_ARG, "layouts is a sequence of layout rows", field="layouts"
            )
        layouts_json = json.dumps(list(layouts), separators=(",", ":")) if layouts else None
        if isinstance(dasha_systems, (str, bytes)) or not isinstance(dasha_systems, Sequence):
            raise TeistroError(
                Status.INVALID_ARG,
                "dasha_systems is a sequence of dasha system definitions",
                field="dasha_systems",
            )
        rows_json = _RowsJson(
            layouts=layouts_json,
            dashas=json.dumps(list(dasha_systems), separators=(",", ":")) if dasha_systems else None,
        )
        host = None if provider is None else HostProvider(self.library, provider)
        # One rule, written once: a named ephemeris wins, and the older
        # flag decides only when none was named (ADR-0028).
        chain: list[EphemerisChoice]
        if ephemeris is None:
            chain = [Ephemeris.TEST if test_provider else Ephemeris.NONE]
        elif isinstance(ephemeris, (Ephemeris, Plugin)):
            chain = [ephemeris]
        else:
            chain = list(ephemeris)
        if not chain:
            raise ValueError(
                "an ephemeris chain of none names nothing; give an entry or "
                "omit it"
            )
        # A chain of one is not a chain, so nothing is caught for it: a
        # bad *profile* is not an ephemeris failure, and catching it to
        # try the next entry would replace a refusal carrying its status,
        # its field and its hint with a bare "nothing could be opened".
        if len(chain) == 1:
            return Context(
                self,
                self._open(chain[0], profile, settings_json, locale, rows_json, host),
                host,
                layouts,
                dasha_systems,
            )
        # With more than one, every refusal is kept and reported
        # together, because a chain that said only why its last entry
        # failed would hide the one the caller actually wanted.
        refusals: list[str] = []
        for entry in chain:
            try:
                return Context(
                    self,
                    self._open(entry, profile, settings_json, locale, rows_json, host),
                    host,
                    layouts,
                    dasha_systems,
                )
            except TeistroError as refusal:
                named = entry.plugin if isinstance(entry, Plugin) else entry.key
                refusals.append(f"{named}: {refusal}")
        joined = "\n  ".join(refusals)
        raise ValueError(f"no ephemeris in the chain could be opened:\n  {joined}")

    def _open(
        self,
        entry: EphemerisChoice,
        profile: Optional[str],
        settings_json: Optional[str],
        locale: Optional[str],
        rows_json: _RowsJson,
        host: Optional[HostProvider],
    ) -> TeistroContext:
        """Opens the context on one entry of the chain."""
        named = Ephemeris.NONE if isinstance(entry, Plugin) else entry
        options = ContextOptions(
            flags=0,
            profile=profile,
            settings_json=settings_json,
            locale=locale,
            layouts_json=rows_json.layouts,
            dashas_json=rows_json.dashas,
            ephemeris=named,
        )
        if not isinstance(entry, Plugin):
            return TeistroContext._new(
                self.library, options, None if host is None else host.vtable
            )
        # The context takes its own reference to the adapter, so the
        # handle this loads is closed at once: what keeps the library
        # loaded is the context, and a consumer holds neither.
        loaded = TeistroProvider._new(
            self.library, entry.plugin, json.dumps(dict(entry.config or {}))
        )
        try:
            return TeistroContext._new_with_provider(self.library, options, loaded)
        finally:
            loaded.close()


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


class Engine:
    """The engine's own operations, reached by the names it gives them.

    The SDK names eight operations. An engine names far more, and what it
    names beyond them is reached through here rather than around the SDK.
    **Nothing in this class is a list of an engine's operations**: it asks
    the engine what it offers and calls what the answer names, so a
    function the engine gains after this package ships is callable
    without a new release of it.

    The engine's own spelling is the name to use, because that is what
    its manifest says and what its documentation calls it. In Python that
    reads naturally, since an engine's C names are already snake case::

        engine = context.ephemeris
        answer = engine.tp_echo(value=6.0)

    `dir()` lists what the engine offers, so a REPL completes the names
    without this class ever holding one.
    """

    def __init__(self, inner: TeistroContext) -> None:
        self._inner = inner
        self._manifest: Optional[Any] = None

    @property
    def manifest_json(self) -> str:
        """The manifest as the engine wrote it."""
        return self._inner.ephemeris_manifest()

    @property
    def manifest(self) -> Any:
        """The manifest, parsed and remembered.

        Read once per engine: it changes when the engine does, and an
        engine does not change under a live context.
        """
        if self._manifest is None:
            self._manifest = json.loads(self.manifest_json)
        return self._manifest

    @property
    def names(self) -> Tuple[str, ...]:
        """Every operation the engine offers, in its own order."""
        return tuple(
            str(function.get("name", ""))
            for function in self.manifest.get("functions", [])
        )

    def signature(self, function: str) -> Any:
        """What the manifest says about one operation, or `None`.

        The parameters carry the role of each, which says which of them a
        caller supplies and which the engine fills.
        """
        for candidate in self.manifest.get("functions", []):
            if candidate.get("name") == function:
                return candidate
        return None

    def call(self, function: str, **arguments: Any) -> Any:
        """Calls an operation by name, with its parameters as keywords.

        The keywords are the parameter names the manifest gives. What
        comes back is the engine's own answer, parsed.
        """
        return json.loads(self.call_json(function, json.dumps(arguments, separators=(",", ":"))))

    def call_json(self, function: str, arguments_json: str) -> str:
        """Calls an operation with arguments already written as JSON, and
        answers with the engine's own JSON: the form to use when the
        answer is being handed on rather than read.
        """
        return self._inner.ephemeris_call(function, arguments_json)

    def __getattr__(self, name: str) -> Any:
        if name.startswith("_"):
            raise AttributeError(name)
        if name not in self.names:
            raise AttributeError(
                f"this engine names no operation `{name}`; "
                f"it offers {len(self.names)}, and `dir()` lists them"
            )

        def operation(**arguments: Any) -> Any:
            return self.call(name, **arguments)

        operation.__name__ = name
        signature = self.signature(name) or {}
        operation.__doc__ = signature.get("doc") or f"The engine's `{name}`."
        return operation

    def __dir__(self) -> List[str]:
        return sorted(set(list(super().__dir__()) + list(self.names)))

    def __contains__(self, name: str) -> bool:
        return name in self.names

    def __len__(self) -> int:
        return len(self.names)

    def __repr__(self) -> str:
        manifest = self.manifest
        return (
            f"Engine({manifest.get('engine', '?')!r} "
            f"{manifest.get('version', '?')!r}, {len(self)} operations)"
        )


class _Area:
    """What every area is.

    A **value**: one object per context, built on first read of the
    `cached_property` that holds it and kept, so a consumer may hold it
    and pass it (`calendar = sdk.calendar`). That is what makes the areas
    worth having rather than merely tidy
    (`03-design/surface-areas.md`).

    Each holds the context and nothing else, and reaches the boundary
    through it, so the guard that re-raises what a provider written in
    Python raised stays in one place.
    """

    __slots__ = ("_context",)

    def __init__(self, context: Context) -> None:
        self._context = context


class CalendarArea(_Area):
    """`sdk.calendar` — the calendars, and the fixed day they share."""

    def date_of(self, calendar: Calendar, fixed: int) -> CalendarDate:
        """The date a fixed day number is, in a calendar."""
        return self._context.inner.calendar_from_fixed(calendar, fixed)

    def fixed_of(self, date: CalendarDate) -> int:
        """The fixed day number a date is."""
        return self._context.inner.calendar_to_fixed(date)

    def convert(self, date: CalendarDate, into: Calendar) -> CalendarDate:
        """The same day in another calendar."""
        return self._context.inner.calendar_convert(date, into)

    def weekday_of(self, date: CalendarDate) -> int:
        """The weekday of a date as its ISO number: Monday `1`, Sunday `7`.

        Not the catalogue's `Vara`, which counts from Sunday: a vara is
        `weekday_of(day) % 7`, and the panchanga example does exactly
        that.
        """
        return self._context.inner.calendar_weekday(date)

    def month_length(self, calendar: Calendar, year: int, month: int) -> int:
        """How many days a month has."""
        return self._context.inner.calendar_month_length(calendar, year, month)

    def is_leap(self, calendar: Calendar, year: int) -> bool:
        """Whether a year is a leap year in a calendar."""
        return self._context.inner.calendar_is_leap(calendar, year) != 0


class TimeArea(_Area):
    """`sdk.time` — the scales, the zones and what separates them."""

    def resolve(self, civil: CivilDateTime, zone: ZoneSpec) -> ZoneResolution:
        """The instant a civil date and time in a zone stands for."""
        return self._context.inner.time_resolve(civil, zone)

    def civil_of(
        self, jd_utc: float, zone: ZoneSpec, calendar: Calendar
    ) -> tuple[CivilDateTime, ZoneResolution]:
        """The civil date and time an instant is, in a zone."""
        found = self._context.inner.time_civil(jd_utc, zone, calendar)
        return found.civil, found.resolution

    def convert(self, jd: float, scale: Scale, into: Scale) -> TimeConversion:
        """The same instant on another time scale.

        `Scale` and not `TimeScale`: the time layer knows UTC as well as
        the two the port carries, and the two enums agree on the ids they
        share, so passing the wrong one would convert from the wrong
        scale without any complaint.
        """
        return self._context.inner.time_convert(jd, scale, into)

    def delta_t(self, jd_ut1: float) -> DeltaT:
        """TT less UT1 at an instant, and where the value came from."""
        return self._context.inner.time_delta_t(jd_ut1)


@dataclass(frozen=True)
class MessagePart:
    """One part of a rendered message: its text, or a markup tag standing
    in the text.

    MF2 markup (`{#b}...{/b}`) is how a message says that part of it is a
    link, a name or emphasis, **without saying what that looks like** —
    the message stays free of markup languages and the renderer decides.
    A renderer walks the parts, writes the text ones and opens or closes
    whatever a tag means in its own world.
    """

    is_text: bool
    """Whether this is text rather than a tag."""

    value: str = ""
    """The text, already formatted and localised; empty for a tag."""

    kind: str = ""
    """`open`, `close` or `standalone`; empty for text."""

    name: str = ""
    """The tag's name, as the message wrote it: `b`, `link`, ..."""

    options: Mapping[str, str] = MappingProxyType({})
    """Its options, each already resolved to a string."""

    def __str__(self) -> str:
        return self.value if self.is_text else f"<{self.kind} {self.name}>"


def message_parts(rendered: IntlRender) -> List[MessagePart]:
    """A rendered message in parts, its markup kept: what a rich renderer
    walks.

    Joining the text parts gives exactly `rendered.text`, so a renderer
    that does not know a tag can ignore it and lose nothing. The boundary
    sends nothing when the message has no markup, because the parts would
    then be the text written twice; the one part is made here rather than
    carried.

    ```python
    for part in message_parts(ctx.intl.render("sdk.reason.lordship", ...)):
        print(part.value if part.is_text else part.name)
    ```
    """
    written = json.loads(rendered.parts or "[]")
    if not written:
        return [MessagePart(is_text=True, value=rendered.text)]
    return [
        MessagePart(is_text=True, value=part["value"])
        if part["type"] == "text"
        else MessagePart(
            is_text=False,
            kind=part["kind"],
            name=part["name"],
            options=MappingProxyType(dict(part["options"])),
        )
        for part in written
    ]


class IntlArea(_Area):
    """`sdk.intl` — the locale, its messages and the scripts they are in."""

    # `cached_property` writes to the instance, so this area cannot use
    # `__slots__` the way the others do.
    __slots__ = ("__dict__",)

    @property
    def locale(self) -> str:
        """The locale tag messages are rendered in."""
        return self._context.inner.intl_locale()

    @locale.setter
    def locale(self, tag: str) -> None:
        self._context.inner.intl_set_locale(tag)

    def render(self, key: str, params: Optional[Mapping[str, object]] = None) -> IntlRender:
        """A message rendered in the context's locale."""
        return decode_intl_render(
            self._context.inner.intl_render(key, json.dumps({} if params is None else params))
        )

    def has(self, key: str) -> bool:
        """Whether the current locale carries a message."""
        return self._context.inner.intl_has(key) != 0

    def entity(self, key: str) -> intl.EntityForms:
        """A catalogued entity's forms in the current locale."""
        return intl.EntityForms.of(self._context.inner.intl_entity(key))

    def transliterate(self, text: str, source: str = "Deva", into: str = "Latn") -> str:
        """Text from one script into another."""
        return self._context.inner.intl_transliterate(text, source, into)

    def load_pack(self, data: bytes) -> IntlLoaded:
        """Loads a locale pack's bytes into the engine."""
        return self._context.inner.intl_load_pack(data)

    @cached_property
    def messages(self) -> intl.Messages:
        """The typed accessors: every message of the SDK, by its key.

        ```python
        sdk.intl.messages.sdk.reason.graha_in_bhava(graha="graha.JUPITER", bhava=7)
        ```
        """
        return intl.Messages(_Renderer(self._context))


class KeysArea(_Area):
    """`sdk.keys` — the catalogue's keys and their packed ids."""

    def id(self, key: str) -> int:
        """The catalogue id a key stands for."""
        return self._context.inner.key_parse(key)

    def name(self, identifier: int) -> str:
        """The key an id stands for."""
        return self._context.inner.key_name(identifier)


class FrameArea(_Area):
    """`sdk.frame` — the coordinate conventions a request is expressed in."""

    @property
    def canonical(self) -> Frame:
        """The SDK's canonical frame: apparent geocentric ecliptic of
        date, tropical."""
        return self._context.teistro.canonical_frame

    def pack(self, frame: Frame) -> int:
        """A frame's fields as the bits a position request carries."""
        return self._context.teistro.pack_frame(frame)

    def unpack(self, bits: int) -> Frame:
        """The frame a packed set of bits describes."""
        return self._context.teistro.unpack_frame(bits)


class ChartArea(_Area):
    """`sdk.chart` — a chart founded at an instant and a place."""

    def layout(self, key: Union[ChartLayout, str]) -> LayoutRow:
        """A layout this context can draw in, shipped or registered, as its
        row: a fresh mapping to copy, give a key of its own and register
        (`03-design/chart-geometry.md` §7f).

        `key` is a `ChartLayout`, or a layout's key, bare (`NORTH_INDIAN`)
        or full (`chart_layout.ACME_KERALA`).
        """
        named = key.full_key if isinstance(key, ChartLayout) else key
        row: LayoutRow = json.loads(
            self._context._through_provider(lambda: self._context.inner.chart_layout_row(named))
        )
        return row

    def found(
        self,
        *,
        instant: float,
        place: Observer,
        utc_offset_seconds: int,
        kind: ChartKind = ChartKind.NATAL,
        vargas: Sequence[Varga] = (),
        dashas: Sequence[Union[DashaSystem, str]] = (),
        drawings: Sequence[Tuple[Union[ChartLayout, str], Varga]] = (),
        theme: Optional[Theme] = None,
        rules: Optional[RuleRequest] = None,
        interpret: Optional[PlanRequest] = None,
        varsha: Optional[VarshaRequest] = None,
        aspects: bool = False,
        points: bool = False,
        houses: bool = False,
        ashtakavarga: bool = False,
        vimshopaka: bool = False,
        vaiseshikamsa: bool = False,
        dasha_phala: bool = False,
        shadbala: bool = False,
        bhava_bala: bool = False,
        state: bool = False,
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
            vargas=vargas,
            dashas=dashas,
            drawings=drawings,
            theme=theme,
            rules=rules,
            interpret=interpret,
            varsha=varsha,
            aspects=aspects,
            points=points,
            houses=houses,
            ashtakavarga=ashtakavarga,
            vimshopaka=vimshopaka,
            vaiseshikamsa=vaiseshikamsa,
            dasha_phala=dasha_phala,
            shadbala=shadbala,
            bhava_bala=bhava_bala,
            state=state,
        ).at(0)

    def found_many(
        self,
        *,
        instants: Sequence[float],
        place: Observer,
        utc_offset_seconds: int,
        kind: ChartKind = ChartKind.NATAL,
        vargas: Sequence[Varga] = (),
        dashas: Sequence[Union[DashaSystem, str]] = (),
        drawings: Sequence[Tuple[Union[ChartLayout, str], Varga]] = (),
        theme: Optional[Theme] = None,
        rules: Optional[RuleRequest] = None,
        interpret: Optional[PlanRequest] = None,
        varsha: Optional[VarshaRequest] = None,
        aspects: bool = False,
        points: bool = False,
        houses: bool = False,
        ashtakavarga: bool = False,
        vimshopaka: bool = False,
        vaiseshikamsa: bool = False,
        dasha_phala: bool = False,
        shadbala: bool = False,
        bhava_bala: bool = False,
        state: bool = False,
    ) -> ChartBatch:
        """Founds a chart at each of many instants, at one place, in one
        crossing.

        The founder shares the settings and the solar model across the
        batch, so a hundred instants cost one setup rather than a hundred
        — which is what a rectification pass wants. A batch of none is an
        empty result rather than an error.

        `vargas` names the divisional charts to compute, in the order to
        answer them; none by default, because a caller who wants a birth
        chart should not pay for twenty-one of them
        (`03-design/chart-reading.md` §4). `drawings` names charts to draw,
        each a `(ChartLayout, Varga)` pair with `Varga.D1` the founded chart,
        in the order to answer them. `dashas` names the dasha systems to
        compute, their periods to the settings' `dasha.depth`
        (`03-design/dasha-kernels.md`). `aspects` asks for the drishti.
        """
        request = ChartRequest(
            kind=kind,
            instants=list(instants),
            latitude_deg=place.latitude_deg,
            longitude_deg=place.longitude_deg,
            altitude_m=place.altitude_m,
            utc_offset_seconds=utc_offset_seconds,
            # The sections beside the foundation, which the SDK takes as
            # a bit set and nothing here writes as one
            # (`03-design/chart-reading.md` §5): a named argument each,
            # and one more as each crosses.
            sections=(_SECTION_ASPECTS if aspects else 0)
            | (_SECTION_POINTS if points else 0)
            | (_SECTION_HOUSES if houses else 0)
            | (_SECTION_ASHTAKAVARGA if ashtakavarga else 0)
            | (_SECTION_VIMSHOPAKA if vimshopaka else 0)
            | (_SECTION_VAISESHIKAMSA if vaiseshikamsa else 0)
            | (_SECTION_DASHA_PHALA if dasha_phala else 0)
            | (_SECTION_SHADBALA if shadbala else 0)
            | (_SECTION_BHAVA_BALA if bhava_bala else 0)
            | (_SECTION_STATE if state else 0),
            vargas=list(vargas),
            dashas=_dasha_ids(dashas, self._context._registered_dashas),
            drawings=_drawing_bits(drawings, self._context._registered_layouts),
            theme_json=_theme_json(theme),
            rules_json=_rules_json(rules),
            interpret_json=_interpret_json(interpret),
            varsha_json=_varsha_json(varsha),
        )
        return ChartBatch(
            decode_charts(self._context._through_provider(lambda: self._context.inner.chart_found(request))),
            self._context._dasha_names,
        )


class AlmanacArea(_Area):
    """`sdk.almanac` — a day, or a run of days, with its limbs.

    The boundary calls this `panchanga`; the area takes the consumer's
    word, because an almanac is what the operation answers and a
    panchanga is one tradition's name for five of its limbs
    (`03-design/surface-areas.md`).
    """

    def of(
        self,
        *,
        from_date: CalendarDate,
        to_date: CalendarDate,
        place: Observer,
        utc_offset_seconds: int,
    ) -> Almanac:
        """The almanac of every day in a range, at one place.

        A **range** rather than a list of dates, because consecutive days
        share a boundary — day *n*'s next sunrise is day *n+1*'s sunrise
        — so a month of days costs much less than thirty days computed
        separately. A range holding more than a year and a day is refused
        by name.
        """
        request = PanchangaRequest(
            calendar=from_date.calendar,
            from_year=from_date.year,
            from_month=from_date.month,
            from_day=from_date.day,
            to_year=to_date.year,
            to_month=to_date.month,
            to_day=to_date.day,
            latitude_deg=place.latitude_deg,
            longitude_deg=place.longitude_deg,
            altitude_m=place.altitude_m,
            utc_offset_seconds=utc_offset_seconds,
        )
        return Almanac(
            decode_panchanga(
                self._context._through_provider(lambda: self._context.inner.panchanga_days(request))
            )
        )

    def day(
        self,
        *,
        date: CalendarDate,
        place: Observer,
        utc_offset_seconds: int,
    ) -> AlmanacDay:
        """The almanac of one day, which is the range of one unwrapped."""
        return self.of(
            from_date=date,
            to_date=date,
            place=place,
            utc_offset_seconds=utc_offset_seconds,
        ).at(0)


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
        layouts: Sequence[LayoutRow] = (),
        dasha_systems: Sequence[DashaDefinition] = (),
    ) -> None:
        self.teistro = teistro
        """The library this context was built on."""
        self.inner = inner
        """The generated context, for a call this layer does not wrap."""
        self._host = host
        # The member id of each layout this context registered, by its full
        # key: asked once, here, so a request resolves a consumer's own
        # layout without crossing the boundary again
        # (`03-design/chart-geometry.md` §7f).
        self._registered_layouts: dict[str, int] = {
            f"chart_layout.{row['key']}": inner.key_parse(f"chart_layout.{row['key']}") & 0xFFFF
            for row in layouts
        }
        # The same for the dasha systems it registered, and turned round so a
        # batch names a registered id by its key.
        self._registered_dashas: dict[str, int] = {
            f"dasha_system.{row['key']}": inner.key_parse(f"dasha_system.{row['key']}") & 0xFFFF
            for row in dasha_systems
        }
        self._dasha_names: dict[int, str] = {
            id: key for key, id in self._registered_dashas.items()
        }

    # ── The areas ─────────────────────────────────────────────────────
    #
    # A `cached_property` apiece: the first read builds it and every later
    # one is the same object, so `calendar = sdk.calendar` is a value a
    # consumer can hold (`03-design/surface-areas.md`).

    @cached_property
    def calendar(self) -> CalendarArea:
        """The calendars, and the fixed day they share."""
        return CalendarArea(self)

    @cached_property
    def time(self) -> TimeArea:
        """The scales, the zones and what separates them."""
        return TimeArea(self)

    @cached_property
    def intl(self) -> IntlArea:
        """The locale, its messages and the scripts they are in."""
        return IntlArea(self)

    @cached_property
    def keys(self) -> KeysArea:
        """The catalogue's keys and their packed ids."""
        return KeysArea(self)

    @cached_property
    def frame(self) -> FrameArea:
        """The coordinate conventions a request is expressed in."""
        return FrameArea(self)

    @cached_property
    def chart(self) -> ChartArea:
        """A chart founded at an instant and a place."""
        return ChartArea(self)

    @cached_property
    def almanac(self) -> AlmanacArea:
        """A day, or a run of days, with its limbs."""
        return AlmanacArea(self)

    # ── The context itself ────────────────────────────────────────────

    @property
    def engine(self) -> Engine:
        """The engine's own operations, beyond the eight the SDK names.

        **Not `ephemeris`**: `engine` says *this particular engine, not
        the portable contract*, so a consumer reading their own code sees
        the difference between a call that survives changing provider and
        one that does not (ADR-0030).

        Raises `TeistroError` when the context has no ephemeris, or when
        the one it has describes nothing of its own.
        """
        engine = Engine(self.inner)
        # Ask now rather than at the first call, so a context that cannot
        # offer this says so where a caller can act on it.
        engine.manifest_json
        return engine

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


#: `TS_CHART_ASPECTS`, the one section bit this layer offers so far.
#:
#: The bits are the C ABI's vocabulary; a consumer of this binding passes
#: `aspects=True` (`03-design/chart-reading.md` §5).
_SECTION_ASPECTS = 4

#: `TS_CHART_POINTS`, the derived points.
_SECTION_POINTS = 8

#: `TS_CHART_HOUSES`, the houses service.
_SECTION_HOUSES = 16

#: `TS_CHART_ASHTAKAVARGA`, the Ashtakavarga.
_SECTION_ASHTAKAVARGA = 32

#: `TS_CHART_VIMSHOPAKA`, the Vimshopaka.
_SECTION_VIMSHOPAKA = 64

#: `TS_CHART_VAISESHIKAMSA`, the Vaiseshikamsa.
_SECTION_VAISESHIKAMSA = 512
#: `TS_CHART_DASHA_PHALA`, the dasha phala.
_SECTION_DASHA_PHALA = 1024

#: `TS_CHART_SHADBALA`, the Shadbala.
_SECTION_SHADBALA = 128

#: `TS_CHART_BHAVA_BALA`, the Bhava bala.
_SECTION_BHAVA_BALA = 256

#: `TS_CHART_STATE`, the planetary states.
_SECTION_STATE = 2


@dataclass(frozen=True)
class EdgeDistance:
    """How near a body stands to a boundary, which is what an ayanamsha
    that moved would change."""

    sign_deg: float
    """To the nearer edge of its sign, degrees."""

    nakshatra_deg: float
    """To the nearer edge of its nakshatra, degrees."""

    pada_deg: float
    """To the nearer edge of its pada, degrees."""


@dataclass(frozen=True)
class Combustion:
    """What the Sun does to a body."""

    burning: Burning
    """How badly it burns."""

    from_sun_deg: float | None
    """How far from the Sun it stands, degrees, or None when the chart
    carries no Sun — in which case nothing is burnt and this says why
    rather than claiming the sky is clear."""

    orb_deg: float | None
    """Combust inside this, degrees, or None for a body that does not
    burn at all."""

    deep_orb_deg: float | None
    """Deeply combust inside this, degrees, where the table gives one."""


@dataclass(frozen=True)
class Friendship:
    """How a body stands to its dispositor, three ways."""

    natural: Relationship
    """The table's own reading."""

    temporary: Relationship
    """Where the dispositor stands."""

    compound: Relationship
    """The five-fold compound of the two."""

    dispositor: Graha | None
    """The lord of the sign, which all three are with; None only for a
    body the catalogue gives no sign."""


@dataclass(frozen=True)
class Lajjitadi:
    """The lajjitadi a body holds, is ruled out of, and nothing decides."""

    holding: list[AvasthaLajjitadi]
    """The states that hold."""

    ruled_out: list[AvasthaLajjitadi]
    """The states that certainly do not hold."""

    undecided: list[AvasthaLajjitadi]
    """The states nothing decides: the necessary condition holds and what
    narrows it further is not in the chart."""


@dataclass(frozen=True)
class War:
    """A planetary war a body is in."""

    opponent: Graha
    """The other body."""

    is_winner: bool
    """Whether this body won it."""

    apart_deg: float
    """How far apart they stand, degrees."""


@dataclass(frozen=True)
class Sayanadi:
    """A graha's Sayanadi state, with its sub-state under a name of each
    anka (BPHS ch. 45 vv. 30 to 37)."""

    avastha: AvasthaSayanadi
    """The state, Shayana to Nidra."""

    cheshtas: tuple[AvasthaCheshta, ...]
    """The sub-state under a name whose first syllable's anka is 1 to 5, in
    that order."""

    def cheshta(self, anka: int) -> AvasthaCheshta:
        """The sub-state under a name of this anka.

        >>> # state.sayanadi.cheshta(3)

        Raises `ValueError` outside 1 to 5.
        """
        if not 1 <= anka <= 5:
            raise ValueError(f"anka: {anka} is not a syllable's anka; it is 1 to 5")
        return self.cheshtas[anka - 1]


@dataclass(frozen=True)
class GrahaState:
    """What one graha **is**, as opposed to where it is."""

    graha: Graha
    """Which graha."""

    sign: Rashi
    """The sign it stands in."""

    house: int
    """The bhava it falls in, under the chart's placement system."""

    dignity: Dignity
    """Its dignity."""

    friendship: Friendship
    """How it stands to its dispositor."""

    combustion: Combustion
    """What the Sun does to it."""

    age: AvasthaBaladi
    """Which fifth of its sign it stands in."""

    wakefulness: AvasthaJagradadi
    """Awake, dreaming or asleep."""

    deeptadi: AvasthaDeeptadi | None
    """The bright state, where the SDK can decide one."""

    lajjitadi: Lajjitadi
    """The lajjitadi that hold, and the ones nothing decides."""

    war: War | None
    """The war it is in, if it is in one."""

    sayanadi: Sayanadi | None
    """The Sayanadi state and its sub-states, or `None` for a body the
    verses give no number."""

    boundaries: EdgeDistance
    """How near it stands to a classification boundary."""


@dataclass(frozen=True)
class ServiceBhava:
    """One bhava as the houses service reads it."""

    number: int
    """The bhava, 1 to 12."""

    sign: Rashi
    """The sign its **middle** falls in, which under an unequal division
    is not the sign it begins in."""

    lord: Graha
    """The lord of that sign."""

    quadrant: Quadrant
    """Which third of the wheel it stands in."""


@dataclass(frozen=True)
class DerivedPoint:
    """One derived point: an upagraha or a special lagna."""

    point: Point
    """Which point."""

    longitude_deg: float
    """Its longitude in the chart's zodiac, degrees."""

    sign: Rashi
    """The sign it falls in."""

    boundaries: EdgeDistance
    """How near it stands to a sign, nakshatra or pada edge."""


@dataclass(frozen=True)
class Drishti:
    """One body looking at another."""

    from_graha: Graha
    """The body looking. Named `from_graha` because `from` is a keyword."""

    to: Graha
    """The body looked at."""

    houses: int
    """Which house of the first's sign the second stands in, counting
    inclusively from one."""

    strength: Strength
    """How strongly."""

    from_edge: EdgeDistance
    """How near the looking body stands to a boundary."""

    to_edge: EdgeDistance
    """How near the body looked at stands to one."""


@dataclass(frozen=True)
class GrahaAshtakavarga:
    """One graha's Ashtakavarga."""

    graha: Graha
    """Which graha, Sun to Saturn."""

    bindus: Tuple[int, ...]
    """Its bindus by sign, Aries to Pisces, 0 to 8."""

    reduced: Optional[Tuple[int, ...]]
    """The same after both reductions, when they were made in each graha's
    own Ashtakavarga; None otherwise."""

    rashi_pinda: int
    """Its rashi pinda."""

    graha_pinda: int
    """Its graha pinda."""

    yoga_pinda: int
    """Its yoga pinda, the two together."""


@dataclass(frozen=True)
class BhavaStrength:
    """One bhava's Bhava bala, in virupas."""

    bhava: int
    """Which bhava, 1 to 12."""

    lord: Graha
    """The lord of the sign its madhya falls in."""

    adhipati: float
    """The lord's Shadbala."""

    dig: float
    """From its direction, 0 to 60."""

    drishti: float
    """From the drishtis it receives, which may be negative."""

    special: float
    """From its occupants and its sign's rising, under BPHS's special rules."""

    virupas: float
    """The four together."""


@dataclass(frozen=True)
class BhavaBala:
    """A chart's Bhava bala, read under the context's `strength.bhava_*`
    settings (`03-design/bhava-bala-measured.md`)."""

    bhavas: Tuple[BhavaStrength, ...]
    """Each bhava's, the first to the twelfth."""


@dataclass(frozen=True)
class SthanaBala:
    """A graha's Sthana bala by component, virupas."""

    uchcha: float
    """From its distance to its debilitation point, 0 to 60."""

    saptavargaja: float
    """From its dignity in the seven vargas."""

    ojayugma: float
    """From its rasi's and navamsha's parity, 0, 15 or 30."""

    kendradi: float
    """From its house: 60, 30 or 15."""

    drekkana: float
    """From its decanate: 0 or 15."""

    @property
    def total(self) -> float:
        """The five together."""
        return self.uchcha + self.saptavargaja + self.ojayugma + self.kendradi + self.drekkana


@dataclass(frozen=True)
class KaalaBala:
    """A graha's Kaala bala by component, virupas."""

    nathonnatha: float
    """From the hour, 0 to 60."""

    paksha: float
    """From the Moon's elongation, the Moon's doubled."""

    tribhaga: float
    """60 to the lord of the third of the day or night, and to Jupiter."""

    abda: float
    """15 to the year's lord."""

    masa: float
    """30 to the month's lord."""

    vara: float
    """45 to the weekday's lord."""

    hora: float
    """60 to the hour's lord."""

    ayana: float
    """From its declination."""

    yuddha: float
    """Gained by the victor and lost by the vanquished of a planetary war."""

    @property
    def total(self) -> float:
        """The nine together."""
        return (
            self.nathonnatha
            + self.paksha
            + self.tribhaga
            + self.vara
            + self.hora
            + self.ayana
            + self.abda
            + self.masa
            + self.yuddha
        )


@dataclass(frozen=True)
class GrahaShadbala:
    """One graha's Shadbala, in virupas."""

    graha: Graha
    """Which graha, Sun to Saturn."""

    sthana: SthanaBala
    """Positional strength by component."""

    dig: float
    """Directional strength, 0 to 60."""

    kaala: KaalaBala
    """Temporal strength by component."""

    cheshta: float
    """Motional strength."""

    naisargika: float
    """Natural strength."""

    drik: float
    """Aspectual strength, which may be negative."""

    virupas: float
    """The six together."""

    rupas: float
    """The six together, in rupas."""

    required_rupas: float
    """The rupas it must reach to be strong."""

    strong: bool
    """Whether it reaches them."""

    ishta: float
    """How far it tends to good, 0 to 60 (BPHS ch. 28)."""

    kashta: float
    """How far it tends to harm, 0 to 60."""

    subha_rashmi: float
    """Its auspicious rays, 1 to 7: the mean of its Uchcha and Cheshta rays
    (BPHS ch. 28 v. 5)."""

    ashubha_rashmi: float
    """Its inauspicious rays, 8 less the auspicious."""


@dataclass(frozen=True)
class Shadbala:
    """A chart's Shadbala, read under the context's `strength.*` settings
    (`03-design/shadbala-measured.md`)."""

    grahas: Tuple[GrahaShadbala, ...]
    """Each graha's, Sun to Saturn."""


@dataclass(frozen=True)
class GrahaDashaPhala:
    """One graha's dasha phala (BPHS ch. 28 vv. 7 to 10, ch. 47 vv. 3 to 6)."""

    graha: Graha
    """Which graha, Sun to Ketu."""

    subhankas: Tuple[float, ...]
    """Its Subhanka in the D1, D2, D3, D7, D9, D12 and D30: out of 60 in the
    first and 30 in the rest."""

    subhanka: float
    """The seven together, out of 240."""

    asubhanka: float
    """Their complements together, out of 240."""

    nature: Nature
    """Whether its rasi place is auspicious (benefic), neutral or
    inauspicious (malefic)."""

    phase: DashaPhase
    """Where in its dasha its effects come."""

    favourable: bool
    """Whether its placement makes its dasha favourable."""

    unfavourable: bool
    """Whether its placement makes its dasha unfavourable; both can hold."""


@dataclass(frozen=True)
class DashaPhalaReading:
    """A chart's dasha phala, read under `dasha.shanta_sign`.

    >>> # chart = ctx.chart.found(..., dasha_phala=True)
    >>> # saturn = next(g for g in chart.dasha_phala.grahas if g.graha is Graha.SATURN)
    """

    grahas: Tuple[GrahaDashaPhala, ...]
    """Each graha's, Sun to Ketu."""


@dataclass(frozen=True)
class VaiseshikamsaStanding:
    """A graha's standing in one scheme of vargas."""

    good_vargas: int
    """How many of the scheme's vargas are good for it."""

    name: Optional[Vaiseshikamsa]
    """The name that count earns, from two good vargas; None below."""


@dataclass(frozen=True)
class GrahaVaiseshikamsa:
    """One graha's Vaiseshikamsa (BPHS ch. 6 vv. 42 to 53)."""

    graha: Graha
    """Which graha, Sun to Saturn."""

    shadvarga: VaiseshikamsaStanding
    """Over the six vargas."""

    saptavarga: VaiseshikamsaStanding
    """Over the seven."""

    dashavarga: VaiseshikamsaStanding
    """Over the ten."""

    shodashavarga: VaiseshikamsaStanding
    """Over the sixteen."""

    impaired: bool
    """Whether it is combust, defeated in war or in Shayana, its names then not auspicious."""


@dataclass(frozen=True)
class VaiseshikamsaReading:
    """A chart's Vaiseshikamsa."""

    grahas: Tuple[GrahaVaiseshikamsa, ...]
    """Each graha's, Sun to Saturn."""


@dataclass(frozen=True)
class GrahaVimshopaka:
    """One graha's Vimshopaka, each score out of 20."""

    graha: Graha
    """Which graha, Sun to Saturn."""

    shadvarga: float
    """Over the six vargas."""

    saptavarga: float
    """Over the seven."""

    dashavarga: float
    """Over the ten."""

    shodashavarga: float
    """Over the sixteen."""


@dataclass(frozen=True)
class Vimshopaka:
    """A chart's Vimshopaka: each graha's strength across the divisional
    charts under the four schemes (`03-design/vimshopaka-measured.md`)."""

    scoring: VimshopakaScoring
    """How each varga was scored."""

    grahas: Tuple[GrahaVimshopaka, ...]
    """Each graha's, Sun to Saturn."""


@dataclass(frozen=True)
class Ashtakavarga:
    """A chart's Ashtakavarga: each graha's, the sarvashtakavarga, and their
    reductions and pindas (`03-design/ashtakavarga-measured.md`)."""

    shodhana: Shodhana
    """Where the reductions and pindas were made."""

    ekadhipatya: Ekadhipatya
    """How a co-ruled sign beside an occupied one was reduced."""

    grahas: Tuple[GrahaAshtakavarga, ...]
    """Each graha's, Sun to Saturn."""

    sarva: Tuple[int, ...]
    """The seven grahas' bindus by sign, 337 in all."""

    trikona: Tuple[int, ...]
    """The sum after the trine reduction."""

    reduced: Tuple[int, ...]
    """The sum after both reductions."""


@dataclass(frozen=True)
class DashaPeriod:
    """One period of a dasha."""

    path: str
    """Its place at each level from the mahadasha down, joined by `/`:
    `2/5/3`."""

    level: int
    """How deep: 1 for a mahadasha."""

    sign: Optional[Rashi]
    """The sign it is the period of, in a sign-based dasha; None otherwise."""

    lord: Graha
    """Its lord."""

    span: Interval
    """When it runs."""


@dataclass(frozen=True)
class WrittenBalance:
    """A balance written as a reader writes it."""

    years: int
    """Whole years of the year length."""

    months: int
    """Whole months of a twelfth of it."""

    days: int
    """Whole days."""

    hours: int
    """Hours."""

    minutes: int
    """Minutes, rounded."""


@dataclass(frozen=True)
class DashaBalance:
    """What remained of a dasha's first period at birth."""

    method: Balance
    """How it was measured."""

    remaining: float
    """The fraction still to run, 0 to 1."""

    days: float
    """That fraction of the first lord's years, in days."""

    written: WrittenBalance
    """The same in years, months, days, hours and minutes."""


@dataclass(frozen=True)
class Dasha:
    """A dasha of a founded chart: its periods, and for a nakshatra-seeded one
    its seed and balance at birth. A sign-based dasha has neither, and its
    periods name their signs."""

    system: Union[DashaSystem, str]
    """Which system: a `DashaSystem`, or a registered one by its full key
    (`"dasha_system.ACME_SAPTAKA"`)."""

    seed: Optional[Nakshatra]
    """The nakshatra the Moon stood in, which seeds it; None for a sign-based
    dasha."""

    first_lord: Graha
    """The lord it starts with."""

    overflow: bool
    """Whether the seed lay outside a conditional system's nakshatras."""

    balance: Optional[DashaBalance]
    """What remained of the first period at birth; None for a sign-based
    dasha, whose first period runs whole from birth."""

    moon_span: Optional[Interval]
    """The Moon's stay in its nakshatra, when the balance read one."""

    depth: int
    """How many levels the periods go down."""

    periods: Tuple[DashaPeriod, ...]
    """Every period of the birth cycle to `depth`, depth first in time
    order: a mahadasha, then its antardashas and theirs, then the next."""

    def at(self, jd: float) -> list[DashaPeriod]:
        """The periods running at a Julian day (UTC), from the mahadasha
        down to `depth`; empty before birth and past the end of the cycle."""
        return _chain_at(self.periods, jd)


def _chain_at(periods: Sequence[DashaPeriod], jd: float) -> list[DashaPeriod]:
    """The periods running at a Julian day (UTC), from the mahadasha down.

    Depth first order means a period's children follow it, so one walk
    that takes the next level's running period finds the chain.
    """
    chain: list[DashaPeriod] = []
    for period in periods:
        if period.level == len(chain) + 1 and period.span.from_jd <= jd < period.span.to_jd:
            chain.append(period)
    return chain


def _periods(cells: Any, start: int, count: int, signed: Callable[[int], bool]) -> Tuple[DashaPeriod, ...]:
    """A dasha's periods from a period section — the births'
    `dasha_periods` or the years' `year_dasha_periods`, which share a
    layout — so a period is decoded in one place.

    A period's path is its index below the nearest earlier period one level
    up, so it is rebuilt by truncating the path to the level before it.
    """
    periods: list[DashaPeriod] = []
    path: list[str] = []
    for i in range(start, start + count):
        level = cells.level[i]
        del path[level - 1 :]
        path.append(str(cells.index[i]))
        periods.append(
            DashaPeriod(
                path="/".join(path),
                level=level,
                sign=Rashi(cells.sign[i]) if signed(i) else None,
                lord=Graha(cells.lord[i]),
                span=Interval(from_jd=cells.from_jd[i], to_jd=cells.to_jd[i]),
            )
        )
    return tuple(periods)


@dataclass(frozen=True)
class AnnualDashaShare:
    """One lord of the ring a year's dasha runs round."""

    lord: Graha
    """Its lord: the graha, or the sign's lord when the share is a sign's."""

    sign: Optional[Rashi]
    """The sign, when the share is one's: the Patyayini's lagna; None for a
    planet's."""

    weight: float
    """Its weight, of which a lord's share of the year is its weight over
    the ring's: a nakshatra year's lord's natal years, or a Patyayini
    share's patyamsha in nanoarcseconds. 0 for a lord that runs for no
    time."""


@dataclass(frozen=True)
class DashaRing:
    """The lords a year's dasha runs round, and where it opens."""

    shares: Tuple[AnnualDashaShare, ...]
    """In the order the ring runs."""

    first: int
    """The place in `shares` the year opens with, from 0."""

    remaining: Optional[float]
    """How much of the first lord's share was still to run at the return, 0
    to 1, the rest closing the year; None when it runs whole from the
    return: the Patyayini, and a `"whole"` balance."""


@dataclass(frozen=True)
class AnnualDasha:
    """One annual dasha of a year: its ring, the year it divides, and its
    periods (`03-design/annual-dashas.md`)."""

    system: DashaSystem
    """Which of the three: `PATYAYINI`, `MUDDA` or `VARSHA_YOGINI`."""

    seed: Optional[Nakshatra]
    """The birth Moon's nakshatra, which seeds a nakshatra year; None for
    the Patyayini."""

    ring: DashaRing
    """The lords the year runs round."""

    year: Interval
    """The year: from its return to where the clock closes it, under the
    default clock the next return."""

    periods: Tuple[DashaPeriod, ...]
    """Every period to the rules' depth, depth first in time order; a
    period that runs for no time is not listed."""

    @property
    def first_lord(self) -> Graha:
        """The lord the year opens with."""
        return self.ring.shares[self.ring.first].lord

    def at(self, jd: float) -> list[DashaPeriod]:
        """The periods running at a Julian day (UTC), from the mahadasha
        down; empty outside the year."""
        return _chain_at(self.periods, jd)


@dataclass(frozen=True)
class VargaPlacement:
    """Where one body stands in a divisional chart."""

    rashi: Rashi
    """The sign the body stands in, in the rashi chart."""

    part: int
    """Which part of that sign it falls in, counted from zero."""

    sign: Rashi
    """The sign the divisional chart puts it in."""

    @property
    def keeps_its_sign(self) -> bool:
        """Whether the divisional chart leaves the body where it was.

        In the navamsha this is **vargottama**, the term the texts use;
        in another chart it is the same fact without the name.
        """
        return self.sign == self.rashi


@dataclass(frozen=True)
class PlacedInVarga:
    """Where one graha stands in a divisional chart."""

    graha: Graha
    """Which graha."""

    at: VargaPlacement
    """Where it stands."""


@dataclass(frozen=True)
class VargaChart:
    """One divisional chart of one founded moment."""

    varga: Varga
    """Which divisional chart."""

    lagna: VargaPlacement
    """Where the lagna falls in it."""

    grahas: list[PlacedInVarga]
    """Every graha, in the order the foundation carries them."""


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


@dataclass(frozen=True)
class UnitPoint:
    """A point in a drawing's unit square, y downwards."""

    x: float
    """From the left edge, 0 to 1."""

    y: float
    """From the top edge, 0 to 1."""


@dataclass(frozen=True)
class LineSegment:
    """A straight line to a point."""

    to: UnitPoint
    """Where the line ends."""


@dataclass(frozen=True)
class QuadSegment:
    """A quadratic curve to a point, pulled towards its control."""

    control: UnitPoint
    """The control point."""

    to: UnitPoint
    """Where the curve ends."""


@dataclass(frozen=True)
class ArcSegment:
    """A circular arc about a centre to a point the same distance from it."""

    centre: UnitPoint
    """The circle's centre."""

    clockwise: bool
    """Which way the arc runs, as a reader sees it."""

    to: UnitPoint
    """Where the arc ends."""


Segment = Union[LineSegment, QuadSegment, ArcSegment]
"""One step of an outline, from wherever the previous step ended."""


@dataclass(frozen=True)
class Outline:
    """A closed outline: a start and the steps back to it."""

    start: UnitPoint
    """Where the outline starts."""

    segments: list[Segment]
    """The steps around it."""


@dataclass(frozen=True)
class DrawnCell:
    """One region of a drawn chart."""

    outline: Outline
    """The region's outline in the unit square."""

    sign: Rashi
    """The sign the cell shows; for a house between cusps, its cusp's sign."""

    house: int
    """The house the cell shows, 1 to 12."""

    lagna: bool
    """Whether the lagna stands in this cell."""

    ring: int
    """The ring, innermost 0; a grid's cells are all 0."""

    label: UnitPoint
    """Where the sign or house number is drawn."""

    anchor: UnitPoint
    """Where the cell's bodies are stacked about."""

    bodies: list[str]
    """The bodies in the cell, as catalogue keys (`graha.SUN`)."""


@dataclass(frozen=True)
class DrawnMark:
    """A body drawn at its own degree on a wheel."""

    body: str
    """The body, as a catalogue key."""

    ring: int
    """The ring it is drawn in."""

    at: UnitPoint
    """Where it is drawn."""

    longitude_deg: float
    """The longitude that put it there, degrees."""


class UnitPointRow(TypedDict):
    """A point in the unit square, as a row spells it."""

    x: float
    y: float


class SegmentRow(TypedDict, total=False):
    """One step of an outline: `kind` is `line`, `quad` or `arc`, with
    `to`, and a `quad`'s `control` or an `arc`'s `centre` and `clockwise`."""

    kind: Literal["line", "quad", "arc"]
    to: UnitPointRow
    control: UnitPointRow
    centre: UnitPointRow
    clockwise: bool


class OutlineRow(TypedDict):
    """A closed outline: its start and the steps back to it."""

    start: UnitPointRow
    segments: List[SegmentRow]


class HoldsRow(TypedDict):
    """What a grid cell always carries: `{"kind": "sign", "value": "ARIES"}`
    or `{"kind": "house", "value": 1}`."""

    kind: Literal["sign", "house"]
    value: Union[str, int]


class LayoutCellRow(TypedDict):
    """One region of a grid layout."""

    outline: OutlineRow
    holds: HoldsRow
    label: UnitPointRow
    bodies: UnitPointRow


class LayoutRingRow(TypedDict):
    """One ring of a radial layout."""

    inner: float
    outer: float
    counts_from: Literal["lagna", "moon", "sun", "cusps", "zodiac"]


class LayoutShapeRow(TypedDict, total=False):
    """Twelve cells fixed in the row (`kind` `grid`: `cells`, `frame`), or
    rings computed per chart (`kind` `radial`: `rings`, `starts_at`); both
    carry `direction`."""

    kind: Literal["grid", "radial"]
    cells: List[LayoutCellRow]
    frame: List[OutlineRow]
    rings: List[LayoutRingRow]
    starts_at: int
    direction: Literal["clockwise", "anticlockwise"]


class _VarshaRequestRequired(TypedDict):
    through: int


class VarshaRequest(_VarshaRequestRequired, total=False):
    """The annual charts a request asks for: how many years, and which
    longitude the Sun returns to (`03-design/annual-chart.md`).

    `reading` is `"sidereal"` (the tradition's, and the default),
    `"tropical"` (the Western solar return, most of a circle of lagna away
    at forty years, so a choice and never a fallback) or `"mean"` (a whole
    sidereal year each time, the older arithmetic).

    >>> asked: VarshaRequest = {"reading": "sidereal", "through": 40}
    """

    reading: Literal["sidereal", "tropical", "mean"]

    muntha: Literal["sign_start", "natal_degree"]
    """Where the Muntha stands inside the sign it has reached (crux C107).
    `"sign_start"` (the default, and the source's own reading) enters each
    year at the sign's first degree; `"natal_degree"` carries the natal
    lagna's degree across. Both give the same sign."""

    place: Union[Literal["birth"], "VarshaPlace"]
    """Where each year's own chart is cast, when you want the charts and not
    only their instants: `"birth"`, or a residence in the parts `found`
    takes. **Absent, none is founded** — the SDK does not choose between the
    birthplace and a residence for you, because the schools differ."""

    varshesha: "VarsheshaRules"
    """The readings the year lord's chain parts on, where authorities
    differ; the source's own by default."""

    matters: Union[Literal["all"], List[int]]
    """The matters each year's sixteen Tajika yogas are judged for:
    `"all"`, or house numbers 1 to 12 in the order you want them answered.
    Fourteen of the sixteen are judgements about the lagnesha and the lord
    of the house asked about, so they answer a matter and not a chart.
    **Needs `place`**; absent, none is judged."""

    yogas: "YogaRules"
    """The readings the sixteen part on, where the source leaves a choice;
    its own by default."""

    sahams: Union[Literal["all"], List[Union["Saham", str]]]
    """The sahams each year's chart is read for: `"all"`, the forty-one in
    the source's order, or `Saham` members (or their keys, `"punya"`) in
    the order you want them answered. **Needs `place`**; absent, none is
    read."""

    saham_rules: "SahamRules"
    """The readings the sahams part on, where the sources differ; the
    source's own by default."""

    saham_strength: "SahamStrengthReadings"
    """The readings a saham's strength parts on; the chapter's by
    default."""

    harsha_rules: "HarshaRules"
    """The Harsha bala's reading of Venus's house of joy."""

    dashas: Union[Literal["all"], List[Union["DashaSystem", str]]]
    """The annual dashas each year is divided by: `"all"`, the three in the
    catalogue's order, or `DashaSystem.PATYAYINI`, `MUDDA` and
    `VARSHA_YOGINI` (or their keys) in the order you want them answered.
    The Sun is read over a year once however many are asked for. **Needs
    `place`**; absent, none is read."""

    dasha_rules: "AnnualDashaRules"
    """The readings the annual dashas part on, where the sources differ;
    theirs by default."""


class AnnualDashaRules(TypedDict, total=False):
    """Where the sources differ on an annual dasha, each a named reading
    (`03-design/annual-dashas.md`).

    >>> rules: AnnualDashaRules = {"clock": {"days": 360}, "depth": 1}
    """

    clock: Union[Literal["sun_degrees", "even"], "YearOfDays"]
    """What a unit of the year is (crux C122): `"sun_degrees"`, the Sun's
    motion through one degree and the source's own, so the year closes on
    the next return; an `"even"` share of the time between the returns; or
    `{"days": n}`, the whole year as so many civil days from the return,
    the printed durations (360, and 365 for the Patyayini)."""

    balance: Literal["natal_moon", "entry_moon", "whole"]
    """Where a nakshatra year's balance comes from (crux C123): what
    remained of the birth Moon's nakshatra, the source's own; the Moon's at
    the return; or none."""

    measure: Literal["SPATIAL", "TEMPORAL"]
    """How the balance is measured, by arc or by time; absent, each
    balance's source's own: by arc for the birth Moon, by time for the Moon
    at the return."""

    birth_period: Literal["COMPRESSED", "ELAPSED"]
    """How the first lord's two pieces are divided among sub-lords, as the
    natal birth period's; `"COMPRESSED"` by default."""

    depth: int
    """How many levels the periods go down: 2, mahadashas and antardashas,
    by default."""


class YearOfDays(TypedDict):
    """A year of so many civil days from the return."""

    days: float


class SahamStrengthReadings(TypedDict, total=False):
    """Where the sources differ on a saham's strength
    (`03-design/tajika-saham-strength.md`).

    >>> readings: SahamStrengthReadings = {"natures": "parashari"}
    """

    natures: Literal["chapter", "parashari"]
    """Which planets are benefic and malefic: the chapter's own, the Sun a
    malefic among them, or the catalogue's Parashari natures."""

    friendship: Literal["positional", "natural"]
    """Tajika's positional friendship, the source's, or the catalogue's
    natural one."""

    weak_below: int
    """The Vishwa bala below which a saham's lord is weak, in **sub-sub
    units**, 3600 to a unit: `5 * 3600` by default."""


class HarshaRules(TypedDict, total=False):
    """Where the sources differ on the Harsha bala
    (`03-design/tajika-harsha.md`)."""

    venus: Literal["fifth", "twelfth"]
    """Venus's house of joy: the verse's fifth, or the twelfth a widely
    used program reads."""


class SahamRules(TypedDict, total=False):
    """Where the sources differ on a saham, each a named reading
    (`03-design/tajika-sahams.md`).

    >>> rules: SahamRules = {"add_sign": "signs", "houses": "equal"}
    """

    add_sign: Literal["degrees", "signs", "never"]
    """When a saham is carried a sign further: when c does not fall between
    b and a by degrees, the source's own; by whole signs, as a widely used
    program reads it; or never."""

    houses: Literal["sripati", "chalit", "equal"]
    """Where a house's point stands: Sripati's mid-point built from the
    angles, the source's own; the chart's chalit under its profile; or
    equal houses from the lagna's degree."""

    roga: Literal["lagna", "saturn"]
    """Roga's formula: lagna − Moon + lagna, or the other authority's
    Saturn − Moon + lagna."""


class YogaRules(TypedDict, total=False):
    """Where the source leaves the sixteen Tajika yogas a choice, each a
    named reading (`03-design/tajika-yogas.md`).

    >>> rules: YogaRules = {"tambira": "either_lord", "strong_from": 12 * 3600}
    """

    drishti: "DrishtiRules"
    """How a pair less than a degree past reads (crux C112)."""

    weak_below: int
    """The strength below which a planet with no dignity is weak, in
    **sub-sub units**, 3600 to a unit: `5 * 3600` by default (crux C116)."""

    strong_from: int
    """The strength from which a planet is strong, sub-sub units; `10 * 3600`
    by default."""

    tambira: Literal["karyesha", "either_lord"]
    """Which lord a Tambira lets reach the next sign: the definition's
    karyesha by default, or either, the source's "some authorities"."""

    moon_benefic: Literal["always", "waxing"]
    """When the Moon counts among Kuttha's benefics: always by default,
    Charak's list, or waxing, the commentary's "full Moon" (crux C117)."""


class DrishtiRules(TypedDict, total=False):
    """How the Tajika aspects read a pair less than a degree past."""

    sub_degree: Literal["poorna", "ishrafa"]
    """Poorna, the default, or Ishrafa (crux C112)."""


class VarsheshaRules(TypedDict, total=False):
    """Where the sources differ on the lord of the year, each a named
    reading (`03-design/varshesha.md`).

    >>> rules: VarsheshaRules = {"moon": "like_any_other"}
    """

    none_aspects: Literal["muntha_lord", "annual_lagna_lord", "strongest"]
    """Who takes the year when nobody aspects the lagna: the Muntha's lord
    by default, the annual lagna's lord, or the Nilakanthi's strongest of
    the five."""

    tied: Literal["muntha_lord", "dina_ratri_pati"]
    """Who takes it on an outright tie."""

    moon: Literal["passed_over", "ithasala", "like_any_other"]
    """Whether the Moon may hold it: passed over by default, stepping down
    to the next claimant and else to its Ithasala successor; the
    Nilakanthi's successor at once; or like any other."""

    moon_partner: Literal["any_planet", "office_bearer"]
    """Who may succeed the Moon: any planet by default, or only an
    office-bearer."""

    drishti: "DrishtiRules"
    """How the Ithasala the Moon's successor needs is read."""


class VarshaPlace(TypedDict):
    """A residence to cast each year's chart for: the observer and clock
    `found` itself takes.

    >>> home: VarshaPlace = {"observer": Observer(latitude_deg=Latitude(28.6139),
    ...     longitude_deg=Longitude(77.209), altitude_m=Altitude(216)),
    ...     "utc_offset_seconds": 19800}
    """

    observer: Observer
    utc_offset_seconds: int


@dataclass(frozen=True)
class OfficeBearers:
    """The annual chart's five office-bearers, one of whom becomes the lord
    of the year."""

    muntha: Graha
    """The lord of the Muntha's sign."""

    janma_lagna: Graha
    """The lord of the birth lagna."""

    varsha_lagna: Graha
    """The lord of the annual lagna."""

    tri_rashi: Graha
    """The annual lagna's triplicity lord for the part of the day."""

    dina_ratri: Graha
    """The lord of the Sun's sign by day, of the Moon's by night."""


@dataclass(frozen=True)
class Bala:
    """A Tajika strength, exact. The boundary carries it as an integer
    count of **sub-sub units**, 3600 to a unit, because two office-bearers
    a sub-sub unit apart decide a year between them."""

    units: int
    """Whole units, at most twenty: the figure a reader compares."""

    sub_units: int
    """The sub-units after those, 0 to 59."""

    sub_sub: int
    """The sub-sub units after those, 0 to 59."""

    total: int
    """The whole of it in sub-sub units: what to compare and sum."""

    def __str__(self) -> str:
        """`14:20:15`, as the sources write one."""
        return f"{self.units:02}:{self.sub_units:02}:{self.sub_sub:02}"


@dataclass(frozen=True)
class YearClaim:
    """One office-bearer's claim on the year's lordship."""

    graha: Graha
    """Whose claim it is."""

    vishwa: Bala
    """Its five-fold strength."""

    portfolios: int
    """How many of the five offices it holds, 1 to 5: the tie-break."""

    aspects_lagna: bool
    """Whether it gives the Tajika aspect to the annual lagna, which it must
    to hold the year."""


@dataclass(frozen=True)
class YearLord:
    """The lord of the year, and the reckoning it came out of."""

    graha: Graha
    """The lord of the year."""

    chosen: VarsheshaChosen
    """Which step of the chain decided it."""

    vishwa: Bala
    """Its five-fold strength."""

    moon_passed_over: bool
    """Whether the Moon led on strength and stepped aside, being "unable to
    govern"."""

    claims: List[YearClaim]
    """Every claimant, strongest first, so the decision can be read rather
    than trusted."""


@dataclass(frozen=True)
class TajikaPair:
    """Two planets of an annual chart, and what they make."""

    faster: Graha
    """The faster of the two by the tradition's ranking."""

    slower: Graha
    """The slower."""

    drishti: TajikaDrishti
    """The aspect between the signs they stand in."""

    yoga: TajikaYoga
    """What they are doing: coming together or drawing apart."""

    orb_deg: float
    """The orb governing them, degrees: the mean of their deeptamshas."""

    apart_deg: float
    """How far apart within their signs, degrees; positive when the faster
    is behind the slower and coming to it."""


@dataclass(frozen=True)
class TajikaBetween:
    """Two planets of an annual chart, and what they make — which may be
    nothing."""

    faster: Graha
    """The faster of the two by the tradition's ranking."""

    slower: Graha
    """The slower."""

    drishti: TajikaDrishti
    """The aspect between the signs they stand in."""

    yoga: Optional[TajikaYoga]
    """What they are doing; `None` when they make neither an Ithasala nor an
    Ishrafa."""

    orb_deg: float
    """The orb governing them, degrees: the mean of their deeptamshas."""

    apart_deg: float
    """How far apart within their signs, degrees; positive when the faster
    is behind the slower and coming to it."""


@dataclass(frozen=True)
class Afflictions:
    """The two lords' afflictions, clause by clause: what made a Rudda or a
    Durapha."""

    lagnesha: List[Affliction]
    """The lagnesha's."""

    karyesha: List[Affliction]
    """The karyesha's."""


@dataclass(frozen=True)
class HeldYearYoga:
    """One of the sixteen holding, with what made it hold."""

    yoga: YearYoga
    """Which of the sixteen."""

    between: Optional[TajikaBetween]
    """The lords' own relation, where that is what made it."""

    through: Optional[Graha]
    """The third planet it turns on, where one does."""

    entering: Optional[Graha]
    """The planet judged on entering the next sign: Gairi-Kamboola's Moon,
    Tambira's lord."""

    legs: Optional[Tuple[TajikaBetween, TajikaBetween]]
    """How the third planet stands to each of the pair, read from the next
    sign for `entering`."""

    afflictions: Optional[Afflictions]
    """The lords' afflictions, where those made it: Rudda and Durapha."""


@dataclass(frozen=True)
class TajikaMatter:
    """The sixteen Tajika yogas for one matter of a year: the question it
    asked as well as the answer, because a list of yogas whose pair a reader
    cannot see is not checkable."""

    house: int
    """The house asked about, 1 to 12, counted from the annual lagna."""

    sign: Rashi
    """The sign that house falls in."""

    lagnesha: Graha
    """The lord of the annual lagna."""

    karyesha: Graha
    """The lord of the house asked about."""

    same_lord: bool
    """One planet is both — always so of the first house — so there is no
    pair to judge."""

    between: Optional[TajikaBetween]
    """How the two lords stand to each other; `None` when they are one."""

    held: List[HeldYearYoga]
    """Every yoga that holds, once for each third planet that makes it."""

    unanswered: List[YearYoga]
    """The yogas this call could not answer for. A yoga absent from `held`
    did not hold **only** if it is not listed here."""

    def holds(self, yoga: YearYoga) -> Optional[bool]:
        """Whether `yoga` holds: `None` where this call could not say, which
        is not the same answer as `False`."""
        if yoga in self.unanswered:
            return None
        return any(one.yoga == yoga for one in self.held)


@dataclass(frozen=True)
class AnnualChart:
    """A return's own chart, read down to what Tajika reads from it."""

    lagna_deg: float
    """The annual chart's lagna, sidereal degrees, at the place it was cast
    for."""

    by_day: bool
    """Whether the return fell between sunrise and sunset there."""

    office_bearers: OfficeBearers
    """The five office-bearers."""

    year_lord: YearLord
    """The lord of the year, chosen among them."""

    yogas: List[TajikaPair]
    """The pairs of the seven that make a Tajika yoga in this chart. The
    pairs that make none do not cross; Rust's `sdk.chart().drishtis` has all
    twenty-one."""

    retrograde: List[Graha]
    """The seven retrograde in this chart: what the matters were judged on."""

    combust: List[Graha]
    """The seven combust in this chart, under the context's combustion
    table."""

    matters: List[TajikaMatter]
    """The sixteen yogas for each matter `varsha["matters"]` asked about, in
    its order; empty otherwise."""

    sahams: List["TajikaSaham"]
    """Each saham `varsha["sahams"]` asked for, in its order, with its
    strength under the year's lord; empty otherwise."""

    harsha: List["HarshaBala"]
    """The seven's Harsha bala in this year's chart, in the catalogue's
    order."""

    dashas: List["AnnualDasha"]
    """Each annual dasha `varsha["dashas"]` asked for, in its order, under
    `varsha["dasha_rules"]`; empty otherwise."""


@dataclass(frozen=True)
class TajikaSaham:
    """Where a saham fell in a year's chart, and what it fell in."""

    saham: Saham
    """Which of the forty-one."""

    longitude_deg: float
    """Where it fell, sidereal degrees in [0, 360)."""

    sign: Rashi
    """The sign it fell in."""

    lord: Graha
    """That sign's lord: the saham's lord, by whose strength the source
    judges it."""

    house: int
    """The house it fell in, 1 to 12, by whole signs from the annual
    lagna."""

    added_sign: bool
    """Whether it was carried a sign further because c did not fall between
    b and a."""

    strong: List[SahamStrong]
    """The clauses of the source's strong list that hold. Reported and
    never weighed: the source gives no score, and three sahams in five
    meet clauses on both lists."""

    weak: List[SahamWeak]
    """The clauses of the source's weak list that hold."""

    lord_vishwa: "Bala"
    """The saham lord's Panchavargiya Vishwa bala."""

    lord_harsha: HarshaGrade
    """The saham lord's Harsha bala grade."""

    in_node_axis: Optional[bool]
    """Whether it stands in the Rahu-Ketu axis; `None` when the chart
    placed no nodes."""

    seven: List["SahamSeven"]
    """How each of the seven stands to it, in the catalogue's order."""

    @property
    def handicapped(self) -> bool:
        """In the 6th, 8th or 12th, where the source calls a saham
        handicapped."""
        return self.house in (6, 8, 12)


@dataclass(frozen=True)
class SahamSeven:
    """How one of the seven stands to a saham."""

    graha: Graha
    """Which planet."""

    drishti: TajikaDrishti
    """The Tajika aspect its sign casts on the saham's."""

    relation: TajikaRelation
    """How it stands to the saham's lord, under the friendship read."""

    company: bool
    """Whether it keeps the saham company, in the saham's sign."""


@dataclass(frozen=True)
class HarshaBala:
    """One planet's Harsha bala: four places it is happy in, five units
    each (`03-design/tajika-harsha.md`)."""

    graha: Graha
    """Whose."""

    house: int
    """The house it stands in, whole signs from the annual lagna."""

    sthana: bool
    """In its house of joy."""

    uchcha_swakshetra: bool
    """In its exaltation or own sign."""

    stri_purusha: bool
    """In a house of its own gender, Tajika's genders."""

    dina_ratri: bool
    """In a year opening at its own part of the day."""

    total: int
    """The parts held, five units each: 0 to 20."""

    grade: HarshaGrade
    """What the source calls that total."""


@dataclass(frozen=True)
class Muntha:
    """The Muntha at one return: the birth lagna's sign advanced one sign
    for each completed year, and that sign's lord."""

    sign: Rashi
    """The sign it has reached; the same under either reading."""

    lord: Graha
    """The lord of that sign: the Munthesha, first of the annual chart's
    five office-bearers and the one that takes the year's lordship when no
    other qualifies."""

    longitude_deg: float
    """Its longitude at the return, degrees, under the reading asked for.
    It advances thirty degrees over the year."""


@dataclass(frozen=True)
class Pravesha:
    """One annual chart's instant."""

    year: int
    """How many years the native has completed at this instant: 1 is the
    first return, a year after birth. Counted in returns and not in years
    of life, because the two namings differ by one and both are in use."""

    instant: float
    """The instant, a Julian day (UTC), to pass to `found`."""

    muntha: Muntha
    """The Muntha standing at it, progressed by this year's own count."""

    annual: Optional[AnnualChart]
    """The year's own chart, or `None` unless `varsha=` named a `place`."""


class DashaLord(TypedDict):
    """One lord of a dasha system and its whole years."""

    graha: str
    years: int


class _UduDashaDefinitionRequired(TypedDict):
    kernel: Literal["udu"]
    key: str
    lords: List[DashaLord]
    reference: str


class UduDashaDefinition(_UduDashaDefinitionRequired, total=False):
    """A nakshatra-seeded dasha system of your own, as `dasha_systems` takes
    it: its key, its lords in order and the reference nakshatra, bare keys
    (`"SUN"`, `"KRITTIKA"`) as the document spells them; every other field
    defaults to Vimshottari's shape (`03-design/dasha-kernels.md`).

    >>> saptaka: UduDashaDefinition = {
    ...     "kernel": "udu",
    ...     "key": "ACME_SAPTAKA",
    ...     "lords": [{"graha": g, "years": 10} for g in ("SUN", "MOON", "MARS")],
    ...     "reference": "KRITTIKA",
    ... }
    """

    sources: List[str]
    count: str
    span: int
    offset: int
    repeats: bool
    scale: Dict[str, int]
    year_length: str
    depth: int


class _RashiDashaDefinitionRequired(TypedDict):
    kernel: Literal["rashi"]
    key: str


class RashiDashaDefinition(_RashiDashaDefinitionRequired, total=False):
    """A sign-based (Jaimini) dasha system of your own: its key, and
    optionally where it starts, the order it visits the signs in, how long a
    sign runs, which lord a mahadasha names, and the houses to start from
    the strongest of. Everything unsaid is Chara's
    (`03-design/dasha-kernels.md`).

    >>> sthira: RashiDashaDefinition = {
    ...     "kernel": "rashi",
    ...     "key": "ACME_STHIRA",
    ...     "length": {"by_modality": {"movable": 7, "fixed": 8, "dual": 9}},
    ... }
    """

    sources: List[str]
    start: Literal["lagna", "arudha_lagna", "navamsa_lagna"]
    order: Literal["consecutive", "trine_groups", "drishti_chain", "leap"]
    length: Union[str, Dict[str, object]]
    namedLord: Literal["stronger", "first"]
    strongerOf: List[int]
    year_length: str
    depth: int


DashaDefinition = Union[UduDashaDefinition, RashiDashaDefinition]
"""A dasha system of your own, of either kernel. Which one runs it is
**stated** in `kernel` and never guessed from the fields present, so a typo
is refused by the field you wrote rather than by one you did not."""


class _RowsJson(NamedTuple):
    """The consumer's own rows a context registers, serialised."""

    layouts: Optional[str]
    dashas: Optional[str]


class LayoutRow(TypedDict):
    """A chart layout as a row: its key, what cites it, and its shape.
    Crosses as JSON with the SDK's own field names, as a theme does
    (`03-design/chart-geometry.md` §7f)."""

    key: str
    sources: List[str]
    shape: LayoutShapeRow


class ThemeStyle(TypedDict, total=False):
    """How a drawing looks: every field optional, over the theme it extends
    (`03-design/render-svg.md`)."""

    size: float
    background: str
    ink: str
    cell: str
    lagna_cell: str
    accent: str
    stroke: float
    font_family: str
    body_size: float
    label_size: float
    mark_size: float
    advance: float
    line_height: float
    baseline_shift: float


class ThemeContent(TypedDict, total=False):
    """What a drawing says: every field optional, over the theme it extends."""

    body_form: Literal["short", "glyph"]
    cell_label: Literal["auto", "sign_number", "sign_short", "sign_glyph", "house", "nothing"]
    lagna_mark: bool
    retrograde_mark: Optional[str]
    degrees: bool


class ThemeRecord(TypedDict, total=False):
    """A theme naming only what it changes, over the light theme or the
    shipped one `extends` names."""

    extends: Literal["light", "dark"]
    style: ThemeStyle
    content: ThemeContent


Theme = Union[Literal["light", "dark"], ThemeRecord]
"""The theme a request writes its drawings as SVG in: a shipped theme's
name, or a record naming only what it changes."""


def _theme_json(theme: Optional[Theme]) -> Optional[str]:
    """The theme as the JSON the boundary reads, or nothing for no SVG."""
    if theme is None:
        return None
    if isinstance(theme, str):
        return json.dumps({"extends": theme})
    if isinstance(theme, Mapping):
        return json.dumps(theme)
    raise TeistroError(
        Status.INVALID_ARG,
        "a theme is 'light', 'dark' or a theme record",
        field="theme",
    )


ShippedRules = Literal["doshas", "yogas", "gandantas", "arishtas", "readings", "nabhasas"]
"""A set of rules the SDK ships."""


class RuleRequest(TypedDict, total=False):
    """The rules a request asks a chart to answer
    (`03-design/rules-at-the-boundary.md`): shipped sets by name and a
    consumer's own rules in the SDK's rule format."""

    shipped: List[ShippedRules]
    rules: List[Mapping[str, Any]]
    readings: Literal["texts", "recording-engine"]
    houses: bool
    longevity: bool


class RulesReading(TypedDict, total=False):
    """What a chart answers by rule, as the SDK writes it: `present`, each
    `{rule, result}` with the rule by key, and `houses` and `longevity` when
    asked. A maraka result is a vulnerability, never a date."""

    present: List[Dict[str, Any]]
    houses: List[Dict[str, Any]]
    longevity: Dict[str, Any]
    unreadable: List[str]


class PlanRequest(TypedDict, total=False):
    """The narrative plans a request asks a chart for
    (`03-design/plans-at-the-boundary.md`). Each composer is off by default,
    and `readings` needs `rules` beside it, since it says what the rules a
    chart held answered."""

    placements: bool
    readings: bool
    strength: bool
    houses: bool
    positions: bool
    aspects: bool
    conditions: bool
    karakas: bool
    chalit: bool
    phala: bool
    bhavaBala: bool
    vimshopaka: bool
    panchanga: bool
    states: bool
    dashaPhala: bool
    ashtakavarga: bool


class PlanItem(TypedDict):
    """One thing to say: a message key and its slots. The slots are the very
    mapping `intl.render` takes, so `sdk.intl.render(item["key"],
    item["params"])` says it, in whatever locale the context is in."""

    key: str
    params: Dict[str, Any]


class Plans(TypedDict, total=False):
    """What a chart has to say, holding no words: the composers asked for,
    and only those."""

    placements: List[PlanItem]
    readings: List[PlanItem]
    strength: List[PlanItem]
    houses: List[PlanItem]
    positions: List[PlanItem]
    aspects: List[PlanItem]
    conditions: List[PlanItem]
    karakas: List[PlanItem]
    chalit: List[PlanItem]
    phala: List[PlanItem]
    bhavaBala: List[PlanItem]
    vimshopaka: List[PlanItem]
    panchanga: List[PlanItem]
    states: List[PlanItem]
    dashaPhala: List[PlanItem]
    ashtakavarga: List[PlanItem]


def _rules_json(rules: Optional[RuleRequest]) -> Optional[str]:
    """The rules as the JSON the boundary reads, or nothing for none."""
    return _record_json(rules, "rules", "{'shipped': ['nabhasas']}")


def _interpret_json(interpret: Optional[PlanRequest]) -> Optional[str]:
    """The plans as the JSON the boundary reads, or nothing for none."""
    return _record_json(interpret, "interpret", "{'placements': True}")


def _varsha_json(varsha: Optional[VarshaRequest]) -> Optional[str]:
    """The annual charts as the JSON the boundary reads, or nothing for none.

    A residence is taken in the parts `found` takes and written in the
    boundary's words; a word crosses as written, so a wrong one is refused
    by the SDK, by `varsha_json.place`, as in every binding."""
    if not isinstance(varsha, Mapping):
        return _record_json(varsha, "varsha", "{'reading': 'sidereal', 'through': 40}")
    written: Dict[str, Any] = dict(varsha)
    place = written.get("place")
    if isinstance(place, Mapping):
        written["place"] = _annual_place(place)
    # The rule records are written in Python's own keys and read in the
    # boundary's, as the place is.
    for snake, camel in (
        ("saham_rules", "sahamRules"),
        ("saham_strength", "sahamStrength"),
        ("harsha_rules", "harshaRules"),
        ("dasha_rules", "dashaRules"),
    ):
        if snake in written:
            written[camel] = written.pop(snake)
    for rules in ("varshesha", "yogas", "sahamRules", "sahamStrength", "harshaRules", "dashaRules"):
        if isinstance(written.get(rules), Mapping):
            written[rules] = _camel_keys(written[rules])
    # A saham crosses as its key, the spelling it is read back in.
    sahams = written.get("sahams")
    if isinstance(sahams, (list, tuple)):
        written["sahams"] = [one.key if isinstance(one, Saham) else one for one in sahams]
    # And a dasha system as its full key, the spelling it is read back in.
    dashas = written.get("dashas")
    if isinstance(dashas, (list, tuple)):
        written["dashas"] = [one.full_key if isinstance(one, DashaSystem) else one for one in dashas]
    return _record_json(written, "varsha", "{'through': 40, 'place': 'birth'}")


def _camel_keys(record: Mapping[str, Any]) -> Dict[str, Any]:
    """A rule record's keys in the boundary's casing, all the way down: the
    values, which are the readings' own words, are left as written. A key
    this does not know is translated all the same, so the SDK refuses it by
    the name it crosses as."""
    def camel(key: str) -> str:
        head, *rest = key.split("_")
        return head + "".join(part[:1].upper() + part[1:] for part in rest)

    return {
        camel(key): _camel_keys(value) if isinstance(value, Mapping) else value
        for key, value in record.items()
    }


def _annual_place(place: Mapping[str, Any]) -> Dict[str, Any]:
    """A residence in the boundary's words. A key this does not know is
    passed through rather than dropped, so the SDK refuses it by name."""
    rest = {key: value for key, value in place.items()
            if key not in ("observer", "utc_offset_seconds")}
    observer = place.get("observer")
    if not isinstance(observer, Observer):
        raise TypeError("varsha['place']['observer']: expected an Observer")
    return {
        **rest,
        "latitudeDeg": float(observer.latitude_deg),
        "longitudeDeg": float(observer.longitude_deg),
        "altitudeM": float(observer.altitude_m),
        "utcOffsetSeconds": place.get("utc_offset_seconds"),
    }


class _Starts(NamedTuple):
    """Where each row's block starts in each ragged section under the
    annual charts: `starts[row]` to `starts[row + 1]`. Computed once for a
    chart's years rather than once a year, so reading them is linear."""

    claims: List[int]
    yogas: List[int]
    matters: List[int]
    held: List[int]
    legs: List[int]
    sahams: List[int]
    dashas: List[int]
    shares: List[int]
    periods: List[int]

    @staticmethod
    def of(decoded: Any) -> "_Starts":
        def running(counts: Sequence[int]) -> List[int]:
            return [0, *itertools.accumulate(counts)]

        charts = decoded.annual_charts
        return _Starts(
            claims=running(charts.claim_count),
            yogas=running(charts.yoga_count),
            matters=running(charts.matter_count),
            held=running(decoded.year_matters.held_count),
            legs=running(decoded.matter_yogas.leg_count),
            sahams=running(charts.saham_count),
            dashas=running(charts.dasha_count),
            shares=running(decoded.year_dashas.share_count),
            periods=running(decoded.year_dashas.period_count),
        )


def _between(section: Any, k: int, prefix: str = "") -> TajikaBetween:
    """How two planets stand, from a section carrying the pair columns: a
    matter's lords (under `pair_`) or a yoga's legs."""
    def at(name: str) -> Any:
        return getattr(section, prefix + name)

    return TajikaBetween(
        faster=Graha(at("faster")[k]),
        slower=Graha(at("slower")[k]),
        drishti=TajikaDrishti(at("drishti")[k]),
        yoga=TajikaYoga(at("yoga")[k]) if at("yoga_present")[k] == 1 else None,
        orb_deg=at("orb_deg")[k],
        apart_deg=at("apart_deg")[k],
    )


def _members(bits: int, of: Any) -> List[Any]:
    """The members of a bit set over a small closed enum, in id order: bit
    `n` is the member with id `n`."""
    return [member for member in of if bits & (1 << int(member))]


def _matters(decoded: Any, row: int, starts: _Starts) -> List[TajikaMatter]:
    """A year's matters, each with its question, the lords' pair, what it
    could not answer, and every yoga that held — ragged three deep
    (`03-design/tajika-yogas.md`, "Crossing the boundary")."""
    matters = decoded.year_matters
    held = decoded.matter_yogas
    found = []
    for m in range(starts.matters[row], starts.matters[row + 1]):
        between = None if matters.same_lord[m] == 1 else _between(matters, m, "pair_")
        yogas = [
            HeldYearYoga(
                yoga=YearYoga(held.yoga[h]),
                between=between if held.by_pair[h] == 1 else None,
                through=Graha(held.through[h]) if held.through_present[h] == 1 else None,
                entering=Graha(held.entering[h]) if held.entering_present[h] == 1 else None,
                legs=(
                    (_between(decoded.matter_legs, starts.legs[h]),
                     _between(decoded.matter_legs, starts.legs[h] + 1))
                    if held.leg_count[h] == 2
                    else None
                ),
                afflictions=(
                    Afflictions(
                        lagnesha=_members(held.lagnesha_afflictions[h], Affliction),
                        karyesha=_members(held.karyesha_afflictions[h], Affliction),
                    )
                    if held.afflictions_present[h] == 1
                    else None
                ),
            )
            for h in range(starts.held[m], starts.held[m + 1])
        ]
        found.append(
            TajikaMatter(
                house=matters.house[m],
                sign=Rashi(matters.sign[m]),
                lagnesha=Graha(matters.lagnesha[m]),
                karyesha=Graha(matters.karyesha[m]),
                same_lord=matters.same_lord[m] == 1,
                between=between,
                held=yogas,
                unanswered=_members(matters.unanswered[m], YearYoga),
            )
        )
    return found


def _annual_dashas(decoded: Any, row: int, starts: _Starts) -> List[AnnualDasha]:
    """A year's annual dashas, ragged by `dasha_count`, each with its ring
    and its periods ragged under it by `share_count` and `period_count`
    (`03-design/annual-dashas.md`)."""
    rows = decoded.year_dashas
    shares = decoded.year_dasha_shares
    cells = decoded.year_dasha_periods
    found = []
    for k in range(starts.dashas[row], starts.dashas[row + 1]):
        remaining = rows.remaining[k]
        found.append(
            AnnualDasha(
                system=DashaSystem(rows.system[k]),
                seed=Nakshatra(rows.seed[k]) if rows.seeded[k] == 1 else None,
                ring=DashaRing(
                    shares=tuple(
                        AnnualDashaShare(
                            lord=Graha(shares.lord[i]),
                            sign=Rashi(shares.sign[i]) if shares.has_sign[i] == 1 else None,
                            weight=shares.weight[i],
                        )
                        for i in range(starts.shares[k], starts.shares[k + 1])
                    ),
                    first=rows.first[k],
                    remaining=None if math.isnan(remaining) else remaining,
                ),
                year=Interval(from_jd=rows.from_jd[k], to_jd=rows.to_jd[k]),
                periods=_periods(
                    cells,
                    starts.periods[k],
                    rows.period_count[k],
                    lambda i: cells.has_sign[i] == 1,
                ),
            )
        )
    return found


def _sahams(decoded: Any, row: int, starts: _Starts) -> List[TajikaSaham]:
    """A year's sahams, ragged by `saham_count`
    (`03-design/tajika-sahams.md`)."""
    return [
        _saham_at(decoded.year_sahams, decoded.year_saham_seven, k)
        for k in range(starts.sahams[row], starts.sahams[row + 1])
    ]


def _saham_at(cols: Any, seven: Any, k: int) -> TajikaSaham:
    """One saham row, from the years' sections or the births': where it
    fell, its strength clause by clause, and the seven rows under it. One
    decoder for both, so they cannot drift
    (`03-design/tajika-saham-strength.md`)."""
    axis = cols.node_axis[k]
    return TajikaSaham(
        saham=Saham(cols.saham[k]),
        longitude_deg=cols.longitude_deg[k],
        sign=Rashi(cols.sign[k]),
        lord=Graha(cols.lord[k]),
        house=cols.house[k],
        added_sign=cols.added_sign[k] == 1,
        strong=_members(cols.strong[k], SahamStrong),
        weak=_members(cols.weak[k], SahamWeak),
        lord_vishwa=_bala(cols.lord_vishwa[k]),
        lord_harsha=HarshaGrade(cols.lord_harsha[k]),
        # 2 is the boundary's "the chart placed no nodes to read".
        in_node_axis=None if axis == 2 else axis == 1,
        seven=[
            SahamSeven(
                graha=Graha(seven.graha[at]),
                drishti=TajikaDrishti(seven.drishti[at]),
                relation=TajikaRelation(seven.relation[at]),
                company=seven.company[at] == 1,
            )
            for at in range(7 * k, 7 * k + 7)
        ],
    )


def _harsha(decoded: Any, row: int) -> List[HarshaBala]:
    """A founded year's Harsha bala: seven rows a year, fixed, in the
    catalogue's order (`03-design/tajika-harsha.md`)."""
    h = decoded.year_harsha
    return [
        HarshaBala(
            graha=Graha(h.graha[at]),
            house=h.house[at],
            sthana=h.sthana[at] == 1,
            uchcha_swakshetra=h.uchcha_swakshetra[at] == 1,
            stri_purusha=h.stri_purusha[at] == 1,
            dina_ratri=h.dina_ratri[at] == 1,
            total=h.total[at],
            grade=HarshaGrade(h.grade[at]),
        )
        for at in range(7 * row, 7 * row + 7)
    ]


def _annual_chart(decoded: Any, row: int, starts: _Starts) -> Optional[AnnualChart]:
    """Row `row` of `annual_charts`, which runs beside `praveshas` row for
    row or is empty; anything between is a layout this layer cannot pair,
    and it says so rather than giving a year another year's chart."""
    charts = decoded.annual_charts
    if len(charts.lagna_deg) == 0:
        return None
    if len(charts.lagna_deg) != len(decoded.praveshas.year):
        raise RuntimeError(
            f"annual_charts has {len(charts.lagna_deg)} rows beside "
            f"{len(decoded.praveshas.year)} returns; it is all of them or none"
        )
    # The claims are ragged by `claim_count`, as the returns are by
    # `pravesha_count`: this year's block starts where the ones before end.
    start = starts.claims[row]
    count = charts.claim_count[row]
    claims = decoded.year_claims
    yoga_start = starts.yogas[row]
    pairs = decoded.year_yogas
    return AnnualChart(
        lagna_deg=charts.lagna_deg[row],
        by_day=charts.daylight[row] == 1,
        office_bearers=OfficeBearers(
            muntha=Graha(decoded.praveshas.muntha_lord[row]),
            janma_lagna=Graha(charts.janma_lagna_lord[row]),
            varsha_lagna=Graha(charts.varsha_lagna_lord[row]),
            tri_rashi=Graha(charts.tri_rashi_lord[row]),
            dina_ratri=Graha(charts.dina_ratri_lord[row]),
        ),
        year_lord=YearLord(
            graha=Graha(charts.year_lord[row]),
            chosen=VarsheshaChosen(charts.year_lord_chosen[row]),
            vishwa=_bala(charts.year_lord_vishwa[row]),
            moon_passed_over=charts.moon_passed_over[row] == 1,
            claims=[
                YearClaim(
                    graha=Graha(claims.graha[i]),
                    vishwa=_bala(claims.vishwa[i]),
                    portfolios=claims.portfolios[i],
                    aspects_lagna=claims.aspects_lagna[i] == 1,
                )
                for i in range(start, start + count)
            ],
        ),
        yogas=[
            TajikaPair(
                faster=Graha(pairs.faster[i]),
                slower=Graha(pairs.slower[i]),
                drishti=TajikaDrishti(pairs.drishti[i]),
                yoga=TajikaYoga(pairs.yoga[i]),
                orb_deg=pairs.orb_deg[i],
                apart_deg=pairs.apart_deg[i],
            )
            for i in range(yoga_start, yoga_start + charts.yoga_count[row])
        ],
        retrograde=_members(charts.retrograde[row], _SEVEN),
        combust=_members(charts.combust[row], _SEVEN),
        matters=_matters(decoded, row, starts),
        sahams=_sahams(decoded, row, starts),
        harsha=_harsha(decoded, row),
        dashas=_annual_dashas(decoded, row, starts),
    )


#: The seven the Tajika bit sets range over, Sun to Saturn: graha ids 0 to 6.
_SEVEN = [Graha(n) for n in range(7)]


def _bala(sub_sub: int) -> Bala:
    """A strength from the integer sub-sub units the boundary carries."""
    units, rest = divmod(sub_sub, 3600)
    return Bala(units=units, sub_units=rest // 60, sub_sub=rest % 60, total=sub_sub)


def _record_json(value: Optional[Mapping[str, Any]], field: str, example: str) -> Optional[str]:
    """A request option that crosses as a JSON record, written down; nothing
    where it was not given. Anything else is refused here, named and shown,
    rather than across the boundary."""
    if value is None:
        return None
    if isinstance(value, Mapping):
        return json.dumps(value)
    raise TeistroError(
        Status.INVALID_ARG,
        f"{field} is a request record, such as {example}",
        field=field,
    )


@dataclass(frozen=True)
class Drawing:
    """A chart drawn in a layout (`03-design/chart-geometry.md`)."""

    layout: Union[ChartLayout, str]
    """The layout it is drawn in: a `ChartLayout`, or a layout the context
    registered, by its full key (`chart_layout.ACME_KERALA`)."""

    @property
    def layout_key(self) -> str:
        """The layout's full key, shipped or registered
        (`chart_layout.NORTH_INDIAN`), for a caller that reads either."""
        return self.layout.full_key if isinstance(self.layout, ChartLayout) else self.layout

    varga: Varga
    """Which chart: `Varga.D1` for the founded chart, or a divisional one."""

    cells: list[DrawnCell]
    """The cells, in the layout's order."""

    frame: list[Outline]
    """The lines drawn that hold nothing."""

    marks: list[DrawnMark]
    """Each body at its own degree, on a wheel; empty for a grid."""

    svg: Optional[str] = None
    """The drawing as SVG, in the request's theme and the context's locale;
    `None` when the request gave no theme."""


def _member(kind: Any, key: str) -> Any:
    """A catalogue member by its bare key, or a refusal naming both."""
    found = kind.by_key(key)
    if found is None:
        raise TeistroError(Status.INTERNAL, f"the library drew a {kind.__name__} this build does not know: {key}")
    return found


def _point(raw: Mapping[str, Any]) -> UnitPoint:
    return UnitPoint(x=float(raw["x"]), y=float(raw["y"]))


def _segment(raw: Mapping[str, Any]) -> Segment:
    kind = raw["kind"]
    if kind == "line":
        return LineSegment(to=_point(raw["to"]))
    if kind == "quad":
        return QuadSegment(control=_point(raw["control"]), to=_point(raw["to"]))
    return ArcSegment(centre=_point(raw["centre"]), clockwise=bool(raw["clockwise"]), to=_point(raw["to"]))


def _outline(raw: Mapping[str, Any]) -> Outline:
    return Outline(start=_point(raw["start"]), segments=[_segment(step) for step in raw["segments"]])


def _layout_of(key: str) -> Union[ChartLayout, str]:
    """A drawn layout: the shipped member, or a registered one's full key."""
    shipped = ChartLayout.by_key(key)
    return shipped if isinstance(shipped, ChartLayout) else f"chart_layout.{key}"


def _drawing(raw: Mapping[str, Any], svg: Optional[str]) -> Drawing:
    placed = raw["placed"]
    return Drawing(
        svg=svg,
        layout=_layout_of(placed["layout"]),
        varga=_member(Varga, raw["varga"]),
        cells=[
            DrawnCell(
                outline=_outline(cell["outline"]),
                sign=_member(Rashi, cell["sign"]),
                house=int(cell["house"]),
                lagna=bool(cell["lagna"]),
                ring=int(cell["ring"]),
                label=_point(cell["label"]),
                anchor=_point(cell["anchor"]),
                bodies=list(cell["bodies"]),
            )
            for cell in placed["cells"]
        ],
        frame=[_outline(path) for path in placed["frame"]],
        marks=[
            DrawnMark(
                body=mark["body"],
                ring=int(mark["ring"]),
                at=_point(mark["at"]),
                longitude_deg=float(mark["longitude_deg"]),
            )
            for mark in placed["marks"]
        ],
    )


def _dasha_ids(dashas: Sequence[Union[DashaSystem, str]], registered: Mapping[str, int]) -> list[int]:
    """The dashas asked for, as the ids the boundary takes: a `DashaSystem`,
    or the `dasha_system.*` key of a system this context registered
    (`03-design/dasha-kernels.md`)."""
    ids = []
    for at, system in enumerate(dashas):
        if isinstance(system, DashaSystem):
            ids.append(int(system))
        elif isinstance(system, str) and system in registered:
            ids.append(registered[system])
        else:
            raise TeistroError(
                Status.INVALID_ARG,
                f"dashas[{at}] is not a DashaSystem, or the dasha_system.* key of a system this "
                "context registered",
                field=f"dashas[{at}]",
            )
    return ids


def _dasha_system(id: int, names: Mapping[int, str]) -> Union[DashaSystem, str]:
    """A dasha row's system: the catalogue's member, or a registered one's key."""
    if id in names:
        return names[id]
    return DashaSystem(id)


def _drawing_bits(
    drawings: Sequence[Tuple[Union[ChartLayout, str], Varga]], registered: Mapping[str, int]
) -> list[int]:
    """The drawings asked for, as the packed ids the boundary takes:
    `layout << 16 | varga` each, so a caller names pairs and nothing else
    writes bits (`03-design/chart-geometry.md`).

    A layout is a `ChartLayout`, or a consumer's own by its full key
    (`chart_layout.ACME_KERALA`), from the ids its context resolved when it
    was made (§7f)."""
    bits = []
    for at, pair in enumerate(drawings):
        layout, varga = pair if isinstance(pair, tuple) and len(pair) == 2 else (None, None)
        if isinstance(layout, str):
            member = registered.get(layout, -1)
        elif isinstance(layout, ChartLayout):
            member = int(layout)
        else:
            member = -1
        if member < 0 or not isinstance(varga, Varga):
            raise TeistroError(
                Status.INVALID_ARG,
                f"drawings[{at}] is not a (ChartLayout, or the chart_layout.* key of a layout this "
                "context registered, Varga) pair",
                field=f"drawings[{at}]",
            )
        bits.append((member << 16) | int(varga))
    return bits


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
    def day_part(self) -> DayPart:
        """Which arc of its day the instant falls in.

        This and `day_elapsed` belong to the **instant**, not to the day,
        so they are the chart's rather than the day section's — which is
        what the panchanga blob's arrival settled.
        """
        return DayPart(self.batch.decoded.cast.day_part[self.index])

    @property
    def day_elapsed(self) -> float:
        """How far through that arc the instant is, 0 to 1."""
        return self.batch.decoded.cast.day_elapsed[self.index]

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
    def states(self) -> list[GrahaState]:
        """What each graha **is**, as opposed to where it is — or an
        empty list unless `state=True` asked for it.

        The motion is not here: `grahas[j].retrograde` already says it.
        """
        decoded = self.batch.decoded
        columns = decoded.states
        if len(columns.graha) == 0:
            return []
        count = decoded.graha_count
        base = self.index * count

        def members(bits: int) -> list[AvasthaLajjitadi]:
            # `>= 0` skips the generated `UNKNOWN = -1` sentinel every
            # catalogue enum carries: a bit set is over the members the
            # catalogue has, and shifting by a negative is an error
            # rather than a miss.
            return [one for one in AvasthaLajjitadi if one.id >= 0 and bits & (1 << one.id)]

        return [
            GrahaState(
                graha=Graha(columns.graha[i]),
                sign=Rashi(columns.sign[i]),
                house=columns.house[i],
                dignity=Dignity(columns.dignity[i]),
                friendship=Friendship(
                    natural=Relationship(columns.natural[i]),
                    temporary=Relationship(columns.temporary[i]),
                    compound=Relationship(columns.compound[i]),
                    dispositor=Graha(columns.dispositor[i])
                    if columns.has_dispositor[i]
                    else None,
                ),
                combustion=Combustion(
                    burning=Burning(columns.burning[i]),
                    from_sun_deg=columns.from_sun_deg[i] if columns.has_from_sun[i] else None,
                    orb_deg=columns.orb_deg[i] if columns.has_orbs[i] else None,
                    deep_orb_deg=columns.deep_orb_deg[i] if columns.has_deep_orb[i] else None,
                ),
                age=AvasthaBaladi(columns.age[i]),
                wakefulness=AvasthaJagradadi(columns.wakefulness[i]),
                deeptadi=AvasthaDeeptadi(columns.deeptadi[i])
                if columns.has_deeptadi[i]
                else None,
                lajjitadi=Lajjitadi(
                    holding=members(columns.lajjitadi_holding[i]),
                    ruled_out=members(columns.lajjitadi_ruled_out[i]),
                    undecided=members(columns.lajjitadi_undecided[i]),
                ),
                war=War(
                    opponent=Graha(columns.war_opponent[i]),
                    is_winner=bool(columns.war_won[i]),
                    apart_deg=columns.war_apart_deg[i],
                )
                if columns.has_war[i]
                else None,
                sayanadi=Sayanadi(
                    avastha=AvasthaSayanadi(columns.sayanadi[i]),
                    cheshtas=tuple(
                        AvasthaCheshta(column[i])
                        for column in (
                            columns.cheshta_1,
                            columns.cheshta_2,
                            columns.cheshta_3,
                            columns.cheshta_4,
                            columns.cheshta_5,
                        )
                    ),
                )
                if columns.has_sayanadi[i]
                else None,
                boundaries=EdgeDistance(
                    sign_deg=columns.sign_deg[i],
                    nakshatra_deg=columns.nakshatra_deg[i],
                    pada_deg=columns.pada_deg[i],
                ),
            )
            for i in range(base, base + count)
        ]

    @property
    def bhavas(self) -> list[ServiceBhava]:
        """The twelve bhavas as the houses service reads them, or an
        empty list unless `houses=True` asked for them."""
        columns = self.batch.decoded.bhavas
        if len(columns.sign) == 0:
            return []
        base = self.index * 12
        return [
            ServiceBhava(
                number=j + 1,
                sign=Rashi(columns.sign[base + j]),
                lord=Graha(columns.lord[base + j]),
                quadrant=Quadrant(columns.quadrant[base + j]),
            )
            for j in range(12)
        ]

    @property
    def points(self) -> list[DerivedPoint]:
        """The derived points — the upagrahas and the special lagnas — or
        an empty list unless `points=True` asked for them.

        Gulika and Mandi are Saturn's eighth of the day's arc, so they
        are the two a chart with no arc to divide cannot have — which is
        why the section is ragged.
        """
        decoded = self.batch.decoded
        counts = decoded.cast.point_count
        start = sum(counts[i] for i in range(self.index))
        columns = decoded.points
        return [
            DerivedPoint(
                point=Point(columns.point[i]),
                longitude_deg=columns.longitude_deg[i],
                sign=Rashi(columns.sign[i]),
                boundaries=EdgeDistance(
                    sign_deg=columns.sign_deg[i],
                    nakshatra_deg=columns.nakshatra_deg[i],
                    pada_deg=columns.pada_deg[i],
                ),
            )
            for i in range(start, start + counts[self.index])
        ]

    @property
    def sahams(self) -> List[TajikaSaham]:
        """The birth chart's own sahams, each with its strength clause by
        clause — which has no year lord — in the order `varsha["sahams"]`
        named them; empty unless it asked. The source reads a year's
        sahams beside these, and they need no place
        (`03-design/tajika-saham-strength.md`)."""
        decoded = self.batch.decoded
        counts = decoded.cast.natal_saham_count
        start = sum(counts[i] for i in range(self.index))
        return [
            _saham_at(decoded.natal_sahams, decoded.natal_saham_seven, k)
            for k in range(start, start + counts[self.index])
        ]

    @property
    def praveshas(self) -> List[Pravesha]:
        """The annual charts' instants: the Sun's returns to where it stood
        at birth, in year order; empty unless `varsha=` asked for them
        (`03-design/annual-chart.md`).

        The section is **ragged** for a reason of its own: the request
        settles how many returns are *wanted* and the ephemeris settles how
        many there *are*, so read the length rather than the number you
        asked for.

        The place is yours. A return is an instant, and whether the annual
        chart is cast for the birthplace or for a residence is a choice the
        schools differ on, so pass the instant to `found` yourself.
        """
        decoded = self.batch.decoded
        counts = decoded.cast.pravesha_count
        start = sum(counts[i] for i in range(self.index))
        columns = decoded.praveshas
        starts = _Starts.of(decoded)
        return [
            Pravesha(
                year=columns.year[i],
                instant=columns.jd[i],
                muntha=Muntha(
                    sign=Rashi(columns.muntha_sign[i]),
                    lord=Graha(columns.muntha_lord[i]),
                    longitude_deg=columns.muntha_deg[i],
                ),
                annual=_annual_chart(decoded, i, starts),
            )
            for i in range(start, start + counts[self.index])
        ]

    @property
    def aspects(self) -> list[Drishti]:
        """The drishti this chart casts, strongest first among those a
        body casts; empty unless `aspects=True` asked for them.

        The section is **ragged**: a chart's relations depend on where
        the bodies stand rather than on how many there are, so two charts
        of the same nine grahas hold different numbers of them, and
        `cast.aspect_count` is what says where each chart's begin.
        """
        decoded = self.batch.decoded
        counts = decoded.cast.aspect_count
        start = sum(counts[i] for i in range(self.index))
        columns = decoded.aspects
        return [
            Drishti(
                from_graha=Graha(columns.from_[i]),
                to=Graha(columns.to[i]),
                houses=columns.houses[i],
                strength=Strength(columns.strength[i]),
                from_edge=EdgeDistance(
                    sign_deg=columns.from_sign_deg[i],
                    nakshatra_deg=columns.from_nakshatra_deg[i],
                    pada_deg=columns.from_pada_deg[i],
                ),
                to_edge=EdgeDistance(
                    sign_deg=columns.to_sign_deg[i],
                    nakshatra_deg=columns.to_nakshatra_deg[i],
                    pada_deg=columns.to_pada_deg[i],
                ),
            )
            for i in range(start, start + counts[self.index])
        ]

    @property
    def ashtakavarga(self) -> Optional[Ashtakavarga]:
        """The Ashtakavarga, when `ashtakavarga=True` asked for it."""
        parsed = self.batch._ashtakavargas
        return parsed[self.index] if self.index < len(parsed) else None

    @property
    def bhava_bala(self) -> Optional[BhavaBala]:
        """The Bhava bala, when `bhava_bala=True` asked for it."""
        parsed = self.batch._bhava_balas
        return parsed[self.index] if self.index < len(parsed) else None

    @property
    def shadbala(self) -> Optional[Shadbala]:
        """The Shadbala, when `shadbala=True` asked for it."""
        parsed = self.batch._shadbalas
        return parsed[self.index] if self.index < len(parsed) else None

    @property
    def dasha_phala(self) -> Optional[DashaPhalaReading]:
        """The dasha phala, when `dasha_phala=True` asked for it."""
        parsed = self.batch._dasha_phalas
        return parsed[self.index] if self.index < len(parsed) else None

    @property
    def vaiseshikamsa(self) -> Optional[VaiseshikamsaReading]:
        """The Vaiseshikamsa, when `vaiseshikamsa=True` asked for it."""
        parsed = self.batch._vaiseshikamsas
        return parsed[self.index] if self.index < len(parsed) else None

    @property
    def vimshopaka(self) -> Optional[Vimshopaka]:
        """The Vimshopaka, when `vimshopaka=True` asked for it."""
        parsed = self.batch._vimshopakas
        return parsed[self.index] if self.index < len(parsed) else None

    @property
    def dashas(self) -> list[Dasha]:
        """The dashas asked for, in the order asked; empty unless `dashas`
        named some (`03-design/dasha-kernels.md`)."""
        parsed = self.batch._dashas
        return parsed[self.index] if self.index < len(parsed) else []

    @property
    def rules(self) -> Optional[RulesReading]:
        """What this chart answers by rule; `None` unless the request named
        rules (`03-design/rules-at-the-boundary.md`)."""
        parsed = self.batch._rules
        return parsed[self.index] if self.index < len(parsed) else None

    @property
    def plans(self) -> Optional[Plans]:
        """What this chart has to say, as the composers wrote it: a key per
        composer the request asked for, each a list of `{key, params}`
        holding no words at all — and only those the request named. An
        item's `params` are the mapping `intl.render`
        takes, so it says itself in the context's locale — and the same plan
        says it in any other (`03-design/plans-at-the-boundary.md`). `None`
        unless the request named a composer."""
        parsed = self.batch._plans
        return parsed[self.index] if self.index < len(parsed) else None

    @property
    def drawings(self) -> list[Drawing]:
        """The charts drawn in the layouts asked for, in the order asked;
        empty unless `drawings` named some (`03-design/chart-geometry.md`).

        Each cell carries both the sign and the house it shows, and the
        bodies standing in it; `marks` places each body at its own degree
        on a wheel and is empty for a grid.
        """
        parsed = self.batch._drawings
        return parsed[self.index] if self.index < len(parsed) else []

    @property
    def vargas(self) -> list[VargaChart]:
        """The divisional charts asked for, in the order they were asked.

        Empty unless `vargas` named some: a caller who wants a birth
        chart does not pay for twenty-one of them
        (`03-design/chart-reading.md` §4).
        """
        decoded = self.batch.decoded
        count = decoded.varga_count
        graha_count = decoded.graha_count
        charts = decoded.vargas
        placed = decoded.varga_grahas
        out: list[VargaChart] = []
        for at in range(count):
            row = self.index * count + at
            base = row * graha_count
            out.append(
                VargaChart(
                    varga=Varga(charts.varga[row]),
                    lagna=VargaPlacement(
                        rashi=Rashi(charts.lagna_rashi[row]),
                        part=charts.lagna_part[row],
                        sign=Rashi(charts.lagna_sign[row]),
                    ),
                    grahas=[
                        PlacedInVarga(
                            graha=Graha(decoded.grahas.graha[self.index * graha_count + j]),
                            at=VargaPlacement(
                                rashi=Rashi(placed.rashi[base + j]),
                                part=placed.part[base + j],
                                sign=Rashi(placed.sign[base + j]),
                            ),
                        )
                        for j in range(graha_count)
                    ],
                )
            )
        return out

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

    def __init__(self, decoded: Charts, dasha_names: Optional[Mapping[int, str]] = None) -> None:
        self.decoded = decoded
        """The blob as its generated decoder read it."""
        self.dasha_names: Mapping[int, str] = dasha_names or {}
        """The full key of each dasha system the founding context registered,
        by its id; a batch read without it names such a system by its id."""

    def __len__(self) -> int:
        """How many charts the batch holds."""
        return self.decoded.chart_count

    @cached_property
    def _rules(self) -> list[RulesReading]:
        """Every chart's answers by rule, parsed once; empty when none were asked for."""
        text = self.decoded.rules
        return json.loads(text) if text else []

    @cached_property
    def _plans(self) -> list[Plans]:
        """Every chart's narrative plans, parsed once; empty when no composer
        was asked for."""
        text = self.decoded.plans
        return json.loads(text) if text else []

    @cached_property
    def _drawings(self) -> list[list[Drawing]]:
        """Every chart's drawings, parsed once however many charts read them."""
        text = self.decoded.drawings
        if not text:
            return []
        written: list[list[str]] = json.loads(self.decoded.svgs) if self.decoded.svgs else []
        return [
            [
                _drawing(raw, written[chart][index] if chart < len(written) else None)
                for index, raw in enumerate(drawings)
            ]
            for chart, drawings in enumerate(json.loads(text))
        ]

    @cached_property
    def _ashtakavargas(self) -> list[Ashtakavarga]:
        """Every chart's Ashtakavarga, decoded once; empty when none was asked for."""
        decoded = self.decoded
        rows = decoded.ashtakavarga
        bins = decoded.ashtakavarga_bindus
        sums = decoded.sarvashtakavarga

        def twelve(column: Any, start: int) -> Tuple[int, ...]:
            return tuple(column[start : start + 12])

        out: list[Ashtakavarga] = []
        for chart in range(rows.length // 7):
            grahas = []
            for g in range(7):
                row = chart * 7 + g
                each = Shodhana(rows.shodhana[row]) is Shodhana.EACH_GRAHA
                grahas.append(
                    GrahaAshtakavarga(
                        graha=Graha(rows.graha[row]),
                        bindus=twelve(bins.bindus, row * 12),
                        reduced=twelve(bins.reduced, row * 12) if each else None,
                        rashi_pinda=rows.rashi_pinda[row],
                        graha_pinda=rows.graha_pinda[row],
                        yoga_pinda=rows.yoga_pinda[row],
                    )
                )
            out.append(
                Ashtakavarga(
                    shodhana=Shodhana(rows.shodhana[chart * 7]),
                    ekadhipatya=Ekadhipatya(rows.ekadhipatya[chart * 7]),
                    grahas=tuple(grahas),
                    sarva=twelve(sums.sarva, chart * 12),
                    trikona=twelve(sums.trikona, chart * 12),
                    reduced=twelve(sums.reduced, chart * 12),
                )
            )
        return out

    @cached_property
    def _bhava_balas(self) -> list[BhavaBala]:
        """Every chart's Bhava bala, decoded once; empty when none was asked for."""
        c = self.decoded.bhava_bala
        return [
            BhavaBala(
                bhavas=tuple(
                    BhavaStrength(
                        bhava=h + 1,
                        lord=Graha(c.lord[chart * 12 + h]),
                        adhipati=c.adhipati[chart * 12 + h],
                        dig=c.dig[chart * 12 + h],
                        drishti=c.drishti[chart * 12 + h],
                        special=c.special[chart * 12 + h],
                        virupas=c.virupas[chart * 12 + h],
                    )
                    for h in range(12)
                )
            )
            for chart in range(c.length // 12)
        ]

    @cached_property
    def _shadbalas(self) -> list[Shadbala]:
        """Every chart's Shadbala, decoded once; empty when none was asked for."""
        c = self.decoded.shadbala

        def graha(row: int) -> GrahaShadbala:
            return GrahaShadbala(
                graha=Graha(c.graha[row]),
                sthana=SthanaBala(
                    uchcha=c.uchcha[row],
                    saptavargaja=c.saptavargaja[row],
                    ojayugma=c.ojayugma[row],
                    kendradi=c.kendradi[row],
                    drekkana=c.drekkana[row],
                ),
                dig=c.dig[row],
                kaala=KaalaBala(
                    nathonnatha=c.nathonnatha[row],
                    paksha=c.paksha[row],
                    tribhaga=c.tribhaga[row],
                    abda=c.abda[row],
                    masa=c.masa[row],
                    vara=c.vara[row],
                    hora=c.hora[row],
                    ayana=c.ayana[row],
                    yuddha=c.yuddha[row],
                ),
                cheshta=c.cheshta[row],
                naisargika=c.naisargika[row],
                drik=c.drik[row],
                virupas=c.virupas[row],
                rupas=c.rupas[row],
                required_rupas=c.required_rupas[row],
                strong=c.strong[row] == 1,
                ishta=c.ishta[row],
                kashta=c.kashta[row],
                subha_rashmi=c.subha_rashmi[row],
                ashubha_rashmi=c.ashubha_rashmi[row],
            )

        return [
            Shadbala(grahas=tuple(graha(row) for row in range(chart * 7, chart * 7 + 7)))
            for chart in range(c.length // 7)
        ]

    @cached_property
    def _dasha_phalas(self) -> list[DashaPhalaReading]:
        """Every chart's dasha phala, decoded once; empty when none was asked for."""
        c = self.decoded.dasha_phala
        subhankas = (
            c.subhanka_d1,
            c.subhanka_d2,
            c.subhanka_d3,
            c.subhanka_d7,
            c.subhanka_d9,
            c.subhanka_d12,
            c.subhanka_d30,
        )
        return [
            DashaPhalaReading(
                grahas=tuple(
                    GrahaDashaPhala(
                        graha=Graha(c.graha[row]),
                        subhankas=tuple(column[row] for column in subhankas),
                        subhanka=c.subhanka[row],
                        asubhanka=c.asubhanka[row],
                        nature=Nature(c.nature[row]),
                        phase=DashaPhase(c.phase[row]),
                        favourable=c.favourable[row] == 1,
                        unfavourable=c.unfavourable[row] == 1,
                    )
                    for row in range(chart * 9, chart * 9 + 9)
                )
            )
            for chart in range(c.length // 9)
        ]

    @cached_property
    def _vaiseshikamsas(self) -> list[VaiseshikamsaReading]:
        """Every chart's Vaiseshikamsa, decoded once; empty when none was asked for."""
        c = self.decoded.vaiseshikamsa

        def standing(good: memoryview, names: memoryview, row: int) -> VaiseshikamsaStanding:
            count = good[row]
            return VaiseshikamsaStanding(
                good_vargas=count, name=Vaiseshikamsa(names[row]) if count >= 2 else None
            )

        return [
            VaiseshikamsaReading(
                grahas=tuple(
                    GrahaVaiseshikamsa(
                        graha=Graha(c.graha[row]),
                        shadvarga=standing(c.shadvarga_good, c.shadvarga_name, row),
                        saptavarga=standing(c.saptavarga_good, c.saptavarga_name, row),
                        dashavarga=standing(c.dashavarga_good, c.dashavarga_name, row),
                        shodashavarga=standing(c.shodashavarga_good, c.shodashavarga_name, row),
                        impaired=c.impaired[row] == 1,
                    )
                    for row in range(chart * 7, chart * 7 + 7)
                )
            )
            for chart in range(c.length // 7)
        ]

    @cached_property
    def _vimshopakas(self) -> list[Vimshopaka]:
        """Every chart's Vimshopaka, decoded once; empty when none was asked for."""
        rows = self.decoded.vimshopaka
        return [
            Vimshopaka(
                scoring=VimshopakaScoring(rows.scoring[chart * 7]),
                grahas=tuple(
                    GrahaVimshopaka(
                        graha=Graha(rows.graha[row]),
                        shadvarga=rows.shadvarga[row],
                        saptavarga=rows.saptavarga[row],
                        dashavarga=rows.dashavarga[row],
                        shodashavarga=rows.shodashavarga[row],
                    )
                    for row in range(chart * 7, chart * 7 + 7)
                ),
            )
            for chart in range(rows.length // 7)
        ]

    @cached_property
    def _dashas(self) -> list[list[Dasha]]:
        """Every chart's dashas, decoded once however many charts read them.

        The periods are **ragged** by each dasha's `period_count`, so a
        chart's begin where the one before it ends.
        """
        decoded = self.decoded
        per = decoded.dasha_count
        if per == 0:
            return []
        out: list[list[Dasha]] = []
        start = 0
        for chart in range(decoded.dashas.length // per):
            row_dashas: list[Dasha] = []
            for j in range(per):
                row = chart * per + j
                count = decoded.dashas.period_count[row]
                row_dashas.append(_dasha(decoded, row, start, count, self.dasha_names))
                start += count
            out.append(row_dashas)
        return out

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

def _dasha(decoded: Charts, row: int, start: int, count: int, names: Mapping[int, str]) -> Dasha:
    """One dasha row and its periods, in this layer's shape.

    A period's path is its index below the nearest earlier period one level
    up, so it is rebuilt by truncating the path to the level before it.
    """
    rows = decoded.dashas
    cells = decoded.dasha_periods
    seeded = rows.seeded[row] != 0
    signed = rows.signed[row] != 0
    periods = _periods(cells, start, count, lambda _: signed)
    span_from = rows.moon_span_from[row]
    return Dasha(
        system=_dasha_system(rows.system[row], names),
        seed=Nakshatra(rows.seed[row]) if seeded else None,
        first_lord=Graha(rows.first_lord[row]),
        overflow=rows.overflow[row] != 0,
        balance=DashaBalance(
            method=Balance(rows.balance[row]),
            remaining=rows.remaining[row],
            days=rows.balance_days[row],
            written=WrittenBalance(
                years=rows.balance_years[row],
                months=rows.balance_months[row],
                days=rows.balance_day_count[row],
                hours=rows.balance_hours[row],
                minutes=rows.balance_minutes[row],
            ),
        )
        if seeded
        else None,
        moon_span=None if math.isnan(span_from) else Interval(from_jd=span_from, to_jd=rows.moon_span_to[row]),
        depth=rows.depth[row],
        periods=periods,
    )


@dataclass(frozen=True)
class Interval:
    """A span of time, as every almanac row carries one."""

    from_jd: float
    """When it begins, as a Julian day (UTC)."""

    to_jd: float
    """When it ends."""


@dataclass(frozen=True)
class Span(Generic[T]):
    """One member of a limb, with its own bounds and the clipped ones."""

    member: T
    """Which member ran."""

    whole: Interval
    """When the member itself began and ended, inside the day or not."""

    inside: Interval
    """The part inside the day: what an almanac row prints."""


@dataclass(frozen=True)
class Month:
    """The lunar month a day falls in, under both conventions."""

    month: Masa
    """The month under the profile's own convention."""

    amanta: Masa
    """The amanta month: new moon to new moon."""

    purnimanta: Masa
    """The purnimanta month: full moon to full moon."""

    paksha: Paksha
    """Which fortnight the day opens in."""

    convention: LunarMonth
    """Which convention `month` leads with."""

    kind: MonthKind
    """Whether the month is ordinary, intercalary or omitted.

    The name above needs no case for the intercalary one — an adhika
    month and the nija month after it take the same name — so this is the
    mark beside the name.
    """


@dataclass(frozen=True)
class KaalaPeriod:
    """One inauspicious eighth of the daylight."""

    kaala: Kaala
    """Which one."""

    at: Interval
    """When it runs."""


@dataclass(frozen=True)
class ChoghadiyaPeriod:
    """One choghadiya, of the daylight or of the night."""

    choghadiya: Choghadiya
    """Which choghadiya."""

    lord: Graha
    """The graha that rules it."""

    at: Interval
    """When it runs."""

    daytime: bool
    """Whether it is one of the eight of the daylight."""


@dataclass(frozen=True)
class Hora:
    """One hora, from sunrise."""

    number: int
    """Its number, 1 to 24."""

    lord: Graha
    """The graha that rules it."""

    start: float
    """When it begins, as a Julian day (UTC)."""

    end: float
    """When it ends."""


@dataclass(frozen=True)
class Muhurta:
    """One of the thirty muhurtas."""

    at: Interval
    """When it runs."""

    daylight: bool
    """Whether it is one of the fifteen of the daylight."""


@dataclass(frozen=True)
class MoonEvent:
    """A moonrise or a moonset."""

    rise: bool
    """True for a rise, false for a set."""

    instant: float
    """When, as a Julian day (UTC)."""


@dataclass(frozen=True)
class HeldYoga:
    """A muhurta yoga that held, and what made it hold."""

    yoga: MuhurtaYoga
    """Which yoga."""

    at: Interval
    """While it held, clipped to the day."""

    vara: Vara
    """The vara that makes it; every cause has one."""

    tithi: Optional[Tithi]
    """The tithi that makes it, or `None` when the cause has none."""

    nakshatra: Nakshatra
    """The nakshatra that makes it."""


@dataclass(frozen=True)
class Abhijit:
    """Abhijit, with whether it is effective."""

    at: Interval
    """When it runs."""

    effective: bool
    """True on every day but a Wednesday."""


class AlmanacDay:
    """One day of an almanac: a view over its batch, not a copy."""

    def __init__(self, batch: "Almanac", index: int) -> None:
        self.batch = batch
        """The batch this day belongs to."""
        self.index = index
        """Where in that batch it sits."""

    @property
    def vara(self) -> Vara:
        """The weekday the day carries."""
        return Vara(self.batch.decoded.day.vara[self.index])

    @property
    def sunrise(self) -> float:
        """The sunrise that opened the day, as a Julian day (UTC)."""
        return self.batch.decoded.day.sunrise[self.index]

    @property
    def sunset(self) -> float:
        """The sunset that closed its daylight."""
        return self.batch.decoded.day.sunset[self.index]

    @property
    def window(self) -> Interval:
        """What the spans are clipped to."""
        days = self.batch.decoded.days
        return Interval(days.window_from[self.index], days.window_to[self.index])

    @property
    def month(self) -> Month:
        """The lunar month, under both conventions."""
        days = self.batch.decoded.days
        return Month(
            month=Masa(days.month[self.index]),
            amanta=Masa(days.amanta[self.index]),
            purnimanta=Masa(days.purnimanta[self.index]),
            paksha=Paksha(days.paksha[self.index]),
            convention=LunarMonth(self.batch.decoded.lunar_month),
            kind=MonthKind(days.month_kind[self.index]),
        )

    @property
    def ayana(self) -> Ayana:
        """Which half of the year the day falls in."""
        return Ayana(self.batch.decoded.days.ayana[self.index])

    @property
    def disha_shool(self) -> Direction:
        """The direction not to travel in, which is the vara's."""
        return Direction(self.batch.decoded.days.disha_shool[self.index])

    @property
    def sankranti(self) -> Optional[float]:
        """When the Sun entered a new sign inside the day, or `None`."""
        days = self.batch.decoded.days
        if not days.has_sankranti[self.index]:
            return None
        return days.sankranti[self.index]

    @property
    def abhijit(self) -> Optional[Abhijit]:
        """Abhijit; `None` on a day with no daylight."""
        days = self.batch.decoded.days
        if not days.has_abhijit[self.index]:
            return None
        return Abhijit(
            at=Interval(days.abhijit_from[self.index], days.abhijit_to[self.index]),
            effective=bool(days.abhijit_effective[self.index]),
        )

    @property
    def brahma(self) -> Optional[Interval]:
        """Brahma muhurta; `None` when the night before is not known."""
        days = self.batch.decoded.days
        if not days.has_brahma[self.index]:
            return None
        return Interval(days.brahma_from[self.index], days.brahma_to[self.index])

    @property
    def tithi(self) -> list[Span[Tithi]]:
        """The tithis that touch the day."""
        return self._spans("tithi", self.batch.decoded.tithi, Tithi)

    @property
    def nakshatra(self) -> list[Span[Nakshatra]]:
        """The nakshatras the Moon was in."""
        return self._spans("nakshatra", self.batch.decoded.nakshatra, Nakshatra)

    @property
    def yoga(self) -> list[Span[Yoga]]:
        """The nitya yogas."""
        return self._spans("yoga", self.batch.decoded.yoga, Yoga)

    @property
    def karana(self) -> list[Span[Karana]]:
        """The karanas: half-tithis, so three or four on an ordinary day."""
        return self._spans("karana", self.batch.decoded.karana, Karana)

    @property
    def panchaka(self) -> list[Span[Panchaka]]:
        """Panchaka, while the Moon is in the last five nakshatras."""
        return self._spans("panchaka", self.batch.decoded.panchaka, Panchaka)

    @property
    def moon_signs(self) -> list[Span[Rashi]]:
        """The signs the Moon stood in."""
        return self._spans("moon_signs", self.batch.decoded.moon_signs, Rashi)

    @property
    def sun_signs(self) -> list[Span[Rashi]]:
        """The signs the Sun stood in; two only on a sankranti day."""
        return self._spans("sun_signs", self.batch.decoded.sun_signs, Rashi)

    @property
    def kaalas(self) -> list[KaalaPeriod]:
        """The inauspicious eighths of the daylight."""
        columns = self.batch.decoded.kaalas
        return self._rows(
            "kaalas",
            lambda i: KaalaPeriod(
                kaala=Kaala(columns.kaala[i]),
                at=Interval(columns.from_[i], columns.to[i]),
            ),
        )

    @property
    def choghadiya(self) -> list[ChoghadiyaPeriod]:
        """Eight choghadiya of the daylight and eight of the night."""
        columns = self.batch.decoded.choghadiya
        return self._rows(
            "choghadiya",
            lambda i: ChoghadiyaPeriod(
                choghadiya=Choghadiya(columns.choghadiya[i]),
                lord=Graha(columns.lord[i]),
                at=Interval(columns.from_[i], columns.to[i]),
                daytime=bool(columns.daytime[i]),
            ),
        )

    @property
    def horas(self) -> list[Hora]:
        """The twenty-four horas, from sunrise."""
        columns = self.batch.decoded.horas
        return self._rows(
            "horas",
            lambda i: Hora(
                number=columns.number[i],
                lord=Graha(columns.lord[i]),
                start=columns.start[i],
                end=columns.end[i],
            ),
        )

    @property
    def muhurtas(self) -> list[Muhurta]:
        """The thirty muhurtas: fifteen of the daylight, then of the night."""
        columns = self.batch.decoded.muhurtas
        return self._rows(
            "muhurtas",
            lambda i: Muhurta(
                at=Interval(columns.from_[i], columns.to[i]),
                daylight=bool(columns.daylight[i]),
            ),
        )

    @property
    def moon_events(self) -> list[MoonEvent]:
        """Every moonrise and moonset inside the day's moon window."""
        columns = self.batch.decoded.moon_events
        return self._rows(
            "moon_events",
            lambda i: MoonEvent(
                rise=columns.kind[i] == 0, instant=columns.instant[i]
            ),
        )

    @property
    def muhurta_yogas(self) -> list[HeldYoga]:
        """The muhurta yogas that held, with what made each hold."""
        columns = self.batch.decoded.muhurta_yogas
        return self._rows(
            "muhurta_yogas",
            lambda i: HeldYoga(
                yoga=MuhurtaYoga(columns.yoga[i]),
                at=Interval(columns.from_[i], columns.to[i]),
                vara=Vara(columns.because_vara[i]),
                # A VARA_NAKSHATRA cause has no tithi, and the blob leaves
                # the column at nought rather than at a tithi that did not
                # make it.
                tithi=None
                if columns.because_kind[i] == 0
                else Tithi(columns.because_tithi[i]),
                nakshatra=Nakshatra(columns.because_nakshatra[i]),
            ),
        )

    def _rows(self, list_name: str, build: Any) -> list[Any]:
        start, end = self.batch.range(list_name, self.index)
        return [build(i) for i in range(start, end)]

    def _spans(self, list_name: str, columns: Any, member: Any) -> list[Any]:
        return self._rows(
            list_name,
            lambda i: Span(
                member=member(columns.member[i]),
                whole=Interval(columns.whole_from[i], columns.whole_to[i]),
                inside=Interval(columns.inside_from[i], columns.inside_to[i]),
            ),
        )


class Almanac:
    """A batch of daily panchangas at one place, read one day at a time.

    Every per-day list is concatenated across the batch, so a day's rows
    are found by adding up every earlier day's count. That sum is done
    **once**, when the batch is built, rather than per access: the
    alternative is quadratic over a year of days, which is the shape an
    almanac is actually asked for.
    """

    def __init__(self, decoded: Panchanga) -> None:
        self.decoded = decoded
        """The blob as its generated decoder read it."""
        counts = decoded.counts
        self._starts: dict[str, list[int]] = {}
        for name in (
            "tithi",
            "nakshatra",
            "yoga",
            "karana",
            "panchaka",
            "moon_signs",
            "sun_signs",
            "kaalas",
            "choghadiya",
            "horas",
            "muhurtas",
            "moon_events",
            "muhurta_yogas",
        ):
            column = getattr(counts, name)
            starts = [0] * (len(column) + 1)
            for i, count in enumerate(column):
                starts[i + 1] = starts[i] + count
            self._starts[name] = starts

    def __len__(self) -> int:
        """How many days the batch holds."""
        return self.decoded.day_count

    def at(self, index: int) -> AlmanacDay:
        """One day of the batch, by index."""
        if not 0 <= index < len(self):
            raise IndexError(f"day {index} is outside a batch of {len(self)}")
        return AlmanacDay(self, index)

    def __getitem__(self, index: int) -> AlmanacDay:
        """The same as `at`, so a batch indexes as well as iterates."""
        return self.at(index)

    def __iter__(self) -> Iterator[AlmanacDay]:
        """Every day, in the order the range runs."""
        return (AlmanacDay(self, index) for index in range(len(self)))

    def range(self, list_name: str, index: int) -> tuple[int, int]:
        """Where day `index`'s rows of a per-day list begin and end."""
        starts = self._starts[list_name]
        return starts[index], starts[index + 1]

    @property
    def place(self) -> Observer:
        """The place they were all founded at."""
        return Observer(
            latitude_deg=Latitude(self.decoded.latitude_deg),
            longitude_deg=Longitude(self.decoded.longitude_deg),
            altitude_m=Altitude(self.decoded.altitude_m),
        )

    @property
    def calendar(self) -> Calendar:
        """The civil calendar the days' dates are read in."""
        return Calendar(self.decoded.calendar)

    @property
    def model(self) -> str:
        """The solar model that reckoned the days, as it describes itself."""
        return self.decoded.model

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
        return self._context.intl.render(key, params).text

    def entity(self, key: str) -> intl.EntityForms:
        return self._context.intl.entity(key)


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
