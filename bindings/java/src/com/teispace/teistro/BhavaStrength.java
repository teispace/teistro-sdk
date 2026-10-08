package com.teispace.teistro;

/**
 * One bhava's Bhava bala, in virupas.
 *
 * @param bhava Which bhava, 1 to 12.
 * @param lord The lord of the sign its madhya falls in.
 * @param adhipati The lord's Shadbala.
 * @param dig From its direction, 0 to 60.
 * @param drishti From the drishtis it receives, which may be negative.
 * @param special From its occupants and its sign's rising, under BPHS's special rules.
 * @param virupas The four together.
 */
public record BhavaStrength(
        int bhava,
        Graha lord,
        double adhipati,
        double dig,
        double drishti,
        double special,
        double virupas) {
}
