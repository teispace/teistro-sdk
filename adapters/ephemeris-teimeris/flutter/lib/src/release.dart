/// The release this package's hook bundles the adapter from.
library;

import 'package:teistro/prebuilt.dart' show prebuiltBase, prebuiltVersion;
import 'package:teistro_flutter/hook.dart' show Release;

/// The adapter's library name, without its platform's prefix and suffix:
/// `libteistro_ephemeris_teimeris.so` on Android, wrapped by Flutter as
/// `teistro_ephemeris_teimeris.framework` on iOS.
const String adapterStem = 'teistro_ephemeris_teimeris';

/// SHA-256 of the adapter's library for each mobile platform, written at
/// release time beside the SDK's own table. Empty in the repository, where
/// nothing is released: an app names the libraries it built with the
/// `library` user define until there is a release to fetch.
const Map<String, String> adapterDigests = <String, String>{};

/// The adapter's release: the SDK's version and release page, its own
/// library and digests, so an app never mixes an adapter with another
/// release's SDK.
const Release adapterRelease = Release(
  stem: adapterStem,
  version: prebuiltVersion,
  base: prebuiltBase,
  digests: adapterDigests,
);
