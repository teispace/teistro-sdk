package com.teispace.teistro;

import java.util.List;

/**
 * When the matter comes to pass, and the rule and graha that timed it.
 *
 * @param rule The rule that timed it.
 * @param graha The graha that timed it.
 * @param tie Whether that graha won a tie in strength (C338).
 * @param count The count the rule read.
 * @param multiplier What the count is multiplied by.
 * @param amount The time; null where the rule gives none. May be null.
 * @param unit The time's unit.
 * @param between The grahas between the lagna and the Moon, under {@code MOON_DAYS}.
 */
public record PrashnaTiming(
        String rule,
        Graha graha,
        boolean tie,
        int count,
        int multiplier,
        Integer amount,
        String unit,
        List<Graha> between) {
    /** The value, its lists copied and unmodifiable. */
    public PrashnaTiming {
        between = List.copyOf(between);
    }
}
