package com.teispace.teistro;

import java.util.List;

/**
 * One of the sixteen holding, with what made it hold.
 *
 * @param yoga Which of the sixteen.
 * @param between The lords' own relation, where that is what made it. May be null.
 * @param through The third planet it turns on, where one does. May be null.
 * @param entering The planet judged on entering the next sign: Gairi-Kamboola's Moon, Tambira's
 *     lord. May be null.
 * @param legs How the third planet stands to each of the pair, two of them, read from the next
 *     sign for {@code entering}. May be null.
 * @param afflictions The lords' afflictions, where those made it: Rudda and Durapha. May be null.
 */
public record HeldYearYoga(
        YearYoga yoga,
        TajikaBetween between,
        Graha through,
        Graha entering,
        List<TajikaBetween> legs,
        Afflictions afflictions) {
    /** The value, its lists copied and unmodifiable. */
    public HeldYearYoga {
        legs = legs == null ? null : List.copyOf(legs);
    }
}
