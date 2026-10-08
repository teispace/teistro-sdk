package com.teispace.teistro;

import java.util.LinkedHashMap;
import java.util.Map;

/**
 * A nakshatra's quarter.
 *
 * @param nakshatra the nakshatra
 * @param pada the quarter, 1 to 4
 */
public record MuhurtaPada(Nakshatra nakshatra, int pada) {
    /**
     * The quarter as a request writes it.
     *
     * @return the request record, for {@link Json#write}
     */
    public Map<String, Object> request() {
        Map<String, Object> out = new LinkedHashMap<>();
        out.put("nakshatra", nakshatra);
        out.put("pada", pada);
        return out;
    }

    static MuhurtaPada read(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        String key = Reads.string(o, "nakshatra");
        return new MuhurtaPada(Reads.member(Nakshatra.byKey(key), "Nakshatra", key), Reads.integer(o, "pada"));
    }
}
