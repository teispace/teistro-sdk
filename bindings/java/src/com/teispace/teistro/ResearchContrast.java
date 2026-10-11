package com.teispace.teistro;

/**
 * What a design's groups are compared by ({@code 03-design/research.md}).
 *
 * <pre>{@code
 * ResearchContrast cases = ResearchContrast.caseVsRest(1);
 * ResearchContrast any = ResearchContrast.ANY_DIFFERENCE;
 * }</pre>
 */
public sealed interface ResearchContrast {
    /** Whether the share differs between any of the groups. */
    ResearchContrast ANY_DIFFERENCE = new AnyDifference();

    /**
     * One group, the cases, against every other chart.
     *
     * @param cases the label of the cases' group
     * @return the contrast
     */
    static ResearchContrast caseVsRest(int cases) {
        return new CaseVsRest(cases);
    }

    /**
     * One group, the cases, against every other chart.
     *
     * @param cases the label of the cases' group
     */
    record CaseVsRest(int cases) implements ResearchContrast {}

    /** Whether the share differs between any of the groups; {@link #ANY_DIFFERENCE}. */
    record AnyDifference() implements ResearchContrast {}
}
