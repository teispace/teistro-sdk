"""The Teimeris ephemeris, as a Teistro adapter.

Two things, and a consumer needs both: `teimeris()`, the **descriptor**
an `ephemeris` chain takes, and `engine()`, the typed façade over the
engine's own operations.

```python
import teistro
from teistro_ephemeris_teimeris import engine, teimeris

sdk = teistro.Teistro.open()
with sdk.context(ephemeris=[teimeris(data_dir="./ephe")]) as ctx:
    engine(ctx.engine).tm_body_name(body=0)  # 'Sun'
```

**This package is AGPL-3.0-only**, because the library it ships links
Teimeris. The SDK is Apache-2.0 and never links it: it loads this
adapter at run time, so the licence stays on this side of the boundary
(ADR-0029).
"""

from __future__ import annotations

import os
import platform
import sys
from pathlib import Path
from typing import Optional

from teistro import Plugin

from .engine import TeimerisEngine, teimeris as engine

__all__ = ["PATH_VARIABLE", "TeimerisEngine", "binary", "engine", "search_path", "teimeris"]

#: The environment variable that names the adapter's platform binary,
#: which wins over every other place it is looked for.
#:
#: The same variable `crates/ffi/tests/abi.rs` and every binding's plugin
#: test read, so a contributor sets it once.
PATH_VARIABLE = "TEISTRO_TEIMERIS_ADAPTER"

_HERE = Path(__file__).resolve().parent


def library_file_name() -> str:
    """What this platform calls the adapter's shared library."""
    if sys.platform == "darwin":
        return "libteistro_ephemeris_teimeris.dylib"
    if sys.platform == "win32":
        return "teistro_ephemeris_teimeris.dll"
    return "libteistro_ephemeris_teimeris.so"


def host_platform() -> str:
    """This host as the release names it, in the words every binding uses."""
    system = {"darwin": "darwin", "win32": "win32"}.get(sys.platform, sys.platform)
    machine = platform.machine().lower()
    cpu = {"arm64": "arm64", "aarch64": "arm64", "x86_64": "x64", "amd64": "x64"}.get(
        machine, machine
    )
    return f"{system}-{cpu}"


def search_path() -> list[Path]:
    """Every place `binary` looks, in order.

    The SDK's own order, for the same reason: a consumer only ever has
    the one this package ships and a contributor only ever has the build,
    and the order matters for whoever has both.
    """
    named = os.environ.get(PATH_VARIABLE)
    name = library_file_name()
    found = [
        *( [Path(named)] if named else [] ),
        # Beside this package, which is where a published wheel puts it.
        _HERE / name,
        *( _HERE.parents[2] / "rust" / "target" / build / name
           for build in ("release", "debug") ),
    ]
    return found


def binary(named: Optional[str] = None) -> str:
    """The adapter's platform binary.

    Raises `FileNotFoundError` naming every place it looked when there is
    none, because a path a consumer cannot see is a path they cannot fix.
    """
    if named is not None:
        return named
    looked = search_path()
    for candidate in looked:
        if candidate.is_file():
            return str(candidate)
    joined = "\n  ".join(str(path) for path in looked)
    raise FileNotFoundError(
        f"no Teimeris adapter for {host_platform()}. Looked in:\n  {joined}\n"
        "Build it with `cargo build --release` in "
        f"`adapters/ephemeris-teimeris/rust`, or set {PATH_VARIABLE} to its path."
    )


def teimeris(
    *,
    data_dir: Optional[str] = None,
    profile: Optional[str] = None,
    path: Optional[str] = None,
) -> Plugin:
    """The descriptor an `ephemeris` chain takes (ADR-0029).

    **It fails here** -- at the call, in the line that names the engine --
    when the adapter is not built or installed, rather than when a chart
    is cast. That is the whole reason a descriptor is a value rather than
    a string the SDK looks up.

    `data_dir` is where the engine's data files are; it looks beside its
    own build when omitted. `profile` is the engine's own accuracy
    profile. `path` names the platform binary, for a caller who would
    rather say than let this package look.
    """
    config: dict[str, object] = {}
    if data_dir is not None:
        config["data_dir"] = data_dir
    if profile is not None:
        config["profile"] = profile
    return Plugin(plugin=binary(path), config=config)
