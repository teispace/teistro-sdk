package com.teispace.teistro;

/**
 * Two points meeting in a harmonic chart, within the orb of each other
 * there: the planets in the catalogue's order first, then the ascendant,
 * then the midheaven.
 *
 * @param first the first point
 * @param second the second point
 * @param apartDeg how far apart they stand in the harmonic chart, degrees
 * @param multiple which multiple k of the harmonic's aspect, k × 360° / n,
 *     they stand at in the chart itself
 * @param orbDeg the orb the request allowed, degrees
 */
public record HarmonicRow(HarmonicPoint first, HarmonicPoint second, double apartDeg, int multiple, double orbDeg) {}
