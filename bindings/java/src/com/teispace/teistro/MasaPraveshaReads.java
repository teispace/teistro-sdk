package com.teispace.teistro;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.function.Function;

import com.teispace.teistro.request.HuddaTable;
import com.teispace.teistro.request.MasaDashaRules;
import com.teispace.teistro.request.MasaDashaSign;
import com.teispace.teistro.request.MaseshaReading;
import com.teispace.teistro.request.MonthClock;

/**
 * The {@code masa_pravesha} section parsed once a batch into {@link MasaPravesha} records, as
 * the Python façade's {@code Chart.masa_pravesha} reads it.
 */
final class MasaPraveshaReads {
    private MasaPraveshaReads() {
    }

    /**
     * The chart's month or day charts; empty unless {@code masaPravesha} asked for them
     * ({@code 03-design/annual-chart.md}). Ports {@code Chart.masa_pravesha}.
     */
    static Optional<List<MasaPravesha>> masaPravesha(Chart chart) {
        return VedicReads.at(chart.batch().cached("masa_pravesha", () -> {
            String text = chart.batch().decoded().masaPravesha();
            if (text.isEmpty()) {
                return List.<List<MasaPravesha>>of();
            }
            List<List<MasaPravesha>> charts = new ArrayList<>();
            for (Object rows : JsonRead.list(Json.read(text))) {
                List<MasaPravesha> one = new ArrayList<>();
                for (Object row : JsonRead.list(rows)) {
                    one.add(new MasaPravesha(
                            JsonRead.integer(JsonRead.at(row, "year")),
                            JsonRead.integer(JsonRead.at(row, "month")),
                            JsonRead.integer(JsonRead.at(row, "day")),
                            JsonRead.decimal(JsonRead.at(row, "at")),
                            JsonRead.decimal(JsonRead.at(row, "sunDeg")),
                            row instanceof Map<?, ?> fields && fields.get("masesha") != null
                                    ? masesha(fields.get("masesha")) : null,
                            row instanceof Map<?, ?> fields && fields.get("results") != null
                                    ? results(fields.get("results")) : null,
                            row instanceof Map<?, ?> fields && fields.get("dasha") != null
                                    ? dasha(fields.get("dasha")) : null,
                            row instanceof Map<?, ?> fields && fields.get("dinesha") != null
                                    ? dinesha(fields.get("dinesha")) : null,
                            row instanceof Map<?, ?> fields && fields.get("dayResults") != null
                                    ? dayResults(fields.get("dayResults")) : null));
                }
                charts.add(List.copyOf(one));
            }
            return List.copyOf(charts);
        }), chart.index());
    }

    /** A month's lord, its keys made members and its strengths read as the year lord's are. */
    private static Masesha masesha(Object raw) {
        Object six = JsonRead.at(raw, "bearers");
        return new Masesha(
                graha(JsonRead.at(raw, "graha")),
                JsonRead.member(MaseshaReading::byKey, "MaseshaReading", JsonRead.at(raw, "reading")),
                new MonthBearers(
                        graha(JsonRead.at(six, "masaLagna")),
                        graha(JsonRead.at(six, "muntha")),
                        graha(JsonRead.at(six, "janmaLagna")),
                        graha(JsonRead.at(six, "triRashi")),
                        graha(JsonRead.at(six, "dinaRatri")),
                        graha(JsonRead.at(six, "varshaLagna")),
                        JsonRead.bool(JsonRead.at(six, "byDay"))),
                chain(JsonRead.at(raw, "chain")));
    }

    /** A day's lord, read as the month's lord is. */
    private static Dinesha dinesha(Object raw) {
        Object seven = JsonRead.at(raw, "bearers");
        return new Dinesha(
                graha(JsonRead.at(raw, "graha")),
                JsonRead.member(MaseshaReading::byKey, "MaseshaReading", JsonRead.at(raw, "reading")),
                new DayBearers(
                        graha(JsonRead.at(seven, "muntha")),
                        graha(JsonRead.at(seven, "janmaLagna")),
                        graha(JsonRead.at(seven, "triRashi")),
                        graha(JsonRead.at(seven, "dinaRatri")),
                        graha(JsonRead.at(seven, "dinaLagna")),
                        graha(JsonRead.at(seven, "masaLagna")),
                        graha(JsonRead.at(seven, "varshaLagna")),
                        JsonRead.bool(JsonRead.at(seven, "byDay"))),
                chain(JsonRead.at(raw, "chain")));
    }

    /** A month's or a day's chain, its strengths read as the year lord's are. */
    private static YearLord chain(Object chain) {
        List<YearClaim> claims = new ArrayList<>();
        for (Object claim : JsonRead.list(JsonRead.at(chain, "claims"))) {
            claims.add(new YearClaim(
                    graha(JsonRead.at(claim, "graha")),
                    TajikaReads.bala(JsonRead.integer(JsonRead.at(claim, "vishwa"))),
                    JsonRead.integer(JsonRead.at(claim, "portfolios")),
                    JsonRead.bool(JsonRead.at(claim, "aspectsLagna"))));
        }
        return new YearLord(
                graha(JsonRead.at(chain, "graha")),
                JsonRead.member(VarsheshaChosen::byKey, "VarsheshaChosen", JsonRead.at(chain, "chosen")),
                TajikaReads.bala(JsonRead.integer(JsonRead.at(chain, "vishwa"))),
                JsonRead.bool(JsonRead.at(chain, "moonPassedOver")),
                List.copyOf(claims));
    }

    /** A month's results, their grahas members. */
    private static MasaPhala results(Object raw) {
        List<MonthHouse> houses = new ArrayList<>();
        for (Object one : JsonRead.list(JsonRead.at(raw, "houses"))) {
            houses.add(new MonthHouse(
                    JsonRead.integer(JsonRead.at(one, "house")),
                    graha(JsonRead.at(one, "amshaLord")),
                    graha(JsonRead.at(one, "lordAmshaLord")),
                    keys(JsonRead.at(one, "clauses"), MonthHouseClause::valueOf),
                    keys(JsonRead.at(one, "seenByAmshaLord"), MonthLord::valueOf),
                    keys(JsonRead.at(one, "seenByMoon"), MonthLord::valueOf)));
        }
        List<MonthLordPlace> lords = new ArrayList<>();
        for (Object one : JsonRead.list(JsonRead.at(raw, "lords"))) {
            lords.add(new MonthLordPlace(
                    MonthLord.valueOf(JsonRead.text(JsonRead.at(one, "office"))),
                    graha(JsonRead.at(one, "graha")),
                    JsonRead.integer(JsonRead.at(one, "house")),
                    keys(JsonRead.at(one, "clauses"), MonthLordClause::valueOf)));
        }
        return new MasaPhala(List.copyOf(houses), List.copyOf(lords));
    }

    /** A month's dasha, its keys made members. */
    private static MasaDasha dasha(Object raw) {
        Object rules = JsonRead.at(raw, "rules");
        List<MasaDashaPeriod> periods = new ArrayList<>();
        for (Object one : JsonRead.list(JsonRead.at(raw, "periods"))) {
            periods.add(new MasaDashaPeriod(
                    graha(JsonRead.at(one, "lord")),
                    JsonRead.integer(JsonRead.at(one, "degrees")),
                    JsonRead.decimal(JsonRead.at(one, "starts")),
                    JsonRead.decimal(JsonRead.at(one, "ends"))));
        }
        return new MasaDasha(
                MonthOffice.valueOf(JsonRead.text(JsonRead.at(raw, "office"))),
                graha(JsonRead.at(raw, "graha")),
                JsonRead.member(Rashi::byKey, "Rashi", JsonRead.at(raw, "sign")),
                new MasaDashaRules(
                        JsonRead.member(MonthClock::byKey, "MonthClock", JsonRead.at(rules, "clock")),
                        JsonRead.member(MasaDashaSign::byKey, "MasaDashaSign", JsonRead.at(rules, "sign")),
                        JsonRead.member(HuddaTable::byKey, "HuddaTable", JsonRead.at(rules, "table"))),
                List.copyOf(periods));
    }

    /** A day's results, their grahas members. */
    private static DinaPhala dayResults(Object raw) {
        List<DayHouse> houses = new ArrayList<>();
        for (Object one : JsonRead.list(JsonRead.at(raw, "houses"))) {
            houses.add(new DayHouse(
                    JsonRead.integer(JsonRead.at(one, "house")),
                    graha(JsonRead.at(one, "amshaLord")),
                    keys(JsonRead.at(one, "clauses"), DayHouseClause::valueOf)));
        }
        List<DayLordPlace> lords = new ArrayList<>();
        for (Object one : JsonRead.list(JsonRead.at(raw, "lords"))) {
            lords.add(new DayLordPlace(
                    DayLord.valueOf(JsonRead.text(JsonRead.at(one, "office"))),
                    graha(JsonRead.at(one, "graha")),
                    JsonRead.integer(JsonRead.at(one, "house")),
                    keys(JsonRead.at(one, "clauses"), MonthLordClause::valueOf)));
        }
        return new DinaPhala(
                MoonState.valueOf(JsonRead.text(JsonRead.at(raw, "moonState"))),
                List.copyOf(houses),
                List.copyOf(lords),
                keys(JsonRead.at(raw, "clauses"), DayClause::valueOf));
    }

    private static <T> List<T> keys(Object raw, Function<String, T> member) {
        List<T> out = new ArrayList<>();
        for (Object key : JsonRead.list(raw)) {
            out.add(member.apply(JsonRead.text(key)));
        }
        return List.copyOf(out);
    }

    private static Graha graha(Object key) {
        return JsonRead.member(Graha::byKey, "Graha", key);
    }
}
