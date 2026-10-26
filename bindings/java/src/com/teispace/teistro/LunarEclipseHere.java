package com.teispace.teistro;

/**
 * A lunar eclipse and how the place sees it.
 *
 * @param eclipse the eclipse
 * @param here how the place sees it
 * @param vedha the vedha, the sutak, under {@code panchanga.eclipse_vedha}: from when food is given up to the
 *     eclipse's end as seen; null where the place does not see it
 */
public record LunarEclipseHere(LunarEclipse eclipse, LunarEclipseView here, EclipseSeen vedha) {}
