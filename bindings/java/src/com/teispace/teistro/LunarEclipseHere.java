package com.teispace.teistro;

/**
 * A lunar eclipse and how the place sees it.
 *
 * @param eclipse the eclipse
 * @param here how the place sees it
 */
public record LunarEclipseHere(LunarEclipse eclipse, LunarEclipseView here) {}
