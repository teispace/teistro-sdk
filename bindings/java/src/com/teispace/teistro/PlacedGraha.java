package com.teispace.teistro;

/**
 * One graha of a chart, read out of the batch's columns.
 *
 * @param graha Which graha.
 * @param longitudeDeg Its longitude in the chart's zodiac, degrees.
 * @param tropicalDeg Its tropical longitude, degrees.
 * @param latitudeDeg Its ecliptic latitude, degrees.
 * @param distanceAu Its distance, astronomical units.
 * @param speedDegPerDay Its longitude speed, degrees per day.
 * @param house The bhava for "which house is it in".
 * @param placement The bhava of the chart's chalit, which is a different question.
 */
public record PlacedGraha(
        Graha graha,
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
