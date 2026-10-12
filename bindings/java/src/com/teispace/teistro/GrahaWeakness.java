package com.teispace.teistro;

import java.util.List;

/**
 * The clauses of the Samjna Tantra v. 72 that hold for one graha: a graha "unable to bring about
 * the matter", none weighed.
 *
 * @param graha Whose.
 * @param clauses {@code UNSEEN_LAGNA}, {@code SETTING_PART}, {@code OFFICELESS},
 *     {@code MALEFIC_ISHRAFA}, in the verse's order.
 */
public record GrahaWeakness(
        Graha graha,
        List<String> clauses) {
    /** The value, its lists copied and unmodifiable. */
    public GrahaWeakness {
        clauses = List.copyOf(clauses);
    }
}
