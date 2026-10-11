package com.teispace.teistro;

import java.math.BigInteger;
import java.util.Objects;

/**
 * A permutation test of a design's groups ({@code 03-design/research.md}): a seed, 1 to 10 000 000
 * permutations fixed in the request, and what is compared. Every interval is at {@code level};
 * {@code alpha}, when given, asks which methods put each row under it; {@code threads} count the
 * permutations, the same bits for each.
 *
 * <pre>{@code
 * ResearchGroupTest test = ResearchGroupTest.builder(2026, 999, ResearchContrast.caseVsRest(1))
 *         .alpha(0.05)
 *         .build();
 * }</pre>
 *
 * @param seed the seed, 0 to 2<sup>64</sup> − 1, the same draws in every binding
 * @param permutations how many relabellings, 1 to 10 000 000
 * @param contrast what the groups are compared by
 * @param alternative the direction looked in
 * @param level every interval's confidence level, above 0 and under 1
 * @param alpha the level each method's verdict is read at; none when null
 * @param threads how many threads count the permutations; the SDK's choice when null
 */
public record ResearchGroupTest(BigInteger seed, int permutations, ResearchContrast contrast,
        ResearchAlternative alternative, double level, Double alpha, Integer threads) {
    /** The value, its seed, contrast and alternative required. */
    public ResearchGroupTest {
        Objects.requireNonNull(seed, "seed");
        Objects.requireNonNull(contrast, "contrast");
        Objects.requireNonNull(alternative, "alternative");
    }

    /**
     * A builder with the required fields, the rest at their defaults: two-sided, at 0.95.
     *
     * @param seed the seed
     * @param permutations how many relabellings
     * @param contrast what the groups are compared by
     * @return the builder
     */
    public static Builder builder(long seed, int permutations, ResearchContrast contrast) {
        return builder(BigInteger.valueOf(seed), permutations, contrast);
    }

    /**
     * A builder with the required fields, a seed past {@code Long.MAX_VALUE} as a {@code BigInteger}.
     *
     * @param seed the seed
     * @param permutations how many relabellings
     * @param contrast what the groups are compared by
     * @return the builder
     */
    public static Builder builder(BigInteger seed, int permutations, ResearchContrast contrast) {
        return new Builder(seed, permutations, contrast);
    }

    /** Builds {@link ResearchGroupTest} a field at a time. */
    public static final class Builder {
        private final BigInteger seed;
        private final int permutations;
        private final ResearchContrast contrast;
        private ResearchAlternative alternative = ResearchAlternative.TWO_SIDED;
        private double level = 0.95;
        private Double alpha;
        private Integer threads;

        private Builder(BigInteger seed, int permutations, ResearchContrast contrast) {
            this.seed = seed;
            this.permutations = permutations;
            this.contrast = contrast;
        }

        /**
         * Sets {@code alternative}.
         *
         * @param alternative the direction looked in
         * @return this builder
         */
        public Builder alternative(ResearchAlternative alternative) {
            this.alternative = alternative;
            return this;
        }

        /**
         * Sets {@code level}.
         *
         * @param level every interval's confidence level
         * @return this builder
         */
        public Builder level(double level) {
            this.level = level;
            return this;
        }

        /**
         * Sets {@code alpha}.
         *
         * @param alpha the level each method's verdict is read at
         * @return this builder
         */
        public Builder alpha(double alpha) {
            this.alpha = alpha;
            return this;
        }

        /**
         * Sets {@code threads}.
         *
         * @param threads how many threads count the permutations
         * @return this builder
         */
        public Builder threads(int threads) {
            this.threads = threads;
            return this;
        }

        /**
         * The test.
         *
         * @return the test
         */
        public ResearchGroupTest build() {
            return new ResearchGroupTest(seed, permutations, contrast, alternative, level, alpha, threads);
        }
    }
}
