/// The release's prebuilt libraries: where each platform's is published,
/// what it must hash to, and the check that holds a download to it.
///
/// This is what the `teistro_flutter` package's build hook reads, so the
/// digests an app's build verifies are this package's own table and the
/// check is this package's own [verified], with its refusals: there is no
/// second table to drift from this one.
library;

export 'src/install.dart'
    show
        InstallException,
        archiveUri,
        bundledLibraryName,
        download,
        libraryFileNameOf,
        prebuiltUri,
        verified;
export 'src/prebuilt.dart' show prebuiltBase, prebuiltDigests, prebuiltVersion;
export 'src/sha256.dart' show sha256Hex;
