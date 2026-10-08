package com.teispace.teistro;

import java.util.List;

/**
 * A return's own chart, read down to what Tajika reads from it.
 *
 * @param lagnaDeg The annual chart's lagna, sidereal degrees, at the place it was cast for.
 * @param byDay Whether the return fell between sunrise and sunset there.
 * @param officeBearers The five office-bearers.
 * @param yearLord The lord of the year, chosen among them.
 * @param yogas The pairs of the seven that make a Tajika yoga in this chart. The pairs that make
 *     none do not cross.
 * @param retrograde The seven retrograde in this chart: what the matters were judged on.
 * @param combust The seven combust in this chart, under the context's combustion table.
 * @param matters The sixteen yogas for each matter the varsha request's {@code matters} asked
 *     about, in its order; empty otherwise.
 * @param sahams Each saham the varsha request's {@code sahams} asked for, in its order, with its
 *     strength under the year's lord; empty otherwise.
 * @param harsha The seven's Harsha bala in this year's chart, in the catalogue's order.
 * @param dashas Each annual dasha the varsha request's {@code dashas} asked for, in its order,
 *     under its {@code dasha_rules}; empty otherwise.
 */
public record AnnualChart(
        double lagnaDeg,
        boolean byDay,
        OfficeBearers officeBearers,
        YearLord yearLord,
        List<TajikaPair> yogas,
        List<Graha> retrograde,
        List<Graha> combust,
        List<TajikaMatter> matters,
        List<TajikaSaham> sahams,
        List<HarshaBala> harsha,
        List<AnnualDasha> dashas) {
    /** The value, its lists copied and unmodifiable. */
    public AnnualChart {
        yogas = List.copyOf(yogas);
        retrograde = List.copyOf(retrograde);
        combust = List.copyOf(combust);
        matters = List.copyOf(matters);
        sahams = List.copyOf(sahams);
        harsha = List.copyOf(harsha);
        dashas = List.copyOf(dashas);
    }
}
