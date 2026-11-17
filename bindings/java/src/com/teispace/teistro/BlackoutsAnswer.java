package com.teispace.teistro;

import com.teispace.teistro.record.Provenance;
import java.util.List;

/**
 * What {@link AlmanacArea#blackouts} answers.
 *
 * @param value the blackouts, in order of their start
 * @param provenance what computed it, the range's bounds and the zodiac's instant among the applied conventions,
 *     and the hash of {@code value}
 */
public record BlackoutsAnswer(List<Blackout> value, Provenance provenance) {}
