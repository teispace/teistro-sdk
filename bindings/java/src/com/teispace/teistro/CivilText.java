package com.teispace.teistro;

import java.util.List;
import java.util.Map;

/**
 * A date or a time as text and parts, through the locale's pattern
 * ({@code 03-design/intl-engine-and-packs.md}).
 *
 * @param text the whole
 * @param parts the parts, in the locale's order; joined, they are {@code text}
 * @param locale the locale asked for
 * @param resolvedFrom the locale whose pattern answered
 * @param isFallback whether a fallback locale answered
 * @param isDraft whether any of it is text no reviewer has signed off
 * @param warnings every problem met on the way
 */
public record CivilText(String text, List<CivilPart> parts, String locale, String resolvedFrom, boolean isFallback,
        boolean isDraft, List<String> warnings) {

    /**
     * One part of a date or a time.
     *
     * @param kind what it is: {@code YEAR}, {@code MONTH}, {@code DAY}, {@code WEEKDAY}, {@code ERA},
     *     {@code HOUR}, {@code MINUTE}, {@code SECOND}, {@code DAY_PERIOD} or {@code LITERAL}
     * @param text its text
     */
    public record CivilPart(String kind, String text) {}

    static CivilText read(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        List<CivilPart> parts = Reads.array(o, "parts").stream().map(part -> {
            Map<?, ?> p = Reads.object(part);
            return new CivilPart(Reads.string(p, "kind"), Reads.string(p, "text"));
        }).toList();
        List<String> warnings = Reads.array(o, "warnings").stream().map(String::valueOf).toList();
        return new CivilText(Reads.string(o, "text"), parts, Reads.string(o, "locale"),
                Reads.string(o, "resolvedFrom"), Reads.flag(o, "isFallback"), Reads.flag(o, "isDraft"), warnings);
    }
}
