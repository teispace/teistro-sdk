package com.teispace.teistro;

/**
 * A retrograde planet rejecting a ruler through its star, or its sub (C153).
 *
 * @param retrograde The retrograde planet.
 * @param byStar Whether it rejects through its star rather than its sub.
 */
public record KpRejection(
        Graha retrograde,
        boolean byStar) {
}
