package com.teispace.teistro;

/**
 * The Sun at one of a lunar month's two new moons, against the nearest sign
 * boundary (SR-92).
 *
 * @param newMoon the new moon, as a Julian day (UTC)
 * @param degrees the Sun's sidereal longitude less the nearest sign boundary,
 *     degrees, -15 to 15: positive when the Sun had passed it, negative when
 *     short of it
 * @param minutes the new moon less the nearest sankranti, minutes, with the
 *     same sign
 */
public record NewMoonMargin(double newMoon, double degrees, double minutes) {}
