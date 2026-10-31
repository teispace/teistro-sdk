package com.teispace.teistro;

import java.util.List;
import java.util.Map;

/**
 * The spans of a text that may name a catalogue member, each with every
 * member it may name ({@code 03-design/lexicon.md}), as {@code intl.lookup}
 * gives them.
 *
 * @param matches the spans, in the text's order
 */
public record Lookup(List<Match> matches) {

    /**
     * A member a span may name.
     *
     * @param key the catalogue key
     * @param via the forms whose fold the span's is
     */
    public record Candidate(String key, List<Lexicon.Form> via) {}

    /**
     * One span of the text.
     *
     * @param start where it begins, counted in code points, not UTF-16 units
     * @param end where it ends, exclusive, in the same count
     * @param text the span's text
     * @param candidates every member it may name, in key order
     */
    public record Match(int start, int end, String text, List<Candidate> candidates) {}

    static Lookup read(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        List<Match> matches = Reads.array(o, "matches").stream().map(found -> {
            Map<?, ?> m = Reads.object(found);
            List<Candidate> candidates = Reads.array(m, "candidates").stream().map(candidate -> {
                Map<?, ?> c = Reads.object(candidate);
                return new Candidate(Reads.string(c, "key"), Lexicon.Form.list(c, "via"));
            }).toList();
            return new Match((int) Reads.number(m, "start"), (int) Reads.number(m, "end"), Reads.string(m, "text"),
                    candidates);
        }).toList();
        return new Lookup(matches);
    }
}
