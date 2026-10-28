package com.teispace.teistro;

import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.function.Function;

import com.teispace.teistro.request.Birth;
import com.teispace.teistro.request.Book;
import com.teispace.teistro.request.NamingRules;
import com.teispace.teistro.request.SuggestFilter;
import com.teispace.teistro.request.SyllablesOf;

/**
 * {@code sky.naming()}: the naming of a child (nāmakaraṇa,
 * {@code 03-design/namakarana.md}): the syllables a pada, a star or a sign
 * names a child by, names read into the śatapada cakra and checked against a
 * birth, the names a birth gives, and the names a consumer's own book
 * suggests. It reads no sky, and the SDK ships no names. Every refusal is
 * named under {@code naming}.
 *
 * <pre>{@code
 * Names names = sky.naming().of(new Birth(Nakshatra.KRITTIKA, 1));
 * }</pre>
 */
public final class NamingArea {
    private final Context context;

    /**
     * The naming area of a context.
     *
     * @param context the context every call goes through
     */
    NamingArea(Context context) {
        this.context = context;
    }

    private static <T> List<T> each(Object raw, Function<Object, T> read) {
        return Reads.array(raw).stream().map(read).toList();
    }

    /**
     * The syllables a pada, a star or a sign names a child by, in cakra
     * order, each with every initial that reaches it.
     *
     * @param of what is asked, such as {@code new SyllablesOfRashi(Rashi.CAPRICORN)}
     * @param rules the naming rules; may be null for the sources' own
     * @return the syllables
     */
    public List<PadaSyllable> syllables(SyllablesOf of, NamingRules rules) {
        Map<String, Object> request = new LinkedHashMap<>();
        request.put("of", of);
        String json = written(request, rules);
        return each(Json.read(context.locked((lib, raw) -> Calls.namingSyllables(lib, raw, json))), PadaSyllable::read);
    }

    /**
     * Names read into the cakra, each on its own; a name refused is its row's
     * {@code refused}, named {@code naming.names[i]}.
     *
     * @param names in Devanagari, or in the Latin scheme the rules declare
     * @param rules the naming rules; may be null for the sources' own
     * @return one row a name, in the request's order
     */
    public List<NameRead> read(List<String> names, NamingRules rules) {
        Map<String, Object> request = new LinkedHashMap<>();
        request.put("names", names);
        String json = written(request, rules);
        return each(Json.read(context.locked((lib, raw) -> Calls.namingRead(lib, raw, json))), NameRead::read);
    }

    /**
     * Names checked against a birth's pada, each on its own: facts, not a
     * score.
     *
     * @param names the names
     * @param birth the birth's nakshatra and pada
     * @param rules the naming rules; may be null for the sources' own
     * @return one row a name, in the request's order
     */
    public List<NameChecked> check(List<String> names, Birth birth, NamingRules rules) {
        Map<String, Object> request = new LinkedHashMap<>();
        request.put("names", names);
        request.put("birth", birth);
        String json = written(request, rules);
        return each(Json.read(context.locked((lib, raw) -> Calls.namingCheck(lib, raw, json))), NameChecked::read);
    }

    /**
     * The names a birth gives: its pada's syllable and its star's deity.
     *
     * @param birth the birth's nakshatra and pada
     * @param rules the naming rules; may be null for the sources' own
     * @return the names
     */
    public Names of(Birth birth, NamingRules rules) {
        Map<String, Object> request = new LinkedHashMap<>();
        request.put("birth", birth);
        String json = written(request, rules);
        return Names.read(Json.read(context.locked((lib, raw) -> Calls.namingOf(lib, raw, json))));
    }

    /**
     * Loads a name book into this context, indexed by the cakra's cell, and
     * answers its id; the same book loaded again is the same book.
     *
     * @param book the book
     * @return the book's id, its count and the names the cakra refused
     */
    public LoadedBook load(Book book) {
        Map<String, Object> request = new LinkedHashMap<>();
        request.put("book", book);
        String json = Json.write(request);
        return LoadedBook.read(Json.read(context.locked((lib, raw) -> Calls.namingLoad(lib, raw, json))));
    }

    /**
     * The names a loaded book suggests for a birth.
     *
     * @param book the id {@link #load} answered
     * @param birth the birth's nakshatra and pada
     * @param filter what a suggestion keeps; may be null for every name
     * @param rules the naming rules; may be null for the sources' own
     * @return the suggestions
     */
    public Suggestions suggest(String book, Birth birth, SuggestFilter filter, NamingRules rules) {
        return suggest("bookId", book, birth, filter, rules);
    }

    /**
     * The names a book given whole suggests for a birth, for this one call.
     *
     * @param book the book
     * @param birth the birth's nakshatra and pada
     * @param filter what a suggestion keeps; may be null for every name
     * @param rules the naming rules; may be null for the sources' own
     * @return the suggestions
     */
    public Suggestions suggest(Book book, Birth birth, SuggestFilter filter, NamingRules rules) {
        return suggest("book", book, birth, filter, rules);
    }

    private Suggestions suggest(String key, Object book, Birth birth, SuggestFilter filter, NamingRules rules) {
        Map<String, Object> request = new LinkedHashMap<>();
        request.put(key, book);
        request.put("birth", birth);
        if (filter != null) {
            request.put("filter", filter);
        }
        String json = written(request, rules);
        return Suggestions.read(Json.read(context.locked((lib, raw) -> Calls.namingSuggest(lib, raw, json))));
    }

    private static String written(Map<String, Object> request, NamingRules rules) {
        if (rules != null) {
            request.put("rules", rules);
        }
        return Json.write(request);
    }
}
