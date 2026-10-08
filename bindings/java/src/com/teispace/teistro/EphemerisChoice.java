package com.teispace.teistro;

/**
 * One entry of an ephemeris chain: one of the SDK's own by name, or an
 * adapter's {@link Plugin}. A context tries its chain in order and opens on
 * the first entry that opens (ADR-0029).
 *
 * <pre>{@code
 * ContextOptions.builder()
 *     .ephemeris(Plugin.of("/opt/teimeris/libteistro_ephemeris_teimeris.so"),
 *                EphemerisChoice.of(Ephemeris.BUILTIN))
 *     .build();
 * }</pre>
 */
public sealed interface EphemerisChoice permits EphemerisChoice.Own, Plugin {
    /**
     * One of the SDK's own ephemerides, as a chain entry.
     *
     * @param ephemeris the ephemeris
     * @return the entry
     */
    static EphemerisChoice of(Ephemeris ephemeris) {
        return new Own(ephemeris);
    }

    /**
     * What a refusal names this entry by.
     *
     * @return the ephemeris's key or the plugin's path
     */
    String named();

    /**
     * One of the SDK's own ephemerides.
     *
     * @param ephemeris which
     */
    record Own(Ephemeris ephemeris) implements EphemerisChoice {
        /** The entry, never of a null ephemeris. */
        public Own {
            java.util.Objects.requireNonNull(ephemeris, "ephemeris");
        }

        @Override
        public String named() {
            return ephemeris.key();
        }
    }
}
