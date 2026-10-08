package com.teispace.teistro;

/**
 * One pair of planets the same distance from the equator within the orb,
 * the pair in catalogue order.
 *
 * @param first the first planet
 * @param second the second planet
 * @param contrary whether the two stand on opposite sides of the equator (C243)
 * @param apartDeg how far apart their distances from the equator are, degrees
 * @param orbDeg the orb the request allowed, degrees
 */
public record ParallelRow(Graha first, Graha second, boolean contrary, double apartDeg, double orbDeg) {}
