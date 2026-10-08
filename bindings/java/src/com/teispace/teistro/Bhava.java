package com.teispace.teistro;

/**
 * One of the twelve bhavas.
 *
 * @param madhyaDeg The bhava's centre, degrees.
 * @param sandhiDeg The bhava's opening cusp, degrees.
 */
public record Bhava(
        double madhyaDeg,
        double sandhiDeg) {
}
