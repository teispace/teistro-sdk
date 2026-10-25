package com.teispace.teistro;

import java.util.List;

/**
 * A festival day's sait ({@code 03-design/festival-sait.md}): the window it
 * was found in, every candidate there, and the one chosen.
 *
 * @param window which of the rule's windows gave it, from 0; the last when
 *     none did
 * @param within that window on the day
 * @param candidates every candidate in it, in order
 * @param chosen the chosen candidate's index; null when the window holds
 *     none
 * @param at the chosen candidate's middle, a UTC Julian day: the instant an
 *     almanac prints; null when none was chosen
 */
public record FestivalSait(int window, Interval within, List<Candidate> candidates, Integer chosen, Double at) {
    /** Keeps the candidates unmodifiable. */
    public FestivalSait {
        candidates = List.copyOf(candidates);
    }

    /**
     * A stretch a sait may be in.
     *
     * @param span when
     * @param whole whether its navamsa lies wholly inside the free window;
     *     always for a rule reading no rising sign
     * @param lagna the lagna's sign, when the rule reads a rising sign;
     *     else null
     * @param navamsa the navamsa's sign, when the rule reads a rising sign;
     *     else null
     */
    public record Candidate(Interval span, boolean whole, Rashi lagna, Rashi navamsa) {}
}
