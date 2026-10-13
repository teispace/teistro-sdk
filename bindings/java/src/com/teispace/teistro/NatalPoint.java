package com.teispace.teistro;

/**
 * A natal point a transit aspects: a natal graha, the lagna or the midheaven. It goes into a hit
 * request's {@code points} as it comes back in an aspect's {@code to}.
 *
 * @param point {@code GRAHA}, {@code LAGNA} or {@code MIDHEAVEN}.
 * @param graha Which graha, for a {@code GRAHA} point; null for the angles. May be null.
 */
public record NatalPoint(
        String point,
        Graha graha) {
}
