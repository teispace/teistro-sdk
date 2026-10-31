package com.teispace.teistro;

/**
 * An office's weekday and its lord under one rule ({@code 03-design/year-rulers.md}).
 *
 * @param rule the rule: {@code SUNRISE}, {@code NEW_MOON} or {@code GARGA}
 *     for the raja, {@code DAYLIGHT} or {@code VARA} for an ingress
 * @param vara the weekday the rule reads
 * @param lord its lord, who holds the office
 * @param day the sunrise that opened that weekday, a UTC Julian day
 */
public record RulerReading(String rule, Vara vara, Graha lord, double day) {}
