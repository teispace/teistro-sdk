package com.teispace.teistro;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

/**
 * The {@code western_configurations} section parsed once a batch into {@link
 * WesternConfiguration} records, as the Python façade's {@code Chart.western_configurations}
 * reads it.
 */
final class WesternConfigurationReads {
    private WesternConfigurationReads() {
    }

    /**
     * The chart's aspect configurations; empty unless {@code westernConfigurations} asked for
     * them ({@code 03-design/western-configurations.md}). Ports {@code Chart.western_configurations}.
     */
    static Optional<List<WesternConfiguration>> westernConfigurations(Chart chart) {
        return VedicReads.at(chart.batch().cached("western_configurations", () -> {
            String text = chart.batch().decoded().westernConfigurations();
            if (text.isEmpty()) {
                return List.<List<WesternConfiguration>>of();
            }
            List<List<WesternConfiguration>> charts = new ArrayList<>();
            for (Object shapes : JsonRead.list(Json.read(text))) {
                List<WesternConfiguration> one = new ArrayList<>();
                for (Object shape : JsonRead.list(shapes)) {
                    one.add(shape(shape));
                }
                charts.add(List.copyOf(one));
            }
            return List.copyOf(charts);
        }), chart.index());
    }

    /** One shape, its catalogue keys made members. */
    private static WesternConfiguration shape(Object raw) {
        Object widest = JsonRead.at(raw, "widestFromExactDeg");
        List<Integer> inside = new ArrayList<>();
        for (Object at : JsonRead.list(JsonRead.at(raw, "inside"))) {
            inside.add(JsonRead.integer(at));
        }
        return new WesternConfiguration(
                WesternConfigurationKind.valueOf(JsonRead.text(JsonRead.at(raw, "kind"))),
                JsonRead.members(Graha::byKey, "Graha", JsonRead.at(raw, "bodies")),
                JsonRead.memberOrNull(Graha::byKey, "Graha", JsonRead.at(raw, "apex")),
                JsonRead.memberOrNull(Rashi::byKey, "Rashi", JsonRead.at(raw, "sign")),
                widest == null ? null : JsonRead.decimal(widest),
                inside);
    }
}
