"""Fetching the shared library this package's version was built with.

A pure-Python wheel cannot carry a binary for every platform without
making every consumer download all of them, so this package carries none
and fetches the one it needs: `teistro-install` downloads the library
from the release its version was cut from, checks it against a digest
recorded when it was built, and writes it where `open` looks. A machine
with no network installs from a file it already has (`--from`), and a
machine that builds from source needs none of this — `cargo build
--release -p teistro-ffi` and the search path finds it.

Everything here is a plain function over bytes so that it can be tested
without a network: `verified` is the check, `write_library` is the write,
and `install` is the two with a download in front.
"""

from __future__ import annotations

import argparse
import ctypes
import gzip
import hashlib
import platform
import sys
from pathlib import Path
from typing import Optional
from urllib.request import urlopen

from ._prebuilt import PREBUILT_BASE, PREBUILT_DIGESTS, PREBUILT_VERSION

#: How long to wait for the release before giving up, in seconds.
TIMEOUT = 60


class InstallError(Exception):
    """What went wrong, in a sentence a person can act on."""

    def __init__(self, message: str, hint: str = "") -> None:
        super().__init__(message)
        self.message = message
        self.hint = hint

    def __str__(self) -> str:
        return self.message if not self.hint else f"{self.message}\n{self.hint}"


def library_file_name() -> str:
    """The file name this platform gives the SDK's shared library."""
    if sys.platform == "darwin":
        return "libteistro_ffi.dylib"
    if sys.platform == "win32":
        return "teistro_ffi.dll"
    return "libteistro_ffi.so"


def host_platform() -> str:
    """This host as the release names it: `<os>-<cpu>`.

    In the words Node's `process.platform` and `process.arch` use, so that
    one release page names one artefact for every binding.
    """
    system = {"darwin": "darwin", "win32": "win32"}.get(sys.platform, sys.platform)
    machine = platform.machine().lower()
    cpu = {
        "arm64": "arm64",
        "aarch64": "arm64",
        "x86_64": "x64",
        "amd64": "x64",
        "i386": "ia32",
        "i686": "ia32",
        "riscv64": "riscv64",
    }.get(machine, machine)
    return f"{system}-{cpu}"


def install_directory(project: Optional[Path] = None) -> Path:
    """Where an installed library is kept.

    Under the project's own tool directory, in a directory named for the
    version it belongs to. Naming the version means a project that changes
    the SDK's version fetches again rather than loading the library of the
    version before, which would be refused at load time but only after the
    download had been skipped.
    """
    root = project if project is not None else Path.cwd()
    return root / ".teistro" / PREBUILT_VERSION


def installed_library(project: Optional[Path] = None) -> Path:
    """The installed library's path, whether or not it is there yet."""
    return install_directory(project) / library_file_name()


def prebuilt_url(name: Optional[str] = None) -> Optional[str]:
    """The archive this platform's library is published as.

    `None` when the release carries none for it, which is what an
    unreleased checkout always says.
    """
    target = name if name is not None else host_platform()
    if target not in PREBUILT_DIGESTS:
        return None
    file_name = library_file_name()
    stem, _, extension = file_name.rpartition(".")
    if not stem:
        stem, extension = file_name, "so"
    return (
        f"{PREBUILT_BASE}/v{PREBUILT_VERSION}/"
        f"{stem}-{PREBUILT_VERSION}-{target}.{extension}.gz"
    )


def digest(data: bytes) -> str:
    """SHA-256 of some bytes, as the release's manifest writes it."""
    return hashlib.sha256(data).hexdigest()


def verified(
    data: bytes,
    *,
    name: Optional[str] = None,
    packed: bool = True,
    digests: Optional[dict[str, str]] = None,
) -> bytes:
    """The bytes of the library, unpacked if they arrived packed, checked
    against what the release recorded for this platform.

    `digests` is the table to check against, which is the release's own
    unless a test supplies one: the check is the part worth testing, and
    the repository's table is empty until a release fills it.

    Raises `InstallError` when the platform has no recorded digest or the
    bytes are not the ones that were built.
    """
    table = PREBUILT_DIGESTS if digests is None else digests
    target = name if name is not None else host_platform()
    expected = table.get(target)
    if expected is None:
        raise InstallError(
            (
                "this build of the package carries no prebuilt libraries "
                f"(version {PREBUILT_VERSION} is not a release)"
            )
            if not table
            else f"no prebuilt library for {target} in version {PREBUILT_VERSION}",
            hint=(
                "build it instead: `cargo build --release -p teistro-ffi`, then "
                f"point $TEISTRO_LIBRARY at target/release/{library_file_name()}"
            ),
        )
    if packed:
        try:
            library = gzip.decompress(data)
        except (OSError, EOFError) as error:
            raise InstallError(
                f"the download is not the archive it should be: {error}",
                hint=f"try again, or fetch it by hand from {prebuilt_url(target)}",
            ) from error
    else:
        library = data
    found = digest(library)
    if found != expected:
        raise InstallError(
            "the library that arrived is not the one this package was built "
            f"against.\n  expected {expected}\n  found    {found}",
            hint=(
                "do not load it. Fetch it again, and report it if it happens "
                "twice: https://github.com/teispace/teistro-sdk/issues"
            ),
        )
    return library


def write_library(library: bytes, project: Optional[Path] = None) -> Path:
    """Writes the library where `open` looks, and returns its path."""
    directory = install_directory(project)
    directory.mkdir(parents=True, exist_ok=True)
    path = directory / library_file_name()
    path.write_bytes(library)
    return path


def install(
    *, source: Optional[Path] = None, project: Optional[Path] = None
) -> tuple[Path, bool]:
    """Installs the library for this platform, and says whether it fetched.

    From `source` when a file is named, from the release otherwise. A
    library that is already installed and hashes correctly is left where it
    is; anything else is fetched and checked before it is written, so a
    failed install never leaves a half-written library behind.
    """
    path = installed_library(project)
    if path.is_file() and PREBUILT_DIGESTS.get(host_platform()) == digest(
        path.read_bytes()
    ):
        return path, False
    if source is not None:
        if not source.is_file():
            raise InstallError(f"no file at {source}")
        data, packed = source.read_bytes(), source.suffix == ".gz"
    else:
        url = prebuilt_url()
        if url is None:
            # `verified` says why, with the same words for both paths.
            verified(b"", packed=False)
            raise InstallError("unreachable: no prebuilt library and no refusal")
        with urlopen(url, timeout=TIMEOUT) as response:  # noqa: S310 — a release URL
            data = response.read()
        packed = True
    return write_library(verified(data, packed=packed), project), True


def main(argv: Optional[list[str]] = None) -> int:
    """`teistro-install`: fetches the library and says where it went."""
    parser = argparse.ArgumentParser(
        prog="teistro-install",
        description="Install the Teistro shared library this package needs.",
    )
    parser.add_argument(
        "--from",
        dest="source",
        type=Path,
        default=None,
        help="install from a file already on this machine, rather than the release",
    )
    parser.add_argument(
        "--project",
        type=Path,
        default=None,
        help="the project to install into; the working directory by default",
    )
    arguments = parser.parse_args(argv)
    try:
        path, fetched = install(source=arguments.source, project=arguments.project)
    except InstallError as error:
        print(error, file=sys.stderr)
        return 1
    size = path.stat().st_size
    print(
        f"{'installed' if fetched else 'already installed'} {path} "
        f"({size} bytes, {ctypes.sizeof(ctypes.c_void_p) * 8}-bit)"
    )
    return 0


if __name__ == "__main__":  # pragma: no cover — the console script is the entry
    raise SystemExit(main())
