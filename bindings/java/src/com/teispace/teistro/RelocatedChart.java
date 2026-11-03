package com.teispace.teistro;

import java.util.List;

/**
 * One place's angles and houses at a chart's instant
 * ({@code 03-design/astrocartography.md}).
 *
 * @param latitudeDeg the place's latitude, north positive
 * @param longitudeDeg the place's longitude, east positive
 * @param altitudeM the place's altitude in metres
 * @param ascendantDeg the ascendant, in the context's zodiac
 * @param midheavenDeg the midheaven, in the context's zodiac
 * @param cuspsDeg the twelve cusps, the first house's first
 * @param houseSystem the house division the cusps are of
 */
public record RelocatedChart(
        double latitudeDeg,
        double longitudeDeg,
        double altitudeM,
        double ascendantDeg,
        double midheavenDeg,
        List<Double> cuspsDeg,
        HouseSystem houseSystem) {
    /** Keeps the cusps unmodifiable. */
    public RelocatedChart {
        cuspsDeg = List.copyOf(cuspsDeg);
    }
}
