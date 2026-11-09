package com.teispace.teistro;

import java.util.List;
import java.util.Map;

/**
 * One period's answer: the reading, the baseline's twelve scores and the
 * twelve plans when asked.
 *
 * @param period the reading
 * @param baseline Aries to Pisces; may be null, unless a baseline period was asked
 * @param plans each sign's reading in words, Aries to Pisces, each plan a list of
 *     {@code {key, params}} that {@code intl().render} says; may be null, unless
 *     plans were asked
 */
public record RashifalAnswer(RashifalPeriod period, List<BaselineScore> baseline,
        List<List<Map<String, Object>>> plans) {
    /** Keeps the lists unmodifiable. */
    public RashifalAnswer {
        baseline = baseline == null ? null : List.copyOf(baseline);
        plans = plans == null ? null : plans.stream().map(List::copyOf).toList();
    }
}
