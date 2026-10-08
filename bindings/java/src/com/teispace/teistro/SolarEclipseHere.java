package com.teispace.teistro;

/**
 * A solar eclipse and how the place sees it.
 *
 * @param eclipse the eclipse
 * @param here how the place sees it; may be null, where the penumbra never reaches
 */
public record SolarEclipseHere(SolarEclipse eclipse, SolarEclipseView here) {}
