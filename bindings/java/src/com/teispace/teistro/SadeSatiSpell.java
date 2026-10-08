package com.teispace.teistro;

import java.util.List;

/**
 * Every stay of Saturn's in one house of one period, a retrograde re-entry a visit of its own
 * (C148).
 *
 * @param house The house from the reference, 1 to 12.
 * @param visits Each stay, in time order.
 */
public record SadeSatiSpell(
        int house,
        List<SadeSatiVisit> visits) {
    /** The value, its lists copied and unmodifiable. */
    public SadeSatiSpell {
        visits = List.copyOf(visits);
    }
}
