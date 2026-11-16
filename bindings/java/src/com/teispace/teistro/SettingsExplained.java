package com.teispace.teistro;

import java.util.List;
import java.util.Map;

/**
 * Why each settings knob has its value ({@code settings.explain}): what
 * {@link Context#explainSettings} answers.
 *
 * <pre>{@code
 * SettingsExplained explained = ctx.explainSettings(List.of("frame", "day.sunrise"));
 * String source = explained.knobs().get(0).citation().reference();
 * }</pre>
 *
 * @param profile the profile the settings were resolved from
 * @param settingsHash their hash, as every result computed under them carries
 * @param knobs the knobs asked, in the settings document's order
 */
public record SettingsExplained(String profile, String settingsHash, List<Knob> knobs) {
    /**
     * A source cited for a knob's value.
     *
     * @param text what the source says
     * @param reference where it says it
     */
    public record Source(String text, String reference) {}

    /**
     * One knob explained.
     *
     * @param knob {@code group.knob}
     * @param value its value, as {@link Json#read} reads the canonical settings document's
     * @param layer {@code ROOT}, {@code PROFILE}, {@code CONTEXT}, {@code REQUEST} or {@code DOCUMENT}
     * @param profile the profile that set it, where a profile did; null otherwise
     * @param citation the root's or the profile's citation; null where a patch set it
     * @param moves {@code SKY}, {@code SUNRISE}, {@code RULE}, {@code TIME} or {@code FORM}
     * @param effect what it changes, in one line, in the context's locale
     */
    public record Knob(String knob, Object value, String layer, String profile, Source citation, String moves,
            String effect) {}

    static SettingsExplained read(String text) {
        Map<?, ?> value = Reads.object(Json.read(text));
        return new SettingsExplained(Reads.string(value, "profile"), Reads.string(value, "settingsHash"),
                Reads.each(value, "knobs", raw -> {
                    Map<?, ?> k = Reads.object(raw);
                    Object cited = Reads.field(k, "citation");
                    Source citation = null;
                    if (cited != null) {
                        Map<?, ?> c = Reads.object(cited);
                        citation = new Source(Reads.string(c, "text"), Reads.string(c, "reference"));
                    }
                    return new Knob(Reads.string(k, "knob"), Reads.field(k, "value"), Reads.string(k, "layer"),
                            Reads.optionalString(k, "profile"), citation, Reads.string(k, "moves"),
                            Reads.string(k, "effect"));
                }));
    }

    /** A settings query as its JSON; the library names an unknown knob ({@code knobs[0]}). */
    static String query(List<String> knobs) {
        return knobs == null ? "{}" : Json.write(Map.of("knobs", List.copyOf(knobs)));
    }
}
