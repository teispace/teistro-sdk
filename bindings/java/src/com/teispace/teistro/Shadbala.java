package com.teispace.teistro;

import java.util.List;

/**
 * A chart's Shadbala, read under the context's {@code strength.*} settings
 * ({@code 03-design/shadbala-measured.md}).
 *
 * @param grahas Each graha's, Sun to Saturn.
 */
public record Shadbala(
        List<GrahaShadbala> grahas) {
    /** The value, its lists copied and unmodifiable. */
    public Shadbala {
        grahas = List.copyOf(grahas);
    }
}
