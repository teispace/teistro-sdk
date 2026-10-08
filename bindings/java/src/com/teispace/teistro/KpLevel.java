package com.teispace.teistro;

/**
 * One level of a point's lords below the sign: its lord, and the arc it rules.
 *
 * @param lord The lord.
 * @param span The arc it rules.
 */
public record KpLevel(
        Graha lord,
        KpSpan span) {
}
