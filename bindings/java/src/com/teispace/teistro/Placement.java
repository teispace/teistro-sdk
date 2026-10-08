package com.teispace.teistro;

/**
 * Where a graha sits in a set of bhavas.
 *
 * @param bhava The bhava, 1 to 12.
 * @param method The house system that produced it.
 * @param through How far through the bhava it is, 0 to 1.
 * @param fromMadhyaDeg Its distance from the bhava's centre, degrees.
 */
public record Placement(
        int bhava,
        HouseSystem method,
        double through,
        double fromMadhyaDeg) {
}
