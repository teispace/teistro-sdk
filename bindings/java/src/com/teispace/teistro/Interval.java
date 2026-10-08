package com.teispace.teistro;

/**
 * A span of time, as every almanac row carries one.
 *
 * @param fromJd When it begins, as a Julian day (UTC).
 * @param toJd When it ends.
 */
public record Interval(
        double fromJd,
        double toJd) {
}
