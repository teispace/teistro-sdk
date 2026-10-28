package com.teispace.teistro;

import java.util.Map;

/**
 * The names a birth gives, of <i>Dharmasindhu</i>'s four that the SDK
 * computes (C773).
 *
 * @param syllable the pada's syllable: the default, the avakahada reading
 * @param deity the star's deity, whose name the Śāṅkhāyanas give
 */
public record Names(PadaSyllable syllable, Deity deity) {
    static Names read(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        Object deity = Reads.field(o, "deity");
        return new Names(PadaSyllable.read(Reads.field(o, "syllable")),
                Reads.member(Deity.byKey(Reads.string(deity)), "Deity", deity));
    }
}
