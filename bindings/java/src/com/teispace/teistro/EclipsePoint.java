package com.teispace.teistro;

/**
 * Where on the Earth a solar eclipse is greatest, in degrees.
 *
 * @param latitude degrees north
 * @param longitude degrees east
 */
public record EclipsePoint(double latitude, double longitude) {}
