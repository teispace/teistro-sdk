package com.teispace.teistro;

import com.teispace.teistro.record.Provenance;

/**
 * What {@link AlmanacArea#sankalpa} answers.
 *
 * @param value the time recited
 * @param provenance what computed it, and the hash of {@code value}
 */
public record SankalpaAnswer(Sankalpa value, Provenance provenance) {}
