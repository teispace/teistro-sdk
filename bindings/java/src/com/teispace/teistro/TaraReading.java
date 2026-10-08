package com.teispace.teistro;

import java.util.LinkedHashMap;
import java.util.Map;

/**
 * The birth star's count to the day's, and the tara it gives.
 *
 * @param count the count from the birth star to the day's
 * @param tara one of {@code JANMA}, {@code SAMPAT}, {@code VIPAT},
 *     {@code KSHEMA}, {@code PRATYAK}, {@code SADHANA}, {@code NAIDHANA},
 *     {@code MITRA} and {@code PARAMA_MITRA}
 * @param cycle which round of nine the count is in
 */
public record TaraReading(int count, String tara, int cycle) {
    /**
     * The reading as a request writes it.
     *
     * @return the request record, for {@link Json#write}
     */
    public Map<String, Object> request() {
        Map<String, Object> out = new LinkedHashMap<>();
        out.put("count", count);
        out.put("tara", tara);
        out.put("cycle", cycle);
        return out;
    }

    static TaraReading read(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        return new TaraReading(Reads.integer(o, "count"), Reads.string(o, "tara"), Reads.integer(o, "cycle"));
    }
}
