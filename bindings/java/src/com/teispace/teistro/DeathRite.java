package com.teispace.teistro;

/**
 * A rite the texts tie to the death-time readings, with its clause.
 *
 * @param rite {@code PANCHAKA_FIGURES}, {@code CLOSING_SHANTI} or {@code PUSHKARA_FIGURES}
 * @param clause the text and page
 */
public record DeathRite(String rite, String clause) {}
