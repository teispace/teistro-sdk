package com.teispace.teistro;

import java.util.List;

/**
 * One shape a chart's planets make, of aspects the chart holds under the request's orbs
 * ({@code 03-design/western-configurations.md}).
 *
 * @param kind Which shape.
 * @param bodies Its bodies, in the order the chart's planets are read.
 * @param apex The graha a T-square, yod or kite points at; null for the others.
 * @param sign The tropical rashi a stellium stands in; null for the others.
 * @param widestFromExactDeg How far from exact its loosest aspect is, degrees; null for a stellium.
 * @param inside The positions in the chart's list of the larger shapes this one is wholly inside;
 *     empty when it stands alone.
 */
public record WesternConfiguration(
        WesternConfigurationKind kind,
        List<Graha> bodies,
        Graha apex,
        Rashi sign,
        Double widestFromExactDeg,
        List<Integer> inside) {
    /** Copies the lists, so the record's own cannot change. */
    public WesternConfiguration {
        bodies = List.copyOf(bodies);
        inside = List.copyOf(inside);
    }
}
