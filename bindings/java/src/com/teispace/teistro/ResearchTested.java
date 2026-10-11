package com.teispace.teistro;

import java.util.List;

import com.teispace.teistro.record.Provenance;

/**
 * A test's rows ({@code 03-design/research.md}), per predicate and never a single verdict.
 *
 * @param rows one row per predicate, in the request's order
 * @param permutations how many permutations were drawn
 * @param resolution the smallest p-value they can give, {@code 1/(m + 1)}
 * @param shuffle the generator and shuffle that drew them, so a reader can rerun the study
 * @param provenance how it was produced; its {@code inputHash} is the study's pre-registration
 */
public record ResearchTested(List<Row> rows, int permutations, double resolution, String shuffle,
        Provenance provenance) {
    /**
     * An estimate with its interval at the test's level.
     *
     * @param estimate the point estimate
     * @param low the interval's lower end
     * @param high its upper end
     */
    public record Estimate(double estimate, double low, double high) {
    }

    /**
     * A permutation p-value, {@code (exceed + 1)/(m + 1)} and never zero, with its Clopper–Pearson
     * interval.
     *
     * @param exceed how many permutations were at least as extreme
     * @param value the p-value
     * @param low the interval's lower end
     * @param high its upper end
     */
    public record PValue(long exceed, double value, double low, double high) {
    }

    /**
     * The family's adjusted p-values.
     *
     * @param maxT Westfall–Young step-down max-T
     * @param holm Holm's step-down
     * @param bonferroni Bonferroni
     * @param bh Benjamini–Hochberg
     * @param by Benjamini–Yekutieli
     */
    public record Adjusted(double maxT, double holm, double bonferroni, double bh, double by) {
    }

    /**
     * The cases against the rest.
     *
     * @param riskCase the cases' share, with Wilson's interval
     * @param riskRest the rest's share
     * @param riskDifference their difference, with Newcombe's interval
     * @param riskRatio their ratio, with Katz's interval; null when either share is empty
     * @param oddsRatio the odds ratio; null when a cell is empty
     * @param cohenH Cohen's h
     */
    public record Effect(Estimate riskCase, Estimate riskRest, Estimate riskDifference, Estimate riskRatio,
            Double oddsRatio, double cohenH) {
    }

    /**
     * A one-group study's share against the share its null expects.
     *
     * @param observed the share observed
     * @param expected the share the null expects
     * @param ratio observed over expected; null when nothing is expected
     */
    public record Expectation(double observed, double expected, Double ratio) {
    }

    /**
     * Which methods put a predicate at or under the caller's alpha.
     *
     * @param raw the raw p-value
     * @param maxT max-T
     * @param holm Holm
     * @param bonferroni Bonferroni
     * @param bh Benjamini–Hochberg
     * @param by Benjamini–Yekutieli
     */
    public record UnderAlpha(boolean raw, boolean maxT, boolean holm, boolean bonferroni, boolean bh,
            boolean by) {
    }

    /**
     * One predicate's row of a test.
     *
     * @param predicate the rule's key
     * @param counts its charts in each group
     * @param observed the statistic under the observed labels; null when it is unbounded, a recombined
     *     sample beyond replicates that all agree
     * @param p the permutation p-value
     * @param exact the hypergeometric p of the same statistic; null where it does not apply
     * @param adjusted the family's adjusted p-values
     * @param effect the cases against the rest; null where it does not apply
     * @param expected the share against its null's; null where it does not apply
     * @param underAlpha which methods put it under the caller's alpha; null when none was given
     */
    public record Row(String predicate, List<ResearchCounts.Group> counts, Double observed, PValue p,
            Double exact, Adjusted adjusted, Effect effect, Expectation expected, UnderAlpha underAlpha) {
    }
}
