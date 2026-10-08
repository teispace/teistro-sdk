package com.teispace.teistro;

import java.util.List;

/**
 * The composite of a chart and a synastry's partner
 * ({@code 03-design/western-composites.md}, C247): its planets in the
 * chart's order, the midheaven at the near midpoint of the two, and the
 * lagna at the near midpoint of the two lagnas, turned by 180° when that
 * stood before the midheaven, as {@code lagnaTurned} says.
 *
 * @param planets the composite planets
 * @param lagnaDeg the composite lagna
 * @param midheavenDeg the composite midheaven
 * @param lagnaTurned whether the lagna was turned by 180°
 * @param cuspsDeg the twelve composite cusps, each the near midpoint of the
 *     two charts' same cusp, turned when more than 90° from where the
 *     midheaven puts it (Astrolog); may be null, when either chart's cusps
 *     could not be read, as at a polar place
 */
public record Composite(
        List<CompositePlanet> planets, double lagnaDeg, double midheavenDeg, boolean lagnaTurned,
        List<Double> cuspsDeg) {
    /** Keeps the lists unmodifiable. */
    public Composite {
        planets = List.copyOf(planets);
        cuspsDeg = cuspsDeg == null ? null : List.copyOf(cuspsDeg);
    }
}
