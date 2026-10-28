package com.teispace.teistro;

import java.util.Map;

/**
 * A name a book holds that the cakra refused.
 *
 * @param index its place in the book
 * @param error why, named {@code book.names[index].devanagari}
 */
public record RefusedName(int index, NameRefusal error) {
    static RefusedName read(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        return new RefusedName(Reads.integer(o, "index"), NameRefusal.read(Reads.field(o, "error")));
    }
}
