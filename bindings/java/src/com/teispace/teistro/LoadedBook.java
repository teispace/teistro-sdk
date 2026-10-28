package com.teispace.teistro;

import java.util.List;
import java.util.Map;

/**
 * A name book loaded into a context.
 *
 * @param book its id, the SHA-256 of its canonical JSON, which a suggestion names it by
 * @param names how many names it holds
 * @param refused every name it holds that the cakra refused, in the book's order
 */
public record LoadedBook(String book, int names, List<RefusedName> refused) {
    static LoadedBook read(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        return new LoadedBook(Reads.string(o, "book"), Reads.integer(o, "names"), Reads.each(o, "refused", RefusedName::read));
    }
}
