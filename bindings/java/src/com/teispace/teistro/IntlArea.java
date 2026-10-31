package com.teispace.teistro;

import java.util.Map;

import com.teispace.teistro.blob.IntlRender;
import com.teispace.teistro.messages.Messages;

/** {@code sky.intl()}: the locale, its messages and the scripts they are in. */
public final class IntlArea {
    private final Context context;
    private final Messages messages;

    IntlArea(Context context) {
        this.context = context;
        this.messages = new Messages(new Messages.Renderer() {
            @Override
            public String render(String key, Map<String, Object> params) {
                return IntlArea.this.render(key, params).text();
            }

            @Override
            public Messages.EntityForms entity(String key) {
                return IntlArea.this.entity(key);
            }
        });
    }

    /**
     * The locale tag messages are rendered in.
     *
     * @return the tag
     */
    public String locale() {
        return context.locked(Calls::intlLocale);
    }

    /**
     * Renders messages in another locale from now on.
     *
     * @param tag the locale's tag ({@code ne-Deva-NP}), or a shorter one
     *     ({@code ne}), which RFC 4647 lookup takes to the loaded locale it
     *     means
     * @return the locale chosen
     */
    public String setLocale(String tag) {
        context.locked((lib, raw) -> {
            Calls.intlSetLocale(lib, raw, tag);
            return null;
        });
        return locale();
    }

    /**
     * The loaded locale a tag means, by RFC 4647 lookup, changing nothing.
     *
     * @param tag the tag ({@code ne})
     * @return the locale it means ({@code ne-Deva-NP})
     */
    public String lookupLocale(String tag) {
        return context.locked((lib, raw) -> Calls.intlLookupLocale(lib, raw, tag));
    }

    /**
     * A message rendered in the context's locale.
     *
     * <pre>{@code
     * String said = sky.intl().render("sdk.reason.grahaInBhava",
     *         Map.of("graha", Map.of("$entity", "graha.JUPITER"), "bhava", 7)).text();
     * }</pre>
     *
     * @param key the message's key
     * @param params its arguments, as {@link Json#write} writes them; an entity is
     *     {@code {"$entity": "graha.JUPITER"}}
     * @return the text, where it came from and what the renderer warned
     */
    public IntlRender render(String key, Map<String, ?> params) {
        return renderJson(key, Json.write(params == null ? Map.of() : params));
    }

    /**
     * A catalogued entity's forms in the current locale.
     *
     * @param key the entity's full key, {@code graha.SUN}
     * @return its forms
     */
    public Messages.EntityForms entity(String key) {
        return Messages.EntityForms.of(Json.read(context.locked((lib, raw) -> Calls.intlEntity(lib, raw, key))));
    }

    /**
     * The typed accessors: every message of the SDK, by its key, each
     * rendered in the context's locale.
     *
     * <pre>{@code
     * String said = sky.intl().messages().sdk().reason().grahaInBhava(7, Messages.GrahaKey.JUPITER);
     * }</pre>
     *
     * @return the accessors
     */
    public Messages messages() {
        return messages;
    }

    /**
     * A message rendered in the context's locale, its arguments given as JSON.
     *
     * @param key the message's key
     * @param paramsJson its arguments as a JSON object
     * @return the text, where it came from and what the renderer warned
     */
    public IntlRender renderJson(String key, String paramsJson) {
        return IntlRender.decode(context.locked((lib, raw) -> Calls.intlRender(lib, raw, key, paramsJson)));
    }

    /**
     * Whether the current locale carries a message.
     *
     * @param key the message's key
     * @return true when it does
     */
    public boolean has(String key) {
        return context.locked((lib, raw) -> Calls.intlHas(lib, raw, key)) != 0;
    }

    /**
     * Text from one script into another.
     *
     * @param text the text
     * @param source its script ({@code Deva})
     * @param into the script to write it in ({@code Latn})
     * @return the transliterated text
     */
    public String transliterate(String text, String source, String into) {
        return context.locked((lib, raw) -> Calls.intlTransliterate(lib, raw, text, source, into));
    }

    /**
     * Devanagari text in the Latin script.
     *
     * @param text the text
     * @return the transliterated text
     */
    public String transliterate(String text) {
        return transliterate(text, "Deva", "Latn");
    }

    /**
     * A number or an amount of money in words and in figures, by CLDR's
     * rule-based number formats ({@code 03-design/amounts-in-words.md}):
     * Nepali, Hindi and English with the Indian grouping, English with the
     * international one too. An amount's value is a count of minor units
     * or a decimal string, never a double.
     *
     * <pre>{@code
     * InWords words = sdk.intl().inWords(InWordsRequest.builder()
     *         .amount(WordsAmount.builder().currency("NPR").value(new WordsAmountValueMinor(123_456)).only(true).build())
     *         .locale("ne-Deva-NP").build());
     * String text = words.text(); // एक हजार दुई सय चौँतिस रुपैयाँ र छपन्न पैसा मात्र
     * }</pre>
     *
     * @param request the number or the amount, and how to write it
     * @return the words and the figures
     */
    public InWords inWords(com.teispace.teistro.request.InWordsRequest request) {
        if (request == null) {
            throw Reads.invalid("request is an InWordsRequest", "request");
        }
        String json = Json.write(request);
        return InWords.read(Json.read(context.locked((lib, raw) -> Calls.intlInWords(lib, raw, json))));
    }

    /**
     * A time of day as text and parts, through the locale's pattern and
     * parts of the day ({@code 03-design/intl-engine-and-packs.md}).
     *
     * @param request the time, and how to write it
     * @return the text and its parts
     */
    public CivilText time(com.teispace.teistro.request.TimeTextRequest request) {
        if (request == null) {
            throw Reads.invalid("request is a TimeTextRequest", "request");
        }
        String json = Json.write(request);
        return CivilText.read(Json.read(context.locked((lib, raw) -> Calls.intlTime(lib, raw, json))));
    }

    /**
     * A date as text and parts, in its own calendar, through the locale's
     * pattern.
     *
     * @param request the date, and how to write it
     * @return the text and its parts
     */
    public CivilText date(com.teispace.teistro.request.DateTextRequest request) {
        if (request == null) {
            throw Reads.invalid("request is a DateTextRequest", "request");
        }
        String json = Json.write(request);
        return CivilText.read(Json.read(context.locked((lib, raw) -> Calls.intlDate(lib, raw, json))));
    }

    /**
     * Every catalogue member's names in a locale, romanised forms too
     * ({@code 03-design/lexicon.md}): built once per context and locale.
     *
     * @param request the locale, and the kinds to list
     * @return the names
     */
    public Lexicon lexicon(com.teispace.teistro.request.LexiconRequest request) {
        if (request == null) {
            throw Reads.invalid("request is a LexiconRequest", "request");
        }
        String json = Json.write(request);
        return Lexicon.read(Json.read(context.locked((lib, raw) -> Calls.intlLexicon(lib, raw, json))));
    }

    /**
     * The catalogue members a text may name: each span with every member
     * it may name, never a guess.
     *
     * @param request the text, and the locale and kinds to search
     * @return the spans
     */
    public Lookup lookup(com.teispace.teistro.request.LookupRequest request) {
        if (request == null) {
            throw Reads.invalid("request is a LookupRequest", "request");
        }
        String json = Json.write(request);
        return Lookup.read(Json.read(context.locked((lib, raw) -> Calls.intlLookup(lib, raw, json))));
    }

    /**
     * Loads a locale pack's bytes into the engine.
     *
     * @param data the pack
     * @return what it loaded
     */
    public IntlLoaded loadPack(byte[] data) {
        return context.locked((lib, raw) -> Calls.intlLoadPack(lib, raw, data));
    }
}
