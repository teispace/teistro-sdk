package com.teispace.teistro;

import java.util.List;

/**
 * Every graha's transit at one instant, read from the natal chart.
 *
 * @param instant The instant, a UTC Julian day.
 * @param reference What the houses were counted from.
 * @param rules The readings it was judged under.
 * @param grahas Each graha's, the Sun to Ketu.
 * @param ashtakavarga The seven judged by the natal Ashtakavarga, Sun to Saturn; null unless
 *     {@code ashtakavarga} asked. May be null.
 */
public record GocharReading(
        double instant,
        GocharReference reference,
        GocharRules rules,
        List<GrahaGochar> grahas,
        List<AshtakavargaTransit> ashtakavarga) {
    /** The value, its lists copied and unmodifiable. */
    public GocharReading {
        grahas = List.copyOf(grahas);
        ashtakavarga = ashtakavarga == null ? null : List.copyOf(ashtakavarga);
    }
}
