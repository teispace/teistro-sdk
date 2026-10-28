package com.teispace.teistro;

import java.util.Map;

import com.teispace.teistro.request.NameClause;

/**
 * One clause of a name in daily use, as one sutra gives it (C774).
 *
 * @param clause the clause
 * @param holds whether the name meets it
 * @param source whose text gives it
 */
public record NameClauseRow(NameClause clause, boolean holds, ClauseSource source) {
    static NameClauseRow read(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        return new NameClauseRow(NameClause.of(Reads.field(o, "clause")), Reads.flag(o, "holds"),
                ClauseSource.of(Reads.field(o, "source")));
    }
}
