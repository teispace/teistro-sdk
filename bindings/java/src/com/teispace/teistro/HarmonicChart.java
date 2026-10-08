package com.teispace.teistro;

import java.util.List;

/**
 * A chart's harmonic chart (Addey, <i>Harmonics in Astrology</i>), in the
 * chart's own zodiac (C253).
 *
 * @param harmonic the number every longitude was multiplied by
 * @param points the planets in the catalogue's order, then the ascendant and the midheaven
 * @param rows the pairs meeting within the orb, closest first
 */
public record HarmonicChart(int harmonic, List<HarmonicPlaced> points, List<HarmonicRow> rows) {
    /** Keeps the lists unmodifiable. */
    public HarmonicChart {
        points = List.copyOf(points);
        rows = List.copyOf(rows);
    }
}
