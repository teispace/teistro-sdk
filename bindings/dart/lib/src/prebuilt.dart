/// Where a prebuilt library comes from, and what it must hash to.
///
/// The table is written at release time from the manifest the release
/// matrix produced (`cargo xtask package --stage`), so a download is
/// checked against a digest taken from the build rather than from the
/// download. In the repository the table is empty and the version is the
/// unreleased one: there is nothing to fetch, and `dart run teistro:install`
/// says so and points at `cargo build --release -p teistro-ffi`.
library;

/// The release these digests were taken from, which is the version of this
/// package: an installer never mixes a library with another build's types.
const String prebuiltVersion = '0.0.0';

/// The release the archives are attached to.
const String prebuiltBase =
    'https://github.com/teispace/teistro-sdk/releases/download';

/// SHA-256 of the shared library for each platform, by `<os>-<cpu>` as
/// Dart and Node both name it.
const Map<String, String> prebuiltDigests = <String, String>{};
