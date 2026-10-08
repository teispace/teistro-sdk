package com.teispace.teistro;

import java.util.List;

/**
 * One period's answer: the reading, and the baseline's twelve scores when asked.
 *
 * @param period the reading
 * @param baseline Aries to Pisces; may be null, unless a baseline period was asked
 */
public record RashifalAnswer(RashifalPeriod period, List<BaselineScore> baseline) {
    /** Keeps the list unmodifiable. */
    public RashifalAnswer {
        baseline = baseline == null ? null : List.copyOf(baseline);
    }
}
