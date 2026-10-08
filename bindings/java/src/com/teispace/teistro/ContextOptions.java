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
 * @param ephemeris which of the SDK's own ephemerides to use
 * @param testProvider whether the context runs on the library's test provider
 */
public record ContextOptions(
        String profile,
        String settingsJson,
        String locale,
        String layoutsJson,
        String dashasJson,
        Ephemeris ephemeris,
        boolean testProvider) {

    /** The options, with the ephemeris never null. */
    public ContextOptions {
        if (ephemeris == null) {
            ephemeris = Ephemeris.NONE;
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
        private Ephemeris ephemeris = Ephemeris.NONE;
        private boolean testProvider;

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
            this.ephemeris = ephemeris;
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
         * The options.
         *
         * @return the options
         */
        public ContextOptions build() {
            return new ContextOptions(profile, settingsJson, locale, layoutsJson, dashasJson, ephemeris, testProvider);
        }
    }
}
