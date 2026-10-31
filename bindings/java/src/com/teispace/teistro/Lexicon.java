package com.teispace.teistro;

import java.util.List;
import java.util.Map;

/**
 * Every catalogue member's names in a locale, romanised forms too
 * ({@code 03-design/lexicon.md}), as {@code intl.lexicon} gives them.
 *
 * @param locale the loaded locale the request's tag meant
 * @param entries every member a locale in the chain names, in the catalogue's order
 */
public record Lexicon(String locale, List<Entry> entries) {

    /**
     * One form of a name.
     *
     * @param form {@code name}, {@code prose}, {@code short}, {@code iast}, any the locale adds, or
     *     {@code romanised}, derived
     * @param text its text
     * @param locale the locale that gave it: the one asked for, or a fallback
     */
    public record Form(String form, String text, String locale) {

        static List<Form> list(Map<?, ?> o, String key) {
            return Reads.array(o, key).stream().map(raw -> {
                Map<?, ?> f = Reads.object(raw);
                return new Form(Reads.string(f, "form"), Reads.string(f, "text"), Reads.string(f, "locale"));
            }).toList();
        }
    }

    /**
     * A member and its forms.
     *
     * @param key the catalogue key, {@code kind.MEMBER}
     * @param forms its forms, each text once, the locale's own first
     */
    public record Entry(String key, List<Form> forms) {}

    static Lexicon read(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        List<Entry> entries = Reads.array(o, "entries").stream().map(entry -> {
            Map<?, ?> e = Reads.object(entry);
            return new Entry(Reads.string(e, "key"), Form.list(e, "forms"));
        }).toList();
        return new Lexicon(Reads.string(o, "locale"), entries);
    }
}
