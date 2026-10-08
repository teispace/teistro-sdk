package com.teispace.teistro;

/**
 * A sign lord's lucky elements, as the baseline engine gives them.
 *
 * @param colour the colour
 * @param number the number
 * @param day the day
 * @param direction the direction
 */
public record LuckyElements(String colour, int number, Vara day, Direction direction) {}
