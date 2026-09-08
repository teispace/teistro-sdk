"""Where a prebuilt library comes from, and what it must hash to.

The table is written at release time from the manifest the release matrix
produced (`cargo xtask package --stage`), so a download is checked
against a digest taken from the build rather than from the download. In
the repository the table is empty and the version is the unreleased one:
there is nothing to fetch, and `teistro-install` says so and points at
`cargo build --release -p teistro-ffi`.
"""

from __future__ import annotations

from typing import Final

#: The release these digests were taken from, which is the version of this
#: package: an installer never mixes a library with another build's types.
PREBUILT_VERSION: Final = "0.0.0"

#: The release the archives are attached to.
PREBUILT_BASE: Final = "https://github.com/teispace/teistro-sdk/releases/download"

#: SHA-256 of the shared library for each platform, by `<os>-<cpu>` as the
#: Node, Dart and Python packages all name it.
PREBUILT_DIGESTS: Final[dict[str, str]] = {}
