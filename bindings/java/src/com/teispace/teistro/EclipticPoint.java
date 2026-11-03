package com.teispace.teistro;

/**
 * A point of the ecliptic in the local sky, by tropical longitude of date ({@code 03-design/sky-view.md}).
 *
 * @param kind {@code SAMPLE}, {@code ASCENDANT} or {@code DESCENDANT}
 * @param longitudeDeg tropical, on the true ecliptic of date, degrees in [0, 360)
 * @param altitudeDeg the airless altitude, degrees
 * @param refractedAltitudeDeg the altitude under the air's refraction
 * @param azimuthDeg from north through east, degrees in [0, 360)
 */
public record EclipticPoint(String kind, double longitudeDeg, double altitudeDeg, double refractedAltitudeDeg,
        double azimuthDeg) {}
