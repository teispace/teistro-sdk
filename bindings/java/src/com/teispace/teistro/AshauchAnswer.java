package com.teispace.teistro;

import com.teispace.teistro.record.Provenance;

/**
 * What {@link AlmanacArea#ashauch} answers.
 *
 * @param value the period
 * @param provenance what computed it, and the hash of {@code value}
 */
public record AshauchAnswer(Ashauch value, Provenance provenance) {}
