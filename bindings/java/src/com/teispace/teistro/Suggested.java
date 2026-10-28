package com.teispace.teistro;

import java.util.Map;

/**
 * A name a book suggests for a birth.
 *
 * @param index its place in the book
 * @param reading the name read, with its clauses under the request's rules
 * @param by how it reaches the birth pada
 * @param samePada whether it begins in the pada's own cell
 */
public record Suggested(int index, SpelledName reading, InitialBy by, boolean samePada) {
    static Suggested read(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        return new Suggested(Reads.integer(o, "index"), SpelledName.read(Reads.field(o, "reading")),
                InitialBy.of(Reads.field(o, "by")), Reads.flag(o, "samePada"));
    }
}
