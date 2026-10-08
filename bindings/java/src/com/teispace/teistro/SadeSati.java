package com.teispace.teistro;

import java.util.List;

/**
 * One Sade Sati: the rising (12th), peak (1st) and setting (2nd) spells, in order.
 *
 * @param phases The spells, in order.
 */
public record SadeSati(
        List<SadeSatiSpell> phases) {
    /** The value, its lists copied and unmodifiable. */
    public SadeSati {
        phases = List.copyOf(phases);
    }
}
