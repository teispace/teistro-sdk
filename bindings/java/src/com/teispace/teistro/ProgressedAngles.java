package com.teispace.teistro;

/**
 * The progressed angles, by the request's {@code angles} (C237).
 *
 * @param ascendantDeg the progressed ascendant
 * @param midheavenDeg the progressed midheaven
 */
public record ProgressedAngles(double ascendantDeg, double midheavenDeg) {}
