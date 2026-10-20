package com.teispace.teistro.teimeris;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.stream.Collectors;

import com.teispace.teistro.Engine;
import com.teispace.teistro.Plugin;

/**
 * The Teimeris ephemeris, as a Teistro adapter: {@link #builder()}, the
 * descriptor an {@code ephemeris} chain takes, and {@link #engine(Engine)},
 * the typed façade over the engine's own operations.
 *
 * <pre>{@code
 * Teistro sdk = Teistro.open();
 * try (Context sky = sdk.context(ContextOptions.builder()
 *         .ephemeris(Teimeris.builder().dataDir("./ephe").build())
 *         .build())) {
 *     Teimeris.engine(sky.engine()).tmBodyName(0);  // "Sun"
 * }
 * }</pre>
 */
public final class Teimeris {
    private Teimeris() {}

    /**
     * The environment variable that names the adapter's platform binary,
     * which wins over every other place it is looked for: the same one
     * every binding's plugin test reads, so a contributor sets it once.
     */
    public static final String PATH_VARIABLE = "TEISTRO_TEIMERIS_ADAPTER";

    /** The system property that does what {@link #PATH_VARIABLE} does, and wins over it. */
    public static final String PATH_PROPERTY = "teistro.teimeris.adapter";

    /**
     * The typed façade over an engine.
     *
     * @param engine {@code sky.engine()}
     * @return the engine's own operations, typed
     */
    public static TeimerisEngine engine(Engine engine) {
        return new TeimerisEngine(engine);
    }

    /**
     * A descriptor to build.
     *
     * @return a builder with nothing set
     */
    public static Builder builder() {
        return new Builder();
    }

    /**
     * What this platform calls the adapter's shared library.
     *
     * @return the file name
     */
    public static String libraryFileName() {
        String os = System.getProperty("os.name", "").toLowerCase(Locale.ROOT);
        if (os.startsWith("mac")) {
            return "libteistro_ephemeris_teimeris.dylib";
        }
        if (os.startsWith("windows")) {
            return "teistro_ephemeris_teimeris.dll";
        }
        return "libteistro_ephemeris_teimeris.so";
    }

    /**
     * Every place {@link #binary()} looks, in order: the property, the
     * variable, then the adapter's own release and debug builds from the
     * working directory upward, the SDK's own order and for its reason.
     *
     * @return the candidates
     */
    public static List<Path> searchPath() {
        List<Path> found = new ArrayList<>();
        for (String named : new String[] {System.getProperty(PATH_PROPERTY), System.getenv(PATH_VARIABLE)}) {
            if (named != null && !named.isEmpty()) {
                found.add(Path.of(named));
            }
        }
        for (Path dir = Path.of("").toAbsolutePath(); dir != null; dir = dir.getParent()) {
            for (String build : new String[] {"release", "debug"}) {
                found.add(dir.resolve("adapters/ephemeris-teimeris/rust/target").resolve(build)
                        .resolve(libraryFileName()));
            }
        }
        return found;
    }

    /**
     * The adapter's platform binary.
     *
     * @return the first of {@link #searchPath()} that is a file
     * @throws IllegalStateException naming every place it looked, when
     *     none is: a path a consumer cannot see is one they cannot fix
     */
    public static Path binary() {
        List<Path> looked = searchPath();
        for (Path candidate : looked) {
            if (Files.isRegularFile(candidate)) {
                return candidate;
            }
        }
        throw new IllegalStateException("no Teimeris adapter for this host. Looked in:\n  "
                + looked.stream().map(Path::toString).collect(Collectors.joining("\n  "))
                + "\nBuild it with `cargo build --release` in `adapters/ephemeris-teimeris/rust`, or set "
                + PATH_VARIABLE + " to its path.");
    }

    /**
     * The descriptor an {@code ephemeris} chain takes (ADR-0029). {@link
     * #build()} fails when the adapter is not there, in the line that names
     * the engine rather than when a chart is cast: that is why a
     * descriptor is a value and not a string the SDK looks up.
     */
    public static final class Builder {
        private String dataDir;
        private final List<String> files = new ArrayList<>();
        private Boolean embedded;
        private Boolean verify;
        private String profile;
        private Path path;

        private Builder() {}

        /**
         * A directory of the engine's data files, read after the data built
         * into the engine. With nothing named, the provider reads the
         * built-in data, 1800 to 2399, and the directory
         * {@code TEIMERIS_DATA_DIR} names when that is set.
         *
         * @param dataDir the directory
         * @return this builder
         */
        public Builder dataDir(String dataDir) {
            this.dataDir = dataDir;
            return this;
        }

        /**
         * Adds a range an app downloaded, one block file at a time: a
         * 600-year block is its {@code sepl_*.se1} and {@code semo_*.se1}
         * files (and {@code seas_*.se1} for Chiron, Ceres, Pallas, Juno and
         * Vesta), so {@code sepl_12.se1} and {@code semo_12.se1} add 1200 to
         * 1799. Each is checked against the published size and SHA-256 and
         * refused by name when it differs, which is what a download cut
         * short looks like; a span with a hole in it is refused with the
         * files that fill it.
         *
         * @param file the block file's path
         * @return this builder
         */
        public Builder file(String file) {
            this.files.add(file);
            return this;
        }

        /**
         * Whether the data built into the engine is read; it is unless this
         * says no.
         *
         * @param embedded false to leave it out
         * @return this builder
         */
        public Builder embedded(boolean embedded) {
            this.embedded = embedded;
            return this;
        }

        /**
         * Whether each block file named is checked against the published
         * manifest; it is unless this says no.
         *
         * @param verify false to skip the check
         * @return this builder
         */
        public Builder verify(boolean verify) {
            this.verify = verify;
            return this;
        }

        /**
         * The engine's own accuracy profile.
         *
         * @param profile its name
         * @return this builder
         */
        public Builder profile(String profile) {
            this.profile = profile;
            return this;
        }

        /**
         * The platform binary, for a caller who would rather say than let
         * this package look.
         *
         * @param path the adapter's shared library
         * @return this builder
         */
        public Builder path(Path path) {
            this.path = path;
            return this;
        }

        /**
         * The descriptor.
         *
         * @return the plugin entry for the chain
         * @throws IllegalStateException when no path was given and none is found
         */
        public Plugin build() {
            // The keys are the adapter's: its own `Config` spells them in
            // camelCase and refuses an unknown one, so a misspelling is an
            // error rather than a default.
            Map<String, Object> config = new LinkedHashMap<>();
            if (dataDir != null) {
                config.put("dataDir", dataDir);
            }
            if (!files.isEmpty()) {
                config.put("files", List.copyOf(files));
            }
            if (embedded != null) {
                config.put("embedded", embedded);
            }
            if (verify != null) {
                config.put("verify", verify);
            }
            if (profile != null) {
                config.put("profile", profile);
            }
            return new Plugin((path != null ? path : binary()).toString(), config);
        }
    }
}
