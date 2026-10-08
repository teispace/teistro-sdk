package com.teispace.teistro;

/**
 * Cheiro's numbers of a date, which are "not added together" (p. 93).
 *
 * @param birth the day of the month, reduced
 * @param year the year's digits reduced
 */
public record ChaldeanDate(Reduction birth, Reduction year) {}
