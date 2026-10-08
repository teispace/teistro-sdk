package com.teispace.teistro;

import java.util.List;

/**
 * Whether a horary matter is brought to pass: the relations between two
 * significators with the facts each rests on, never a verdict
 * ({@code 03-design/hellenistic-perfection.md}).
 *
 * <pre>{@code
 * boolean perfects = !chart.perfection().orElseThrow().ways().held().isEmpty();
 * }</pre>
 *
 * @param querent the querent's significator
 * @param quesited the quesited's significator
 * @param application the application between them; may be null
 * @param separation the separation between them; may be null
 * @param impediments what stops or hinders the application
 * @param translations the translations of light between them
 * @param collections the collections of their light
 * @param ways the ways of perfection
 * @param horizonDays how many days ahead it was read
 * @param rules the rules it was read under
 */
public record Matter(
        Graha querent, Graha quesited, Application application, Separation separation,
        List<Impediment> impediments, List<Translation> translations, List<Collection> collections, Ways ways,
        double horizonDays, PerfectionRules rules) {
    /** Keeps the lists unmodifiable. */
    public Matter {
        impediments = List.copyOf(impediments);
        translations = List.copyOf(translations);
        collections = List.copyOf(collections);
    }
}
