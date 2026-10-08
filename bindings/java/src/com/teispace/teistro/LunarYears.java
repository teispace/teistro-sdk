package com.teispace.teistro;

import java.util.List;

import com.teispace.teistro.record.Provenance;

/**
 * The lunar years an almanac's days fall in, in order and abutting.
 *
 * <pre>{@code
 * Samvatsara name = almanac.years().orElseThrow().value().get(0).samvatsara();
 * }</pre>
 *
 * @param value the years
 * @param provenance what computed them, and the hash of {@code value}
 */
public record LunarYears(List<LunarYear> value, Provenance provenance) {
    /** Keeps the list unmodifiable. */
    public LunarYears {
        value = List.copyOf(value);
    }
}
