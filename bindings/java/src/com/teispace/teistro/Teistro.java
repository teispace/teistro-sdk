package com.teispace.teistro;

import java.lang.foreign.Arena;
import java.lang.foreign.Linker;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.lang.foreign.ValueLayout;
import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.TreeSet;

import com.teispace.teistro.ffi.Native;

/**
 * The library, opened: what needs no context (the versions, the build it
 * is), and the contexts made from it.
 *
 * <pre>{@code
 * try (Teistro teistro = Teistro.open();
 *      Context sky = teistro.context(ContextOptions.builder().ephemeris(Ephemeris.BUILTIN).build())) {
 *     System.out.println(sky.profile());
 * }
 * }</pre>
 *
 * <p>The library is mapped for the life of the process; closing this
 * releases nothing a context needs, and a context made from it stays
 * usable until it is closed itself.
 */
public final class Teistro implements AutoCloseable {
    /** The system property naming the library to load, which is then the only one tried. */
    public static final String LIBRARY_PROPERTY = "teistro.library";

    /** The environment variable naming the library, as every binding's loader reads it. */
    public static final String LIBRARY_VARIABLE = "TEISTRO_LIBRARY";

    private final Native lib;
    private final Path path;
    private final Map<String, Object> buildInfo;

    private Teistro(Native lib, Path path, Map<String, Object> buildInfo) {
        this.lib = lib;
        this.path = path;
        this.buildInfo = buildInfo;
    }

    /** The system property naming the directory a packaged library is written under. */
    public static final String CACHE_PROPERTY = NativeCache.CACHE_PROPERTY;

    /** The system property naming this host's platform, for a host the detection misreads. */
    public static final String PLATFORM_PROPERTY = Host.PLATFORM_PROPERTY;

    /**
     * Opens the library, the first of:
     *
     * <ol>
     *   <li>the one {@value #LIBRARY_PROPERTY} or {@value #LIBRARY_VARIABLE}
     *       names, which is then the only one tried: a library named outright
     *       that is missing is refused;
     *   <li>the jar's own, for this host's platform, written once under
     *       {@value #CACHE_PROPERTY} (else {@code java.io.tmpdir}) and hashed
     *       against the digest it was staged with every time it is opened;
     *   <li>the workspace's release and then debug build, from the working
     *       directory upward;
     *   <li>the platform's loader by name ({@code LD_LIBRARY_PATH},
     *       {@code PATH} and the like).
     * </ol>
     *
     * @return the library, checked against the build these declarations describe
     * @throws TeistroException with {@link Status#UNSUPPORTED} for a library
     *     of another ABI or version, a sanitizer build, an unoptimised build
     *     that was searched for rather than named, or none found, naming each
     *     place tried and why it was passed over
     */
    public static Teistro open() {
        String named = Optional.ofNullable(System.getProperty(LIBRARY_PROPERTY))
                .orElseGet(() -> System.getenv(LIBRARY_VARIABLE));
        if (named != null && !named.isEmpty()) {
            Path path = Path.of(named);
            if (!Files.isRegularFile(path)) {
                throw unsupported("the library named by " + LIBRARY_PROPERTY + " or " + LIBRARY_VARIABLE
                        + " is not a file: " + path);
            }
            return checked(lookup(path), path, true);
        }
        List<String> passed = new ArrayList<>();
        String platform = Host.platform();
        Teistro packaged = packaged(platform, passed);
        if (packaged != null) {
            return packaged;
        }
        for (Path candidate : workspaceCandidates()) {
            if (Files.isRegularFile(candidate)) {
                return checked(lookup(candidate), candidate, false);
            }
        }
        passed.add("no workspace build above " + Path.of("").toAbsolutePath());
        try {
            return checked(byName(Host.fileName()), Path.of(Host.fileName()), false);
        } catch (IllegalArgumentException notOnTheSearchPath) {
            passed.add("the platform's loader has no " + Host.fileName());
        }
        throw unsupported("no Teistro library was found for " + platform + ": " + String.join("; ", passed)
                + ". Set " + LIBRARY_PROPERTY + " or " + LIBRARY_VARIABLE + " to its path, "
                + PLATFORM_PROPERTY + " if " + platform + " is not this host, or build it with "
                + "`cargo build --release -p teistro-ffi`");
    }

    /** The jar's library for a platform, or null with why it was passed over. */
    private static Teistro packaged(String platform, List<String> passed) {
        String resource = "/native/" + platform + "/" + Host.fileName();
        String digest = Prebuilt.DIGESTS.get(platform);
        try (InputStream library = Teistro.class.getResourceAsStream(resource)) {
            if (library == null) {
                passed.add(digest == null
                        ? "this jar carries no library"
                        : "this jar carries the libraries of " + String.join(", ", new TreeSet<>(Prebuilt.DIGESTS.keySet()))
                                + ", not " + platform + "'s");
                return null;
            }
            if (digest == null) {
                throw unsupported("this jar carries " + resource + " and no digest for it: a staging defect");
            }
            Path path = NativeCache.extract(library, digest, NativeCache.root(), Native.GENERATED_SDK_VERSION,
                    Host.fileName());
            try {
                return checked(lookup(path), path, false);
            } catch (IllegalArgumentException unmappable) {
                throw unsupported("the jar's library was written to " + path + " and cannot be loaded from there ("
                        + unmappable.getMessage() + "); if that directory is on a noexec mount, set -D"
                        + CACHE_PROPERTY + " to one that is not");
            }
        } catch (IOException failed) {
            throw unsupported("the jar's library for " + platform + " could not be written: " + failed.getMessage());
        }
    }

    /** The workspace's builds, release first, from the working directory upward. */
    private static List<Path> workspaceCandidates() {
        List<Path> found = new ArrayList<>();
        for (Path dir = Path.of("").toAbsolutePath(); dir != null; dir = dir.getParent()) {
            if (Files.isRegularFile(dir.resolve("Cargo.toml")) && Files.isDirectory(dir.resolve("target"))) {
                found.add(dir.resolve("target/release").resolve(Host.fileName()));
                found.add(dir.resolve("target/debug").resolve(Host.fileName()));
                break;
            }
        }
        return found;
    }

    private static Teistro checked(SymbolLookup symbols, Path path, boolean named) {
        if (ValueLayout.ADDRESS.byteSize() != 8) {
            throw unsupported("this JVM's pointers are " + ValueLayout.ADDRESS.byteSize()
                    + " bytes; the binding describes the 64-bit boundary only");
        }
        Native lib = new Native(symbols);
        Map<String, Object> info = Json.object(Boundary.call(() -> Boundary.text((MemorySegment) lib.ts_build_info.invokeExact())));
        String refusal = refusal(info, named);
        if (refusal != null) {
            throw unsupported(refusal + " (" + path + ")");
        }
        return new Teistro(lib, path, info);
    }

    @SuppressWarnings("restricted")
    private static SymbolLookup byName(String fileName) {
        return SymbolLookup.libraryLookup(fileName, Arena.global()).or(Linker.nativeLinker().defaultLookup());
    }

    @SuppressWarnings("restricted")
    private static SymbolLookup lookup(Path path) {
        // Mapped for the process: contexts and the strings they lend outlive
        // any one `Teistro`, and the platform cannot unmap a library safely
        // while a callback into it may still run.
        return SymbolLookup.libraryLookup(path, Arena.global()).or(Linker.nativeLinker().defaultLookup());
    }

    /**
     * Why a build is refused, or null: the three rules every binding's loader
     * applies (`ffi-abi-and-api-description.md`, "The build handshake").
     */
    static String refusal(Map<String, Object> info, boolean named) {
        Object abi = info.get("abi");
        if (!(abi instanceof Long version) || version != Native.GENERATED_ABI_VERSION) {
            return "the library implements ABI " + abi + ", and these declarations describe ABI "
                    + Native.GENERATED_ABI_VERSION;
        }
        if (!Native.GENERATED_SDK_VERSION.equals(info.get("sdk"))) {
            return "the library is SDK " + info.get("sdk") + ", and these declarations are SDK "
                    + Native.GENERATED_SDK_VERSION;
        }
        if (info.get("sanitizer") instanceof String sanitizer && !sanitizer.isEmpty()) {
            return "the library is a " + sanitizer + " build, which is not for use";
        }
        if (!named && !Boolean.TRUE.equals(info.get("optimised"))) {
            return "the library found is an unoptimised build; name it outright with "
                    + LIBRARY_PROPERTY + " to use it";
        }
        return null;
    }

    private static TeistroException unsupported(String message) {
        return new TeistroException(Status.UNSUPPORTED, message, "", "", "", "");
    }

    Native lib() {
        return lib;
    }

    /**
     * The library this opened.
     *
     * @return its path
     */
    public Path path() {
        return path;
    }

    /**
     * What the library says it is: its versions, commit, target and build
     * profile, as {@code ts_build_info} writes them.
     *
     * @return the build's description
     */
    public Map<String, Object> buildInfo() {
        return buildInfo;
    }

    /**
     * The SDK's canonical frame: apparent geocentric ecliptic of date, tropical.
     *
     * @return the frame
     */
    public Frame canonicalFrame() {
        return Calls.frameCanonical(lib);
    }

    /**
     * A frame's fields as the bits a position request carries.
     *
     * @param frame the frame
     * @return its bits
     */
    public long packFrame(Frame frame) {
        return Calls.framePack(lib, frame);
    }

    /**
     * The frame a packed set of bits describes.
     *
     * @param bits the bits
     * @return the frame
     * @throws TeistroException for bits no frame packs to
     */
    public Frame unpackFrame(long bits) {
        return Calls.frameUnpack(lib, bits);
    }

    /**
     * The Julian day at the UTC midnight that begins a fixed day (fixed day 1
     * is Monday, 1 January 1 CE).
     *
     * @param fixed the fixed day number
     * @return its Julian day
     */
    public double julianDayOfFixed(long fixed) {
        return Calls.calendarJdOfFixed(lib, fixed);
    }

    /**
     * The fixed day number a Julian day falls on, and how far into it.
     *
     * @param jd the Julian day
     * @return the fixed day and the fraction of it
     */
    public CalendarFixedOfJdResult fixedOfJulianDay(double jd) {
        return Calls.calendarFixedOfJd(lib, jd);
    }

    /**
     * The ABI version the library implements.
     *
     * @return the ABI version
     */
    public int abiVersion() {
        return Boundary.call(() -> (int) lib.ts_abi_version.invokeExact());
    }

    /**
     * The catalogue schema version every result's provenance stamps.
     *
     * @return the catalogue version
     */
    public int catalogueVersion() {
        return Boundary.call(() -> (int) lib.ts_catalogue_version.invokeExact());
    }

    /**
     * The SDK version the library is.
     *
     * @return the SDK version
     */
    public String sdkVersion() {
        return Boundary.call(() -> Boundary.text((MemorySegment) lib.ts_sdk_version.invokeExact()));
    }

    /**
     * The profile a context uses when its options name none.
     *
     * @return the profile's id
     */
    public String defaultProfile() {
        return Boundary.call(() -> Boundary.text((MemorySegment) lib.ts_default_profile.invokeExact()));
    }

    /**
     * A context over this library.
     *
     * @param options what the context is made with
     * @return the context, which the caller closes
     * @throws TeistroException carrying the library's record of the refusal
     */
    public Context context(ContextOptions options) {
        return Context.open(this, options);
    }

    /** Releases nothing: the library stays mapped, and every context made from it stays usable. */
    @Override
    public void close() {
        // Nothing to release; see the class's documentation.
    }
}
