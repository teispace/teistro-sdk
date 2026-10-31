package com.teispace.teistro;

import java.util.Optional;

/**
 * A point on a body's track: a step's reading or an exact event.
 *
 * @param at the instant, Julian day, UT1 read as UTC
 * @param kind {@code SAMPLE}, {@code RISE}, {@code TRANSIT} or {@code SET}
 * @param altitudeDeg the airless topocentric altitude of the centre, degrees
 * @param refractedAltitudeDeg the altitude with the standard atmosphere's refraction, degrees; the airless altitude
 *     where the body stays below the horizon
 * @param azimuthDeg the topocentric azimuth, degrees in [0, 360), from north through east
 * @param shape the Moon's shape at the instant, null off the Moon's track
 */
public record SkyPoint(double at, String kind, double altitudeDeg, double refractedAltitudeDeg, double azimuthDeg,
        MoonShape shape) {
    /**
     * The Moon's shape at the instant.
     *
     * @return the shape, on the Moon's track only
     */
    public Optional<MoonShape> moon() {
        return Optional.ofNullable(shape);
    }
}
