package com.teispace.teistro;

/**
 * A chart read as KP: the chart, its significators and the ruling planets of its moment
 * ({@code 03-design/kp.md}).
 *
 * @param chart The chart.
 * @param significators Its significators.
 * @param ruling The ruling planets of its moment.
 */
public record KpReading(
        KpChart chart,
        KpSignificators significators,
        KpRuling ruling) {
}
