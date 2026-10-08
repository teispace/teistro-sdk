package com.teispace.teistro;

/**
 * A planet equally distant from two others (Leo, <i>How to Judge a
 * Nativity</i>, pp. 47–48), within the orb of the axis through their
 * midpoint: the pair in catalogue order, and the planet between.
 *
 * @param first the pair's first
 * @param second the pair's second
 * @param middle the planet between
 * @param far whether it stands opposite the midpoint of the pair's shorter
 *     arc, on the longer arc's midpoint (C246)
 * @param distanceDeg how far it stands from each of the two, the mean of the two arcs, degrees
 * @param fromAxisDeg how far it stands from the nearer point of the axis, degrees
 * @param orbDeg the orb the request allowed, degrees
 */
public record MidpointRow(
        Graha first, Graha second, Graha middle, boolean far, double distanceDeg, double fromAxisDeg,
        double orbDeg) {}
