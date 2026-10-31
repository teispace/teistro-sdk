package com.teispace.teistro;

import java.util.List;

import com.teispace.teistro.record.Provenance;

/**
 * Each day's sky, in the days' order.
 *
 * <pre>{@code
 * BodySky sun = almanac.sky().orElseThrow().value().get(0).sun();
 * }</pre>
 *
 * @param value one record a day
 * @param provenance the days' own provenance with the window's reading named, and the hash of {@code value}
 */
public record DaySkies(List<DaySky> value, Provenance provenance) {
    /** Keeps the list unmodifiable. */
    public DaySkies {
        value = List.copyOf(value);
    }
}
