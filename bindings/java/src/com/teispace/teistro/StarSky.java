package com.teispace.teistro;

/**
 * A star in the local sky, as a graha's ({@code 03-design/sky-view.md}).
 *
 * @param star the star
 * @param nakshatra the nakshatra it is the yogatara of, or null
 * @param magnitude its visual magnitude, or null
 * @param raDeg the right ascension on the true equator and equinox of date, degrees in [0, 360)
 * @param decDeg the declination, degrees
 * @param altitudeDeg the airless altitude, degrees
 * @param refractedAltitudeDeg the altitude under the air's refraction; airless below the horizon (C973)
 * @param azimuthDeg from north through east, degrees in [0, 360)
 */
public record StarSky(Star star, Nakshatra nakshatra, Double magnitude, double raDeg, double decDeg,
        double altitudeDeg, double refractedAltitudeDeg, double azimuthDeg) {}
