package com.teispace.teistro;

/**
 * One point of a chart and one of the partner's within an aspect's orb
 * ({@code 03-design/western-synastry.md}): {@code first} is the chart's,
 * {@code second} the partner's.
 *
 * @param first the chart's point
 * @param second the partner's point
 * @param aspect the aspect
 * @param apartDeg the shorter arc between them, degrees 0 to 180
 * @param fromExactDeg how far that arc is from the aspect's exact angle, degrees
 * @param orbDeg the orb the model allowed the pair at this aspect, degrees
 */
public record SynastryRow(
        NatalPoint first, NatalPoint second, WesternAspect aspect, double apartDeg, double fromExactDeg,
        double orbDeg) {}
