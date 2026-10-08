package com.teispace.teistro;

/**
 * An equal distance across a synastry ({@code 03-design/western-midpoints.md},
 * decision 9): a planet of one chart on the axis through two of the
 * other's. Python's subclass of {@link MidpointRow}; a record cannot extend
 * one, so it carries the same components and {@link #row()} reads it as one.
 *
 * @param first the pair's first
 * @param second the pair's second
 * @param middle the planet between
 * @param far whether it stands opposite the midpoint of the pair's shorter
 *     arc, on the longer arc's midpoint (C246)
 * @param distanceDeg how far it stands from each of the two, the mean of the two arcs, degrees
 * @param fromAxisDeg how far it stands from the nearer point of the axis, degrees
 * @param orbDeg the orb the request allowed, degrees
 * @param partnersPair true when the pair is the partner's and {@code middle}
 *     the chart's planet
 */
public record SynastryMidpointRow(
        Graha first, Graha second, Graha middle, boolean far, double distanceDeg, double fromAxisDeg,
        double orbDeg, boolean partnersPair) {
    /**
     * The equal distance without the side it was read across.
     *
     * @return the row as a chart's own would be
     */
    public MidpointRow row() {
        return new MidpointRow(first, second, middle, far, distanceDeg, fromAxisDeg, orbDeg);
    }
}
