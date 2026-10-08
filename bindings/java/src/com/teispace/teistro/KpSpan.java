package com.teispace.teistro;

/**
 * An arc of the zodiac, half-open, in nanoarcseconds (divide by {@code 3.6e12} for degrees).
 *
 * @param start Where it starts.
 * @param end Where it ends.
 */
public record KpSpan(
        long start,
        long end) {
}
