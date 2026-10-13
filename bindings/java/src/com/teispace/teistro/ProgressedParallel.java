package com.teispace.teistro;

/**
 * A stretch of life a progressed planet stands within the orb of a radical
 * point's distance from the equator, on either side of it (Leo, <i>The
 * Progressed Horoscope</i>, p. 34).
 *
 * @param graha the progressed planet
 * @param to the radical point
 * @param enters the instant of life it came within the orb, a UTC Julian day;
 *     null when it already stood within it at the window's start
 * @param leaves the instant of life it left the orb; null when it still stood
 *     within it at the window's end
 * @param closest the first instant of life the two distances stood closest:
 *     the exact parallel, when the span reaches it
 * @param closestDeg how far apart the two distances stood then, degrees
 * @param contrary whether the two stood on opposite sides of the equator then
 */
public record ProgressedParallel(
        Graha graha, NatalPoint to, Double enters, Double leaves, double closest, double closestDeg,
        boolean contrary) {}
