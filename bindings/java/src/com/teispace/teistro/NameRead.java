package com.teispace.teistro;

import java.util.Map;

/**
 * One name of a batch read: its reading, or its refusal.
 *
 * @param reading the name read; may be null, when it was refused
 * @param refused the refusal, named {@code naming.names[i]}; may be null, when it was read
 */
public record NameRead(NameReading reading, NameRefusal refused) {
    static NameRead read(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        Object reading = Reads.field(o, "reading");
        return new NameRead(reading == null ? null : NameReading.read(reading), NameRefusal.read(Reads.field(o, "refused")));
    }
}
