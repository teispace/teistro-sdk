package com.teispace.teistro;

import com.teispace.teistro.record.Provenance;

/**
 * What {@link AlmanacArea#household} answers.
 *
 * @param value each observer's periods
 * @param provenance what computed it, and the hash of {@code value}
 */
public record HouseholdAnswer(HouseholdAshauch value, Provenance provenance) {}
