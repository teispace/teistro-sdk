package com.teispace.teistro;

import java.util.List;

/**
 * Whether the matter succeeds: every clause that holds, and <i>Shatpanchashika</i> I.4's outcome
 * over them ({@code SUCCEEDS}, {@code WITH_DIFFICULTY} or {@code FAILS}), never a score (C337).
 *
 * @param clauses Every clause that holds.
 * @param outcome The outcome.
 */
public record PrashnaVerdict(
        List<PrashnaClause> clauses,
        String outcome) {
    /** The value, its lists copied and unmodifiable. */
    public PrashnaVerdict {
        clauses = List.copyOf(clauses);
    }
}
