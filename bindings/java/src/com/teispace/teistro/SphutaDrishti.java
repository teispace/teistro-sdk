package com.teispace.teistro;

/**
 * One graha's sphuta drishti on another body, by degree (crux C45).
 *
 * @param fromGraha The graha looking: one of the seven.
 * @param to The body looked at, a node included.
 * @param virupas How strongly, in virupas: 0 to 60, Mars's, Jupiter's and Saturn's special aspects
 *     adding 15, 30 and 45 over their spans.
 */
public record SphutaDrishti(Graha fromGraha, Graha to, double virupas) {
}
