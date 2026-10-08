package com.teispace.teistro;

/**
 * The stretch of an eclipse the place sees, the body above its horizon.
 *
 * @param from a UT1 Julian day
 * @param to a UT1 Julian day
 */
public record EclipseSeen(double from, double to) {}
