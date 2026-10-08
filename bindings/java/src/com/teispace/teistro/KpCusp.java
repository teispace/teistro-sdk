package com.teispace.teistro;

/**
 * A cusp, its longitude in nanoarcseconds of the sidereal zodiac.
 *
 * @param house 1 to 12.
 * @param longitude Its longitude, nanoarcseconds.
 * @param lords Its lords.
 */
public record KpCusp(
        int house,
        long longitude,
        KpLords lords) {
}
