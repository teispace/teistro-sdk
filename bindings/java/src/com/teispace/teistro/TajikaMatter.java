package com.teispace.teistro;

import java.util.List;
import java.util.Optional;

/**
 * The sixteen Tajika yogas for one matter of a year: the question it asked as well as the answer,
 * because a list of yogas whose pair a reader cannot see is not checkable.
 *
 * @param house The house asked about, 1 to 12, counted from the annual lagna.
 * @param sign The sign that house falls in.
 * @param lagnesha The lord of the annual lagna.
 * @param karyesha The lord of the house asked about.
 * @param sameLord One planet is both, always so of the first house, so there is no pair to judge.
 * @param between How the two lords stand to each other; null when they are one. May be null.
 * @param held Every yoga that holds, once for each third planet that makes it.
 * @param unanswered The yogas this call could not answer for. A yoga absent from {@code held} did
 *     not hold <b>only</b> if it is not listed here.
 */
public record TajikaMatter(
        int house,
        Rashi sign,
        Graha lagnesha,
        Graha karyesha,
        boolean sameLord,
        TajikaBetween between,
        List<HeldYearYoga> held,
        List<YearYoga> unanswered) {
    /** The value, its lists copied and unmodifiable. */
    public TajikaMatter {
        held = List.copyOf(held);
        unanswered = List.copyOf(unanswered);
    }

    /**
     * Whether {@code yoga} holds: empty where this call could not say, which
     * is not the same answer as false.
     *
     * @param yoga the yoga
     * @return whether it holds, if this call could say
     */
    public Optional<Boolean> holds(YearYoga yoga) {
        if (unanswered.contains(yoga)) {
            return Optional.empty();
        }
        return Optional.of(held.stream().anyMatch(one -> one.yoga() == yoga));
    }
}
