package com.teispace.teistro;

import java.util.Map;

/**
 * A clause a vastu text judges a measure by, and whether it holds, the good
 * reading ({@code 03-design/aayadi.md}).
 *
 * @param clause the condition
 * @param holds whether it holds
 * @param source the verse or gloss that judges it
 */
public record AayadiClause(AayadiClauseKey clause, boolean holds, String source) {

    static AayadiClause read(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        return new AayadiClause(
                AayadiClauseKey.read(Reads.field(o, "clause")), Reads.flag(o, "holds"), Reads.string(o, "source"));
    }
}
