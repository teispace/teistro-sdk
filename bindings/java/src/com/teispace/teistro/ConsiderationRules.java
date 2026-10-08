package com.teispace.teistro;

import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * The rules a chart's considerations were read under, every field filled.
 *
 * @param moonLateFromDeg from what degree of her sign the Moon is late (C229)
 * @param orbsDeg the seven whole orbs in the Chaldean order: Saturn,
 *     Jupiter, Mars, the Sun, Venus, Mercury, the Moon (C230)
 */
public record ConsiderationRules(double moonLateFromDeg, List<Double> orbsDeg) {
    /** Keeps the list unmodifiable. */
    public ConsiderationRules {
        orbsDeg = List.copyOf(orbsDeg);
    }

    /**
     * The rules as a request writes them, to hand back as a considerations request.
     *
     * @return the request record, for {@link Json#write}
     */
    public Map<String, Object> request() {
        Map<String, Object> out = new LinkedHashMap<>();
        out.put("moonLateFromDeg", moonLateFromDeg);
        out.put("orbsDeg", orbsDeg);
        return out;
    }
}
