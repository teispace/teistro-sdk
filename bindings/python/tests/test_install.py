"""The installer, without a network.

Everything that decides whether a download may be loaded is a plain
function over bytes, so the part worth testing is testable: the digest
check, where the library is written, and the refusals. The repository's
own digest table is empty until a release fills it, so a table is passed
in rather than read.
"""

from __future__ import annotations

import contextlib
import gzip
import io
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from teistro import _install
from teistro._prebuilt import PREBUILT_BASE, PREBUILT_VERSION
from teistro._install import (
    InstallError,
    digest,
    host_platform,
    install_directory,
    installed_library,
    library_file_name,
    prebuilt_url,
    verified,
    write_library,
)

LIBRARY = b"not really a library, but bytes are bytes"
PLATFORM = "linux-x64"
TABLE = {PLATFORM: digest(LIBRARY)}


class TheHost(unittest.TestCase):
    def test_it_names_itself_the_way_the_release_does(self) -> None:
        name = host_platform()
        self.assertRegex(name, r"^[a-z0-9]+-[a-z0-9]+(-musl)?$")
        system, cpu, *libc = name.split("-")
        self.assertIn(system, ("linux", "darwin", "win32"))
        self.assertIn(cpu, ("x64", "arm64", "ia32", "riscv64"))
        self.assertTrue(not libc or system == "linux", "only Linux names its C library")

    def test_a_musl_linux_is_named_for_its_c_library(self) -> None:
        """glibc answers `confstr` and `libc_ver`; musl answers neither.
        Windows' `os` has no `confstr`, so the patch creates it."""

        def unnamed(_name: str) -> str:
            raise ValueError("unrecognized configuration name")

        with contextlib.ExitStack() as stack:
            stack.enter_context(mock.patch("teistro._install.sys.platform", "linux"))
            stack.enter_context(mock.patch("teistro._install.platform.machine", lambda: "x86_64"))
            stack.enter_context(mock.patch("teistro._install.os.confstr", unnamed, create=True))
            stack.enter_context(mock.patch("teistro._install.platform.libc_ver", lambda: ("", "")))
            self.assertEqual(host_platform(), "linux-x64-musl")
        with contextlib.ExitStack() as stack:
            stack.enter_context(mock.patch("teistro._install.sys.platform", "linux"))
            stack.enter_context(mock.patch("teistro._install.platform.machine", lambda: "aarch64"))
            stack.enter_context(mock.patch("teistro._install.os.confstr", lambda _name: "glibc 2.39", create=True))
            self.assertEqual(host_platform(), "linux-arm64")

    def test_the_library_has_the_platforms_own_file_name(self) -> None:
        name = library_file_name()
        self.assertTrue(
            name.endswith((".so", ".dylib", ".dll")), name
        )
        self.assertIn("teistro_ffi", name)

    def test_the_install_directory_is_named_for_the_version(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            project = Path(directory)
            self.assertEqual(
                install_directory(project),
                project / ".teistro" / PREBUILT_VERSION,
            )
            self.assertEqual(
                installed_library(project),
                install_directory(project) / library_file_name(),
            )


class TheCheck(unittest.TestCase):
    def test_bytes_that_hash_as_recorded_are_accepted(self) -> None:
        packed = gzip.compress(LIBRARY)
        self.assertEqual(verified(packed, name=PLATFORM, digests=TABLE), LIBRARY)
        self.assertEqual(
            verified(LIBRARY, name=PLATFORM, packed=False, digests=TABLE), LIBRARY
        )

    def test_bytes_that_do_not_are_refused_and_say_both_digests(self) -> None:
        with self.assertRaises(InstallError) as caught:
            verified(b"something else", name=PLATFORM, packed=False, digests=TABLE)
        message = str(caught.exception)
        self.assertIn("expected", message)
        self.assertIn("found", message)
        self.assertIn("do not load it", message.lower())

    def test_an_archive_that_is_not_one_is_refused(self) -> None:
        with self.assertRaises(InstallError) as caught:
            verified(b"not gzip", name=PLATFORM, digests=TABLE)
        self.assertIn("archive", str(caught.exception))

    def test_a_platform_the_release_has_none_for(self) -> None:
        with self.assertRaises(InstallError) as caught:
            verified(LIBRARY, name="plan9-mips", packed=False, digests=TABLE)
        self.assertIn("no prebuilt library", str(caught.exception))
        self.assertIn("cargo build", caught.exception.hint)

    def test_a_checkout_carries_no_prebuilt_libraries_and_says_so(self) -> None:
        # The repository's own table is empty until a release fills it,
        # which is what a contributor building from source will meet.
        with self.assertRaises(InstallError) as caught:
            verified(LIBRARY, packed=False, digests={})
        self.assertIn("not a release", str(caught.exception))
        self.assertIsNone(prebuilt_url())


class TheWrite(unittest.TestCase):
    def test_it_writes_where_the_loader_looks(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            project = Path(directory)
            path = write_library(LIBRARY, project)
            self.assertEqual(path, installed_library(project))
            self.assertEqual(path.read_bytes(), LIBRARY)

    def test_a_missing_source_file_is_named(self) -> None:
        with self.assertRaises(InstallError) as caught:
            _install.install(source=Path("/no/such/archive.gz"))
        self.assertIn("no file at", str(caught.exception))


class EachPlatform(unittest.TestCase):
    """What the installer names on each platform, read on any host: the
    branches for the two this host is not are otherwise left to another
    machine's run."""

    def test_each_platform_names_its_own_library_file(self) -> None:
        for system, name in (
            ("darwin", "libteistro_ffi.dylib"),
            ("win32", "teistro_ffi.dll"),
            ("linux", "libteistro_ffi.so"),
        ):
            with mock.patch("teistro._install.sys.platform", system):
                self.assertEqual(library_file_name(), name, system)

    def test_windows_is_never_musl_and_never_asks_confstr(self) -> None:
        def asked(_name: str) -> str:
            raise AssertionError("Windows has no confstr to ask")

        with contextlib.ExitStack() as stack:
            stack.enter_context(mock.patch("teistro._install.sys.platform", "win32"))
            stack.enter_context(mock.patch("teistro._install.platform.machine", lambda: "AMD64"))
            stack.enter_context(mock.patch("teistro._install.os.confstr", asked, create=True))
            self.assertEqual(host_platform(), "win32-x64")

    def test_a_release_url_names_the_version_the_target_and_the_extension(self) -> None:
        with contextlib.ExitStack() as stack:
            stack.enter_context(mock.patch.dict("teistro._install.PREBUILT_DIGESTS", {"win32-x64": "0" * 64}))
            stack.enter_context(mock.patch("teistro._install.sys.platform", "win32"))
            self.assertEqual(
                prebuilt_url("win32-x64"),
                f"{PREBUILT_BASE}/v{PREBUILT_VERSION}/teistro_ffi-{PREBUILT_VERSION}-win32-x64.dll.gz",
            )
            self.assertIsNone(prebuilt_url("plan9-mips"), "a target the release has none for")


class TheInstall(unittest.TestCase):
    """`install` end to end over a release table and a download that are
    patched in: it fetches, checks and writes, and a second run finds the
    library it wrote and does not fetch again."""

    def test_a_release_is_fetched_once_and_then_found(self) -> None:
        fetched: list[str] = []

        class Response(io.BytesIO):
            def __enter__(self) -> "Response":
                return self

            def __exit__(self, *_: object) -> None:
                self.close()

        def download(url: str, timeout: int) -> Response:
            fetched.append(url)
            self.assertEqual(timeout, _install.TIMEOUT)
            return Response(gzip.compress(LIBRARY))

        with tempfile.TemporaryDirectory() as directory, contextlib.ExitStack() as stack:
            project = Path(directory)
            stack.enter_context(mock.patch.dict("teistro._install.PREBUILT_DIGESTS", {host_platform(): digest(LIBRARY)}))
            stack.enter_context(mock.patch("teistro._install.urlopen", download))
            path, did = _install.install(project=project)
            self.assertEqual((path, did, path.read_bytes()), (installed_library(project), True, LIBRARY))
            self.assertEqual(fetched, [prebuilt_url()])
            again, did = _install.install(project=project)
            self.assertEqual((again, did), (path, False))
            self.assertEqual(len(fetched), 1, "an installed library that hashes is left alone")

    def test_a_file_already_here_installs_packed_or_not_and_a_wrong_one_is_refused(self) -> None:
        with tempfile.TemporaryDirectory() as directory, contextlib.ExitStack() as stack:
            root = Path(directory)
            stack.enter_context(mock.patch.dict("teistro._install.PREBUILT_DIGESTS", {host_platform(): digest(LIBRARY)}))
            packed = root / "library.gz"
            packed.write_bytes(gzip.compress(LIBRARY))
            plain = root / "library.bin"
            plain.write_bytes(LIBRARY)
            for source, project in ((packed, root / "a"), (plain, root / "b")):
                path, did = _install.install(source=source, project=project)
                self.assertEqual((did, path.read_bytes()), (True, LIBRARY), source.name)
            wrong = root / "wrong.bin"
            wrong.write_bytes(b"another library")
            with self.assertRaises(InstallError) as caught:
                _install.install(source=wrong, project=root / "c")
            self.assertIn("expected", str(caught.exception))
            self.assertFalse(installed_library(root / "c").exists(), "a refused library is never written")


class TheCommand(unittest.TestCase):
    def test_it_reports_a_refusal_rather_than_raising(self) -> None:
        # An unreleased checkout has nothing to fetch, so the command says
        # what to do instead and exits non-zero.
        said = io.StringIO()
        with contextlib.redirect_stderr(said):
            self.assertEqual(_install.main([]), 1)
        self.assertIn("not a release", said.getvalue())
        self.assertIn("cargo build", said.getvalue())


if __name__ == "__main__":
    unittest.main()
