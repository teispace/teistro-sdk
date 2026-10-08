package com.teispace.teistro;

/**
 * A planet's distance from the equator.
 *
 * @param graha the planet
 * @param declinationDeg degrees north of the equator
 */
public record Declined(Graha graha, double declinationDeg) {}
