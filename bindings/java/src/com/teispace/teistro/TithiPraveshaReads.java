package com.teispace.teistro;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

import com.teispace.teistro.request.TithiRule;

/**
 * The {@code tithi_pravesha} section parsed once a batch into {@link TithiPravesha} records, as
 * the Python façade's {@code Chart.tithi_pravesha} reads it.
 */
final class TithiPraveshaReads {
    private TithiPraveshaReads() {
    }

    /**
     * The chart's years, {@code 1} up; empty unless {@code tithiPravesha} asked for them
     * ({@code 03-design/tithi-pravesha.md}). Ports {@code Chart.tithi_pravesha}.
     */
    static Optional<List<TithiPravesha>> tithiPravesha(Chart chart) {
        return VedicReads.at(chart.batch().cached("tithi_pravesha", () -> {
            String text = chart.batch().decoded().tithiPravesha();
            if (text.isEmpty()) {
                return List.<List<TithiPravesha>>of();
            }
            List<List<TithiPravesha>> charts = new ArrayList<>();
            for (Object years : JsonRead.list(Json.read(text))) {
                List<TithiPravesha> one = new ArrayList<>();
                for (Object year : JsonRead.list(years)) {
                    one.add(yearOf(year));
                }
                charts.add(List.copyOf(one));
            }
            return List.copyOf(charts);
        }), chart.index());
    }

    private static TithiPravesha yearOf(Object raw) {
        return new TithiPravesha(
                JsonRead.integer(JsonRead.at(raw, "year")),
                JsonRead.decimal(JsonRead.at(raw, "at")),
                JsonRead.decimal(JsonRead.at(raw, "solarReturn")),
                JsonRead.decimal(JsonRead.at(raw, "elongationDeg")),
                TithiRule.of(JsonRead.at(raw, "rule")),
                TithiRule.of(JsonRead.at(raw, "picked")),
                JsonRead.integer(JsonRead.at(raw, "inSign")));
    }
}
