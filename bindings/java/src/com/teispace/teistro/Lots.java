package com.teispace.teistro;

import java.util.List;

/**
 * A chart's lots, with its sect and the rules they were read under
 * ({@code 03-design/hellenistic-lots.md}).
 *
 * <pre>{@code
 * Rashi fortune = chart.lots().orElseThrow().lots().stream()
 *         .filter(at -> at.lot() == Lot.FORTUNE).findFirst().orElseThrow().place().sign();
 * }</pre>
 *
 * @param sect the chart's sect
 * @param request the rules they were read under
 * @param fortuneReversed whether Fortune was counted from the Moon to the
 *     Sun, and Daimon the other way
 * @param lots all fourteen, in the catalogue's order
 */
public record Lots(Sect sect, LotRules request, boolean fortuneReversed, List<PlacedLot> lots) {
    /** Keeps the list unmodifiable. */
    public Lots {
        lots = List.copyOf(lots);
    }
}
