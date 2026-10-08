package com.teispace.teistro;

import java.util.List;

import com.teispace.teistro.record.Provenance;

/**
 * Each day of an almanac's Nepal Sambat date, in the days' order.
 *
 * <pre>{@code
 * int year = almanac.nepalSambat().orElseThrow().value().get(0).year();
 * }</pre>
 *
 * @param value one date a day
 * @param provenance the days' own provenance, and the hash of {@code value}
 */
public record NepalSambatDates(List<NepalSambatDate> value, Provenance provenance) {
    /** Keeps the list unmodifiable. */
    public NepalSambatDates {
        value = List.copyOf(value);
    }
}
