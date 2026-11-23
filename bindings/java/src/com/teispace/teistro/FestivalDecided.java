package com.teispace.teistro;

/**
 * What decided an observance's day: a guard, by its index in the rule's
 * list, or the rule's {@code otherwise}; or, for a following rule, the rule
 * it counts from and how many days; or, for a rule decided as an Ekadashi,
 * the 10th's vedha and the excess.
 *
 * @param by {@code GUARD}, {@code OTHERWISE}, {@code AFTER} or {@code EKADASHI}
 * @param index the guard's index; may be null, unless a guard decided
 * @param rule the rule counted from; may be null, unless {@code by} is {@code AFTER}
 * @param days the days counted; may be null, unless {@code by} is {@code AFTER}
 * @param pierced whether the 10th pierces the 11th's day by the rule's vedha; may be null, unless {@code by} is {@code EKADASHI}
 * @param excess which of the 11th and the 12th stand in excess ({@code ELEVENTH}, {@code TWELFTH}, {@code BOTH}, {@code NEITHER}); may be null, unless {@code by} is {@code EKADASHI}
 */
public record FestivalDecided(String by, Integer index, String rule, Integer days, Boolean pierced, String excess) {}
