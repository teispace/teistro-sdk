package com.teispace.teistro;

import java.util.Map;

/**
 * The SHA-256 of each platform's library a released jar carries, by the
 * release's platform name ({@code linux-x64}, {@code darwin-arm64}, ...).
 *
 * <p>Empty in the repository: {@code cargo xtask package stage} writes the
 * release's table into the sources it compiles, so the classes of the
 * default jar and of every classifier jar are the same bytes and the
 * table cannot drift from them.
 */
final class Prebuilt {
    private Prebuilt() {}

    /** The digests, by platform. */
    static final Map<String, String> DIGESTS = Map.of();
}
