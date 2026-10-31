package com.teispace.teistro;

import com.teispace.teistro.record.Provenance;

/**
 * What {@link AlmanacArea#deathTime} answers.
 *
 * @param value the readings and the rites
 * @param provenance what computed it, and the hash of {@code value}
 */
public record DeathTimeAnswer(DeathTime value, Provenance provenance) {}
