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
 * @param dated each dated rule's days, in the order of the days; empty when
 *     none was asked
 * @param anniversaries each anniversary's days, in the order of the tithis;
 *     empty when none was asked
 * @param added the days published sets added; empty when none
 * @param unplaced the published entries in range no occurrence took
 * @param applied the published sets laid over the answer, in order
 * @param provenance what computed it: the widened days among the applied
 *     conventions as {@code festival.days}, and the hash of the value
 */
public record FestivalAnswer(
        List<FestivalObservance> observances, List<EkadashiFast> ekadashis, List<FestivalUnjudged> unjudged,
        List<DatedObservance> dated, List<AnniversaryDay> anniversaries, List<Published.Added> added,
        List<Published.Unplaced> unplaced, List<Published.Applied> applied, Provenance provenance) {
    /** Keeps the lists unmodifiable. */
    public FestivalAnswer {
        observances = List.copyOf(observances);
        ekadashis = List.copyOf(ekadashis);
        unjudged = List.copyOf(unjudged);
        dated = List.copyOf(dated);
        anniversaries = List.copyOf(anniversaries);
        added = List.copyOf(added);
        unplaced = List.copyOf(unplaced);
        applied = List.copyOf(applied);
    }

    /**
     * An answer no published set was laid over, as one was written before
     * sets were.
     *
     * @param observances each rule's day
     * @param ekadashis each Ekadashi rule's fasts
     * @param unjudged the occurrences no day could be given to
     * @param dated each dated rule's days
     * @param anniversaries each anniversary's days
     * @param provenance what computed it
     */
    public FestivalAnswer(List<FestivalObservance> observances, List<EkadashiFast> ekadashis,
            List<FestivalUnjudged> unjudged, List<DatedObservance> dated, List<AnniversaryDay> anniversaries,
            Provenance provenance) {
        this(observances, ekadashis, unjudged, dated, anniversaries, List.of(), List.of(), List.of(), provenance);
    }

    /**
     * An answer with no anniversaries, as one was written before they were
     * asked for.
     *
     * @param observances each rule's day
     * @param ekadashis each Ekadashi rule's fasts
     * @param unjudged the occurrences no day could be given to
     * @param dated each dated rule's days
     * @param provenance what computed it
     */
    public FestivalAnswer(List<FestivalObservance> observances, List<EkadashiFast> ekadashis,
            List<FestivalUnjudged> unjudged, List<DatedObservance> dated, Provenance provenance) {
        this(observances, ekadashis, unjudged, dated, List.of(), List.of(), List.of(), List.of(), provenance);
    }
}
