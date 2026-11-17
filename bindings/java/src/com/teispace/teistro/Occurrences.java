package com.teispace.teistro;

import java.util.List;

import com.teispace.teistro.record.Provenance;

/**
 * Every day a rule or a kind of day falls on, year by year, as {@link AlmanacArea#occurrences}
 * answers it ({@code 03-design/rule-years.md}).
 *
 * <pre>{@code
 * Occurrences span = sdk.almanac().occurrences(asked);
 * for (OccurrenceYear year : span.years()) {
 *     System.out.println(year.year() + ": " + year.days().size());
 * }
 * }</pre>
 *
 * @param keys the keys looked for: the rule's, or each of a kind's rules
 * @param years each year of the span, in order
 * @param provenance what computed it
 */
public record Occurrences(List<String> keys, List<OccurrenceYear> years, Provenance provenance) {
    /** Copies the lists. */
    public Occurrences {
        keys = List.copyOf(keys);
        years = List.copyOf(years);
    }
}
