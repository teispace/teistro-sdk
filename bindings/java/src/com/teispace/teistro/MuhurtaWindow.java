package com.teispace.teistro;

import java.util.List;

/**
 * A window judged: when, by which clauses, what barred it, and the
 * baseline's score under that ranking.
 *
 * @param at when it runs
 * @param clauses the clauses that held in it
 * @param barredBy the bars that struck it (a clause's tag, which bars every
 *     clause of that kind, or one clause); empty when the rite may be held in it
 * @param score the baseline's score; may be null, under the texts' ranking
 */
public record MuhurtaWindow(Interval at, List<MuhurtaClause> clauses, List<MuhurtaBar> barredBy, MuhurtaScore score) {
    /** Keeps the lists unmodifiable. */
    public MuhurtaWindow {
        clauses = List.copyOf(clauses);
        barredBy = List.copyOf(barredBy);
    }
}
