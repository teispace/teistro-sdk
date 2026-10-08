package com.teispace.teistro;

import java.util.LinkedHashMap;
import java.util.Map;

/**
 * How a chart's almutens are read, Lilly's by default
 * ({@code 03-design/essential-dignities.md} §The almuten).
 *
 * <pre>{@code
 * AlmutenRules sign = new AlmutenRules(PlaceReading.SIGN, FortuneRule.DAY_AND_NIGHT);
 * }</pre>
 *
 * @param place what of a place its dignities are counted from: the degree
 *     (all five) or the sign (house, exaltation, triplicity), C218
 * @param fortune how Fortune is taken by night: Lilly's, reversed, or
 *     reversed while the Moon is up (C220, C221)
 */
public record AlmutenRules(PlaceReading place, FortuneRule fortune) {
    /**
     * Lilly's: the degree, and Fortune taken the same way by day and night.
     *
     * @return the default rules
     */
    public static AlmutenRules lilly() {
        return new AlmutenRules(PlaceReading.DEGREE, FortuneRule.DAY_AND_NIGHT);
    }

    /**
     * The rules as a request writes them, to hand back as a fortitude
     * request's {@code almuten}.
     *
     * @return the request record, for {@link Json#write}
     */
    public Map<String, Object> request() {
        Map<String, Object> out = new LinkedHashMap<>();
        out.put("place", place);
        out.put("fortune", fortune);
        return out;
    }
}
