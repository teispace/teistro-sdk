package com.teispace.teistro;

import java.util.List;

/**
 * A chart's Bhava bala, read under the context's {@code strength.bhava_*} settings
 * ({@code 03-design/bhava-bala-measured.md}).
 *
 * @param bhavas Each bhava's, the first to the twelfth.
 */
public record BhavaBala(
        List<BhavaStrength> bhavas) {
    /** The value, its lists copied and unmodifiable. */
    public BhavaBala {
        bhavas = List.copyOf(bhavas);
    }
}
