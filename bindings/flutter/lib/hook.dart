/// The build hook's body, for a package whose own hook bundles a library
/// the same way (an engine adapter's): `bundlePrebuilt` with its own
/// [Release].
library;

export 'src/bundle.dart'
    show
        Release,
        androidFloor,
        builtinDefine,
        bundlePrebuilt,
        iosFloor,
        libraryDefine,
        mobilePlatforms,
        namedLibrary,
        omitsBuiltin,
        platformFor,
        prebuiltDirDefine,
        processorOf,
        refuseFloor,
        refuseNamed,
        slice,
        withoutBuiltinSuffix;
