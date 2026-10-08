package com.teispace.teistro;

/**
 * A day with no sunrise or no sunset: which, and what the policy did about it.
 *
 * @param kind Whether the Sun stayed up or stayed down.
 * @param policy The policy that put bounds on the day.
 */
public record PolarDay(
        PolarKind kind,
        PolarDayPolicy policy) {
}
