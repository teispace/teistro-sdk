package com.teispace.teistro;

import java.util.Map;

/**
 * One name of a batch checked: its check, or its refusal.
 *
 * @param check the name checked; may be null, when it was refused
 * @param refused the refusal, named {@code naming.names[i]}; may be null, when it was checked
 */
public record NameChecked(NameCheck check, NameRefusal refused) {
    static NameChecked read(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        Object check = Reads.field(o, "check");
        return new NameChecked(check == null ? null : NameCheck.read(check), NameRefusal.read(Reads.field(o, "refused")));
    }
}
