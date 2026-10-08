package com.teispace.teistro;

/**
 * Where in its day a chart's moment falls: the ishtakaal and the hora. The same record in every
 * binding.
 *
 * @param ghati The ishtakaal's ghatis since sunrise, 0 to 59.
 * @param pala Its palas, 0 to 59.
 * @param vipala Its vipalas, 0 to 59.
 * @param ghatiReckoning How the ghatis were measured.
 * @param horaNumber Which hora of the day holds the instant, 1 to 24.
 * @param horaLord The graha that rules it.
 * @param horaStart When that hora began, as a Julian day (UTC).
 * @param horaEnd When it ends, as a Julian day (UTC).
 */
public record ChartTiming(
        int ghati,
        int pala,
        int vipala,
        GhatiReckoning ghatiReckoning,
        int horaNumber,
        Graha horaLord,
        double horaStart,
        double horaEnd) {
}
