package com.teispace.teistro;

import java.util.List;

/**
 * One native's Kuja dosha.
 *
 * @param readings Mars's house from the lagna, the Moon and Venus, whatever the rules count.
 * @param dosha Whether Mars stands in one of the rules' houses from a reference the rules count.
 */
public record KujaSide(
        List<KujaReading> readings,
        boolean dosha) {
    /** The value, its lists copied and unmodifiable. */
    public KujaSide {
        readings = List.copyOf(readings);
    }
}
