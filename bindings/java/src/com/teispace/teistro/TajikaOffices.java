package com.teispace.teistro;

import java.util.List;

/**
 * The offices one graha holds (Samjna Tantra vv. 36 to 46).
 *
 * @param graha Whose.
 * @param held Every clause of the gloss's definition it meets, in the definition's order:
 *     {@code OWN_SIGN}, {@code EXALTED}, {@code OWN_HUDDA}, {@code OWN_DREKKANA},
 *     {@code OWN_NAVAMSHA}, {@code DEBILITATED}, {@code ENEMY_SIGN}.
 * @param adhikara The office that counts: {@code UTTAMA}, {@code MADHYAMA}, {@code SAMA} or
 *     {@code ADHAMA}.
 */
public record TajikaOffices(
        Graha graha,
        List<String> held,
        String adhikara) {
    /** The value, its lists copied and unmodifiable. */
    public TajikaOffices {
        held = List.copyOf(held);
    }
}
