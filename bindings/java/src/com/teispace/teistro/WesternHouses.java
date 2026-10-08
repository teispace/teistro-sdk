package com.teispace.teistro;

import java.util.List;

/**
 * A chart's Western houses ({@code 03-design/western-houses.md}), in the
 * chart's own zodiac.
 *
 * @param system the division the cusps are of: the one asked, or the one a
 *     polar policy fell back to
 * @param cuspsDeg the twelve cusps, first to twelfth, degrees
 * @param ascendantDeg the ascendant
 * @param reachDeg the degree that rose one sidereal hour before the birth (C250)
 * @param planets each planet's house
 */
public record WesternHouses(
        HouseSystem system, List<Double> cuspsDeg, double ascendantDeg, double reachDeg,
        List<WesternHousePlacement> planets) {
    /** Keeps the lists unmodifiable. */
    public WesternHouses {
        cuspsDeg = List.copyOf(cuspsDeg);
        planets = List.copyOf(planets);
    }
}
