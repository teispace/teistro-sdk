package com.teispace.teistro;

import java.util.Map;

/**
 * A letter a name may begin with to take a pada's syllable.
 *
 * @param devanagari the letter, as Devanagari
 * @param iast its IAST
 * @param by how it reaches the cell
 */
public record NameInitial(String devanagari, String iast, InitialBy by) {
    static NameInitial read(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        return new NameInitial(Reads.string(o, "devanagari"), Reads.string(o, "iast"), InitialBy.of(Reads.field(o, "by")));
    }
}
