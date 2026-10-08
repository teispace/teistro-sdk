package com.teispace.teistro;

import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * The 12th from the karakamsha in one chart, read as BPHS vv. 70 to 76 read it: its devotions in
 * catalogue order (empty when the sign is empty), and the grahas there that make a devotee of
 * minor deities in a malefic's sign: Saturn or Venus from the karakamsha (vv. 75 to 76), every
 * natural malefic too from the amatya (v. 78).
 *
 * @param rules The devata rules it was read under, as the library wrote them.
 * @param sign The 12th's sign.
 * @param devotions Its devotions.
 * @param minor The grahas that make a devotee of minor deities.
 */
public record IshtaDevata(
        Map<String, Object> rules,
        Rashi sign,
        List<Devotion> devotions,
        List<Graha> minor) {
    /** The value, its lists copied and unmodifiable. */
    public IshtaDevata {
        rules = Collections.unmodifiableMap(new LinkedHashMap<>(rules));
        devotions = List.copyOf(devotions);
        minor = List.copyOf(minor);
    }
}
