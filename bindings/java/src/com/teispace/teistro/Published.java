package com.teispace.teistro;

import java.util.Map;

/**
 * What a published entry did to a day ({@code 03-design/published-overrides.md}):
 * null on a day no entry touched, which is the rule's.
 *
 * @param status {@code PUBLISHED} or {@code WITHDRAWN}
 * @param publisher the set's publisher
 * @param set the set's id
 * @param version its version
 * @param source where the publisher says it
 * @param computed the rule's own day, when the publisher's differs; null otherwise
 * @param window a published time, or null
 */
public record Published(String status, String publisher, String set, long version, String source,
        CalendarDate computed, Interval window) {
    /**
     * A day a set added, which no rule computes.
     *
     * @param key its key
     * @param day the day
     * @param names its names, by locale
     * @param published who published it, and where
     */
    public record Added(String key, CalendarDate day, Map<String, String> names, Published published) {
        /** Keeps the names unmodifiable. */
        public Added {
            names = Map.copyOf(names);
        }
    }

    /**
     * An entry inside the range that no occurrence took.
     *
     * @param set the set's id
     * @param entry the entry's index in it
     * @param rule the rule it names
     * @param day the day it names
     */
    public record Unplaced(String set, long entry, String rule, CalendarDate day) {}

    /**
     * A set laid over the answer.
     *
     * @param id its id
     * @param version its version
     * @param publisher its publisher
     * @param hash the SHA-256 of its canonical JSON, in hex
     */
    public record Applied(String id, long version, String publisher, String hash) {}
}
