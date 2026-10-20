package com.teispace.teistro;

/**
 * One entry of an ephemeris chain: one of the SDK's own by name, an
 * adapter's {@link Plugin}, or an ephemeris written in Java, which is how one
 * stands behind {@link Ephemeris#SURYA_SIDDHANTA} for the day's modern
 * sunrise. A context tries its chain in order and opens on the first entry
 * that opens (ADR-0029).
 *
 * <pre>{@code
 * ContextOptions.builder()
 *     .ephemeris(Plugin.of("/opt/teimeris/libteistro_ephemeris_teimeris.so"),
 *                EphemerisChoice.of(Ephemeris.BUILTIN))
 *     .build();
 * }</pre>
 */
public sealed interface EphemerisChoice permits EphemerisChoice.Own, EphemerisChoice.Provided, Plugin {
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
     * An ephemeris written in Java, as a chain entry.
     *
     * @param provider the provider
     * @return the entry
     */
    static EphemerisChoice of(EphemerisProvider provider) {
        return new Provided(provider);
    }

    /**
     * What a refusal names this entry by.
     *
     * @return the ephemeris's key, the plugin's path or the provider's name
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

    /**
     * An ephemeris written in Java, which the context binds when it opens on
     * this entry and releases when it closes.
     *
     * @param provider the provider
     */
    record Provided(EphemerisProvider provider) implements EphemerisChoice {
        /** The entry, never of a null provider. */
        public Provided {
            java.util.Objects.requireNonNull(provider, "provider");
        }

        @Override
        public String named() {
            return provider.name();
        }
    }
}
