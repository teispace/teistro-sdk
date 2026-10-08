package com.teispace.teistro;

import java.util.List;

/**
 * A chart's dasha phala, read under {@code dasha.shanta_sign}.
 *
 * @param grahas Each graha's, Sun to Ketu.
 */
public record DashaPhalaReading(
        List<GrahaDashaPhala> grahas) {
    /** The value, its lists copied and unmodifiable. */
    public DashaPhalaReading {
        grahas = List.copyOf(grahas);
    }
}
