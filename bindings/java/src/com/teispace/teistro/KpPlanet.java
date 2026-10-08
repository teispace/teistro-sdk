package com.teispace.teistro;

/**
 * A planet, its longitude in nanoarcseconds of the sidereal zodiac.
 *
 * @param graha Which planet.
 * @param longitude Its longitude, nanoarcseconds.
 * @param retrograde Whether it is retrograde.
 * @param house The house whose cusp arc holds it, 1 to 12.
 * @param lords Its lords.
 */
public record KpPlanet(
        Graha graha,
        long longitude,
        boolean retrograde,
        int house,
        KpLords lords) {
}
