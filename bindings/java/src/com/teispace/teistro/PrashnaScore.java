package com.teispace.teistro;

import java.util.List;

/**
 * The baseline engine's points for a query chart. Unsourced (C337).
 *
 * @param points The points.
 * @param answer {@code YES} over 1, {@code NO} under -1, {@code UNCERTAIN} between.
 * @param factors What gave them.
 * @param isVoid The baseline's void Moon, over grahas held still.
 * @param applyingTo The graha nearest ahead of the Moon by conjunction distance. May be null.
 */
public record PrashnaScore(
        int points,
        String answer,
        List<PrashnaFactor> factors,
        boolean isVoid,
        Graha applyingTo) {
    /** The value, its lists copied and unmodifiable. */
    public PrashnaScore {
        factors = List.copyOf(factors);
    }
}
