package com.teispace.teistro;

/**
 * A planetary war a body is in.
 *
 * @param opponent The other body.
 * @param isWinner Whether this body won it.
 * @param apartDeg How far apart they stand, degrees.
 */
public record War(
        Graha opponent,
        boolean isWinner,
        double apartDeg) {
}
