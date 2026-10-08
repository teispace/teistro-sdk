package com.teispace.teistro;

import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * A chart's remedies ({@code 03-design/remedies.md}).
 *
 * @param rules The readings it was given under, every member filled, as the library wrote them.
 * @param functional What the lagna's lordships make of the seven.
 * @param subjects Whom a remedy is for.
 * @param shantis The graha-shanti of each subject, in their order.
 * @param ishtaDevata The ishta-devata.
 */
public record Remedies(
        Map<String, Object> rules,
        Functional functional,
        RemedySubjects subjects,
        List<Shanti> shantis,
        IshtaDevatas ishtaDevata) {
    /** The value, its lists copied and unmodifiable. */
    public Remedies {
        rules = Collections.unmodifiableMap(new LinkedHashMap<>(rules));
        shantis = List.copyOf(shantis);
    }
}
