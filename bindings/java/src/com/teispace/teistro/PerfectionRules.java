package com.teispace.teistro;

import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * The rules a chart's perfection was read under.
 *
 * @param orbsDeg the seven whole orbs in the Chaldean order (p. 107)
 * @param horizonDays the horizon asked for; may be null, when unset: until
 *     the swifter significator leaves its sign (C232)
 * @param withinSign whether a third planet's contact counted only before the
 *     planet applying left its sign (C234)
 */
public record PerfectionRules(List<Double> orbsDeg, Double horizonDays, boolean withinSign) {
    /** Keeps the list unmodifiable. */
    public PerfectionRules {
        orbsDeg = List.copyOf(orbsDeg);
    }

    /**
     * The rules as a request writes them, to hand back as a perfection
     * request's {@code rules}.
     *
     * @return the request record, for {@link Json#write}
     */
    public Map<String, Object> request() {
        Map<String, Object> out = new LinkedHashMap<>();
        out.put("orbsDeg", orbsDeg);
        out.put("horizonDays", horizonDays);
        out.put("withinSign", withinSign);
        return out;
    }
}
