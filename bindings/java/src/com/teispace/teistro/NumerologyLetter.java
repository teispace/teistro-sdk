package com.teispace.teistro;

/**
 * One letter of a word and what the system's table makes it worth.
 *
 * @param letter the letter, upper case
 * @param value what it is worth
 */
public record NumerologyLetter(String letter, int value) {}
