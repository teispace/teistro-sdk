package com.teispace.teistro;

import java.util.List;

import com.teispace.teistro.record.Provenance;

/**
 * Why each published day differs from the rule's
 * ({@code 03-design/published-overrides.md} §4), from
 * {@link AlmanacArea#publishedReasons}: one reason per occurrence a {@code SET} moved.
 *
 * @param reasons each published day explained
 * @param provenance the answer's provenance
 */
public record PublishedReasons(List<Reason> reasons, Provenance provenance) {
    /** Keeps the list unmodifiable. */
    public PublishedReasons {
        reasons = List.copyOf(reasons);
    }

    /**
     * A place a publisher may have reckoned its days at.
     *
     * @param name how the reason names it
     * @param place where
     * @param utcOffsetSeconds its civil days' offset from UTC, east positive
     */
    public record Place(String name, Observer place, int utcOffsetSeconds) {}

    /**
     * A change tried, or the reason a day differs: {@code by} is {@code PLACE} with its {@code name},
     * {@code SKY} with its {@code profile} and {@code settingsHash}, {@code RULE} with its {@code pack},
     * or {@code PUBLISHER}, the publisher's own choice, when no change gives the day. A field the kind
     * does not carry is null.
     *
     * @param by the kind of change
     * @param name the place's name
     * @param profile the sky's profile
     * @param settingsHash the sky's settings hash
     * @param pack the pack whose rule was tried
     */
    public record Cause(String by, String name, String profile, String settingsHash, String pack) {}

    /**
     * One change tried.
     *
     * @param change the change
     * @param day the rule's day under it, or null when it gives none within seven days
     * @param reproduces whether that is the published day
     */
    public record Tried(Cause change, CalendarDate day, boolean reproduces) {}

    /**
     * One published day explained.
     *
     * @param rule the rule's key
     * @param set the set that published it
     * @param day the published day
     * @param computed the rule's own day here
     * @param reason the first change that gives the day, else {@code PUBLISHER}
     * @param tried every change tried, in order
     */
    public record Reason(String rule, String set, CalendarDate day, CalendarDate computed, Cause reason,
            List<Tried> tried) {
        /** Keeps the list unmodifiable. */
        public Reason {
            tried = List.copyOf(tried);
        }
    }
}
