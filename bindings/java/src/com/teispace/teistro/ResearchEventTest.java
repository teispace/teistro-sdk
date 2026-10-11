package com.teispace.teistro;

import java.math.BigInteger;
import java.util.Objects;

/**
 * A test of events against the shuffled-event null ({@code 03-design/research.md} §1.4). With
 * {@code restrictPairings}, a permutation draws only pairings that keep every event after its birth;
 * without, a study in which any would not is refused.
 *
 * <pre>{@code
 * ResearchEventTest test = ResearchEventTest.builder(3, 49).restrictPairings(true).build();
 * }</pre>
 *
 * @param seed the seed, 0 to 2<sup>64</sup> − 1, the same draws in every binding
 * @param permutations how many shuffles
 * @param alternative the direction looked in
 * @param restrictPairings whether a shuffle draws only pairings that keep every event after its birth
 * @param level every interval's confidence level, above 0 and under 1
 * @param alpha the level each method's verdict is read at; none when null
 * @param threads how many threads count the permutations; the SDK's choice when null
 */
public record ResearchEventTest(BigInteger seed, int permutations, ResearchAlternative alternative,
        boolean restrictPairings, double level, Double alpha, Integer threads) {
    /** The value, its seed and alternative required. */
    public ResearchEventTest {
        Objects.requireNonNull(seed, "seed");
        Objects.requireNonNull(alternative, "alternative");
    }

    /**
     * A builder with the required fields, the rest at their defaults: two-sided, refusing an event
     * before its birth, at 0.95.
     *
     * @param seed the seed
     * @param permutations how many shuffles
     * @return the builder
     */
    public static Builder builder(long seed, int permutations) {
        return builder(BigInteger.valueOf(seed), permutations);
    }

    /**
     * A builder with the required fields, a seed past {@code Long.MAX_VALUE} as a {@code BigInteger}.
     *
     * @param seed the seed
     * @param permutations how many shuffles
     * @return the builder
     */
    public static Builder builder(BigInteger seed, int permutations) {
        return new Builder(seed, permutations);
    }

    /** Builds {@link ResearchEventTest} a field at a time. */
    public static final class Builder {
        private final BigInteger seed;
        private final int permutations;
        private ResearchAlternative alternative = ResearchAlternative.TWO_SIDED;
        private boolean restrictPairings;
        private double level = 0.95;
        private Double alpha;
        private Integer threads;

        private Builder(BigInteger seed, int permutations) {
            this.seed = seed;
            this.permutations = permutations;
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
         * Sets {@code restrictPairings}.
         *
         * @param restrict whether a shuffle draws only pairings that keep every event after its birth
         * @return this builder
         */
        public Builder restrictPairings(boolean restrict) {
            this.restrictPairings = restrict;
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
        public ResearchEventTest build() {
            return new ResearchEventTest(seed, permutations, alternative, restrictPairings, level, alpha, threads);
        }
    }
}
