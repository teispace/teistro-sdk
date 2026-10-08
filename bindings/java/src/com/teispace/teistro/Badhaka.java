package com.teispace.teistro;

/**
 * The badhaka house, the 11th from a movable lagna, the 9th from a fixed and the 7th from a dual,
 * and its lord.
 *
 * @param house The house.
 * @param lord Its lord.
 */
public record Badhaka(
        int house,
        Graha lord) {
}
