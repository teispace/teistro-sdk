package com.teispace.teistro;

/**
 * One body looking at another.
 *
 * @param fromGraha The body looking.
 * @param to The body looked at.
 * @param houses Which house of the first's sign the second stands in, counting inclusively from
 *     one.
 * @param strength How strongly.
 * @param fromEdge How near the looking body stands to a boundary.
 * @param toEdge How near the body looked at stands to one.
 */
public record Drishti(
        Graha fromGraha,
        Graha to,
        int houses,
        Strength strength,
        EdgeDistance fromEdge,
        EdgeDistance toEdge) {
}
