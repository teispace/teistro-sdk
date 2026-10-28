package com.teispace.teistro;

import java.util.List;
import java.util.Map;

/**
 * A name read: one entry per first syllable its spelling reaches.
 *
 * @param readings each distinct first syllable, in cakra order
 * @param ambiguous whether there is more than one
 */
public record NameReading(List<SpelledName> readings, boolean ambiguous) {
    static NameReading read(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        return new NameReading(Reads.each(o, "readings", SpelledName::read), Reads.flag(o, "ambiguous"));
    }
}
