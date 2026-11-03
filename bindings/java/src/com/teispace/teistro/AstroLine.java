package com.teispace.teistro;

import java.util.List;

/**
 * One body's line on one angle.
 *
 * @param body the body
 * @param angle {@code MC}, {@code IC}, {@code ASC} or {@code DSC}
 * @param points {@code [longitudeDeg, latitudeDeg]} pairs, south to north
 * @param polarLimitDeg where a rising or setting line stops, when it stops inside the map; otherwise null
 */
public record AstroLine(Graha body, String angle, List<List<Double>> points, Double polarLimitDeg) {
    /** Keeps the points unmodifiable. */
    public AstroLine {
        points = points.stream().map(List::copyOf).toList();
    }
}
