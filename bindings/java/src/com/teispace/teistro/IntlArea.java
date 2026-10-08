package com.teispace.teistro;

/** {@code sky.intl()}: the locale, its messages and the scripts they are in. */
public final class IntlArea {
    private final Context context;

    IntlArea(Context context) {
        this.context = context;
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
     * @param tag the locale's tag ({@code ne-Deva-NP})
     */
    public void setLocale(String tag) {
        context.locked((lib, raw) -> {
            Calls.intlSetLocale(lib, raw, tag);
            return null;
        });
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
     * Loads a locale pack's bytes into the engine.
     *
     * @param data the pack
     * @return what it loaded
     */
    public IntlLoaded loadPack(byte[] data) {
        return context.locked((lib, raw) -> Calls.intlLoadPack(lib, raw, data));
    }
}
