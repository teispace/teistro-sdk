package com.teispace.teistro;

import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * The clauses of the Samjna Tantra vv. 73 to 74 that hold for the Moon, in the verses' order, none
 * weighed (C352).
 *
 * @param rules The Moon's rules it was read under, as the library wrote them.
 * @param clauses The clauses that hold.
 */
public record MoonWeakness(
        Map<String, Object> rules,
        List<String> clauses) {
    /** The value, its lists copied and unmodifiable. */
    public MoonWeakness {
        rules = Collections.unmodifiableMap(new LinkedHashMap<>(rules));
        clauses = List.copyOf(clauses);
    }
}
