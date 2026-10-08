package com.teispace.teistro;

/**
 * A birth's planet moved by the direction's arc.
 *
 * @param graha the planet
 * @param longitudeDeg where the arc moved it
 */
public record DirectedPlanet(Graha graha, double longitudeDeg) {}
