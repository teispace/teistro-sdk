package com.teispace.teistro;

/**
 * One derived point: an upagraha or a special lagna.
 *
 * @param point Which point.
 * @param longitudeDeg Its longitude in the chart's zodiac, degrees.
 * @param sign The sign it falls in.
 * @param boundaries How near it stands to a sign, nakshatra or pada edge.
 */
public record DerivedPoint(
        Point point,
        double longitudeDeg,
        Rashi sign,
        EdgeDistance boundaries) {
}
