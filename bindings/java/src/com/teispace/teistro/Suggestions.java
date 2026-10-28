package com.teispace.teistro;

import java.util.List;
import java.util.Map;

/**
 * The names a book suggests for a birth.
 *
 * @param book the book asked
 * @param prescribed the birth pada's syllable, with its initials
 * @param names the names, in cakra order and then the book's
 * @param refused every name the book holds that the cakra refused, listed, never dropped
 */
public record Suggestions(String book, PadaSyllable prescribed, List<Suggested> names, List<RefusedName> refused) {
    static Suggestions read(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        return new Suggestions(Reads.string(o, "book"), PadaSyllable.read(Reads.field(o, "prescribed")),
                Reads.each(o, "names", Suggested::read), Reads.each(o, "refused", RefusedName::read));
    }
}
