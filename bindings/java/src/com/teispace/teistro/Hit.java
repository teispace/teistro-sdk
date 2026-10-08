package com.teispace.teistro;

/**
 * One event of the transit hit list: {@code event} is one of {@code SignIngress},
 * {@code NakshatraIngress}, {@code Station} or {@code AspectHit}, each with its {@code kind}.
 *
 * @param instant When, a UTC Julian day.
 * @param graha The transiting graha.
 * @param event What happened.
 */
public record Hit(
        double instant,
        Graha graha,
        HitEvent event) {
}
