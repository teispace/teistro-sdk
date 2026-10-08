package com.teispace.teistro;

/**
 * One factor of the baseline's points, and what it gave.
 *
 * @param kind The factor.
 * @param points What it gave.
 */
public record PrashnaFactor(
        String kind,
        int points) {
}
