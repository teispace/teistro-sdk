package com.teispace.teistro;

import java.util.List;
import java.util.Map;

/**
 * The settings knobs on which two contexts differ ({@code settings.diff}):
 * what {@link Context#diffSettings} answers.
 *
 * <pre>{@code
 * SettingsDiff diff = committee.diffSettings(drik);
 * List<String> knobs = diff.knobs().stream().map(SettingsDiff.Knob::knob).toList();
 * }</pre>
 *
 * @param profile the first context's profile
 * @param otherProfile the other's
 * @param settingsHash the first context's settings hash
 * @param otherSettingsHash the other's
 * @param knobs each knob asked whose values differ, in the settings document's order; none when the two
 *     hash alike
 */
public record SettingsDiff(String profile, String otherProfile, String settingsHash, String otherSettingsHash,
        List<Knob> knobs) {
    /**
     * One knob on which the two differ.
     *
     * @param knob {@code group.knob}
     * @param value its value under the first context
     * @param otherValue its value under the other
     * @param moves {@code SKY}, {@code SUNRISE}, {@code RULE}, {@code TIME} or {@code FORM}
     * @param effect what it changes, in one line, in the first context's locale
     */
    public record Knob(String knob, Object value, Object otherValue, String moves, String effect) {}

    static SettingsDiff read(String text) {
        Map<?, ?> value = Reads.object(Json.read(text));
        return new SettingsDiff(Reads.string(value, "profile"), Reads.string(value, "otherProfile"),
                Reads.string(value, "settingsHash"), Reads.string(value, "otherSettingsHash"),
                Reads.each(value, "knobs", raw -> {
                    Map<?, ?> k = Reads.object(raw);
                    return new Knob(Reads.string(k, "knob"), Reads.field(k, "value"), Reads.field(k, "otherValue"),
                            Reads.string(k, "moves"), Reads.string(k, "effect"));
                }));
    }
}
