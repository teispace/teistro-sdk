package com.teispace.teistro;

/**
 * A solar eclipse: its kind at greatest, gamma, magnitude and where on the
 * Earth it is greatest.
 *
 * @param greatest the greatest eclipse, a UT1 Julian day
 * @param kind its kind at greatest
 * @param gamma its gamma
 * @param magnitude its magnitude
 * @param point where on the Earth it is greatest
 */
public record SolarEclipse(double greatest, SolarEclipseKind kind, double gamma, double magnitude, EclipsePoint point) {}
