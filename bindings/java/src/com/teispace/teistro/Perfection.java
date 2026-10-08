package com.teispace.teistro;

/**
 * An aspect the Moon perfects with a planet.
 *
 * @param planet the planet
 * @param aspect the aspect
 * @param days days until it is exact, at the motions of the moment
 * @param gapDeg how far it is from exact now, degrees
 */
public record Perfection(Graha planet, PtolemaicAspect aspect, double days, double gapDeg) {}
