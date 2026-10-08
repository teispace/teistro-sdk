package com.teispace.teistro;

/**
 * One clause of the verdict, naming its verse by {@code kind} ({@code LAGNA_RISING},
 * {@code IN_LAGNA}, {@code RISING_NAVAMSHA}, {@code ASPECTS_LAGNA}, {@code ASPECTS_MOON} or
 * {@code KARYA_HOUSE}) and telling {@code FOR}, {@code AGAINST} or {@code BOTH}.
 *
 * @param kind The verse.
 * @param graha The graha it is about; null for how the lagna rises. May be null.
 * @param favour {@code FOR}, {@code AGAINST} or {@code BOTH}.
 */
public record PrashnaClause(
        String kind,
        Graha graha,
        String favour) {
}
