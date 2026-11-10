package com.teispace.teistro;

import com.teispace.teistro.record.Provenance;

/**
 * What {@link AlmanacArea#sankalpaWords} answers.
 *
 * @param value the words
 * @param provenance what computed the time, and the hash of {@code value}
 */
public record SankalpaWordsAnswer(SankalpaWords value, Provenance provenance) {}
