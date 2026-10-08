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
     * @param tag the locale's tag ({@code ne-Deva-NP})
     */
    public void setLocale(String tag) {
        context.locked((lib, raw) -> {
            Calls.intlSetLocale(lib, raw, tag);
            return null;
        });
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
     * Loads a locale pack's bytes into the engine.
     *
     * @param data the pack
     * @return what it loaded
     */
    public IntlLoaded loadPack(byte[] data) {
        return context.locked((lib, raw) -> Calls.intlLoadPack(lib, raw, data));
    }
}
