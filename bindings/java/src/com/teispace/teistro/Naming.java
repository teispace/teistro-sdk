package com.teispace.teistro;

import java.util.Map;

/** The naming answers' shared readers: a member by its bare key, and a name's syllable. */
final class Naming {
    private Naming() {}

    static Nakshatra nakshatra(Object raw) {
        return raw == null ? null : Reads.member(Nakshatra.byKey(Reads.string(raw)), "Nakshatra", raw);
    }

    static Rashi rashi(Object raw) {
        return Reads.member(Rashi.byKey(Reads.string(raw)), "Rashi", raw);
    }

    static NameVarga varga(Object raw) {
        return Reads.member(NameVarga.byKey(Reads.string(raw)), "NameVarga", raw);
    }

    static NameSyllable syllable(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        return new NameSyllable(Reads.integer(o, "cell"), nakshatra(Reads.field(o, "nakshatra")),
                Reads.integer(o, "quarter"), varga(Reads.field(o, "varga")));
    }
}
