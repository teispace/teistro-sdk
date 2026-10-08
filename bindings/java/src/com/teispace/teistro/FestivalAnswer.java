package com.teispace.teistro;

import java.util.List;

import com.teispace.teistro.record.Provenance;

/**
 * What a set of festival rules gives over an almanac's days
 * ({@code 03-design/festival-rules.md} §7.3).
 *
 * <pre>{@code
 * Almanac almanac = sky.almanac().of(first, last, place, offset,
 *         null, Map.of("rules", "DHARMASINDHU"), false, false, false);
 * FestivalObservance first = almanac.festivals().orElseThrow().observances().get(0);
 * }</pre>
 *
 * @param observances each rule's day
 * @param ekadashis each Ekadashi rule's fasts, in the order of the tithis
 * @param unjudged the occurrences no day could be given to
 * @param provenance what computed it: the widened days among the applied
 *     conventions as {@code festival.days}, and the hash of the value
 */
public record FestivalAnswer(
        List<FestivalObservance> observances, List<EkadashiFast> ekadashis, List<FestivalUnjudged> unjudged,
        Provenance provenance) {
    /** Keeps the lists unmodifiable. */
    public FestivalAnswer {
        observances = List.copyOf(observances);
        ekadashis = List.copyOf(ekadashis);
        unjudged = List.copyOf(unjudged);
    }
}
