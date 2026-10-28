package com.teispace.teistro;

import java.util.List;
import java.util.Map;

import com.teispace.teistro.request.FinalA;

/**
 * A name's syllables and the clauses of a name in daily use (C774).
 *
 * @param syllables its syllables, counted by their vowels
 * @param counted how the final inherent a was counted
 * @param rows the rows of the rules' sutra
 */
public record NameClauses(int syllables, FinalA counted, List<NameClauseRow> rows) {
    static NameClauses read(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        return new NameClauses(Reads.integer(o, "syllables"), FinalA.of(Reads.field(o, "counted")),
                Reads.each(o, "rows", NameClauseRow::read));
    }
}
