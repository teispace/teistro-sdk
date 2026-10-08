package com.teispace.teistro;

import java.time.LocalDate;
import java.util.LinkedHashMap;
import java.util.Map;

/**
 * {@code sky.numerology()}: a name and a birth date under Balliett's letter
 * cycle and Cheiro's Chaldean table ({@code 03-design/numerology.md}, C320
 * to C328). It reads no sky.
 *
 * <pre>{@code
 * NumerologyProfile read = sky.numerology().profile("Henry Elder", LocalDate.of(1872, 1, 17));
 * }</pre>
 */
public final class NumerologyArea {
    private final Context context;

    /**
     * The numerology area of a context.
     *
     * @param context the context every call goes through
     */
    NumerologyArea(Context context) {
        this.context = context;
    }

    /**
     * Everything numerology says of a name and a birth date, under the
     * sources' own readings.
     *
     * @param name the name
     * @param date the birth date
     * @return the profile
     * @see #profile(String, LocalDate, Map)
     */
    public NumerologyProfile profile(String name, LocalDate date) {
        return profile(name, date, null);
    }

    /**
     * Everything numerology says of a name and a birth date: the name under
     * both systems, word by word with every reduction step, Balliett's birth
     * number, Cheiro's day and year, and the baseline engine's own numbers
     * under every baseline reading only.
     *
     * <p>The name is read in the 26 Latin letters; anything else is refused,
     * named {@code numerology.name}, unless {@code rules.nonLatin} is
     * {@code "SKIP"}.
     *
     * @param name the name
     * @param date the birth date
     * @param rules the readings, such as {@code Map.of("masters", "NONE")}, as
     *     {@link Json#write} writes them; may be null for the sources' own
     * @return the profile
     * @throws TeistroException naming {@code date} when it is null
     */
    public NumerologyProfile profile(String name, LocalDate date, Map<String, ?> rules) {
        if (date == null) {
            throw Reads.invalid("date is a LocalDate, such as LocalDate.of(1872, 1, 17)", "date");
        }
        Map<String, Object> request = new LinkedHashMap<>();
        request.put("name", name);
        Map<String, Object> day = new LinkedHashMap<>();
        day.put("year", date.getYear());
        day.put("month", date.getMonthValue());
        day.put("day", date.getDayOfMonth());
        request.put("date", day);
        String written = Reads.recordJson(rules, "rules", "Map.of(\"masters\", \"NONE\")");
        if (written != null) {
            request.put("rules", Json.read(written));
        }
        String json = Json.write(request);
        return NumerologyProfile.read(Json.read(context.locked((lib, raw) -> Calls.numerologyProfile(lib, raw, json))));
    }
}
