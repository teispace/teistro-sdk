package com.teispace.teistro;

import java.util.List;

/**
 * A chart's Brahma graha, and how it was found (BPHS ch. 46 vv. 170 to 173).
 *
 * @param rule The rule it was sought under, {@code jaimini.brahma}.
 * @param countedFrom The stronger of the lagna and the 7th, which the rule counts from.
 * @param qualified The planets that met the rule's marks, in id order.
 * @param graha The Brahma graha; null where the rule finds none. May be null.
 * @param passedFrom Saturn or the node that passed Brahma-hood to the planet in the 6th from it
 *     (C127). May be null.
 * @param none Why there is none; null when there is one, and never {@code FOUND}. May be null.
 */
public record Brahma(
        BrahmaRule rule,
        Rashi countedFrom,
        List<Graha> qualified,
        Graha graha,
        Graha passedFrom,
        BrahmaOutcome none) {
    /** The value, its lists copied and unmodifiable. */
    public Brahma {
        qualified = List.copyOf(qualified);
    }
}
