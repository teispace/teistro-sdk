package com.teispace.teistro;

import java.util.List;

/**
 * A chart's essential dignities, with everything that made them: the sect,
 * the rule that chose it, the rules and the scores
 * ({@code 03-design/essential-dignities.md}).
 *
 * <pre>{@code
 * Chart chart = sky.chart().found(instant, place, offset, ChartOptions.builder().dignities(Map.of()).build());
 * PlanetDignity mars = chart.dignities().orElseThrow().planets().stream()
 *         .filter(at -> at.planet() == Graha.MARS).findFirst().orElseThrow();
 * }</pre>
 *
 * @param sect the chart's sect
 * @param sectRule the rule that chose it
 * @param rules the terms and triplicities used
 * @param scores what each dignity was worth
 * @param planets the seven in the Chaldean order, Saturn first
 * @param receptions every pair in reception, in the Chaldean order of the
 *     first and then the second
 */
public record Dignities(
        Sect sect, SectRule sectRule, AppliedDignityRules rules, DignityScores scores, List<PlanetDignity> planets,
        List<Reception> receptions) {
    /** Keeps the lists unmodifiable. */
    public Dignities {
        planets = List.copyOf(planets);
        receptions = List.copyOf(receptions);
    }
}
