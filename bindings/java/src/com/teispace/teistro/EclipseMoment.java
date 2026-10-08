package com.teispace.teistro;

/**
 * One moment of an eclipse at the place: when, and the body's topocentric
 * geometric altitude there.
 *
 * @param at a UT1 Julian day
 * @param altitudeDeg the eclipsed body's centre above the horizon, before refraction
 */
public record EclipseMoment(double at, double altitudeDeg) {}
