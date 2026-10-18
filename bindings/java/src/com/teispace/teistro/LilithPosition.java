package com.teispace.teistro;

/**
 * The Black Moon Lilith of a chart, placed as the grahas are
 * ({@code 03-design/lilith.md}): the mean apogee is a direction on the
 * ecliptic, its latitude zero to rounding and its distance zero, and the
 * osculating one's are the osculating orbit's.
 *
 * @param apogee Which lunar apogee the request named: never {@link Lilith#NONE}.
 * @param longitudeDeg Its longitude in the chart's zodiac, degrees.
 * @param tropicalDeg Its tropical longitude, degrees.
 * @param latitudeDeg Its ecliptic latitude, degrees.
 * @param distanceAu Its distance, astronomical units.
 * @param speedDegPerDay Its longitude speed, degrees per day.
 * @param house The bhava for "which house is it in".
 * @param placement The bhava of the chart's chalit, which is a different question.
 */
public record LilithPosition(
        Lilith apogee,
        double longitudeDeg,
        double tropicalDeg,
        double latitudeDeg,
        double distanceAu,
        double speedDegPerDay,
        Placement house,
        Placement placement) {
    /**
     * Whether its longitude speed is negative.
     *
     * @return true when retrograde
     */
    public boolean retrograde() {
        return speedDegPerDay < 0;
    }
}
