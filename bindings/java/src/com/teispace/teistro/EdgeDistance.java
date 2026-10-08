package com.teispace.teistro;

/**
 * How near a body stands to a boundary, which is what an ayanamsha that moved would change.
 *
 * @param signDeg To the nearer edge of its sign, degrees.
 * @param nakshatraDeg To the nearer edge of its nakshatra, degrees.
 * @param padaDeg To the nearer edge of its pada, degrees.
 */
public record EdgeDistance(
        double signDeg,
        double nakshatraDeg,
        double padaDeg) {
}
