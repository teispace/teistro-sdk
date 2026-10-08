package com.teispace.teistro;

import java.lang.foreign.Arena;
import java.lang.foreign.Linker;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.lang.foreign.ValueLayout;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Optional;

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

    /**
     * Opens the library: the one {@value #LIBRARY_PROPERTY} or
     * {@value #LIBRARY_VARIABLE} names, else the workspace's release and then
     * debug build, else the platform's loader by name. A library named
     * outright that is missing is refused, and nothing else is tried.
     *
     * @return the library, checked against the build these declarations describe
     * @throws TeistroException with {@link Status#UNSUPPORTED} for a library
     *     of another ABI or version, a sanitizer build, or an unoptimised
     *     build that was searched for rather than named
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
            return checked(path, true);
        }
        for (Path candidate : workspaceCandidates()) {
            if (Files.isRegularFile(candidate)) {
                return checked(candidate, false);
            }
        }
        throw unsupported("no Teistro library was found: set " + LIBRARY_PROPERTY + " or "
                + LIBRARY_VARIABLE + " to its path, or build it with `cargo build --release -p teistro-ffi`");
    }

    /** The file name the platform gives the shared library. */
    static String fileName() {
        String os = System.getProperty("os.name", "").toLowerCase(Locale.ROOT);
        if (os.startsWith("windows")) {
            return "teistro_ffi.dll";
        }
        return os.startsWith("mac") ? "libteistro_ffi.dylib" : "libteistro_ffi.so";
    }

    /** The workspace's builds, release first, from the working directory upward. */
    private static List<Path> workspaceCandidates() {
        List<Path> found = new ArrayList<>();
        for (Path dir = Path.of("").toAbsolutePath(); dir != null; dir = dir.getParent()) {
            if (Files.isRegularFile(dir.resolve("Cargo.toml")) && Files.isDirectory(dir.resolve("target"))) {
                found.add(dir.resolve("target/release").resolve(fileName()));
                found.add(dir.resolve("target/debug").resolve(fileName()));
                break;
            }
        }
        return found;
    }

    private static Teistro checked(Path path, boolean named) {
        if (ValueLayout.ADDRESS.byteSize() != 8) {
            throw unsupported("this JVM's pointers are " + ValueLayout.ADDRESS.byteSize()
                    + " bytes; the binding describes the 64-bit boundary only");
        }
        SymbolLookup symbols = lookup(path);
        Native lib = new Native(symbols);
        Map<String, Object> info = Json.object(Boundary.call(() -> Boundary.text((MemorySegment) lib.ts_build_info.invokeExact())));
        String refusal = refusal(info, named);
        if (refusal != null) {
            throw unsupported(refusal + " (" + path + ")");
        }
        return new Teistro(lib, path, info);
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
