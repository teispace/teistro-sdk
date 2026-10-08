package com.teispace.teistro;

import java.util.List;

/**
 * Both halves of Lilly's table in one chart, with everything that made
 * them ({@code 03-design/essential-dignities.md} §Accidental fortitudes).
 *
 * <pre>{@code
 * PlanetAccidents strongest = chart.fortitudes().orElseThrow().planets().stream()
 *         .max(Comparator.comparingInt(PlanetAccidents::net)).orElseThrow();
 * }</pre>
 *
 * @param dignities the essential half, which the chart's {@code dignities} also reads
 * @param sky what the accidental half was read from
 * @param rules the orbs and limits it was judged by
 * @param scores what each line was worth
 * @param planets the seven in the Chaldean order, Saturn first
 * @param almutens the chart's almutens
 */
public record Fortitudes(
        Dignities dignities, AccidentalSky sky, AccidentalRules rules, AccidentalScores scores,
        List<PlanetAccidents> planets, Almutens almutens) {
    /** Keeps the list unmodifiable. */
    public Fortitudes {
        planets = List.copyOf(planets);
    }
}
