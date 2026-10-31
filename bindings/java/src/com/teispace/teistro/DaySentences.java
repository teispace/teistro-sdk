package com.teispace.teistro;

import java.util.List;
import java.util.Map;

/**
 * A day said in a locale ({@code 03-design/day-in-sentences.md}).
 *
 * <pre>{@code
 * DaySentences day = sdk.almanac().sentences(date, kathmandu, 20_700, "en-Latn");
 * String first = day.sentences().get(0).text();
 * }</pre>
 *
 * @param locale the locale the sentences are in
 * @param draft whether its templates still wait for a native reviewer (Q42)
 * @param sentences the sentences, in the order a printed day runs
 * @param withheld the facts a polar day holds elsewhere and so does not say here
 */
public record DaySentences(String locale, boolean draft, List<Sentence> sentences, List<String> withheld) {
    /** Copies the lists. */
    public DaySentences {
        sentences = List.copyOf(sentences);
        withheld = List.copyOf(withheld);
    }

    /**
     * One sentence.
     *
     * @param key the template, as {@code sdk.day.tithi}
     * @param params its slots, as {@code intl.render} takes them
     * @param text the sentence
     */
    public record Sentence(String key, Map<?, ?> params, String text) {
    }
}
