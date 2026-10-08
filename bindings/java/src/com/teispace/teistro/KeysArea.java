package com.teispace.teistro;

/** {@code sky.keys()}: the catalogue's keys and their packed ids. */
public final class KeysArea {
    private final Context context;

    KeysArea(Context context) {
        this.context = context;
    }

    /**
     * The packed id a key stands for: the kind in the high half, the member
     * in the low.
     *
     * @param key the key, full ({@code graha.SUN}) or bare where it is unambiguous
     * @return the packed id
     * @throws TeistroException with {@link Status#UNSUPPORTED} for an unknown key, with a suggestion
     */
    public long id(String key) {
        return context.locked((lib, raw) -> Calls.keyParse(lib, raw, key));
    }

    /**
     * The key a packed id stands for.
     *
     * @param id the packed id
     * @return the full key
     */
    public String name(long id) {
        return context.locked((lib, raw) -> Calls.keyName(lib, raw, id));
    }
}
