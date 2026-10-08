package com.teispace.teistro;

import java.util.List;

/**
 * A chart's Vaiseshikamsa.
 *
 * @param grahas Each graha's, Sun to Saturn.
 */
public record VaiseshikamsaReading(
        List<GrahaVaiseshikamsa> grahas) {
    /** The value, its lists copied and unmodifiable. */
    public VaiseshikamsaReading {
        grahas = List.copyOf(grahas);
    }
}
