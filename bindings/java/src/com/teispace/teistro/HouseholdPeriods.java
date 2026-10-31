package com.teispace.teistro;

import java.util.List;

/**
 * One observer's periods in a household ({@code 03-design/ashauch.md} §11), in the order the news
 * reached the observer.
 *
 * @param key the observer's key
 * @param periods a period for each tie, each overlap resolved against those before it
 * @param pureAt when the last ends, a UTC Julian day
 */
public record HouseholdPeriods(String key, List<Ashauch> periods, double pureAt) {
    /** Keeps the lists unmodifiable. */
    public HouseholdPeriods {
        periods = List.copyOf(periods);
    }
}
