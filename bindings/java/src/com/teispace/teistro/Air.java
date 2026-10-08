package com.teispace.teistro;

/**
 * An air as it was applied: a part the settings left out is the engines' standard at the place,
 * the ICAO atmosphere's pressure at its height and 15 degrees Celsius.
 *
 * @param pressureHpa The pressure at the observer, hectopascals.
 * @param temperatureC The temperature at the observer, degrees Celsius.
 */
public record Air(
        double pressureHpa,
        double temperatureC) {
}
