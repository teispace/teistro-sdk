package com.teispace.teistro;

import java.util.List;
import java.util.Optional;

/**
 * One native's Kuja dosha.
 *
 * @param readings Mars's house from the lagna, the Moon and Venus, whatever the rules count.
 * @param dosha Whether Mars stands in one of the rules' houses from a reference the rules count.
 * @param held Every lift that holds, whatever the rules count; empty where the chart gave no sky
 *     to read them in.
 * @param lifted Whether the side carries the dosha and a lift the rules count holds.
 */
public record KujaSide(
        List<KujaReading> readings,
        boolean dosha,
        Optional<KujaLifts> held,
        boolean lifted) {
    /** The value, its lists copied and unmodifiable. */
    public KujaSide {
        readings = List.copyOf(readings);
    }
}
