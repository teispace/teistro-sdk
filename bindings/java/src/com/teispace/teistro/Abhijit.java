package com.teispace.teistro;

/**
 * Abhijit, with whether it is effective.
 *
 * @param at when it runs
 * @param effective true on every day but a Wednesday
 */
public record Abhijit(Interval at, boolean effective) {}
