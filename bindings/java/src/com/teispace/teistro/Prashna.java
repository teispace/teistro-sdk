package com.teispace.teistro;

import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.Map;

/**
 * A chart read as a prashna ({@code 03-design/prashna.md}).
 *
 * @param rules The readings it was given under, every member filled, as the library wrote them.
 * @param verdict Whether the matter succeeds.
 * @param change {@code STAYS} or {@code CHANGES} (II.1 to 2).
 * @param timing When the matter comes to pass.
 * @param mook What an unspoken question is about.
 * @param links The Tajika links; null when no house was asked. May be null.
 * @param moon The Moon's weaknesses.
 * @param score The baseline's points; null unless {@code rules.score} is {@code BASELINE}. May be
 *     null.
 * @param numberSign The sign of the querent's number; null unless one was given. May be null.
 */
public record Prashna(
        Map<String, Object> rules,
        PrashnaVerdict verdict,
        String change,
        PrashnaTiming timing,
        PrashnaMook mook,
        PrashnaLinks links,
        MoonWeakness moon,
        PrashnaScore score,
        Rashi numberSign) {
    /** The value, its lists copied and unmodifiable. */
    public Prashna {
        rules = Collections.unmodifiableMap(new LinkedHashMap<>(rules));
    }
}
