package com.teispace.teistro;

import java.util.Map;

/**
 * One cell a name's spelling reaches.
 *
 * @param spelled the Devanagari it was read as
 * @param syllable its first syllable in the cakra
 * @param rashi the sign that syllable's pada stands in
 * @param clauses its syllables and clauses
 */
public record SpelledName(String spelled, NameSyllable syllable, Rashi rashi, NameClauses clauses) {
    static SpelledName read(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        return new SpelledName(Reads.string(o, "spelled"), Naming.syllable(Reads.field(o, "syllable")),
                Naming.rashi(Reads.field(o, "rashi")), NameClauses.read(Reads.field(o, "clauses")));
    }
}
