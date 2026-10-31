package com.teispace.teistro;

import java.util.List;

import com.teispace.teistro.record.Provenance;

/**
 * A range's change points, sorted, with the window they were read over
 * ({@code 03-design/change-points.md}).
 *
 * @param points every point at or after the window's start and before its end
 * @param window the instants the range covers
 * @param provenance what computed it, and the clock it was asked on
 */
public record ChangePoints(List<Point> points, Interval window, Provenance provenance) {
    /** Copies the points. */
    public ChangePoints {
        points = List.copyOf(points);
    }

    /**
     * One instant at which something changes.
     *
     * @param at the instant, as a UTC Julian day
     * @param kind what changes: {@code TITHI}, {@code NAKSHATRA}, {@code YOGA},
     *     {@code KARANA}, {@code MOON_SIGN}, {@code SUN_SIGN} or {@code VARA}
     * @param from the catalogue key of the member that ended
     * @param to the catalogue key of the member that began
     */
    public record Point(double at, String kind, String from, String to) {
    }
}
