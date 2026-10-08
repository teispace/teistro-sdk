package com.teispace.teistro;

import java.util.Map;
import java.util.Objects;

/**
 * An adapter's descriptor: the platform binary its package ships, and that
 * adapter's own configuration. What the configuration means is the
 * adapter's to say and its package's to type; the SDK hands it over as JSON
 * and reads none of it (ADR-0029).
 *
 * <p>Loading a library runs its initialisers, so {@code path} must be a file
 * the caller trusts.
 *
 * @param path the adapter's platform binary
 * @param config that adapter's own options, or null for none
 */
public record Plugin(String path, Map<String, ?> config) implements EphemerisChoice {
    /** The descriptor, with its configuration copied. */
    public Plugin {
        Objects.requireNonNull(path, "path");
        config = config == null ? Map.of() : Map.copyOf(config);
    }

    /**
     * An adapter with no options of its own.
     *
     * @param path the adapter's platform binary
     * @return the descriptor
     */
    public static Plugin of(String path) {
        return new Plugin(path, Map.of());
    }

    @Override
    public String named() {
        return path;
    }
}
