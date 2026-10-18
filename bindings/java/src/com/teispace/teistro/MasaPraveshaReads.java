package com.teispace.teistro;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

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
                            JsonRead.decimal(JsonRead.at(row, "sunDeg"))));
                }
                charts.add(List.copyOf(one));
            }
            return List.copyOf(charts);
        }), chart.index());
    }
}
