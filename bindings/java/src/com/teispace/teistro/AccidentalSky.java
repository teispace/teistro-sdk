package com.teispace.teistro;

import java.util.List;

/**
 * What a chart's accidental fortitudes were read from, in the chart's zodiac.
 *
 * @param houses Regiomontanus, Lilly's, unless a profile names another
 *     division for the {@code hellenistic} module
 * @param cuspsDeg the twelve cusps, the first to the twelfth
 * @param ascendantDeg the ascendant, from the chart's angles: whole-sign and
 *     equal houses do not put it on a cusp
 * @param midheavenDeg the midheaven, from the chart's angles
 * @param speedsDegPerDay the seven's daily motions in the Chaldean order,
 *     negative when retrograde
 * @param northNodeDeg the north node
 * @param regulusDeg the star's apparent place of date, as are Spica's and Algol's
 * @param spicaDeg Spica's apparent place of date
 * @param algolDeg Algol's apparent place of date
 */
public record AccidentalSky(
        HouseSystem houses, List<Double> cuspsDeg, double ascendantDeg, double midheavenDeg,
        List<Double> speedsDegPerDay, double northNodeDeg, double regulusDeg, double spicaDeg, double algolDeg) {
    /** Keeps the lists unmodifiable. */
    public AccidentalSky {
        cuspsDeg = List.copyOf(cuspsDeg);
        speedsDegPerDay = List.copyOf(speedsDegPerDay);
    }
}
