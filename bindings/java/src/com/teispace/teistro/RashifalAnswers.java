package com.teispace.teistro;

import java.util.List;

import com.teispace.teistro.record.Provenance;

/**
 * Many periods' answers, in the requests' order, under one provenance.
 *
 * @param value the answers
 * @param provenance what computed them, and under what; {@code inputHash} seals the request
 */
public record RashifalAnswers(List<RashifalAnswer> value, Provenance provenance) {
    /** Keeps the list unmodifiable. */
    public RashifalAnswers {
        value = List.copyOf(value);
    }
}
