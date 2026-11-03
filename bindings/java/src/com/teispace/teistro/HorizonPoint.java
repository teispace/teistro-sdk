package com.teispace.teistro;

/**
 * A point of the airless horizon, and where it lies on the sphere of date ({@code 03-design/sky-view.md}).
 *
 * @param azimuthDeg from north through east, degrees in [0, 360)
 * @param raDeg the right ascension on the true equator of date, degrees
 * @param decDeg the declination, degrees
 * @param longitudeDeg tropical, on the true ecliptic of date, degrees
 * @param latitudeDeg from that ecliptic, degrees
 */
public record HorizonPoint(double azimuthDeg, double raDeg, double decDeg, double longitudeDeg,
        double latitudeDeg) {}
