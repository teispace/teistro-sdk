package com.teispace.teistro;

import java.util.Objects;

/**
 * How a sample is read against its replicates ({@code 03-design/research.md}).
 *
 * <pre>{@code
 * ResearchReplicateTest test = new ResearchReplicateTest(ResearchAlternative.GREATER, 0.95, 0.05);
 * }</pre>
 *
 * @param alternative the direction looked in
 * @param level every interval's confidence level, above 0 and under 1
 * @param alpha the level each method's verdict is read at; none when null
 */
public record ResearchReplicateTest(ResearchAlternative alternative, double level, Double alpha) {
    /** Two-sided, at 0.95, no alpha: what {@code expected} reads when given none. */
    public static final ResearchReplicateTest DEFAULT =
            new ResearchReplicateTest(ResearchAlternative.TWO_SIDED, 0.95, null);

    /** The value, its alternative required. */
    public ResearchReplicateTest {
        Objects.requireNonNull(alternative, "alternative");
    }
}
