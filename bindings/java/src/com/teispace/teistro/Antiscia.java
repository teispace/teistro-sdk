package com.teispace.teistro;

import java.util.List;

/**
 * A chart's antiscia (Lilly, <i>Christian Astrology</i>, pp. 90–92): each
 * planet's reflections, the pairs within the orb closest first, and the
 * planets the orbs give none.
 *
 * @param points each planet's reflections
 * @param pairs the pairs within the orb, closest first
 * @param unpaired the planets in no pair
 * @param onCusps the reflections upon a cusp's very degree (C251); empty
 *     unless the request asked {@code cusps}
 * @param cuspSystem the division {@code onCusps} was read in; may be null,
 *     unless asked
 */
public record Antiscia(
        List<Antiscion> points, List<AntiscionRow> pairs, List<Graha> unpaired, List<CuspAntiscion> onCusps,
        HouseSystem cuspSystem) {
    /** Keeps the lists unmodifiable. */
    public Antiscia {
        points = List.copyOf(points);
        pairs = List.copyOf(pairs);
        unpaired = List.copyOf(unpaired);
        onCusps = List.copyOf(onCusps);
    }
}
