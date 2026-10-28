package com.teispace.teistro;

import java.util.Map;

/**
 * Why a name of a batch, or of a book, was refused: the SDK's error as a
 * record.
 *
 * @param status its status's key, such as {@code INVALID_ARG}
 * @param detail its detail's key; may be null
 * @param message what was refused
 * @param field the field it names, such as {@code naming.names[2]}; may be null
 * @param hint what to do instead; may be null
 */
public record NameRefusal(String status, String detail, String message, String field, String hint) {
    static NameRefusal read(Object raw) {
        if (raw == null) {
            return null;
        }
        Map<?, ?> o = Reads.object(raw);
        return new NameRefusal(Reads.string(o, "status"), Reads.optionalString(o, "detail"), Reads.string(o, "message"),
                Reads.optionalString(o, "field"), Reads.optionalString(o, "hint"));
    }
}
