package com.teispace.teistro;

import com.teispace.teistro.record.Provenance;

/**
 * The eclipses an almanac's days hold ({@code 03-design/eclipses.md}).
 *
 * <pre>{@code
 * List<LunarEclipseHere> seen = almanac.eclipses().orElseThrow().value().lunar().stream()
 *         .filter(e -> e.here().seen() != null).toList();
 * }</pre>
 *
 * @param value the eclipses
 * @param provenance what computed them, the window searched among its
 *     conventions, and the hash of {@code value}
 */
public record Eclipses(EclipsesFound value, Provenance provenance) {}
