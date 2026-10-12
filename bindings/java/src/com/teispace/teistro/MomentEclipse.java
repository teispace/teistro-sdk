package com.teispace.teistro;

/**
 * The eclipse a chart's instant falls in, seen where the chart is cast: a lunar eclipse's umbral
 * phase with the Moon up, or a solar eclipse with the Sun up. A classical astronomy's chart is
 * {@link BirthEclipse#UNJUDGED} (C188).
 *
 * @param kind which, or {@link BirthEclipse#NONE}
 * @param greatest its greatest moment, a UT1 Julian day; null unless lunar or solar
 */
public record MomentEclipse(BirthEclipse kind, Double greatest) {}
