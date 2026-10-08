package com.teispace.teistro;

/**
 * A natal point a transit aspects: a natal graha, or the lagna. It goes into a hit request's
 * {@code points} as it comes back in an aspect's {@code to}.
 *
 * @param point {@code GRAHA} or {@code LAGNA}.
 * @param graha Which graha, for a {@code GRAHA} point; null for the lagna. May be null.
 */
public record NatalPoint(
        String point,
        Graha graha) {
}
