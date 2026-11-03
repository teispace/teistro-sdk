package com.teispace.teistro;

import com.teispace.teistro.record.Provenance;

/**
 * What {@link AlmanacArea#skyView} answers.
 *
 * @param value the sky
 * @param provenance what computed it, and the hash of {@code value}
 */
public record SkyViewAnswer(SkyView value, Provenance provenance) {}
