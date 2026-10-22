package com.teispace.teistro;

import java.util.List;

/**
 * A mahadvadashi of <i>Hari Bhakti Vilasa</i> 13.265 that held, and the fast it
 * set ({@code 03-design/festival-rules.md} §8.9). An Ekadashi rule keeps them as
 * {@code mahadvadashi: {kinds, trisprisha, joining, vyanjuli, sampurna}}, each
 * field defaulting to the text's reading.
 *
 * @param kind the kind that set the fast, the first of the rule's kinds that
 *     held: {@code UNMILANI}, {@code VYANJULI}, {@code TRISPRISHA},
 *     {@code PAKSHAVARDHINI}, {@code JAYA}, {@code VIJAYA}, {@code JAYANTI} or
 *     {@code PAPANASHINI}
 * @param held every kind that held, in the rule's order
 * @param clause the verse that defines {@code kind}
 * @param joinedNakshatra the nakshatra a bright 12th is joined with, for Jaya,
 *     Vijaya, Jayanti and Papanashini; null otherwise
 * @param joinedSpan its whole stay; null otherwise
 */
public record Mahadvadashi(
        String kind, List<String> held, String clause, Nakshatra joinedNakshatra, Interval joinedSpan) {
    /** Keeps the list unmodifiable. */
    public Mahadvadashi {
        held = List.copyOf(held);
    }
}
