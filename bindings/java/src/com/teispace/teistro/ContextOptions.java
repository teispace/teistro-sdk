package com.teispace.teistro;

/**
 * What a context is made with. Every part is optional: an empty options
 * object is the library's default profile over no ephemeris.
 *
 * @param profile the shipped profile's id ({@code parashari-classical},
 *     {@code nepali-default} …), or null for the library's default
 * @param settingsJson a JSON settings patch over the profile, or null
 * @param locale the locale every render resolves from, or null
 * @param layoutsJson chart layouts of the caller's own, as JSON, or null
 * @param dashasJson dasha systems of the caller's own, as JSON, or null
 * @param ephemeris the ephemeris chain, tried in order: the SDK's own by
 *     name or an adapter's {@link Plugin}; empty for none
 * @param testProvider whether the context runs on the library's test provider
 * @param provider an ephemeris written in Java, or null; the context owns
 *     its binding and releases it when closed
 */
public record ContextOptions(
        String profile,
        String settingsJson,
        String locale,
        String layoutsJson,
        String dashasJson,
        java.util.List<EphemerisChoice> ephemeris,
        boolean testProvider,
        EphemerisProvider provider) {

    /**
     * The options, checked: a provider and a named ephemeris each answer the
     * one question of what computes positions, so both together are refused.
     */
    public ContextOptions {
        ephemeris = ephemeris == null ? java.util.List.of() : java.util.List.copyOf(ephemeris);
        if (provider != null && !ephemeris.isEmpty()) {
            throw new IllegalArgumentException(
                    "a provider and an ephemeris each say what computes positions: give one of them");
        }
    }

    /**
     * A builder over the defaults.
     *
     * @return the builder
     */
    public static Builder builder() {
        return new Builder();
    }

    /** Builds {@link ContextOptions} one part at a time. */
    public static final class Builder {
        private String profile;
        private String settingsJson;
        private String locale;
        private String layoutsJson;
        private String dashasJson;
        private java.util.List<EphemerisChoice> ephemeris = java.util.List.of();
        private boolean testProvider;
        private EphemerisProvider provider;

        private Builder() {}

        /**
         * The shipped profile.
         *
         * @param profile its id
         * @return this builder
         */
        public Builder profile(String profile) {
            this.profile = profile;
            return this;
        }

        /**
         * A settings patch over the profile.
         *
         * @param settingsJson the patch as JSON
         * @return this builder
         */
        public Builder settingsJson(String settingsJson) {
            this.settingsJson = settingsJson;
            return this;
        }

        /**
         * The locale renders resolve from.
         *
         * @param locale its tag ({@code ne-Deva-NP})
         * @return this builder
         */
        public Builder locale(String locale) {
            this.locale = locale;
            return this;
        }

        /**
         * Chart layouts of the caller's own.
         *
         * @param layoutsJson the layouts as JSON
         * @return this builder
         */
        public Builder layoutsJson(String layoutsJson) {
            this.layoutsJson = layoutsJson;
            return this;
        }

        /**
         * Dasha systems of the caller's own.
         *
         * @param dashasJson the definitions as JSON
         * @return this builder
         */
        public Builder dashasJson(String dashasJson) {
            this.dashasJson = dashasJson;
            return this;
        }

        /**
         * Which of the SDK's own ephemerides to use.
         *
         * @param ephemeris the ephemeris
         * @return this builder
         */
        public Builder ephemeris(Ephemeris ephemeris) {
            this.ephemeris = java.util.List.of(EphemerisChoice.of(ephemeris));
            return this;
        }

        /**
         * An ephemeris chain, tried in order: the context opens on the first
         * entry that opens, and when none does, one refusal names each.
         *
         * @param chain the entries, at least one
         * @return this builder
         * @throws IllegalArgumentException for a chain of none, which is a
         *     mistake rather than a default
         */
        public Builder ephemeris(EphemerisChoice... chain) {
            if (chain.length == 0) {
                throw new IllegalArgumentException("an ephemeris chain that names nothing opens nothing");
            }
            this.ephemeris = java.util.List.of(chain);
            return this;
        }

        /**
         * Whether the context runs on the library's test provider.
         *
         * @param testProvider true for the test provider
         * @return this builder
         */
        public Builder testProvider(boolean testProvider) {
            this.testProvider = testProvider;
            return this;
        }

        /**
         * An ephemeris written in Java, which the SDK asks for every
         * position: the port's contract, as {@link EphemerisProvider} says.
         *
         * @param provider the provider
         * @return this builder
         */
        public Builder provider(EphemerisProvider provider) {
            this.provider = provider;
            return this;
        }

        /**
         * The options.
         *
         * @return the options
         */
        public ContextOptions build() {
            return new ContextOptions(profile, settingsJson, locale, layoutsJson, dashasJson, ephemeris, testProvider,
                    provider);
        }
    }
}
