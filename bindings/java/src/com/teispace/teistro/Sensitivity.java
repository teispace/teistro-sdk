package com.teispace.teistro;

import java.util.List;
import java.util.Optional;

/**
 * How far a chart's time can move before each fact it states changes
 * ({@code 03-design/rectification.md}, "Birth-time sensitivity"). Instants are Julian days (UTC).
 *
 * @param birth The chart's instant.
 * @param from Where the span searched starts.
 * @param to Where it ends.
 * @param precisionSeconds How closely each change is placed: a hundredth of a second.
 * @param grid The grid the span was seeded on.
 * @param lagnaSign The lagna's sign.
 * @param lagnaPada The lagna's nakshatra and pada.
 * @param vargas Each varga's lagna: D9, D10 and those the chart carries, unless asked.
 * @param moonPada The Moon's nakshatra and pada.
 * @param dasha The dasha running at the reference instant; empty unless one was asked.
 * @param cusps Each bhava middle's sign, houses 1 to 12; none under a whole-sign chalit.
 */
public record Sensitivity(
        double birth,
        double from,
        double to,
        double precisionSeconds,
        Purified.Grid grid,
        Fact<Rashi> lagnaSign,
        Fact<Pada> lagnaPada,
        List<VargaLagna> vargas,
        Fact<Pada> moonPada,
        Optional<Running> dasha,
        List<Cusp> cusps) {
    /** The value, its lists copied and unmodifiable. */
    public Sensitivity {
        vargas = List.copyOf(vargas);
        cusps = List.copyOf(cusps);
    }

    /**
     * Where a fact changes, nearest the birth on one side.
     *
     * @param <T> What the fact is.
     * @param at The instant, to a hundredth of a second.
     * @param minutes How far it is from the birth, in minutes, positive on either side.
     * @param to What the fact becomes.
     */
    public record Change<T>(
            double at,
            double minutes,
            T to) {
    }

    /**
     * A fact at the birth and its nearest change either side; empty on a side where it holds over
     * the whole span searched.
     *
     * @param <T> What the fact is.
     * @param value The fact at the birth.
     * @param earlier Its nearest change before the birth.
     * @param later Its nearest change after the birth.
     */
    public record Fact<T>(
            T value,
            Optional<Change<T>> earlier,
            Optional<Change<T>> later) {
    }

    /**
     * A nakshatra's quarter.
     *
     * @param nakshatra The nakshatra.
     * @param pada The pada, 1 to 4.
     */
    public record Pada(
            Nakshatra nakshatra,
            int pada) {
    }

    /**
     * One varga's lagna and its changes.
     *
     * @param varga Which varga.
     * @param lagna Its lagna's sign.
     */
    public record VargaLagna(
            Varga varga,
            Fact<Rashi> lagna) {
    }

    /**
     * One bhava middle's sign under the chart's chalit, and its changes.
     *
     * @param house The house, 1 to 12.
     * @param sign Its middle's sign.
     */
    public record Cusp(
            int house,
            Fact<Rashi> sign) {
    }

    /**
     * The dasha running at a reference instant, and its changes.
     *
     * @param at The reference instant.
     * @param system The system.
     * @param lords The lords running then, the mahadasha first; none before the birth or past the
     *     cycle's end.
     */
    public record Running(
            double at,
            DashaSystem system,
            Fact<List<Graha>> lords) {
    }
}
