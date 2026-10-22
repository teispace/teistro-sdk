package com.teispace.teistro;

import com.teispace.teistro.record.Provenance;

/**
 * What {@link AlmanacArea#firstYear} answers.
 *
 * @param value the schedule
 * @param provenance what computed it, and the hash of {@code value}
 */
public record FirstYearAnswer(FirstYearSchedule value, Provenance provenance) {}
