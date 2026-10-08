package com.teispace.teistro;

/**
 * One point of a chart and one of the partner's the same distance from the
 * equator within the orb ({@code 03-design/western-declinations.md}):
 * {@code first} is the chart's, {@code second} the partner's.
 *
 * @param first the chart's point
 * @param second the partner's point
 * @param contrary whether the two stand on opposite sides of the equator (C243)
 * @param apartDeg how far apart their distances from the equator are, degrees
 * @param orbDeg the orb the request allowed, degrees
 */
public record SynastryParallelRow(
        NatalPoint first, NatalPoint second, boolean contrary, double apartDeg, double orbDeg) {}
