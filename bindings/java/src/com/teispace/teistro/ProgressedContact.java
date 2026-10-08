package com.teispace.teistro;

/**
 * One exact aspect a progressed planet makes to a radical point.
 *
 * @param life the instant of life it falls due, a UTC Julian day
 * @param sky the instant of sky it is exact at
 * @param graha the progressed planet
 * @param to the radical point
 * @param angle a whole degree 0 to 180
 * @param motion the planet's motion
 */
public record ProgressedContact(double life, double sky, Graha graha, NatalPoint to, int angle, Motion motion) {}
