package com.teispace.teistro;

import java.util.List;
import java.util.Map;

import com.teispace.teistro.record.Provenance;

/**
 * One period's answer as {@link ChartArea#rashifal(RashifalRequest)} hands it out, with what
 * sealed it.
 *
 * @param answer the reading, and the baseline's scores and the plans when asked
 * @param provenance what computed it, and under what; {@code inputHash} seals the request
 */
public record RashifalSealed(RashifalAnswer answer, Provenance provenance) {
    /**
     * The reading.
     *
     * @return the period's reading
     */
    public RashifalPeriod period() {
        return answer.period();
    }

    /**
     * The baseline's scores.
     *
     * @return Aries to Pisces; null unless a baseline period was asked
     */
    public List<BaselineScore> baseline() {
        return answer.baseline();
    }

    /**
     * Each sign's reading in words.
     *
     * @return Aries to Pisces, each plan a list of {@code {key, params}}; null
     *     unless plans were asked
     */
    public List<List<Map<String, Object>>> plans() {
        return answer.plans();
    }
}
