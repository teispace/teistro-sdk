package com.teispace.teistro;

/**
 * Two planets in antiscion within the orb: in a chart's own pair the two in
 * catalogue order, across a synastry the chart's first and the partner's
 * second.
 *
 * @param first the first planet
 * @param second the second planet
 * @param contrary whether it is the contrantiscion, the reflection about the equinoxes
 * @param apartDeg how far the one's reflection stands from the other, degrees
 * @param orbDeg the orb the request allowed the pair, degrees
 */
public record AntiscionRow(Graha first, Graha second, boolean contrary, double apartDeg, double orbDeg) {}
