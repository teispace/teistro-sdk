package com.teispace.teistro;

import java.util.List;

/**
 * The Shiva Svarodaya around a chart's instant: the reading at it and every run of the window
 * ({@code 03-design/rectification.md}). Instants are Julian days (UTC).
 *
 * @param at The nadi and the tattva at the chart's instant.
 * @param runs Every run of the window, in order and clipped to it.
 */
public record SvarodayaAround(
        Reading at,
        List<Run> runs) {
    /** The value, its lists copied and unmodifiable. */
    public SvarodayaAround {
        runs = List.copyOf(runs);
    }

    /**
     * The nadi and the tattva at an instant, and the day they are counted in.
     *
     * @param sunrise The sunrise the turns are counted from.
     * @param nextSunrise The sunrise that ends the day.
     * @param tithi The tithi at the sunrise, which gives its nadi.
     * @param sunriseNadi The nadi rising at the sunrise: {@code MOON} or {@code SUN}.
     * @param run The run the instant falls in.
     * @param junctions The turn's junctions, where the sushumna flows for a moment: its start and
     *     its end.
     */
    public record Reading(
            double sunrise,
            double nextSunrise,
            Tithi tithi,
            String sunriseNadi,
            Run run,
            List<Double> junctions) {
        /** The value, its lists copied and unmodifiable. */
        public Reading {
            junctions = List.copyOf(junctions);
        }
    }

    /**
     * One stretch of a day under one nadi and one tattva.
     *
     * @param from Where it starts.
     * @param to Where it ends.
     * @param nadi The nadi flowing: {@code MOON}, the left (ida), or {@code SUN}, the right
     *     (pingala).
     * @param turn Its turn in the day, 0 the one rising at sunrise, to 23.
     * @param tattva The tattva flowing in it: {@code PRITHVI}, {@code JALA}, {@code AGNI},
     *     {@code VAYU} or {@code AKASHA}.
     * @param sex The sex v. 60 gives the nadi.
     */
    public record Run(
            double from,
            double to,
            String nadi,
            int turn,
            String tattva,
            Sex sex) {
    }
}
