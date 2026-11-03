package com.teispace.teistro;

/**
 * A graha in the local sky ({@code 03-design/sky-view.md}): topocentric, apparent and of date.
 *
 * @param graha the graha
 * @param raDeg the right ascension on the true equator and equinox of date, degrees in [0, 360)
 * @param decDeg the declination, degrees
 * @param altitudeDeg the airless altitude of the centre, degrees
 * @param refractedAltitudeDeg the altitude under the air's refraction; airless below the horizon (C973)
 * @param azimuthDeg from north through east, degrees in [0, 360)
 */
public record GrahaSky(Graha graha, double raDeg, double decDeg, double altitudeDeg, double refractedAltitudeDeg,
        double azimuthDeg) {
    /**
     * Whether the refracted centre stands above the horizon.
     *
     * @return true when it does
     */
    public boolean isUp() {
        return refractedAltitudeDeg > 0.0;
    }
}
