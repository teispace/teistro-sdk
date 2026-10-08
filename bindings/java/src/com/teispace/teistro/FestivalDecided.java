package com.teispace.teistro;

/**
 * What decided an observance's day: a guard, by its index in the rule's
 * list, or the rule's {@code otherwise}; or, for a following rule, the rule
 * it counts from and how many days.
 *
 * @param by {@code GUARD}, {@code OTHERWISE} or {@code AFTER}
 * @param index the guard's index; may be null, unless a guard decided
 * @param rule the rule counted from; may be null, unless {@code by} is {@code AFTER}
 * @param days the days counted; may be null, unless {@code by} is {@code AFTER}
 */
public record FestivalDecided(String by, Integer index, String rule, Integer days) {}
