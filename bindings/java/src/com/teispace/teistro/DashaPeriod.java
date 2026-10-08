package com.teispace.teistro;

/**
 * One period of a dasha.
 *
 * @param path Its place at each level from the mahadasha down, joined by {@code /}: {@code 2/5/3}.
 * @param level How deep: 1 for a mahadasha.
 * @param sign The sign it is the period of, in a sign-based dasha; null otherwise. May be null.
 * @param lord Its lord.
 * @param span When it runs.
 */
public record DashaPeriod(
        String path,
        int level,
        Rashi sign,
        Graha lord,
        Interval span) {
}
