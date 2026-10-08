package com.teispace.teistro;

import java.util.List;

/**
 * Which of the seven are retrograde and which combust: what Tajika's yogas were judged on.
 *
 * @param retrograde The retrograde.
 * @param combust The combust.
 */
public record AnnualStatesRead(
        List<Graha> retrograde,
        List<Graha> combust) {
    /** The value, its lists copied and unmodifiable. */
    public AnnualStatesRead {
        retrograde = List.copyOf(retrograde);
        combust = List.copyOf(combust);
    }
}
