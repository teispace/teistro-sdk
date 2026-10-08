package com.teispace.teistro;

/**
 * A point's place in a harmonic chart.
 *
 * @param point the point
 * @param longitudeDeg its longitude multiplied by the harmonic, degrees in [0, 360)
 * @param house its equal house from the harmonic ascendant, 1 to 12 (C254)
 */
public record HarmonicPlaced(HarmonicPoint point, double longitudeDeg, int house) {}
