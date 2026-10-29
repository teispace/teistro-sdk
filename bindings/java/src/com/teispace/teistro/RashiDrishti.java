package com.teispace.teistro;

import java.util.List;

/**
 * One body's Jaimini rashi drishti.
 *
 * @param graha The body whose sign looks.
 * @param signs The three signs its sign aspects, in the zodiac's order from Aries; each aspects it
 *     back.
 */
public record RashiDrishti(Graha graha, List<Rashi> signs) {
    /** The signs, held unmodifiable. */
    public RashiDrishti {
        signs = List.copyOf(signs);
    }
}
