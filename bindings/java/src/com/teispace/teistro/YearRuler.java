package com.teispace.teistro;

import java.util.List;

/**
 * One office of the year and who holds it ({@code 03-design/year-rulers.md}).
 *
 * @param office the office: {@code RAJA}, {@code MANTRI}, {@code SENADHIPATI},
 *     {@code SASYADHIPATI}, {@code DHANYADHIPATI}, {@code ARGHADHIPATI},
 *     {@code MEGHADHIPATI}, {@code RASADHIPATI} or {@code NIRASADHIPATI}
 * @param event the new moon that opened the year for the raja, the Sun's
 *     ingress for the rest, a UTC Julian day
 * @param texts the texts that name the office: {@code JYOTIRNIBANDHA},
 *     {@code NARADA_SAMHITA}, {@code NAVANAYAKA}
 * @param readings its weekday and lord under each rule its texts give, the
 *     plainest first
 */
public record YearRuler(YearOffice office, double event, List<RulerText> texts, List<RulerReading> readings) {
    /** Keeps the lists unmodifiable. */
    public YearRuler {
        texts = List.copyOf(texts);
        readings = List.copyOf(readings);
    }

    /**
     * The lord under the first rule, the plainest reading.
     *
     * @return the graha holding the office
     */
    public Graha lord() {
        return readings.get(0).lord();
    }
}
