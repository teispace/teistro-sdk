package com.teispace.teistro;

import java.util.Map;

import com.teispace.teistro.record.Provenance;

/**
 * What {@code almanac.elect}, {@code almanac.observance} and {@code almanac.people} answer: the record
 * as the wire spells it, catalogue members as bare keys, and its provenance.
 *
 * @param value the record
 * @param provenance what computed it
 */
public record FinderAnswer(Map<?, ?> value, Provenance provenance) {

    /**
     * An answer read from the envelope the library wrote.
     *
     * @param json the envelope, {@code {value, provenance}}
     * @return the answer
     */
    static FinderAnswer of(String json) {
        Map<?, ?> envelope = Reads.object(Json.read(json));
        return new FinderAnswer(Reads.object(envelope, "value"), Provenance.of(Reads.field(envelope, "provenance")));
    }
}
