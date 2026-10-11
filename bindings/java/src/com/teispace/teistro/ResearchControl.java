package com.teispace.teistro;

import java.math.BigInteger;
import java.util.List;
import java.util.Objects;

/**
 * Gauquelin's control ({@code 03-design/research.md}): the sample refounded {@code replicates} times
 * (1 to 100 000) from {@code seed}, clock times moving only inside {@code strata} when given.
 *
 * <pre>{@code
 * ResearchControl control = new ResearchControl(4, 3);
 * }</pre>
 *
 * @param seed the seed, 0 to 2<sup>64</sup> − 1, the same draws in every binding
 * @param replicates how many recombined samples, 1 to 100 000
 * @param strata each birth's stratum, inside which clock times move; every birth one when null
 */
public record ResearchControl(BigInteger seed, int replicates, List<Long> strata) {
    /** The value, its seed required and its list copied. */
    public ResearchControl {
        Objects.requireNonNull(seed, "seed");
        strata = strata == null ? null : List.copyOf(strata);
    }

    /**
     * A control without strata.
     *
     * @param seed the seed
     * @param replicates how many recombined samples
     */
    public ResearchControl(long seed, int replicates) {
        this(BigInteger.valueOf(seed), replicates, null);
    }
}
