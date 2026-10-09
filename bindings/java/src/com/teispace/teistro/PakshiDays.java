package com.teispace.teistro;

import java.util.List;

import com.teispace.teistro.record.Provenance;

/**
 * A native's bird over a range of days, with what sealed the request.
 *
 * <pre>{@code
 * PakshiDays days = sky.almanac().pakshi(date, date, madras, 19_800,
 *         PakshiNative.star(Nakshatra.UTTARA_ASHADHA, Paksha.SHUKLA));
 * String sealed = days.provenance().inputHash();
 * }</pre>
 *
 * @param value the days, in order
 * @param provenance what computed them, and under what; {@code inputHash} seals the request
 */
public record PakshiDays(List<PakshiDay> value, Provenance provenance) {
    /** Keeps the list unmodifiable. */
    public PakshiDays {
        value = List.copyOf(value);
    }
}
