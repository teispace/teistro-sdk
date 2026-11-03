package com.teispace.teistro;

import java.util.List;
import java.util.Optional;

/**
 * The sky at a place and an instant ({@code 03-design/sky-view.md}): each graha and each star asked, topocentric,
 * apparent and of date, and the ecliptic and the horizon as point lists.
 *
 * @param at the instant, UT1 read as UTC, a Julian day
 * @param siderealTimeDeg the local apparent sidereal time, degrees
 * @param obliquityDeg the true obliquity, degrees
 * @param pressureHpa the air's pressure the refraction was reckoned under, hPa
 * @param temperatureC the air's temperature, degrees Celsius
 * @param eclipticStepDeg the ecliptic's step, degrees
 * @param horizonStepDeg the horizon's step, degrees
 * @param grahas each graha asked, in the order asked
 * @param stars each star asked, in the order asked
 * @param ecliptic every step of longitude and the two crossings, in longitude order
 * @param horizon every step of azimuth from north
 */
public record SkyView(double at, double siderealTimeDeg, double obliquityDeg, double pressureHpa,
        double temperatureC, double eclipticStepDeg, double horizonStepDeg, List<GrahaSky> grahas,
        List<StarSky> stars, List<EclipticPoint> ecliptic, List<HorizonPoint> horizon) {
    /** Keeps the lists unmodifiable. */
    public SkyView {
        grahas = List.copyOf(grahas);
        stars = List.copyOf(stars);
        ecliptic = List.copyOf(ecliptic);
        horizon = List.copyOf(horizon);
    }

    /**
     * A graha's place, if it was asked.
     *
     * <pre>{@code
     * double azimuth = view.graha(Graha.MOON).orElseThrow().azimuthDeg();
     * }</pre>
     *
     * @param graha the graha
     * @return its place, or empty when it was not asked
     */
    public Optional<GrahaSky> graha(Graha graha) {
        return grahas.stream().filter(placed -> placed.graha() == graha).findFirst();
    }
}
