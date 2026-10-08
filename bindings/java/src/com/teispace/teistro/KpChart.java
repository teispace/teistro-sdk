package com.teispace.teistro;

import java.util.List;

/**
 * A chart as KP reads it: its cusps, the horary number's when one was named, and its planets.
 *
 * @param system The house system of the cusps.
 * @param cusps The twelve cusps.
 * @param planets The planets.
 */
public record KpChart(
        HouseSystem system,
        List<KpCusp> cusps,
        List<KpPlanet> planets) {
    /** The value, its lists copied and unmodifiable. */
    public KpChart {
        cusps = List.copyOf(cusps);
        planets = List.copyOf(planets);
    }
}
