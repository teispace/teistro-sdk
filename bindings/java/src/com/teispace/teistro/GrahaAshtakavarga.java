package com.teispace.teistro;

import java.util.List;

/**
 * One graha's Ashtakavarga.
 *
 * @param graha Which graha, Sun to Saturn.
 * @param bindus Its bindus by sign, Aries to Pisces, 0 to 8.
 * @param reduced The same after both reductions, when they were made in each graha's own
 *     Ashtakavarga; null otherwise. May be null.
 * @param rashiPinda Its rashi pinda.
 * @param grahaPinda Its graha pinda.
 * @param yogaPinda Its yoga pinda, the two together.
 */
public record GrahaAshtakavarga(
        Graha graha,
        List<Integer> bindus,
        List<Integer> reduced,
        long rashiPinda,
        long grahaPinda,
        long yogaPinda) {
    /** The value, its lists copied and unmodifiable. */
    public GrahaAshtakavarga {
        bindus = List.copyOf(bindus);
        reduced = reduced == null ? null : List.copyOf(reduced);
    }
}
