package com.teispace.teistro;

import java.util.Map;

/**
 * A name read against a birth's pada: facts, not a score (C334, C770).
 *
 * @param read the name's first syllable
 * @param prescribed the birth pada's syllable, with every initial that reaches it
 * @param sameStar whether the name begins in the birth star's row
 * @param samePada whether it begins in the birth pada's own cell
 * @param bySubstitute whether it begins with the pada's substitute (C770)
 * @param clauses its syllables and clauses
 */
public record NameCheck(NameSyllable read, PadaSyllable prescribed, boolean sameStar, boolean samePada,
        boolean bySubstitute, NameClauses clauses) {
    static NameCheck read(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        return new NameCheck(Naming.syllable(Reads.field(o, "read")), PadaSyllable.read(Reads.field(o, "prescribed")),
                Reads.flag(o, "sameStar"), Reads.flag(o, "samePada"), Reads.flag(o, "bySubstitute"),
                NameClauses.read(Reads.field(o, "clauses")));
    }
}
